mod get;

pub use self::get::get;

// use futures::FutureExt;
// use tangram_app_common::Context;
// use tangram_error::Result;
// pub fn handle(
// 	context: &Context,
// 	request: http::Request<hyper::Body>,
// ) -> std::pin::Pin<
// 	Box<dyn std::future::Future<Output = Option<Result<http::Response<hyper::Body>>>> + '_>,
// > {
// 	match request.method() {
// 		&http::Method::GET => self::get::get(context, request).map(Some).boxed(),
// 		_ => futures::future::ready(None).boxed(),
// 	}
// }
