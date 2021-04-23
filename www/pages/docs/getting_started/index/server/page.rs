use html::{component, html};
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage},
	document::{Document, DocumentProps},
};

#[component]
pub fn Page() {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<DocsLayout selected_page={DocsPage::GettingStarted(GettingStartedPage::Index)} headings={None}>
				<ui::S1>
					<ui::H1>{"Getting Started"}</ui::H1>
					<ui::S2>
						<ui::P>
							{"Thanks for trying Tangram!"}
						</ui::P>
						<ui::P>
							{"In this getting started guide, we will:"}
						</ui::P>
						<ui::List>
							<ui::ListItem>
								{"Train a model with the Tangram CLI to predict whether cardiac patients have heart disease."}
							</ui::ListItem>
							<ui::ListItem>
								{"Make predictions using the Tangram language libraries."}
							</ui::ListItem>
							<ui::ListItem>
								{"Learn more about our model with the Tangram web app."}
							</ui::ListItem>
							<ui::ListItem>
								{"Set up production monitoring and debug our model's performance."}
							</ui::ListItem>
						</ui::List>
					</ui::S2>
				</ui::S1>
			</DocsLayout>
		</Document>
	}
}
