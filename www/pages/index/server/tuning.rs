use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_index_common::{self as common, tuning::ThresholdMetrics};

pub struct Tuning;

impl Component for Tuning {
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
		let title = div()
			.class("index-step-title")
			.child("Tune your model to get the best performance.");
		let text = ui::Markdown::new(ui::doc!(
			r#"
				Tune binary classification models to your preferred tradeoff between precision and recall. To use your selected threshold, update the `predict` call in your code.
			"#
		));
		let left = div().child(title).child(text);
		let right = ui::Window::new().child(Dehydrate::new(
			"tuning",
			common::tuning::Tuning { threshold_metrics },
		));
		div()
			.class("index-step")
			.child(left)
			.child(right)
			.into_node()
	}
}
