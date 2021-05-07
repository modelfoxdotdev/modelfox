use std::net::IpAddr;
use std::path::PathBuf;
use url::Url;

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

pub struct AuthOptions {}

pub struct DatabaseOptions {
	pub max_connections: Option<u32>,
	pub url: Url,
}

#[derive(Clone)]
pub struct SmtpOptions {
	pub host: String,
	pub username: String,
	pub password: String,
}

#[derive(Clone)]
pub enum StorageOptions {
	Local(LocalStorageOptions),
	S3(S3StorageOptions),
}

#[derive(Clone)]
pub struct LocalStorageOptions {
	pub path: PathBuf,
}

#[derive(Clone)]
pub struct S3StorageOptions {
	pub cache_path: PathBuf,
}

impl Options {
	pub fn auth_enabled(&self) -> bool {
		self.auth.is_some()
	}
}
