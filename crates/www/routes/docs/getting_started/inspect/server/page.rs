use modelfox_ui as ui;
use modelfox_www_docs_inspect_common::{ThresholdMetrics, Tuning};
use modelfox_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage, PrevNextButtons},
	document::Document,
};
use pinwheel::prelude::*;

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let threshold_metrics = vec![
			ThresholdMetrics {
				accuracy: 0.5696,
				f1_score: 0.5294,
				false_negatives: 17,
				false_positives: 2786,
				precision: 0.3614,
				recall: 0.9893,
				threshold: 0.05,
				true_negatives: 2133,
				true_positives: 1577,
			},
			ThresholdMetrics {
				accuracy: 0.7005,
				f1_score: 0.6121,
				false_negatives: 55,
				false_positives: 1895,
				precision: 0.4481,
				recall: 0.9654,
				threshold: 0.1,
				true_negatives: 3024,
				true_positives: 1539,
			},
			ThresholdMetrics {
				accuracy: 0.7595,
				f1_score: 0.6577,
				false_negatives: 89,
				false_positives: 1477,
				precision: 0.5046,
				recall: 0.9441,
				threshold: 0.15,
				true_negatives: 3442,
				true_positives: 1505,
			},
			ThresholdMetrics {
				accuracy: 0.8040,
				f1_score: 0.6920,
				false_negatives: 160,
				false_positives: 1116,
				precision: 0.5623,
				recall: 0.8996,
				threshold: 0.2,
				true_negatives: 3803,
				true_positives: 1434,
			},
			ThresholdMetrics {
				accuracy: 0.8238,
				f1_score: 0.7026,
				false_negatives: 239,
				false_positives: 908,
				precision: 0.5987,
				recall: 0.8500,
				threshold: 0.25,
				true_negatives: 4011,
				true_positives: 1355,
			},
			ThresholdMetrics {
				accuracy: 0.8430,
				f1_score: 0.7068,
				false_negatives: 362,
				false_positives: 660,
				precision: 0.6511,
				recall: 0.7728,
				threshold: 0.3,
				true_negatives: 4259,
				true_positives: 1232,
			},
			ThresholdMetrics {
				accuracy: 0.8486,
				f1_score: 0.6943,
				false_negatives: 474,
				false_positives: 512,
				precision: 0.6863,
				recall: 0.7026,
				threshold: 0.35,
				true_negatives: 4407,
				true_positives: 1120,
			},
			ThresholdMetrics {
				accuracy: 0.8542,
				f1_score: 0.6903,
				false_negatives: 536,
				false_positives: 413,
				precision: 0.7192,
				recall: 0.6637,
				threshold: 0.4,
				true_negatives: 4506,
				true_positives: 1058,
			},
			ThresholdMetrics {
				accuracy: 0.8558,
				f1_score: 0.6749,
				false_negatives: 619,
				false_positives: 320,
				precision: 0.7528,
				recall: 0.6116,
				threshold: 0.45,
				true_negatives: 4599,
				true_positives: 975,
			},
			ThresholdMetrics {
				accuracy: 0.8567,
				f1_score: 0.6591,
				false_negatives: 692,
				false_positives: 241,
				precision: 0.7891,
				recall: 0.5658,
				threshold: 0.5,
				true_negatives: 4678,
				true_positives: 902,
			},
			ThresholdMetrics {
				accuracy: 0.8567,
				f1_score: 0.6467,
				false_negatives: 740,
				false_positives: 193,
				precision: 0.8156,
				recall: 0.5357,
				threshold: 0.55,
				true_negatives: 4726,
				true_positives: 854,
			},
			ThresholdMetrics {
				accuracy: 0.8565,
				f1_score: 0.6311,
				false_negatives: 795,
				false_positives: 139,
				precision: 0.8518,
				recall: 0.5012,
				threshold: 0.6,
				true_negatives: 4780,
				true_positives: 799,
			},
			ThresholdMetrics {
				accuracy: 0.8486,
				f1_score: 0.5881,
				false_negatives: 890,
				false_positives: 96,
				precision: 0.8799,
				recall: 0.4416,
				threshold: 0.65,
				true_negatives: 4823,
				true_positives: 704,
			},
			ThresholdMetrics {
				accuracy: 0.8401,
				f1_score: 0.5383,
				false_negatives: 987,
				false_positives: 54,
				precision: 0.9183,
				recall: 0.3808,
				threshold: 0.7,
				true_negatives: 4865,
				true_positives: 607,
			},
			ThresholdMetrics {
				accuracy: 0.8289,
				f1_score: 0.4745,
				false_negatives: 1091,
				false_positives: 23,
				precision: 0.9562,
				recall: 0.3155,
				threshold: 0.75,
				true_negatives: 4896,
				true_positives: 503,
			},
			ThresholdMetrics {
				accuracy: 0.8166,
				f1_score: 0.4059,
				false_negatives: 1186,
				false_positives: 8,
				precision: 0.9807,
				recall: 0.2559,
				threshold: 0.8,
				true_negatives: 4911,
				true_positives: 408,
			},
			ThresholdMetrics {
				accuracy: 0.8120,
				f1_score: 0.3793,
				false_negatives: 1220,
				false_positives: 4,
				precision: 0.9894,
				recall: 0.2346,
				threshold: 0.85,
				true_negatives: 4915,
				true_positives: 374,
			},
			ThresholdMetrics {
				accuracy: 0.8051,
				f1_score: 0.3407,
				false_negatives: 1266,
				false_positives: 3,
				precision: 0.9909,
				recall: 0.2057,
				threshold: 0.9,
				true_negatives: 4916,
				true_positives: 328,
			},
			ThresholdMetrics {
				accuracy: 0.7557,
				f1_score: 0.0037,
				false_negatives: 1591,
				false_positives: 0,
				precision: 1.0,
				recall: 0.0018,
				threshold: 0.95,
				true_negatives: 4919,
				true_positives: 3,
			},
		];
		let m1 = ui::Markdown::new(ui::doc!(
			r#"
				We can learn more about our model with the modelfox app. Run `modelfox app` and open your browser to http://localhost:8080, or use the cloud hosted app at https://app.modelfox.dev.

				Click the "Create Repo" button to create a new repo. Repos allow you to manage and compare multiple versions of the same model, just like git repos hold multiple versions of the same codebase. Click "Upload Model" to upload the first version of your model.

				Click Training Metrics in the sidebar and have a look at the confusion matrix.
			"#
		));
		let m2 = ui::Markdown::new(ui::doc!(
			r#"
				It looks like false negatives are a bit high. This means we are predicting people are healthy when they actually aren't. It would be better if the model had fewer false negatives, even if it means more false positives, because doctors can rule out heart disease with further testing. Let's make that change by going to the Tuning page. Drag the tuning slider to see how different thresholds affect precision and recall.
			"#
		));
		let m3 = ui::Markdown::new(ui::doc!(
			r#"
				When we lower the threhold, we predict that more people have heart disease which results in lower precision but higher recall. Once you've chosen a threshold, you can update your prediction code to use it.
			"#
		));
		let prev_next_buttons = PrevNextButtons::new()
			.prev("predict/", "Make a prediction.")
			.next("monitor", "Monitor your model in production.");
		let content = ui::S1::new()
			.child(ui::H1::new("Inspect"))
			.child(
				ui::S2::new()
					.child(m1)
					.child(TrainingMetrics)
					.child(m2)
					.child(
						ui::Window::new()
							.child(Dehydrate::new("tuning", Tuning { threshold_metrics })),
					)
					.child(m3)
					.child(TuningCode),
			)
			.child(prev_next_buttons);
		let layout = DocsLayout::new()
			.selected_page(DocsPage::GettingStarted(GettingStartedPage::Inspect))
			.child(content);
		Document::new()
			.client("modelfox_www_docs_getting_started_inspect_client")
			.child(layout)
			.into_node()
	}
}

pub struct TrainingMetrics;

impl Component for TrainingMetrics {
	fn into_node(self) -> Node {
		ui::Window::new()
			.child(ui::S1::new().child(ui::H1::new("Training Metrics")).child(
				ui::ConfusionMatrix {
					class_label: "positive".to_owned(),
					false_negatives: Some(20),
					false_positives: Some(19),
					true_negatives: Some(299),
					true_positives: Some(400),
				},
			))
			.into_node()
	}
}

pub struct TuningCode;

impl Component for TuningCode {
	fn into_node(self) -> Node {
		let code_for_language = ui::highlight_code_for_language(ui::CodeForLanguage {
			elixir: ui::doc!(
				r#"
					predict_options = %ModelFox.PredictOptions{
						threshold: 0.5,
						compute_feature_contributions: false
					}
					output = ModelFox.predict(model, input, predict_options)
				"#
			)
			.into(),
			go: ui::doc!(
				r#"
					predictOptions := modelfox.PredictOptions{
						Threshold:                   0.5,
						ComputeFeatureContributions: false,
					}
					output := model.PredictOne(input, &predictOptions)
				"#
			)
			.into(),
			javascript: ui::doc!(
				r#"
					options = {
						threshold: 0.5,
						computeFeatureContributions: true
					}
					let output = model.predict(input, options)
				"#
			)
			.into(),
			php: ui::doc!(
				r#"
					$options = new \modelfox::PredictOptions('true', 0.5);
					$output = model->predict($input, $options);
				"#
			)
			.into(),
			python: ui::doc!(
				r#"
					predict_options = modelfox.PredictOptions(
							threshold=0.5,
							compute_feature_contributions=True
					)
					output = model.predict(input, predict_options)
				"#
			)
			.into(),
			ruby: ui::doc!(
				r#"
					options = ModelFox::PredictOptions.new(
						threshold: 0.5,
						compute_feature_contributions: true
					)
					output = model.predict(input, options: options)
				"#
			)
			.into(),
			rust: ui::doc!(
				r#"
					let options = modelfox::PredictOptions {
						threshold: Some(0.5),
						compute_feature_contributions: Some(true),
					};
					let output = model.predict_one(input.clone(), Some(options.clone()));
				"#
			)
			.into(),
		});
		ui::Window::new()
			.child(ui::CodeSelect::new(code_for_language))
			.into_node()
	}
}
