use html::{component, html};
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage, PredictPage},
	document::{Document, DocumentProps},
};

#[component]
pub fn Page() {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<DocsLayout selected_page={DocsPage::GettingStarted(GettingStartedPage::Predict(PredictPage::Index))} headings={None}>
				<ui::S1>
					<ui::H1>{"Predict"}</ui::H1>
					<ui::S2>
						<ui::List>
							<ui::ListItem>
								<ui::Link title?="Elixir" href="elixir">
									{"Elixir"}
								</ui::Link>
							</ui::ListItem>
							<ui::ListItem>
								<ui::Link title?="Go" href="go">
									{"Go"}
								</ui::Link>
							</ui::ListItem>
							<ui::ListItem>
								<ui::Link title?="Node.js" href="node">
									{"Node.js"}
								</ui::Link>
							</ui::ListItem>
							<ui::ListItem>
								<ui::Link title?="Python" href="python">
									{"Python"}
								</ui::Link>
							</ui::ListItem>
							<ui::ListItem>
								<ui::Link title?="Ruby" href="ruby">
									{"Ruby"}
								</ui::Link>
							</ui::ListItem>
							<ui::ListItem>
								<ui::Link title?="Rust" href="rust">
									{"Rust"}
								</ui::Link>
							</ui::ListItem>
						</ui::List>
					</ui::S2>
				</ui::S1>
			</DocsLayout>
		</Document>
	}
}
