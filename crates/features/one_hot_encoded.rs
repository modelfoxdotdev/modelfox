use modelfox_table::prelude::*;
use ndarray::prelude::*;
use modelfox_zip::zip;

/**
A `OneHotEncodedFeatureGroup` creates one number feature for each variant in an enum column, plus one number feature for invalid values. For each example, all of the features will have the value 0.0, except the feature corresponding to the column's value, which will have the value 1.0.

# Example

```
use std::num::NonZeroUsize;
use modelfox_table::prelude::*;

EnumTableColumn::new(
  Some("color".to_owned()),
  vec!["red".to_owned(), "green".to_owned(), "blue".to_owned()],
  vec![None, Some(NonZeroUsize::new(1).unwrap()), Some(NonZeroUsize::new(2).unwrap()), Some(NonZeroUsize::new(3).unwrap())],
);
```

| input value     | feature values |
|-----------------|----------------|
| "INVALID!"      | [0, 0, 0]      |
| "red"           | [1, 0, 0]      |
| "green"         | [0, 1, 0]      |
| "blue"          | [0, 0, 1]      |
*/
#[derive(Clone, Debug)]
pub struct OneHotEncodedFeatureGroup {
	pub source_column_name: String,
	pub variants: Vec<String>,
}

impl OneHotEncodedFeatureGroup {
	pub fn compute_for_column(column: TableColumnView) -> OneHotEncodedFeatureGroup {
		match column {
			TableColumnView::Enum(column) => Self::compute_for_enum_column(column),
			_ => unimplemented!(),
		}
	}

	fn compute_for_enum_column(column: EnumTableColumnView) -> Self {
		Self {
			source_column_name: column.name().unwrap().to_owned(),
			variants: column.variants().to_owned(),
		}
	}
}

impl OneHotEncodedFeatureGroup {
	pub fn compute_array_f32(
		&self,
		features: ArrayViewMut2<f32>,
		column: TableColumnView,
		progress: &impl Fn(),
	) {
		match column {
			TableColumnView::Enum(column) => {
				self.compute_array_f32_for_enum_column(features, column, progress)
			}
			TableColumnView::Unknown(_) => unimplemented!(),
			TableColumnView::Number(_) => unimplemented!(),
			TableColumnView::Text(_) => unimplemented!(),
		}
	}

	fn compute_array_f32_for_enum_column(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: EnumTableColumnView,
		progress: &impl Fn(),
	) {
		// Fill the features with zeros.
		features.fill(0.0);
		// For each example, set the features corresponding to the enum value to one.
		for (mut features, value) in zip!(features.axis_iter_mut(Axis(0)), column.as_slice().iter())
		{
			let feature_index = value.map(|v| v.get()).unwrap_or(0);
			features[feature_index] = 1.0;
			progress();
		}
	}
}
