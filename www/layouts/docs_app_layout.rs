use html::{component, html};
use tangram_ui as ui;

#[component]
pub fn DocsAppLayout() {
	html! {
		<ui::Window padding={Some(true)}>
			<div class="docs-app-layout-wrapper">
				{children}
			</div>
		</ui::Window>
	}
}
