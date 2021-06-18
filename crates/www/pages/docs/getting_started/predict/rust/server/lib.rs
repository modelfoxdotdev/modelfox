use pinwheel::prelude::*;

mod page;

pub fn init() -> sunfish::Page {
	sunfish::Page::new_static(|_| html(self::page::Page))
}
