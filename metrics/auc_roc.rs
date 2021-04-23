use itertools::Itertools;
use std::num::NonZeroUsize;

/// This `Metric` computes the area under the receiver operating characteristic curve.
pub struct AucRoc;

impl AucRoc {
	pub fn compute(mut input: Vec<(f32, NonZeroUsize)>) -> f32 {
		// Sort by probabilities in descending order.
		input.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap().reverse());
		// Collect the true_positives and false_positives counts for each unique probability.
		let mut true_positives_false_positives: Vec<TruePositivesFalsePositivesPoint> = Vec::new();
		for (probability, label) in input.iter() {
			// Labels are 1-indexed.
			let label = label.get() - 1;
			// If the classification threshold were to be this probability and the label is 1, the prediction is a true_positive. If the label is 0, its not a true_positive.
			let true_positive = label;
			// If the classification threshold were to be this probability and the label is 0, the prediction is a false_positive. If the label is 1, its not a false_positive.
			let false_positive = 1 - label;
			match true_positives_false_positives.last() {
				Some(last_point)
					if f32::abs(probability - last_point.probability) < std::f32::EPSILON =>
				{
					let last = true_positives_false_positives.last_mut().unwrap();
					last.true_positives += true_positive;
					last.false_positives += false_positive;
				}
				_ => {
					true_positives_false_positives.push(TruePositivesFalsePositivesPoint {
						probability: *probability,
						true_positives: true_positive,
						false_positives: false_positive,
					});
				}
			}
		}
		// Compute the cumulative sum of true positives and false positives.
		for i in 1..true_positives_false_positives.len() {
			true_positives_false_positives[i].true_positives +=
				true_positives_false_positives[i - 1].true_positives;
			true_positives_false_positives[i].false_positives +=
				true_positives_false_positives[i - 1].false_positives;
		}
		// Get the total count of positives.
		let count_positives = input.iter().map(|l| l.1.get() - 1).sum::<usize>();
		// Get the total count of negatives.
		let count_negatives = input.len() - count_positives;
		// The true_positive_rate at threshold x is the percent of the total positives that have a prediction probability >= x. At the maximum probability `x` observed in the dataset, either the true_positive_rate or false_positive_rate will be nonzero depending on whether the label at the this highest probability point is positive or negative respectively. This means that we will not have a point on the ROC curve with a true_positive_rate and false_positive_rate of 0. We create a dummy point with an impossible threshold of 2.0 such that no predictions have probability >= 2.0. At this threshold, both the true_positive_rate and false_positive_rate is 0.
		let mut roc_curve = vec![RocCurvePoint {
			threshold: 2.0,
			true_positive_rate: 0.0,
			false_positive_rate: 0.0,
		}];
		for true_positives_false_positives_point in true_positives_false_positives.iter() {
			roc_curve.push(RocCurvePoint {
				// The true positive rate is the number of true positives divided by the total number of positives.
				true_positive_rate: true_positives_false_positives_point.true_positives as f32
					/ count_positives as f32,
				threshold: true_positives_false_positives_point.probability,
				// The false positive rate is the number of false positives divided by the total number of negatives.
				false_positive_rate: true_positives_false_positives_point.false_positives as f32
					/ count_negatives as f32,
			});
		}
		// Compute the riemann sum using the trapezoidal rule.
		roc_curve
			.iter()
			.tuple_windows()
			.map(|(left, right)| {
				let y_avg =
					(left.true_positive_rate as f64 + right.true_positive_rate as f64) / 2.0;
				let dx = right.false_positive_rate as f64 - left.false_positive_rate as f64;
				y_avg * dx
			})
			.sum::<f64>() as f32
	}
}

/// A point on the ROC curve, parameterized by thresholds.
#[derive(Debug, PartialEq)]
struct RocCurvePoint {
	/// The classification threshold.
	threshold: f32,
	/// The true positive rate for all predictions with probability <= threshold.
	true_positive_rate: f32,
	/// The false positive rate for all predictions with probability <= threshold.
	false_positive_rate: f32,
}

#[derive(Debug)]
struct TruePositivesFalsePositivesPoint {
	/// The prediction probability.
	probability: f32,
	/// The true positives for this threshold.
	true_positives: usize,
	/// The false positives for this threshold.
	false_positives: usize,
}

#[test]
fn test_roc_curve() {
	use tangram_zip::zip;
	let labels = vec![
		NonZeroUsize::new(2).unwrap(),
		NonZeroUsize::new(2).unwrap(),
		NonZeroUsize::new(1).unwrap(),
		NonZeroUsize::new(1).unwrap(),
	];
	let probabilities = vec![0.9, 0.4, 0.4, 0.2];
	let input = zip!(probabilities.into_iter(), labels.into_iter()).collect();
	let actual = AucRoc::compute(input);
	let expected = 0.875;
	assert!(f32::abs(actual - expected) < f32::EPSILON)
}
