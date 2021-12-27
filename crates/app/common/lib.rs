use anyhow::Result;
use std::{future::Future, pin::Pin};

pub mod alerts;
pub mod cookies;
pub mod error;
pub mod heuristics;
pub mod model;
pub mod monitor_event;
pub mod options;
pub mod organizations;
pub mod predict;
pub mod repos;
pub mod storage;
pub mod timezone;
pub mod user;

pub struct Context {
	pub database_pool: sqlx::AnyPool,
	pub options: self::options::Options,
	pub smtp_transport: Option<lettre::AsyncSmtpTransport<lettre::Tokio1Executor>>,
	pub storage: self::storage::Storage,
	pub sunfish: sunfish::Sunfish,
}

pub type HandleOutput<'a> =
	Pin<Box<dyn 'a + Send + Future<Output = Result<http::Response<hyper::Body>>>>>;

pub fn path_components(request: &http::Request<hyper::Body>) -> Vec<&str> {
	request.uri().path().split('/').skip(1).collect::<Vec<_>>()
}
