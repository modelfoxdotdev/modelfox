use html::{component, html, Props};
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutProps},
	document::{Document, DocumentProps},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub app_layout_props: AppLayoutProps,
	pub member_email: String,
	pub is_admin: bool,
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
					<ui::H1>{"Edit Member"}</ui::H1>
					<ui::S2>
						<ui::TextField
							label?="Email"
							disabled?={Some(true)}
							value?={Some(props.member_email)}
						/>
						<ui::CheckboxField
							label?="Admin"
							disabled?={Some(true)}
							name?="is_admin"
						/>
					</ui::S2>
					<DangerZone />
				</ui::S1>
			</AppLayout>
		</Document>
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
