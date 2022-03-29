use futures::FutureExt;
use modelfox_app_core::error::method_not_allowed;

mod binary_classifier;
mod get;
mod multiclass_classifier;
mod page;
mod regressor;

pub fn init() -> sunfish::Route {
	sunfish::Route::new_dynamic(|request| match *request.method() {
		http::Method::GET => self::get::get(request).boxed(),
		_ => async { Ok(method_not_allowed()) }.boxed(),
	})
}
