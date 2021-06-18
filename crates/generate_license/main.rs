use clap::Clap;
use duct::cmd;
use std::path::PathBuf;
use tangram_error::Result;

#[derive(Clap)]
pub struct Args {
	#[clap(short, long, about = "the path to write the license file")]
	pub output: PathBuf,
}

pub fn main() -> Result<()> {
	let args = Args::parse();
	let private_key = cmd!("pass", "tangram/keys/license.private.rsa")
		.run()?
		.stdout;
	let private_key = String::from_utf8(private_key)?;
	let license = tangram_license::generate(&private_key)?;
	std::fs::write(args.output, license)?;
	Ok(())
}
