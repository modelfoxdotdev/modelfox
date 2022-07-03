use super::{Table, TableColumn, TableColumnType};
use anyhow::Result;
use modelfox_progress_counter::ProgressCounter;
use modelfox_zip::zip;
// NOTE - this import is actually used, false positive with the lint.
#[allow(unused_imports)]
use num::ToPrimitive;
use std::{
	collections::{BTreeMap, BTreeSet},
	path::Path,
};

#[derive(Clone)]
pub struct FromCsvOptions<'a> {
	pub column_types: Option<BTreeMap<String, TableColumnType>>,
	pub infer_options: InferOptions,
	pub invalid_values: &'a [&'a str],
}

impl<'a> Default for FromCsvOptions<'a> {
	fn default() -> FromCsvOptions<'a> {
		FromCsvOptions {
			column_types: None,
			infer_options: InferOptions::default(),
			invalid_values: DEFAULT_INVALID_VALUES,
		}
	}
}

#[derive(Clone, Debug)]
pub struct InferOptions {
	pub enum_max_unique_values: usize,
}

impl Default for InferOptions {
	fn default() -> InferOptions {
		InferOptions {
			enum_max_unique_values: 100,
		}
	}
}

/// These values are the default values that are considered invalid.
const DEFAULT_INVALID_VALUES: &[&str] = &[
	"", "+Inf", "+inf", "-Inf", "-NaN", "-inf", "-nan", "?", "N/A", "NA", "NULL", "NaN", "n/a",
	"nan", "null",
];

#[derive(Clone, Debug)]
pub enum ProgressEvent {
	InferStarted(ProgressCounter),
	InferDone,
	LoadStarted(ProgressCounter),
	LoadDone,
}

impl Table {
	/// # Errors
	///
	/// Returns an error if unable to load CSV from reader.
	pub fn from_bytes(
		bytes: &[u8],
		options: FromCsvOptions,
		handle_progress_event: &mut impl FnMut(ProgressEvent),
	) -> Result<Table> {
		let len = bytes.len();
		Table::from_csv(
			&mut csv::Reader::from_reader(std::io::Cursor::new(bytes)),
			len as u64,
			options,
			handle_progress_event,
		)
	}

	/// # Errors
	///
	/// Returns an error if unable to load CSV from reader.
	pub fn from_path(
		path: &Path,
		options: FromCsvOptions,
		handle_progress_event: &mut impl FnMut(ProgressEvent),
	) -> Result<Table> {
		let len = std::fs::metadata(path)?.len();
		Table::from_csv(
			&mut csv::Reader::from_path(path)?,
			len,
			options,
			handle_progress_event,
		)
	}

	#[allow(clippy::too_many_lines)]
	#[allow(clippy::missing_errors_doc)]
	#[allow(clippy::missing_panics_doc)]
	pub fn from_csv<R>(
		reader: &mut csv::Reader<R>,
		len: u64,
		options: FromCsvOptions,
		handle_progress_event: &mut impl FnMut(ProgressEvent),
	) -> Result<Table>
	where
		R: std::io::Read + std::io::Seek,
	{
		#[derive(Clone, Debug)]
		enum ColumnTypeOrInferStats<'a> {
			ColumnType(TableColumnType),
			InferStats(InferStats<'a>),
		}
		let column_names: Vec<String> = reader
			.headers()?
			.into_iter()
			.map(std::borrow::ToOwned::to_owned)
			.collect();
		let n_columns = column_names.len();
		let start_position = reader.position().clone();
		let infer_options = &options.infer_options;
		let mut n_rows = None;

		// Retrieve any column types present in the options.
		let mut column_types: Vec<ColumnTypeOrInferStats> =
			if let Some(column_types) = options.column_types {
				column_names
					.iter()
					.map(|column_name| {
						column_types.get(column_name).map_or_else(
							|| ColumnTypeOrInferStats::InferStats(InferStats::new(infer_options)),
							|column_type| ColumnTypeOrInferStats::ColumnType(column_type.clone()),
						)
					})
					.collect()
			} else {
				vec![
					ColumnTypeOrInferStats::InferStats(InferStats::new(&options.infer_options));
					n_columns
				]
			};

		// Passing over the csv to infer column types is only necessary if one or more columns did not have its type specified.
		let needs_infer =
			column_types.iter().any(
				|column_type_or_infer_stats| match column_type_or_infer_stats {
					ColumnTypeOrInferStats::ColumnType(_) => false,
					ColumnTypeOrInferStats::InferStats(_) => true,
				},
			);

		// If the infer pass is necessary, pass over the dataset and infer the types for those columns whose types were not specified.
		let column_types: Vec<TableColumnType> = if needs_infer {
			let mut infer_stats: Vec<(usize, &mut InferStats)> = column_types
				.iter_mut()
				.enumerate()
				.filter_map(
					|(index, column_type_or_infer_stats)| match column_type_or_infer_stats {
						ColumnTypeOrInferStats::ColumnType(_) => None,
						ColumnTypeOrInferStats::InferStats(infer_stats) => {
							Some((index, infer_stats))
						}
					},
				)
				.collect();
			// Iterate over each record in the csv file and update the infer stats for the columns that need to be inferred.
			let mut record = csv::StringRecord::new();
			let mut n_records_read = 0;
			let progress_counter = ProgressCounter::new(len);
			handle_progress_event(ProgressEvent::InferStarted(progress_counter.clone()));
			while reader.read_record(&mut record)? {
				progress_counter.set(record.position().unwrap().byte());
				for (index, infer_stats) in &mut infer_stats {
					let value = record.get(*index).unwrap();
					infer_stats.update(value);
				}
				n_records_read += 1;
			}
			handle_progress_event(ProgressEvent::InferDone);
			n_rows = Some(n_records_read);
			let column_types = column_types
				.into_iter()
				.map(
					|column_type_or_infer_stats| match column_type_or_infer_stats {
						ColumnTypeOrInferStats::ColumnType(column_type) => column_type,
						ColumnTypeOrInferStats::InferStats(infer_stats) => infer_stats.finalize(),
					},
				)
				.collect();
			// After inference, return back to the beginning of the csv to load the values.
			reader.seek(start_position)?;
			column_types
		} else {
			column_types
				.into_iter()
				.map(
					|column_type_or_infer_stats| match column_type_or_infer_stats {
						ColumnTypeOrInferStats::ColumnType(column_type) => column_type,
						ColumnTypeOrInferStats::InferStats(_) => unreachable!(),
					},
				)
				.collect()
		};

		// Create the table.
		let column_names = column_names.into_iter().map(Some).collect();
		let mut table = Table::new(column_names, column_types);
		// If an inference pass was done, reserve storage for the values because we know how many rows are in the csv.
		if let Some(n_rows) = n_rows {
			for column in &mut table.columns {
				match column {
					TableColumn::Unknown(_) => {}
					TableColumn::Number(column) => column.data.reserve_exact(n_rows),
					TableColumn::Enum(column) => column.data.reserve_exact(n_rows),
					TableColumn::Text(column) => column.data.reserve_exact(n_rows),
				}
			}
		}
		// Read each csv record and insert the values into the columns of the table.
		let mut record = csv::ByteRecord::new();
		let progress_counter = ProgressCounter::new(len);
		handle_progress_event(ProgressEvent::LoadStarted(progress_counter.clone()));
		while reader.read_byte_record(&mut record)? {
			progress_counter.set(record.position().unwrap().byte());
			for (column, value) in zip!(table.columns.iter_mut(), record.iter()) {
				match column {
					TableColumn::Unknown(column) => {
						column.len += 1;
					}
					TableColumn::Number(column) => {
						let value = match fast_float::parse::<f32, &[u8]>(value) {
							Ok(value) if value.is_finite() => value,
							_ => std::f32::NAN,
						};
						column.data.push(value);
					}
					TableColumn::Enum(column) => {
						let value = std::str::from_utf8(value)
							.ok()
							.and_then(|value| column.value_for_variant(value));
						column.data.push(value);
					}
					TableColumn::Text(column) => {
						column.data.push(std::str::from_utf8(value)?.to_owned());
					}
				}
			}
		}
		handle_progress_event(ProgressEvent::LoadDone);
		Ok(table)
	}
}

#[derive(Clone, Debug)]
pub struct InferStats<'a> {
	infer_options: &'a InferOptions,
	column_type: InferColumnType,
	unique_values: Option<BTreeSet<String>>,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum InferColumnType {
	Unknown,
	Number,
	Enum,
	Text,
}

impl<'a> InferStats<'a> {
	pub fn new(infer_options: &'a InferOptions) -> InferStats<'a> {
		InferStats {
			infer_options,
			column_type: InferColumnType::Unknown,
			unique_values: Some(BTreeSet::new()),
		}
	}

	pub fn update(&mut self, value: &str) {
		if DEFAULT_INVALID_VALUES.contains(&value) {
			return;
		}
		if let Some(unique_values) = self.unique_values.as_mut() {
			if !unique_values.contains(value) {
				unique_values.insert(value.to_owned());
			}
			if unique_values.len() > self.infer_options.enum_max_unique_values {
				self.unique_values = None;
			}
		}
		match self.column_type {
			InferColumnType::Unknown | InferColumnType::Number => {
				if fast_float::parse::<f32, &str>(value)
					.map(f32::is_finite)
					.unwrap_or(false)
				{
					self.column_type = InferColumnType::Number;
				} else if self.unique_values.is_some() {
					self.column_type = InferColumnType::Enum;
				} else {
					self.column_type = InferColumnType::Text;
				}
			}
			InferColumnType::Enum => {
				if self.unique_values.is_none() {
					self.column_type = InferColumnType::Text;
				}
			}
			InferColumnType::Text => {}
		}
	}

	pub fn finalize(self) -> TableColumnType {
		match self.column_type {
			InferColumnType::Unknown => TableColumnType::Unknown,
			InferColumnType::Number => {
				// If all the values in a number column are zero or one then make this an enum column instead.
				if let Some(unique_values) = self.unique_values {
					if unique_values.len() == 2 {
						let mut values = unique_values.iter();
						if values.next().map(std::string::String::as_str) == Some("0")
							&& values.next().map(std::string::String::as_str) == Some("1")
						{
							return TableColumnType::Enum {
								variants: unique_values.into_iter().collect(),
							};
						}
					}
				}
				TableColumnType::Number
			}
			InferColumnType::Enum => TableColumnType::Enum {
				variants: self.unique_values.unwrap().into_iter().collect(),
			},
			InferColumnType::Text => TableColumnType::Text,
		}
	}
}

#[test]
fn test_infer() {
	let csv = r#"number,enum,text
1,test,hello
2,test,world
"#;
	let table = Table::from_csv(
		&mut csv::Reader::from_reader(std::io::Cursor::new(csv)),
		csv.len().to_u64().unwrap(),
		FromCsvOptions {
			column_types: None,
			infer_options: InferOptions {
				enum_max_unique_values: 1,
			},
			..Default::default()
		},
		&mut |_| {},
	)
	.unwrap();
	insta::assert_debug_snapshot!(table, @r###"
 Table {
     columns: [
         Number(
             NumberTableColumn {
                 name: Some(
                     "number",
                 ),
                 data: [
                     1.0,
                     2.0,
                 ],
             },
         ),
         Enum(
             EnumTableColumn {
                 name: Some(
                     "enum",
                 ),
                 variants: [
                     "test",
                 ],
                 data: [
                     Some(
                         1,
                     ),
                     Some(
                         1,
                     ),
                 ],
                 variants_map: {
                     "test": 1,
                 },
             },
         ),
         Text(
             TextTableColumn {
                 name: Some(
                     "text",
                 ),
                 data: [
                     "hello",
                     "world",
                 ],
             },
         ),
     ],
 }
 "###);
}

#[test]
fn test_column_types() {
	let csv = r#"number,text,enum
1,test,hello
2,test,world
"#;
	let mut column_types = BTreeMap::new();
	column_types.insert("text".to_owned(), TableColumnType::Text);
	column_types.insert(
		"enum".to_owned(),
		TableColumnType::Enum {
			variants: vec!["hello".to_owned(), "world".to_owned()],
		},
	);
	let table = Table::from_csv(
		&mut csv::Reader::from_reader(std::io::Cursor::new(csv)),
		csv.len().to_u64().unwrap(),
		FromCsvOptions {
			column_types: Some(column_types),
			infer_options: InferOptions {
				enum_max_unique_values: 2,
			},
			..Default::default()
		},
		&mut |_| {},
	)
	.unwrap();
	insta::assert_debug_snapshot!(table, @r###"
 Table {
     columns: [
         Number(
             NumberTableColumn {
                 name: Some(
                     "number",
                 ),
                 data: [
                     1.0,
                     2.0,
                 ],
             },
         ),
         Text(
             TextTableColumn {
                 name: Some(
                     "text",
                 ),
                 data: [
                     "test",
                     "test",
                 ],
             },
         ),
         Enum(
             EnumTableColumn {
                 name: Some(
                     "enum",
                 ),
                 variants: [
                     "hello",
                     "world",
                 ],
                 data: [
                     Some(
                         1,
                     ),
                     Some(
                         2,
                     ),
                 ],
                 variants_map: {
                     "hello": 1,
                     "world": 2,
                 },
             },
         ),
     ],
 }
 "###);
}
