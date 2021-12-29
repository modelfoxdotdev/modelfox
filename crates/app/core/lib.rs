pub mod alerts;
pub mod app;
pub mod cookies;
pub mod error;
pub mod heuristics;
pub mod model;
pub mod monitor_event;
pub mod options;
pub mod organizations;
pub mod predict;
pub mod production_metrics;
pub mod production_stats;
pub mod repos;
pub mod storage;
pub mod timezone;
pub mod user;

pub use app::App;

pub fn path_components(request: &http::Request<hyper::Body>) -> Vec<&str> {
	request.uri().path().split('/').skip(1).collect::<Vec<_>>()
}
