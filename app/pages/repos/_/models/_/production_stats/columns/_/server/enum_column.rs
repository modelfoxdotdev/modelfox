use html::{component, html, Props};
use num::ToPrimitive;
use tangram_app_common::{
	date_window::DateWindow,
	metrics_row::MetricsRow,
	time::overall_chart_title,
	tokens::{PRODUCTION_COLOR, TRAINING_COLOR},
};
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_ui as ui;

#[derive(Props)]
pub struct EnumColumnProps {
	pub alert: Option<String>,
	pub counts_section_props: EnumColumnCountsSectionProps,
	pub stats_section_props: EnumColumnStatsSectionProps,
	pub unique_values_section_props: EnumColumnUniqueValuesSectionProps,
	pub invalid_values_section_props: EnumColumnInvalidValuesSectionProps,
}

pub struct EnumColumnOverallHistogramEntry {
	pub production_count: u64,
	pub production_fraction: Option<f32>,
	pub training_count: u64,
	pub training_fraction: f32,
}

#[component]
pub fn EnumColumn(props: EnumColumnProps) {
	html! {
		<>
			{props.alert.map(|alert| html! {
				<ui::Alert level={ui::Level::Danger}>
					{alert}
				</ui::Alert>
			})}
			<EnumStatsSection {props.stats_section_props} />
			<EnumCountsSection {props.counts_section_props} />
			<EnumUniqueValuesSection {props.unique_values_section_props} />
			<EnumInvalidValuesSection {props.invalid_values_section_props} />
		</>
	}
}

#[derive(Props)]
pub struct EnumColumnStatsSectionProps {
	pub overall_chart_data: Vec<(String, EnumColumnOverallHistogramEntry)>,
	pub column_name: String,
	pub date_window: DateWindow,
}

#[component]
pub fn EnumStatsSection(props: EnumColumnStatsSectionProps) {
	let overall_chart_series = vec![
		BarChartSeries {
			color: TRAINING_COLOR.to_owned(),
			data: props
				.overall_chart_data
				.iter()
				.enumerate()
				.map(|(index, (label, value))| BarChartPoint {
					label: label.to_owned(),
					x: index.to_f64().unwrap(),
					y: Some(value.training_fraction.to_f64().unwrap()),
				})
				.collect(),
			title: Some("Training".to_owned()),
		},
		BarChartSeries {
			color: PRODUCTION_COLOR.to_owned(),
			data: props
				.overall_chart_data
				.iter()
				.enumerate()
				.map(|(index, (label, value))| BarChartPoint {
					label: label.to_owned(),
					x: index.to_f64().unwrap(),
					y: value
						.production_fraction
						.map(|production_fraction| production_fraction.to_f64().unwrap()),
				})
				.collect(),
			title: Some("Production".to_owned()),
		},
	];
	let overall_distribution_chart_title = overall_chart_title(
		&props.date_window,
		format!("Distribution of Unique Values for {}", props.column_name),
	);
	html! {
		<ui::Card>
			<BarChart
				id?="enum_overall"
				series?={Some(overall_chart_series)}
				title?={Some(overall_distribution_chart_title)}
				x_axis_title?={Some(props.column_name)}
				y_axis_title?="Percent"
				y_max?={Some(1.0)}
				y_min?={Some(0.0)}
			/>
		</ui::Card>
	}
}

#[derive(Props)]
pub struct EnumColumnCountsSectionProps {
	pub absent_count: u64,
	pub invalid_count: u64,
	pub row_count: u64,
}

#[component]
pub fn EnumCountsSection(props: EnumColumnCountsSectionProps) {
	html! {
		<MetricsRow>
			<ui::NumberCard
				title="Row Count"
				value={props.row_count.to_string()}
			/>
			<ui::NumberCard
				title="Absent Count"
				value={props.absent_count.to_string()}
			/>
			<ui::NumberCard
				title="Invalid Count"
				value={props.invalid_count.to_string()}
			/>
		</MetricsRow>
	}
}

#[derive(Props)]
pub struct EnumColumnUniqueValuesSectionProps {
	pub enum_unique_values_table_props: EnumUniqueValuesTableProps,
}

#[component]
pub fn EnumUniqueValuesSection(props: EnumColumnUniqueValuesSectionProps) {
	html! {
		<ui::S2>
			<ui::H2>{"Unique Values"}</ui::H2>
			<EnumUniqueValuesTable {props.enum_unique_values_table_props} />
		</ui::S2>
	}
}

#[derive(Props)]
pub struct EnumUniqueValuesTableProps {
	pub rows: Vec<EnumUniqueValuesTableRow>,
}

pub struct EnumUniqueValuesTableRow {
	pub name: String,
	pub training_count: usize,
	pub production_count: usize,
	pub training_fraction: f32,
	pub production_fraction: Option<f32>,
}

#[component]
pub fn EnumUniqueValuesTable(props: EnumUniqueValuesTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"Value"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Training Count"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Production Count"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Training Fraction"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Production Fraction"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{props.rows.iter().map(|row| html! {
					<ui::TableRow>
						<ui::TableCell>
							{row.name.to_owned()}
						</ui::TableCell>
						<ui::TableCell>
							{row.training_count.to_string()}
						</ui::TableCell>
						<ui::TableCell>
							{row.production_count.to_string()}
						</ui::TableCell>
						<ui::TableCell>
							{ui::format_percent(row.training_fraction)}
						</ui::TableCell>
						<ui::TableCell>
							{ui::format_option_percent(row.production_fraction)}
						</ui::TableCell>
					</ui::TableRow>
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}

#[derive(Props)]
pub struct EnumColumnInvalidValuesSectionProps {
	pub enum_invalid_values_table_props: Option<EnumInvalidValuesTableProps>,
}

#[component]
pub fn EnumInvalidValuesSection(props: EnumColumnInvalidValuesSectionProps) {
	html! {
		<>
		{props.enum_invalid_values_table_props.map(|enum_invalid_values_table_props| html! {
			<ui::S2>
				<ui::H2>{"Invalid Values"}</ui::H2>
				<EnumInvalidValuesTable {enum_invalid_values_table_props} />
			</ui::S2>
			})}
		</>
	}
}

#[derive(Props)]
pub struct EnumInvalidValuesTableProps {
	pub rows: Vec<EnumInvalidValuesTableRow>,
}

pub struct EnumInvalidValuesTableRow {
	pub name: String,
	pub count: usize,
	pub production_fraction: f32,
}

#[component]
pub fn EnumInvalidValuesTable(props: EnumInvalidValuesTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"Value"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Count"}
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{"Production Fraction"}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
			{props.rows.into_iter().map(|row| html! {
				<ui::TableRow>
					<ui::TableCell>
						{row.name}
					</ui::TableCell>
					<ui::TableCell>
						{row.count.to_string()}
					</ui::TableCell>
					<ui::TableCell>
						{ui::format_percent(row.production_fraction)}
					</ui::TableCell>
				</ui::TableRow>
			}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}
