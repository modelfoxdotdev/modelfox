/*!
This crate implements two dimensional collections where each column can have a different data type, like a spreadsheet or database.
*/

pub use self::load::{FromCsvOptions, LoadProgressEvent};
use fnv::FnvHashMap;
use ndarray::prelude::*;
use num::ToPrimitive;
use std::num::NonZeroUsize;
use tangram_zip::zip;

mod load;

pub mod prelude {
	pub use super::{
		EnumTableColumn, EnumTableColumnView, NumberTableColumn, NumberTableColumnView, Table,
		TableColumn, TableColumnType, TableColumnView, TableValue, TableView, TableViewMut,
		TextTableColumn, TextTableColumnView, TextTableColumnViewMut, UnknownTableColumn,
		UnknownTableColumnView,
	};
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
	columns: Vec<TableColumn>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableColumn {
	Unknown(UnknownTableColumn),
	Number(NumberTableColumn),
	Enum(EnumTableColumn),
	Text(TextTableColumn),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnknownTableColumn {
	name: Option<String>,
	len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberTableColumn {
	name: Option<String>,
	data: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumTableColumn {
	name: Option<String>,
	variants: Vec<String>,
	data: Vec<Option<NonZeroUsize>>,
	variants_map: FnvHashMap<String, NonZeroUsize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextTableColumn {
	name: Option<String>,
	data: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableView<'a> {
	columns: Vec<TableColumnView<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableColumnView<'a> {
	Unknown(UnknownTableColumnView<'a>),
	Number(NumberTableColumnView<'a>),
	Enum(EnumTableColumnView<'a>),
	Text(TextTableColumnView<'a>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnknownTableColumnView<'a> {
	name: Option<&'a str>,
	len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberTableColumnView<'a> {
	name: Option<&'a str>,
	data: &'a [f32],
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumTableColumnView<'a> {
	name: Option<&'a str>,
	variants: &'a [String],
	data: &'a [Option<NonZeroUsize>],
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextTableColumnView<'a> {
	name: Option<&'a str>,
	data: &'a [String],
}

#[derive(Debug, PartialEq)]
pub struct TableViewMut<'a> {
	columns: Vec<TableColumnViewMut<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum TableColumnViewMut<'a> {
	Number(NumberTableColumnViewMut<'a>),
	Enum(EnumTableColumnViewMut<'a>),
	Text(TextTableColumnViewMut<'a>),
}

#[derive(Debug, PartialEq)]
pub struct NumberTableColumnViewMut<'a> {
	name: Option<&'a mut str>,
	data: &'a mut [f32],
}

#[derive(Debug, PartialEq)]
pub struct EnumTableColumnViewMut<'a> {
	name: Option<&'a mut str>,
	variants: &'a mut [String],
	data: &'a mut [usize],
}

#[derive(Debug, PartialEq)]
pub struct TextTableColumnViewMut<'a> {
	name: Option<&'a mut str>,
	data: &'a mut [String],
}

#[derive(Debug, Clone)]
pub enum TableColumnType {
	Unknown,
	Number,
	Enum { variants: Vec<String> },
	Text,
}

#[derive(Debug, Clone)]
pub enum TableColumnTypeView<'a> {
	Unknown,
	Number,
	Enum { variants: &'a [String] },
	Text,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TableValue<'a> {
	Unknown,
	Number(f32),
	Enum(Option<NonZeroUsize>),
	Text(&'a str),
}

impl Table {
	pub fn new(column_names: Vec<Option<String>>, column_types: Vec<TableColumnType>) -> Table {
		let columns = zip!(column_names, column_types)
			.map(|(column_name, column_type)| match column_type {
				TableColumnType::Unknown => {
					TableColumn::Unknown(UnknownTableColumn::new(column_name))
				}
				TableColumnType::Number => {
					TableColumn::Number(NumberTableColumn::new(column_name, Vec::new()))
				}
				TableColumnType::Enum { variants } => {
					TableColumn::Enum(EnumTableColumn::new(column_name, variants, Vec::new()))
				}
				TableColumnType::Text => {
					TableColumn::Text(TextTableColumn::new(column_name, Vec::new()))
				}
			})
			.collect();
		Table { columns }
	}

	pub fn columns(&self) -> &Vec<TableColumn> {
		&self.columns
	}

	pub fn columns_mut(&mut self) -> &mut Vec<TableColumn> {
		&mut self.columns
	}

	pub fn ncols(&self) -> usize {
		self.columns.len()
	}

	pub fn nrows(&self) -> usize {
		self.columns.first().map(|column| column.len()).unwrap_or(0)
	}

	pub fn view(&self) -> TableView {
		let columns = self.columns.iter().map(|column| column.view()).collect();
		TableView { columns }
	}

	pub fn to_rows_f32(&self) -> Option<Array2<f32>> {
		let mut features_train = Array::zeros((self.nrows(), self.ncols()));
		for (mut ndarray_column, table_column) in
			zip!(features_train.axis_iter_mut(Axis(1)), self.columns.iter())
		{
			match table_column {
				TableColumn::Number(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = *b;
					}
				}
				TableColumn::Enum(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = b.map(|b| b.get().to_f32().unwrap()).unwrap_or(0.0);
					}
				}
				_ => return None,
			}
		}
		Some(features_train)
	}

	pub fn to_rows(&self) -> Array2<TableValue> {
		let mut rows = Array::from_elem((self.nrows(), self.ncols()), TableValue::Unknown);
		for (mut ndarray_column, table_column) in
			zip!(rows.axis_iter_mut(Axis(1)), self.columns.iter())
		{
			match table_column {
				TableColumn::Unknown(_) => ndarray_column.fill(TableValue::Unknown),
				TableColumn::Number(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = TableValue::Number(*b);
					}
				}
				TableColumn::Enum(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = TableValue::Enum(*b);
					}
				}
				TableColumn::Text(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data.as_slice()) {
						*a = TableValue::Text(b);
					}
				}
			}
		}
		rows
	}
}

impl TableColumn {
	pub fn len(&self) -> usize {
		match self {
			TableColumn::Unknown(s) => s.len(),
			TableColumn::Number(s) => s.len(),
			TableColumn::Enum(s) => s.len(),
			TableColumn::Text(s) => s.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			TableColumn::Unknown(s) => s.len == 0,
			TableColumn::Number(s) => s.data.is_empty(),
			TableColumn::Enum(s) => s.data.is_empty(),
			TableColumn::Text(s) => s.data.is_empty(),
		}
	}

	pub fn name(&self) -> Option<&str> {
		match self {
			TableColumn::Unknown(s) => s.name.as_deref(),
			TableColumn::Number(s) => s.name.as_deref(),
			TableColumn::Enum(s) => s.name.as_deref(),
			TableColumn::Text(s) => s.name.as_deref(),
		}
	}

	pub fn as_number(&self) -> Option<&NumberTableColumn> {
		match self {
			TableColumn::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<&EnumTableColumn> {
		match self {
			TableColumn::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<&TextTableColumn> {
		match self {
			TableColumn::Text(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_number_mut(&mut self) -> Option<&mut NumberTableColumn> {
		match self {
			TableColumn::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum_mut(&mut self) -> Option<&mut EnumTableColumn> {
		match self {
			TableColumn::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text_mut(&mut self) -> Option<&mut TextTableColumn> {
		match self {
			TableColumn::Text(s) => Some(s),
			_ => None,
		}
	}

	pub fn view(&self) -> TableColumnView {
		match self {
			TableColumn::Unknown(column) => TableColumnView::Unknown(column.view()),
			TableColumn::Number(column) => TableColumnView::Number(column.view()),
			TableColumn::Enum(column) => TableColumnView::Enum(column.view()),
			TableColumn::Text(column) => TableColumnView::Text(column.view()),
		}
	}
}

impl UnknownTableColumn {
	pub fn new(name: Option<String>) -> UnknownTableColumn {
		UnknownTableColumn { name, len: 0 }
	}

	pub fn name(&self) -> &Option<String> {
		&self.name
	}

	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn len_mut(&mut self) -> &mut usize {
		&mut self.len
	}

	pub fn view(&self) -> UnknownTableColumnView {
		UnknownTableColumnView {
			name: self.name.as_deref(),
			len: self.len,
		}
	}
}

impl NumberTableColumn {
	pub fn new(name: Option<String>, data: Vec<f32>) -> NumberTableColumn {
		NumberTableColumn { name, data }
	}

	pub fn name(&self) -> &Option<String> {
		&self.name
	}

	pub fn is_empty(&self) -> bool {
		self.data.len() == 0
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &f32> {
		self.data.iter()
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f32> {
		self.data.iter_mut()
	}

	pub fn data_mut(&mut self) -> &mut Vec<f32> {
		&mut self.data
	}

	pub fn view(&self) -> NumberTableColumnView {
		NumberTableColumnView {
			name: self.name.as_deref(),
			data: &self.data,
		}
	}
}

impl EnumTableColumn {
	pub fn new(
		name: Option<String>,
		variants: Vec<String>,
		data: Vec<Option<NonZeroUsize>>,
	) -> EnumTableColumn {
		let variants_map = variants
			.iter()
			.cloned()
			.enumerate()
			.map(|(i, variant)| (variant, NonZeroUsize::new(i + 1).unwrap()))
			.collect();
		EnumTableColumn {
			name,
			variants,
			data,
			variants_map,
		}
	}

	pub fn name(&self) -> &Option<String> {
		&self.name
	}

	pub fn variants(&self) -> &[String] {
		&self.variants
	}

	pub fn is_empty(&self) -> bool {
		self.data.len() == 0
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &Option<NonZeroUsize>> {
		self.data.iter()
	}

	pub fn data_mut(&mut self) -> &mut Vec<Option<NonZeroUsize>> {
		&mut self.data
	}

	pub fn view(&self) -> EnumTableColumnView {
		EnumTableColumnView {
			name: self.name.as_deref(),
			data: &self.data,
			variants: &self.variants,
		}
	}

	pub fn value_for_variant(&self, variant: &str) -> Option<NonZeroUsize> {
		self.variants_map.get(variant).cloned()
	}
}

impl TextTableColumn {
	pub fn new(name: Option<String>, data: Vec<String>) -> TextTableColumn {
		TextTableColumn { name, data }
	}

	pub fn name(&self) -> &Option<String> {
		&self.name
	}

	pub fn is_empty(&self) -> bool {
		self.data.len() == 0
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &String> {
		self.data.iter()
	}

	pub fn data_mut(&mut self) -> &mut Vec<String> {
		&mut self.data
	}

	pub fn view(&self) -> TextTableColumnView {
		TextTableColumnView {
			name: self.name.as_deref(),
			data: &self.data,
		}
	}
}

impl<'a> TableView<'a> {
	pub fn columns(&self) -> &Vec<TableColumnView<'a>> {
		&self.columns
	}

	pub fn view_columns(&self, column_indexes: &[usize]) -> TableView {
		let mut columns = Vec::with_capacity(column_indexes.len());
		for column_index in column_indexes {
			columns.push(self.columns[*column_index].clone())
		}
		Self { columns }
	}

	pub fn ncols(&self) -> usize {
		self.columns.len()
	}

	pub fn nrows(&self) -> usize {
		self.columns.first().map(|column| column.len()).unwrap_or(0)
	}

	pub fn view(&self) -> TableView {
		self.clone()
	}

	pub fn read_row(&self, index: usize, row: &mut [TableValue<'a>]) {
		for (value, column) in zip!(row.iter_mut(), self.columns.iter()) {
			*value = match column {
				TableColumnView::Unknown(_) => TableValue::Unknown,
				TableColumnView::Number(column) => TableValue::Number(column.data[index]),
				TableColumnView::Enum(column) => TableValue::Enum(column.data[index]),
				TableColumnView::Text(column) => TableValue::Text(&column.data[index]),
			}
		}
	}

	pub fn split_at_row(&self, index: usize) -> (TableView<'a>, TableView<'a>) {
		let iter = self.columns.iter().map(|column| column.split_at_row(index));
		let mut columns_a = Vec::with_capacity(self.columns.len());
		let mut columns_b = Vec::with_capacity(self.columns.len());
		for (column_a, column_b) in iter {
			columns_a.push(column_a);
			columns_b.push(column_b);
		}
		(
			TableView { columns: columns_a },
			TableView { columns: columns_b },
		)
	}

	pub fn to_rows_f32(&self) -> Option<Array2<f32>> {
		let mut features_train = Array::zeros((self.nrows(), self.ncols()));
		for (mut ndarray_column, table_column) in
			zip!(features_train.axis_iter_mut(Axis(1)), self.columns.iter())
		{
			match table_column {
				TableColumnView::Number(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data) {
						*a = *b;
					}
				}
				TableColumnView::Enum(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data) {
						*a = b.unwrap().get().to_f32().unwrap();
					}
				}
				_ => return None,
			}
		}
		Some(features_train)
	}

	pub fn to_rows(&self) -> Array2<TableValue<'a>> {
		let mut rows = Array::from_elem((self.nrows(), self.ncols()), TableValue::Unknown);
		for (mut ndarray_column, table_column) in
			zip!(rows.axis_iter_mut(Axis(1)), self.columns.iter())
		{
			match table_column {
				TableColumnView::Unknown(_) => ndarray_column.fill(TableValue::Unknown),
				TableColumnView::Number(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data) {
						*a = TableValue::Number(*b);
					}
				}
				TableColumnView::Enum(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data) {
						*a = TableValue::Enum(*b);
					}
				}
				TableColumnView::Text(column) => {
					for (a, b) in zip!(ndarray_column.iter_mut(), column.data) {
						*a = TableValue::Text(b);
					}
				}
			}
		}
		rows
	}
}

impl<'a> TableColumnView<'a> {
	pub fn len(&self) -> usize {
		match self {
			TableColumnView::Unknown(s) => s.len,
			TableColumnView::Number(s) => s.data.len(),
			TableColumnView::Enum(s) => s.data.len(),
			TableColumnView::Text(s) => s.data.len(),
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			TableColumnView::Unknown(s) => s.len == 0,
			TableColumnView::Number(s) => s.data.is_empty(),
			TableColumnView::Enum(s) => s.data.is_empty(),
			TableColumnView::Text(s) => s.data.is_empty(),
		}
	}

	pub fn name(&self) -> Option<&str> {
		match self {
			TableColumnView::Unknown(s) => s.name,
			TableColumnView::Number(s) => s.name,
			TableColumnView::Enum(s) => s.name,
			TableColumnView::Text(s) => s.name,
		}
	}

	pub fn column_type(&self) -> TableColumnTypeView {
		match self {
			TableColumnView::Unknown(_) => TableColumnTypeView::Unknown,
			TableColumnView::Number(_) => TableColumnTypeView::Number,
			TableColumnView::Enum(column) => TableColumnTypeView::Enum {
				variants: column.variants,
			},
			TableColumnView::Text(_) => TableColumnTypeView::Text,
		}
	}

	pub fn as_number(&self) -> Option<NumberTableColumnView> {
		match self {
			TableColumnView::Number(s) => Some(s.clone()),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<EnumTableColumnView> {
		match self {
			TableColumnView::Enum(s) => Some(s.clone()),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<TextTableColumnView> {
		match self {
			TableColumnView::Text(s) => Some(s.clone()),
			_ => None,
		}
	}

	pub fn split_at_row(&self, index: usize) -> (TableColumnView<'a>, TableColumnView<'a>) {
		match self {
			TableColumnView::Unknown(column) => (
				TableColumnView::Unknown(UnknownTableColumnView {
					name: column.name,
					len: index,
				}),
				TableColumnView::Unknown(UnknownTableColumnView {
					name: column.name,
					len: column.len - index,
				}),
			),
			TableColumnView::Number(column) => {
				let (data_a, data_b) = column.data.split_at(index);
				(
					TableColumnView::Number(NumberTableColumnView {
						name: column.name,
						data: data_a,
					}),
					TableColumnView::Number(NumberTableColumnView {
						name: column.name,
						data: data_b,
					}),
				)
			}
			TableColumnView::Enum(column) => {
				let (data_a, data_b) = column.data.split_at(index);
				(
					TableColumnView::Enum(EnumTableColumnView {
						name: column.name,
						variants: column.variants,
						data: data_a,
					}),
					TableColumnView::Enum(EnumTableColumnView {
						name: column.name,
						variants: column.variants,
						data: data_b,
					}),
				)
			}
			TableColumnView::Text(column) => {
				let (data_a, data_b) = column.data.split_at(index);
				(
					TableColumnView::Text(TextTableColumnView {
						name: column.name,
						data: data_a,
					}),
					TableColumnView::Text(TextTableColumnView {
						name: column.name,
						data: data_b,
					}),
				)
			}
		}
	}

	pub fn view(&self) -> TableColumnView {
		match self {
			TableColumnView::Unknown(s) => TableColumnView::Unknown(s.view()),
			TableColumnView::Number(s) => TableColumnView::Number(s.view()),
			TableColumnView::Enum(s) => TableColumnView::Enum(s.view()),
			TableColumnView::Text(s) => TableColumnView::Text(s.view()),
		}
	}
}

impl<'a> UnknownTableColumnView<'a> {
	pub fn name(&self) -> Option<&str> {
		self.name
	}

	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn view(&self) -> UnknownTableColumnView {
		self.clone()
	}
}

impl<'a> NumberTableColumnView<'a> {
	pub fn name(&self) -> Option<&str> {
		self.name
	}

	pub fn data(&self) -> &[f32] {
		self.data
	}

	pub fn is_empty(&self) -> bool {
		self.data.len() == 0
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &f32> {
		self.data.iter()
	}

	pub fn as_slice(&self) -> &[f32] {
		self.data
	}

	pub fn view(&self) -> NumberTableColumnView {
		self.clone()
	}
}

impl<'a> EnumTableColumnView<'a> {
	pub fn name(&self) -> Option<&str> {
		self.name
	}

	pub fn data(&self) -> &[Option<NonZeroUsize>] {
		self.data
	}

	pub fn variants(&self) -> &[String] {
		self.variants
	}

	pub fn is_empty(&self) -> bool {
		self.data.len() == 0
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &Option<NonZeroUsize>> {
		self.data.iter()
	}

	pub fn as_slice(&self) -> &[Option<NonZeroUsize>] {
		self.data
	}

	pub fn view(&self) -> EnumTableColumnView {
		self.clone()
	}
}

impl<'a> TextTableColumnView<'a> {
	pub fn name(&self) -> Option<&str> {
		self.name
	}

	pub fn data(&self) -> &'a [String] {
		self.data
	}

	pub fn is_empty(&self) -> bool {
		self.data.len() == 0
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &String> {
		self.data.iter()
	}

	pub fn as_slice(&self) -> &[String] {
		self.data
	}

	pub fn view(&self) -> TextTableColumnView {
		self.clone()
	}
}

impl<'a> TableValue<'a> {
	pub fn as_number(&self) -> Option<&f32> {
		match self {
			TableValue::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_number_mut(&mut self) -> Option<&mut f32> {
		match self {
			TableValue::Number(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum(&self) -> Option<&Option<NonZeroUsize>> {
		match self {
			TableValue::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_enum_mut(&mut self) -> Option<&mut Option<NonZeroUsize>> {
		match self {
			TableValue::Enum(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text(&self) -> Option<&str> {
		match self {
			TableValue::Text(s) => Some(s),
			_ => None,
		}
	}

	pub fn as_text_mut(&mut self) -> Option<&mut &'a str> {
		match self {
			TableValue::Text(s) => Some(s),
			_ => None,
		}
	}
}
