use crate::app::{data_path, default_database_url};
use std::net::IpAddr;
use std::path::PathBuf;
use url::Url;

#[derive(Debug)]
pub struct Options {
	pub auth: Option<AuthOptions>,
	pub cookie_domain: Option<String>,
	pub database: DatabaseOptions,
	pub host: IpAddr,
	pub port: u16,
	pub smtp: Option<SmtpOptions>,
	pub storage: StorageOptions,
	pub url: Option<Url>,
}

#[derive(Debug)]
pub struct AuthOptions {}

#[derive(Debug)]
pub struct DatabaseOptions {
	pub max_connections: Option<u32>,
	pub url: Url,
}

#[derive(Debug, Clone)]
pub struct SmtpOptions {
	pub host: String,
	pub username: String,
	pub password: String,
}

#[derive(Debug, Clone)]
pub enum StorageOptions {
	Local(LocalStorageOptions),
	S3(S3StorageOptions),
}

#[derive(Debug, Clone)]
pub struct LocalStorageOptions {
	pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct S3StorageOptions {
	pub access_key: String,
	pub secret_key: String,
	pub endpoint: String,
	pub bucket: String,
	pub region: String,
	pub cache_path: PathBuf,
}

impl Options {
	pub fn auth_enabled(&self) -> bool {
		self.auth.is_some()
	}
}

impl Default for Options {
	fn default() -> Self {
		let host: IpAddr = if let Ok(host) = std::env::var("HOST") {
			host.parse()
				.expect("Could not parse HOST environment variable")
		} else {
			"0.0.0.0".parse().unwrap()
		};
		let port = if let Ok(port) = std::env::var("PORT") {
			port.parse()
				.expect("Could not parse PORT environment variable")
		} else {
			8080u16
		};
		let database = DatabaseOptions {
			max_connections: None,
			url: default_database_url(),
		};
		let storage = StorageOptions::Local(LocalStorageOptions {
			path: data_path()
				.expect("Could not read or create tangram data directory")
				.join("data"),
		});
		Options {
			auth: None,
			cookie_domain: None,
			database,
			host,
			port,
			smtp: None,
			storage,
			url: None,
		}
	}
}
