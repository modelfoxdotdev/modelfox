use futures::FutureExt;
use std::sync::Arc;
use tangram_app_common::{error::method_not_allowed, Context, HandleOutput};

mod get;
mod page;
mod post;

pub fn handle(context: Arc<Context>, request: http::Request<hyper::Body>) -> HandleOutput {
	match *request.method() {
		http::Method::GET => self::get::get(context, request).boxed(),
		http::Method::POST => self::post::post(context, request).boxed(),
		_ => async { Ok(method_not_allowed()) }.boxed(),
	}
}
