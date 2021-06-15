use ndarray::prelude::*;

#[derive(Clone, Debug, buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct Regressor {
	#[buffalo(id = 0, required)]
	pub bias: f32,
	#[buffalo(id = 1, required)]
	pub weights: Array1<f32>,
	#[buffalo(id = 2, required)]
	pub means: Vec<f32>,
}

#[derive(Clone, Debug, buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct BinaryClassifier {
	#[buffalo(id = 0, required)]
	pub bias: f32,
	#[buffalo(id = 1, required)]
	pub weights: Array1<f32>,
	#[buffalo(id = 2, required)]
	pub means: Vec<f32>,
}

#[derive(Clone, Debug, buffalo::Read, buffalo::Write)]
#[buffalo(size = "dynamic")]
pub struct MulticlassClassifier {
	#[buffalo(id = 0, required)]
	pub biases: Array1<f32>,
	#[buffalo(id = 1, required)]
	pub weights: Array2<f32>,
	#[buffalo(id = 2, required)]
	pub means: Vec<f32>,
}

pub(crate) fn serialize_regressor(
	regressor: &crate::Regressor,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<RegressorWriter> {
	let weights = writer.write(&regressor.weights);
	let means = writer.write(regressor.means.as_slice());
	writer.write(&RegressorWriter {
		bias: regressor.bias,
		weights,
		means,
	})
}

pub(crate) fn deserialize_regressor(regressor: RegressorReader) -> crate::Regressor {
	let bias = regressor.bias();
	let weights = regressor
		.weights()
		.iter()
		.map(|weights| weights.to_owned())
		.collect::<Vec<_>>()
		.into();
	let means = regressor
		.means()
		.iter()
		.map(|mean| mean.to_owned())
		.collect::<Vec<_>>();
	crate::Regressor {
		bias,
		weights,
		means,
	}
}

pub(crate) fn serialize_binary_classifier(
	binary_classifier: &crate::BinaryClassifier,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<BinaryClassifierWriter> {
	let weights = writer.write(&binary_classifier.weights);
	let means = writer.write(binary_classifier.means.as_slice());
	writer.write(&BinaryClassifierWriter {
		bias: binary_classifier.bias,
		weights,
		means,
	})
}

pub(crate) fn deserialize_binary_classifier(
	binary_classifier: BinaryClassifierReader,
) -> crate::BinaryClassifier {
	let bias = binary_classifier.bias();
	let weights = binary_classifier
		.weights()
		.iter()
		.map(|weights| weights.to_owned())
		.collect::<Vec<_>>()
		.into();
	let means = binary_classifier
		.means()
		.iter()
		.map(|mean| mean.to_owned())
		.collect::<Vec<_>>();
	crate::BinaryClassifier {
		bias,
		weights,
		means,
	}
}

pub(crate) fn serialize_multiclass_classifier(
	multiclass_classifier: &crate::MulticlassClassifier,
	writer: &mut buffalo::Writer,
) -> buffalo::Position<MulticlassClassifierWriter> {
	let weights = writer.write(&multiclass_classifier.weights);
	let biases = writer.write(&multiclass_classifier.biases);
	let means = writer.write(multiclass_classifier.means.as_slice());
	writer.write(&MulticlassClassifierWriter {
		biases,
		weights,
		means,
	})
}

pub(crate) fn deserialize_multiclass_classifier(
	multiclass_classifier: MulticlassClassifierReader,
) -> crate::MulticlassClassifier {
	let biases = multiclass_classifier
		.biases()
		.iter()
		.map(|bias| bias.to_owned())
		.collect::<Vec<_>>();
	let weights = multiclass_classifier.weights();
	let means = multiclass_classifier
		.means()
		.iter()
		.map(|mean| mean.to_owned())
		.collect::<Vec<_>>();
	crate::MulticlassClassifier {
		biases: biases.into(),
		weights,
		means,
	}
}
