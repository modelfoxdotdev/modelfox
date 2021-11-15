use duct::cmd;
use tangram_build::{Target, TargetFileNames};
use which::which;

fn main() {
	let root_path = std::env::current_dir().unwrap();
	let target_path = root_path.join("target");
	let dist_path = root_path.join("dist");

	// Create the dist directory.
	if std::fs::metadata(&dist_path)
		.map(|m| m.is_dir())
		.unwrap_or(false)
	{
		std::fs::remove_dir_all(&dist_path).unwrap();
	}
	std::fs::create_dir_all(&dist_path).unwrap();

	// Compile!
	cmd!(
		which("cargo").unwrap(),
		"build",
		"-Zbuild-std",
		"-Zbuild-std-features=system-llvm-libunwind",
		"-Zmultitarget",
		"--release",
		"--target",
		Target::AArch64LinuxGnu228.rust_target_name(),
		// "--target",
		// Target::AArch64LinuxMusl.rust_target_name(),
		//"--target",
		//Target::AArch64MacOS.rust_target_name(),
		"--target",
		Target::X8664LinuxGnu228.rust_target_name(),
		// "--target",
		// Target::X8664LinuxMusl.rust_target_name(),
		//"--target",
		//Target::X8664MacOS.rust_target_name(),
		"--target",
		Target::X8664WindowsGnu.rust_target_name(),
		"--target",
		Target::X8664WindowsMsvc.rust_target_name(),
		"--package",
		"tangram_cli",
		"--package",
		"libtangram",
		"--package",
		"tangram_elixir",
		"--package",
		"tangram_node",
	)
	.run()
	.unwrap();

	// Copy the artifacts.
	for target in [
		Target::AArch64LinuxGnu228,
		// Target::AArch64LinuxMusl,
		// Target::AArch64MacOS,
		Target::X8664LinuxGnu228,
		// Target::X8664LinuxMusl,
		// Target::X8664MacOS,
		Target::X8664WindowsGnu,
		Target::X8664WindowsMsvc,
	] {
		let target_file_names = TargetFileNames::for_target(target);
		let cargo_artifact_path = target_path.join(target.rust_target_name()).join("release");
		let dist_target_path = dist_path.join(target.target_name());
		std::fs::create_dir(&dist_target_path).unwrap();
		let tangram_cli_src = cargo_artifact_path.join(target_file_names.tangram_cli_file_name);
		let tangram_cli_dst = dist_target_path.join(target_file_names.tangram_cli_file_name);
		std::fs::copy(tangram_cli_src, tangram_cli_dst).unwrap();
		let tangram_h_dst = dist_target_path.join(target_file_names.tangram_h_file_name);
		cbindgen::generate(root_path.join("languages/c"))
			.unwrap()
			.write(std::fs::File::create(tangram_h_dst).unwrap());
		let libtangram_dynamic_src =
			cargo_artifact_path.join(target_file_names.libtangram_dynamic_file_name);
		let libtangram_dynamic_dst =
			dist_target_path.join(target_file_names.libtangram_dynamic_file_name);
		std::fs::copy(libtangram_dynamic_src, libtangram_dynamic_dst).unwrap();
		let libtangram_static_src =
			cargo_artifact_path.join(target_file_names.libtangram_static_file_name);
		let libtangram_static_dst =
			dist_target_path.join(target_file_names.libtangram_static_file_name);
		std::fs::copy(libtangram_static_src, libtangram_static_dst).unwrap();
	}
}
