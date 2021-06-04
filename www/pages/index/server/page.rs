use self::{
	inspection::Inspection, monitoring::Monitoring, predict::Predict,
	production_metrics::ProductionMetrics, production_predictions::ProductionExplanations,
	production_stats::ProductionStats, train::Train, tuning::Tuning,
};
use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_layouts::{document::Document, layout::Layout};

mod inspection;
mod monitoring;
mod predict;
mod production_metrics;
mod production_predictions;
mod production_stats;
mod train;
mod tuning;

#[derive(ComponentBuilder)]
pub struct Page {
	#[children]
	pub children: Vec<Node>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.client("tangram_www_index_client")
			.child(
				Layout::new().child(
					div()
						.class("index-grid")
						.child(Hero::new())
						.child(Train::new())
						.child(Predict::new())
						.child(Inspection::new())
						.child(Tuning::new())
						.child(Monitoring::new())
						.child(ProductionExplanations::new())
						.child(ProductionStats::new())
						.child(ProductionMetrics::new()),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct Hero {
	#[children]
	pub children: Vec<Node>,
}

impl Component for Hero {
	fn into_node(self) -> Node {
		let title = h1()
			.class("index-hero-title")
			.child("Tangram is an automated machine learning framework designed for programmers.");
		let subtitle= "Train a model from a CSV file on the command line. Make predictions from Elixir, Go, JavaScript, Python, Ruby, or Rust. Learn about your models and monitor them in production from your browser.";
		let subtitle = div().class("index-hero-subtitle").child(subtitle);
		let buttons = div()
			.class("index-hero-buttons")
			.child(
				ui::Button::new()
					.href("https://github.com/tangramxyz/tangram".to_owned())
					.child("View on GitHub"),
			)
			.child(
				ui::Button::new()
					.href("/docs/install".to_owned())
					.color(ui::colors::TEAL.to_owned())
					.child("Install the CLI"),
			)
			.child(
				ui::Button::new()
					.href("https://cozycal.com/tangram/demo".to_owned())
					.color(ui::colors::GREEN.to_owned())
					.child("Schedule a Demo"),
			);
		div()
			.class("index-hero-wrapper")
			.child(title)
			.child(subtitle)
			.child(buttons)
			.into_node()
	}
}

// <div class="index-video-placeholder">
// 	<iframe
// 		allow_full_screen={true}
// 		class="index-video"
// 		src="https://player.vimeo.com/video/385352664"
// 		title="tangram video">
// 	</iframe>
// </div>
