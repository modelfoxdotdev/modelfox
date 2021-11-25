use anyhow::{anyhow, Result};
use indoc::indoc;
use rsa::{
	pkcs1::{FromRsaPrivateKey, FromRsaPublicKey},
	PublicKey, RsaPrivateKey, RsaPublicKey,
};
use serde_json::json;
use sha2::Digest;
use tangram_id::Id;

pub const TANGRAM_LICENSE_PUBLIC_KEY: &str = include_str!("./license.public.rsa");

pub fn generate(private_key: &str) -> Result<String> {
	let private_key = RsaPrivateKey::from_pkcs1_pem(private_key)?;
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
	let public_key = RsaPublicKey::from_pkcs1_pem(public_key)?;
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
	use rsa::pkcs1::{ToRsaPrivateKey, ToRsaPublicKey};
	let private_key = rsa::RsaPrivateKey::new(&mut rand::thread_rng(), 2048).unwrap();
	let public_key = rsa::RsaPublicKey::from(&private_key);
	let private_key = private_key.to_pkcs1_pem().unwrap();
	let public_key = public_key.to_pkcs1_pem().unwrap();
	let license = generate(&private_key).unwrap();
	assert!(verify(&license, &public_key).unwrap());
}
