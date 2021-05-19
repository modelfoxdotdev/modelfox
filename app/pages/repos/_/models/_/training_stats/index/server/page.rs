use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_app_ui::{
	column_type::ColumnType,
	metrics_row::MetricsRow,
	tokens::{EnumColumnToken, NumberColumnToken, TextColumnToken},
};
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct Page {
	pub model_layout_info: ModelLayoutInfo,
	pub target_column_stats_table: TargetColumnStatsTable,
	pub column_stats_table: ColumnStatsTable,
	pub column_count: usize,
	pub row_count: usize,
}

pub struct ColumnStatsTableRow {
	pub invalid_count: Option<usize>,
	pub max: Option<f32>,
	pub mean: Option<f32>,
	pub min: Option<f32>,
	pub name: String,
	pub std: Option<f32>,
	pub column_type: ColumnType,
	pub unique_count: Option<usize>,
	pub variance: Option<f32>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new().child("Training Stats"))
						.child(
							ui::S2::new()
								.child(ui::H2::new().child("Target Column"))
								.child(self.target_column_stats_table),
						)
						.child(
							ui::S2::new()
								.child(ui::H2::new().child("Columns"))
								.child(
									MetricsRow::new()
										.child(ui::NumberCard::new(
											"Rows".to_owned(),
											self.row_count.to_string(),
										))
										.child(ui::NumberCard::new(
											"Columns".to_owned(),
											self.column_count.to_string(),
										)),
								)
								.child(self.column_stats_table),
						),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct TargetColumnStatsTable {
	pub target_column_stats_table_row: ColumnStatsTableRow,
}

impl Component for TargetColumnStatsTable {
	fn into_node(self) -> Node {
		let number_columns_table_header_cells =
			if self.target_column_stats_table_row.column_type == ColumnType::Number {
				Some(
					fragment()
						.child(ui::TableHeaderCell::new().child("Min"))
						.child(ui::TableHeaderCell::new().child("Max"))
						.child(ui::TableHeaderCell::new().child("Mean"))
						.child(ui::TableHeaderCell::new().child("Std")),
				)
			} else {
				None
			};
		let number_columns_table_cells = if self.target_column_stats_table_row.column_type
			== ColumnType::Number
		{
			Some(
				fragment()
					.child(
						ui::TableCell::new()
							.child(self.target_column_stats_table_row.min.unwrap().to_string()),
					)
					.child(
						ui::TableCell::new()
							.child(self.target_column_stats_table_row.max.unwrap().to_string()),
					)
					.child(
						ui::TableCell::new()
							.child(self.target_column_stats_table_row.mean.unwrap().to_string()),
					)
					.child(
						ui::TableCell::new()
							.child(self.target_column_stats_table_row.std.unwrap().to_string()),
					),
			)
		} else {
			None
		};
		let header = ui::TableRow::new()
			.child(ui::TableHeaderCell::new().child("Column"))
			.child(ui::TableHeaderCell::new().child("Type"))
			.child(ui::TableHeaderCell::new().child("Unique Count"))
			.children(number_columns_table_header_cells);
		let href = format!("./columns/{}", self.target_column_stats_table_row.name);
		let body = ui::TableRow::new()
			.child(
				ui::TableCell::new().child(
					ui::Link::new()
						.href(href)
						.child(self.target_column_stats_table_row.name),
				),
			)
			.child(ui::TableCell::new().child(ColumnTypeToken::new(
				self.target_column_stats_table_row.column_type,
			)))
			.child(
				ui::TableCell::new().child(
					self.target_column_stats_table_row
						.unique_count
						.map(|unique_count| unique_count.to_string()),
				),
			)
			.children(number_columns_table_cells);
		ui::Table::new()
			.width("100%".to_owned())
			.child(ui::TableHeader::new().child(header))
			.child(ui::TableBody::new().child(body))
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ColumnStatsTable {
	pub column_stats_table_rows: Vec<ColumnStatsTableRow>,
}

impl Component for ColumnStatsTable {
	fn into_node(self) -> Node {
		let table_header = ui::TableRow::new()
			.child(ui::TableHeaderCell::new().child("Column"))
			.child(ui::TableHeaderCell::new().child("Type"))
			.child(ui::TableHeaderCell::new().child("Unique Values Count"))
			.child(ui::TableHeaderCell::new().child("Null Count"))
			.child(ui::TableHeaderCell::new().child("Min"))
			.child(ui::TableHeaderCell::new().child("Max"))
			.child(ui::TableHeaderCell::new().child("Mean"))
			.child(ui::TableHeaderCell::new().child("Std"));
		let table_body = self.column_stats_table_rows.iter().map(|column_stats| {
			let link = if column_stats.column_type == ColumnType::Unknown {
				fragment().child(column_stats.name.clone()).into_node()
			} else {
				let href = format!("./columns/{}", column_stats.name);
				ui::Link::new()
					.href(href)
					.child(column_stats.name.clone())
					.into_node()
			};
			ui::TableRow::new()
				.child(ui::TableCell::new().child(link))
				.child(ui::TableCell::new().child(ColumnTypeToken::new(column_stats.column_type)))
				.child(
					ui::TableCell::new().child(
						column_stats
							.unique_count
							.map(|unique_count| unique_count.to_string()),
					),
				)
				.child(
					ui::TableCell::new().child(
						column_stats
							.invalid_count
							.map(|invalid_count| invalid_count.to_string()),
					),
				)
				.child(ui::TableCell::new().child(column_stats.min.map(ui::format_float)))
				.child(ui::TableCell::new().child(column_stats.max.map(ui::format_float)))
				.child(ui::TableCell::new().child(column_stats.mean.map(ui::format_float)))
				.child(ui::TableCell::new().child(column_stats.std.map(ui::format_float)))
		});
		ui::Table::new()
			.width("100%".to_owned())
			.child(ui::TableHeader::new().child(table_header))
			.child(ui::TableBody::new().children(table_body))
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ColumnTypeToken {
	column_type: ColumnType,
}

impl Component for ColumnTypeToken {
	fn into_node(self) -> Node {
		match self.column_type {
			ColumnType::Number => Some(NumberColumnToken::new().into_node()),
			ColumnType::Enum => Some(EnumColumnToken::new().into_node()),
			ColumnType::Text => Some(TextColumnToken::new().into_node()),
			ColumnType::Unknown => None,
		}
		.into_node()
	}
}
