mod enum_column;
mod get;
mod number_column;
mod page;
mod text_column;

use futures::FutureExt;
use tangram_app_core::error::method_not_allowed;

pub fn init() -> sunfish::Route {
	sunfish::Route::new_dynamic(|request| match *request.method() {
		http::Method::GET => self::get::get(request).boxed(),
		_ => async { Ok(method_not_allowed()) }.boxed(),
	})
}
