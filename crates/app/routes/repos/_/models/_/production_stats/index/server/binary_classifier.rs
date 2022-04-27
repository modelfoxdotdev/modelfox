use crate::common::{
	ClassificationProductionStatsChart, ClassificationProductionStatsIntervalChart,
	ClassifierChartEntry, ColumnStatsTable, DateWindowSelectForm, PredictionCountChart,
	PredictionCountChartEntry,
};
use modelfox_app_date_window::{DateWindow, DateWindowInterval};
use modelfox_ui as ui;
use pinwheel::prelude::*;

pub struct BinaryClassifier {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub prediction_count_chart: Vec<PredictionCountChartEntry>,
	pub prediction_stats_chart: ClassifierChartEntry,
	pub prediction_stats_interval_chart: Vec<ClassifierChartEntry>,
	pub overall_column_stats_table: ColumnStatsTable,
}

impl Component for BinaryClassifier {
	fn into_node(self) -> Node {
		ui::S1::new()
			.child(ui::H1::new("Production Stats"))
			.child(DateWindowSelectForm {
				date_window: self.date_window,
			})
			.child(
				ui::Card::new().child(ClassificationProductionStatsIntervalChart {
					chart_data: self.prediction_stats_interval_chart,
					date_window_interval: self.date_window_interval,
				}),
			)
			.child(ui::Card::new().child(PredictionCountChart {
				chart_data: self.prediction_count_chart,
				date_window_interval: self.date_window_interval,
			}))
			.child(ui::Card::new().child(ClassificationProductionStatsChart {
				chart_data: self.prediction_stats_chart,
				date_window: self.date_window,
			}))
			.child(self.overall_column_stats_table)
			.into_node()
	}
}
