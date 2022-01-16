use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_layouts::{document::Document, page_layout::PageLayout};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let p1 = ui::P::new().child("At Tangram, we want to make it really easy for developers to train machine learning models and deploy them directly in their Javascript, Ruby, Elixir, Golang, PHP, Python, and Rust apps. We also want to give developers all the tools they need to understand how their model works and monitor it once it is deployed.");
		let link = ui::Link::new()
			.href("mailto:hello@tangram.dev".to_string())
			.title("hello@tangram.dev".to_string())
			.child("hello@tangram.dev");
		let p2 = ui::P::new()
			.child("We would love to hear from you, please don't hesistate to reach out! ")
			.child(link);
		let tangram = ui::S2::new()
			.child(ui::H1::new("About"))
			.child(p1)
			.child(p2);
		let david = Person::new()
			.name("David Yamnitsky")
			.linkedin("https://www.linkedin.com/in/david-yamnitsky".to_string())
			.github("https://www.github.com/nitsky".to_string())
			.gravatar("https://gravatar.com/avatar/5833197f9632bacbd8820f3b6cbf82c2?s=100")
			.twitter("https://twitter.com/davidyamnitsky".to_string());
		let bella = Person::new()
			.name("Isabella Tromba")
			.linkedin("https://www.linkedin.com/in/isabella-tromba".to_string())
			.github("https://www.github.com/isabella".to_string())
			.gravatar("https://gravatar.com/avatar/b5c16153bae7a6fa6663d7f555906dd0?s=100")
			.twitter("https://twitter.com/isabellatromba".to_string());
		let ben = Person::new()
			.name("Ben Lovy")
			.linkedin("https://www.linkedin.com/in/benlovy".to_string())
			.github("https://github.com/deciduously".to_string())
			.gravatar("https://gravatar.com/avatar/98c040317325f49915f91168ec8805bc?s=100")
			.twitter("https://twitter.com/ben_deciduously".to_string());
		let team = ui::S2::new()
			.child(ui::H1::new("Team"))
			.child(david)
			.child(bella)
			.child(ben);
		let content = div()
			.style("display", "grid")
			.style("justify-items", "center")
			.child(ui::S1::new().child(tangram).child(team));
		Document::new()
			.child(PageLayout::new().child(content))
			.into_node()
	}
}

#[derive(builder, Default, new)]
#[new(default)]
pub struct Person {
	#[builder]
	pub name: String,
	#[builder]
	pub gravatar: String,
	#[builder]
	pub linkedin: Option<String>,
	#[builder]
	pub github: Option<String>,
	#[builder]
	pub twitter: Option<String>,
}

impl Component for Person {
	fn into_node(self) -> Node {
		let facehole = img()
			.attribute("src", self.gravatar)
			.style("border-radius", "50px");
		div()
			.style("display", "grid")
			.style("grid", "auto/auto 1fr")
			.style("gap", "2rem")
			.style("justify-items", "start")
			.child(facehole)
			.child(
				div()
					.style("line-height", "1.5")
					.child(div().style("font-size", "1.25rem").child(self.name))
					.child(
						div()
							.style("display", "grid")
							.style("grid-auto-flow", "column")
							.style("gap", "1rem")
							.child(
								ui::Link::new()
									.target("_blank".to_string())
									.href(self.github)
									.child("GitHub"),
							)
							.child(
								ui::Link::new()
									.target("_blank".to_string())
									.href(self.linkedin)
									.child("LinkedIn"),
							)
							.child(
								ui::Link::new()
									.target("_blank".to_string())
									.href(self.twitter)
									.child("Twitter"),
							),
					),
			)
			.into_node()
	}
}
