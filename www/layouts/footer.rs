use html::{component, html};
use tangram_www_common::logo::{Logo, LogoScheme};

#[component]
pub fn Footer() {
	html! {
		<div class="footer-wrapper">
			<Logo class="footer-logo" color_scheme={LogoScheme::Multi} color={None} />
			<p class="footer-copyright">{"Tangram © 2020"}</p>
		</div>
	}
}
