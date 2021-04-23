use html::{component, html, Props};
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutProps},
	document::{Document, DocumentProps},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub app_layout_props: AppLayoutProps,
	pub error: Option<String>,
	pub owner: Option<String>,
	pub owners: Option<Vec<Owner>>,
	pub title: Option<String>,
}

pub struct Owner {
	pub value: String,
	pub title: String,
}

#[component]
pub fn Page(props: PageProps) {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	let owner = props.owner;
	html! {
		<Document {document_props}>
			<AppLayout {props.app_layout_props}>
				<ui::S1>
					<ui::H1>{"Create New Repo"}</ui::H1>
					<ui::Form post?={Some(true)}>
						{props.error.map(|error| html! {
							<ui::Alert level={ui::Level::Danger}>
								{error}
							</ui::Alert>
						})}
						<ui::TextField
							label?="Title"
							name?="title"
							required?={Some(true)}
							value?={props.title}
						/>
						{props.owners.map(|owners| html! {
							<ui::SelectField
								label?="Owner"
								name?="owner"
								options?={Some(owners.into_iter().map(|owner| ui::SelectFieldOption {
									text: owner.title,
									value: owner.value,
								}).collect::<Vec<_>>())}
								required?={Some(true)}
								value?={owner}
							/>
						})}
						<ui::Button button_type?={Some(ui::ButtonType::Submit)}>
							{"Submit"}
						</ui::Button>
					</ui::Form>
				</ui::S1>
			</AppLayout>
		</Document>
	}
}
