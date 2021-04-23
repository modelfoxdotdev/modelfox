use html::{component, html, Props};
use tangram_app_common::{
	column_type::ColumnType,
	metrics_row::MetricsRow,
	tokens::{EnumColumnToken, NumberColumnToken, TextColumnToken},
};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub model_layout_props: ModelLayoutProps,
	pub target_column_stats_table_props: TargetColumnStatsTableProps,
	pub column_stats_table_props: ColumnStatsTableProps,
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

#[component]
pub fn Page(props: PageProps) {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<ModelLayout {props.model_layout_props}>
				<ui::S1>
					<ui::H1>{"Training Stats"}</ui::H1>
					<ui::S2>
						<ui::H2>{"Target Column"}</ui::H2>
						<TargetColumnStatsTable {props.target_column_stats_table_props} />
					</ui::S2>
					<ui::S2>
						<ui::H2>{"Columns"}</ui::H2>
						<MetricsRow>
							<ui::NumberCard
								title="Rows"
								value={props.row_count.to_string()}
							/>
							<ui::NumberCard
								title="Columns"
								value={props.column_count.to_string()}
							/>
						</MetricsRow>
						<ColumnStatsTable {props.column_stats_table_props} />
					</ui::S2>
				</ui::S1>
			</ModelLayout>
		</Document>
	}
}

#[derive(Props)]
pub struct TargetColumnStatsTableProps {
	pub target_column_stats_table_row: ColumnStatsTableRow,
}

#[component]
fn TargetColumnStatsTable(props: TargetColumnStatsTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"Column"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Type"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Unique Count"}
					</ui::TableHeaderCell>
					{if props.target_column_stats_table_row.column_type == ColumnType::Number {
						Some(html! {
							<>
								<ui::TableHeaderCell>
									{"Min"}
								</ui::TableHeaderCell>
								<ui::TableHeaderCell>
									{"Max"}
								</ui::TableHeaderCell>
								<ui::TableHeaderCell>
									{"Mean"}
								</ui::TableHeaderCell>
								<ui::TableHeaderCell>
									{"Std"}
								</ui::TableHeaderCell>
							</>
						})
					} else {
						None
					}}
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				<ui::TableRow>
					<ui::TableCell>
						<ui::Link href={format!("./columns/{}", props.target_column_stats_table_row.name)}>
							{props.target_column_stats_table_row.name}
						</ui::Link>
					</ui::TableCell>
					<ui::TableCell>
						<ColumnTypeToken column_type={props.target_column_stats_table_row.column_type} />
					</ui::TableCell>
					<ui::TableCell>
						{props.target_column_stats_table_row.unique_count.map(|unique_count| unique_count.to_string())}
					</ui::TableCell>
					{if props.target_column_stats_table_row.column_type == ColumnType::Number {
						Some(html! {
							<>
								<ui::TableCell>
									{props.target_column_stats_table_row.min.unwrap().to_string()}
								</ui::TableCell>
								<ui::TableCell>
									{props.target_column_stats_table_row.max.unwrap().to_string()}
								</ui::TableCell>
								<ui::TableCell>
									{props.target_column_stats_table_row.mean.unwrap().to_string()}
								</ui::TableCell>
								<ui::TableCell>
									{props.target_column_stats_table_row.std.unwrap().to_string()}
								</ui::TableCell>
							</>
						})
					} else {
						None
					}}
				</ui::TableRow>
			</ui::TableBody>
		</ui::Table>
	}
}

#[derive(Props)]
pub struct ColumnStatsTableProps {
	pub column_stats_table_rows: Vec<ColumnStatsTableRow>,
}

#[component]
fn ColumnStatsTable(props: ColumnStatsTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"Column"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Type"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Unique Values Count"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Null Count"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Min"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Max"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Mean"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Std"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{props.column_stats_table_rows.iter().map(|column_stats| html! {
					<ui::TableRow>
						<ui::TableCell>
							{if column_stats.column_type == ColumnType::Unknown {
								html! {
									<>
										{column_stats.name.clone()}
									</>
								}
							} else {
								html! {
									<ui::Link href={format!("./columns/{}", column_stats.name)}>
										{column_stats.name.clone()}
									</ui::Link>
								}
							}}
						</ui::TableCell>
						<ui::TableCell>
							<ColumnTypeToken column_type={column_stats.column_type} />
						</ui::TableCell>
						<ui::TableCell>
							{column_stats.unique_count.map(|unique_count| unique_count.to_string())}
						</ui::TableCell>
						<ui::TableCell>
							{column_stats.invalid_count.map(|invalid_count| invalid_count.to_string())}
						</ui::TableCell>
						<ui::TableCell>
							{column_stats.min.map(ui::format_float)}
						</ui::TableCell>
						<ui::TableCell>
							{column_stats.max.map(ui::format_float)}
						</ui::TableCell>
						<ui::TableCell>
							{column_stats.mean.map(ui::format_float)}
						</ui::TableCell>
						<ui::TableCell>
							{column_stats.std.map(ui::format_float)}
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}

#[derive(Props)]
pub struct ColumnTypeTokenProps {
	column_type: ColumnType,
}

#[component]
fn ColumnTypeToken(props: ColumnTypeTokenProps) {
	match props.column_type {
		ColumnType::Number => html! { <NumberColumnToken /> },
		ColumnType::Enum => html! { <EnumColumnToken /> },
		ColumnType::Text => html! { <TextColumnToken /> },
		ColumnType::Unknown => html! { <></> },
	}
}
