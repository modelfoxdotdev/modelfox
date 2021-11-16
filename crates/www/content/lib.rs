use anyhow::Result;
use std::{
	io::Read,
	path::{Path, PathBuf},
};
use tangram_ui as ui;

pub struct BlogPost;

#[derive(serde::Deserialize)]
pub struct BlogPostFrontMatter {
	pub title: String,
	pub date: String,
	pub author: Option<BlogPostAuthor>,
}

#[derive(serde::Deserialize)]
pub struct BlogPostAuthor {
	pub name: String,
	pub gravatar: String,
}

impl Content for BlogPost {
	const PATH: &'static str = "crates/www/content/blog";
	type FrontMatter = BlogPostFrontMatter;
}

pub struct DocsGuide;

#[derive(serde::Deserialize)]
pub struct DocsGuideFrontMatter {
	pub title: String,
}

impl Content for DocsGuide {
	const PATH: &'static str = "crates/www/content/docs_guides";
	type FrontMatter = DocsGuideFrontMatter;
}

pub struct DocsInternals;

#[derive(serde::Deserialize)]
pub struct DocsInternalsFrontMatter {
	pub title: String,
}

impl Content for DocsInternals {
	const PATH: &'static str = "crates/www/content/docs_internals";
	type FrontMatter = DocsInternalsFrontMatter;
}

pub struct ContentItem<T> {
	pub path: PathBuf,
	pub slug: String,
	pub front_matter: T,
	pub markdown: ui::Markdown,
}

pub trait Content: Sized {
	const PATH: &'static str;
	type FrontMatter: serde::de::DeserializeOwned;

	fn slugs() -> Result<Vec<String>> {
		let entries = std::fs::read_dir(Path::new(Self::PATH)).unwrap();
		let slug_and_paths = entries
			.into_iter()
			.map(|entry| {
				entry
					.unwrap()
					.path()
					.file_name()
					.unwrap()
					.to_str()
					.unwrap()
					.to_owned()
			})
			.collect();
		Ok(slug_and_paths)
	}

	fn list() -> Result<Vec<ContentItem<Self::FrontMatter>>> {
		Self::slugs()?
			.into_iter()
			.map(Self::from_slug)
			.collect::<Result<Vec<_>>>()
	}

	fn from_slug(slug: String) -> Result<ContentItem<Self::FrontMatter>> {
		let dir_path = Path::new(Self::PATH).join(&slug);
		let post_path = dir_path.join("post.md");
		let mut reader = std::io::BufReader::new(std::fs::File::open(&post_path)?);
		let front_matter: Self::FrontMatter = serde_json::Deserializer::from_reader(&mut reader)
			.into_iter()
			.next()
			.unwrap()?;
		let mut markdown = String::new();
		reader.read_to_string(&mut markdown)?;
		let markdown = ui::Markdown::new(markdown.into());
		Ok(ContentItem {
			path: post_path,
			slug,
			front_matter,
			markdown,
		})
	}
}
