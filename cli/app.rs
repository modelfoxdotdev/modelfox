use crate::AppArgs;
use rsa::PublicKey;
use sha2::Digest;
use std::path::Path;
use std::path::PathBuf;
use tangram_error::{err, Result};
use url::Url;

#[derive(serde::Deserialize)]
struct AppConfig {
	auth_enabled: Option<bool>,
	cookie_domain: Option<String>,
	data_dir: Option<PathBuf>,
	database_max_connections: Option<u32>,
	database_url: Option<String>,
	host: Option<std::net::IpAddr>,
	license: Option<PathBuf>,
	port: Option<u16>,
	s3_url: Option<String>,
	smtp_host: Option<String>,
	smtp_password: Option<String>,
	smtp_username: Option<String>,
	url: Option<String>,
}

#[cfg(feature = "app")]
pub fn app(args: AppArgs) -> Result<()> {
	let config: Option<AppConfig> = if let Some(config_path) = args.config {
		let config = std::fs::read(config_path)?;
		Some(serde_json::from_slice(&config)?)
	} else {
		None
	};
	let auth_enabled = config
		.as_ref()
		.and_then(|c| c.auth_enabled)
		.unwrap_or(false);
	let cookie_domain = config.as_ref().and_then(|c| c.cookie_domain.clone());
	let data_storage = if let Some(data_s3_url) = config.as_ref().and_then(|c| c.s3_url.clone()) {
		let cache_dir = cache_dir()?;
		tangram_app::DataStorage::S3(data_s3_url.parse()?, cache_dir)
	} else if let Some(data_dir) = config.as_ref().and_then(|c| c.data_dir.clone()) {
		tangram_app::DataStorage::Local(data_dir)
	} else {
		tangram_app::DataStorage::Local(data_dir()?.join("data"))
	};
	let database_max_connections = config.as_ref().and_then(|c| c.database_max_connections);
	let database_url = match config.as_ref().and_then(|c| c.database_url.clone()) {
		Some(database_url) => database_url.parse()?,
		None => default_database_url()?,
	};
	let host = config
		.as_ref()
		.and_then(|c| c.host)
		.unwrap_or_else(|| "0.0.0.0".parse().unwrap());
	let port = config.as_ref().and_then(|c| c.port).unwrap_or(8080);
	// Verify the license if one was provided.
	let license_verified: Option<bool> =
		if let Some(license_file_path) = config.as_ref().and_then(|c| c.license.clone()) {
			Some(verify_license(&license_file_path)?)
		} else {
			None
		};
	// Require a verified license if auth is enabled.
	if auth_enabled {
		match license_verified {
			#[cfg(debug_assertions)]
			None => {}
			#[cfg(not(debug_assertions))]
			None => return Err(err!("a license is required to enable authentication")),
			Some(false) => return Err(err!("failed to verify license")),
			Some(true) => {}
		}
	}
	let smtp_options = if let Some(smtp_host) = config.as_ref().and_then(|c| c.smtp_host.clone()) {
		let smtp_username = config
			.as_ref()
			.ok_or_else(|| err!("smtp username is required if smtp host is provided"))?
			.smtp_username
			.clone()
			.ok_or_else(|| err!("smtp username is required if smtp host is provided"))?;
		let smtp_password = config
			.as_ref()
			.ok_or_else(|| err!("smtp password is required if smtp host is provided"))?
			.smtp_password
			.clone()
			.ok_or_else(|| err!("smtp password is required if smtp host is provided"))?;
		Some(tangram_app::SmtpOptions {
			host: smtp_host,
			username: smtp_username,
			password: smtp_password,
		})
	} else {
		None
	};
	let url = if let Some(url) = config.as_ref().and_then(|c| c.url.clone()) {
		Some(url.parse()?)
	} else {
		None
	};
	let options = tangram_app::Options {
		auth_enabled,
		cookie_domain,
		data_storage,
		database_max_connections,
		database_url,
		host,
		port,
		smtp_options,
		url,
	};
	tangram_app::run(options)
}

/// Retrieve the user cache directory using the `dirs` crate.
#[cfg(feature = "app")]
fn cache_dir() -> Result<PathBuf> {
	let cache_dir = dirs::cache_dir().ok_or_else(|| err!("failed to find user cache directory"))?;
	let tangram_cache_dir = cache_dir.join("tangram");
	std::fs::create_dir_all(&tangram_cache_dir).map_err(|_| {
		err!(
			"failed to create tangram cache directory in {}",
			tangram_cache_dir.display()
		)
	})?;
	Ok(tangram_cache_dir)
}

/// Retrieve the user data directory using the `dirs` crate.
#[cfg(feature = "app")]
fn data_dir() -> Result<PathBuf> {
	let data_dir = dirs::data_dir().ok_or_else(|| err!("failed to find user data directory"))?;
	let tangram_data_dir = data_dir.join("tangram");
	std::fs::create_dir_all(&tangram_data_dir).map_err(|_| {
		err!(
			"failed to create tangram data directory in {}",
			tangram_data_dir.display()
		)
	})?;
	Ok(tangram_data_dir)
}

/// Retrieve the default database url, which is a sqlite database in the user data directory.
#[cfg(feature = "app")]
pub fn default_database_url() -> Result<Url> {
	let tangram_database_path = data_dir()?.join("db").join("tangram.db");
	std::fs::create_dir_all(tangram_database_path.parent().unwrap())?;
	let url = format!(
		"sqlite:{}",
		tangram_database_path.to_str().unwrap().to_owned()
	);
	let url = Url::parse(&url)?;
	Ok(url)
}

pub fn verify_license(license_file_path: &Path) -> Result<bool> {
	let tangram_license_public_key: &str = "
-----BEGIN RSA PUBLIC KEY-----
MIIBCgKCAQEAq+JphywG8wCe6cX+bx4xKH8xphMhaI5BgYefQHUXwp8xavoor6Fy
B54yZba/pkfTnao+P9BvPT0PlSJ1L9aGzq45lcQCcaT+ZdPC5qUogTrKu4eB2qSj
yTt5pGnPsna+/7yh2sDhC/SHMvTPKt4oHgobWYkH3/039Rj7z5X2WGq69gJzSknX
/lraNlVUqCWi3yCnMP9QOV5Tou5gQi4nxlfEJO3razrif5jHw1NufQ+xpx1GCpN9
WhFBU2R4GFZsxlEXV9g1Os1ZpyVuoOe9BnenuS57TixU9SC8kFUHAyAWRSiuLjoP
xAmGGm4wQ4FlMAt+Bj/K6rvdG3FJUu5ttQIDAQAB
-----END RSA PUBLIC KEY-----
";
	let tangram_license_public_key = tangram_license_public_key
		.lines()
		.skip(1)
		.filter(|line| !line.starts_with('-'))
		.fold(String::new(), |mut data, line| {
			data.push_str(&line);
			data
		});
	let tangram_license_public_key = base64::decode(tangram_license_public_key).unwrap();
	let tangram_license_public_key =
		rsa::RSAPublicKey::from_pkcs1(&tangram_license_public_key).unwrap();
	let license_data = std::fs::read(license_file_path)?;
	let mut sections = license_data.split(|byte| *byte == b':');
	let license_data = sections.next().ok_or_else(|| err!("invalid license"))?;
	let license_data = base64::decode(&license_data)?;
	let signature = sections.next().ok_or_else(|| err!("invalid license"))?;
	let signature = base64::decode(&signature)?;
	let mut digest = sha2::Sha256::new();
	digest.update(&license_data);
	let digest = digest.finalize();
	tangram_license_public_key.verify(
		rsa::PaddingScheme::new_pkcs1v15_sign(None),
		&digest,
		&signature,
	)?;
	Ok(true)
}
