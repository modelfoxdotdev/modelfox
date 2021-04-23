use html::{component, html};
use indoc::indoc;
use tangram_ui as ui;

#[component]
pub fn Monitoring() {
	let elixir = indoc! {
		r#"
			# Log the prediction.
			Tangram.log_prediction(model, %Tangram.LogPredictionArgs{
				identifier: "John Doe",
				options: predict_options,
				input: input,
				output: output,
			})

			# Later on, if we get an official diagnosis for the patient, log the true value.
			Tangram.log_true_value(model, %Tangram.LogTrueValueArgs{
				identifier: "John Doe",
				true_value: "Positive",
			})
		"#
	}
	.into();
	let go = indoc! {
		r#"
			// Log the prediction.
			err = model.LogPrediction(tangram.LogPredictionArgs{
				Identifier: "John Doe",
				Input:      input,
				Options:    predictOptions,
				Output:     output,
			})
			if err != nil {
				log.Fatal(err)
			}

			// Later on, if we get an official diagnosis for the patient, log the true value.
			err = model.LogTrueValue(tangram.LogTrueValueArgs{
				Identifier: "John Doe",
				TrueValue:  "Positive",
			})
			if err != nil {
				log.Fatal(err)
			}
		"#
	}
	.into();
	let javascript = indoc! {
		r#"
			// Log the prediction.
			model.logPrediction({
				identifier: "6c955d4f-be61-4ca7-bba9-8fe32d03f801",
				input,
				options,
				output,
			})

			// Later on, if we get an official diagnosis for the patient, log the true value.
			model.logTrueValue({
				identifier: "6c955d4f-be61-4ca7-bba9-8fe32d03f801",
				trueValue: "Positive",
			})
		"#
	}
	.into();
	let python = indoc! {
		r#"
			# Log the prediction.
			model.log_prediction(
					identifier="John Doe",
					input=input,
					output=output,
					options=predict_options,
			)

			# Later on, if we get an official diagnosis for the patient, log the true value.
			model.log_true_value(
					identifier="John Doe",
					true_value="Positive",
			)
		"#
	}
	.into();
	let ruby = indoc! {
		r#"
			# Log the prediction.
			model.log_prediction(
				identifier: 'John Doe',
				input: input,
				output: output,
				options: options
			)

			# Later on, if we get an official diagnosis for the patient, log the true value.
			model.log_true_value(
				identifier: 'John Doe',
				true_value: 'Positive'
			)
		"#
	}
	.into();
	let rust = indoc! {
		r#"
			// Log the prediction.
			model.log_prediction(tangram::LogPredictionArgs {
				identifier: "John Doe".into(),
				input,
				options: Some(options),
				output,
			})?;

			// Later on, if we get an official diagnosis for the patient, log the true value.
			model.log_true_value(tangram::LogTrueValueArgs {
				identifier: "John Doe".into(),
				true_value: "Positive".into(),
			})?;
		"#
	}
	.into();
	let code_for_language = ui::highlight_code_for_language(ui::CodeForLanguage {
		elixir,
		go,
		javascript,
		python,
		ruby,
		rust,
	});
	html! {
		<div class="index-step">
			<div>
				<div class="index-step-title">{"Monitor your models in production."}</div>
				<div class="index-step-text">
					{"Once your model is deployed, make sure that it performs as well in production as it did in training."}
				</div>
				<br />
				<div class="index-step-text">
					{"Opt in to logging by calling "}
					<ui::InlineCode>{"logPrediction"}</ui::InlineCode>
					{"."}
				</div>
				<br />
				<div class="index-step-text">
					{"Later on, if you find out the true value for a prediction, call "}
					<ui::InlineCode>{"logTrueValue"}</ui::InlineCode>
					{"."}
				</div>
			</div>
			<ui::Window padding={Some(true)}>
				<ui::CodeSelect
					id="prediction"
					code_for_language={code_for_language}
					hide_line_numbers?={Some(false)}
				/>
			</ui::Window>
		</div>
	}
}
