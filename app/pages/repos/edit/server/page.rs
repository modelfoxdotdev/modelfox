use html::{component, html, Props};
use tangram_app_common::page_heading::PageHeading;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutProps},
	document::{Document, DocumentProps},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub app_layout_props: AppLayoutProps,
	pub title: String,
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
							{props.title.clone()}
						</ui::H1>
					</PageHeading>
					<UpdateTitleForm title={props.title} />
					<DangerZone />
				</ui::S1>
			</AppLayout>
		</Document>
	}
}

#[derive(Props)]
struct UpdateTitleFormProps {
	title: String,
}

#[component]
fn UpdateTitleForm(props: UpdateTitleFormProps) {
	html! {
		<ui::S2>
			<ui::H2>{"Title"}</ui::H2>
			<ui::Form post?={Some(true)}>
				<input name="action" type="hidden" value="update_title" />
				<ui::TextField
					label?="Title"
					name?="title"
					value?={Some(props.title)}
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
