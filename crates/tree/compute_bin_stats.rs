use crate::compute_binned_features::{BinnedFeaturesColumnMajorColumn, BinnedFeaturesRowMajor};
use num::ToPrimitive;
use tangram_zip::zip;

//const HESSIANS_ARE_CONSTANT: bool = false;

/// This struct tracks the sum of gradients and hessians for all examples across all bins for all features.
#[derive(Clone, Debug)]
pub enum BinStats {
	/// In the `RowMajor` variant, the `Vec` has one item for each bin across all the features. The `offsets` field of `RowMajorBinnedFeatures` stores the offset into this `Vec` for each feature.
	RowMajor(Vec<BinStatsEntry>),
	/// In the `ColumnMajor` variant, the outer `Vec` has one item for each feature, and the inner `Vec` has one item for each bin for that particular feature.
	ColumnMajor(Vec<Vec<BinStatsEntry>>),
}

impl BinStats {
	pub fn as_row_major_mut(&mut self) -> Option<&mut Vec<BinStatsEntry>> {
		match self {
			BinStats::RowMajor(bin_stats) => Some(bin_stats),
			_ => None,
		}
	}

	pub fn as_column_major_mut(&mut self) -> Option<&mut Vec<Vec<BinStatsEntry>>> {
		match self {
			BinStats::ColumnMajor(bin_stats) => Some(bin_stats),
			_ => None,
		}
	}
}

/// This struct tracks the sum of gradients and hessians for all examples whose value for a particular feature falls in a particular bin.
#[derive(Clone, Default, Debug)]
pub struct BinStatsEntry {
	pub sum_gradients: f64,
	pub sum_hessians: f64,
}

/// This value controls how far ahead in the `examples_index` the `compute_bin_stats_*` functions should prefetch binned_features to be used in subsequent iterations.
#[cfg(target_arch = "x86_64")]
const PREFETCH_OFFSET: usize = 16;

pub fn compute_bin_stats_column_major<const ROOT: bool>(
	bin_stats_for_feature: &mut [BinStatsEntry],
	examples_index: &[u32],
	binned_feature_column: &BinnedFeaturesColumnMajorColumn,
	// (n_examples)
	gradients: &[f32],
	// (n_examples)
	hessians: &[f32],
	// The hessians are constant in least squares loss, so we do not have to waste time updating them.
	hessians_are_constant: bool,
) {
	if ROOT {
		for entry in bin_stats_for_feature.iter_mut() {
			*entry = BinStatsEntry {
				sum_gradients: 0.0,
				sum_hessians: 0.0,
			};
		}
		if hessians_are_constant {
			match binned_feature_column {
				BinnedFeaturesColumnMajorColumn::U8(binned_feature_values) => unsafe {
					compute_bin_stats_column_major_root::<u8, false>(
						gradients,
						&[],
						binned_feature_values,
						bin_stats_for_feature,
					)
				},
				BinnedFeaturesColumnMajorColumn::U16(binned_feature_values) => unsafe {
					compute_bin_stats_column_major_root::<u16, false>(
						gradients,
						&[],
						binned_feature_values,
						bin_stats_for_feature,
					)
				},
			}
		} else {
			match binned_feature_column {
				BinnedFeaturesColumnMajorColumn::U8(binned_feature_values) => unsafe {
					compute_bin_stats_column_major_root::<u8, true>(
						gradients,
						hessians,
						binned_feature_values,
						bin_stats_for_feature,
					)
				},
				BinnedFeaturesColumnMajorColumn::U16(binned_feature_values) => unsafe {
					compute_bin_stats_column_major_root::<u16, true>(
						gradients,
						hessians,
						binned_feature_values,
						bin_stats_for_feature,
					)
				},
			}
		}
	} else {
		for entry in bin_stats_for_feature.iter_mut() {
			*entry = BinStatsEntry {
				sum_gradients: 0.0,
				sum_hessians: 0.0,
			};
		}
		if hessians_are_constant {
			match binned_feature_column {
				BinnedFeaturesColumnMajorColumn::U8(binned_feature_values) => unsafe {
					compute_bin_stats_column_major_not_root::<u8, false>(
						gradients,
						&[],
						binned_feature_values.as_slice(),
						bin_stats_for_feature,
						examples_index,
					)
				},
				BinnedFeaturesColumnMajorColumn::U16(binned_feature_values) => unsafe {
					compute_bin_stats_column_major_not_root::<u16, false>(
						gradients,
						&[],
						binned_feature_values.as_slice(),
						bin_stats_for_feature,
						examples_index,
					)
				},
			}
		} else {
			match binned_feature_column {
				BinnedFeaturesColumnMajorColumn::U8(binned_feature_values) => unsafe {
					compute_bin_stats_column_major_not_root::<u8, true>(
						gradients,
						hessians,
						binned_feature_values.as_slice(),
						bin_stats_for_feature,
						examples_index,
					)
				},
				BinnedFeaturesColumnMajorColumn::U16(binned_feature_values) => unsafe {
					compute_bin_stats_column_major_not_root::<u16, true>(
						gradients,
						hessians,
						binned_feature_values.as_slice(),
						bin_stats_for_feature,
						examples_index,
					)
				},
			}
		}
	}
}

pub fn compute_bin_stats_subtraction(
	smaller_child_bin_stats_for_feature: &[BinStatsEntry],
	larger_child_bin_stats_for_feature: &mut [BinStatsEntry],
) {
	for (smaller_child_bin_stats, larger_child_bin_stats) in zip!(
		smaller_child_bin_stats_for_feature,
		larger_child_bin_stats_for_feature,
	) {
		larger_child_bin_stats.sum_gradients -= smaller_child_bin_stats.sum_gradients;
		larger_child_bin_stats.sum_hessians -= smaller_child_bin_stats.sum_hessians;
	}
}

unsafe fn compute_bin_stats_column_major_root<T, const HESSIANS: bool>(
	gradients: &[f32],
	hessians: &[f32],
	binned_feature_values: &[T],
	bin_stats_for_feature: &mut [BinStatsEntry],
) where
	T: ToPrimitive,
{
	if HESSIANS {
		let len = gradients.len();
		for i in 0..len {
			let ordered_gradient = *gradients.get_unchecked(i);
			let ordered_hessian = *hessians.get_unchecked(i);
			let bin_index = binned_feature_values.get_unchecked(i).to_usize().unwrap();
			let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
			bin_stats.sum_gradients += ordered_gradient as f64;
			bin_stats.sum_hessians += ordered_hessian as f64;
		}
	} else {
		let len = gradients.len();
		for i in 0..len {
			let ordered_gradient = *gradients.get_unchecked(i);
			let bin_index = binned_feature_values.get_unchecked(i).to_usize().unwrap();
			let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
			bin_stats.sum_gradients += ordered_gradient as f64;
			bin_stats.sum_hessians += 1.0;
		}
	}
}

unsafe fn compute_bin_stats_column_major_not_root<T, const HESSIANS: bool>(
	ordered_gradients: &[f32],
	ordered_hessians: &[f32],
	binned_feature_values: &[T],
	bin_stats_for_feature: &mut [BinStatsEntry],
	examples_index: &[u32],
) where
	T: ToPrimitive,
{
	if HESSIANS {
		let len = examples_index.len();
		#[cfg(target_arch = "x86_64")]
		let prefetch_len = len.saturating_sub(PREFETCH_OFFSET);
		#[cfg(not(target_arch = "x86_64"))]
		let prefetch_len = 0;
		for i in 0..prefetch_len {
			#[cfg(target_arch = "x86_64")]
			{
				let prefetch_index = *examples_index.get_unchecked(i + PREFETCH_OFFSET);
				let prefetch_index = prefetch_index.to_usize().unwrap();
				let prefetch_ptr = binned_feature_values.as_ptr().add(prefetch_index) as *const i8;
				core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
			}
			let ordered_gradient = *ordered_gradients.get_unchecked(i);
			let ordered_hessian = *ordered_hessians.get_unchecked(i);
			let example_index = *examples_index.get_unchecked(i);
			let example_index = example_index.to_usize().unwrap();
			let bin_index = binned_feature_values
				.get_unchecked(example_index)
				.to_usize()
				.unwrap();
			let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
			bin_stats.sum_gradients += ordered_gradient as f64;
			bin_stats.sum_hessians += ordered_hessian as f64;
		}
		for i in prefetch_len..len {
			let ordered_gradient = *ordered_gradients.get_unchecked(i);
			let ordered_hessian = *ordered_hessians.get_unchecked(i);
			let example_index = *examples_index.get_unchecked(i);
			let example_index = example_index.to_usize().unwrap();
			let bin_index = binned_feature_values
				.get_unchecked(example_index)
				.to_usize()
				.unwrap();
			let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
			bin_stats.sum_gradients += ordered_gradient as f64;
			bin_stats.sum_hessians += ordered_hessian as f64;
		}
	} else {
		let len = examples_index.len();
		#[cfg(target_arch = "x86_64")]
		let prefetch_len = len.saturating_sub(PREFETCH_OFFSET);
		#[cfg(not(target_arch = "x86_64"))]
		let prefetch_len = 0;
		for i in 0..prefetch_len {
			#[cfg(target_arch = "x86_64")]
			{
				let prefetch_index = *examples_index.get_unchecked(i + PREFETCH_OFFSET);
				let prefetch_index = prefetch_index.to_usize().unwrap();
				let prefetch_ptr = binned_feature_values.as_ptr().add(prefetch_index) as *const i8;
				core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
			}
			let ordered_gradient = *ordered_gradients.get_unchecked(i);
			let example_index = *examples_index.get_unchecked(i);
			let example_index = example_index.to_usize().unwrap();
			let bin_index = binned_feature_values
				.get_unchecked(example_index)
				.to_usize()
				.unwrap();
			let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
			bin_stats.sum_gradients += ordered_gradient as f64;
			bin_stats.sum_hessians += 1.0;
		}
		for i in prefetch_len..len {
			let ordered_gradient = *ordered_gradients.get_unchecked(i);
			let example_index = *examples_index.get_unchecked(i);
			let example_index = example_index.to_usize().unwrap();
			let bin_index = binned_feature_values
				.get_unchecked(example_index)
				.to_usize()
				.unwrap();
			let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_index);
			bin_stats.sum_gradients += ordered_gradient as f64;
			bin_stats.sum_hessians += 1.0;
		}
	}
}

pub fn compute_bin_stats_row_major<const ROOT: bool>(
	bin_stats: &mut [BinStatsEntry],
	examples_index: &[u32],
	binned_features: &BinnedFeaturesRowMajor,
	gradients: &[f32],
	hessians: &[f32],
	hessians_are_constant: bool,
	_splittable_features: &[bool],
) {
	if ROOT {
		for entry in bin_stats.iter_mut() {
			*entry = BinStatsEntry {
				sum_gradients: 0.0,
				sum_hessians: 0.0,
			};
		}
		if hessians_are_constant {
			match binned_features {
				BinnedFeaturesRowMajor::U16(binned_features) => unsafe {
					let n_features = binned_features.values_with_offsets.ncols();
					compute_bin_stats_row_major_root::<u16, false>(
						bin_stats,
						examples_index,
						binned_features.values_with_offsets.as_slice().unwrap(),
						gradients,
						&[],
						n_features,
					)
				},
				BinnedFeaturesRowMajor::U32(binned_features) => unsafe {
					let n_features = binned_features.values_with_offsets.ncols();
					compute_bin_stats_row_major_root::<u32, false>(
						bin_stats,
						examples_index,
						binned_features.values_with_offsets.as_slice().unwrap(),
						gradients,
						&[],
						n_features,
					)
				},
			}
		} else {
			match binned_features {
				BinnedFeaturesRowMajor::U16(binned_features) => unsafe {
					let n_features = binned_features.values_with_offsets.ncols();
					compute_bin_stats_row_major_root::<u16, true>(
						bin_stats,
						examples_index,
						binned_features.values_with_offsets.as_slice().unwrap(),
						gradients,
						hessians,
						n_features,
					)
				},
				BinnedFeaturesRowMajor::U32(binned_features) => unsafe {
					let n_features = binned_features.values_with_offsets.ncols();
					compute_bin_stats_row_major_root::<u32, true>(
						bin_stats,
						examples_index,
						binned_features.values_with_offsets.as_slice().unwrap(),
						gradients,
						hessians,
						n_features,
					)
				},
			}
		}
	} else {
		for entry in bin_stats.iter_mut() {
			*entry = BinStatsEntry {
				sum_gradients: 0.0,
				sum_hessians: 0.0,
			};
		}
		if hessians_are_constant {
			match binned_features {
				BinnedFeaturesRowMajor::U16(binned_features) => unsafe {
					let n_features = binned_features.values_with_offsets.ncols();
					compute_bin_stats_row_major_not_root::<u16, false>(
						bin_stats,
						examples_index,
						binned_features.values_with_offsets.as_slice().unwrap(),
						gradients,
						&[],
						n_features,
					)
				},
				BinnedFeaturesRowMajor::U32(binned_features) => unsafe {
					let n_features = binned_features.values_with_offsets.ncols();
					compute_bin_stats_row_major_not_root::<u32, false>(
						bin_stats,
						examples_index,
						binned_features.values_with_offsets.as_slice().unwrap(),
						gradients,
						&[],
						n_features,
					)
				},
			}
		} else {
			match binned_features {
				BinnedFeaturesRowMajor::U16(binned_features) => unsafe {
					let n_features = binned_features.values_with_offsets.ncols();
					compute_bin_stats_row_major_not_root::<u16, true>(
						bin_stats,
						examples_index,
						binned_features.values_with_offsets.as_slice().unwrap(),
						gradients,
						hessians,
						n_features,
					)
				},
				BinnedFeaturesRowMajor::U32(binned_features) => unsafe {
					let n_features = binned_features.values_with_offsets.ncols();
					compute_bin_stats_row_major_not_root::<u32, true>(
						bin_stats,
						examples_index,
						binned_features.values_with_offsets.as_slice().unwrap(),
						gradients,
						hessians,
						n_features,
					)
				},
			}
		}
	}
}

pub unsafe fn compute_bin_stats_row_major_root<T, const HESSIANS: bool>(
	bin_stats_for_feature: &mut [BinStatsEntry],
	examples_index: &[u32],
	binned_feature_values: &[T],
	gradients: &[f32],
	hessians: &[f32],
	n_features: usize,
) where
	T: ToPrimitive,
{
	if HESSIANS {
		for example_index in examples_index {
			let example_index = example_index.to_usize().unwrap();
			let gradient = *gradients.get_unchecked(example_index);
			let hessian = *hessians.get_unchecked(example_index);
			let binned_feature_row_start = example_index * n_features;
			for binned_feature_value_index in
				binned_feature_row_start..binned_feature_row_start + n_features
			{
				let bin_stats_index =
					binned_feature_values.get_unchecked(binned_feature_value_index);
				let bin_stats_index = bin_stats_index.to_usize().unwrap();
				let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_stats_index);
				bin_stats.sum_gradients += gradient as f64;
				bin_stats.sum_hessians += hessian as f64;
			}
		}
	} else {
		for example_index in examples_index {
			let example_index = example_index.to_usize().unwrap();
			let gradient = *gradients.get_unchecked(example_index);
			let binned_feature_row_start = example_index * n_features;
			for binned_feature_value_index in
				binned_feature_row_start..binned_feature_row_start + n_features
			{
				let bin_stats_index =
					binned_feature_values.get_unchecked(binned_feature_value_index);
				let bin_stats_index = bin_stats_index.to_usize().unwrap();
				let bin_stats = bin_stats_for_feature.get_unchecked_mut(bin_stats_index);
				bin_stats.sum_gradients += gradient as f64;
				bin_stats.sum_hessians += 1.0;
			}
		}
	}
}

pub unsafe fn compute_bin_stats_row_major_not_root<T, const HESSIANS: bool>(
	bin_stats: &mut [BinStatsEntry],
	examples_index: &[u32],
	binned_feature_values: &[T],
	gradients: &[f32],
	hessians: &[f32],
	n_features: usize,
) where
	T: ToPrimitive,
{
	if HESSIANS {
		let len = examples_index.len();
		#[cfg(target_arch = "x86_64")]
		let prefetch_len = len.saturating_sub(PREFETCH_OFFSET);
		#[cfg(not(target_arch = "x86_64"))]
		let prefetch_len = 0;
		for i in 0..prefetch_len {
			#[cfg(target_arch = "x86_64")]
			{
				let prefetch_index = *examples_index.get_unchecked(i + PREFETCH_OFFSET);
				let prefetch_index = prefetch_index.to_usize().unwrap();
				let prefetch_ptr = binned_feature_values
					.as_ptr()
					.add(prefetch_index * n_features) as *const i8;
				core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
				let prefetch_ptr = binned_feature_values
					.as_ptr()
					.add(prefetch_index * n_features + n_features - 1) as *const i8;
				core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
				core::arch::x86_64::_mm_prefetch(
					gradients.as_ptr().add(prefetch_index) as *const i8,
					core::arch::x86_64::_MM_HINT_T0,
				);
				core::arch::x86_64::_mm_prefetch(
					hessians.as_ptr().add(prefetch_index) as *const i8,
					core::arch::x86_64::_MM_HINT_T0,
				);
			}
			let example_index = *examples_index.get_unchecked(i);
			let example_index = example_index.to_usize().unwrap();
			let gradient = *gradients.get_unchecked(example_index);
			let hessian = *hessians.get_unchecked(example_index);
			let binned_feature_row_start = example_index * n_features;
			for binned_feature_value_index in
				binned_feature_row_start..binned_feature_row_start + n_features
			{
				let bin_stats_index =
					binned_feature_values.get_unchecked(binned_feature_value_index);
				let bin_stats_index = bin_stats_index.to_usize().unwrap();
				let bin_stats = bin_stats.get_unchecked_mut(bin_stats_index);
				bin_stats.sum_gradients += gradient as f64;
				bin_stats.sum_hessians += hessian as f64;
			}
		}
		for i in prefetch_len..len {
			let example_index = *examples_index.get_unchecked(i);
			let example_index = example_index.to_usize().unwrap();
			let gradient = *gradients.get_unchecked(example_index);
			let hessian = *hessians.get_unchecked(example_index);
			let binned_feature_row_start = example_index * n_features;
			for binned_feature_value_index in
				binned_feature_row_start..binned_feature_row_start + n_features
			{
				let bin_stats_index =
					binned_feature_values.get_unchecked(binned_feature_value_index);
				let bin_stats_index = bin_stats_index.to_usize().unwrap();
				let bin_stats = bin_stats.get_unchecked_mut(bin_stats_index);
				bin_stats.sum_gradients += gradient as f64;
				bin_stats.sum_hessians += hessian as f64;
			}
		}
	} else {
		let len = examples_index.len();
		#[cfg(target_arch = "x86_64")]
		let prefetch_len = len.saturating_sub(PREFETCH_OFFSET);
		#[cfg(not(target_arch = "x86_64"))]
		let prefetch_len = 0;
		for i in 0..prefetch_len {
			#[cfg(target_arch = "x86_64")]
			{
				let prefetch_index = *examples_index.get_unchecked(i + PREFETCH_OFFSET);
				let prefetch_index = prefetch_index.to_usize().unwrap();
				let prefetch_ptr = binned_feature_values
					.as_ptr()
					.add(prefetch_index * n_features) as *const i8;
				core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
				let prefetch_ptr = binned_feature_values
					.as_ptr()
					.add(prefetch_index * n_features + n_features - 1) as *const i8;
				core::arch::x86_64::_mm_prefetch(prefetch_ptr, core::arch::x86_64::_MM_HINT_T0);
				core::arch::x86_64::_mm_prefetch(
					gradients.as_ptr().add(prefetch_index) as *const i8,
					core::arch::x86_64::_MM_HINT_T0,
				);
			}
			let example_index = *examples_index.get_unchecked(i);
			let example_index = example_index.to_usize().unwrap();
			let gradient = *gradients.get_unchecked(example_index);
			let binned_feature_row_start = example_index * n_features;
			for binned_feature_value_index in
				binned_feature_row_start..binned_feature_row_start + n_features
			{
				let bin_stats_index =
					binned_feature_values.get_unchecked(binned_feature_value_index);
				let bin_stats_index = bin_stats_index.to_usize().unwrap();
				let bin_stats = bin_stats.get_unchecked_mut(bin_stats_index);
				bin_stats.sum_gradients += gradient as f64;
				bin_stats.sum_hessians += 1.0;
			}
		}
		for i in prefetch_len..len {
			let example_index = *examples_index.get_unchecked(i);
			let example_index = example_index.to_usize().unwrap();
			let ordered_gradient = *gradients.get_unchecked(example_index);
			let binned_feature_row_start = example_index * n_features;
			for binned_feature_value_index in
				binned_feature_row_start..binned_feature_row_start + n_features
			{
				let bin_stats_index =
					binned_feature_values.get_unchecked(binned_feature_value_index);
				let bin_stats_index = bin_stats_index.to_usize().unwrap();
				let bin_stats = bin_stats.get_unchecked_mut(bin_stats_index);
				bin_stats.sum_gradients += ordered_gradient as f64;
				bin_stats.sum_hessians += 1.0;
			}
		}
	}
}
