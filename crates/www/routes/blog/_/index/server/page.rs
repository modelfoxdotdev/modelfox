use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_content::{BlogPost, Content};
use tangram_www_layouts::{document::Document, page_layout::PageLayout};

#[derive(new)]
pub struct Page {
	slug: String,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let blog_post = BlogPost::from_slug(self.slug.clone()).unwrap();
		let author = if let Some(author) = blog_post.front_matter.author {
			Author::new().name(author.name).gravatar(author.gravatar)
		} else {
			Author::new()
				.name("Tangram Team")
				.gravatar("https://gravatar.com/avatar/048af5cc491ae1881dba85a78c228902")
		};
		let heading = div()
			.style("line-height", "1.5")
			.child(ui::H1::new().child(blog_post.front_matter.title.clone()))
			.child(div().class("blog-post-date").child(format!(
				"Originally published on {}",
				blog_post.front_matter.date
			)))
			.child(author);
		let url = format!("https://www.tangram.dev/blog/{}", self.slug);
		let about_tangram = div().child(
			ui::Card::new().child(
				ui::Markdown::new(
					"Tangram makes it easy for programmers to train, deploy, and monitor machine learning models. With Tangram, developers can train models and make predictions on the command line or with libraries for languages including [Elixir](https://hex.pm/packages/tangram), [Golang](https://pkg.go.dev/github.com/tangramdotdev/tangram-go), [Javascript](https://www.npmjs.com/package/@tangramdotdev/tangram), [PHP](https://packagist.org/packages/tangram/tangram), [Python](https://pypi.org/project/tangram), [Ruby](https://rubygems.org/gems/tangram), and [Rust](https://lib.rs/tangram), and learn about their models and monitor them in production from a web application. Watch the demo on the [homepage](https://www.tangram.dev).".into()))
			);
		let jobs = div().child(
			ui::Card::new().child(
				ui::Markdown::new("We are hiring! Tangram is open source and everything is written in Rust, from the core machine learning algorithms to the web application. Check it out on [GitHub](https://www.github.com/tangramdotdev/tangram). We are looking for programmers who love developer tools and are excited to build machine learning tools with Rust! If you are interested, email us at jobs@tangram.dev.".into())
			));
		Document::new()
			.child(
				PageLayout::new().child(
					div().class("blog-post-content").child(
						ui::S1::new()
							.child(heading)
							.child(about_tangram)
							.child(blog_post.markdown)
							.child(jobs)
							.child(
								ShareButtons::new()
									.title(blog_post.front_matter.title)
									.url(url),
							),
					),
				),
			)
			.into_node()
	}
}

#[derive(builder, Default, new)]
#[new(default)]
pub struct Author {
	#[builder]
	pub name: String,
	#[builder]
	pub gravatar: String,
}

impl Component for Author {
	fn into_node(self) -> Node {
		let facehole = img()
			.attribute("src", self.gravatar)
			.attribute("alt", self.name.clone())
			.class("blog-post-author-facehole");
		div()
			.class("blog-post-author")
			.child(facehole)
			.child(div().child(format!("By {}", self.name)))
			.into_node()
	}
}

#[derive(builder, Default, new)]
#[new(default)]
pub struct ShareButtons {
	#[builder]
	title: String,
	#[builder]
	url: String,
}

impl Component for ShareButtons {
	fn into_node(self) -> Node {
		let twitter = a()
			.href(format!(
				"https://twitter.com/intent/tweet?text={}&url={}",
				self.title, self.url
			))
			.child(Twitter);
		let hn = a()
			.href(format!(
				"https://news.ycombinator.com/submitlink?u={}&t={}",
				self.url, self.title
			))
			.child(HN);
		let reddit = a()
			.href(format!(
				"https://www.reddit.com/submit?url={}&title={}",
				self.url, self.title
			))
			.child(Reddit);
		ui::Card::new()
			.child(
				div()
					.class("blog-post-share-wrapper")
					.child("Share this post:")
					.child(
						div()
							.class("blog-post-share-buttons")
							.child(twitter)
							.child(hn)
							.child(reddit),
					),
			)
			.into_node()
	}
}

pub struct HN;
impl Component for HN {
	fn into_node(self) -> Node {
		let hn = include_str!("./hn.svg");
		div()
			.style("width", "30px")
			.attribute("alt", "hackernews")
			.inner_html(hn)
			.into_node()
	}
}

pub struct Twitter;
impl Component for Twitter {
	fn into_node(self) -> Node {
		let twitter = include_str!("./twitter.svg");
		div()
			.style("width", "30px")
			.attribute("alt", "twitter")
			.inner_html(twitter)
			.into_node()
	}
}

pub struct Reddit;
impl Component for Reddit {
	fn into_node(self) -> Node {
		let reddit = include_str!("./reddit.svg");
		div()
			.style("width", "30px")
			.attribute("alt", "reddit")
			.inner_html(reddit)
			.into_node()
	}
}
