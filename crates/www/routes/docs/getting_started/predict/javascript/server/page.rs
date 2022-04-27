use modelfox_ui as ui;
use modelfox_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage, PredictPage, PrevNextButtons},
	document::Document,
};
use pinwheel::prelude::*;
use std::borrow::Cow;

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let predict_text = ui::P::new().child("First, import the modelfox library and load the model file. Then, make an object with info for a new patient that matches the CSV, excluding the diagnosis column. Finally, call predict and print out the result.");
		Document::new()
			.child(
				DocsLayout::new()
					.selected_page(DocsPage::GettingStarted(GettingStartedPage::Predict(
						PredictPage::Node,
					)))
					.child(
						ui::S1::new()
							.child(ui::H1::new("Predict with JavaScript"))
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
				dependencies: {
					"@modelfoxdotdev/modelfox": "*",
				}
			"#
		);
		let code = ui::highlight(code, ui::Language::Elixir);
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
				const fs = require("fs");
				const path = require("path");
				const modelfox = require("@modelfoxdotdev/modelfox");

				// Get the path to the .modelfox file.
				const modelPath = path.join(__dirname, "heart_disease.modelfox");
				// Load the model from the path.
				const modelData = fs.readFileSync(modelPath);
				const model = new modelfox.Model(modelData.buffer);

				// Create an example input matching the schema of the CSV file the model was trained on.
				// Here the data is just hard-coded, but in your application you will probably get this
				// from a database or user input.
				const input = {
					age: 63,
					gender: "male",
					chest_pain: "typical angina",
					resting_blood_pressure: 145,
					cholesterol: 233,
					fasting_blood_sugar_greater_than_120: "true",
					resting_ecg_result: "probable or definite left ventricular hypertrophy",
					exercise_max_heart_rate: 150,
					exercise_induced_angina: "no",
					exercise_st_depression: 2.3,
					exercise_st_slope: "downsloping",
					fluoroscopy_vessels_colored: "0",
					thallium_stress_test: "fixed defect",
				};

				// Make the prediction!
				const output = model.predict(input);

				// Print the output.
				console.log("Output:", output);
			"#
		);
		let code = ui::highlight(code, ui::Language::Javascript);
		ui::Window::new()
			.child(ui::Code::new().code(Cow::Owned(code)).line_numbers(true))
			.into_node()
	}
}
