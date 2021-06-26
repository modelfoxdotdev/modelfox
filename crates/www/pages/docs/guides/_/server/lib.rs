use pinwheel::prelude::*;
use tangram_www_content::Content;

mod page;

pub fn init() -> sunfish::Page {
	sunfish::Page::new_static_with_paths(
		|| {
			tangram_www_content::DocsGuide::slugs()
				.unwrap()
				.into_iter()
				.map(|slug| format!("/docs/guides/{}", slug))
				.collect()
		},
		|path| {
			let slug = if let ["docs", "guides", slug] = *sunfish::path_components(&path).as_slice()
			{
				slug
			} else {
				panic!()
			};
			html(self::page::Page::new(slug.to_owned()))
		},
	)
}
