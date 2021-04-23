use html::{component, html};
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage},
	document::{Document, DocumentProps},
};

#[component]
pub fn Page() {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<DocsLayout selected_page={DocsPage::Overview} headings={None}>
				<ui::S1>
					<ui::H1>{"Overview"}</ui::H1>
					<ui::S2>
						<ui::P>
							{"Welcome to the documentation for Tangram!"}
						</ui::P>
						<ui::P>
							{"Tangram is an automated machine learning framework designed for programmers. With Tangram, you:"}
						</ui::P>
						<ui::List>
							<ui::ListItem>
								{"Train a model from a CSV on the command line."}
							</ui::ListItem>
							<ui::ListItem>
								{"Make predictions from Elixir, Go, JavaScript, Python, Ruby, or Rust."}
							</ui::ListItem>
							<ui::ListItem>
								{"Learn about your models and monitor them in production from your browser."}
							</ui::ListItem>
						</ui::List>
					</ui::S2>
				</ui::S1>
			</DocsLayout>
		</Document>
	}
}
