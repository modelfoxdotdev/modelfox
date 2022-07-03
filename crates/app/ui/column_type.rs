#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ColumnType {
	Unknown,
	Number,
	Enum,
	Text,
}
