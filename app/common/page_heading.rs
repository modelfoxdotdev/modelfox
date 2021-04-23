use html::{component, html};

#[component]
pub fn PageHeading() {
	html! {
		<div class="page-heading">
			{children}
		</div>
	}
}

#[component]
pub fn PageHeadingButtons() {
	html! {
		<div class="page-heading-buttons">
			{children}
		</div>
	}
}
