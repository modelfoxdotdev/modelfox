use num::ToPrimitive;
use tangram_app_common::monitor_event::NumberOrString;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct BinaryClassificationProductionPredictionMetrics {
	negative_class: String,
	positive_class: String,
	confusion_matrix: BinaryConfusionMatrix,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct BinaryConfusionMatrix {
	false_negatives: u64,
	false_positives: u64,
	true_negatives: u64,
	true_positives: u64,
}

impl BinaryConfusionMatrix {
	fn new() -> BinaryConfusionMatrix {
		BinaryConfusionMatrix {
			false_negatives: 0,
			false_positives: 0,
			true_negatives: 0,
			true_positives: 0,
		}
	}
	fn n_examples(&self) -> u64 {
		self.false_positives + self.false_negatives + self.true_positives + self.true_negatives
	}
}

#[derive(Debug, serde::Deserialize)]
pub struct BinaryClassificationProductionPredictionMetricsOutput {
	pub accuracy: f32,
	pub f1_score: f32,
	pub false_negatives: u64,
	pub false_positives: u64,
	pub precision: f32,
	pub recall: f32,
	pub true_negatives: u64,
	pub true_positives: u64,
}

impl BinaryClassificationProductionPredictionMetrics {
	pub fn new(
		negative_class: String,
		positive_class: String,
	) -> BinaryClassificationProductionPredictionMetrics {
		let confusion_matrix = BinaryConfusionMatrix::new();
		BinaryClassificationProductionPredictionMetrics {
			negative_class,
			positive_class,
			confusion_matrix,
		}
	}

	pub fn update(&mut self, value: (NumberOrString, NumberOrString)) {
		let label = match value.1 {
			NumberOrString::Number(_) => return,
			NumberOrString::String(label) => label,
		};
		let prediction = match value.0 {
			NumberOrString::Number(_) => return,
			NumberOrString::String(label) => label,
		};
		let confusion_matrix = &mut self.confusion_matrix;
		let predicted = prediction == self.positive_class;
		let actual = label == self.positive_class;
		match (predicted, actual) {
			(false, false) => {
				confusion_matrix.true_negatives += 1;
			}
			(false, true) => {
				confusion_matrix.false_negatives += 1;
			}
			(true, false) => {
				confusion_matrix.false_positives += 1;
			}
			(true, true) => {
				confusion_matrix.true_positives += 1;
			}
		}
	}

	pub fn merge(&mut self, other: BinaryClassificationProductionPredictionMetrics) {
		self.confusion_matrix.false_negatives += other.confusion_matrix.false_negatives;
		self.confusion_matrix.false_positives += other.confusion_matrix.false_positives;
		self.confusion_matrix.true_negatives += other.confusion_matrix.true_negatives;
		self.confusion_matrix.true_positives += other.confusion_matrix.true_positives;
	}

	pub fn finalize(self) -> Option<BinaryClassificationProductionPredictionMetricsOutput> {
		let n_examples = self.confusion_matrix.n_examples();
		let true_positives = self.confusion_matrix.true_positives;
		let false_positives = self.confusion_matrix.false_positives;
		let false_negatives = self.confusion_matrix.false_negatives;
		let true_negatives = self.confusion_matrix.true_negatives;
		// This is the fraction of the total predictions that are correct.
		let accuracy =
			(true_positives + true_negatives).to_f32().unwrap() / n_examples.to_f32().unwrap();
		// This is the fraction of the total predictive positive examples that are actually positive.
		let precision =
			true_positives.to_f32().unwrap() / (true_positives + false_positives).to_f32().unwrap();
		// This is the fraction of the total positive examples that are correctly predicted as positive.
		let recall =
			true_positives.to_f32().unwrap() / (true_positives + false_negatives).to_f32().unwrap();
		let f1_score = 2.0 * (precision * recall) / (precision + recall);
		if n_examples == 0 {
			None
		} else {
			Some(BinaryClassificationProductionPredictionMetricsOutput {
				accuracy,
				f1_score,
				false_negatives,
				false_positives,
				precision,
				recall,
				true_negatives,
				true_positives,
			})
		}
	}
}

#[test]
fn test_binary() {
	use tangram_zip::zip;
	let mut metrics =
		BinaryClassificationProductionPredictionMetrics::new("Cat".to_owned(), "Dog".to_owned());
	metrics.update((
		NumberOrString::String("Cat".to_owned()),
		NumberOrString::String("Cat".to_owned()),
	));
	let labels = vec![
		"Cat", "Cat", "Cat", "Cat", "Cat", "Cat", "Cat", "Dog", "Dog", "Dog", "Dog", "Dog",
	];
	let predictions = vec![
		"Cat", "Cat", "Cat", "Cat", "Dog", "Dog", "Dog", "Dog", "Dog", "Dog", "Cat", "Cat",
	];
	for (label, prediction) in zip!(labels, predictions) {
		metrics.update((
			NumberOrString::String(prediction.to_owned()),
			NumberOrString::String(label.to_owned()),
		));
	}
	let metrics = metrics.finalize();
	insta::assert_debug_snapshot!(metrics, @r###"
 Some(
     BinaryClassificationProductionPredictionMetricsOutput {
         accuracy: 0.61538464,
         f1_score: 0.54545456,
         false_negatives: 2,
         false_positives: 3,
         precision: 0.5,
         recall: 0.6,
         true_negatives: 5,
         true_positives: 3,
     },
 )
 "###);
}
