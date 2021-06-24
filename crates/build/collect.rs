use clap::Clap;
use duct::cmd;

#[derive(Clap)]
pub struct Args {}

pub fn run(_args: Args) {
	for machine in ["mba", "win"] {
		cmd!(
			"rsync",
			"--archive",
			"--compress",
			"--delete",
			"--progress",
			format!("{}:tangram/dist/*", machine),
			"dist",
		)
		.run()
		.unwrap();
	}
}
