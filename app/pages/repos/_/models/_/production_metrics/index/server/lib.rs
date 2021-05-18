mod binary_classifier;
mod get;
mod multiclass_classifier;
mod page;
mod regressor;

use futures::FutureExt;
use std::sync::Arc;
use tangram_app_common::{error::method_not_allowed, Context, HandleOutput};

pub fn handle(context: Arc<Context>, request: http::Request<hyper::Body>) -> HandleOutput {
	match request.method() {
		&http::Method::GET => self::get::get(context, request).boxed(),
		_ => return async { Ok(method_not_allowed()) }.boxed(),
	}
}
