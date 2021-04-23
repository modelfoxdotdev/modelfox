use self::{
	inspection::Inspection, monitoring::Monitoring, predict::Predict,
	production_metrics::ProductionMetrics, production_predictions::ProductionExplanations,
	production_stats::ProductionStats, train::Train, tuning::Tuning,
};
use html::{component, html};
use tangram_serve::{self, client};
use tangram_ui as ui;
use tangram_www_layouts::{
	document::{Document, DocumentProps},
	layout::Layout,
};

mod inspection;
mod monitoring;
mod predict;
mod production_metrics;
mod production_predictions;
mod production_stats;
mod train;
mod tuning;

#[component]
pub fn Page() {
	let document_props = DocumentProps {
		client_wasm_js_src: Some(client!()),
	};
	html! {
		<Document {document_props}>
			<Layout>
				<div class="index-grid">
					<Hero />
					<Train />
					<Predict />
					<Inspection />
					<Tuning />
					<Monitoring />
					<ProductionExplanations />
					<ProductionStats />
					<ProductionMetrics />
				</div>
			</Layout>
		</Document>
	}
}

#[component]
pub fn Hero() {
	html! {
		<div class="index-hero-wrapper">
			<h1 class="index-hero-title">
				{"Tangram is an automated machine learning framework designed for programmers."}
			</h1>
			<div class="index-hero-subtitle">
				{"Train a model from a CSV file on the command line. Make predictions from Elixir, Go, JavaScript, Python, Ruby, or Rust. Learn about your models and monitor them in production from your browser."}
			</div>
			<div class="index-hero-buttons">
				<ui::Button href?="https://github.com/tangramxyz/tangram">
					{"View on GitHub"}
				</ui::Button>
				<ui::Button href?="/docs/install">
					{"Install the CLI"}
				</ui::Button>
			</div>
			// <Video />
		</div>
	}
}

// #[component]
// pub fn Video() {
// 	html! {
// 		<div class="index-video-placeholder">
// 			<iframe
// 				allow_full_screen={true}
// 				class="index-video"
// 				src="https://player.vimeo.com/video/385352664"
// 				title="tangram video">
// 			</iframe>
// 		</div>
// 	}
// }
