use pinwheel::prelude::*;
use tangram_ui as ui;

pub struct Predict;

impl Component for Predict {
	fn into_node(self) -> Node {
		let elixir = ui::doc!(
			r#"
				model = Tangram.load_model_from_path("./heart_disease.tangram")

				output = Tangram.predict(model, %{
					:age =>    63,
					:gender => "male",
					# ...
				})
			"#
		)
		.into();
		let go = ui::doc!(
			r#"
				import "github.com/tangramxyz/tangram/languages/go"

				model, _ := tangram.LoadModelFromPath("./heart_disease.tangram", nil)

				output := model.PredictOne(tangram.Input{
					"age":    63,
					"gender": "male",
					// ...
				}, nil)
			"#
		)
		.into();
		let javascript = ui::doc!(
			r#"
				const tangram = require("@tangramxyz/tangram");

				const model = new tangram.Model("./heart_disease.tangram");

				const output = model.predict({
					age: 63,
					gender: "male",
					// ...
				});
			"#
		)
		.into();
		let python = ui::doc!(
			r#"
				import tangram

				model = tangram.Model.from_path('./census.tangram')

				output = model.predict({
					'age': 63,
					'gender': 'male',
					# ...
				})
			"#
		)
		.into();
		let ruby = ui::doc!(
			r#"
				require 'tangram'

				model = Tangram::Model.from_path('./heart_disease.tangram')

				output = model.predict({
					age: 63,
					gender: 'male',
					# ...
				})
			"#
		)
		.into();
		let rust = ui::doc!(
			r#"
				let model: tangram::Model =
				tangram::Model::from_path("./heart_disease.tangram", None).unwrap();

				let input = tangram::predict_input! {
					"age": 63.0,
					"gender": "male",
					// ...
				};

				let output = model.predict_one(input, None);
			"#
		)
		.into();
		let code_for_language = ui::highlight_code_for_language(ui::CodeForLanguage {
			elixir,
			go,
			javascript,
			python,
			ruby,
			rust,
		});
		let title = div()
			.class("index-step-title")
			.child("Make predictions in your favorite language.");
		let p1 = div()
			.class("index-step-text")
			.child("Make predictions with libraries for ")
			.child(
				ui::Link::new()
					.href("https://hex.pm/packages/tangram".to_owned())
					.title("Elixir".to_owned())
					.child("Elixir"),
			)
			.child(", ")
			.child(
				ui::Link::new()
					.href("https://pkg.go.dev/github.com/tangramxyz/tangram-go".to_owned())
					.title("Go".to_owned())
					.child("Go"),
			)
			.child(", ")
			.child(
				ui::Link::new()
					.href("https://www.npmjs.com/package/@tangramxyz/tangram".to_owned())
					.title("JavaScript".to_owned())
					.child("JavaScript"),
			)
			.child(", ")
			.child(
				ui::Link::new()
					.href("https://pypi.org/project/tangram".to_owned())
					.title("Python".to_owned())
					.child("Python"),
			)
			.child(", ")
			.child(
				ui::Link::new()
					.href("https://rubygems.org/gems/tangram".to_owned())
					.title("Ruby".to_owned())
					.child("Ruby"),
			)
			.child(", and ")
			.child(
				ui::Link::new()
					.href("https://lib.rs/tangram".to_owned())
					.title("Rust".to_owned())
					.child("Rust"),
			)
			.child(".");
		let p2 = div().class("index-step-text").child("Tangram is written in Rust and exposed to each langauge via native extensions, so predictions are fast and your data never travels over the network.");
		let left = div().child(title).child(p1).child(br()).child(p2);
		let right =
			ui::Window::new().child(ui::CodeSelect::new(code_for_language).line_numbers(true));
		div()
			.class("index-step")
			.child(left)
			.child(right)
			.into_node()
	}
}
