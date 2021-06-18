use futures::FutureExt;
use tangram_app_common::error::method_not_allowed;

mod binary_classifier;
mod get;
mod multiclass_classifier;
mod page;
mod regressor;

pub fn init() -> sunfish::Page {
	sunfish::Page::new_dynamic(|request| match *request.method() {
		http::Method::GET => self::get::get(request).boxed(),
		_ => async { Ok(method_not_allowed()) }.boxed(),
	})
}
