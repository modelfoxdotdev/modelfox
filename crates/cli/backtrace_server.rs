//! This module encapsulates the backtrace capturing functionality

use futures::{Future, future::FutureExt};
use hyper::http;
use std::{cell::RefCell, convert::Infallible, panic::AssertUnwindSafe, sync::Arc};

fn internal_server_error(msg: &str) -> http::Response<hyper::Body> {
	http::Response::builder()
		.status(http::StatusCode::INTERNAL_SERVER_ERROR)
		.body(hyper::Body::from(format!("internal server error: {}", msg)))
		.unwrap()
}

pub async fn serve<F, T>(
	addr: std::net::SocketAddr,
	contexts: &[std::sync::Arc<T>],
	handle: F,
) -> anyhow::Result<()>
where
	F: Future<Output=dyn FnMut(http::Request<hyper::Body>) -> http::Response<hyper::Body>>,
	T: Send + Sync,
{
	// Storage for any potential panic in this Tokio task
	tokio::task_local! {
		static PANIC_MESSAGE_AND_BACKTRACE: RefCell<Option<(String, backtrace::Backtrace)>>;
	}
	// Define service
	let make_svc =
		hyper::service::make_service_fn(move |_socket: &hyper::server::conn::AddrStream| {
			// handle connection
			let context = Arc::clone(&contexts[0]);
			async {
				Ok::<_, Infallible>(hyper::service::service_fn(
					move |mut request: http::Request<hyper::Body>| {
						// handle request
						let context = Arc::clone(&contexts[0]);
						PANIC_MESSAGE_AND_BACKTRACE.scope(RefCell::new(None), async move {
							request.extensions_mut().insert(context);
							tracing::debug!(
								"Processing request: {} {}",
								request.method(),
								request.uri()
							);
							let start = std::time::SystemTime::now();
							let response =
								match AssertUnwindSafe(handle(request)).catch_unwind().await {
									Ok(response) => response,
									Err(_) => {
										let message = PANIC_MESSAGE_AND_BACKTRACE.with(
											|panic_message_and_backtrace| {
												let panic_message_and_backtrace =
													panic_message_and_backtrace.borrow();
												let (message, backtrace) =
													panic_message_and_backtrace.as_ref().unwrap();
												format!(
													"panic: {}, backtrace: {:?}",
													message, backtrace
												)
											},
										);
										tracing::error!(%message, "panic!");
										internal_server_error(&message)
									}
								};

							tracing::debug!(
								"Produced response in {}μs",
								start.elapsed().unwrap().as_micros()
							);
							Ok::<_, Infallible>(response)
						})
					},
				))
			}
		});
	// Record the panic message and backtrace if a panic occurs.
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(|panic_info| {
		let value = (panic_info.to_string(), backtrace::Backtrace::new());
		PANIC_MESSAGE_AND_BACKTRACE.with(|panic_message_and_backtrace| {
			panic_message_and_backtrace.borrow_mut().replace(value);
		})
	}));

	hyper::Server::bind(&addr).serve(make_svc).await?;
	std::panic::set_hook(hook);
	Ok(())
}
