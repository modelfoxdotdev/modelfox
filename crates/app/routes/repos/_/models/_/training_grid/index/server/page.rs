use pinwheel::prelude::*;
use tangram_app_layouts::{
	document::Document,
	model_layout::{ModelLayout, ModelLayoutInfo},
};
use tangram_ui as ui;

pub struct Page {
	pub id: String,
	pub model_layout_info: ModelLayoutInfo,
	pub num_models: usize,
	pub trained_models_metrics: Vec<TrainedModel>,
	pub best_model_metrics: TrainedModel,
	pub comparison_metric_name: String,
	pub best_model_hyperparameters: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct TrainedModel {
	pub identifier: String,
	pub comparison_metric_value: f32,
	pub model_type: String,
	pub time: String,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let description = "This page shows you details of all the models that you trained.";
		Document::new()
			.child(
				ModelLayout::new(self.model_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new("Training Grid"))
						.child(ui::P::new().child(description))
						.child(
							ui::S2::new()
								.child(ui::H2::new("Best Model Metrics"))
								.child(WinningModelMetricsTable {
									best_model: self.best_model_metrics,
									comparison_metric_name: self.comparison_metric_name.clone(),
								}),
						)
						.child(
							ui::S2::new()
								.child(ui::H2::new("Best Model Hyperparameters"))
								.child(ModelHyperparametersTable {
									hyperparameters: self.best_model_hyperparameters,
								}),
						)
						.child(ui::S2::new().child(ui::H2::new("All Models")).child(
							AllTrainedModelsMetricsTable {
								trained_models: self.trained_models_metrics,
								comparison_metric_name: self.comparison_metric_name,
							},
						)),
				),
			)
			.into_node()
	}
}

pub struct WinningModelMetricsTable {
	best_model: TrainedModel,
	comparison_metric_name: String,
}

impl Component for WinningModelMetricsTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Model Number"))
						.child(ui::TableHeaderCell::new().child("Model Type"))
						.child(ui::TableHeaderCell::new().child("Training Time"))
						.child(ui::TableHeaderCell::new().child(self.comparison_metric_name)),
				),
			)
			.child(
				ui::TableRow::new()
					.child(ui::TableCell::new().child(self.best_model.identifier))
					.child(ui::TableCell::new().child(self.best_model.model_type))
					.child(ui::TableCell::new().child(self.best_model.time))
					.child(
						ui::TableCell::new()
							.child(ui::format_float(self.best_model.comparison_metric_value)),
					),
			)
			.into_node()
	}
}

pub struct AllTrainedModelsMetricsTable {
	trained_models: Vec<TrainedModel>,
	comparison_metric_name: String,
}

impl Component for AllTrainedModelsMetricsTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Model Number"))
						.child(ui::TableHeaderCell::new().child("Model Type"))
						.child(ui::TableHeaderCell::new().child("Training Time"))
						.child(ui::TableHeaderCell::new().child(self.comparison_metric_name)),
				),
			)
			.children(self.trained_models.into_iter().map(|trained_model| {
				ui::TableRow::new()
					.child(
						ui::TableCell::new().child(
							ui::Link::new()
								.href(format!("./grid_item/{}", trained_model.identifier))
								.child(trained_model.identifier),
						),
					)
					.child(ui::TableCell::new().child(trained_model.model_type))
					.child(ui::TableCell::new().child(trained_model.time))
					.child(
						ui::TableCell::new()
							.child(ui::format_float(trained_model.comparison_metric_value)),
					)
			}))
			.into_node()
	}
}

pub struct ModelHyperparametersTable {
	hyperparameters: Vec<(String, String)>,
}

impl Component for ModelHyperparametersTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.children(self.hyperparameters.into_iter().map(
				|(hyperparam_name, hyperparam_value)| {
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child(hyperparam_name))
						.child(
							ui::TableCell::new()
								.width("100%".to_owned())
								.child(hyperparam_value),
						)
				},
			))
			.into_node()
	}
}
