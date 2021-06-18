/*!
This crate implements a number of metrics such as [`MeanSquaredError`](crate::mean_squared_error::MeanSquaredError) and [`Accuracy`](crate::accuracy::Accuracy).
*/

pub use self::{
	accuracy::Accuracy,
	auc_roc::*,
	binary_classification::{
		BinaryClassificationMetrics, BinaryClassificationMetricsInput,
		BinaryClassificationMetricsOutput, BinaryClassificationMetricsOutputForThreshold,
	},
	binary_cross_entropy::{BinaryCrossEntropy, BinaryCrossEntropyInput},
	cross_entropy::{CrossEntropy, CrossEntropyInput, CrossEntropyOutput},
	mean::Mean,
	mean_squared_error::MeanSquaredError,
	mean_variance::{m2_to_variance, merge_mean_m2, MeanVariance},
	mode::Mode,
	multiclass_classification::{
		ClassMetrics, MulticlassClassificationMetrics, MulticlassClassificationMetricsInput,
		MulticlassClassificationMetricsOutput,
	},
	regression::{RegressionMetrics, RegressionMetricsInput, RegressionMetricsOutput},
};

mod accuracy;
mod auc_roc;
mod binary_classification;
mod binary_cross_entropy;
mod cross_entropy;
mod mean;
mod mean_squared_error;
mod mean_variance;
mod mode;
mod multiclass_classification;
mod regression;
