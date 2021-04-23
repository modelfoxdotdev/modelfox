pub fn redirect_to_login() -> http::Response<hyper::Body> {
	http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(http::header::LOCATION, "/login")
		.body(hyper::Body::empty())
		.unwrap()
}

/// 400
pub fn bad_request() -> http::Response<hyper::Body> {
	http::Response::builder()
		.status(http::StatusCode::BAD_REQUEST)
		.body(hyper::Body::from("bad request"))
		.unwrap()
}

/// 401
pub fn unauthorized() -> http::Response<hyper::Body> {
	http::Response::builder()
		.status(http::StatusCode::UNAUTHORIZED)
		.body(hyper::Body::from("unauthorized"))
		.unwrap()
}

/// 403
pub fn forbidden() -> http::Response<hyper::Body> {
	http::Response::builder()
		.status(http::StatusCode::FORBIDDEN)
		.body(hyper::Body::from("forbidden"))
		.unwrap()
}

/// 404
pub fn not_found() -> http::Response<hyper::Body> {
	http::Response::builder()
		.status(http::StatusCode::NOT_FOUND)
		.body(hyper::Body::from("not found"))
		.unwrap()
}

/// 503
pub fn service_unavailable() -> http::Response<hyper::Body> {
	http::Response::builder()
		.status(http::StatusCode::SERVICE_UNAVAILABLE)
		.body(hyper::Body::from("service unavailable"))
		.unwrap()
}
