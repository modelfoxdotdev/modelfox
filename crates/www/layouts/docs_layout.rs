use crate::layout::Layout;
use modelfox_ui as ui;
use modelfox_www_content::{Content, DocsGuide, DocsInternals};
use pinwheel::prelude::*;

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

#[derive(PartialEq, Eq)]
pub enum DocsPage {
	Overview,
	Install,
	GettingStarted(GettingStartedPage),
	Guides(String),
	Internals(String),
}

#[derive(PartialEq, Eq)]
pub enum GettingStartedPage {
	Index,
	Train,
	Predict(PredictPage),
	Inspect,
	Monitor,
}

#[derive(PartialEq, Eq)]
pub enum PredictPage {
	Index,
	CLI,
	Elixir,
	Go,
	Node,
	PHP,
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
									.title("CLI".to_owned())
									.href("/docs/getting_started/predict/cli".to_owned())
									.selected(matches!(
										self.selected_page,
										Some(DocsPage::GettingStarted(
											GettingStartedPage::Predict(PredictPage::CLI)
										)),
									)),
							)
							.child(
								ui::NavItem::new()
									.title("Elixir".to_owned())
									.href("/docs/getting_started/predict/elixir".to_owned())
									.selected(matches!(
										self.selected_page,
										Some(DocsPage::GettingStarted(
											GettingStartedPage::Predict(PredictPage::Elixir)
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
									.title("JavaScript".to_owned())
									.href("/docs/getting_started/predict/javascript".to_owned())
									.selected(matches!(
										self.selected_page,
										Some(DocsPage::GettingStarted(
											GettingStartedPage::Predict(PredictPage::Node)
										)),
									)),
							)
							.child(
								ui::NavItem::new()
									.title("PHP".to_owned())
									.href("/docs/getting_started/predict/php".to_owned())
									.selected(matches!(
										self.selected_page,
										Some(DocsPage::GettingStarted(
											GettingStartedPage::Predict(PredictPage::PHP)
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
											GettingStartedPage::Predict(PredictPage::Python)
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
											GettingStartedPage::Predict(PredictPage::Ruby)
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
											GettingStartedPage::Predict(PredictPage::Rust)
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
			.child(ui::NavSection::new("Internals".to_owned()).children(
				DocsInternals::list().unwrap().into_iter().map(|internal| {
					ui::NavItem::new()
						.title(internal.front_matter.title)
						.href(format!("/docs/internals/{}", internal.slug))
						.selected(self.selected_page == Some(DocsPage::Internals(internal.slug)))
				}),
			))
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

#[derive(Default, new)]
#[new(default)]
pub struct PrevNextButtons {
	prev: Option<PrevNextButton>,
	next: Option<PrevNextButton>,
}

pub struct PrevNextButton {
	href: String,
	title: String,
}

impl PrevNextButtons {
	#[must_use]
	pub fn prev(mut self, href: impl Into<String>, title: impl Into<String>) -> PrevNextButtons {
		self.prev = Some(PrevNextButton {
			href: href.into(),
			title: title.into(),
		});
		self
	}

	#[must_use]
	pub fn next(mut self, href: impl Into<String>, title: impl Into<String>) -> PrevNextButtons {
		self.next = Some(PrevNextButton {
			href: href.into(),
			title: title.into(),
		});
		self
	}
}

impl Component for PrevNextButtons {
	fn into_node(self) -> Node {
		div()
			.class("docs-prev-next-buttons")
			.child(self.prev.map(|button| {
				ui::Link::new()
					.href(button.href)
					.child(format!("< Previous: {}", button.title))
			}))
			.child(self.next.map(|button| {
				ui::Link::new()
					.href(button.href)
					.child(format!("Next: {} >", button.title))
			}))
			.into_node()
	}
}
