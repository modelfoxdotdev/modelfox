use std::{
	io::Read,
	path::{Path, PathBuf},
};
use tangram_error::Result;

pub struct BlogPostSlugAndPath {
	pub path: PathBuf,
	pub slug: String,
}

pub fn blog_post_slugs_and_paths(path: &Path) -> Result<Vec<BlogPostSlugAndPath>> {
	let entries = std::fs::read_dir(path).unwrap();
	let slug_and_paths = entries
		.into_iter()
		.map(|entry| {
			let path = entry.unwrap().path();
			let slug = path.file_name().unwrap().to_str().unwrap().to_owned();
			BlogPostSlugAndPath { path, slug }
		})
		.collect();
	Ok(slug_and_paths)
}

pub struct BlogPost {
	pub slug: String,
	pub title: String,
	pub date: String,
	pub markdown: String,
}

impl BlogPost {
	pub fn from_path(path: &Path) -> Result<BlogPost> {
		let slug = path.file_name().unwrap().to_str().unwrap().to_owned();
		#[derive(serde::Deserialize)]
		pub struct BlogPostFrontMatter {
			pub title: String,
			pub date: String,
		}
		let mut reader = std::io::BufReader::new(std::fs::File::open(path.join("post.md"))?);
		let front_matter: BlogPostFrontMatter = serde_json::Deserializer::from_reader(&mut reader)
			.into_iter()
			.next()
			.unwrap()?;
		let mut markdown = String::new();
		reader.read_to_string(&mut markdown)?;
		Ok(BlogPost {
			slug,
			title: front_matter.title,
			date: front_matter.date,
			markdown,
		})
	}
}
