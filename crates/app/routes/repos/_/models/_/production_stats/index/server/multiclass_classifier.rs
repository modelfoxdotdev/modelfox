use crate::common::{
	ClassificationProductionStatsChart, ClassificationProductionStatsIntervalChart,
	ClassifierChartEntry, ColumnStatsTable, PredictionCountChart, PredictionCountChartEntry,
};
use pinwheel::prelude::*;
use modelfox_app_date_window::{DateWindow, DateWindowInterval};
use modelfox_app_ui::{
	class_select_field::ClassSelectField, date_window_select_field::DateWindowSelectField,
};
use modelfox_ui as ui;

pub struct MulticlassClassifier {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub class: String,
	pub classes: Vec<String>,
	pub prediction_count_chart: Vec<PredictionCountChartEntry>,
	pub prediction_stats_chart: ClassifierChartEntry,
	pub prediction_stats_interval_chart: Vec<ClassifierChartEntry>,
	pub overall_column_stats_table: ColumnStatsTable,
}

impl Component for MulticlassClassifier {
	fn into_node(self) -> Node {
		ui::S1::new()
			.child(ui::H1::new("Production Stats"))
			.child(DateWindowAndClassSelectForm {
				date_window: self.date_window,
				date_window_interval: self.date_window_interval,
				class: self.class,
				classes: self.classes,
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

pub struct DateWindowAndClassSelectForm {
	pub date_window: DateWindow,
	pub date_window_interval: DateWindowInterval,
	pub class: String,
	pub classes: Vec<String>,
}

impl Component for DateWindowAndClassSelectForm {
	fn into_node(self) -> Node {
		ui::Form::new()
			.child(DateWindowSelectField::new(self.date_window))
			.child(ClassSelectField {
				class: self.class.clone(),
				classes: self.classes,
			})
			.child(
				noscript().child(
					ui::Button::new()
						.button_type(ui::ButtonType::Submit)
						.child("Submit"),
				),
			)
			.into_node()
	}
}
