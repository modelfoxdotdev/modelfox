use crate::common::{
	ClassificationProductionStatsChart, ClassificationProductionStatsIntervalChart,
	ClassifierChartEntry, ColumnStatsTable, ColumnStatsTableProps, DateWindowSelectForm,
	PredictionCountChart, PredictionCountChartEntry,
};
use html::{component, html, Props};
use tangram_app_common::date_window::{DateWindow, DateWindowInterval};
use tangram_ui as ui;

#[derive(Props)]
pub struct BinaryClassifierProps {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub prediction_count_chart: Vec<PredictionCountChartEntry>,
	pub prediction_stats_chart: ClassifierChartEntry,
	pub prediction_stats_interval_chart: Vec<ClassifierChartEntry>,
	pub overall_column_stats_table_props: ColumnStatsTableProps,
}

#[component]
pub fn BinaryClassifierPage(props: BinaryClassifierProps) {
	html! {
		<ui::S1>
			<ui::H1>{"Production Stats"}</ui::H1>
			<DateWindowSelectForm date_window={props.date_window} />
			<ui::Card>
				<ClassificationProductionStatsIntervalChart
					chart_data={props.prediction_stats_interval_chart}
					date_window_interval={props.date_window_interval}
				/>
			</ui::Card>
			<ui::Card>
				<PredictionCountChart
					chart_data={props.prediction_count_chart}
					date_window_interval={props.date_window_interval}
				/>
			</ui::Card>
			<ui::Card>
				<ClassificationProductionStatsChart
					chart_data={props.prediction_stats_chart}
					date_window={props.date_window}
				/>
			</ui::Card>
			<ColumnStatsTable {props.overall_column_stats_table_props} />
		</ui::S1>
	}
}
