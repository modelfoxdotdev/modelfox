use self::data_storage::DataStorage;
use std::net::IpAddr;
use url::Url;

pub mod class_select_field;
pub mod column_type;
pub mod cookies;
pub mod data_storage;
pub mod date_window;
pub mod date_window_select_field;
pub mod error;
pub mod heuristics;
pub mod logo;
pub mod metrics_row;
pub mod model;
pub mod monitor_event;
pub mod organizations;
pub mod page_heading;
pub mod pagination;
pub mod predict;
pub mod production_metrics;
pub mod production_stats;
pub mod repos;
pub mod time;
pub mod timezone;
pub mod tokens;
pub mod topbar;
pub mod user;

pub struct Options {
	pub auth_enabled: bool,
	pub cookie_domain: Option<String>,
	pub data_storage: DataStorage,
	pub database_max_connections: Option<u32>,
	pub database_url: Url,
	pub host: IpAddr,
	pub port: u16,
	pub smtp_options: Option<SmtpOptions>,
	pub stripe_publishable_key: Option<String>,
	pub stripe_secret_key: Option<String>,
	pub url: Option<Url>,
}

#[derive(Clone)]
pub struct SmtpOptions {
	pub host: String,
	pub username: String,
	pub password: String,
}

pub struct Context {
	pub options: Options,
	pub database_pool: sqlx::AnyPool,
}
