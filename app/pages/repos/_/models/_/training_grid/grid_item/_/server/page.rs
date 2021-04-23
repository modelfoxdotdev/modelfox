use html::{component, html, Props};
use tangram_app_layouts::{
	document::{Document, DocumentProps},
	model_layout::{ModelLayout, ModelLayoutProps},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub id: String,
	pub model_grid_item_identifier: String,
	pub model_layout_props: ModelLayoutProps,
	pub model_hyperparameters: Vec<(String, String)>,
}

#[component]
pub fn Page(props: PageProps) {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<ModelLayout {props.model_layout_props}>
				<ui::S1>
					<ui::H1>{"Hyperparameters"}</ui::H1>
					<ModelHyperparametersTable hyperparameters={props.model_hyperparameters} />
				</ui::S1>
			</ModelLayout>
		</Document>
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
