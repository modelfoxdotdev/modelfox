use futures::FutureExt;
use tangram_app_common::error::method_not_allowed;

mod get;
mod page;

pub fn init() -> sunfish::Page {
	sunfish::Page::new_dynamic(|request| match *request.method() {
		http::Method::GET => self::get::get(request).boxed(),
		_ => async { Ok(method_not_allowed()) }.boxed(),
	})
}
