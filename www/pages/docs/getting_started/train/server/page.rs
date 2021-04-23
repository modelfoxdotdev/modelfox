use dataset_preview::DatasetPreview;
use html::{component, html};
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage},
	document::{Document, DocumentProps},
};

mod dataset_preview;

#[component]
pub fn Page() {
	let document_props = DocumentProps {
		client_wasm_js_src: None,
	};
	html! {
		<Document {document_props}>
			<DocsLayout selected_page={DocsPage::GettingStarted(GettingStartedPage::Train)} headings={None}>
				<ui::S1>
					<ui::H1>{"Train"}</ui::H1>
					<ui::S2>
						<ui::H2>{"Install the Tangram CLI"}</ui::H2>
						<ui::P>
							{"If you haven't already, "}
							<ui::Link href="/docs/install">
								{"install the Tangram CLI."}
							</ui::Link>
						</ui::P>
					</ui::S2>
					<ui::S2>
						<ui::H2>{"Get the data"}</ui::H2>
						<ui::P>
							<ui::Link href="/heart_disease.csv">
								{"download heart_disease.csv"}
							</ui::Link>
						</ui::P>
						<ui::P>
							{"The heart disease dataset contains information from cardiac patients such as their age, cholesterol, and stress test results. Below are some example rows."}
						</ui::P>
						<DatasetPreview />
						<ui::P>
							{"The last column, called "}
							<ui::InlineCode>{"diagnosis"}</ui::InlineCode>
							{", is either "}
							<ui::InlineCode>{"Positive"}</ui::InlineCode>
							{" if the patient has heart disease or "}
							<ui::InlineCode>{"Negative"}</ui::InlineCode>
							{" if they don't."}
						</ui::P>
					</ui::S2>
					<ui::S2>
						<ui::H2>{"Train"}</ui::H2>
						<ui::P>
							{"We can train a model to predict the "}
							<ui::InlineCode>{"diagnosis"}</ui::InlineCode>
							{" column using the "}
							<ui::InlineCode>{"tangram train"}</ui::InlineCode>
							{" command, passing in the path to the CSV file and the name of the column we want to predict, called the "}
							<ui::InlineCode>{"target"}</ui::InlineCode>
							{" column."}
						</ui::P>
						<ui::Window padding={Some(true)}>
							<ui::Code
								code="$ tangram train --file heart_disease.csv --target diagnosis"
								hide_line_numbers?={Some(true)}
							/>
						</ui::Window>
						<ui::P>
							{"The CLI automatically transforms the data into features, trains a number of models to predict the target column, and writes the best model to a "}
							<ui::InlineCode>{".tangram"}</ui::InlineCode>
							{" file. We can use this file to make predictions from our code."}
						</ui::P>
					</ui::S2>
					<div class="docs-prev-next-buttons">
						<div></div>
						<ui::Link href="predict/">
							{"Next: Make a Prediction. >"}
						</ui::Link>
					</div>
				</ui::S1>
			</DocsLayout>
		</Document>
	}
}
