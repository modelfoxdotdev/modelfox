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
