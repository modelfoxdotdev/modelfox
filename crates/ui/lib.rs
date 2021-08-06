pub use self::{
	alert::*, alert_icon::*, asciicast::*, avatar::*, button::*, callout::*, card::*, code::*,
	confusion_matrix::*, confusion_matrix_comparison::*, details::*, form::*, image::*, layout::*,
	link::*, markdown::*, nav::*, number_card::*, number_comparison_card::*, slider::*, tab_bar::*,
	table::*, theme::*, token::*, topbar::*, window::*,
};
pub use indoc::{formatdoc, indoc as doc};
pub use tangram_number_formatter::*;

mod alert;
mod alert_icon;
mod asciicast;
mod avatar;
mod button;
mod callout;
mod card;
mod code;
mod confusion_matrix;
mod confusion_matrix_comparison;
mod details;
mod form;
mod image;
mod layout;
mod link;
mod markdown;
mod nav;
mod number_card;
mod number_comparison_card;
mod slider;
mod tab_bar;
mod table;
mod theme;
mod token;
mod topbar;
mod window;

pub fn client_start() {
	console_error_panic_hook::set_once();
	tracing_wasm::set_as_global_default();
}

pub fn percent_encode(input: &str) -> std::borrow::Cow<str> {
	percent_encoding::utf8_percent_encode(input, percent_encoding::NON_ALPHANUMERIC).into()
}

pub fn percent_decode(input: &str) -> std::borrow::Cow<str> {
	percent_encoding::percent_decode_str(input).decode_utf8_lossy()
}
