use std::{future::Future, pin::Pin};
use tangram_error::Result;

pub mod class_select_field;
pub mod column_type;
pub mod cookies;
pub mod date_window;
pub mod date_window_select_field;
pub mod error;
pub mod heuristics;
pub mod logo;
pub mod metrics_row;
pub mod model;
pub mod monitor_event;
pub mod options;
pub mod organizations;
pub mod page_heading;
pub mod pagination;
pub mod predict;
pub mod production_metrics;
pub mod production_stats;
pub mod repos;
pub mod storage;
pub mod time;
pub mod timezone;
pub mod tokens;
pub mod topbar;
pub mod user;

pub struct Context {
	pub database_pool: sqlx::AnyPool,
	pub options: self::options::Options,
	pub smtp_transport: Option<lettre::AsyncSmtpTransport<lettre::Tokio1Executor>>,
	pub storage: self::storage::Storage,
}

pub type HandleOutput = Pin<Box<dyn Send + Future<Output = Result<http::Response<hyper::Body>>>>>;

pub fn path_components(request: &http::Request<hyper::Body>) -> Vec<&str> {
	request.uri().path().split('/').skip(1).collect::<Vec<_>>()
}
