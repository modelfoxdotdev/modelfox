use log::info;
use std::{io::prelude::*, path::Path};
use tangram_error::Result;
use tangram_script_build::{target_file_names, Target};

fn main() -> Result<()> {
	let env = env_logger::Env::new().filter_or("LOG", "info");
	env_logger::Builder::from_env(env)
		.format(|buf, record| writeln!(buf, "{}", record.args()))
		.init();

	let tangram_path = std::env::current_dir()?;
	let dist_path = tangram_path.join("dist");

	info!("elixir");
	let elixir_priv_path = tangram_path.join("languages/elixir/priv");
	std::fs::remove_dir_all(&elixir_priv_path)?;
	for target in &[
		Target::X8664UnknownLinuxGnu,
		Target::X8664UnknownLinuxMusl,
		Target::X8664AppleDarwin,
		Target::AArch64AppleDarwin,
		Target::X8664PcWindowsMsvc,
	] {
		let target_file_names = target_file_names(*target);
		install(
			&dist_path
				.join(target.as_str())
				.join(target_file_names.tangram_elixir_file_name),
			&elixir_priv_path
				.join(target.as_str())
				.join(target_file_names.tangram_elixir_file_name),
		)?;
	}

	info!("go");
	let go_libtangram_path = tangram_path.join("languages/go/libtangram");
	std::fs::remove_dir_all(&go_libtangram_path)?;
	for target in &[
		Target::X8664UnknownLinuxMusl,
		Target::X8664AppleDarwin,
		Target::AArch64AppleDarwin,
		Target::X8664PcWindowsGnu,
	] {
		let target_file_names = target_file_names(*target);
		install(
			&dist_path
				.join(target.as_str())
				.join(target_file_names.tangram_h_file_name),
			&go_libtangram_path
				.join(target.as_str())
				.join(target_file_names.tangram_h_file_name),
		)?;
		install(
			&dist_path
				.join(target.as_str())
				.join(target_file_names.libtangram_static_file_name),
			&go_libtangram_path
				.join(target.as_str())
				.join(target_file_names.libtangram_static_file_name),
		)?;
	}

	info!("node");
	let node_dist_path = tangram_path.join("languages/node/native/dist");
	std::fs::remove_dir_all(&node_dist_path)?;
	for target in &[
		Target::X8664UnknownLinuxGnu,
		Target::X8664UnknownLinuxMusl,
		Target::X8664AppleDarwin,
		Target::AArch64AppleDarwin,
		Target::X8664PcWindowsMsvc,
	] {
		let target_file_names = target_file_names(*target);
		install(
			&dist_path
				.join(target.as_str())
				.join(target_file_names.tangram_node_file_name),
			&node_dist_path.join(target.as_str()).join("tangram.node"),
		)?;
	}

	info!("python");
	let python_dist_path = tangram_path.join("languages/python/dist");
	std::fs::remove_dir_all(&python_dist_path)?;
	for target in &[
		Target::X8664UnknownLinuxGnu,
		Target::X8664UnknownLinuxMusl,
		Target::X8664AppleDarwin,
		Target::AArch64AppleDarwin,
		Target::X8664PcWindowsMsvc,
	] {
		let dist_target_path = dist_path.join(target.as_str());
		for entry in std::fs::read_dir(dist_target_path)? {
			let path = entry?.path();
			let is_wheel = path
				.extension()
				.and_then(|e| e.to_str())
				.map(|e| e == "whl")
				.unwrap_or(false);
			if is_wheel {
				install(&path, &python_dist_path.join(path.file_name().unwrap()))?;
			}
		}
	}

	info!("ruby");
	let ruby_libtangram_path = tangram_path.join("languages/ruby/lib/tangram/libtangram");
	std::fs::remove_dir_all(&ruby_libtangram_path)?;
	for target in &[
		Target::X8664UnknownLinuxGnu,
		Target::X8664UnknownLinuxMusl,
		Target::X8664AppleDarwin,
		Target::AArch64AppleDarwin,
		Target::X8664PcWindowsMsvc,
	] {
		let target_file_names = target_file_names(*target);
		install(
			&dist_path
				.join(target.as_str())
				.join(target_file_names.libtangram_dynamic_file_name),
			&ruby_libtangram_path
				.join(target.as_str())
				.join(target_file_names.libtangram_dynamic_file_name),
		)?;
	}

	Ok(())
}

fn install(src: &Path, dst: &Path) -> Result<()> {
	std::fs::create_dir_all(dst.parent().unwrap())?;
	std::fs::copy(src, dst)?;
	Ok(())
}
