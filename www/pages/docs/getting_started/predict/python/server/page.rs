use html::{component, html};
use indoc::indoc;
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
			<DocsLayout selected_page={DocsPage::GettingStarted(GettingStartedPage::Predict(PredictPage::Python))} headings={None}>
				<ui::S1>
					<ui::H1>{"Predict with Python"}</ui::H1>
					<ui::S2>
						<Install />
						<Predict />
					</ui::S2>
					<div class="docs-prev-next-buttons">
						<ui::Link href="../train">
							{"< Previous: Train a model."}
						</ui::Link>
						<ui::Link href="../inspect">
							{"Next: Inspect your model. >"}
						</ui::Link>
					</div>
				</ui::S1>
			</DocsLayout>
		</Document>
	}
}

#[component]
pub fn Install() {
	let code = indoc! {
		r#"
			pip install tangram
		"#
	};
	let code = ui::highlight(code, ui::Language::Elixir);
	html! {
		<ui::Window padding={Some(true)}>
			<ui::Code
				code={code}
				hide_line_numbers?={Some(true)}
			/>
		</ui::Window>
	}
}

#[component]
pub fn Predict() {
	let code = indoc! {
		r#"
			import os
			import tangram

			# Get the path to the .tangram file.
			model_path = os.path.join(os.path.dirname(__file__), 'heart_disease.tangram')
			# Load the model from the path.
			model = tangram.Model.from_path(model_path)

			# Create an example input matching the schema of the CSV file the model was trained on.
			# Here the data is just hard-coded, but in your application you will probably get this
			# from a database or user input.
			input = {
				'age': 63,
				'gender': 'male',
				'chest_pain': 'typical angina',
				'resting_blood_pressure': 145,
				'cholesterol': 233,
				'fasting_blood_sugar_greater_than_120': 'true',
				'resting_ecg_result': 'probable or definite left ventricular hypertrophy',
				'exercise_max_heart_rate': 150,
				'exercise_induced_angina': 'no',
				'exercise_st_depression': 2.3,
				'exercise_st_slope': 'downsloping',
				'fluoroscopy_vessels_colored': '0',
				'thallium_stress_test': 'fixed defect',
			}

			# Make the prediction!
			output = model.predict(input)

			# Print the output.
			print('Output:', output)
		"#
	};
	let code = ui::highlight(code, ui::Language::Python);
	html! {
		<ui::Window padding={Some(true)}>
			<ui::Code
				code={code}
			/>
		</ui::Window>
	}
}
