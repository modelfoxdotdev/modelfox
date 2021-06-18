use crate::hash::hash;
use ignore::Walk;
use rayon::prelude::*;
use std::path::PathBuf;
use tangram_error::{err, Result};
use which::which;

pub struct BuildOptions {
	pub workspace_path: PathBuf,
	pub crate_path: PathBuf,
	pub crate_out_dir: PathBuf,
	pub css_paths: Vec<PathBuf>,
}

pub fn build(options: BuildOptions) -> Result<()> {
	let target_wasm_dir = options.crate_out_dir.join("target_wasm");
	let output_dir = options.crate_out_dir.join("output");
	let assets_dir = output_dir.join("assets");
	let js_dir = output_dir.join("js");
	let profile = std::env::var("PROFILE").unwrap();
	std::fs::create_dir_all(&output_dir).unwrap();
	std::fs::create_dir_all(&assets_dir).unwrap();
	std::fs::create_dir_all(&js_dir).unwrap();
	// Build client crates.
	let mut client_crate_manifest_paths = Vec::new();
	for entry in Walk::new(options.crate_path.join("pages")) {
		let entry = entry.unwrap();
		let path = entry.path();
		if path.ends_with("client/Cargo.toml") {
			let client_crate_manifest_path = path.strip_prefix(&options.workspace_path).unwrap();
			client_crate_manifest_paths.push(client_crate_manifest_path.to_owned());
		}
	}
	let client_crate_package_names = client_crate_manifest_paths
		.iter()
		.map(|client_crate_manifest_path| {
			let client_crate_manifest =
				std::fs::read_to_string(&options.workspace_path.join(client_crate_manifest_path))?;
			let client_crate_manifest: toml::Value = toml::from_str(&client_crate_manifest)?;
			let client_crate_package_name = client_crate_manifest
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
			Ok(client_crate_package_name)
		})
		.collect::<Result<Vec<_>>>()?;
	let enabled_client_crate_package_names = client_crate_package_names
		.into_iter()
		.filter(|client_crate_package_name| {
			std::env::var(format!(
				"CARGO_FEATURE_{}",
				client_crate_package_name.to_uppercase()
			))
			.is_ok()
		})
		.collect::<Vec<_>>();
	if !enabled_client_crate_package_names.is_empty() {
		let cmd = which("cargo")?;
		let mut args = vec![
			"build".to_owned(),
			"--target-dir".to_owned(),
			target_wasm_dir.display().to_string(),
			"--target".to_owned(),
			"wasm32-unknown-unknown".to_owned(),
		];
		if profile == "release" {
			args.push("--release".to_owned())
		}
		for enabled_client_crate_package_name in enabled_client_crate_package_names.iter() {
			args.push("--package".to_owned());
			args.push(enabled_client_crate_package_name.clone());
		}
		let mut process = std::process::Command::new(cmd).args(&args).spawn()?;
		let status = process.wait()?;
		if !status.success() {
			return Err(err!("cargo {}", status.to_string()));
		}
	}
	enabled_client_crate_package_names
		.par_iter()
		.for_each(|client_crate_package_name| {
			let hash = hash(client_crate_package_name);
			let input_path = format!(
				"{}/wasm32-unknown-unknown/{}/{}.wasm",
				target_wasm_dir.display(),
				profile,
				client_crate_package_name,
			);
			let output_path = js_dir.join(format!("{}_bg.wasm", hash));
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
				.keep_debug(profile == "debug")
				.remove_producers_section(true)
				.remove_name_section(true)
				.input_path(input_path)
				.out_name(&hash)
				.generate(&js_dir)
				.map_err(|error| err!(error))
				.unwrap();
		});
	// Collect CSS.
	let mut css = String::new();
	for dir in options.css_paths {
		for entry in Walk::new(&dir) {
			let entry = entry?;
			let path = entry.path();
			if path.extension().map(|e| e.to_str().unwrap()) == Some("css") {
				css.push_str(&std::fs::read_to_string(path)?);
			}
		}
	}
	std::fs::write(output_dir.join("styles.css"), css).unwrap();
	// Copy static files.
	let static_dir = options.crate_path.join("static");
	for entry in Walk::new(&static_dir) {
		let entry = entry.unwrap();
		let input_path = entry.path();
		if !input_path.is_file() {
			continue;
		}
		let output_path = output_dir.join(input_path.strip_prefix(&static_dir).unwrap());
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
	for entry in Walk::new(&options.crate_path) {
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
		let asset_path = input_path.strip_prefix(&options.workspace_path).unwrap();
		let hash = hash(asset_path.to_str().unwrap().as_bytes());
		let output_path = assets_dir.join(&format!("{}.{}", hash, extension));
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
