use futures::FutureExt;
use tangram_app_common::error::method_not_allowed;

mod post;

pub fn init() -> sunfish::Route {
	sunfish::Route::new_dynamic(|request| match *request.method() {
		http::Method::POST => self::post::post(request).boxed(),
		_ => async { Ok(method_not_allowed()) }.boxed(),
	})
}
