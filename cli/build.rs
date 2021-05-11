use duct::cmd;

fn main() {
	let git_commit = cmd!("git", "rev-parse", "HEAD").read().unwrap();
	println!("cargo:rustc-env=GIT_COMMIT={}", git_commit);
}
