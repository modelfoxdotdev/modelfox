use crate::common::{
	ClassificationProductionStatsChart, ClassificationProductionStatsIntervalChart,
	ClassifierChartEntry, ColumnStatsTable, ColumnStatsTableProps, PredictionCountChart,
	PredictionCountChartEntry,
};
use html::{component, html, Props};
use tangram_app_common::{
	class_select_field::ClassSelectField,
	date_window::{DateWindow, DateWindowInterval},
	date_window_select_field::DateWindowSelectField,
};
use tangram_ui as ui;

#[derive(Props)]
pub struct MulticlassClassifierProps {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub class: String,
	pub classes: Vec<String>,
	pub prediction_count_chart: Vec<PredictionCountChartEntry>,
	pub prediction_stats_chart: ClassifierChartEntry,
	pub prediction_stats_interval_chart: Vec<ClassifierChartEntry>,
	pub overall_column_stats_table_props: ColumnStatsTableProps,
}

#[component]
pub fn MulticlassClassifierPage(props: MulticlassClassifierProps) {
	html! {
		<ui::S1>
			<ui::H1>{"Production Stats"}</ui::H1>
			<DateWindowAndClassSelectForm
				date_window={props.date_window}
				date_window_interval={props.date_window_interval}
				class={props.class}
				classes={props.classes}
			/>
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

#[derive(Props)]
pub struct DateWindowAndClassSelectFormProps {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub class: String,
	pub classes: Vec<String>,
}

#[component]
fn DateWindowAndClassSelectForm(props: DateWindowAndClassSelectFormProps) {
	html! {
		<ui::Form>
			<DateWindowSelectField date_window={props.date_window} />
			<ClassSelectField class={props.class.clone()} classes={props.classes} />
			<noscript>
				<ui::Button button_type?={Some(ui::ButtonType::Submit)}>
					{"Submit"}
				</ui::Button>
			</noscript>
		</ui::Form>
	}
}
