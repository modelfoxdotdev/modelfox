use crate::compute_binning_instructions::BinningInstruction;
use crate::TrainOptions;
use ndarray::prelude::*;
use num::{Num, NumCast, ToPrimitive};
use rayon::{self, prelude::*};
use std::collections::BTreeMap;
use tangram_table::{TableColumnView, TableView};
use tangram_zip::pzip;

#[derive(Debug)]
pub enum BinnedFeaturesRowMajor {
	U16(BinnedFeaturesRowMajorInner<u16>),
	U32(BinnedFeaturesRowMajorInner<u32>),
}

#[derive(Debug)]
pub struct BinnedFeaturesRowMajorInner<T>
where
	T: NumCast,
{
	pub values_with_offsets: Array2<T>,
	pub offsets: Vec<T>,
}

#[derive(Debug)]
pub struct BinnedFeaturesColumnMajor {
	pub columns: Vec<BinnedFeaturesColumnMajorColumn>,
}

#[derive(Debug)]
pub enum BinnedFeaturesColumnMajorColumn {
	U8(Vec<u8>),
	U16(Vec<u16>),
}

impl BinnedFeaturesColumnMajorColumn {
	pub fn len(&self) -> usize {
		match self {
			BinnedFeaturesColumnMajorColumn::U8(values) => values.len(),
			BinnedFeaturesColumnMajorColumn::U16(values) => values.len(),
		}
	}
}

pub fn compute_binned_features_row_major(
	features: &TableView,
	binning_instructions: &[BinningInstruction],
	progress: &(impl Fn() + Sync),
) -> BinnedFeaturesRowMajor {
	let n_bins_across_all_features = binning_instructions
		.iter()
		.map(|binning_instructions| binning_instructions.n_bins())
		.sum::<usize>();
	match n_bins_across_all_features {
		n_bins_across_all_features if n_bins_across_all_features <= 65536 => {
			BinnedFeaturesRowMajor::U16(compute_binned_features_row_major_inner(
				&features,
				binning_instructions,
				progress,
			))
		}
		n_bins_across_all_features if n_bins_across_all_features <= 4294967296 => {
			BinnedFeaturesRowMajor::U32(compute_binned_features_row_major_inner(
				&features,
				binning_instructions,
				progress,
			))
		}
		_ => unreachable!(),
	}
}

fn compute_binned_features_row_major_inner<T, P>(
	splittable_features: &TableView,
	binning_instructions: &[BinningInstruction],
	_progress: &P,
) -> BinnedFeaturesRowMajorInner<T>
where
	T: Send + Sync + Num + NumCast + Copy + std::ops::Add + std::ops::AddAssign,
	P: Sync + Fn(),
{
	let n_features = splittable_features.ncols();
	let n_examples = splittable_features.nrows();
	let mut values_with_offsets: Array2<T> =
		unsafe { Array::uninit((n_examples, n_features)).assume_init() };
	let mut offsets: Vec<T> = Vec::with_capacity(n_features);
	let mut current_offset: T = T::zero();
	for binning_instruction in binning_instructions.iter() {
		offsets.push(current_offset);
		current_offset += T::from(binning_instruction.n_bins()).unwrap();
	}
	pzip!(
		values_with_offsets.axis_iter_mut(Axis(1)),
		splittable_features.columns().as_slice(),
		binning_instructions,
		&offsets,
	)
	.for_each(
		|(mut binned_features_column, feature, binning_instruction, offset)| {
			match binning_instruction {
				BinningInstruction::Number { thresholds } => {
					pzip!(
						binned_features_column.axis_iter_mut(Axis(0)),
						feature.as_number().unwrap().as_slice(),
					)
					.for_each(|(binned_feature_value, feature_value)| {
						// Invalid values go to the first bin.
						let binned_feature_value = binned_feature_value.into_scalar();
						if !feature_value.is_finite() {
							*binned_feature_value = *offset;
						} else {
							let threshold = thresholds
								.binary_search_by(|threshold| {
									threshold.partial_cmp(feature_value).unwrap()
								})
								.unwrap_or_else(|bin| bin);
							let threshold = T::from(threshold).unwrap();
							// Use binary search on the thresholds to find the bin for the feature value.
							*binned_feature_value = *offset + threshold + T::one();
						}
					});
				}
				BinningInstruction::Enum { .. } => {
					pzip!(
						binned_features_column.axis_iter_mut(Axis(0)),
						feature.as_enum().unwrap().as_slice(),
					)
					.for_each(|(binned_feature_value, feature_value)| {
						let feature_value = feature_value.map(|v| v.get()).unwrap_or(0);
						let feature_value = T::from(feature_value).unwrap();
						*binned_feature_value.into_scalar() = *offset + feature_value;
					});
				}
			}
		},
	);
	BinnedFeaturesRowMajorInner {
		values_with_offsets,
		offsets,
	}
}

pub struct ComputeBinnedFeaturesColumnMajorOutput {
	pub binned_features: BinnedFeaturesColumnMajor,
	pub used_feature_indexes: Vec<usize>,
}

/// Compute the binned features based on the binning instructions.
pub fn compute_binned_features_column_major(
	features: &TableView,
	binning_instructions: &[BinningInstruction],
	train_options: &TrainOptions,
	progress: &(impl Fn() + Sync),
) -> ComputeBinnedFeaturesColumnMajorOutput {
	let columns = pzip!(features.columns().as_slice(), binning_instructions)
		.map(|(feature, binning_instruction)| match binning_instruction {
			BinningInstruction::Number { thresholds } => {
				let output = compute_binned_features_column_major_for_number_feature(
					feature,
					thresholds,
					train_options,
					progress,
				);
				output.binned_feature_column.map(|binned_feature_column| {
					BinnedFeaturesColumnMajorColumn::U8(binned_feature_column)
				})
			}
			BinningInstruction::Enum { n_variants } => {
				if *n_variants <= 255 {
					let output = compute_binned_features_column_major_for_enum_feature_inner(
						feature, progress,
					);
					Some(BinnedFeaturesColumnMajorColumn::U8(
						output.binned_feature_column,
					))
				} else if *n_variants <= 65535 {
					let output = compute_binned_features_column_major_for_enum_feature_inner(
						feature, progress,
					);
					Some(BinnedFeaturesColumnMajorColumn::U16(
						output.binned_feature_column,
					))
				} else {
					panic!("enum column has too many variants")
				}
			}
		})
		.collect::<Vec<_>>();
	let mut splittable_features = Vec::new();
	let mut train_feature_index_to_feature_index = Vec::new();
	for (feature_index, output) in columns.into_iter().enumerate() {
		if let Some(output) = output {
			train_feature_index_to_feature_index.push(feature_index);
			splittable_features.push(output);
		}
	}
	ComputeBinnedFeaturesColumnMajorOutput {
		binned_features: BinnedFeaturesColumnMajor {
			columns: splittable_features,
		},
		used_feature_indexes: train_feature_index_to_feature_index,
	}
}

struct ComputeBinnedFeaturesColumnMajorForNumberFeatureOutput {
	binned_feature_column: Option<Vec<u8>>,
}

fn compute_binned_features_column_major_for_number_feature(
	feature: &TableColumnView,
	thresholds: &[f32],
	train_options: &TrainOptions,
	_progress: &(impl Fn() + Sync),
) -> ComputeBinnedFeaturesColumnMajorForNumberFeatureOutput {
	let mut n_examples_per_bin = BTreeMap::new();
	let binned_feature_column = feature
		.as_number()
		.unwrap()
		.as_slice()
		.iter()
		.map(|feature_value| {
			// Invalid values go to the first bin.
			if !feature_value.is_finite() {
				return 0;
			}
			// Use binary search on the thresholds to find the bin for the feature value.
			let bin = thresholds
				.binary_search_by(|threshold| threshold.partial_cmp(feature_value).unwrap())
				.unwrap_or_else(|bin| bin)
				.to_u8()
				.unwrap() + 1;
			if let Some(entry) = n_examples_per_bin.get_mut(&bin) {
				*entry += 1;
			} else {
				n_examples_per_bin.insert(bin, 1);
			}
			bin
		})
		.collect();
	let binned_feature_column =
		if compute_is_splittable(&n_examples_per_bin, feature.len(), train_options) {
			Some(binned_feature_column)
		} else {
			None
		};
	ComputeBinnedFeaturesColumnMajorForNumberFeatureOutput {
		binned_feature_column,
	}
}

fn compute_is_splittable(
	n_examples_per_bin: &BTreeMap<u8, usize>,
	n_examples: usize,
	train_options: &TrainOptions,
) -> bool {
	let mut n_examples_so_far = 0;
	let mut is_splittable = false;
	for (_, n_examples_in_bin) in n_examples_per_bin.iter().take(n_examples_per_bin.len() - 1) {
		n_examples_so_far += n_examples_in_bin;
		if n_examples_so_far >= train_options.min_examples_per_node
			&& (n_examples - n_examples_so_far) >= train_options.min_examples_per_node
		{
			is_splittable = true;
			break;
		}
	}
	is_splittable
}

struct ComputeBinnedFeaturesColumnMajorForEnumFeatureInnerOuptut<T> {
	binned_feature_column: Vec<T>,
}

fn compute_binned_features_column_major_for_enum_feature_inner<T, P>(
	feature: &TableColumnView,
	_progress: &P,
) -> ComputeBinnedFeaturesColumnMajorForEnumFeatureInnerOuptut<T>
where
	T: Send + Sync + NumCast + Ord + Clone,
	P: Sync + Fn(),
{
	let mut n_examples_per_bin = BTreeMap::new();
	let binned_feature_column = feature
		.as_enum()
		.unwrap()
		.iter()
		.map(|feature_value| {
			let bin = T::from(feature_value.map(|v| v.get()).unwrap_or(0)).unwrap();
			if let Some(entry) = n_examples_per_bin.get_mut(&bin) {
				*entry += 1;
			} else {
				n_examples_per_bin.insert(bin.clone(), 1);
			}
			bin
		})
		.collect::<Vec<_>>();
	ComputeBinnedFeaturesColumnMajorForEnumFeatureInnerOuptut {
		binned_feature_column,
	}
}
