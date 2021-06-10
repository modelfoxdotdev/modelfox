use include_out_dir::IncludeOutDir;
use std::path::Path;
use tangram_error::Result;

pub async fn serve_from_dir(
	dir: &Path,
	request: &http::Request<hyper::Body>,
) -> Result<Option<http::Response<hyper::Body>>> {
	let method = request.method().clone();
	let uri = request.uri().clone();
	let path_and_query = uri.path_and_query().unwrap();
	let path = path_and_query.path();
	if method != ::http::Method::GET {
		return Ok(None);
	}
	let path = dir.join(path.strip_prefix('/').unwrap());
	let path_exists = std::fs::metadata(&path)
		.map(|metadata| metadata.is_file())
		.unwrap_or(false);
	if !path_exists {
		return Ok(None);
	}
	let data = tokio::fs::read(&path).await?;
	let mut response = http::Response::builder();
	if let Some(content_type) = content_type(&path) {
		response = response.header(http::header::CONTENT_TYPE, content_type);
	}
	response = response.status(http::StatusCode::OK);
	let response = response.body(hyper::Body::from(data)).unwrap();
	Ok(Some(response))
}

pub async fn serve_from_include_out_dir(
	dir: &IncludeOutDir,
	request: &http::Request<hyper::Body>,
) -> Result<Option<http::Response<hyper::Body>>> {
	let method = request.method().clone();
	let uri = request.uri().clone();
	let path_and_query = uri.path_and_query().unwrap();
	let path = path_and_query.path();
	if method != ::http::Method::GET {
		return Ok(None);
	}
	let path = Path::new(path.strip_prefix('/').unwrap());
	let file = if let Some(file) = dir.read(&path) {
		file
	} else {
		return Ok(None);
	};
	let data = file.data;
	let hash = &file.hash;
	let mut response = http::Response::builder();
	if let Some(content_type) = content_type(&path) {
		response = response.header(http::header::CONTENT_TYPE, content_type);
	}
	response = response.header(http::header::ETAG, hash);
	if let Some(etag) = request.headers().get(http::header::IF_NONE_MATCH) {
		if etag.as_bytes() == hash.as_bytes() {
			response = response.status(http::StatusCode::NOT_MODIFIED);
			let response = response.body(hyper::Body::empty()).unwrap();
			return Ok(Some(response));
		}
	}
	response = response.status(http::StatusCode::OK);
	let response = response.body(hyper::Body::from(data)).unwrap();
	Ok(Some(response))
}

fn content_type(path: &std::path::Path) -> Option<&'static str> {
	let path = path.to_str().unwrap();
	if path.ends_with(".css") {
		Some("text/css")
	} else if path.ends_with(".js") {
		Some("text/javascript")
	} else if path.ends_with(".svg") {
		Some("image/svg+xml")
	} else if path.ends_with(".wasm") {
		Some("application/wasm")
	} else {
		None
	}
}
