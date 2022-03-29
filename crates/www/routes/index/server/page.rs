use modelfox_ui as ui;
use modelfox_www_layouts::{document::Document, layout::Layout};
use pinwheel::prelude::*;

use crate::{
	inspection::Inspection, monitoring::Monitoring, predict::Predict,
	production_metrics::ProductionMetrics, production_predictions::ProductionExplanations,
	production_stats::ProductionStats, train::Train, tuning::Tuning,
};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.client("modelfox_www_index_client")
			.child(
				Layout::new().child(
					div()
						.class("index-grid")
						.child(Hero)
						.child(Train)
						.child(Predict)
						.child(Inspection)
						.child(Tuning)
						.child(Monitoring)
						.child(ProductionExplanations)
						.child(ProductionStats)
						.child(ProductionMetrics),
				),
			)
			.into_node()
	}
}

pub struct Hero;

impl Component for Hero {
	fn into_node(self) -> Node {
		let title = h1()
			.class("index-hero-title")
			.child("ModelFox makes it easy to train, deploy, and monitor machine learning models.");
		let subtitle= "Train a model from a CSV file on the command line. Make predictions from Elixir, Go, JavaScript, PHP, Python, Ruby, or Rust. Learn about your models and monitor them in production from your browser.";
		let subtitle = div().class("index-hero-subtitle").child(subtitle);
		let buttons = div()
			.class("index-hero-buttons")
			.child(
				ui::Button::new()
					.color(ui::colors::BLUE.to_owned())
					.href("https://github.com/modelfoxdotdev/modelfox".to_owned())
					.child("View on GitHub"),
			)
			.child(
				ui::Button::new()
					.color(ui::colors::TEAL.to_owned())
					.href("/docs/install".to_owned())
					.child("Install the CLI"),
			);
		div()
			.class("index-hero-wrapper")
			.child(title)
			.child(subtitle)
			.child(buttons)
			.into_node()
	}
}
