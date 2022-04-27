use modelfox_ui as ui;
use pinwheel::prelude::*;

pub struct Predict;

impl Component for Predict {
	fn into_node(self) -> Node {
		let elixir = ui::doc!(
			r#"
				model = ModelFox.load_model_from_path("./heart_disease.modelfox")

				output = ModelFox.predict(model, %{
					:age =>    63,
					:gender => "male",
					# ...
				})
			"#
		)
		.into();
		let go = ui::doc!(
			r#"
				import "github.com/modelfoxdotdev/modelfox/languages/go"

				model, _ := modelfox.LoadModelFromPath("./heart_disease.modelfox", nil)

				output := model.PredictOne(modelfox.Input{
					"age":    63,
					"gender": "male",
					// ...
				}, nil)
			"#
		)
		.into();
		let javascript = ui::doc!(
			r#"
				const modelfox = require("@modelfoxdotdev/modelfox");

				const model = new modelfox.Model("./heart_disease.modelfox");

				const output = model.predict({
					age: 63,
					gender: "male",
					// ...
				});
			"#
		)
		.into();
		let php = ui::doc!(
			r#"
				<?php

				namespace modelfox\modelfox;

				require_once(dirname(dirname(__FILE__)) . '/vendor/autoload.php');

				$model_path = dirname(dirname(__FILE__)) . '/heart_disease.modelfox';
				$model = Model::from_path($model_path);

				$input = [
					'age' => 63.0,
					'gender' => 'male',
					// ..
				];

				$output = $model->predict($input);
			"#
		)
		.into();
		let python = ui::doc!(
			r#"
				import modelfox

				model = modelfox.Model.from_path('./census.modelfox')

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
				require 'modelfox'

				model = ModelFox::Model.from_path('./heart_disease.modelfox')

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
				let model: modelfox::Model =
				modelfox::Model::from_path("./heart_disease.modelfox", None).unwrap();

				let input = modelfox::predict_input! {
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
			php,
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
					.href("https://hex.pm/packages/modelfox".to_owned())
					.title("Elixir".to_owned())
					.child("Elixir"),
			)
			.child(", ")
			.child(
				ui::Link::new()
					.href("https://pkg.go.dev/github.com/modelfoxdotdev/modelfox-go".to_owned())
					.title("Golang".to_owned())
					.child("Golang"),
			)
			.child(", ")
			.child(
				ui::Link::new()
					.href("https://www.npmjs.com/package/@modelfoxdotdev/modelfox".to_owned())
					.title("JavaScript".to_owned())
					.child("JavaScript"),
			)
			.child(", ")
			.child(
				ui::Link::new()
					.href("https://packagist.org/packages/modelfox/modelfox".to_owned())
					.title("PHP".to_owned())
					.child("PHP"),
			)
			.child(", ")
			.child(
				ui::Link::new()
					.href("https://pypi.org/project/modelfox".to_owned())
					.title("Python".to_owned())
					.child("Python"),
			)
			.child(", ")
			.child(
				ui::Link::new()
					.href("https://rubygems.org/gems/modelfox".to_owned())
					.title("Ruby".to_owned())
					.child("Ruby"),
			)
			.child(", and ")
			.child(
				ui::Link::new()
					.href("https://lib.rs/modelfox".to_owned())
					.title("Rust".to_owned())
					.child("Rust"),
			)
			.child(".");
		let p2 = div().class("index-step-text").child("ModelFox is written in Rust and exposed to each language via native extensions, so predictions are fast and your data never travels over the network.");
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
