use clap::Clap;
use futures_batch::ChunksTimeoutStreamExt;
use notify::Watcher;
use std::{convert::Infallible, sync::Arc};
use tokio::sync::{Mutex, Notify};
use tokio_stream::StreamExt;
use which::which;

#[derive(Clap)]
#[clap(
	setting = clap::AppSettings::TrailingVarArg,
)]
pub struct Args {
	#[clap(arg_enum)]
	target: Target,
	args: Vec<String>,
}

#[derive(Clap)]
enum Target {
	#[clap(name = "app")]
	App,
	#[clap(name = "www")]
	Www,
}

enum State {
	Ground,
	Building {
		notify: Arc<Notify>,
		child: Option<std::process::Child>,
	},
	Running {
		child: Option<std::process::Child>,
	},
}

#[tokio::main]
pub async fn main() {
	let args = Args::parse();
	let host = "0.0.0.0";
	let port = 8080;
	let addr = std::net::SocketAddr::new(host.parse().unwrap(), port);
	let child_host = "0.0.0.0";
	let child_port = 8081;
	let child_addr = std::net::SocketAddr::new(child_host.parse().unwrap(), child_port);
	let workspace_dir = std::env::current_dir().unwrap();
	let ignore_paths = vec![
		workspace_dir.join("target"),
		workspace_dir.join("target_check"),
		workspace_dir.join("languages"),
	];
	let watch_paths = vec![workspace_dir];

	let (cmd, cmd_args) = match args.target {
		Target::App => {
			let cmd = which("cargo")
				.unwrap()
				.as_os_str()
				.to_str()
				.unwrap()
				.to_owned();
			let mut cmd_args = vec![
				"run".to_owned(),
				"-p".to_owned(),
				"tangram_cli".to_owned(),
				// "--no-default-features".to_owned(),
				// "--features".to_owned(),
				// "tangram_app,tangram_app/tangram_app_index_server".to_owned(),
				"--".to_owned(),
				"app".to_owned(),
			];
			cmd_args.extend(args.args);
			(cmd, cmd_args)
		}
		Target::Www => {
			let cmd = which("cargo")
				.unwrap()
				.as_os_str()
				.to_str()
				.unwrap()
				.to_owned();
			let mut cmd_args = vec![
				"run".to_owned(),
				"-p".to_owned(),
				"tangram_www".to_owned(),
				// "--no-default-features".to_owned(),
				// "--features".to_owned(),
				// "tangram_www_index_server, tangram_www_index_client".to_owned(),
				"--".to_owned(),
				"serve".to_owned(),
			];
			cmd_args.extend(args.args);
			(cmd, cmd_args)
		}
	};

	let state: Arc<Mutex<State>> = Arc::new(Mutex::new(State::Ground));

	let (watch_events_tx, watch_events_rx) = tokio::sync::mpsc::unbounded_channel();
	watch_events_tx.send(()).unwrap();

	// Run the file watcher.
	let mut watcher: notify::RecommendedWatcher =
		notify::Watcher::new_immediate(move |result: notify::Result<notify::Event>| {
			let event = result.unwrap();
			let ignored = event.paths.iter().all(|path| {
				ignore_paths
					.iter()
					.any(|ignore_path| path.starts_with(ignore_path))
			});
			if !ignored {
				watch_events_tx.send(()).unwrap();
			}
		})
		.unwrap();
	for path in watch_paths.iter() {
		watcher
			.watch(path, notify::RecursiveMode::Recursive)
			.unwrap();
	}

	tokio::spawn({
		let state = state.clone();
		async move {
			let mut watch_events =
				tokio_stream::wrappers::UnboundedReceiverStream::new(watch_events_rx)
					.chunks_timeout(1_000_000, std::time::Duration::from_millis(10));
			while watch_events.next().await.is_some() {
				// Kill the previous child process if any.
				if let State::Running { child } = &mut *state.lock().await {
					let mut child = child.take().unwrap();
					child.kill().ok();
					child.wait().unwrap();
				}
				// Start the new process.
				let notify = Arc::new(Notify::new());
				let child = std::process::Command::new(&cmd)
					.args(&cmd_args)
					.env("HOST", &child_host)
					.env("PORT", &child_port.to_string())
					.spawn()
					.unwrap();
				*state.lock().await = State::Building {
					notify: notify.clone(),
					child: Some(child),
				};
				loop {
					tokio::time::sleep(std::time::Duration::from_millis(100)).await;
					if let State::Building { child, .. } = &mut *state.lock().await {
						if let Ok(Some(_)) | Err(_) = child.as_mut().unwrap().try_wait() {
							break;
						}
					}
					if tokio::net::TcpStream::connect(&child_addr).await.is_ok() {
						break;
					}
				}
				let child = if let State::Building { child, .. } = &mut *state.lock().await {
					child.take().unwrap()
				} else {
					panic!()
				};
				*state.lock().await = State::Running { child: Some(child) };
				notify.notify_waiters();
			}
		}
	});

	// Handle requests by waiting for a build to finish if one is in progress, then proxying the request to the child process.
	let handler = move |state: Arc<Mutex<State>>, mut request: http::Request<hyper::Body>| async move {
		let notify = if let State::Building { notify, .. } = &mut *state.lock().await {
			Some(notify.clone())
		} else {
			None
		};
		if let Some(notify) = notify {
			notify.notified().await;
		}
		let child_authority = format!("{}:{}", child_host, child_port);
		let child_authority = http::uri::Authority::from_maybe_shared(child_authority).unwrap();
		*request.uri_mut() = http::Uri::builder()
			.scheme("http")
			.authority(child_authority)
			.path_and_query(request.uri().path_and_query().unwrap().clone())
			.build()
			.unwrap();
		hyper::Client::new()
			.request(request)
			.await
			.unwrap_or_else(|_| {
				http::Response::builder()
					.status(http::StatusCode::SERVICE_UNAVAILABLE)
					.body(hyper::Body::from("service unavailable"))
					.unwrap()
			})
	};

	// Start the server.
	let service = hyper::service::make_service_fn(|_| {
		let state = state.clone();
		async move {
			Ok::<_, Infallible>(hyper::service::service_fn(
				move |request: http::Request<hyper::Body>| {
					let state = state.clone();
					async move { Ok::<_, Infallible>(handler(state, request).await) }
				},
			))
		}
	});
	hyper::Server::bind(&addr).serve(service).await.unwrap();
}
