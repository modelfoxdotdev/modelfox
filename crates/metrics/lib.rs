/*!
This crate implements a number of metrics such as [`MeanSquaredError`](crate::mean_squared_error::MeanSquaredError) and [`Accuracy`](crate::accuracy::Accuracy).
*/

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

pub use self::accuracy::Accuracy;
pub use self::auc_roc::*;
pub use self::binary_classification::{
	BinaryClassificationMetrics, BinaryClassificationMetricsInput,
	BinaryClassificationMetricsOutput, BinaryClassificationMetricsOutputForThreshold,
};
pub use self::binary_cross_entropy::{BinaryCrossEntropy, BinaryCrossEntropyInput};
pub use self::cross_entropy::{CrossEntropy, CrossEntropyInput, CrossEntropyOutput};
pub use self::mean::Mean;
pub use self::mean_squared_error::MeanSquaredError;
pub use self::mean_variance::{m2_to_variance, merge_mean_m2, MeanVariance};
pub use self::mode::Mode;
pub use self::multiclass_classification::{
	ClassMetrics, MulticlassClassificationMetrics, MulticlassClassificationMetricsInput,
	MulticlassClassificationMetricsOutput,
};
pub use self::regression::{RegressionMetrics, RegressionMetricsInput, RegressionMetricsOutput};
