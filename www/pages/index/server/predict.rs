use html::{component, html};
use indoc::indoc;
use tangram_ui as ui;

#[component]
pub fn Predict() {
	let elixir = indoc! {
		r#"
			model = Tangram.load_model_from_path("./heart_disease.tangram")

			output = Tangram.predict(model, %{
				:age =>    63,
				:gender => "male",
				# ...
			})
		"#
	}
	.into();
	let go = indoc! {
		r#"
			import "github.com/tangramxyz/tangram/languages/go"

			model, _ := tangram.LoadModelFromPath("./heart_disease.tangram", nil)

			output := model.PredictOne(tangram.Input{
				"age":    63,
				"gender": "male",
				// ...
			}, nil)
		"#
	}
	.into();
	let javascript = indoc! {
		r#"
			const tangram = require("@tangramxyz/tangram");

			const model = new tangram.Model("./heart_disease.tangram");

			const output = model.predictSync({
				age: 63,
				gender: "male",
				// ...
			});
		"#
	}
	.into();
	let python = indoc! {
		r#"
			import tangram

			model = tangram.Model.from_path('./census.tangram')

			output = model.predict({
				'age': 63,
				'gender': 'male',
				# ...
			})
		"#
	}
	.into();
	let ruby = indoc! {
		r#"
			require 'tangram'

			model = Tangram::Model.from_path('./heart_disease.tangram')

			output = model.predict({
				age: 63,
				gender: 'male',
				# ...
			})
		"#
	}
	.into();
	let rust = indoc! {
		r#"
		use tangram_rust as tangram;

		let model: tangram::Model =
		tangram::Model::from_path("./heart_disease.tangram", None).unwrap();

		let input = tangram::predict_input! {
			"age": 63.0,
			"gender": "male",
			// ...
		};

		let output = model.predict_one(input, None);
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
				<div class="index-step-title">{"Make predictions in your favorite language."}</div>
				<div class="index-step-text">
					{"Make predictions with libraries for "}
					<ui::Link
						href="https://hex.pm/packages/tangram"
						title?="Elixir"
					>
						{"Elixir"}
					</ui::Link>
					{", " }
					<ui::Link
						href="https://pkg.go.dev/github.com/tangramxyz/tangram-go"
						title?="Go"
					>
						{"Go"}
					</ui::Link>
					{", " }
					<ui::Link
						href="https://www.npmjs.com/package/@tangramxyz/tangram-node"
						title?="Node.js"
					>
						{"Node.js"}
					</ui::Link>
					{", " }
					<ui::Link
						href="https://pypi.org/project/tangram"
						title?="Python"
					>
						{"Python"}
					</ui::Link>
					{", " }
					<ui::Link
						href="https://rubygems.org/gems/tangram"
						title?="Ruby"
					>
						{"Ruby"}
					</ui::Link>
					{", and "}
					<ui::Link
						href="https://lib.rs/tangram_rust"
						title?="Rust"
					>
						{"Rust"}
					</ui::Link>
					{"."}
				</div>
				<br/>
				<div class="index-step-text">
					{"Tangram is written in Rust and exposed to each langauge via native extensions, so predictions are fast and your data never travels over the network."}
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
