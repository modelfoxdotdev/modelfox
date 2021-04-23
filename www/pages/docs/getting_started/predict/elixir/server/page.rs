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
			<DocsLayout selected_page={DocsPage::GettingStarted(GettingStartedPage::Predict(PredictPage::Elixir))} headings={None}>
				<ui::S1>
					<ui::H1>{"Predict with Elixir"}</ui::H1>
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
	html! {
		<ui::P>
			{"Add the "}
			<ui::InlineCode>{"tangram"}</ui::InlineCode>
			{" package to your "}
			<ui::InlineCode>{"mix.exs"}</ui::InlineCode>
			{"."}
		</ui::P>
	}
}

#[component]
pub fn Predict() {
	let code = indoc! {
		r#"
			# Get the path to the .tangram file.
			# In your application, you will probably want to put your .tangram file in your mix package's `priv`
			# directory and read it like this: `Path.join(:code.priv_dir(:your_app_name), "model.tangram")`.
			model_path = Path.join(Path.dirname(__ENV__.file), "heart_disease.tangram")

			# Load the model from the path.
			model = Tangram.load_model_from_path(model_path)

			# Create an example input matching the schema of the CSV file the model was trained on.
			# Here the data is just hard-coded, but in your application you will probably get this
			# from a database or user input.
			input = %{
				:age => 63.0,
				:gender => "male",
				:chest_pain => "typical angina",
				:resting_blood_pressure => 145.0,
				:cholesterol => 233.0,
				:fasting_blood_sugar_greater_than_120 => "true",
				:resting_ecg_result => "probable or definite left ventricular hypertrophy",
				:exercise_max_heart_rate => 150.0,
				:exercise_induced_angina => "no",
				:exercise_st_depression => 2.3,
				:exercise_st_slope => "downsloping",
				:fluoroscopy_vessels_colored => "0",
				:thallium_stress_test => "fixed defect"
			}

			# Make the prediction!
			output = Tangram.predict(model, input)

			# Print the output.
			IO.write("Output: ")
			IO.inspect(output)
		"#
	};
	let code = ui::highlight(code, ui::Language::Elixir);
	html! {
		<ui::Window padding={Some(true)}>
			<ui::Code
				code={code}
			/>
		</ui::Window>
	}
}
