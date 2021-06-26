use pinwheel::prelude::*;
use std::borrow::Cow;
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage, PredictPage},
	document::Document,
};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let predict_text = ui::P::new().child("First, import the tangram library and load the model file. Then, make an object with info for a new patient that matches the CSV, excluding the diagnosis column. Finally, call predict and print out the result.");
		Document::new()
			.child(
				DocsLayout::new(
					DocsPage::GettingStarted(GettingStartedPage::Predict(PredictPage::Rust)),
					None,
				)
				.child(
					ui::S1::new()
						.child(ui::H1::new().child("Predict with Rust"))
						.child(
							ui::S2::new()
								.child(ui::H2::new().child("1. Install."))
								.child(Install)
								.child(ui::H2::new().child("2. Predict."))
								.child(predict_text)
								.child(Predict),
						)
						.child(
							div()
								.class("docs-prev-next-buttons")
								.child(
									ui::Link::new()
										.href("../train".to_owned())
										.child("< Previous: Train a model."),
								)
								.child(
									ui::Link::new()
										.href("../inspect".to_owned())
										.child("Next: Inspect your model. >"),
								),
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
				[dependencies]
				tangram = { version = "*" }
			"#
		);
		let code = ui::highlight(code, ui::Language::Rust);
		ui::Window::new()
			.child(
				ui::Code::new()
					.code(Cow::Owned(code))
					.hide_line_numbers(Some(true)),
			)
			.into_node()
	}
}

pub struct Predict;

impl Component for Predict {
	fn into_node(self) -> Node {
		let code = ui::doc!(
			r#"
				fn main() {
					// Load the model from the path.
					let model: tangram::Model =
						tangram::Model::from_path("examples/heart_disease.tangram", None).unwrap();

					// Create an example input matching the schema of the CSV file the model was trained on.
					// Here the data is just hard-coded, but in your application you will probably get this
					// from a database or user input.
					let input = tangram::predict_input! {
						"age": 63.0,
						"gender": "male",
						"chest_pain": "typical angina",
						"resting_blood_pressure": 145.0,
						"cholesterol": 233.0,
						"fasting_blood_sugar_greater_than_120": "true",
						"resting_ecg_result": "probable or definite left ventricular hypertrophy",
						"exercise_max_heart_rate": 150.0,
						"exercise_induced_angina": "no",
						"exercise_st_depression": 2.3,
						"exercise_st_slope": "downsloping",
						"fluoroscopy_vessels_colored": 0.0,
						"thallium_stress_test": "fixed defect",
					};

					// Make the prediction!
					let output = model.predict_one(input, None);

					// Print the output.
					println!("Output: {:?}", output);
				}
			"#
		);
		let code = ui::highlight(code, ui::Language::Rust);
		ui::Window::new()
			.child(ui::Code::new().code(Cow::Owned(code)))
			.into_node()
	}
}
