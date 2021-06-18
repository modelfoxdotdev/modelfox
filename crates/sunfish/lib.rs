use self::embed::EmbeddedDirectory;
pub use self::{
	build::{build, BuildOptions},
	hash::hash,
};
use futures::FutureExt;
use ignore::Walk;
use std::{
	future::Future,
	path::{Path, PathBuf},
	pin::Pin,
};
pub use sunfish_macro::init;
use tangram_error::Result;

mod build;
pub mod embed;
mod hash;

pub enum Page {
	Static {
		paths: Option<Box<dyn 'static + Send + Sync + Fn() -> Vec<String>>>,
		handler: Box<dyn 'static + Send + Sync + Fn(String) -> String>,
	},
	Dynamic {
		handler: DynamicHandler,
	},
}

pub type DynamicHandler = Box<
	dyn Send + Sync + for<'a> Fn(&'a mut http::Request<hyper::Body>) -> DynamicHandlerOutput<'a>,
>;

pub type DynamicHandlerOutput<'a> =
	Pin<Box<dyn 'a + Send + Future<Output = Result<http::Response<hyper::Body>>>>>;

impl Page {
	pub fn new_static<H>(handler: H) -> Page
	where
		H: 'static + Send + Sync + Fn(String) -> String,
	{
		Page::Static {
			paths: None,
			handler: Box::new(handler),
		}
	}

	pub fn new_static_with_paths<P, H>(paths: P, handler: H) -> Page
	where
		P: 'static + Send + Sync + Fn() -> Vec<String>,
		H: 'static + Send + Sync + Fn(String) -> String,
	{
		Page::Static {
			paths: Some(Box::new(paths)),
			handler: Box::new(handler),
		}
	}

	pub fn new_dynamic<H>(handler: H) -> Page
	where
		H: 'static
			+ Send
			+ Sync
			+ for<'a> Fn(&'a mut http::Request<hyper::Body>) -> DynamicHandlerOutput<'a>,
	{
		Page::Dynamic {
			handler: Box::new(handler),
		}
	}

	pub fn handle<'a>(
		&self,
		request: &'a mut http::Request<hyper::Body>,
	) -> DynamicHandlerOutput<'a> {
		match self {
			Page::Static { handler, .. } => {
				let html = handler(request.uri().path().to_owned());
				async {
					let response = http::Response::builder()
						.status(http::StatusCode::OK)
						.body(hyper::Body::from(html))
						.unwrap();
					Ok(response)
				}
				.boxed()
			}
			Page::Dynamic { handler } => handler(request),
		}
	}
}

pub fn path_components(path: &str) -> Vec<&str> {
	path.split('/').skip(1).collect::<Vec<_>>()
}

pub fn asset_path(path: &Path) -> String {
	let extension = path.extension().map(|e| e.to_str().unwrap()).unwrap();
	let hash = hash(&path.to_str().unwrap().as_bytes());
	format!("/assets/{}.{}", hash, extension)
}

pub struct ClientPaths {
	pub path_js: String,
	pub path_wasm: String,
}

pub fn client_paths(crate_name: &'static str) -> ClientPaths {
	let hash = hash(crate_name.as_bytes());
	ClientPaths {
		path_js: format!("/js/{}.js", hash),
		path_wasm: format!("/js/{}_bg.wasm", hash),
	}
}

pub enum Sunfish {
	Debug(DebugSunfish),
	Release(ReleaseSunfish),
}

type RoutesHandler = Box<
	dyn Send + Sync + for<'a> Fn(&'a mut http::Request<hyper::Body>) -> RoutesHandlerOutput<'a>,
>;

type RoutesHandlerOutput<'a> =
	Pin<Box<dyn 'a + Send + Future<Output = Result<Option<http::Response<hyper::Body>>>>>>;

pub struct DebugSunfish {
	pub workspace_path: PathBuf,
	pub package_path: PathBuf,
	pub output_path: PathBuf,
	pub routes_handler: RoutesHandler,
}

pub struct ReleaseSunfish {
	pub embedded_dir: EmbeddedDirectory,
	pub routes_handler: RoutesHandler,
	pub routes: Vec<Route>,
}

pub struct Route {
	pub path_with_placeholders: String,
	pub init: fn() -> Page,
}

impl Sunfish {
	pub async fn handle(
		&self,
		request: &mut http::Request<hyper::Body>,
	) -> Result<Option<http::Response<hyper::Body>>> {
		match self {
			Sunfish::Debug(s) => s.handle(request).await,
			Sunfish::Release(s) => s.handle(request).await,
		}
	}

	pub fn export(&self, out_dir: &Path, dist_path: &Path) -> Result<()> {
		match self {
			Sunfish::Debug(_) => unimplemented!(),
			Sunfish::Release(s) => s.export(out_dir, dist_path),
		}
	}
}

impl DebugSunfish {
	pub async fn handle(
		&self,
		request: &mut http::Request<hyper::Body>,
	) -> Result<Option<http::Response<hyper::Body>>> {
		let response = self.serve_page(request).await?;
		let response = match response {
			Some(response) => Some(response),
			None => self.serve_asset(request).await?,
		};
		Ok(response)
	}

	async fn serve_page(
		&self,
		request: &mut http::Request<hyper::Body>,
	) -> Result<Option<http::Response<hyper::Body>>> {
		Ok(self.routes_handler.as_ref()(request).await?)
	}

	async fn serve_asset(
		&self,
		request: &http::Request<hyper::Body>,
	) -> Result<Option<http::Response<hyper::Body>>> {
		let method = request.method().clone();
		let uri = request.uri().clone();
		let path_and_query = uri.path_and_query().unwrap();
		let path = path_and_query.path();
		if method != ::http::Method::GET {
			return Ok(None);
		}
		let path = self.output_path.join(path.strip_prefix('/').unwrap());
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
}

impl ReleaseSunfish {
	pub async fn handle(
		&self,
		request: &mut http::Request<hyper::Body>,
	) -> Result<Option<http::Response<hyper::Body>>> {
		let response = self.serve_page(request).await?;
		let response = match response {
			Some(response) => Some(response),
			None => self.serve_asset(request).await?,
		};
		Ok(response)
	}

	async fn serve_page(
		&self,
		request: &mut http::Request<hyper::Body>,
	) -> Result<Option<http::Response<hyper::Body>>> {
		Ok(self.routes_handler.as_ref()(request).await?)
	}

	async fn serve_asset(
		&self,
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
		let file = if let Some(file) = self.embedded_dir.read(&path) {
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

	pub fn export(&self, out_dir: &Path, dist_path: &Path) -> Result<()> {
		let output_path = out_dir.join("output");
		// Create a new directory at dist_path.
		if std::fs::metadata(&dist_path).is_ok() {
			std::fs::remove_dir_all(&dist_path)?;
		}
		std::fs::create_dir_all(&dist_path)?;
		// Copy the contents of the out_dir to the dist_path.
		for entry in Walk::new(&output_path) {
			let entry = entry.unwrap();
			let input_path = entry.path();
			if !input_path.is_file() {
				continue;
			}
			let path = input_path.strip_prefix(&output_path).unwrap();
			let output_path = dist_path.join(path);
			std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
			std::fs::copy(&input_path, &output_path).unwrap();
		}
		// Render and write the html for each page.
		for route in self.routes.iter() {
			match (route.init)() {
				Page::Static { paths, handler } => {
					let paths = paths
						.map(|paths| paths())
						.unwrap_or_else(|| vec![route.path_with_placeholders.clone()]);
					for path in paths {
						let output_html_path = match path.as_str() {
							"/" => "index.html".to_owned(),
							path if path.ends_with('/') => format!("{}index.html", path),
							path => format!("{}.html", path),
						};
						let output_html_path =
							dist_path.join(&output_html_path.strip_prefix('/').unwrap());
						let html = handler(path);
						std::fs::create_dir_all(output_html_path.parent().unwrap()).unwrap();
						std::fs::write(&output_html_path, html)?;
					}
				}
				Page::Dynamic { .. } => continue,
			}
		}
		Ok(())
	}
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
