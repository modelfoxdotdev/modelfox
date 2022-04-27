use modelfox_charts::{
	bar_chart::{BarChartPoint, BarChartSeries},
	components::BarChart,
};
use modelfox_finite::FiniteF32;
use modelfox_ui as ui;
use num::ToPrimitive;
use pinwheel::prelude::*;

pub struct TrainingSummarySection {
	pub chosen_model_type_name: String,
	pub column_count: usize,
	pub comparison_metric_type_name: String,
	pub train_row_count: usize,
	pub test_row_count: usize,
	pub comparison_row_count: usize,
	pub overall_row_count: usize,
}

impl Component for TrainingSummarySection {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(ui::H2::new("Training Summary"))
			.child(
				ui::P::new()
					.child("Your dataset had ")
					.child(b().child(self.overall_row_count.to_string()))
					.child(" rows and ")
					.child(b().child(self.column_count.to_string()))
					.child(" columns. ")
					.child(b().child(self.train_row_count.to_string()))
					.child(" rows were used in training, ")
					.child(b().child(self.test_row_count.to_string()))
					.child(" rows were used in testing, and ")
					.child(b().child(self.comparison_row_count.to_string()))
					.child(" rows were used in model comparison. The model with the highest ")
					.child(b().child(self.comparison_metric_type_name))
					.child(" was chosen. The best model is a ")
					.child(b().child(self.chosen_model_type_name))
					.child("."),
			)
			.into_node()
	}
}

pub struct FeatureImportancesSection {
	pub n_columns: usize,
	pub n_features: usize,
	pub feature_importances_chart_values: Vec<FeatureImportance>,
	pub feature_importances_table_rows: Vec<FeatureImportance>,
}

#[derive(Clone, Debug)]
pub struct FeatureImportance {
	pub feature_importance_value: FiniteF32,
	pub feature_name: String,
}

impl Component for FeatureImportancesSection {
	fn into_node(self) -> Node {
		let description =
			"The chart and table below show which features were most important to the model.";
		ui::S2::new()
			.child(ui::H2::new("Feature Importances"))
			.child(ui::P::new().child(description))
			.child(FeatureImportancesChart {
				values: self.feature_importances_chart_values,
			})
			.child(FeatureImportancesTable {
				rows: self.feature_importances_table_rows,
			})
			.into_node()
	}
}

struct FeatureImportancesChart {
	values: Vec<FeatureImportance>,
}

impl Component for FeatureImportancesChart {
	fn into_node(self) -> Node {
		let bar_chart_series = vec![BarChartSeries {
			color: ui::colors::BLUE.to_owned(),
			data: self
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
		let n_feature_importances_to_show_in_chart = self.values.len();
		ui::Card::new()
			.child(Dehydrate::new(
				"feature_importances",
				BarChart::new()
					.series(bar_chart_series)
					.title(format!(
						"Feature Importances for Top {} Features",
						n_feature_importances_to_show_in_chart
					))
					.x_axis_title("Feature Name".to_owned())
					.y_axis_title("Feature Importance Value".to_owned())
					.y_min(0.0),
			))
			.into_node()
	}
}

struct FeatureImportancesTable {
	rows: Vec<FeatureImportance>,
}

impl Component for FeatureImportancesTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new()
					.child(ui::TableHeaderCell::new().child("Feature Name"))
					.child(ui::TableHeaderCell::new().child("Feature Importance Value")),
			)
			.child(ui::TableBody::new().children(self.rows.iter().map(
				|feature_importance_table_row| {
					ui::TableRow::new()
						.child(
							ui::TableCell::new()
								.child(feature_importance_table_row.feature_name.clone()),
						)
						.child(
							ui::TableCell::new().child(
								feature_importance_table_row
									.feature_importance_value
									.to_string(),
							),
						)
				},
			)))
			.into_node()
	}
}
