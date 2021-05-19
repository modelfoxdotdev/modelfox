use crate::hash::hash;
use ignore::Walk;
use rayon::prelude::*;
use std::path::PathBuf;
use tangram_error::{err, Result};
use which::which;

pub struct BuildOptions {
	pub workspace_dir: PathBuf,
	pub crate_dir: PathBuf,
	pub crate_out_dir: PathBuf,
	pub cargo_target_wasm_dir: PathBuf,
	pub css_dirs: Vec<PathBuf>,
}

pub fn build(options: BuildOptions) -> Result<()> {
	let client_release =
		option_env!("TANGRAM_SERVE_RELEASE") == Some("1") || cfg!(not(debug_assertions));
	std::fs::create_dir_all(options.crate_out_dir.join("assets")).unwrap();
	std::fs::create_dir_all(options.crate_out_dir.join("js")).unwrap();
	// Build client crates.
	let mut client_crate_manifest_paths = Vec::new();
	for entry in Walk::new(options.crate_dir.join("pages")) {
		let entry = entry.unwrap();
		let path = entry.path();
		if path.ends_with("client/Cargo.toml") {
			let client_crate_manifest_path = path.strip_prefix(&options.workspace_dir).unwrap();
			client_crate_manifest_paths.push(client_crate_manifest_path.to_owned());
		}
	}
	let client_crate_package_names = client_crate_manifest_paths
		.iter()
		.map(|client_crate_manifest_path| {
			let client_crate_manifest =
				std::fs::read_to_string(&options.workspace_dir.join(client_crate_manifest_path))?;
			let client_crate_manifest: toml::Value = toml::from_str(&client_crate_manifest)?;
			let client_crate_name = client_crate_manifest
				.as_table()
				.unwrap()
				.get("package")
				.unwrap()
				.as_table()
				.unwrap()
				.get("name")
				.unwrap()
				.as_str()
				.unwrap()
				.to_owned();
			Ok(client_crate_name)
		})
		.collect::<Result<Vec<_>>>()?;
	if !client_crate_package_names.is_empty() {
		let cmd = which("cargo")?;
		let mut args = vec![
			"build".to_owned(),
			"--target".to_owned(),
			"wasm32-unknown-unknown".to_owned(),
			"--target-dir".to_owned(),
			options.cargo_target_wasm_dir.to_str().unwrap().to_owned(),
		];
		if client_release {
			args.push("--release".to_owned())
		}
		for client_crate_package_name in client_crate_package_names.iter() {
			args.push("--package".to_owned());
			args.push(client_crate_package_name.clone());
		}
		let mut process = std::process::Command::new(cmd).args(&args).spawn()?;
		let status = process.wait()?;
		if !status.success() {
			return Err(err!("cargo {}", status.to_string()));
		}
	}
	client_crate_package_names
		.par_iter()
		.for_each(|client_crate_package_name| {
			let hash = hash(client_crate_package_name);
			let input_path = format!(
				"{}/wasm32-unknown-unknown/{}/{}.wasm",
				options.cargo_target_wasm_dir.to_str().unwrap(),
				if client_release { "release" } else { "debug" },
				client_crate_package_name,
			);
			let output_path = options
				.crate_out_dir
				.join("js")
				.join(format!("{}_bg.wasm", hash));
			// Do not re-run wasm-bindgen if the output wasm exists and is not older than the input wasm.
			let input_metadata = std::fs::metadata(&input_path).unwrap();
			let input_modified_time = input_metadata.modified().unwrap();
			if let Ok(output_wasm_metadata) = std::fs::metadata(&output_path) {
				let output_modified_time = output_wasm_metadata.modified().unwrap();
				if input_modified_time <= output_modified_time {
					return;
				}
			}
			wasm_bindgen_cli_support::Bindgen::new()
				.web(true)
				.unwrap()
				.keep_debug(!client_release)
				.remove_producers_section(true)
				.remove_name_section(true)
				.input_path(input_path)
				.out_name(&hash)
				.generate(&options.crate_out_dir.join("js"))
				.map_err(|error| err!(error))
				.unwrap();
		});
	// Collect CSS.
	let mut css = String::new();
	for dir in options.css_dirs {
		let css_src_dir = options.workspace_dir.join(dir);
		for entry in Walk::new(&css_src_dir) {
			let entry = entry?;
			let path = entry.path();
			if path.extension().map(|e| e.to_str().unwrap()) == Some("css") {
				css.push_str(&std::fs::read_to_string(path)?);
			}
		}
	}
	std::fs::write(options.crate_out_dir.join("styles.css"), css).unwrap();
	// Copy static files.
	let static_dir = options.crate_dir.join("static");
	for entry in Walk::new(&static_dir) {
		let entry = entry.unwrap();
		let input_path = entry.path();
		if !input_path.is_file() {
			continue;
		}
		let output_path = options
			.crate_out_dir
			.join(input_path.strip_prefix(&static_dir).unwrap());
		let input_metadata = std::fs::metadata(&input_path).unwrap();
		let input_modified_time = input_metadata.modified().unwrap();
		if let Ok(output_metadata) = std::fs::metadata(&output_path) {
			let output_modified_time = output_metadata.modified().unwrap();
			if input_modified_time <= output_modified_time {
				continue;
			}
		}
		std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
		std::fs::copy(input_path, output_path).unwrap();
	}
	// Copy assets.
	let asset_extensions = &["gif", "jpg", "png", "svg", "woff2"];
	for entry in Walk::new(&options.crate_dir) {
		let entry = entry.unwrap();
		let input_path = entry.path();
		if !input_path.is_file() {
			continue;
		}
		let extension = input_path.extension().map(|e| e.to_str().unwrap());
		let extension = match extension {
			Some(extension) => extension,
			None => continue,
		};
		if !asset_extensions.contains(&extension) {
			continue;
		}
		let asset_path = input_path.strip_prefix(&options.workspace_dir).unwrap();
		let hash = hash(asset_path.to_str().unwrap().as_bytes());
		let output_path = options
			.crate_out_dir
			.join("assets")
			.join(&format!("{}.{}", hash, extension));
		let input_metadata = std::fs::metadata(&input_path).unwrap();
		let input_modified_time = input_metadata.modified().unwrap();
		if let Ok(output_metadata) = std::fs::metadata(&output_path) {
			let output_modified_time = output_metadata.modified().unwrap();
			if input_modified_time <= output_modified_time {
				continue;
			}
		}
		std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
		std::fs::copy(input_path, output_path).unwrap();
	}
	Ok(())
}
