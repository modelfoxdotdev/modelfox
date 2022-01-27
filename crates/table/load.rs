use super::*;
use anyhow::Result;
use arrow2::{
	array::{ArrayRef, BooleanArray, Utf8Array},
	types::NativeType,
};
use modelfox_progress_counter::ProgressCounter;
use modelfox_zip::zip;
use std::{
	collections::{BTreeMap, BTreeSet},
	path::Path,
};

#[derive(Clone)]
pub struct Options<'a> {
	pub column_types: Option<BTreeMap<String, TableColumnType>>,
	pub infer_options: InferOptions,
	pub invalid_values: &'a [&'a str],
}

impl<'a> Default for Options<'a> {
	fn default() -> Options<'a> {
		Options {
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
pub enum LoadProgressEvent {
	InferStarted(ProgressCounter),
	InferDone,
	LoadStarted(ProgressCounter),
	LoadDone,
}

impl Table {
	pub fn from_bytes(
		bytes: &[u8],
		options: Options,
		handle_progress_event: &mut impl FnMut(LoadProgressEvent),
	) -> Result<Table> {
		let len = bytes.len();
		Table::from_csv(
			&mut csv::Reader::from_reader(std::io::Cursor::new(bytes)),
			len as u64,
			options,
			handle_progress_event,
		)
	}

	pub fn from_path(
		path: &Path,
		options: Options,
		handle_progress_event: &mut impl FnMut(LoadProgressEvent),
	) -> Result<Table> {
		let len = std::fs::metadata(path)?.len();
		Table::from_csv(
			&mut csv::Reader::from_path(path)?,
			len,
			options,
			handle_progress_event,
		)
	}

	pub fn from_csv<R>(
		reader: &mut csv::Reader<R>,
		len: u64,
		options: Options,
		handle_progress_event: &mut impl FnMut(LoadProgressEvent),
	) -> Result<Table>
	where
		R: std::io::Read + std::io::Seek,
	{
		let column_names: Vec<String> = reader
			.headers()?
			.into_iter()
			.map(|column_name| column_name.to_owned())
			.collect();
		let start_position = reader.position().clone();
		let mut n_rows = None;

		// Retrieve any column types present in the options.
		let mut column_types: Vec<ColumnTypeOrInferStats> =
			get_column_types(column_names.as_slice(), &options);

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
			handle_progress_event(LoadProgressEvent::InferStarted(progress_counter.clone()));
			while reader.read_record(&mut record)? {
				progress_counter.set(record.position().unwrap().byte());
				for (index, infer_stats) in infer_stats.iter_mut() {
					let value = record.get(*index).unwrap();
					infer_stats.update(value);
				}
				n_records_read += 1;
			}
			handle_progress_event(LoadProgressEvent::InferDone);
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
						_ => unreachable!(),
					},
				)
				.collect()
		};

		// Create the table.
		let mut table = create_table(column_names, column_types, n_rows);

		// Read each csv record and insert the values into the columns of the table.
		let mut record = csv::ByteRecord::new();
		let progress_counter = ProgressCounter::new(len);
		handle_progress_event(LoadProgressEvent::LoadStarted(progress_counter.clone()));
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
						column.data.push(std::str::from_utf8(value)?.to_owned())
					}
				}
			}
		}
		handle_progress_event(LoadProgressEvent::LoadDone);
		Ok(table)
	}

	pub fn from_arrow_arrays(
		column_names: Vec<String>,
		arrow_arrays: Vec<ArrayRef>,
		options: Options,
		handle_progress_event: &mut impl FnMut(LoadProgressEvent),
	) -> Result<Table> {
		let n_rows = arrow_arrays[0].len();
		let n_columns = column_names.len();

		// Retrieve any column types present in the options.
		let column_types: Vec<ColumnTypeOrInferStats> =
			get_column_types(column_names.as_slice(), &options);

		// Passing over the arrays to infer column types is only necessary if one or more columns did not have its type specified.
		let needs_infer =
			column_types.iter().any(
				|column_type_or_infer_stats| match column_type_or_infer_stats {
					ColumnTypeOrInferStats::ColumnType(_) => false,
					ColumnTypeOrInferStats::InferStats(_) => true,
				},
			);

		// If the infer pass is necessary, pass over the dataset and infer the types for those columns whose types were not specified.
		// TODO: add progress for the infer step
		// 				let progress_counter =
		// 					ProgressCounter::new((n_rows * infer_stats.len()).to_u64().unwrap());
		// 				handle_progress_event(LoadProgressEvent::InferStarted(progress_counter.clone()));
		let column_types: Vec<TableColumnType> = if needs_infer {
			let column_types = column_types
				.into_iter()
				.zip(arrow_arrays.iter())
				.map(|(column_type_or_infer_stats, arrow_array)| {
					match column_type_or_infer_stats {
						ColumnTypeOrInferStats::ColumnType(column_type) => column_type.to_owned(),
						ColumnTypeOrInferStats::InferStats(mut infer_stats) => {
							// Iterate over each array and update the infer stats for the columns that need to be inferred.
							let physical_ty = arrow_array.data_type().to_physical_type();
							match physical_ty {
								arrow2::datatypes::PhysicalType::Primitive(primitive) => {
									match primitive {
										arrow2::datatypes::PrimitiveType::Int8 => {
											TableColumnType::Number
										}
										arrow2::datatypes::PrimitiveType::Int16 => {
											TableColumnType::Number
										}
										arrow2::datatypes::PrimitiveType::Int32 => {
											TableColumnType::Number
										}
										arrow2::datatypes::PrimitiveType::Int64 => {
											TableColumnType::Number
										}
										arrow2::datatypes::PrimitiveType::UInt8 => {
											TableColumnType::Number
										}
										arrow2::datatypes::PrimitiveType::UInt16 => {
											TableColumnType::Number
										}
										arrow2::datatypes::PrimitiveType::UInt32 => {
											TableColumnType::Number
										}
										arrow2::datatypes::PrimitiveType::UInt64 => {
											TableColumnType::Number
										}
										arrow2::datatypes::PrimitiveType::Float64 => {
											TableColumnType::Number
										}
										arrow2::datatypes::PrimitiveType::Float32 => {
											TableColumnType::Number
										}
										primitive_type => {
											unimplemented!("{:?}", primitive_type)
										}
									}
								}
								arrow2::datatypes::PhysicalType::Boolean => TableColumnType::Enum {
									variants: vec!["true".to_owned(), "false".to_owned()],
								},
								arrow2::datatypes::PhysicalType::Utf8 => {
									let array = arrow_array
										.as_any()
										.downcast_ref::<Utf8Array<i32>>()
										.unwrap();
									for value in array.iter() {
										infer_stats.update(value.map_or("None", |v| v));
									}
									infer_stats.finalize()
								}
								_ => unimplemented!(),
							}
						}
					}
				})
				.collect::<Vec<_>>();
			handle_progress_event(LoadProgressEvent::InferDone);
			column_types
		} else {
			column_types
				.into_iter()
				.map(
					|column_type_or_infer_stats| match column_type_or_infer_stats {
						ColumnTypeOrInferStats::ColumnType(column_type) => column_type,
						_ => unreachable!(),
					},
				)
				.collect()
		};

		// Create the table.
		let mut table = create_table(column_names, column_types, Some(n_rows));

		// Read each array and insert the values into the columns of the table.
		let progress_counter = ProgressCounter::new((n_rows * n_columns).to_u64().unwrap());
		handle_progress_event(LoadProgressEvent::LoadStarted(progress_counter.clone()));
		for (column, array) in zip!(table.columns.iter_mut(), arrow_arrays) {
			let physical_ty = array.data_type().to_physical_type();
			match column {
				TableColumn::Unknown(column) => {
					column.len += array.len();
					progress_counter.inc(array.len().to_u64().unwrap());
				}
				TableColumn::Number(column) => {
					match physical_ty {
						arrow2::datatypes::PhysicalType::Primitive(primitive) => match primitive {
							arrow2::datatypes::PrimitiveType::Int8 => {
								copy_primitive_data::<i8>(array, column, &progress_counter);
							}
							arrow2::datatypes::PrimitiveType::Int16 => {
								copy_primitive_data::<i16>(array, column, &progress_counter);
							}
							arrow2::datatypes::PrimitiveType::Int32 => {
								copy_primitive_data::<i32>(array, column, &progress_counter);
							}
							arrow2::datatypes::PrimitiveType::Int64 => {
								copy_primitive_data::<i64>(array, column, &progress_counter);
							}
							arrow2::datatypes::PrimitiveType::UInt8 => {
								copy_primitive_data::<u8>(array, column, &progress_counter);
							}
							arrow2::datatypes::PrimitiveType::UInt16 => {
								copy_primitive_data::<u16>(array, column, &progress_counter);
							}
							arrow2::datatypes::PrimitiveType::UInt32 => {
								copy_primitive_data::<u32>(array, column, &progress_counter);
							}
							arrow2::datatypes::PrimitiveType::UInt64 => {
								copy_primitive_data::<u64>(array, column, &progress_counter);
							}
							arrow2::datatypes::PrimitiveType::Float32 => {
								copy_primitive_data::<f32>(array, column, &progress_counter);
							}
							arrow2::datatypes::PrimitiveType::Float64 => {
								copy_primitive_data::<f64>(array, column, &progress_counter);
							}
							_ => unimplemented!(),
						},
						_ => unimplemented!(),
					};
				}
				TableColumn::Enum(column) => {
					match physical_ty {
						arrow2::datatypes::PhysicalType::Utf8 => {
							if let Some(array) = array.as_any().downcast_ref::<Utf8Array<i32>>() {
								for value in array.iter() {
									let value =
										value.and_then(|value| column.value_for_variant(value));
									column.data.push(value);
									progress_counter.inc(1);
								}
							} else {
								let array =
									array.as_any().downcast_ref::<Utf8Array<i64>>().unwrap();
								for value in array.iter() {
									let value =
										value.and_then(|value| column.value_for_variant(value));
									column.data.push(value);
									progress_counter.inc(1);
								}
							}
						}
						arrow2::datatypes::PhysicalType::Boolean => {
							let array = array.as_any().downcast_ref::<BooleanArray>().unwrap();
							for value in array.iter() {
								let value = value.and_then(|value| {
									column.value_for_variant(match value {
										true => "true",
										false => "false",
									})
								});
								column.data.push(value);
								progress_counter.inc(1);
							}
						}
						_ => unimplemented!(),
					};
				}
				TableColumn::Text(column) => match physical_ty {
					arrow2::datatypes::PhysicalType::Utf8 => {
						if let Some(array) = array.as_any().downcast_ref::<Utf8Array<i32>>() {
							for value in array.iter() {
								column
									.data
									.push(value.map_or("".to_owned(), ToOwned::to_owned));
								progress_counter.inc(1);
							}
						} else {
							let array = array.as_any().downcast_ref::<Utf8Array<i64>>().unwrap();
							for value in array.iter() {
								column
									.data
									.push(value.map_or("".to_owned(), ToOwned::to_owned));
								progress_counter.inc(1);
							}
						}
					}
					_ => unimplemented!(),
				},
			}
		}
		handle_progress_event(LoadProgressEvent::LoadDone);
		Ok(table)
	}
}

fn copy_primitive_data<T>(
	array: ArrayRef,
	column: &mut NumberTableColumn,
	progress_counter: &ProgressCounter,
) where
	T: NativeType + num::ToPrimitive,
{
	let array = array
		.as_any()
		.downcast_ref::<arrow2::array::PrimitiveArray<T>>()
		.unwrap();
	for value in array.iter() {
		let value = match value.and_then(|value| value.to_f32()) {
			Some(value) => value,
			_ => std::f32::NAN,
		};
		column.data.push(value);
		progress_counter.inc(1);
	}
}

fn create_table(
	column_names: Vec<String>,
	column_types: Vec<TableColumnType>,
	n_rows: Option<usize>,
) -> Table {
	let column_names = column_names.into_iter().map(Some).collect();
	let mut table = Table::new(column_names, column_types);
	// If an inference pass was done, reserve storage for the values because we know how many rows are in the csv.
	if let Some(n_rows) = n_rows {
		for column in table.columns.iter_mut() {
			match column {
				TableColumn::Unknown(_) => {}
				TableColumn::Number(column) => column.data.reserve_exact(n_rows),
				TableColumn::Enum(column) => column.data.reserve_exact(n_rows),
				TableColumn::Text(column) => column.data.reserve_exact(n_rows),
			}
		}
	}
	table
}

#[derive(Clone, Debug)]
enum ColumnTypeOrInferStats<'a> {
	ColumnType(TableColumnType),
	InferStats(InferStats<'a>),
}

fn get_column_types<'a>(
	column_names: &[String],
	options: &'a Options<'a>,
) -> Vec<ColumnTypeOrInferStats<'a>> {
	let n_columns = column_names.len();
	if let Some(column_types) = &options.column_types {
		column_names
			.iter()
			.map(|column_name| {
				column_types
					.get(column_name)
					.map(|column_type| ColumnTypeOrInferStats::ColumnType(column_type.clone()))
					.unwrap_or_else(|| {
						ColumnTypeOrInferStats::InferStats(InferStats::new(&options.infer_options))
					})
			})
			.collect()
	} else {
		vec![ColumnTypeOrInferStats::InferStats(InferStats::new(&options.infer_options)); n_columns]
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
					.map(|v| v.is_finite())
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
			_ => {}
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
						if values.next().map(|s| s.as_str()) == Some("0")
							&& values.next().map(|s| s.as_str()) == Some("1")
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
		Options {
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
		Options {
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
