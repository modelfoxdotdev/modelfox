/*!
# wheel_writer

This library produces Python wheels according to
<https://www.python.org/dev/peps/pep-0491/>.

Create a [`WheelBuilder`], providing the dynamic library and the package metadata file.
Optionally set the platform tag, typing interface file, and output directory.
When ready, use [`WheelBuilder::build`] to produce the `.whl` file.

You will need to provide a metadata TOML file:

```toml
author = "Bill Shakespeare"
author_email = "william@globetheatre.org"
classifiers = [
  "License :: OSI Approved :: MIT License",
  "Operating System :: OS Independent",
  "Programming Language :: Python",
]
# The README should point to a file, this file will be read and included in the wheel.
description = "README.md"
description_content_type = "text/markdown"
# The LICENSE should point to a file, this file will be read and included in the wheel.
license = "LICENSE"
name = "much-ado-about-python"
summary = "Two Dicts fall in love, while two Lists fall out of love, then back in love later. They all lie to each other and then everyone gets hitched."
version = "0.1.0"

[project_url]
homepage = "https://en.wikipedia.org/wiki/Much_Ado_About_Nothing"
```
*/

use anyhow::{Context, Result};
use digest::Digest;
use indoc::formatdoc;
use regex::Regex;
use serde::Deserialize;
use sha2::Sha256;
use std::{
	collections::HashMap,
	fmt,
	fs::{self, File},
	io::Write,
	path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// Use this struct to define the inputs for your wheel.
#[derive(Debug)]
pub struct WheelWriter {
	/// The optional ABI tag to target
	pub abi: Option<String>,
	/// The optional build tag to target
	pub build_tag: Option<String>,
	/// The name of the overarching distribution
	pub distribution: String,
	/// metadata.toml file
	pub metadata_toml_path: PathBuf,
	/// The packages to include in the wheel.
	pub packages: Vec<PathBuf>,
	/// The platform to target
	pub platform: Option<String>,
	/// The python tag to target
	pub python_tag: String,
	/// The version of the distribution
	pub version: String,
}

impl WheelWriter {
	/// Produce the `.whl` file.
	pub fn write(&mut self, output_path: impl AsRef<Path>) -> Result<()> {
		let output_path = output_path.as_ref();
		ensure_dir_exists(&output_path)?;
		let tag = Tag::new(
			self.abi.clone(),
			self.build_tag.clone(),
			self.distribution.clone(),
			self.platform.clone(),
			self.python_tag.clone(),
			self.version.clone(),
		);
		let metadata = Metadata21::from_path(&self.metadata_toml_path)?;
		let dist_info = WheelDistInfo::new(metadata, tag);
		let output_name = dist_info.wheel_name();
		let output_file_path = output_path.join(&output_name);
		let file = clobbering_create(&output_file_path)?;
		let mut record: Vec<RecordEntry> = Vec::new();
		let mut zip_writer = zip::ZipWriter::new(file);
		let zip_options = zip::write::FileOptions::default()
			.unix_permissions(0o644)
			.compression_method(zip::CompressionMethod::Deflated);
		let mut write_file = |path: &Path, contents: &[u8]| -> Result<()> {
			zip_writer.start_file(path.to_str().unwrap(), zip_options)?;
			zip_writer
				.write_all(contents)
				.context(format!("Could not write {}", path.display()))?;
			let hash = base64::encode_config(&Sha256::digest(contents), base64::URL_SAFE_NO_PAD);
			record.push((path.to_str().unwrap().to_owned(), hash, contents.len()).into());
			Ok(())
		};
		let mut write_dir = |path: &Path| -> Result<()> {
			for entry in WalkDir::new(path) {
				let entry = entry?;
				if entry.file_type().is_file() {
					let path = entry.path();
					let contents = std::fs::read(path)?;
					write_file(path, &contents)?;
				}
			}
			Ok(())
		};
		// Packages
		for package in &self.packages {
			write_dir(package)?;
		}
		let dist_info_path =
			PathBuf::from(format!("{}-{}.dist-info", self.distribution, self.version));
		// LICENSE
		write_file(
			&dist_info_path.join("LICENSE"),
			dist_info.license().as_bytes(),
		)?;
		// METADATA
		write_file(
			&dist_info_path.join("METADATA"),
			dist_info.metadata.to_file_contents().as_bytes(),
		)?;
		// top_level.txt
		let mut module_name = dist_info.tag.distribution.as_bytes().to_owned();
		module_name.push(b'\n');
		write_file(&dist_info_path.join("top_level.txt"), &module_name)?;
		// WHEEL
		write_file(
			&dist_info_path.join("WHEEL"),
			dist_info.wheel_file().as_bytes(),
		)?;
		// Record
		let record_path = dist_info_path.join("RECORD");
		zip_writer.start_file(record_path.to_str().unwrap(), zip_options)?;
		for entry in record {
			zip_writer.write_all(
				format!(
					"{},sha256-{},{}\n",
					entry.relative_path.display(),
					entry.hash,
					entry.size
				)
				.as_bytes(),
			)?;
		}
		// Record the RECORD file itself
		zip_writer.write_all(format!("{},,\n", record_path.display()).as_bytes())?;
		zip_writer.finish()?;
		Ok(())
	}
}

/// Metadata version 2.1
///
/// <https://www.python.org/dev/peps/pep-0566/>
#[derive(Debug, Deserialize)]
struct Metadata21 {
	// Mandatory fields
	#[serde(default = "Metadata21::metadata_version")]
	metadata_version: String,
	name: String,
	version: String,
	// Optional fields
	#[serde(default)]
	platform: Vec<String>,
	#[serde(default)]
	supported_platform: Vec<String>,
	summary: Option<String>,
	description: Option<String>,
	description_content_type: Option<String>,
	keywords: Option<String>,
	home_page: Option<String>,
	download_url: Option<String>,
	author: Option<String>,
	author_email: Option<String>,
	maintainer: Option<String>,
	maintainer_email: Option<String>,
	license: Option<String>,
	#[serde(default)]
	classifiers: Vec<String>,
	#[serde(default)]
	requires_dist: Vec<String>,
	#[serde(default)]
	provides_dist: Vec<String>,
	#[serde(default)]
	obsoletes_dist: Vec<String>,
	requires_python: Option<String>,
	#[serde(default)]
	requires_external: Vec<String>,
	#[serde(default)]
	project_url: HashMap<String, String>,
	#[serde(default)]
	provides_extra: Vec<String>,
	#[serde(default)]
	#[allow(dead_code)]
	scripts: HashMap<String, String>,
	#[serde(default)]
	#[allow(dead_code)]
	gui_scripts: HashMap<String, String>,
	#[serde(default)]
	#[allow(dead_code)]
	entry_points: HashMap<String, HashMap<String, String>>,
}

impl Metadata21 {
	/// Load metadata from TOML
	pub fn from_path(toml_path: impl AsRef<Path>) -> Result<Self> {
		let contents = fs::read_to_string(&toml_path).context(format!(
			"Can't read TOML file at {}",
			toml_path.as_ref().display()
		))?;
		let mut toml: Self = toml::from_str(&contents).context(format!(
			"Can't parse TOML file at {}",
			toml_path.as_ref().display()
		))?;
		if let Some(location) = toml.description {
			let contents = fs::read_to_string(&location)
				.context(format!("Can't read long description at {}", location))?;
			toml.description = Some(contents);
		}
		if let Some(location) = toml.license {
			let contents = fs::read_to_string(&location)
				.context(format!("Can't read license file at {}", location))?;
			toml.license = Some(contents);
		}
		Ok(toml)
	}

	/// Returns the version encoded according to PEP 427, Section "Escaping
	/// and Unicode"
	pub fn get_version_escaped(&self) -> String {
		let re = Regex::new(r"[^\w\d.]+").unwrap();
		re.replace_all(&self.version, "_").to_string()
	}

	/// Formats the metadata into a list where keys with multiple values
	/// become multiple single-valued key-value pairs. This format is needed for the pypi
	/// uploader and for the METADATA file inside wheels
	pub fn to_vec(&self) -> Vec<(String, String)> {
		let mut fields = vec![
			("Metadata-Version", self.metadata_version.clone()),
			("Name", self.name.clone()),
			("Version", self.get_version_escaped()),
		];

		let mut add_vec = |name, values: &[String]| {
			for i in values {
				fields.push((name, i.clone()));
			}
		};

		add_vec("Platform", &self.platform);
		add_vec("Supported-Platform", &self.supported_platform);
		add_vec("Classifier", &self.classifiers);
		add_vec("Requires-Dist", &self.requires_dist);
		add_vec("Provides-Dist", &self.provides_dist);
		add_vec("Obsoletes-Dist", &self.obsoletes_dist);
		add_vec("Requires-External", &self.requires_external);
		add_vec("Provides-Extra", &self.provides_extra);

		let mut add_option = |name, value: &Option<String>| {
			if let Some(some) = value.clone() {
				fields.push((name, some));
			}
		};

		add_option("Summary", &self.summary);
		add_option("Keywords", &self.keywords);
		add_option("Home-Page", &self.home_page);
		add_option("Download-URL", &self.download_url);
		add_option("Author", &self.author);
		add_option("Author-email", &self.author_email);
		add_option("Maintainer", &self.maintainer);
		add_option("Maintainer-email", &self.maintainer_email);
		add_option("License", &self.license.as_deref().map(fold_header));
		add_option("Requires-Python", &self.requires_python);
		add_option("Description-Content-Type", &self.description_content_type);
		// Project-URL is special
		// "A string containing a browsable URL for the project and a label for it, separated by a comma."
		// `Project-URL: Bug Tracker, http://bitbucket.org/tarek/distribute/issues/`
		for (key, value) in self.project_url.iter() {
			fields.push(("Project-URL", format!("{}, {}", key, value)))
		}

		// Description shall be last, so we can ignore RFC822 and just put the description
		// in the body
		// The borrow checker doesn't like us using add_option here
		if let Some(description) = &self.description {
			fields.push(("Description", description.clone()));
		}

		fields
			.into_iter()
			.map(|(k, v)| (k.to_string(), v))
			.collect()
	}

	/// Writes the format for the metadata file inside wheels
	fn to_file_contents(&self) -> String {
		let mut fields = self.to_vec();
		let mut out = "".to_string();
		let body = match fields.last() {
			Some((key, description)) if key == "Description" => {
				let desc = description.clone();
				fields.pop().unwrap();
				Some(desc)
			}
			Some((_, _)) => None,
			None => None,
		};

		for (key, value) in fields {
			use std::fmt::Write;
			writeln!(&mut out, "{key}: {value}").unwrap();
		}

		if let Some(body) = body {
			use std::fmt::Write;
			writeln!(&mut out, "\n{body}").unwrap()
		}

		out
	}

	fn metadata_version() -> String {
		"2.1".to_owned()
	}
}

/// Each file in the wheel gets a RECORD entry
#[derive(Debug)]
struct RecordEntry {
	relative_path: PathBuf,
	hash: String,
	size: usize,
}

impl RecordEntry {
	fn new(relative_path: PathBuf, hash: String, size: usize) -> Self {
		Self {
			relative_path,
			hash,
			size,
		}
	}
}

impl From<(String, String, usize)> for RecordEntry {
	fn from((relative_path, hash, size): (String, String, usize)) -> Self {
		Self::new(relative_path.into(), hash, size)
	}
}

#[derive(Debug, Clone)]
/// The tag used for the wheel filename.
/// See <https://www.python.org/dev/peps/pep-0491/#file-name-convention> for format.
struct Tag {
	/// The ABI tag to target - defaults to "none"
	abi_tag: Option<String>,
	/// The optional build tag, must start with a digit
	build_tag: Option<String>,
	/// The name of the wheel-wide distribution
	distribution: String,
	/// The platform tag to target, defaults to "any"
	platform: Option<String>,
	/// The targetted python environment and version
	python_tag: String,
	/// The disributions version
	version: String,
}

impl Tag {
	pub fn new(
		abi_tag: Option<String>,
		build_tag: Option<String>,
		distribution: String,
		platform: Option<String>,
		python_tag: String,
		version: String,
	) -> Self {
		Self {
			abi_tag,
			build_tag,
			distribution,
			platform,
			python_tag,
			version,
		}
	}
}

impl fmt::Display for Tag {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let abi = match &self.abi_tag {
			Some(tag) => tag.as_str(),
			None => "none",
		};
		let build_tag = match &self.build_tag {
			Some(tag) => format!("-{}", tag),
			None => "".to_owned(),
		};
		let platform = match &self.platform {
			Some(p) => p.as_str(),
			None => "any",
		};
		write!(
			f,
			"{}-{}{}-{}-{}-{}.whl",
			self.distribution, self.version, build_tag, self.python_tag, abi, platform
		)
	}
}

/// All the configurable options for the output wheel
// NOTE: top_level.txt is just a text file containing `metadata.name`
#[derive(Debug)]
struct WheelDistInfo {
	metadata: Metadata21,
	tag: Tag,
}

impl WheelDistInfo {
	fn new(metadata: Metadata21, tag: Tag) -> Self {
		Self { metadata, tag }
	}

	fn license(&self) -> String {
		self.metadata
			.license
			.as_ref()
			.unwrap_or(&"Unlicensed".to_owned())
			.to_string()
	}

	fn wheel_file(&self) -> String {
		let mut wheel_file = formatdoc!(
			"
				Wheel-Version: 1.0
				Generator: {name} ({version})
				Root-Is-Purelib: false
			",
			name = env!("CARGO_PKG_NAME"),
			version = env!("CARGO_PKG_VERSION"),
		);
		let tags = [self.tag.to_owned()];

		for tag in tags {
			use std::fmt::Write;
			writeln!(&mut wheel_file, "Tag: {tag}").unwrap();
		}

		wheel_file
	}

	fn wheel_name(&self) -> String {
		self.tag.to_string()
	}
}

/// Creates path if it doesn't exist.  Otherwise, no action
fn ensure_dir_exists(path: impl AsRef<Path>) -> Result<()> {
	let path = path.as_ref();
	if fs::metadata(&path).map(|m| m.is_dir()).unwrap_or(false) {
		// leave it!
	} else {
		fs::create_dir_all(&path)?;
	}
	Ok(())
}

/// If file exists, delete existing, then create
fn clobbering_create(path: impl AsRef<Path>) -> Result<File> {
	let path = path.as_ref();
	if fs::metadata(&path).map(|m| m.is_file()).unwrap_or(false) {
		fs::remove_file(&path)?;
	}
	Ok(fs::File::create(&path)?)
}

/// Fold long header field according to RFC 5322 section 2.2.3
/// https://datatracker.ietf.org/doc/html/rfc5322#section-2.2.3
fn fold_header(text: &str) -> String {
	let mut result = String::with_capacity(text.len());

	let options = textwrap::Options::new(78)
		.initial_indent("")
		.subsequent_indent("\t");
	for (i, line) in textwrap::wrap(text, options).iter().enumerate() {
		if i > 0 {
			result.push_str("\r\n");
		}
		if line.is_empty() {
			result.push('\t');
		} else {
			result.push_str(line);
		}
	}

	result
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn test_tag_to_string() {
		let tag = Tag::new(
			None,
			None,
			"dist".to_owned(),
			None,
			"cp36".to_owned(),
			"0.0.0".to_owned(),
		);
		assert_eq!(tag.to_string(), "dist-0.0.0-cp36-none-any.whl".to_owned());
		let tag = Tag::new(
			Some("abi3".to_owned()),
			Some("1".to_owned()),
			"dist".to_owned(),
			Some("manylinux_2_28_x86_64".to_owned()),
			"cp36".to_owned(),
			"0.0.0".to_owned(),
		);
		assert_eq!(
			tag.to_string(),
			"dist-0.0.0-1-cp36-abi3-manylinux_2_28_x86_64.whl".to_owned()
		);
	}
}
