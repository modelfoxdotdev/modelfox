use pinwheel::prelude::*;

pub fn main() {
	tangram_client::client_start();
	hydrate::<tangram_www_benchmarks_common::Benchmarks>("benchmarks");
}
