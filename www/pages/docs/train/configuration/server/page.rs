use html::{component, html};
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, TrainPage},
	document::{Document, DocumentProps},
};

#[component]
pub fn Page() {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	let code = ui::highlight(
		include_str!("./heart_disease.json"),
		ui::Language::Javascript,
	);
	html! {
		<Document {document_props}>
			<DocsLayout selected_page={DocsPage::Train(TrainPage::Configuration)} headings={None}>
				<ui::S1>
					<ui::H1>{"Configuration"}</ui::H1>
					<ui::S2>
						<ui::P>
							{"If you want more control over training you can provide a json config file to the tangram train command: "}<ui::InlineCode>{"--config config.json"}</ui::InlineCode>{"."}
							{" Below is an example config file. It includes all of the possible options you can set. Every field is optional."}
						</ui::P>
						<ui::Window padding={Some(true)}>
							<ui::Code
								code={code}
								hide_line_numbers?={Some(false)}
							/>
						</ui::Window>
					</ui::S2>
				</ui::S1>
			</DocsLayout>
		</Document>
	}
}
