use crate::RouteMap;
use ignore::Walk;
use std::path::Path;
use tangram_error::Result;

pub fn export(route_map: RouteMap, out_dir: &Path, dist_path: &Path) -> Result<()> {
	// Create a new directory at dist_path.
	if std::fs::metadata(dist_path).is_ok() {
		std::fs::remove_dir_all(dist_path)?;
	}
	std::fs::create_dir_all(dist_path)?;
	// Copy the contents of the out_dir to the dist_path.
	for entry in Walk::new(out_dir) {
		let entry = entry.unwrap();
		let input_path = entry.path();
		if !input_path.is_file() {
			continue;
		}
		let path = input_path.strip_prefix(out_dir).unwrap();
		let output_path = dist_path.join(path);
		std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
		std::fs::copy(input_path, output_path).unwrap();
	}
	// Render and write the html for each page.
	for (path, page) in route_map {
		let html = page();
		let path = if path.ends_with('/') {
			let path = path.strip_prefix('/').unwrap();
			format!("{}index.html", path)
		} else {
			let path = path.strip_prefix('/').unwrap();
			format!("{}.html", path)
		};
		let output_path = dist_path.join(path);
		std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();
		std::fs::write(output_path, html)?;
	}
	Ok(())
}
