mod enum_column;
mod get;
mod number_column;
mod page;
mod text_column;

use futures::FutureExt;
use tangram_app_common::{error::method_not_allowed, HandleOutput};

pub fn handle(request: &mut http::Request<hyper::Body>) -> HandleOutput {
	match *request.method() {
		http::Method::GET => self::get::get(request).boxed(),
		_ => async { Ok(method_not_allowed()) }.boxed(),
	}
}
