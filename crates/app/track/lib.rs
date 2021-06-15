use futures::FutureExt;
use tangram_app_common::{error::method_not_allowed, HandleOutput};

mod post;

pub fn handle(request: &mut http::Request<hyper::Body>) -> HandleOutput {
	match *request.method() {
		http::Method::POST => self::post::post(request).boxed(),
		_ => async { Ok(method_not_allowed()) }.boxed(),
	}
}
