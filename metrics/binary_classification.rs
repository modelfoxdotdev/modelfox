use itertools::Itertools;
use num::ToPrimitive;
use std::num::NonZeroUsize;
use tangram_zip::zip;

/// `BinaryClassificationMetrics` computes common metrics used to evaluate binary classifiers at a number of classification thresholds.
pub struct BinaryClassificationMetrics {
	/// This field maps thresholds to the confusion matrix for prediction at that threshold.
	confusion_matrices_for_thresholds: Vec<(f32, BinaryConfusionMatrix)>,
}

#[derive(Clone)]
struct BinaryConfusionMatrix {
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

	fn total(&self) -> u64 {
		self.false_negatives + self.false_positives + self.true_negatives + self.true_positives
	}
}

/// The input to [`BinaryClassificationMetrics`].
pub struct BinaryClassificationMetricsInput<'a> {
	pub probabilities: &'a [f32],
	pub labels: &'a [Option<NonZeroUsize>],
}

/// BinaryClassificationMetrics contains common metrics used to evaluate binary classifiers.
#[derive(Debug, Clone)]
pub struct BinaryClassificationMetricsOutput {
	/// The area under the receiver operating characteristic curve is computed using a fixed number of thresholds equal to `n_thresholds` which is passed to [`BinaryClassificationMetrics::new`].
	pub auc_roc_approx: f32,
	/// This contains metrics specific to each classification threshold.
	pub thresholds: Vec<BinaryClassificationMetricsOutputForThreshold>,
}

/// The output from [`BinaryClassificationMetrics`].
#[derive(Debug, Clone)]
pub struct BinaryClassificationMetricsOutputForThreshold {
	/// The classification threshold.
	pub threshold: f32,
	/// The total number of examples whose label is equal to the positive class that the model predicted as belonging to the positive class.
	pub true_positives: u64,
	/// The total number of examples whose label is equal to the negative class that the model predicted as belonging to the positive class.
	pub false_positives: u64,
	/// The total number of examples whose label is equal to the negative class that the model predicted as belonging to the negative class.
	pub true_negatives: u64,
	/// The total number of examples whose label is equal to the positive class that the model predicted as belonging to the negative class.
	pub false_negatives: u64,
	/// The fraction of examples that were correctly classified.
	pub accuracy: f32,
	/// The precision is the fraction of examples the model predicted as belonging to the positive class whose label is actually the positive class. true_positives / (true_positives + false_positives). See [Precision and Recall](https://en.wikipedia.org/wiki/Precision_and_recall).
	pub precision: Option<f32>,
	/// The recall is the fraction of examples whose label is equal to the positive class that the model predicted as belonging to the positive class. `recall = true_positives / (true_positives + false_negatives)`.
	pub recall: Option<f32>,
	/// The f1 score is the harmonic mean of the precision and the recall. See [F1 Score](https://en.wikipedia.org/wiki/F1_score).
	pub f1_score: Option<f32>,
	/// The true positive rate is the fraction of examples whose label is equal to the positive class that the model predicted as belonging to the positive class. Also known as the recall. See [Sensitivity and Specificity](https://en.wikipedia.org/wiki/Sensitivity_and_specificity).
	pub true_positive_rate: f32,
	/// The false positive rate is the fraction of examples whose label is equal to the negative class that the model falsely predicted as belonging to the positive class. false_positives / (false_positives + true_negatives). See [False Positive Rate](https://en.wikipedia.org/wiki/False_positive_rate)
	pub false_positive_rate: f32,
}

impl BinaryClassificationMetrics {
	/// Create a new `BinaryClassificationMetrics` with the specified number of thresholds. The thresholds will be centered at 0.5 and evenly spaced between 0 and 1 such that 0 and 1 will never be threshold values.
	pub fn new(n_thresholds: usize) -> BinaryClassificationMetrics {
		// The number of thresholds must be odd so that 0.5 is the middle threshold.
		assert!(n_thresholds % 2 == 1);
		let mut confusion_matrices_for_thresholds = vec![(0.0, BinaryConfusionMatrix::new())];
		(0..n_thresholds)
			.map(|i| (i + 1).to_f32().unwrap() * (1.0 / (n_thresholds.to_f32().unwrap() + 1.0)))
			.for_each(|threshold| {
				confusion_matrices_for_thresholds.push((threshold, BinaryConfusionMatrix::new()))
			});
		// Push a dummy threshold so there is a (0, 0) point on the roc curve
		confusion_matrices_for_thresholds.push((2.0, BinaryConfusionMatrix::new()));
		BinaryClassificationMetrics {
			confusion_matrices_for_thresholds,
		}
	}

	pub fn update(&mut self, input: BinaryClassificationMetricsInput) {
		for (threshold, confusion_matrix) in self.confusion_matrices_for_thresholds.iter_mut() {
			for (probability, label) in zip!(input.probabilities.iter(), input.labels.iter()) {
				let predicted = *probability >= *threshold;
				let actual = label.unwrap().get() == 2;
				match (predicted, actual) {
					(false, false) => confusion_matrix.true_negatives += 1,
					(false, true) => confusion_matrix.false_negatives += 1,
					(true, false) => confusion_matrix.false_positives += 1,
					(true, true) => confusion_matrix.true_positives += 1,
				};
			}
		}
	}

	pub fn merge(&mut self, other: BinaryClassificationMetrics) {
		for ((_, confusion_matrix_a), (_, confusion_matrix_b)) in zip!(
			self.confusion_matrices_for_thresholds.iter_mut(),
			other.confusion_matrices_for_thresholds.iter()
		) {
			confusion_matrix_a.true_positives += confusion_matrix_b.true_positives;
			confusion_matrix_a.false_negatives += confusion_matrix_b.false_negatives;
			confusion_matrix_a.true_negatives += confusion_matrix_b.true_negatives;
			confusion_matrix_a.false_positives += confusion_matrix_b.false_positives;
		}
	}

	pub fn finalize(self) -> BinaryClassificationMetricsOutput {
		// Compute the metrics output for each threshold.
		let thresholds: Vec<_> = self
			.confusion_matrices_for_thresholds
			.iter()
			.map(|(threshold, confusion_matrix)| {
				let n_examples = confusion_matrix.total();
				let true_positives = confusion_matrix.true_positives;
				let false_positives = confusion_matrix.false_positives;
				let false_negatives = confusion_matrix.false_negatives;
				let true_negatives = confusion_matrix.true_negatives;
				// This is the fraction of the total predictions that are correct.
				let accuracy = (true_positives + true_negatives).to_f32().unwrap()
					/ n_examples.to_f32().unwrap();
				// This is the fraction of the total predictive positive examples that are actually positive.
				let predicted_positive = true_positives + false_negatives;
				let precision = if predicted_positive > 0 {
					Some(
						true_positives.to_f32().unwrap()
							/ (true_positives + false_positives).to_f32().unwrap(),
					)
				} else {
					None
				};
				// This is the fraction of the total positive examples that are correctly predicted as positive.
				let actual_positive = true_positives + false_negatives;
				let recall = if actual_positive > 0 {
					Some(
						true_positives.to_f32().unwrap()
							/ (true_positives + false_negatives).to_f32().unwrap(),
					)
				} else {
					None
				};
				let f1_score = match (recall, precision) {
					(Some(recall), Some(precision)) => {
						Some(2.0 * (precision * recall) / (precision + recall))
					}
					_ => None,
				};
				// This is true_positive_rate = true_positives / positives.
				let true_positive_rate = (true_positives.to_f32().unwrap())
					/ (true_positives.to_f32().unwrap() + false_negatives.to_f32().unwrap());
				// This is false_positive_rate = false_positives / negatives.
				let false_positive_rate = false_positives.to_f32().unwrap()
					/ (true_negatives.to_f32().unwrap() + false_positives.to_f32().unwrap());
				BinaryClassificationMetricsOutputForThreshold {
					threshold: *threshold,
					false_negatives,
					false_positives,
					true_negatives,
					true_positives,
					accuracy,
					precision,
					recall,
					f1_score,
					false_positive_rate,
					true_positive_rate,
				}
			})
			.collect();
		// Compute the area under the receiver operating characteristic curve using a riemann sum.
		let auc_roc_approx = thresholds
			.iter()
			.rev()
			.tuple_windows()
			.map(|(left, right)| {
				// Use the trapezoid rule.
				let y_avg =
					(left.true_positive_rate as f64 + right.true_positive_rate as f64) / 2.0;
				let dx = right.false_positive_rate as f64 - left.false_positive_rate as f64;
				y_avg * dx
			})
			.sum::<f64>() as f32;
		BinaryClassificationMetricsOutput {
			auc_roc_approx,
			thresholds,
		}
	}
}

#[test]
fn test() {
	let mut metrics = BinaryClassificationMetrics::new(3);
	let labels = &[
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
		Some(NonZeroUsize::new(1).unwrap()),
		Some(NonZeroUsize::new(2).unwrap()),
	];
	let probabilities = &[0.9, 0.2, 0.7, 0.2, 0.1];
	metrics.update(BinaryClassificationMetricsInput {
		probabilities,
		labels,
	});
	let metrics = metrics.finalize();
	insta::assert_debug_snapshot!(metrics, @r###"
 BinaryClassificationMetricsOutput {
     auc_roc_approx: 0.0,
     thresholds: [
         BinaryClassificationMetricsOutputForThreshold {
             threshold: 0.25,
             true_positives: 1,
             false_positives: 1,
             true_negatives: 2,
             false_negatives: 1,
             accuracy: 0.6,
             precision: Some(
                 0.5,
             ),
             recall: Some(
                 0.5,
             ),
             f1_score: Some(
                 0.5,
             ),
             true_positive_rate: 0.5,
             false_positive_rate: 0.33333334,
         },
         BinaryClassificationMetricsOutputForThreshold {
             threshold: 0.5,
             true_positives: 1,
             false_positives: 1,
             true_negatives: 2,
             false_negatives: 1,
             accuracy: 0.6,
             precision: Some(
                 0.5,
             ),
             recall: Some(
                 0.5,
             ),
             f1_score: Some(
                 0.5,
             ),
             true_positive_rate: 0.5,
             false_positive_rate: 0.33333334,
         },
         BinaryClassificationMetricsOutputForThreshold {
             threshold: 0.75,
             true_positives: 0,
             false_positives: 1,
             true_negatives: 2,
             false_negatives: 2,
             accuracy: 0.4,
             precision: Some(
                 0.0,
             ),
             recall: Some(
                 0.0,
             ),
             f1_score: Some(
                 NaN,
             ),
             true_positive_rate: 0.0,
             false_positive_rate: 0.33333334,
         },
     ],
 }
 "###);
}
