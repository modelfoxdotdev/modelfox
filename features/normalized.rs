use ndarray::prelude::*;
use num::ToPrimitive;
use tangram_table::{
	EnumTableColumnView, NumberTableColumn, NumberTableColumnView, TableColumn, TableColumnView,
	TableValue,
};
use tangram_zip::zip;

/**
A `NormalizedFeatureGroup` transforms a number column to zero mean and unit variance. [Learn more](https://en.wikipedia.org/wiki/Feature_scaling#Standardization_(Z-score_Normalization).

# Example

```
use tangram_table::prelude::*;

NumberTableColumn::new(
  Some("values".to_string()),
  vec![0.0, 5.2, 1.3, 10.0],
);
```

Mean: 2.16667

Standard Deviation: 2.70617

`feature_value =  (value - mean) / std`

| input value     | feature value                         |
|-----------------|---------------------------------------|
| 0.0             | (0.0 - 2.16667) / 2.70617  = -0.80064 |
| 5.2             | (5.2 - 2.16667) / 2.70617  = 1.12089  |
| 1.3             | (1.3 - 2.16667) / 2.70617  = -0.32026 |
*/
#[derive(Clone, Debug)]
pub struct NormalizedFeatureGroup {
	pub source_column_name: String,
	pub mean: f32,
	pub variance: f32,
}

impl NormalizedFeatureGroup {
	pub fn compute_for_column(column: TableColumnView) -> NormalizedFeatureGroup {
		match column {
			TableColumnView::Number(column) => Self::compute_for_number_column(column),
			_ => unimplemented!(),
		}
	}

	fn compute_for_number_column(column: NumberTableColumnView) -> Self {
		let mean_variance = tangram_metrics::MeanVariance::compute(column.view().as_slice());
		Self {
			source_column_name: column.name().unwrap().to_owned(),
			mean: mean_variance.mean,
			variance: mean_variance.variance,
		}
	}
}

impl NormalizedFeatureGroup {
	pub fn compute_table(&self, column: TableColumnView, progress: &impl Fn(u64)) -> TableColumn {
		// Set the feature values to the normalized source column values.
		match column {
			TableColumnView::Unknown(_) => unimplemented!(),
			TableColumnView::Number(column) => {
				TableColumn::Number(self.compute_table_for_number_column(column, &|| progress(1)))
			}
			TableColumnView::Enum(column) => {
				TableColumn::Number(self.compute_table_for_enum_column(column, &|| progress(1)))
			}
			TableColumnView::Text(_) => unimplemented!(),
		}
	}

	pub fn compute_array_f32(
		&self,
		features: ArrayViewMut2<f32>,
		column: TableColumnView,
		progress: &impl Fn(),
	) {
		// Set the feature values to the normalized source column values.
		match column {
			TableColumnView::Unknown(_) => unimplemented!(),
			TableColumnView::Number(column) => {
				self.compute_array_f32_for_number_column(features, column, progress)
			}
			TableColumnView::Enum(column) => {
				self.compute_array_f32_for_enum_column(features, column, progress)
			}
			TableColumnView::Text(_) => unimplemented!(),
		}
	}

	pub fn compute_array_value(
		&self,
		features: ArrayViewMut2<TableValue>,
		column: TableColumnView,
		progress: &impl Fn(),
	) {
		match column {
			TableColumnView::Unknown(_) => unimplemented!(),
			TableColumnView::Number(column) => {
				self.compute_array_value_for_number_column(features, column, progress)
			}
			TableColumnView::Enum(column) => {
				self.compute_array_value_for_enum_column(features, column, progress)
			}
			TableColumnView::Text(_) => unimplemented!(),
		}
	}

	fn compute_table_for_number_column(
		&self,
		column: NumberTableColumnView,
		progress: &impl Fn(),
	) -> NumberTableColumn {
		let mut feature_values = Vec::with_capacity(column.len());
		for value in column.iter() {
			let feature = if value.is_nan() || self.variance == 0.0 {
				0.0
			} else {
				(*value - self.mean) / f32::sqrt(self.variance)
			};
			feature_values.push(feature);
			progress()
		}
		NumberTableColumn::new(None, feature_values)
	}

	fn compute_table_for_enum_column(
		&self,
		column: EnumTableColumnView,
		progress: &impl Fn(),
	) -> NumberTableColumn {
		let mut feature_values = Vec::with_capacity(column.len());
		for value in column.iter() {
			let value = value
				.map(|value| value.get().to_f32().unwrap())
				.unwrap_or(0.0);
			let feature = if value.is_nan() || self.variance == 0.0 {
				0.0
			} else {
				(value - self.mean) / f32::sqrt(self.variance)
			};
			feature_values.push(feature);
			progress()
		}
		NumberTableColumn::new(None, feature_values)
	}

	fn compute_array_f32_for_number_column(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: NumberTableColumnView,
		progress: &impl Fn(),
	) {
		for (feature, value) in zip!(features.iter_mut(), column.iter()) {
			*feature = if value.is_nan() || self.variance == 0.0 {
				0.0
			} else {
				(*value - self.mean) / f32::sqrt(self.variance)
			};
			progress()
		}
	}

	fn compute_array_f32_for_enum_column(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: EnumTableColumnView,
		progress: &impl Fn(),
	) {
		for (feature, value) in zip!(features.iter_mut(), column.iter()) {
			let value = value
				.map(|value| value.get().to_f32().unwrap())
				.unwrap_or(0.0);
			*feature = if value.is_nan() || self.variance == 0.0 {
				0.0
			} else {
				(value - self.mean) / f32::sqrt(self.variance)
			};
			progress()
		}
	}

	fn compute_array_value_for_number_column(
		&self,
		mut features: ArrayViewMut2<TableValue>,
		column: NumberTableColumnView,
		progress: &impl Fn(),
	) {
		for (feature, value) in zip!(features.column_mut(0), column.iter()) {
			*feature = if value.is_nan() || self.variance == 0.0 {
				TableValue::Number(0.0)
			} else {
				TableValue::Number((value - self.mean) / f32::sqrt(self.variance))
			};
			progress()
		}
	}

	fn compute_array_value_for_enum_column(
		&self,
		mut features: ArrayViewMut2<TableValue>,
		column: EnumTableColumnView,
		progress: &impl Fn(),
	) {
		for (feature, value) in zip!(features.column_mut(0), column.iter()) {
			*feature = if value.is_none() || self.variance == 0.0 {
				TableValue::Number(0.0)
			} else {
				TableValue::Number(
					(value.unwrap().get().to_f32().unwrap() - self.mean) / f32::sqrt(self.variance),
				)
			};
			progress()
		}
	}
}
