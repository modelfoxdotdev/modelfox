use backtrace::Backtrace;
use futures::FutureExt;
use ignore::Walk;
use include_out_dir::Dir;
use rayon::prelude::*;
use sha2::Digest;
use std::{
	cell::RefCell,
	collections::BTreeMap,
	convert::Infallible,
	future::Future,
	panic::AssertUnwindSafe,
	path::{Path, PathBuf},
	sync::Arc,
};
use tangram_error::{err, Result};
use tangram_id::Id;
use tangram_zip::pzip;
use tracing::{error, info, trace_span, Instrument};
use which::which;

pub async fn serve<C, H, F>(
	host: std::net::IpAddr,
	port: u16,
	request_handler_context: C,
	request_handler: H,
) -> hyper::Result<()>
where
	C: Send + Sync + 'static,
	H: Fn(Arc<C>, http::Request<hyper::Body>) -> F + Send + Sync + 'static,
	F: Future<Output = http::Response<hyper::Body>> + Send,
{
	// Create a task local that will store the panic message and backtrace if a panic occurs.
	tokio::task_local! {
		static PANIC_MESSAGE_AND_BACKTRACE: RefCell<Option<(String, Backtrace)>>;
	}
	async fn service<C, H, F>(
		request_handler: Arc<H>,
		request_handler_context: Arc<C>,
		request: http::Request<hyper::Body>,
	) -> Result<http::Response<hyper::Body>, Infallible>
	where
		C: Send + Sync + 'static,
		H: Fn(Arc<C>, http::Request<hyper::Body>) -> F + Send + Sync + 'static,
		F: Future<Output = http::Response<hyper::Body>> + Send,
	{
		let method = request.method().clone();
		let path_and_query = request.uri().path_and_query().unwrap();
		let path = path_and_query.path();
		let query = path_and_query.query();
		info!(%method, %path, ?query, "request");
		let result = AssertUnwindSafe(request_handler(request_handler_context, request))
			.catch_unwind()
			.await;
		if result.is_err() {
			let message = PANIC_MESSAGE_AND_BACKTRACE.with(|panic_message_and_backtrace| {
				let panic_message_and_backtrace = panic_message_and_backtrace.borrow();
				let (message, _) = panic_message_and_backtrace.as_ref().unwrap();
				message.to_owned()
			});
			error!(%message, "panic!");
		}
		let response = result.unwrap_or_else(|_| {
			http::Response::builder()
				.status(http::StatusCode::INTERNAL_SERVER_ERROR)
				.body(hyper::Body::from("internal server error"))
				.unwrap()
		});
		info!(status = %response.status(), "response");
		Ok(response)
	}
	// Install a panic hook that will record the panic message and backtrace if a panic occurs.
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(|panic_info| {
		let value = (panic_info.to_string(), Backtrace::new());
		PANIC_MESSAGE_AND_BACKTRACE.with(|panic_message_and_backtrace| {
			panic_message_and_backtrace.borrow_mut().replace(value);
		})
	}));
	// Wrap the request handler and context with Arc to allow sharing a reference to it with each task.
	let request_handler = Arc::new(request_handler);
	let request_handler_context = Arc::new(request_handler_context);
	let service = hyper::service::make_service_fn(|_| {
		let request_handler = request_handler.clone();
		let request_handler_context = request_handler_context.clone();
		async move {
			Ok::<_, Infallible>(hyper::service::service_fn(move |request| {
				let request_handler = request_handler.clone();
				let request_handler_context = request_handler_context.clone();
				PANIC_MESSAGE_AND_BACKTRACE.scope(RefCell::new(None), async move {
					let request_id = Id::generate();
					let request_span = trace_span!("request", id = %request_id);
					service(request_handler, request_handler_context, request)
						.instrument(request_span)
						.await
				})
			}))
		}
	});
	let addr = std::net::SocketAddr::new(host, port);
	let server = hyper::server::Server::try_bind(&addr)?;
	eprintln!("🚀 serving on port {}", port);
	server.serve(service).await?;
	std::panic::set_hook(hook);
	Ok(())
}

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
	let hash = &hash(&data);
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

pub async fn serve_from_include_dir(
	dir: &Dir,
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

pub fn hash(string: &[u8]) -> String {
	let mut hash: sha2::Sha256 = Digest::new();
	hash.update(string);
	let hash = hash.finalize();
	let hash = hex::encode(hash);
	let hash = &hash[0..16];
	hash.to_owned()
}

#[macro_export]
macro_rules! serve_from_out_dir {
	($request:expr) => {
		async {
			#[cfg(debug_assertions)]
			{
				let dir = std::path::PathBuf::from(env!("OUT_DIR"));
				$crate::serve_from_dir(&dir, $request).await
			}
			#[cfg(not(debug_assertions))]
			{
				use include_out_dir;
				let dir = include_out_dir::include_out_dir!();
				$crate::serve_from_include_dir(&dir, $request).await
			}
		}
	};
}

#[macro_export]
macro_rules! asset {
	($asset_relative_path:literal) => {{
		let file_path = ::std::path::Path::new(file!());
		let asset_path = file_path.parent().unwrap().join($asset_relative_path);
		let extension = asset_path.extension().map(|e| e.to_str().unwrap()).unwrap();
		let hash = tangram_serve::hash(&asset_path.to_str().unwrap().as_bytes());
		format!("/assets/{}.{}", hash, extension)
	}};
}

#[macro_export]
macro_rules! client {
	() => {{
		let file_path = ::std::path::Path::new(file!());
		let client_crate_manifest_path = file_path
			.parent()
			.unwrap()
			.parent()
			.unwrap()
			.join("client/Cargo.toml");
		let hash = tangram_serve::hash(client_crate_manifest_path.to_str().unwrap().as_bytes());
		format!("/js/{}.js", hash)
	}};
}

pub struct BuildOptions {
	pub workspace_dir: PathBuf,
	pub crate_dir: PathBuf,
	pub crate_out_dir: PathBuf,
	pub cargo_target_wasm_dir: PathBuf,
	pub css_dirs: Vec<PathBuf>,
}

pub fn build(options: BuildOptions) -> Result<()> {
	let client_release =
		option_env!("TANGRAM_SERVE_RELEASE") == Some("1") || cfg!(not(debug_assertions));
	std::fs::create_dir_all(options.crate_out_dir.join("assets")).unwrap();
	std::fs::create_dir_all(options.crate_out_dir.join("js")).unwrap();
	// Build client crates.
	let mut client_crate_manifest_paths = Vec::new();
	for entry in Walk::new(options.crate_dir.join("pages")) {
		let entry = entry.unwrap();
		let path = entry.path();
		if path.ends_with("client/Cargo.toml") {
			let client_crate_manifest_path = path.strip_prefix(&options.workspace_dir).unwrap();
			client_crate_manifest_paths.push(client_crate_manifest_path.to_owned());
		}
	}
	let client_crate_package_names = client_crate_manifest_paths
		.iter()
		.map(|client_crate_manifest_path| {
			let client_crate_manifest =
				std::fs::read_to_string(&options.workspace_dir.join(client_crate_manifest_path))?;
			let client_crate_manifest: toml::Value = toml::from_str(&client_crate_manifest)?;
			let client_crate_name = client_crate_manifest
				.as_table()
				.unwrap()
				.get("package")
				.unwrap()
				.as_table()
				.unwrap()
				.get("name")
				.unwrap()
				.as_str()
				.unwrap()
				.to_owned();
			Ok(client_crate_name)
		})
		.collect::<Result<Vec<_>>>()?;
	if !client_crate_package_names.is_empty() {
		let cmd = which("cargo")?;
		let mut args = vec![
			"build".to_owned(),
			"--target".to_owned(),
			"wasm32-unknown-unknown".to_owned(),
			"--target-dir".to_owned(),
			options.cargo_target_wasm_dir.to_str().unwrap().to_owned(),
		];
		if client_release {
			args.push("--release".to_owned())
		}
		for client_crate_package_name in client_crate_package_names.iter() {
			args.push("--package".to_owned());
			args.push(client_crate_package_name.clone());
		}
		let mut process = std::process::Command::new(cmd).args(&args).spawn()?;
		let status = process.wait()?;
		if !status.success() {
			return Err(err!("cargo {}", status.to_string()));
		}
	}
	pzip!(client_crate_manifest_paths, client_crate_package_names).for_each(
		|(client_crate_manifest_path, client_crate_package_name)| {
			let hash = hash(client_crate_manifest_path.to_str().unwrap().as_bytes());
			let input_path = format!(
				"{}/wasm32-unknown-unknown/{}/{}.wasm",
				options.cargo_target_wasm_dir.to_str().unwrap(),
				if client_release { "release" } else { "debug" },
				client_crate_package_name,
			);
			let output_path = options
				.crate_out_dir
				.join("js")
				.join(format!("{}_bg.wasm", hash));
			// Do not re-run wasm-bindgen if the output wasm exists and is not older than the input wasm.
			let input_metadata = std::fs::metadata(&input_path).unwrap();
			let input_modified_time = input_metadata.modified().unwrap();
			if let Ok(output_wasm_metadata) = std::fs::metadata(&output_path) {
				let output_modified_time = output_wasm_metadata.modified().unwrap();
				if input_modified_time <= output_modified_time {
					return;
				}
			}
			wasm_bindgen_cli_support::Bindgen::new()
				.web(true)
				.unwrap()
				.keep_debug(!client_release)
				.remove_producers_section(true)
				.remove_name_section(true)
				.input_path(input_path)
				.out_name(&hash)
				.generate(&options.crate_out_dir.join("js"))
				.map_err(|error| err!(error))
				.unwrap();
		},
	);
	// Collect CSS.
	let mut css = String::new();
	for dir in options.css_dirs {
		let css_src_dir = options.workspace_dir.join(dir);
		for entry in Walk::new(&css_src_dir) {
			let entry = entry?;
			let path = entry.path();
			if path.extension().map(|e| e.to_str().unwrap()) == Some("css") {
				css.push_str(&std::fs::read_to_string(path)?);
			}
		}
	}
	std::fs::write(options.crate_out_dir.join("styles.css"), css).unwrap();
	// Copy static files.
	let static_dir = options.crate_dir.join("static");
	for entry in Walk::new(&static_dir) {
		let entry = entry.unwrap();
		let input_path = entry.path();
		if !input_path.is_file() {
			continue;
		}
		let output_path = options
			.crate_out_dir
			.join(input_path.strip_prefix(&static_dir).unwrap());
		let input_metadata = std::fs::metadata(&input_path).unwrap();
		let input_modified_time = input_metadata.modified().unwrap();
		if let Ok(output_metadata) = std::fs::metadata(&output_path) {
			let output_modified_time = output_metadata.modified().unwrap();
			if input_modified_time <= output_modified_time {
				continue;
			}
		}
		std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
		std::fs::copy(input_path, output_path).unwrap();
	}
	// Copy assets.
	let asset_extensions = &["gif", "jpg", "png", "svg", "woff2"];
	for entry in Walk::new(&options.crate_dir) {
		let entry = entry.unwrap();
		let input_path = entry.path();
		if !input_path.is_file() {
			continue;
		}
		let extension = input_path.extension().map(|e| e.to_str().unwrap());
		let extension = match extension {
			Some(extension) => extension,
			None => continue,
		};
		if !asset_extensions.contains(&extension) {
			continue;
		}
		let asset_path = input_path.strip_prefix(&options.workspace_dir).unwrap();
		let hash = hash(asset_path.to_str().unwrap().as_bytes());
		let output_path = options
			.crate_out_dir
			.join("assets")
			.join(&format!("{}.{}", hash, extension));
		let input_metadata = std::fs::metadata(&input_path).unwrap();
		let input_modified_time = input_metadata.modified().unwrap();
		if let Ok(output_metadata) = std::fs::metadata(&output_path) {
			let output_modified_time = output_metadata.modified().unwrap();
			if input_modified_time <= output_modified_time {
				continue;
			}
		}
		std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
		std::fs::copy(input_path, output_path).unwrap();
	}
	Ok(())
}

pub type RouteMap = BTreeMap<&'static str, &'static (dyn Fn() -> String + Send + Sync + 'static)>;

pub fn export(out_dir: &Path, dist_path: &Path, route_map: RouteMap) -> Result<()> {
	// Create a new directory at dist_path.
	if std::fs::metadata(dist_path).is_ok() {
		std::fs::remove_dir_all(dist_path)?;
	}
	std::fs::create_dir_all(dist_path)?;
	// Copy the contents of the out_dir to the dist_path.
	for entry in Walk::new(out_dir) {
		let entry = entry.unwrap();
		let input_path = entry.path();
		if !input_path.is_file() {
			continue;
		}
		let path = input_path.strip_prefix(out_dir).unwrap();
		let output_path = dist_path.join(path);
		std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
		std::fs::copy(input_path, output_path).unwrap();
	}
	// Render and write the html for each page.
	for (path, page) in route_map {
		let html = page();
		let path = if path.ends_with('/') {
			let path = path.strip_prefix('/').unwrap();
			format!("{}index.html", path)
		} else {
			let path = path.strip_prefix('/').unwrap();
			format!("{}.html", path)
		};
		let output_path = dist_path.join(path);
		std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
		std::fs::write(output_path, html)?;
	}
	Ok(())
}
