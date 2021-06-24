use anyhow::{anyhow, Result};
use indoc::indoc;
use rsa::{PublicKey, RSAPrivateKey, RSAPublicKey};
use serde_json::json;
use sha2::Digest;
use std::convert::TryFrom;
use tangram_id::Id;

pub const TANGRAM_LICENSE_PUBLIC_KEY: &str = indoc!(
	r#"
		-----BEGIN RSA PUBLIC KEY-----
		MIIBCgKCAQEAq+JphywG8wCe6cX+bx4xKH8xphMhaI5BgYefQHUXwp8xavoor6Fy
		B54yZba/pkfTnao+P9BvPT0PlSJ1L9aGzq45lcQCcaT+ZdPC5qUogTrKu4eB2qSj
		yTt5pGnPsna+/7yh2sDhC/SHMvTPKt4oHgobWYkH3/039Rj7z5X2WGq69gJzSknX
		/lraNlVUqCWi3yCnMP9QOV5Tou5gQi4nxlfEJO3razrif5jHw1NufQ+xpx1GCpN9
		WhFBU2R4GFZsxlEXV9g1Os1ZpyVuoOe9BnenuS57TixU9SC8kFUHAyAWRSiuLjoP
		xAmGGm4wQ4FlMAt+Bj/K6rvdG3FJUu5ttQIDAQAB
		-----END RSA PUBLIC KEY-----
	"#
);

pub fn generate(private_key: &str) -> Result<String> {
	let private_key = RSAPrivateKey::try_from(pem::parse(private_key)?)?;
	let id = Id::generate();
	let license_data = json!({ "id": id });
	let license_data = serde_json::to_vec(&license_data)?;
	let mut digest = sha2::Sha256::new();
	digest.update(&license_data);
	let digest = digest.finalize();
	let padding = rsa::PaddingScheme::new_pkcs1v15_sign(None);
	let signature = private_key.sign(padding, &digest)?;
	let license_data = base64::encode(license_data);
	let signature = base64::encode(signature);
	let mut license = String::new();
	license.push_str(&license_data);
	license.push(':');
	license.push_str(&signature);
	Ok(license)
}

pub fn verify(license: &str, public_key: &str) -> Result<bool> {
	let public_key = RSAPublicKey::try_from(pem::parse(public_key)?)?;
	let mut sections = license.split(|c| c == ':');
	let license_data = sections.next().ok_or_else(|| anyhow!("invalid license"))?;
	let license_data = base64::decode(&license_data)?;
	let signature = sections.next().ok_or_else(|| anyhow!("invalid license"))?;
	let signature = base64::decode(&signature)?;
	let mut digest = sha2::Sha256::new();
	digest.update(&license_data);
	let digest = digest.finalize();
	let padding = rsa::PaddingScheme::new_pkcs1v15_sign(None);
	public_key.verify(padding, &digest, &signature)?;
	Ok(true)
}

#[test]
fn test() {
	use rsa::{PrivateKeyPemEncoding, PublicKeyPemEncoding};
	let private_key = rsa::RSAPrivateKey::new(&mut rand::thread_rng(), 2048).unwrap();
	let public_key = rsa::RSAPublicKey::from(&private_key);
	let private_key = private_key.to_pem_pkcs1().unwrap();
	let public_key = public_key.to_pem_pkcs1().unwrap();
	let license = generate(&private_key).unwrap();
	assert!(verify(&license, &public_key).unwrap());
}
