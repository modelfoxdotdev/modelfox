use modelfox_table::{
	NumberTableColumn, TableColumn, TableColumnView, TableValue, TextTableColumnView,
};
use modelfox_text::{Tokenizer, WordEmbeddingModel};
use ndarray::prelude::*;

#[derive(Clone, Debug)]
pub struct WordEmbeddingFeatureGroup {
	/// This is the name of the text column used to compute features with this feature group.
	pub source_column_name: String,
	/// This is the tokenizer used to split the text into tokens.
	pub tokenizer: Tokenizer,
	/// This is the word embedding model.
	pub model: WordEmbeddingModel,
}

impl WordEmbeddingFeatureGroup {
	pub fn compute_table(
		&self,
		column: modelfox_table::TableColumnView,
		progress: &impl Fn(u64),
	) -> Vec<TableColumn> {
		match column {
			TableColumnView::Unknown(_) => unimplemented!(),
			TableColumnView::Number(_) => unimplemented!(),
			TableColumnView::Enum(_) => unimplemented!(),
			TableColumnView::Text(column) => {
				self.compute_table_for_text_column(column, &|| progress(1))
			}
		}
	}

	pub fn compute_array_f32(
		&self,
		features: ArrayViewMut2<f32>,
		column: modelfox_table::TableColumnView,
		progress: &impl Fn(),
	) {
		match column {
			TableColumnView::Unknown(_) => unimplemented!(),
			TableColumnView::Number(_) => unimplemented!(),
			TableColumnView::Enum(_) => unimplemented!(),
			TableColumnView::Text(column) => {
				self.compute_array_f32_for_text_column(features, column, progress)
			}
		}
	}

	pub fn compute_array_value(
		&self,
		features: ArrayViewMut2<modelfox_table::TableValue>,
		column: modelfox_table::TableColumnView,
		progress: &impl Fn(),
	) {
		match column {
			TableColumnView::Unknown(_) => unimplemented!(),
			TableColumnView::Number(_) => unimplemented!(),
			TableColumnView::Enum(_) => unimplemented!(),
			TableColumnView::Text(column) => {
				self.compute_array_value_for_text_column(features, column, progress)
			}
		}
	}

	fn compute_array_f32_for_text_column(
		&self,
		mut features: ArrayViewMut2<f32>,
		column: TextTableColumnView,
		progress: &impl Fn(),
	) {
		// Fill the features with zeros.
		features.fill(0.0);
		// Compute the feature values for each example.
		for (example_index, value) in column.iter().enumerate() {
			// Set the feature value for each token for this example.
			let mut count = 0;
			for token in self.tokenizer.tokenize(value) {
				// Look up the token in the word model.
				if let Some(embedding) = self.model.get(token.as_ref()) {
					count += 1;
					for (index, value) in embedding.iter().enumerate() {
						*features.get_mut([example_index, index]).unwrap() += value;
					}
				}
			}
			// Average the word embeddings.
			if count > 0 {
				for feature_column_value in features.row_mut(example_index).iter_mut() {
					*feature_column_value /= count as f32;
				}
			}
			progress();
		}
	}

	fn compute_array_value_for_text_column(
		&self,
		mut features: ArrayViewMut2<TableValue>,
		column: TextTableColumnView,
		progress: &impl Fn(),
	) {
		// Fill the features with zeros.
		for feature in features.iter_mut() {
			*feature = TableValue::Number(0.0);
		}
		// Compute the feature values for each example.
		for (example_index, value) in column.iter().enumerate() {
			// Set the feature value for each token for this example.
			let mut count = 0;
			for token in self.tokenizer.tokenize(value) {
				// Look up the token in the word model.
				if let Some(embedding) = self.model.get(token.as_ref()) {
					count += 1;
					for (index, value) in embedding.iter().enumerate() {
						*features
							.get_mut([example_index, index])
							.unwrap()
							.as_number_mut()
							.unwrap() += value;
					}
				}
			}
			if count > 0 {
				// Average the word embeddings.
				for feature_column_value in features.row_mut(example_index).iter_mut() {
					*feature_column_value.as_number_mut().unwrap() /= count as f32;
				}
			}
			progress();
		}
	}

	fn compute_table_for_text_column(
		&self,
		column: modelfox_table::TextTableColumnView,
		progress: &impl Fn(),
	) -> Vec<TableColumn> {
		let mut feature_columns = vec![vec![0.0; column.len()]; self.model.size];
		for (example_index, value) in column.iter().enumerate() {
			// Set the feature value for each token for this example.
			let tokenizer = self.tokenizer.tokenize(value);
			let mut count = 0;
			for token in tokenizer {
				// Look up the token in the word model.
				if let Some(embedding) = self.model.get(token.as_ref()) {
					count += 1;
					for (index, value) in embedding.iter().enumerate() {
						feature_columns[index][example_index] += value;
					}
				}
			}
			if count > 0 {
				// Average the word embeddings.
				for feature_column in feature_columns.iter_mut() {
					feature_column[example_index] /= count as f32;
				}
			}
			progress();
		}
		feature_columns
			.into_iter()
			.map(|feature_column| TableColumn::Number(NumberTableColumn::new(None, feature_column)))
			.collect()
	}
}
