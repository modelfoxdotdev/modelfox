use pinwheel::prelude::*;
use modelfox_www_content::Content;

mod page;

pub fn init() -> sunfish::Route {
	sunfish::Route::new_static_with_paths(
		|| {
			modelfox_www_content::DocsInternals::slugs()
				.unwrap()
				.into_iter()
				.map(|slug| format!("/docs/internals/{}", slug))
				.collect()
		},
		|path| {
			let slug =
				if let ["docs", "internals", slug] = *sunfish::path_components(&path).as_slice() {
					slug
				} else {
					panic!()
				};
			html(self::page::Page::new(slug.to_owned()))
		},
	)
}
