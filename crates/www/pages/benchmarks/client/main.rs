use pinwheel::prelude::*;

pub fn main() {
	tangram_ui::client_start();
	hydrate::<tangram_www_benchmarks_common::Benchmarks>("benchmarks");
}
