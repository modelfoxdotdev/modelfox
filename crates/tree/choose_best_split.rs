#[cfg(feature = "timing")]
use crate::timing::Timing;
use crate::{
	compute_bin_stats::{
		compute_bin_stats_column_major, compute_bin_stats_row_major, compute_bin_stats_subtraction,
		BinStats, BinStatsEntry,
	},
	compute_binned_features::{
		BinnedFeaturesColumnMajor, BinnedFeaturesRowMajor, BinnedFeaturesRowMajorInner,
	},
	compute_binning_instructions::BinningInstruction,
	pool::{Pool, PoolItem},
	train_tree::{TrainBranchSplit, TrainBranchSplitContinuous, TrainBranchSplitDiscrete},
	BinnedFeaturesLayout, SplitDirection, TrainOptions,
};
use bitvec::prelude::*;
use modelfox_zip::{pzip, zip};
use num::{NumCast, ToPrimitive};
use rayon::prelude::*;

pub struct ChooseBestSplitRootOptions<'a> {
	pub bin_stats_pool: &'a Pool<BinStats>,
	pub binned_features_column_major: &'a BinnedFeaturesColumnMajor,
	pub binned_features_row_major: &'a Option<BinnedFeaturesRowMajor>,
	pub binning_instructions: &'a [BinningInstruction],
	pub examples_index: &'a [u32],
	pub gradients: &'a [f32],
	pub hessians_are_constant: bool,
	pub hessians: &'a [f32],
	#[cfg(feature = "timing")]
	pub timing: &'a Timing,
	pub train_options: &'a TrainOptions,
}

pub struct ChooseBestSplitsNotRootOptions<'a> {
	pub bin_stats_pool: &'a Pool<BinStats>,
	pub splittable_features: &'a [bool],
	pub binned_features_column_major: &'a BinnedFeaturesColumnMajor,
	pub binned_features_row_major: &'a Option<BinnedFeaturesRowMajor>,
	pub binning_instructions: &'a [BinningInstruction],
	pub gradients_ordered_buffer: &'a mut [f32],
	pub gradients: &'a [f32],
	pub hessians_are_constant: bool,
	pub hessians_ordered_buffer: &'a mut [f32],
	pub hessians: &'a [f32],
	pub left_child_examples_index: &'a [u32],
	pub left_child_n_examples: usize,
	pub left_child_sum_gradients: f64,
	pub left_child_sum_hessians: f64,
	pub parent_bin_stats: PoolItem<BinStats>,
	pub parent_depth: usize,
	pub right_child_examples_index: &'a [u32],
	pub right_child_n_examples: usize,
	pub right_child_sum_gradients: f64,
	pub right_child_sum_hessians: f64,
	#[cfg(feature = "timing")]
	pub timing: &'a Timing,
	pub train_options: &'a TrainOptions,
}

pub enum ChooseBestSplitOutput {
	Success(ChooseBestSplitSuccess),
	Failure(ChooseBestSplitFailure),
}

pub struct ChooseBestSplitSuccess {
	pub gain: f32,
	pub split: TrainBranchSplit,
	pub sum_gradients: f64,
	pub sum_hessians: f64,
	pub left_n_examples: usize,
	pub left_sum_gradients: f64,
	pub left_sum_hessians: f64,
	pub right_n_examples: usize,
	pub right_sum_gradients: f64,
	pub right_sum_hessians: f64,
	pub bin_stats: PoolItem<BinStats>,
	pub splittable_features: Vec<bool>,
}

pub struct ChooseBestSplitFailure {
	pub sum_gradients: f64,
	pub sum_hessians: f64,
}

pub struct ChooseBestSplitForFeatureOutput {
	pub gain: f32,
	pub split: TrainBranchSplit,
	pub left_approximate_n_examples: usize,
	pub left_sum_gradients: f64,
	pub left_sum_hessians: f64,
	pub right_approximate_n_examples: usize,
	pub right_sum_gradients: f64,
	pub right_sum_hessians: f64,
}

const MIN_EXAMPLES_TO_PARALLELIZE: usize = 1024;

pub fn choose_best_split_root(options: ChooseBestSplitRootOptions) -> ChooseBestSplitOutput {
	let ChooseBestSplitRootOptions {
		bin_stats_pool,
		binned_features_column_major,
		binned_features_row_major,
		binning_instructions,
		examples_index,
		gradients,
		hessians_are_constant,
		hessians,
		train_options,
		..
	} = options;
	#[cfg(feature = "timing")]
	let timing = options.timing;
	// Compute the sums of gradients and hessians.
	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	let sum_gradients = gradients
		.par_iter()
		.map(|gradient| *gradient as f64)
		.sum::<f64>();
	let sum_hessians = if hessians_are_constant {
		hessians.len().to_f64().unwrap()
	} else {
		hessians
			.par_iter()
			.map(|hessian| *hessian as f64)
			.sum::<f64>()
	};
	#[cfg(feature = "timing")]
	timing.sum_gradients_and_hessians_root.inc(start.elapsed());

	// Determine if we should try to split the root.
	let should_try_to_split_root = gradients.len() >= 2 * train_options.min_examples_per_node
		&& sum_hessians >= 2.0 * train_options.min_sum_hessians_per_node as f64;
	if !should_try_to_split_root {
		return ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
			sum_gradients,
			sum_hessians,
		});
	}

	#[cfg(feature = "timing")]
	let start = std::time::Instant::now();
	// For each feature, compute bin stats and use them to choose the best split.
	let mut bin_stats = bin_stats_pool.get().unwrap();
	let best_split_output: (Option<ChooseBestSplitForFeatureOutput>, Vec<bool>) =
		match train_options.binned_features_layout {
			BinnedFeaturesLayout::ColumnMajor => {
				let bin_stats = bin_stats.as_column_major_mut().unwrap();
				choose_best_split_root_column_major(ChooseBestSplitRootColumnMajorOptions {
					bin_stats,
					binned_features_column_major,
					binning_instructions,
					gradients,
					hessians_are_constant,
					hessians,
					sum_gradients,
					sum_hessians,
					train_options,
				})
			}
			BinnedFeaturesLayout::RowMajor => {
				let bin_stats = bin_stats.as_row_major_mut().unwrap();
				choose_best_split_root_row_major(ChooseBestSplitRootRowMajorOptions {
					bin_stats,
					binned_features_row_major: binned_features_row_major.as_ref().unwrap(),
					binning_instructions,
					examples_index,
					gradients,
					hessians_are_constant,
					hessians,
					sum_gradients,
					sum_hessians,
					train_options,
				})
			}
		};
	#[cfg(feature = "timing")]
	timing.choose_best_split_root.inc(start.elapsed());

	// Assemble the output.
	match best_split_output {
		(Some(best_split), splittable_features) => {
			ChooseBestSplitOutput::Success(ChooseBestSplitSuccess {
				gain: best_split.gain,
				split: best_split.split,
				sum_gradients,
				sum_hessians,
				left_n_examples: best_split.left_approximate_n_examples,
				left_sum_gradients: best_split.left_sum_gradients,
				left_sum_hessians: best_split.left_sum_hessians,
				right_n_examples: best_split.right_approximate_n_examples,
				right_sum_gradients: best_split.right_sum_gradients,
				right_sum_hessians: best_split.right_sum_hessians,
				bin_stats,
				splittable_features,
			})
		}
		(None, _) => ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
			sum_gradients,
			sum_hessians,
		}),
	}
}

struct ChooseBestSplitRootColumnMajorOptions<'a> {
	bin_stats: &'a mut Vec<Vec<BinStatsEntry>>,
	binned_features_column_major: &'a BinnedFeaturesColumnMajor,
	binning_instructions: &'a [BinningInstruction],
	gradients: &'a [f32],
	hessians_are_constant: bool,
	hessians: &'a [f32],
	sum_gradients: f64,
	sum_hessians: f64,
	train_options: &'a TrainOptions,
}

fn choose_best_split_root_column_major(
	options: ChooseBestSplitRootColumnMajorOptions,
) -> (Option<ChooseBestSplitForFeatureOutput>, Vec<bool>) {
	let ChooseBestSplitRootColumnMajorOptions {
		bin_stats,
		binned_features_column_major,
		binning_instructions,
		gradients,
		hessians_are_constant,
		hessians,
		sum_gradients,
		sum_hessians,
		train_options,
	} = options;
	let mut splittable_features = vec![false; binned_features_column_major.columns.len()];
	let best_split = pzip!(
		binning_instructions,
		&binned_features_column_major.columns,
		bin_stats,
		splittable_features.par_iter_mut(),
	)
	.enumerate()
	.map(
		|(
			feature_index,
			(
				binning_instructions,
				binned_feature_column,
				bin_stats_for_feature,
				is_feature_splittable,
			),
		)| {
			// Compute the bin stats.
			compute_bin_stats_column_major::<true>(
				bin_stats_for_feature,
				&[],
				binned_feature_column,
				gradients,
				hessians,
				hessians_are_constant,
			);
			// Choose the best split for this featue.
			let best_split_for_feature = choose_best_split_for_feature(
				feature_index,
				binning_instructions,
				bin_stats_for_feature,
				binned_feature_column.len(),
				sum_gradients,
				sum_hessians,
				train_options,
			);
			if best_split_for_feature.is_some() {
				*is_feature_splittable = true;
			}
			best_split_for_feature
		},
	)
	.filter_map(|split| split)
	.max_by(|a, b| a.gain.partial_cmp(&b.gain).unwrap());
	(best_split, splittable_features)
}

struct ChooseBestSplitRootRowMajorOptions<'a> {
	bin_stats: &'a mut Vec<BinStatsEntry>,
	binned_features_row_major: &'a BinnedFeaturesRowMajor,
	binning_instructions: &'a [BinningInstruction],
	examples_index: &'a [u32],
	gradients: &'a [f32],
	hessians_are_constant: bool,
	hessians: &'a [f32],
	sum_gradients: f64,
	sum_hessians: f64,
	train_options: &'a TrainOptions,
}

fn choose_best_split_root_row_major(
	options: ChooseBestSplitRootRowMajorOptions,
) -> (Option<ChooseBestSplitForFeatureOutput>, Vec<bool>) {
	let ChooseBestSplitRootRowMajorOptions {
		bin_stats,
		binned_features_row_major,
		binning_instructions,
		examples_index,
		gradients,
		hessians_are_constant,
		hessians,
		sum_gradients,
		sum_hessians,
		train_options,
	} = options;
	// Compute the bin stats for the child with fewer examples.
	let n_examples = match binned_features_row_major {
		BinnedFeaturesRowMajor::U16(binned_features) => binned_features.values_with_offsets.nrows(),
		BinnedFeaturesRowMajor::U32(binned_features) => binned_features.values_with_offsets.nrows(),
	};
	let n_threads = rayon::current_num_threads();
	let chunk_size = (n_examples + n_threads - 1) / n_threads;
	*bin_stats = examples_index
		.par_chunks(chunk_size)
		.into_par_iter()
		.map(|examples_index_chunk| {
			let mut bin_stats_chunk: Vec<BinStatsEntry> =
				bin_stats.iter().map(|_| BinStatsEntry::default()).collect();
			compute_bin_stats_row_major::<true>(
				bin_stats_chunk.as_mut_slice(),
				examples_index_chunk,
				binned_features_row_major,
				gradients,
				hessians,
				hessians_are_constant,
				&[],
			);
			bin_stats_chunk
		})
		.reduce(
			|| bin_stats.iter().map(|_| BinStatsEntry::default()).collect(),
			|mut res, chunk| {
				for (res, chunk) in zip!(res.iter_mut(), chunk.iter()) {
					res.sum_gradients += chunk.sum_gradients;
					res.sum_hessians += chunk.sum_hessians;
				}
				res
			},
		);
	// Choose the best split for each featue.
	match binned_features_row_major {
		BinnedFeaturesRowMajor::U16(binned_features_row_major_inner) => {
			let options = ChooseBestSplitRootRowMajorForFeaturesOptions {
				bin_stats,
				binning_instructions,
				binned_features_row_major_inner,
				n_examples,
				sum_gradients,
				sum_hessians,
				train_options,
			};
			choose_best_split_root_row_major_for_features(options)
		}
		BinnedFeaturesRowMajor::U32(binned_features_row_major_inner) => {
			let options = ChooseBestSplitRootRowMajorForFeaturesOptions {
				bin_stats,
				binning_instructions,
				binned_features_row_major_inner,
				n_examples,
				sum_gradients,
				sum_hessians,
				train_options,
			};
			choose_best_split_root_row_major_for_features(options)
		}
	}
}

struct ChooseBestSplitRootRowMajorForFeaturesOptions<'a, T>
where
	T: Send + Sync + NumCast,
{
	bin_stats: &'a mut Vec<BinStatsEntry>,
	binning_instructions: &'a [BinningInstruction],
	binned_features_row_major_inner: &'a BinnedFeaturesRowMajorInner<T>,
	n_examples: usize,
	sum_gradients: f64,
	sum_hessians: f64,
	train_options: &'a TrainOptions,
}

fn choose_best_split_root_row_major_for_features<T>(
	options: ChooseBestSplitRootRowMajorForFeaturesOptions<T>,
) -> (Option<ChooseBestSplitForFeatureOutput>, Vec<bool>)
where
	T: Send + Sync + NumCast,
{
	let ChooseBestSplitRootRowMajorForFeaturesOptions {
		bin_stats,
		binning_instructions,
		binned_features_row_major_inner,
		n_examples,
		sum_gradients,
		sum_hessians,
		train_options,
	} = options;
	let bin_stats = BinStatsPtr(bin_stats);
	let mut splittable_features =
		vec![false; binned_features_row_major_inner.values_with_offsets.ncols()];
	let best_split = pzip!(
		binning_instructions,
		&binned_features_row_major_inner.offsets,
		splittable_features.par_iter_mut(),
	)
	.enumerate()
	.map(
		|(feature_index, (binning_instructions, offset, is_feature_splittable))| {
			let _ = &bin_stats;
			let bin_stats = unsafe { &mut *bin_stats.0 };
			let offset = offset.to_usize().unwrap();
			let bin_stats_range = offset..offset + binning_instructions.n_bins();
			let bin_stats_for_feature = &mut bin_stats[bin_stats_range];
			let best_split_for_feature = choose_best_split_for_feature(
				feature_index,
				binning_instructions,
				bin_stats_for_feature,
				n_examples,
				sum_gradients,
				sum_hessians,
				train_options,
			);
			if best_split_for_feature.is_some() {
				*is_feature_splittable = true;
			}
			best_split_for_feature
		},
	)
	.filter_map(|split| split)
	.max_by(|a, b| a.gain.partial_cmp(&b.gain).unwrap());
	(best_split, splittable_features)
}

pub fn choose_best_splits_not_root(
	options: ChooseBestSplitsNotRootOptions,
) -> (ChooseBestSplitOutput, ChooseBestSplitOutput) {
	let ChooseBestSplitsNotRootOptions {
		bin_stats_pool,
		binned_features_column_major,
		binned_features_row_major,
		binning_instructions,
		gradients_ordered_buffer,
		gradients,
		hessians_are_constant,
		hessians_ordered_buffer,
		hessians,
		left_child_examples_index,
		left_child_n_examples,
		left_child_sum_gradients,
		left_child_sum_hessians,
		parent_bin_stats,
		parent_depth,
		right_child_examples_index,
		right_child_n_examples,
		right_child_sum_gradients,
		right_child_sum_hessians,
		splittable_features,
		train_options,
		..
	} = options;
	let mut left_child_output = ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
		sum_gradients: left_child_sum_gradients,
		sum_hessians: left_child_sum_hessians,
	});
	let mut right_child_output = ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
		sum_gradients: right_child_sum_gradients,
		sum_hessians: right_child_sum_hessians,
	});

	// Determine if we should try to split the left and/or right children of this branch.
	let children_will_exceed_max_depth = if let Some(max_depth) = train_options.max_depth {
		parent_depth + 1 > max_depth - 1
	} else {
		false
	};
	let should_try_to_split_left_child = !children_will_exceed_max_depth
		&& left_child_examples_index.len() >= train_options.min_examples_per_node * 2;
	let should_try_to_split_right_child = !children_will_exceed_max_depth
		&& right_child_examples_index.len() >= train_options.min_examples_per_node * 2;

	// If we should not split either left or right, then there is nothing left to do, so we can go to the next item on the queue.
	if !should_try_to_split_left_child && !should_try_to_split_right_child {
		return (left_child_output, right_child_output);
	}

	// Determine which of the left and right children have fewer examples sent to them.
	let smaller_child_direction =
		if left_child_examples_index.len() < right_child_examples_index.len() {
			SplitDirection::Left
		} else {
			SplitDirection::Right
		};
	let smaller_child_examples_index = match smaller_child_direction {
		SplitDirection::Left => left_child_examples_index,
		SplitDirection::Right => right_child_examples_index,
	};
	let mut smaller_child_bin_stats = bin_stats_pool.get().unwrap();
	let mut larger_child_bin_stats = parent_bin_stats;

	// If the binned features are column major, fill the gradients and hessians ordered buffers. The buffers contain the gradients and hessians for each example as ordered by the examples index. This makes the access of the gradients and hessians sequential in the next step.
	if let BinnedFeaturesLayout::ColumnMajor = train_options.binned_features_layout {
		fill_gradients_and_hessians_ordered_buffers(
			smaller_child_examples_index,
			gradients,
			hessians,
			gradients_ordered_buffer,
			hessians_ordered_buffer,
			hessians_are_constant,
		);
	}

	// Collect the best splits for the left and right children for each feature.
	let children_best_splits_for_features: Vec<(
		Option<ChooseBestSplitForFeatureOutput>,
		Option<ChooseBestSplitForFeatureOutput>,
	)> = match train_options.binned_features_layout {
		BinnedFeaturesLayout::RowMajor => {
			let smaller_child_bin_stats = smaller_child_bin_stats.as_row_major_mut().unwrap();
			let larger_child_bin_stats = larger_child_bin_stats.as_row_major_mut().unwrap();
			compute_bin_stats_and_choose_best_splits_not_root_row_major(
				ComputeBinStatsAndChooseBestSplitsNotRootRowMajorOptions {
					should_try_to_split_right_child,
					smaller_child_bin_stats,
					larger_child_bin_stats,
					binned_features_row_major: binned_features_row_major.as_ref().unwrap(),
					binning_instructions,
					gradients,
					hessians_are_constant,
					hessians,
					train_options,
					left_child_n_examples,
					left_child_sum_gradients,
					left_child_sum_hessians,
					right_child_n_examples,
					right_child_sum_gradients,
					right_child_sum_hessians,
					smaller_child_examples_index,
					should_try_to_split_left_child,
					smaller_child_direction,
					splittable_features,
				},
			)
		}
		BinnedFeaturesLayout::ColumnMajor => {
			let smaller_child_bin_stats = smaller_child_bin_stats.as_column_major_mut().unwrap();
			let larger_child_bin_stats = larger_child_bin_stats.as_column_major_mut().unwrap();
			compute_bin_stats_and_choose_best_splits_not_root_column_major(
				ComputeBinStatsAndChooseBestSplitsNotRootColumnMajorOptions {
					binned_features_column_major,
					binning_instructions,
					gradients_ordered_buffer,
					hessians_are_constant,
					hessians_ordered_buffer,
					larger_child_bin_stats,
					left_child_n_examples,
					left_child_sum_gradients,
					left_child_sum_hessians,
					right_child_n_examples,
					right_child_sum_gradients,
					right_child_sum_hessians,
					should_try_to_split_left_child,
					should_try_to_split_right_child,
					smaller_child_bin_stats,
					smaller_child_direction,
					smaller_child_examples_index,
					splittable_features,
					train_options,
				},
			)
		}
	};

	// Choose the features that are still able to be split by children of the current nodes.
	let (left_child_splittable_features, right_child_splittable_features) =
		compute_splittable_features_for_children(&children_best_splits_for_features);

	// Choose the splits for the left and right children with the highest gain.
	let (left_child_best_split, right_child_best_split) =
		choose_splits_with_highest_gain(children_best_splits_for_features);

	// Assign the smaller and larger bin stats to the left and right children depending on which direction was smaller.
	let (left_child_bin_stats, right_child_bin_stats) = match smaller_child_direction {
		SplitDirection::Left => (smaller_child_bin_stats, larger_child_bin_stats),
		SplitDirection::Right => (larger_child_bin_stats, smaller_child_bin_stats),
	};

	// Assemble the output.
	left_child_output = match left_child_best_split {
		Some(best_split) => ChooseBestSplitOutput::Success(ChooseBestSplitSuccess {
			gain: best_split.gain,
			split: best_split.split,
			sum_gradients: left_child_sum_gradients,
			sum_hessians: left_child_sum_hessians,
			left_n_examples: best_split.left_approximate_n_examples,
			left_sum_gradients: best_split.left_sum_gradients,
			left_sum_hessians: best_split.left_sum_hessians,
			right_n_examples: best_split.right_approximate_n_examples,
			right_sum_gradients: best_split.right_sum_gradients,
			right_sum_hessians: best_split.right_sum_hessians,
			bin_stats: left_child_bin_stats,
			splittable_features: left_child_splittable_features,
		}),
		None => ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
			sum_gradients: left_child_sum_gradients,
			sum_hessians: left_child_sum_hessians,
		}),
	};
	right_child_output = match right_child_best_split {
		Some(best_split) => ChooseBestSplitOutput::Success(ChooseBestSplitSuccess {
			gain: best_split.gain,
			split: best_split.split,
			sum_gradients: right_child_sum_gradients,
			sum_hessians: right_child_sum_hessians,
			left_n_examples: best_split.left_approximate_n_examples,
			left_sum_gradients: best_split.left_sum_gradients,
			left_sum_hessians: best_split.left_sum_hessians,
			right_n_examples: best_split.right_approximate_n_examples,
			right_sum_gradients: best_split.right_sum_gradients,
			right_sum_hessians: best_split.right_sum_hessians,
			bin_stats: right_child_bin_stats,
			splittable_features: right_child_splittable_features,
		}),
		None => ChooseBestSplitOutput::Failure(ChooseBestSplitFailure {
			sum_gradients: right_child_sum_gradients,
			sum_hessians: right_child_sum_hessians,
		}),
	};
	(left_child_output, right_child_output)
}

struct ComputeBinStatsAndChooseBestSplitsNotRootColumnMajorOptions<'a> {
	binned_features_column_major: &'a BinnedFeaturesColumnMajor,
	binning_instructions: &'a [BinningInstruction],
	gradients_ordered_buffer: &'a [f32],
	hessians_are_constant: bool,
	hessians_ordered_buffer: &'a [f32],
	larger_child_bin_stats: &'a mut Vec<Vec<BinStatsEntry>>,
	left_child_n_examples: usize,
	left_child_sum_gradients: f64,
	left_child_sum_hessians: f64,
	right_child_n_examples: usize,
	right_child_sum_gradients: f64,
	right_child_sum_hessians: f64,
	should_try_to_split_left_child: bool,
	should_try_to_split_right_child: bool,
	smaller_child_bin_stats: &'a mut Vec<Vec<BinStatsEntry>>,
	smaller_child_direction: SplitDirection,
	smaller_child_examples_index: &'a [u32],
	splittable_features: &'a [bool],
	train_options: &'a TrainOptions,
}

fn compute_bin_stats_and_choose_best_splits_not_root_column_major(
	options: ComputeBinStatsAndChooseBestSplitsNotRootColumnMajorOptions,
) -> Vec<(
	Option<ChooseBestSplitForFeatureOutput>,
	Option<ChooseBestSplitForFeatureOutput>,
)> {
	let ComputeBinStatsAndChooseBestSplitsNotRootColumnMajorOptions {
		binned_features_column_major,
		binning_instructions,
		gradients_ordered_buffer,
		hessians_are_constant,
		hessians_ordered_buffer,
		larger_child_bin_stats,
		left_child_n_examples,
		left_child_sum_gradients,
		left_child_sum_hessians,
		right_child_n_examples,
		right_child_sum_gradients,
		right_child_sum_hessians,
		should_try_to_split_left_child,
		should_try_to_split_right_child,
		smaller_child_bin_stats,
		smaller_child_direction,
		smaller_child_examples_index,
		splittable_features,
		train_options,
	} = options;
	pzip!(
		binning_instructions,
		&binned_features_column_major.columns,
		smaller_child_bin_stats,
		larger_child_bin_stats,
		splittable_features
	)
	.enumerate()
	.map(
		|(
			feature_index,
			(
				binning_instructions,
				binned_features_column,
				smaller_child_bin_stats_for_feature,
				larger_child_bin_stats_for_feature,
				is_feature_splittable,
			),
		)| {
			if !is_feature_splittable {
				return (None, None);
			}
			// Compute the bin stats for the child with fewer examples.
			compute_bin_stats_column_major::<false>(
				smaller_child_bin_stats_for_feature,
				smaller_child_examples_index,
				binned_features_column,
				gradients_ordered_buffer,
				hessians_ordered_buffer,
				hessians_are_constant,
			);
			// Compute the larger child bin stats by subtraction.
			compute_bin_stats_subtraction(
				smaller_child_bin_stats_for_feature,
				larger_child_bin_stats_for_feature,
			);
			let (left_child_bin_stats_for_feature, right_child_bin_stats_for_feature) =
				match smaller_child_direction {
					SplitDirection::Left => (
						smaller_child_bin_stats_for_feature,
						larger_child_bin_stats_for_feature,
					),
					SplitDirection::Right => (
						larger_child_bin_stats_for_feature,
						smaller_child_bin_stats_for_feature,
					),
				};
			// Choose the best splits for the left and right children.
			let left_child_best_split_for_feature = if should_try_to_split_left_child {
				choose_best_split_for_feature(
					feature_index,
					binning_instructions,
					left_child_bin_stats_for_feature,
					left_child_n_examples,
					left_child_sum_gradients,
					left_child_sum_hessians,
					train_options,
				)
			} else {
				None
			};
			let right_child_best_split_for_feature = if should_try_to_split_right_child {
				choose_best_split_for_feature(
					feature_index,
					binning_instructions,
					right_child_bin_stats_for_feature,
					right_child_n_examples,
					right_child_sum_gradients,
					right_child_sum_hessians,
					train_options,
				)
			} else {
				None
			};
			(
				left_child_best_split_for_feature,
				right_child_best_split_for_feature,
			)
		},
	)
	.collect()
}

struct ComputeBinStatsAndChooseBestSplitsNotRootRowMajorOptions<'a> {
	binned_features_row_major: &'a BinnedFeaturesRowMajor,
	binning_instructions: &'a [BinningInstruction],
	gradients: &'a [f32],
	hessians_are_constant: bool,
	hessians: &'a [f32],
	larger_child_bin_stats: &'a mut Vec<BinStatsEntry>,
	left_child_n_examples: usize,
	left_child_sum_gradients: f64,
	left_child_sum_hessians: f64,
	right_child_n_examples: usize,
	right_child_sum_gradients: f64,
	right_child_sum_hessians: f64,
	should_try_to_split_left_child: bool,
	should_try_to_split_right_child: bool,
	smaller_child_bin_stats: &'a mut Vec<BinStatsEntry>,
	smaller_child_direction: SplitDirection,
	smaller_child_examples_index: &'a [u32],
	splittable_features: &'a [bool],
	train_options: &'a TrainOptions,
}

fn compute_bin_stats_and_choose_best_splits_not_root_row_major(
	options: ComputeBinStatsAndChooseBestSplitsNotRootRowMajorOptions,
) -> Vec<(
	Option<ChooseBestSplitForFeatureOutput>,
	Option<ChooseBestSplitForFeatureOutput>,
)> {
	let ComputeBinStatsAndChooseBestSplitsNotRootRowMajorOptions {
		binned_features_row_major,
		binning_instructions,
		gradients,
		hessians_are_constant,
		hessians,
		larger_child_bin_stats,
		left_child_n_examples,
		left_child_sum_gradients,
		left_child_sum_hessians,
		right_child_n_examples,
		right_child_sum_gradients,
		right_child_sum_hessians,
		should_try_to_split_left_child,
		should_try_to_split_right_child,
		smaller_child_bin_stats,
		smaller_child_direction,
		smaller_child_examples_index,
		splittable_features,
		train_options,
	} = options;
	// Compute the bin stats for the child with fewer examples.
	let smaller_child_n_examples = smaller_child_examples_index.len();
	if smaller_child_n_examples < MIN_EXAMPLES_TO_PARALLELIZE {
		compute_bin_stats_row_major::<false>(
			smaller_child_bin_stats.as_mut_slice(),
			smaller_child_examples_index,
			binned_features_row_major,
			gradients,
			hessians,
			hessians_are_constant,
			splittable_features,
		);
	} else {
		let n_threads = rayon::current_num_threads();
		let chunk_size = (smaller_child_n_examples + n_threads - 1) / n_threads;
		*smaller_child_bin_stats = pzip!(smaller_child_examples_index.par_chunks(chunk_size))
			.map(|(smaller_child_examples_index_chunk,)| {
				let mut smaller_child_bin_stats_chunk: Vec<BinStatsEntry> = smaller_child_bin_stats
					.iter()
					.map(|_| BinStatsEntry::default())
					.collect();
				compute_bin_stats_row_major::<false>(
					smaller_child_bin_stats_chunk.as_mut_slice(),
					smaller_child_examples_index_chunk,
					binned_features_row_major,
					gradients,
					hessians,
					hessians_are_constant,
					splittable_features,
				);
				smaller_child_bin_stats_chunk
			})
			.reduce(
				|| {
					smaller_child_bin_stats
						.iter()
						.map(|_| BinStatsEntry::default())
						.collect()
				},
				|mut res, chunk| {
					for (res, chunk) in zip!(res.iter_mut(), chunk.iter()) {
						res.sum_gradients += chunk.sum_gradients;
						res.sum_hessians += chunk.sum_hessians;
					}
					res
				},
			);
	}
	// Choose the best split for each feature.
	match binned_features_row_major {
		BinnedFeaturesRowMajor::U16(binned_features_row_major_inner) => {
			choose_best_splits_not_root_row_major(ChooseBestSplitsNotRootRowMajorOptions {
				binned_features_row_major_inner,
				binning_instructions,
				larger_child_bin_stats,
				left_child_n_examples,
				left_child_sum_gradients,
				left_child_sum_hessians,
				right_child_n_examples,
				right_child_sum_gradients,
				right_child_sum_hessians,
				should_try_to_split_left_child,
				should_try_to_split_right_child,
				smaller_child_bin_stats,
				smaller_child_direction,
				splittable_features,
				train_options,
			})
		}
		BinnedFeaturesRowMajor::U32(binned_features_row_major_inner) => {
			choose_best_splits_not_root_row_major(ChooseBestSplitsNotRootRowMajorOptions {
				binned_features_row_major_inner,
				binning_instructions,
				larger_child_bin_stats,
				left_child_n_examples,
				left_child_sum_gradients,
				left_child_sum_hessians,
				right_child_n_examples,
				right_child_sum_gradients,
				right_child_sum_hessians,
				should_try_to_split_left_child,
				should_try_to_split_right_child,
				smaller_child_bin_stats,
				smaller_child_direction,
				splittable_features,
				train_options,
			})
		}
	}
}

struct ChooseBestSplitsNotRootRowMajorOptions<'a, T>
where
	T: NumCast + Send + Sync,
{
	binned_features_row_major_inner: &'a BinnedFeaturesRowMajorInner<T>,
	binning_instructions: &'a [BinningInstruction],
	larger_child_bin_stats: &'a mut Vec<BinStatsEntry>,
	left_child_n_examples: usize,
	left_child_sum_gradients: f64,
	left_child_sum_hessians: f64,
	right_child_n_examples: usize,
	right_child_sum_gradients: f64,
	right_child_sum_hessians: f64,
	should_try_to_split_left_child: bool,
	should_try_to_split_right_child: bool,
	smaller_child_bin_stats: &'a mut Vec<BinStatsEntry>,
	smaller_child_direction: SplitDirection,
	splittable_features: &'a [bool],
	train_options: &'a TrainOptions,
}

fn choose_best_splits_not_root_row_major<T>(
	options: ChooseBestSplitsNotRootRowMajorOptions<T>,
) -> Vec<(
	Option<ChooseBestSplitForFeatureOutput>,
	Option<ChooseBestSplitForFeatureOutput>,
)>
where
	T: NumCast + Send + Sync,
{
	let ChooseBestSplitsNotRootRowMajorOptions {
		binned_features_row_major_inner,
		binning_instructions,
		larger_child_bin_stats,
		left_child_n_examples,
		left_child_sum_gradients,
		left_child_sum_hessians,
		right_child_n_examples,
		right_child_sum_gradients,
		right_child_sum_hessians,
		should_try_to_split_left_child,
		should_try_to_split_right_child,
		smaller_child_bin_stats,
		smaller_child_direction,
		splittable_features,
		train_options,
	} = options;
	let smaller_child_bin_stats = BinStatsPtr(smaller_child_bin_stats);
	let larger_child_bin_stats = BinStatsPtr(larger_child_bin_stats);
	pzip!(
		binning_instructions,
		&binned_features_row_major_inner.offsets,
		splittable_features,
	)
	.enumerate()
	.map(
		|(feature_index, (binning_instructions, offset, is_feature_splittable))| {
			let _ = (&smaller_child_bin_stats, &larger_child_bin_stats);
			if !is_feature_splittable {
				return (None, None);
			}
			let smaller_child_bin_stats_for_feature = unsafe {
				&mut (&mut *smaller_child_bin_stats.0)[offset.to_usize().unwrap()
					..offset.to_usize().unwrap() + binning_instructions.n_bins()]
			};
			let larger_child_bin_stats_for_feature = unsafe {
				&mut (&mut *larger_child_bin_stats.0)[offset.to_usize().unwrap()
					..offset.to_usize().unwrap() + binning_instructions.n_bins()]
			};
			// Compute the larger child bin stats by subtraction.
			compute_bin_stats_subtraction(
				smaller_child_bin_stats_for_feature,
				larger_child_bin_stats_for_feature,
			);
			// Assign the smaller and larger bin stats to the left and right children depending on which direction was smaller.
			let (left_child_bin_stats_for_feature, right_child_bin_stats_for_feature) =
				match smaller_child_direction {
					SplitDirection::Left => (
						smaller_child_bin_stats_for_feature,
						larger_child_bin_stats_for_feature,
					),
					SplitDirection::Right => (
						larger_child_bin_stats_for_feature,
						smaller_child_bin_stats_for_feature,
					),
				};
			// Choose the best splits for the left and right children.
			let (left_child_best_split_for_feature, right_child_best_split_for_feature) =
				rayon::join(
					|| {
						if should_try_to_split_left_child {
							choose_best_split_for_feature(
								feature_index,
								binning_instructions,
								left_child_bin_stats_for_feature,
								left_child_n_examples,
								left_child_sum_gradients,
								left_child_sum_hessians,
								train_options,
							)
						} else {
							None
						}
					},
					|| {
						if should_try_to_split_right_child {
							choose_best_split_for_feature(
								feature_index,
								binning_instructions,
								right_child_bin_stats_for_feature,
								right_child_n_examples,
								right_child_sum_gradients,
								right_child_sum_hessians,
								train_options,
							)
						} else {
							None
						}
					},
				);
			(
				left_child_best_split_for_feature,
				right_child_best_split_for_feature,
			)
		},
	)
	.collect::<Vec<_>>()
}

/// Choose the best split for a feature by choosing a continuous split for number features and a discrete split for enum features.
fn choose_best_split_for_feature(
	feature_index: usize,
	binning_instructions: &BinningInstruction,
	bin_stats_for_feature: &[BinStatsEntry],
	n_examples: usize,
	sum_gradients: f64,
	sum_hessians: f64,
	train_options: &TrainOptions,
) -> Option<ChooseBestSplitForFeatureOutput> {
	match binning_instructions {
		BinningInstruction::Number { .. } => choose_best_split_for_continuous_feature(
			feature_index,
			binning_instructions,
			bin_stats_for_feature,
			n_examples,
			sum_gradients,
			sum_hessians,
			train_options,
		),
		BinningInstruction::Enum { .. } => choose_best_split_for_discrete_feature(
			feature_index,
			binning_instructions,
			bin_stats_for_feature,
			n_examples,
			sum_gradients,
			sum_hessians,
			train_options,
		),
	}
}

/// Choose the best continuous split for this feature.
fn choose_best_split_for_continuous_feature(
	feature_index: usize,
	binning_instructions: &BinningInstruction,
	bin_stats_for_feature: &[BinStatsEntry],
	n_examples_parent: usize,
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	train_options: &TrainOptions,
) -> Option<ChooseBestSplitForFeatureOutput> {
	let mut best_split_for_feature: Option<ChooseBestSplitForFeatureOutput> = None;
	let l2_regularization = train_options.l2_regularization_for_continuous_splits;
	let negative_loss_for_parent_node =
		compute_negative_loss(sum_gradients_parent, sum_hessians_parent, l2_regularization);
	let mut left_approximate_n_examples = 0;
	let mut left_sum_gradients = 0.0;
	let mut left_sum_hessians = 0.0;
	let thresholds = match binning_instructions {
		BinningInstruction::Number { thresholds } => thresholds,
		_ => unreachable!(),
	};
	// Always send invalid values to the left.
	let invalid_values_direction = SplitDirection::Left;
	let invalid_bin_stats = bin_stats_for_feature.get(0).unwrap().clone();
	left_sum_gradients += invalid_bin_stats.sum_gradients;
	left_sum_hessians += invalid_bin_stats.sum_hessians;
	// For each bin, determine if splitting at that bin's value would produce a better split.
	for (valid_bin_index, bin_stats_entry) in bin_stats_for_feature
		[1..bin_stats_for_feature.len() - 1]
		.iter()
		.enumerate()
	{
		// Approximate the number of examples that would be sent to the left child by assuming the fraction of examples is equal to the fraction of the sum of hessians.
		left_approximate_n_examples += (bin_stats_entry.sum_hessians
			* n_examples_parent.to_f64().unwrap()
			/ sum_hessians_parent)
			.round()
			.to_usize()
			.unwrap();
		left_sum_gradients += bin_stats_entry.sum_gradients;
		left_sum_hessians += bin_stats_entry.sum_hessians;
		// Above we approximate the number of examples based on the sum of hessians. It is possible this approximation is off by enough that left_approximate_n_examples exceeds n_examples_parent. If this happens, we must not consider any further bins as the split value by exiting the loop.
		let right_approximate_n_examples =
			match n_examples_parent.checked_sub(left_approximate_n_examples) {
				Some(right_n_examples) => right_n_examples,
				None => break,
			};
		// Compute the sum of gradients and hessians for the examples that would be sent to the right by this split. To make this fast, subtract the values for the left child from the parent.
		let right_sum_gradients = sum_gradients_parent - left_sum_gradients;
		let right_sum_hessians = sum_hessians_parent - left_sum_hessians;
		// Check if fewer than `min_examples_per_node` would be sent to the left child by this split.
		if left_approximate_n_examples < train_options.min_examples_per_node {
			continue;
		}
		// Check if fewer than `min_examples_per_node` would be sent to the right child by this split. If true, then splitting by the thresholds for all subsequent bins will also fail, so we can exit the loop.
		if right_approximate_n_examples < train_options.min_examples_per_node {
			break;
		}
		// Check if the sum of hessians for examples that would be sent to the left child by this split falls below `min_sum_hessians_per_node`.
		if left_sum_hessians < train_options.min_sum_hessians_per_node as f64 {
			continue;
		}
		// Check if the sum of hessians for examples that would be sent to the right child by this split falls below `min_sum_hessians_per_node`. If true, then splitting by the thresholds for all subsequent bins will also fail, so we can exit the loop. This is true because hessians are always positive.
		if right_sum_hessians < train_options.min_sum_hessians_per_node as f64 {
			break;
		}
		// Compute the gain for this candidate split.
		let gain = compute_gain(
			left_sum_gradients,
			left_sum_hessians,
			right_sum_gradients,
			right_sum_hessians,
			negative_loss_for_parent_node,
			l2_regularization,
		);
		// If this split has a higher gain or if there is no existing best split, then use this split.
		if best_split_for_feature
			.as_ref()
			.map(|best_split_for_feature| gain > best_split_for_feature.gain)
			.unwrap_or(true)
		{
			let split_value = *thresholds.get(valid_bin_index).unwrap();
			let split = TrainBranchSplit::Continuous(TrainBranchSplitContinuous {
				feature_index,
				bin_index: valid_bin_index + 1,
				split_value,
				invalid_values_direction,
			});
			best_split_for_feature = Some(ChooseBestSplitForFeatureOutput {
				gain,
				split,
				left_approximate_n_examples,
				left_sum_gradients,
				left_sum_hessians,
				right_approximate_n_examples,
				right_sum_gradients,
				right_sum_hessians,
			});
		}
	}
	best_split_for_feature
}

/// Choose the best discrete split for this feature.
fn choose_best_split_for_discrete_feature(
	feature_index: usize,
	binning_instructions: &BinningInstruction,
	bin_stats_for_feature: &[BinStatsEntry],
	n_examples_parent: usize,
	sum_gradients_parent: f64,
	sum_hessians_parent: f64,
	train_options: &TrainOptions,
) -> Option<ChooseBestSplitForFeatureOutput> {
	let mut best_split_for_feature: Option<ChooseBestSplitForFeatureOutput> = None;
	let l2_regularization = train_options.l2_regularization_for_discrete_splits;
	let negative_loss_for_parent_node =
		compute_negative_loss(sum_gradients_parent, sum_hessians_parent, l2_regularization);
	let mut left_approximate_n_examples = 0;
	let mut left_sum_gradients = 0.0;
	let mut left_sum_hessians = 0.0;
	// Sort the bin stats using a scoring function.
	let smoothing_factor = train_options.smoothing_factor_for_discrete_bin_sorting as f64;
	let mut sorted_bin_stats_for_feature: Vec<(usize, &BinStatsEntry)> =
		bin_stats_for_feature.iter().enumerate().collect();
	sorted_bin_stats_for_feature.sort_by(|(_, a), (_, b)| {
		let score_a = a.sum_gradients / (a.sum_hessians + smoothing_factor);
		let score_b = b.sum_gradients / (b.sum_hessians + smoothing_factor);
		score_a.partial_cmp(&score_b).unwrap()
	});
	// For each bin, determine if splitting at that bin's value would produce a better split.
	let init_split_direction: bool = SplitDirection::Right.into();
	let mut directions =
		bitvec![u8, Lsb0; init_split_direction as isize; binning_instructions.n_bins()];
	for (bin_index, bin_stats_entry) in
		sorted_bin_stats_for_feature[0..bin_stats_for_feature.len() - 1].iter()
	{
		*directions.get_mut(*bin_index).unwrap() = SplitDirection::Left.into();
		// Approximate the number of examples that would be sent to the left child by assuming the fraction of examples is equal to the fraction of the sum of hessians.
		left_approximate_n_examples += (bin_stats_entry.sum_hessians
			* n_examples_parent.to_f64().unwrap()
			/ sum_hessians_parent)
			.round()
			.to_usize()
			.unwrap();
		left_sum_gradients += bin_stats_entry.sum_gradients;
		left_sum_hessians += bin_stats_entry.sum_hessians;
		// Above we approximate the number of examples based on the sum of hessians. It is possible this approximation is off by enough that left_approximate_n_examples exceeds n_examples_parent. If this happens, we must not consider any further bins as the split value by exiting the loop.
		let right_approximate_n_examples =
			match n_examples_parent.checked_sub(left_approximate_n_examples) {
				Some(right_n_examples) => right_n_examples,
				None => break,
			};
		// Compute the sum of gradients and hessians for the examples that would be sent to the right by this split. To make this fast, subtract the values for the left child from the parent.
		let right_sum_gradients = sum_gradients_parent - left_sum_gradients;
		let right_sum_hessians = sum_hessians_parent - left_sum_hessians;
		// Check if fewer than `min_examples_per_node` would be sent to the left child by this split.
		if left_approximate_n_examples < train_options.min_examples_per_node {
			continue;
		}
		// Check if fewer than `min_examples_per_node` would be sent to the right child by this split. If true, then splitting by the thresholds for all subsequent bins will also fail, so we can exit the loop.
		if right_approximate_n_examples < train_options.min_examples_per_node {
			break;
		}
		// Check if the sum of hessians for examples that would be sent to the left child by this split falls below `min_sum_hessians_per_node`.
		if left_sum_hessians < train_options.min_sum_hessians_per_node as f64 {
			continue;
		}
		// Check if the sum of hessians for examples that would be sent to the right child by this split falls below `min_sum_hessians_per_node`. If true, then splitting by the thresholds for all subsequent bins will also fail, so we can exit the loop. This is true because hessians are always positive.
		if right_sum_hessians < train_options.min_sum_hessians_per_node as f64 {
			break;
		}
		// Compute the gain for this candidate split.
		let gain = compute_gain(
			left_sum_gradients,
			left_sum_hessians,
			right_sum_gradients,
			right_sum_hessians,
			negative_loss_for_parent_node,
			l2_regularization,
		);
		// If this split has a higher gain or if there is no existing best split, then use this split.
		if best_split_for_feature
			.as_ref()
			.map(|best_split_for_feature| gain > best_split_for_feature.gain)
			.unwrap_or(true)
		{
			let split = TrainBranchSplit::Discrete(TrainBranchSplitDiscrete {
				feature_index,
				directions: directions.clone(),
			});
			best_split_for_feature = Some(ChooseBestSplitForFeatureOutput {
				gain,
				split,
				left_approximate_n_examples,
				left_sum_gradients,
				left_sum_hessians,
				right_approximate_n_examples,
				right_sum_gradients,
				right_sum_hessians,
			});
		}
	}
	best_split_for_feature
}

/// Compute the gain for a candidate split.
fn compute_gain(
	sum_gradients_left: f64,
	sum_hessians_left: f64,
	sum_gradients_right: f64,
	sum_hessians_right: f64,
	negative_loss_current_node: f32,
	l2_regularization: f32,
) -> f32 {
	let left = compute_negative_loss(sum_gradients_left, sum_hessians_left, l2_regularization);
	let right = compute_negative_loss(sum_gradients_right, sum_hessians_right, l2_regularization);
	left + right - negative_loss_current_node
}

/// The negative loss is used to compute the gain of a given split.
fn compute_negative_loss(sum_gradients: f64, sum_hessians: f64, l2_regularization: f32) -> f32 {
	((sum_gradients * sum_gradients) / (sum_hessians + l2_regularization as f64))
		.to_f32()
		.unwrap()
}

fn fill_gradients_and_hessians_ordered_buffers(
	smaller_child_examples_index: &[u32],
	gradients: &[f32],
	hessians: &[f32],
	gradients_ordered_buffer: &mut [f32],
	hessians_ordered_buffer: &mut [f32],
	hessians_are_constant: bool,
) {
	#[allow(clippy::collapsible_else_if)]
	if !hessians_are_constant {
		if smaller_child_examples_index.len() < 1024 {
			for (example_index, ordered_gradient, ordered_hessian) in zip!(
				smaller_child_examples_index,
				gradients_ordered_buffer.iter_mut(),
				hessians_ordered_buffer.iter_mut(),
			) {
				unsafe {
					let example_index = example_index.to_usize().unwrap();
					*ordered_gradient = *gradients.get_unchecked(example_index);
					*ordered_hessian = *hessians.get_unchecked(example_index);
				}
			}
		} else {
			let num_threads = rayon::current_num_threads();
			let chunk_size = (smaller_child_examples_index.len() + num_threads - 1) / num_threads;
			pzip!(
				smaller_child_examples_index.par_chunks(chunk_size),
				gradients_ordered_buffer.par_chunks_mut(chunk_size),
				hessians_ordered_buffer.par_chunks_mut(chunk_size),
			)
			.for_each(
				|(example_index_for_node, ordered_gradients, ordered_hessians)| {
					for (example_index, ordered_gradient, ordered_hessian) in
						zip!(example_index_for_node, ordered_gradients, ordered_hessians)
					{
						unsafe {
							let example_index = example_index.to_usize().unwrap();
							*ordered_gradient = *gradients.get_unchecked(example_index);
							*ordered_hessian = *hessians.get_unchecked(example_index);
						}
					}
				},
			);
		}
	} else {
		if smaller_child_examples_index.len() < 1024 {
			for (example_index, ordered_gradient) in zip!(
				smaller_child_examples_index,
				gradients_ordered_buffer.iter_mut()
			) {
				unsafe {
					let example_index = example_index.to_usize().unwrap();
					*ordered_gradient = *gradients.get_unchecked(example_index);
				}
			}
		} else {
			let chunk_size = (smaller_child_examples_index.len() + rayon::current_num_threads()
				- 1) / rayon::current_num_threads();
			pzip!(
				smaller_child_examples_index.par_chunks(chunk_size),
				gradients_ordered_buffer.par_chunks_mut(chunk_size),
			)
			.for_each(|(example_index_for_node, ordered_gradients)| unsafe {
				for (example_index, ordered_gradient) in
					zip!(example_index_for_node, ordered_gradients)
				{
					let example_index = example_index.to_usize().unwrap();
					*ordered_gradient = *gradients.get_unchecked(example_index);
				}
			});
		}
	}
}

/// Choose the splits for the left and right children with the highest gain.
fn choose_splits_with_highest_gain(
	children_best_splits_for_features: Vec<(
		Option<ChooseBestSplitForFeatureOutput>,
		Option<ChooseBestSplitForFeatureOutput>,
	)>,
) -> (
	Option<ChooseBestSplitForFeatureOutput>,
	Option<ChooseBestSplitForFeatureOutput>,
) {
	children_best_splits_for_features.into_iter().fold(
		(None, None),
		|(current_left, current_right), (candidate_left, candidate_right)| {
			(
				choose_split_with_highest_gain(current_left, candidate_left),
				choose_split_with_highest_gain(current_right, candidate_right),
			)
		},
	)
}

fn choose_split_with_highest_gain(
	current: Option<ChooseBestSplitForFeatureOutput>,
	candidate: Option<ChooseBestSplitForFeatureOutput>,
) -> Option<ChooseBestSplitForFeatureOutput> {
	match (current, candidate) {
		(None, None) => None,
		(current, None) => current,
		(None, candidate) => candidate,
		(Some(current), Some(candidate)) => {
			if candidate.gain > current.gain {
				Some(candidate)
			} else {
				Some(current)
			}
		}
	}
}

/// Compute features that are splittable based on whether a valid split was found in this round.
fn compute_splittable_features_for_children(
	children_best_splits_for_features: &[(
		Option<ChooseBestSplitForFeatureOutput>,
		Option<ChooseBestSplitForFeatureOutput>,
	)],
) -> (Vec<bool>, Vec<bool>) {
	let n_features = children_best_splits_for_features.len();
	let mut left_child_splittable_features = vec![false; n_features];
	let mut right_child_splittable_features = vec![false; n_features];
	for (
		left_child_splittable_feature,
		right_child_splittable_feature,
		children_best_splits_for_feature,
	) in zip!(
		left_child_splittable_features.iter_mut(),
		right_child_splittable_features.iter_mut(),
		children_best_splits_for_features
	) {
		let (left_child_best_split_for_feature, right_child_best_split_for_feature) =
			children_best_splits_for_feature;
		if left_child_best_split_for_feature.is_some() {
			*left_child_splittable_feature = true;
		}
		if right_child_best_split_for_feature.is_some() {
			*right_child_splittable_feature = true;
		}
	}
	(
		left_child_splittable_features,
		right_child_splittable_features,
	)
}

struct BinStatsPtr(*mut Vec<BinStatsEntry>);
unsafe impl Send for BinStatsPtr {}
unsafe impl Sync for BinStatsPtr {}
