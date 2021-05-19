mod enum_column;
mod get;
mod number_column;
mod page;
mod text_column;

pub use enum_column::{
	EnumColumnInvalidValuesSection, EnumColumnOverallHistogramEntry, EnumColumnStatsSection,
	EnumColumnUniqueValuesSection, EnumInvalidValuesTable, EnumInvalidValuesTableRow,
	EnumUniqueValuesTable, EnumUniqueValuesTableRow,
};
use futures::FutureExt;
use std::sync::Arc;
use tangram_app_common::{error::method_not_allowed, Context, HandleOutput};

pub fn handle(context: Arc<Context>, request: http::Request<hyper::Body>) -> HandleOutput {
	match *request.method() {
		http::Method::GET => self::get::get(context, request).boxed(),
		_ => async { Ok(method_not_allowed()) }.boxed(),
	}
}
