use html::{component, html, Props};
use tangram_app_common::page_heading::PageHeading;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutProps},
	document::{Document, DocumentProps},
};
use tangram_id::Id;
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub app_layout_props: AppLayoutProps,
	pub model_id: Id,
	pub model_heading: String,
	pub tag: Option<String>,
	pub created_at: String,
}

#[component]
pub fn Page(props: PageProps) {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<AppLayout {props.app_layout_props}>
				<ui::S1>
					<PageHeading>
						<ui::H1>
							{props.model_heading}
						</ui::H1>
					</PageHeading>
					<ModelInfoTable created_at={props.created_at} model_id={props.model_id} />
					<UpdateTagForm tag={props.tag} />
					<DangerZone />
				</ui::S1>
			</AppLayout>
		</Document>
	}
}

#[derive(Props)]
struct ModelInfoTableProps {
	model_id: Id,
	created_at: String,
}

#[component]
fn ModelInfoTable(props: ModelInfoTableProps) {
	html! {
		<ui::S2>
			<ui::Table>
				<ui::TableRow>
					<ui::TableHeaderCell>
					{"Model Id"}
					</ui::TableHeaderCell>
					<ui::TableCell>
						{Some(props.model_id.to_string())}
					</ui::TableCell>
				</ui::TableRow>
				<ui::TableRow>
					<ui::TableHeaderCell>
						{"Uploaded At"}
					</ui::TableHeaderCell>
					<ui::TableCell>
						{Some(props.created_at)}
					</ui::TableCell>
				</ui::TableRow>
			</ui::Table>
		</ui::S2>
	}
}

#[derive(Props)]
struct UpdateTagFormProps {
	tag: Option<String>,
}

#[component]
fn UpdateTagForm(props: UpdateTagFormProps) {
	html! {
		<ui::S2>
			<ui::H2>{"Tag"}</ui::H2>
			<ui::Form post?={Some(true)}>
				<input name="action" type="hidden" value="update_tag" />
				<ui::TextField
					label?="Tag"
					name?="tag"
					value?={props.tag}
				/>
				<ui::Button button_type?={Some(ui::ButtonType::Submit)}>
					{"Update"}
				</ui::Button>
			</ui::Form>
		</ui::S2>
	}
}

#[component]
fn DangerZone() {
	html! {
		<ui::S2>
			<ui::H2>{"Danger Zone"}</ui::H2>
			<ui::Form post?={Some(true)} onsubmit?="return confirm(\"Are you sure?\")">
				<input name="action" type="hidden" value="delete" />
				<ui::Button
					button_type?={Some(ui::ButtonType::Submit)}
					color?={Some(ui::colors::RED.to_owned())}
				>
					{"Delete"}
				</ui::Button>
			</ui::Form>
		</ui::S2>
	}
}
