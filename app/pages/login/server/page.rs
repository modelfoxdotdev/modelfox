use html::{component, html, Props};
use tangram_app_layouts::{
	auth_layout::AuthLayout,
	document::{Document, DocumentProps},
};
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub code: bool,
	pub email: Option<String>,
	pub error: Option<String>,
}

#[component]
pub fn Page(props: PageProps) {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<AuthLayout>
				<ui::Form post?={Some(true)}>
					{props.error.map(|error| html! {
						<ui::Alert level={ui::Level::Danger}>
							{error}
						</ui::Alert>
					})}
					<ui::TextField
						autocomplete?="username"
						name?={Some("email".into())}
						placeholder?="Email"
						value?={props.email}
					/>
					{if props.code {
						Some(html! {
							<ui::TextField
								name?="code"
								placeholder?="Code"
							/>
						})
					} else {
						None
					}}
					<ui::Button button_type?={Some(ui::ButtonType::Submit)}>
						{"Login"}
					</ui::Button>
					{if props.code {
						Some(html! {
							<div class="login-code-message">
								{"We emailed you a code. Copy and paste it above to log in."}
							</div>
						})
					} else {
						None
					}}
				</ui::Form>
			</AuthLayout>
		</Document>
	}
}
