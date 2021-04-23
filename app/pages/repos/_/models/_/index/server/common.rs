use html::{component, html, Props};
use num::ToPrimitive;
use tangram_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use tangram_ui as ui;

#[derive(Props)]
pub struct TrainingSummarySectionProps {
	pub chosen_model_type_name: String,
	pub column_count: usize,
	pub model_comparison_metric_type_name: String,
	pub train_row_count: usize,
	pub test_row_count: usize,
}

#[component]
pub fn TrainingSummarySection(props: TrainingSummarySectionProps) {
	html! {
		<ui::S2>
			<ui::H2>{"Training Summary"}</ui::H2>
			<ui::P>
				{"Your dataset had "}
				<b>
					{(props.train_row_count +
						props.test_row_count).to_string()}
				</b>
				{" rows and "}
				<b>{props.column_count.to_string()}</b>
				{" columns. "}
				<b>{props.train_row_count.to_string()}</b>
				{" rows were used in training and "}
				<b>{props.test_row_count.to_string()}</b>
				{" rows were used in testing. The model with the highest "}
				<b>{props.model_comparison_metric_type_name}</b>
				{" was chosen. The best model is a "}
				<b>{props.chosen_model_type_name}</b>
				{"."}
			</ui::P>
		</ui::S2>
	}
}

#[derive(Props)]
pub struct FeatureImportancesSectionProps {
	pub n_columns: usize,
	pub n_features: usize,
	pub feature_importances_chart_values: Vec<FeatureImportance>,
	pub feature_importances_table_rows: Vec<FeatureImportance>,
}

#[derive(Clone, Debug)]
pub struct FeatureImportance {
	pub feature_importance_value: f32,
	pub feature_name: String,
}

#[component]
pub fn FeatureImportancesSection(props: FeatureImportancesSectionProps) {
	let description =
		"The chart and table below show which features were most important to the model.";
	html! {
		<ui::S2>
			<ui::H2>
				{"Feature Importances"}
			</ui::H2>
			<ui::P>
				{description}
			</ui::P>
			<FeatureImportancesChart values={props.feature_importances_chart_values} />
			<FeatureImportancesTable rows={props.feature_importances_table_rows} />
		</ui::S2>
	}
}

#[derive(Props)]
struct FeatureImportancesChartProps {
	values: Vec<FeatureImportance>,
}

#[component]
fn FeatureImportancesChart(props: FeatureImportancesChartProps) {
	let bar_chart_series = vec![BarChartSeries {
		color: ui::colors::BLUE.to_owned(),
		data: props
			.values
			.iter()
			.enumerate()
			.map(|(index, feature_importance)| BarChartPoint {
				label: feature_importance.feature_name.clone(),
				x: index.to_f64().unwrap(),
				y: Some(
					feature_importance
						.feature_importance_value
						.to_f64()
						.unwrap(),
				),
			})
			.collect(),
		title: Some("Feature Importance".to_owned()),
	}];
	let n_feature_importances_to_show_in_chart = props.values.len();
	html! {
		<ui::Card>
			<BarChart
				id?="feature_importances"
				series?={Some(bar_chart_series)}
				title?={Some(format!("Feature Importances for Top {} Features", n_feature_importances_to_show_in_chart))}
				x_axis_title?="Feature Name"
				y_axis_title?="Feature Importance Value"
				y_min?={Some(0.0)}
			/>
		</ui::Card>
	}
}

#[derive(Props)]
struct FeatureImportancesTableProps {
	rows: Vec<FeatureImportance>,
}

#[component]
fn FeatureImportancesTable(props: FeatureImportancesTableProps) {
	html! {
		<ui::Table width?="100%" >
			<ui::TableHeader>
				<ui::TableHeaderCell>
					{"Feature Name"}
				</ui::TableHeaderCell>
				<ui::TableHeaderCell>
					{"Feature Importance Value"}
				</ui::TableHeaderCell>
			</ui::TableHeader>
			<ui::TableBody>
			{props.rows.iter().map(|feature_importance_table_row| html! {
				<ui::TableRow>
					<ui::TableCell>
						{feature_importance_table_row.feature_name.clone()}
					</ui::TableCell>
					<ui::TableCell>
						{feature_importance_table_row.feature_importance_value.to_string()}
					</ui::TableCell>
				</ui::TableRow>
			}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}
