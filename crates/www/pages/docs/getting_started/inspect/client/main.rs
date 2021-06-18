use pinwheel::prelude::*;
use tangram_ui as ui;

#[derive(Clone)]
struct ThresholdMetrics {
	accuracy: f32,
	f1_score: f32,
	false_negatives: usize,
	false_positives: usize,
	precision: f32,
	recall: f32,
	threshold: f32,
	true_negatives: usize,
	true_positives: usize,
}

pub fn main() {
	console_error_panic_hook::set_once();
	ui::boot_code_select();
	hydrate::<tangram_www_docs_inspect_common::Tuning>("tuning");
}
