use clap::Clap;
use duct::cmd;
use serde_json::json;
use sha2::Digest;
use std::path::PathBuf;
use tangram_error::Result;
use tangram_id::Id;

#[derive(Clap)]
pub struct Args {
	#[clap(short, long, about = "the path to write the license file")]
	pub output: PathBuf,
}

pub fn main() -> Result<()> {
	let args = Args::parse();
	let tangram_license_private_key = cmd!("pass", "tangram/keys/license.private.rsa")
		.run()?
		.stdout;
	let tangram_license_private_key = String::from_utf8(tangram_license_private_key)?
		.lines()
		.skip(1)
		.filter(|line| !line.starts_with('-'))
		.fold(String::new(), |mut string, line| {
			string.push_str(&line);
			string
		});
	let tangram_license_private_key = base64::decode(tangram_license_private_key)?;
	let tangram_license_private_key = rsa::RSAPrivateKey::from_pkcs1(&tangram_license_private_key)?;
	let id = Id::generate();
	let license_data = json!({ "id": id });
	let license_data = serde_json::to_vec(&license_data)?;
	let mut digest = sha2::Sha256::new();
	digest.update(&license_data);
	let digest = digest.finalize();
	let signature =
		tangram_license_private_key.sign(rsa::PaddingScheme::new_pkcs1v15_sign(None), &digest)?;
	let license_data = base64::encode(license_data);
	let signature = base64::encode(signature);
	let mut license = String::new();
	license.push_str(&license_data);
	license.push(':');
	license.push_str(&signature);
	std::fs::write(args.output, license)?;
	Ok(())
}
