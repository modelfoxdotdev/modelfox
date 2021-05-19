use crate::common::{
	ClassificationProductionStatsChart, ClassificationProductionStatsIntervalChart,
	ClassifierChartEntry, ColumnStatsTable, DateWindowSelectForm, PredictionCountChart,
	PredictionCountChartEntry,
};
use pinwheel::prelude::*;
use tangram_app_ui::date_window::{DateWindow, DateWindowInterval};
use tangram_ui as ui;

#[derive(ComponentBuilder)]
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
			.child(ui::H1::new().child("Production Stats"))
			.child(DateWindowSelectForm::new(self.date_window))
			.child(
				ui::Card::new().child(ClassificationProductionStatsIntervalChart::new(
					self.prediction_stats_interval_chart,
					self.date_window_interval,
				)),
			)
			.child(ui::Card::new().child(PredictionCountChart::new(
				self.prediction_count_chart,
				self.date_window_interval,
			)))
			.child(
				ui::Card::new().child(ClassificationProductionStatsChart::new(
					self.prediction_stats_chart,
					self.date_window,
				)),
			)
			.child(self.overall_column_stats_table)
			.into_node()
	}
}
