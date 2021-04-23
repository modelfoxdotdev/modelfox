use html::{component, html, Props};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub id: String,
	pub model_layout_props: ModelLayoutProps,
	pub num_models: usize,
	pub trained_models_metrics: Vec<TrainedModel>,
	pub best_model_metrics: TrainedModel,
	pub model_comparison_metric_name: String,
	pub best_model_hyperparameters: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct TrainedModel {
	pub identifier: String,
	pub model_comparison_metric_value: f32,
	pub model_type: String,
	pub time: String,
}

#[component]
pub fn Page(props: PageProps) {
	let description = "This page shows you details of all the models that you trained.";
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<ModelLayout {props.model_layout_props}>
				<ui::S1>
					<ui::H1>{"Training Grid"}</ui::H1>
					<ui::P>
						{description}
					</ui::P>
					<ui::S2>
						<ui::H2>{"Best Model Metrics"}</ui::H2>
						<WinningModelMetricsTable
							best_model={props.best_model_metrics}
							model_comparison_metric_name={props.model_comparison_metric_name.clone()}
						/>
					</ui::S2>
					<ui::S2>
						<ui::H2>{"Best Model Hyperparameters"}</ui::H2>
						<ModelHyperparametersTable hyperparameters={props.best_model_hyperparameters} />
					</ui::S2>
					<ui::S2>
						<ui::H2>{"All Models"}</ui::H2>
						<AllTrainedModelsMetricsTable
							trained_models={props.trained_models_metrics}
							model_comparison_metric_name={props.model_comparison_metric_name}
						/>
					</ui::S2>
				</ui::S1>
			</ModelLayout>
		</Document>
	}
}

#[derive(Props)]
pub struct WinningModelMetricsTableProps {
	best_model: TrainedModel,
	model_comparison_metric_name: String,
}

#[component]
fn WinningModelMetricsTable(props: WinningModelMetricsTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						"Model Number"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						"Model Type"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						"Training Time"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{props.model_comparison_metric_name}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableRow>
				<ui::TableCell>
					{props.best_model.identifier}
				</ui::TableCell>
				<ui::TableCell>
					{props.best_model.model_type}
				</ui::TableCell>
				<ui::TableCell>
					{props.best_model.time}
				</ui::TableCell>
				<ui::TableCell>
					{ui::format_float(props.best_model.model_comparison_metric_value)}
				</ui::TableCell>
			</ui::TableRow>
		</ui::Table>
	}
}

#[derive(Props)]
pub struct AllTrainedModelsMetricsTableProps {
	trained_models: Vec<TrainedModel>,
	model_comparison_metric_name: String,
}

#[component]
fn AllTrainedModelsMetricsTable(props: AllTrainedModelsMetricsTableProps) {
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>
						"Model Number"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						"Model Type"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						"Training Time"
					</ui::TableHeaderCell>
					<ui::TableHeaderCell>
						{props.model_comparison_metric_name}
					</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			{props.trained_models.into_iter().map(|trained_model| html! {
				<ui::TableRow>
					<ui::TableCell>
						<ui::Link href={format!("./grid_item/{}", trained_model.identifier)}>
							{trained_model.identifier}
						</ui::Link>
					</ui::TableCell>
					<ui::TableCell>
						{trained_model.model_type}
					</ui::TableCell>
					<ui::TableCell>
						{trained_model.time}
					</ui::TableCell>
					<ui::TableCell>
						{ui::format_float(trained_model.model_comparison_metric_value)}
					</ui::TableCell>
				</ui::TableRow>
			}).collect::<Vec<_>>()}
		</ui::Table>
	}
}

#[derive(Props)]
pub struct ModelHyperparametersTableProps {
	hyperparameters: Vec<(String, String)>,
}

#[component]
fn ModelHyperparametersTable(props: ModelHyperparametersTableProps) {
	html! {
		<ui::Table width?="100%">
			{props.hyperparameters.into_iter().map(|(hyperparam_name, hyperparam_value)| {
				html! {
					<ui::TableRow>
						<ui::TableHeaderCell expand?={Some(false)}>
							{hyperparam_name}
						</ui::TableHeaderCell>
						<ui::TableCell expand?={Some(true)}>
							{hyperparam_value}
						</ui::TableCell>
					</ui::TableRow>
				}
			}).collect::<Vec<_>>()}
		</ui::Table>
	}
}
