use pinwheel::prelude::*;

mod page;

pub fn init() -> sunfish::Route {
	sunfish::Route::new_static(|_| html(self::page::Page))
}
