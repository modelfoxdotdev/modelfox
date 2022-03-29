use pinwheel::prelude::*;

pub fn main() {
	modelfox_ui::client_start();
	hydrate::<modelfox_www_benchmarks_common::Benchmarks>("benchmarks");
}
