use html::{component, html};
use tangram_app_common::logo::{Logo, LogoScheme};
use tangram_ui as ui;

#[component]
pub fn AuthLayout() {
	html! {
		<div class="auth-layout">
			<div class="auth-layout-logo-wrapper">
				<Logo color_scheme={LogoScheme::Multi} />
			</div>
			<div class="auth-layout-card-wrapper">
				<ui::Card>{children}</ui::Card>
			</div>
		</div>
	}
}
