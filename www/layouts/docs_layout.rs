use crate::layout::Layout;
use html::{component, html, Props};
use tangram_ui as ui;

#[derive(Props)]
pub struct DocsLayoutProps {
	pub selected_page: DocsPage,
	pub headings: Option<Vec<Heading>>,
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
	Train(TrainPage),
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

#[derive(PartialEq)]
pub enum TrainPage {
	Configuration,
}

#[component]
pub fn DocsLayout(props: DocsLayoutProps) {
	html! {
		<Layout>
			<div class="docs-layout">
				<div class="docs-layout-left">
					<PageNav selected_page={props.selected_page} />
				</div>
				<div class="docs-layout-center">
					{children}
				</div>
				<div class="docs-layout-right">
					{props.headings.map(|headings| html! {
						<Headings headings={headings} />
					})}
				</div>
			</div>
		</Layout>
	}
}

#[derive(Props)]
pub struct PageNavProps {
	pub selected_page: DocsPage,
}

#[component]
pub fn PageNav(props: PageNavProps) {
	html! {
		<ui::Nav title?="Pages">
			<ui::NavSection title="Overview">
				<ui::NavItem
					title="Overview"
					href="/docs/"
					selected={Some(props.selected_page == DocsPage::Overview)}
				/>
			</ui::NavSection>
			<ui::NavSection title="Install">
				<ui::NavItem
					title="Install"
					href="/docs/install"
					selected={Some(props.selected_page == DocsPage::Install)}
				/>
			</ui::NavSection>
			<ui::NavSection title="Getting Started">
				<ui::NavItem
					title="Overview"
					href="/docs/getting_started/"
					selected={Some(props.selected_page == DocsPage::GettingStarted(GettingStartedPage::Index))}
				/>
				<ui::NavItem
					title="Train"
					href="/docs/getting_started/train"
					selected={Some(props.selected_page == DocsPage::GettingStarted(GettingStartedPage::Train))}
				/>
				<ui::NavItem
					title="Predict"
					href="/docs/getting_started/predict/"
					selected={Some(matches! {props.selected_page, DocsPage::GettingStarted(GettingStartedPage::Predict(_))})}
				>
					<ui::NavItem
						title="Elixir"
						href="/docs/getting_started/predict/elixir"
						selected={Some(props.selected_page == DocsPage::GettingStarted(GettingStartedPage::Predict(PredictPage::Elixir)))}
					/>
					<ui::NavItem
						title="Go"
						href="/docs/getting_started/predict/go"
						selected={Some(props.selected_page == DocsPage::GettingStarted(GettingStartedPage::Predict(PredictPage::Go)))}
					/>
					<ui::NavItem
						title="Node"
						href="/docs/getting_started/predict/node"
						selected={Some(props.selected_page == DocsPage::GettingStarted(GettingStartedPage::Predict(PredictPage::Node)))}
					/>
					<ui::NavItem
						title="Python"
						href="/docs/getting_started/predict/python"
						selected={Some(props.selected_page == DocsPage::GettingStarted(GettingStartedPage::Predict(PredictPage::Python)))}
					/>
					<ui::NavItem
						title="Ruby"
						href="/docs/getting_started/predict/ruby"
						selected={Some(props.selected_page == DocsPage::GettingStarted(GettingStartedPage::Predict(PredictPage::Ruby)))}
					/>
					<ui::NavItem
						title="Rust"
						href="/docs/getting_started/predict/rust"
						selected={Some(props.selected_page == DocsPage::GettingStarted(GettingStartedPage::Predict(PredictPage::Rust)))}
					/>
				</ui::NavItem>
				<ui::NavItem
					title="Inspect"
					href="/docs/getting_started/inspect"
					selected={Some(props.selected_page == DocsPage::GettingStarted(GettingStartedPage::Inspect))}
				/>
				<ui::NavItem
					title="Monitor"
					href="/docs/getting_started/monitor"
					selected={Some(props.selected_page == DocsPage::GettingStarted(GettingStartedPage::Monitor))}
				/>
			</ui::NavSection>
			<ui::NavSection title="Train">
				<ui::NavItem
					title="Configuration"
					href="/docs/train/configuration"
					selected={Some(props.selected_page == DocsPage::Train(TrainPage::Configuration))}
				/>
			</ui::NavSection>
			<ui::NavSection title="Languages">
				<ui::NavItem
					title="Elixir"
					href="/docs/languages/elixir"
					selected={Some(false)}
				/>
				<ui::NavItem
					title="Go"
					href="/docs/languages/go"
					selected={Some(false)}
				/>
				<ui::NavItem
					title="Node.js"
					href="/docs/languages/node"
					selected={Some(false)}
				/>
				<ui::NavItem
				  title="Python"
					href="/docs/languages/python"
					selected={Some(false)}
				/>
				<ui::NavItem
					title="Ruby"
					href="/docs/languages/ruby"
					selected={Some(false)}
				/>
				<ui::NavItem
					title="Rust"
					href="/docs/languages/rust"
					selected={Some(false)}
				/>
			</ui::NavSection>
		</ui::Nav>
	}
}

#[derive(Props)]
pub struct HeadingsProps {
	headings: Vec<Heading>,
}

#[component]
fn Headings(props: HeadingsProps) {
	html! {
		<ui::Nav>
			{props.headings.into_iter().map(|heading| html! {
				<ui::NavItem
					title={heading.title}
					href={Some(format!("#{}", heading.id))}
					selected={Some(false)}
				/>
			}).collect::<Vec<_>>()}
		</ui::Nav>
	}
}
