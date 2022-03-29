use anyhow::Result;
use modelfox_ui as ui;
use std::io::Read;
use std::path::{Path, PathBuf};
use sunfish::{include_dir, include_dir::IncludeDir};

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
	type FrontMatter = BlogPostFrontMatter;
	fn content() -> IncludeDir {
		include_dir!("crates/www/content/blog")
	}
}

pub struct DocsGuide;

#[derive(serde::Deserialize)]
pub struct DocsGuideFrontMatter {
	pub title: String,
}

impl Content for DocsGuide {
	type FrontMatter = DocsGuideFrontMatter;
	fn content() -> IncludeDir {
		include_dir!("crates/www/content/docs_guides")
	}
}

pub struct DocsInternals;

#[derive(serde::Deserialize)]
pub struct DocsInternalsFrontMatter {
	pub title: String,
}

impl Content for DocsInternals {
	type FrontMatter = DocsInternalsFrontMatter;
	fn content() -> IncludeDir {
		include_dir!("crates/www/content/docs_internals")
	}
}

pub struct ContentItem<T> {
	pub path: PathBuf,
	pub slug: String,
	pub front_matter: T,
	pub markdown: ui::Markdown,
}

pub trait Content: Sized {
	type FrontMatter: serde::de::DeserializeOwned;
	fn content() -> IncludeDir;

	fn slugs() -> Result<Vec<String>> {
		let content = Self::content();
		let slug_and_paths = content
			.into_iter()
			.map(|(entry, _)| {
				entry
					.parent()
					.unwrap()
					.file_name()
					.unwrap()
					.to_str()
					.unwrap()
					.to_owned()
			})
			.collect::<Vec<_>>();
		Ok(slug_and_paths)
	}

	fn list() -> Result<Vec<ContentItem<Self::FrontMatter>>> {
		Self::slugs()?
			.into_iter()
			.map(Self::from_slug)
			.collect::<Result<Vec<_>>>()
	}

	fn from_slug(slug: String) -> Result<ContentItem<Self::FrontMatter>> {
		let post_path = Path::new(&slug).join("post.md");
		let post = Self::content().read(&post_path).unwrap().data();
		let mut reader = std::io::Cursor::new(post);
		let front_matter: Self::FrontMatter = serde_json::Deserializer::from_reader(&mut reader)
			.into_iter()
			.next()
			.unwrap()?;
		let mut markdown = String::new();
		reader.read_to_string(&mut markdown)?;
		let markdown = ui::Markdown::new(markdown);
		Ok(ContentItem {
			path: post_path,
			slug,
			front_matter,
			markdown,
		})
	}
}
