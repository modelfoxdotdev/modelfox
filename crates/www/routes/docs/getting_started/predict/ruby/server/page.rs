use pinwheel::prelude::*;
use std::borrow::Cow;
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage, PredictPage, PrevNextButtons},
	document::Document,
};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let predict_text = ui::P::new().child("First, import the tangram library and load the model file. Then, make an object with info for a new patient that matches the CSV, excluding the diagnosis column. Finally, call predict and print out the result.");
		Document::new()
			.child(
				DocsLayout::new()
					.selected_page(DocsPage::GettingStarted(GettingStartedPage::Predict(
						PredictPage::Ruby,
					)))
					.child(
						ui::S1::new()
							.child(ui::H1::new("Predict with Ruby"))
							.child(
								ui::S2::new()
									.child(ui::H2::new("1. Install."))
									.child(Install)
									.child(ui::H2::new("2. Predict."))
									.child(predict_text)
									.child(Predict),
							)
							.child(
								PrevNextButtons::new()
									.prev("../train", "Train a model.")
									.next("../inspect", "Inspect your model."),
							),
					),
			)
			.into_node()
	}
}

pub struct Install;

impl Component for Install {
	fn into_node(self) -> Node {
		let code = ui::doc!(
			r#"
				gem install tangram
			"#
		);
		let code = ui::highlight(code, ui::Language::Ruby);
		ui::Window::new()
			.child(ui::Code::new().code(Cow::Owned(code)))
			.into_node()
	}
}

pub struct Predict;

impl Component for Predict {
	fn into_node(self) -> Node {
		let code = ui::doc!(
			r#"
				require 'tangram'

				# Get the path to the .tangram file.
				model_path = File.join(File.dirname(__FILE__), 'heart_disease.tangram')
				# Load the model from the path.
				model = Tangram::Model.from_path(model_path)

				# Create an example input matching the schema of the CSV file the model was trained on.
				# Here the data is just hard-coded, but in your application you will probably get this
				# from a database or user input.
				input = {
					age: 63.0,
					gender: 'male',
					chest_pain: 'typical angina',
					resting_blood_pressure: 145.0,
					cholesterol: 233.0,
					fasting_blood_sugar_greater_than_120: 'true',
					resting_ecg_result: 'probable or definite left ventricular hypertrophy',
					exercise_max_heart_rate: 150.0,
					exercise_induced_angina: 'no',
					exercise_st_depression: 2.3,
					exercise_st_slope: 'downsloping',
					fluoroscopy_vessels_colored: '0',
					thallium_stress_test: 'fixed defect',
				}

				# Make the prediction!
				output = model.predict(input)

				# Print the output.
				puts('Output:', output)
			"#
		);
		let code = ui::highlight(code, ui::Language::Ruby);
		ui::Window::new()
			.child(ui::Code::new().code(Cow::Owned(code)).line_numbers(true))
			.into_node()
	}
}
