use std::task::Poll;
use tangram_id::Id;

pub struct RequestIdLayer;

impl RequestIdLayer {
	pub fn new() -> RequestIdLayer {
		RequestIdLayer
	}
}

impl Default for RequestIdLayer {
	fn default() -> Self {
		Self::new()
	}
}

impl<S> tower::Layer<S> for RequestIdLayer {
	type Service = RequestIdService<S>;
	fn layer(&self, service: S) -> Self::Service {
		RequestIdService { service }
	}
}

#[derive(Clone)]
pub struct RequestIdService<S> {
	service: S,
}

impl<S> tower::Service<http::Request<hyper::Body>> for RequestIdService<S>
where
	S: tower::Service<http::Request<hyper::Body>>,
{
	type Response = S::Response;
	type Error = S::Error;
	type Future = S::Future;

	fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
		self.service.poll_ready(cx)
	}

	fn call(&mut self, mut request: http::Request<hyper::Body>) -> Self::Future {
		request.extensions_mut().insert(Id::generate());
		self.service.call(request)
	}
}
