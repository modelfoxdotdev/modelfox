#[derive(PartialEq, Clone, Copy)]
pub enum ColumnType {
	Unknown,
	Number,
	Enum,
	Text,
}
