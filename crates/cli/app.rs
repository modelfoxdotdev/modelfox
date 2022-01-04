use crate::AppArgs;
use anyhow::{bail, Result};
use std::path::PathBuf;
use url::Url;

#[cfg(feature = "tangram_app")]
use tangram_app_core::{cache_path, data_path, default_database_url};

#[derive(Clone, serde::Deserialize)]
struct AppConfig {
	auth: Option<AuthConfig>,
	cookie_domain: Option<String>,
	database: Option<DatabaseConfig>,
	host: Option<std::net::IpAddr>,
	license: Option<PathBuf>,
	port: Option<u16>,
	smtp: Option<SmtpConfig>,
	storage: Option<StorageConfig>,
	url: Option<String>,
}

#[derive(Clone, serde::Deserialize)]
struct AuthConfig {
	enable: bool,
}

#[derive(Clone, serde::Deserialize)]
struct DatabaseConfig {
	max_connections: Option<u32>,
	url: Url,
}

#[derive(Clone, serde::Deserialize)]
struct SmtpConfig {
	host: String,
	password: String,
	username: String,
}

#[derive(Clone, serde::Deserialize)]
#[serde(tag = "type")]
enum StorageConfig {
	#[serde(rename = "local")]
	Local(LocalStorageConfig),
	#[serde(rename = "s3")]
	S3(S3StorageConfig),
}

#[derive(Clone, serde::Deserialize)]
struct LocalStorageConfig {
	path: PathBuf,
}

#[derive(Clone, serde::Deserialize)]
struct S3StorageConfig {
	pub access_key: String,
	pub secret_key: String,
	pub endpoint: String,
	pub bucket: String,
	pub region: String,
	pub cache_path: Option<PathBuf>,
}

#[cfg(feature = "tangram_app")]
pub fn app(args: AppArgs) -> Result<()> {
	let config: Option<AppConfig> = if let Some(config_path) = args.config {
		let config = std::fs::read(config_path)?;
		Some(serde_json::from_slice(&config)?)
	} else if let Some(config_path) = dirs::config_dir()
		.map(|config_dir_path| config_dir_path.join("tangram").join("app.json"))
		.filter(|config_path| config_path.exists())
	{
		let config = std::fs::read(config_path)?;
		Some(serde_json::from_slice(&config)?)
	} else if let Some(config_path) = dirs::config_dir()
		.map(|config_dir_path| config_dir_path.join("tangram").join("app.yaml"))
		.filter(|config_path| config_path.exists())
	{
		let config = std::fs::read(config_path)?;
		Some(serde_yaml::from_slice(&config)?)
	} else {
		None
	};
	let auth = config
		.as_ref()
		.and_then(|c| c.auth.as_ref())
		.and_then(|auth| {
			if auth.enable {
				Some(tangram_app::options::AuthOptions {})
			} else {
				None
			}
		});
	let cookie_domain = config.as_ref().and_then(|c| c.cookie_domain.clone());
	let storage = if let Some(storage) = config.as_ref().and_then(|c| c.storage.as_ref()) {
		match storage {
			StorageConfig::Local(storage) => tangram_app::options::StorageOptions::Local(
				tangram_app::options::LocalStorageOptions {
					path: storage.path.clone(),
				},
			),
			StorageConfig::S3(storage) => {
				let cache_path = storage
					.cache_path
					.clone()
					.unwrap_or_else(|| cache_path().unwrap());
				tangram_app::options::StorageOptions::S3(tangram_app::options::S3StorageOptions {
					access_key: storage.access_key.clone(),
					secret_key: storage.secret_key.clone(),
					endpoint: storage.endpoint.clone(),
					bucket: storage.bucket.clone(),
					region: storage.region.clone(),
					cache_path,
				})
			}
		}
	} else {
		tangram_app::options::StorageOptions::Local(tangram_app::options::LocalStorageOptions {
			path: data_path()?.join("data"),
		})
	};
	let database = config
		.as_ref()
		.and_then(|c| c.database.as_ref())
		.map(|database| tangram_app::options::DatabaseOptions {
			max_connections: database.max_connections,
			url: database.url.clone(),
		})
		.unwrap_or_else(|| tangram_app::options::DatabaseOptions {
			max_connections: None,
			url: default_database_url(),
		});
	let host_from_env = if let Ok(host) = std::env::var("HOST") {
		Some(host.parse()?)
	} else {
		None
	};
	let host_from_config = config.as_ref().and_then(|c| c.host);
	let host = host_from_env
		.or(host_from_config)
		.unwrap_or_else(|| "0.0.0.0".parse().unwrap());
	let port_from_env = if let Ok(port) = std::env::var("PORT") {
		Some(port.parse()?)
	} else {
		None
	};
	let port_from_config = config.as_ref().and_then(|c| c.port);
	let port = port_from_env.or(port_from_config).unwrap_or(8080);
	// Verify the license if one was provided.
	let license_verified: Option<bool> =
		if let Some(license_file_path) = config.as_ref().and_then(|c| c.license.clone()) {
			let license = std::fs::read_to_string(license_file_path)?;
			let public_key = tangram_license::TANGRAM_LICENSE_PUBLIC_KEY;
			Some(tangram_license::verify(&license, public_key)?)
		} else {
			None
		};
	// Require a verified license if auth is enabled.
	if auth.is_some() {
		match license_verified {
			#[cfg(debug_assertions)]
			None => {}
			#[cfg(not(debug_assertions))]
			None => bail!("a license is required to enable authentication"),
			Some(false) => bail!("failed to verify license"),
			Some(true) => {}
		}
	}
	let smtp = if let Some(smtp) = config.as_ref().and_then(|c| c.smtp.clone()) {
		Some(tangram_app::options::SmtpOptions {
			host: smtp.host,
			username: smtp.username,
			password: smtp.password,
		})
	} else {
		None
	};
	let url = if let Some(url) = config.as_ref().and_then(|c| c.url.clone()) {
		Some(url.parse()?)
	} else {
		None
	};
	let options = tangram_app::options::Options {
		auth,
		cookie_domain,
		database,
		host,
		port,
		smtp,
		storage,
		url,
	};
	tangram_app::run(options)
}
