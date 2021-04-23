use html::{component, html, Props};
use tangram_app_common::page_heading::PageHeading;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutProps},
	document::{Document, DocumentProps},
};
use tangram_serve::client;
use tangram_ui as ui;

#[derive(Props)]
pub struct PageProps {
	pub app_layout_props: AppLayoutProps,
	pub error: Option<String>,
}

#[component]
pub fn Page(props: PageProps) {
	let document_props = DocumentProps {
		client_wasm_js_src: Some(client!()),
	};
	html! {
		<Document {document_props}>
			<AppLayout {props.app_layout_props}>
				<ui::S1>
					<PageHeading>
						<ui::H1>{"Upload Model"}</ui::H1>
					</PageHeading>
					<ui::Form enc_type?="multipart/form-data" post?={Some(true)}>
						{
							props.error.map(|error| html! {
								<ui::Alert level={ui::Level::Danger}>
									{error}
								</ui::Alert>
							})
						}
						<ui::FileField
							disabled={None}
							label="File"
							name="file"
							required={Some(true)}
						/>
						<ui::Button button_type?={Some(ui::ButtonType::Submit)}>
							{"Upload"}
						</ui::Button>
					</ui::Form>
				</ui::S1>
			</AppLayout>
		</Document>
	}
}
