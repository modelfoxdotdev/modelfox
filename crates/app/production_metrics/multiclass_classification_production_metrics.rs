use ndarray::prelude::*;
use num::ToPrimitive;
use serde::ser::SerializeStruct;
use tangram_app_common::monitor_event::NumberOrString;
use tangram_zip::zip;

#[derive(Clone)]
pub struct MulticlassClassificationProductionPredictionMetrics {
	classes: Vec<String>,
	confusion_matrix: Array2<u64>,
}

impl serde::Serialize for MulticlassClassificationProductionPredictionMetrics {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::ser::Serializer,
	{
		let mut state = serializer
			.serialize_struct("MulticlassClassificationProductionPredictionMetrics", 2)?;
		state.serialize_field("classes", &self.classes)?;
		state.serialize_field(
			"confusion_matrix",
			self.confusion_matrix.as_slice().unwrap(),
		)?;
		state.end()
	}
}

impl<'de> serde::de::Deserialize<'de> for MulticlassClassificationProductionPredictionMetrics {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::de::Deserializer<'de>,
	{
		#[derive(serde::Deserialize)]
		#[serde(field_identifier)]
		enum Field {
			#[serde(rename = "classes")]
			Classes,
			#[serde(rename = "confusion_matrix")]
			ConfusionMatrix,
		}
		struct Visitor;
		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = MulticlassClassificationProductionPredictionMetrics;
			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("MulticlassClassificationProductionPredictionMetrics")
			}
			fn visit_map<V>(
				self,
				mut map: V,
			) -> Result<MulticlassClassificationProductionPredictionMetrics, V::Error>
			where
				V: serde::de::MapAccess<'de>,
			{
				let mut classes = None;
				let mut confusion_matrix = None;
				while let Some(key) = map.next_key()? {
					match key {
						Field::Classes => {
							if classes.is_some() {
								return Err(serde::de::Error::duplicate_field("classes"));
							}
							classes = Some(map.next_value()?);
						}
						Field::ConfusionMatrix => {
							if confusion_matrix.is_some() {
								return Err(serde::de::Error::duplicate_field("confusion_matrix"));
							}
							confusion_matrix = Some(map.next_value()?);
						}
					}
				}
				let classes: Vec<String> =
					classes.ok_or_else(|| serde::de::Error::missing_field("classes"))?;
				let confusion_matrix = confusion_matrix
					.ok_or_else(|| serde::de::Error::missing_field("confusion_matrix"))?;
				let confusion_matrix =
					Array2::from_shape_vec((classes.len(), classes.len()), confusion_matrix)
						.map_err(|_| {
							serde::de::Error::custom(format!(
								"{} elements",
								classes.len() * classes.len()
							))
						})?;
				Ok(MulticlassClassificationProductionPredictionMetrics {
					classes,
					confusion_matrix,
				})
			}
		}
		deserializer.deserialize_struct(
			"MulticlassClassificationProductionPredictionMetrics",
			&["classes", "confusion_matrix"],
			Visitor,
		)
	}
}

#[derive(Debug, serde::Deserialize)]
pub struct MulticlassClassificationProductionPredictionMetricsOutput {
	pub class_metrics: Vec<MulticlassClassificationProductionPredictionClassMetricsOutput>,
	pub accuracy: f32,
	pub baseline_accuracy: f32,
	pub precision_unweighted: f32,
	pub precision_weighted: f32,
	pub recall_unweighted: f32,
	pub recall_weighted: f32,
}

#[derive(Debug, serde::Deserialize)]
pub struct MulticlassClassificationProductionPredictionClassMetricsOutput {
	pub class_name: String,
	pub true_positives: u64,
	pub false_positives: u64,
	pub true_negatives: u64,
	pub false_negatives: u64,
	pub accuracy: f32,
	pub precision: f32,
	pub recall: f32,
	pub f1_score: f32,
}

impl MulticlassClassificationProductionPredictionMetrics {
	pub fn new(classes: Vec<String>) -> MulticlassClassificationProductionPredictionMetrics {
		let n_classes = classes.len();
		let confusion_matrix = <Array2<u64>>::zeros((n_classes, n_classes));
		MulticlassClassificationProductionPredictionMetrics {
			classes,
			confusion_matrix,
		}
	}

	pub fn update(&mut self, value: (NumberOrString, NumberOrString)) {
		let label = match value.1 {
			NumberOrString::Number(_) => return,
			NumberOrString::String(s) => s,
		};
		let prediction = match value.0 {
			NumberOrString::Number(_) => return,
			NumberOrString::String(s) => s,
		};
		let actual_label_id = match self.classes.iter().position(|c| *c == label) {
			Some(position) => position,
			None => return,
		};
		if let Some(predicted_label_id) = self.classes.iter().position(|c| *c == prediction) {
			self.confusion_matrix[(predicted_label_id, actual_label_id)] += 1
		}
	}

	pub fn merge(&mut self, other: MulticlassClassificationProductionPredictionMetrics) {
		self.confusion_matrix += &other.confusion_matrix;
	}

	pub fn finalize(self) -> Option<MulticlassClassificationProductionPredictionMetricsOutput> {
		let n_classes = self.classes.len();
		let n_examples = self.confusion_matrix.sum();
		let confusion_matrix = self.confusion_matrix;
		let class_metrics: Vec<_> = self
			.classes
			.into_iter()
			.enumerate()
			.map(|(class_index, class_name)| {
				let true_positives = confusion_matrix[(class_index, class_index)];
				let false_positives = confusion_matrix.row(class_index).sum() - true_positives;
				let false_negatives = confusion_matrix.column(class_index).sum() - true_positives;
				let true_negatives =
					n_examples - true_positives - false_positives - false_negatives;
				let accuracy = (true_positives + true_negatives).to_f32().unwrap()
					/ n_examples.to_f32().unwrap();
				let precision = true_positives.to_f32().unwrap()
					/ (true_positives + false_positives).to_f32().unwrap();
				let recall = true_positives.to_f32().unwrap()
					/ (true_positives + false_negatives).to_f32().unwrap();
				let f1_score = 2.0 * (precision * recall) / (precision + recall);
				MulticlassClassificationProductionPredictionClassMetricsOutput {
					class_name,
					true_positives,
					false_positives,
					true_negatives,
					false_negatives,
					accuracy,
					precision,
					recall,
					f1_score,
				}
			})
			.collect();
		let n_correct: u64 = confusion_matrix.diag().sum();
		let accuracy = n_correct.to_f32().unwrap() / n_examples.to_f32().unwrap();
		let precision_unweighted = class_metrics
			.iter()
			.map(|class| class.precision)
			.sum::<f32>()
			/ n_classes.to_f32().unwrap();
		let recall_unweighted = class_metrics.iter().map(|class| class.recall).sum::<f32>()
			/ n_classes.to_f32().unwrap();
		let n_examples_per_class = confusion_matrix.sum_axis(Axis(0));
		let precision_weighted = zip!(class_metrics.iter(), n_examples_per_class.iter())
			.map(|(class, n_examples_in_class)| {
				class.precision * n_examples_in_class.to_f32().unwrap()
			})
			.sum::<f32>()
			/ n_examples.to_f32().unwrap();
		let recall_weighted = zip!(class_metrics.iter(), n_examples_per_class.iter())
			.map(|(class, n_examples_in_class)| {
				class.recall * n_examples_in_class.to_f32().unwrap()
			})
			.sum::<f32>()
			/ n_examples.to_f32().unwrap();
		let baseline_accuracy = n_examples_per_class
			.iter()
			.map(|n| n.to_f32().unwrap())
			.fold(None, |a: Option<f32>, b| match a {
				None => Some(b),
				Some(a) => Some(f32::max(a, b)),
			})
			.unwrap() / n_examples.to_f32().unwrap();
		if n_examples == 0 {
			None
		} else {
			Some(MulticlassClassificationProductionPredictionMetricsOutput {
				class_metrics,
				accuracy,
				baseline_accuracy,
				precision_unweighted,
				precision_weighted,
				recall_unweighted,
				recall_weighted,
			})
		}
	}
}

#[test]
fn test_binary() {
	let classes = vec!["Cat".to_owned(), "Dog".to_owned()];
	let mut metrics = MulticlassClassificationProductionPredictionMetrics::new(classes);
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
     MulticlassClassificationProductionPredictionMetricsOutput {
         class_metrics: [
             MulticlassClassificationProductionPredictionClassMetricsOutput {
                 class_name: "Cat",
                 true_positives: 5,
                 false_positives: 2,
                 true_negatives: 3,
                 false_negatives: 3,
                 accuracy: 0.61538464,
                 precision: 0.71428573,
                 recall: 0.625,
                 f1_score: 0.6666667,
             },
             MulticlassClassificationProductionPredictionClassMetricsOutput {
                 class_name: "Dog",
                 true_positives: 3,
                 false_positives: 3,
                 true_negatives: 5,
                 false_negatives: 2,
                 accuracy: 0.61538464,
                 precision: 0.5,
                 recall: 0.6,
                 f1_score: 0.54545456,
             },
         ],
         accuracy: 0.61538464,
         baseline_accuracy: 0.61538464,
         precision_unweighted: 0.60714287,
         precision_weighted: 0.6318681,
         recall_unweighted: 0.6125,
         recall_weighted: 0.61538464,
     },
 )
 "###);
}

#[test]
fn test_multiclass() {
	// This example is taken from https://en.wikipedia.org/wiki/Confusion_matrix.
	let classes = vec!["Cat".to_owned(), "Dog".to_owned(), "Rabbit".to_owned()];
	let mut metrics = MulticlassClassificationProductionPredictionMetrics::new(classes);
	metrics.update((
		NumberOrString::String("Cat".to_owned()),
		NumberOrString::String("Cat".to_owned()),
	));
	let labels = vec![
		"Cat", "Cat", "Cat", "Cat", "Dog", "Dog", "Cat", "Cat", "Cat", "Dog", "Dog", "Dog",
		"Rabbit", "Rabbit", "Dog", "Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit",
		"Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit",
	];
	let predictions = vec![
		"Cat", "Cat", "Cat", "Cat", "Cat", "Cat", "Dog", "Dog", "Dog", "Dog", "Dog", "Dog", "Dog",
		"Dog", "Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit", "Rabbit",
		"Rabbit", "Rabbit", "Rabbit", "Rabbit",
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
     MulticlassClassificationProductionPredictionMetricsOutput {
         class_metrics: [
             MulticlassClassificationProductionPredictionClassMetricsOutput {
                 class_name: "Cat",
                 true_positives: 5,
                 false_positives: 2,
                 true_negatives: 17,
                 false_negatives: 3,
                 accuracy: 0.8148148,
                 precision: 0.71428573,
                 recall: 0.625,
                 f1_score: 0.6666667,
             },
             MulticlassClassificationProductionPredictionClassMetricsOutput {
                 class_name: "Dog",
                 true_positives: 3,
                 false_positives: 5,
                 true_negatives: 16,
                 false_negatives: 3,
                 accuracy: 0.7037037,
                 precision: 0.375,
                 recall: 0.5,
                 f1_score: 0.42857143,
             },
             MulticlassClassificationProductionPredictionClassMetricsOutput {
                 class_name: "Rabbit",
                 true_positives: 11,
                 false_positives: 1,
                 true_negatives: 13,
                 false_negatives: 2,
                 accuracy: 0.8888889,
                 precision: 0.9166667,
                 recall: 0.84615386,
                 f1_score: 0.88,
             },
         ],
         accuracy: 0.7037037,
         baseline_accuracy: 0.4814815,
         precision_unweighted: 0.6686508,
         precision_weighted: 0.7363316,
         recall_unweighted: 0.65705127,
         recall_weighted: 0.7037037,
     },
 )
 "###);
}
