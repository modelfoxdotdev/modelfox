use html::{component, html, Props};

#[derive(Props)]
pub struct DetailsProps {
	pub options: Option<Vec<DetailsOption>>,
	pub summary: Option<String>,
}

pub struct DetailsOption {
	pub href: String,
	pub title: String,
}

#[component]
pub fn Details(props: DetailsProps) {
	html! {
		<details class="details">
			<summary>
				{props.summary}
			</summary>
			<div class="details-list">
				{props.options.map(|options|
					options.into_iter().map(|option| {
						html! {
							<a class="details-list-item" href={option.href}>
							{option.title}
							</a>
						}
					}).collect::<Vec<_>>()
				)}
			</div>
		</details>
	}
}
