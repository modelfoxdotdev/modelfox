use crate::layout::Layout;
use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_content::{Content, DocsGuide};

#[derive(builder, children, Default, new)]
#[new(default)]
pub struct DocsLayout {
	#[builder]
	pub selected_page: Option<DocsPage>,
	#[builder]
	pub headings: Option<Vec<Heading>>,
	pub children: Vec<Node>,
}

pub struct Heading {
	id: String,
	title: String,
}

#[derive(PartialEq)]
pub enum DocsPage {
	Overview,
	Install,
	GettingStarted(GettingStartedPage),
	Guides(String),
}

#[derive(PartialEq)]
pub enum GettingStartedPage {
	Index,
	Train,
	Predict(PredictPage),
	Inspect,
	Monitor,
}

#[derive(PartialEq)]
pub enum PredictPage {
	Index,
	Elixir,
	Go,
	Node,
	Python,
	Ruby,
	Rust,
}

impl Component for DocsLayout {
	fn into_node(self) -> Node {
		Layout::new()
			.child(
				div()
					.class("docs-layout")
					.child(div().class("docs-layout-left").child(PageNav {
						selected_page: self.selected_page,
					}))
					.child(div().class("docs-layout-center").child(self.children))
					.child(
						div()
							.class("docs-layout-right")
							.child(self.headings.map(|headings| Headings { headings })),
					),
			)
			.into_node()
	}
}

pub struct PageNav {
	pub selected_page: Option<DocsPage>,
}

impl Component for PageNav {
	fn into_node(self) -> Node {
		ui::Nav::new()
			.title("Pages".to_owned())
			.child(
				ui::NavSection::new("Overview".to_owned()).child(
					ui::NavItem::new()
						.title("Overview".to_owned())
						.href("/docs/".to_owned())
						.selected(matches!(self.selected_page, Some(DocsPage::Overview))),
				),
			)
			.child(
				ui::NavSection::new("Install".to_owned()).child(
					ui::NavItem::new()
						.title("Install".to_owned())
						.href("/docs/install".to_owned())
						.selected(matches!(self.selected_page, Some(DocsPage::Install))),
				),
			)
			.child(
				ui::NavSection::new("Getting Started".to_owned())
					.child(
						ui::NavItem::new()
							.title("Overview".to_owned())
							.href("/docs/getting_started/".to_owned())
							.selected(matches!(
								self.selected_page,
								Some(DocsPage::GettingStarted(GettingStartedPage::Index)),
							)),
					)
					.child(
						ui::NavItem::new()
							.title("Train".to_owned())
							.href("/docs/getting_started/train".to_owned())
							.selected(matches!(
								self.selected_page,
								Some(DocsPage::GettingStarted(GettingStartedPage::Train)),
							)),
					)
					.child(
						ui::NavItem::new()
							.title("Predict".to_owned())
							.href("/docs/getting_started/predict/".to_owned())
							.selected(matches!(
								self.selected_page,
								Some(DocsPage::GettingStarted(GettingStartedPage::Predict(_)))
							))
							.child(
								ui::NavItem::new()
									.title("Elixir".to_owned())
									.href("/docs/getting_started/predict/elixir".to_owned())
									.selected(matches!(
										self.selected_page,
										Some(DocsPage::GettingStarted(
											GettingStartedPage::Predict(PredictPage::Elixir,)
										)),
									)),
							)
							.child(
								ui::NavItem::new()
									.title("Go".to_owned())
									.href("/docs/getting_started/predict/go".to_owned())
									.selected(matches!(
										self.selected_page,
										Some(DocsPage::GettingStarted(
											GettingStartedPage::Predict(PredictPage::Go)
										))
									)),
							)
							.child(
								ui::NavItem::new()
									.title("Node".to_owned())
									.href("/docs/getting_started/predict/node".to_owned())
									.selected(matches!(
										self.selected_page,
										Some(DocsPage::GettingStarted(
											GettingStartedPage::Predict(PredictPage::Node,)
										)),
									)),
							)
							.child(
								ui::NavItem::new()
									.title("Python".to_owned())
									.href("/docs/getting_started/predict/python".to_owned())
									.selected(matches!(
										self.selected_page,
										Some(DocsPage::GettingStarted(
											GettingStartedPage::Predict(PredictPage::Python,)
										)),
									)),
							)
							.child(
								ui::NavItem::new()
									.title("Ruby".to_owned())
									.href("/docs/getting_started/predict/ruby".to_owned())
									.selected(matches!(
										self.selected_page,
										Some(DocsPage::GettingStarted(
											GettingStartedPage::Predict(PredictPage::Ruby,)
										)),
									)),
							)
							.child(
								ui::NavItem::new()
									.title("Rust".to_owned())
									.href("/docs/getting_started/predict/rust".to_owned())
									.selected(matches!(
										self.selected_page,
										Some(DocsPage::GettingStarted(
											GettingStartedPage::Predict(PredictPage::Rust,)
										)),
									)),
							),
					)
					.child(
						ui::NavItem::new()
							.title("Inspect".to_owned())
							.href("/docs/getting_started/inspect".to_owned())
							.selected(matches!(
								self.selected_page,
								Some(DocsPage::GettingStarted(GettingStartedPage::Inspect)),
							)),
					)
					.child(
						ui::NavItem::new()
							.title("Monitor".to_owned())
							.href("/docs/getting_started/monitor".to_owned())
							.selected(matches!(
								self.selected_page,
								Some(DocsPage::GettingStarted(GettingStartedPage::Monitor)),
							)),
					),
			)
			.child(ui::NavSection::new("Guides".to_owned()).children(
				DocsGuide::list().unwrap().into_iter().map(|guide| {
					ui::NavItem::new()
						.title(guide.front_matter.title)
						.href(format!("/docs/guides/{}", guide.slug))
						.selected(self.selected_page == Some(DocsPage::Guides(guide.slug)))
				}),
			))
			.child(
				ui::NavSection::new("Languages".to_owned())
					.child(
						ui::NavItem::new()
							.title("C".to_owned())
							.href("/docs/languages/c".to_owned())
							.selected(false),
					)
					.child(
						ui::NavItem::new()
							.title("Elixir".to_owned())
							.href("/docs/languages/elixir".to_owned())
							.selected(false),
					)
					.child(
						ui::NavItem::new()
							.title("Go".to_owned())
							.href("/docs/languages/go".to_owned())
							.selected(false),
					)
					.child(
						ui::NavItem::new()
							.title("JavaScript".to_owned())
							.href("/docs/languages/javascript".to_owned())
							.selected(false),
					)
					.child(
						ui::NavItem::new()
							.title("Python".to_owned())
							.href("/docs/languages/python".to_owned())
							.selected(false),
					)
					.child(
						ui::NavItem::new()
							.title("Ruby".to_owned())
							.href("/docs/languages/ruby".to_owned())
							.selected(false),
					)
					.child(
						ui::NavItem::new()
							.title("Rust".to_owned())
							.href("/docs/languages/rust".to_owned())
							.selected(false),
					),
			)
			.into_node()
	}
}

pub struct Headings {
	headings: Vec<Heading>,
}

impl Component for Headings {
	fn into_node(self) -> Node {
		ui::Nav::new()
			.children(self.headings.into_iter().map(|heading| {
				ui::NavItem::new()
					.title(heading.title)
					.href(format!("#{}", heading.id))
					.selected(false)
			}))
			.into_node()
	}
}
