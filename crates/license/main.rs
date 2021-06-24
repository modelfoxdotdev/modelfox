use anyhow::Result;
use clap::Clap;
use duct::cmd;
use std::path::PathBuf;

#[derive(Clap)]
pub enum Args {
	#[clap(name = "generate")]
	Generate(GenerateArgs),
}

#[derive(Clap)]
pub struct GenerateArgs {
	#[clap(short, long, about = "the path to write the license file")]
	pub output: PathBuf,
}

pub fn main() -> Result<()> {
	let args = Args::parse();
	match args {
		Args::Generate(generate_args) => generate(generate_args)?,
	};
	Ok(())
}

fn generate(generate_args: GenerateArgs) -> Result<()> {
	let private_key = cmd!("pass", "tangram/keys/license.private.rsa")
		.run()?
		.stdout;
	let private_key = String::from_utf8(private_key)?;
	let license = tangram_license::generate(&private_key)?;
	std::fs::write(generate_args.output, license)?;
	Ok(())
}
