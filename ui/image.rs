use html::{component, html, Props};

#[derive(Props)]
pub struct ImgProps {
	pub alt: String,
	pub src: String,
}

#[component]
pub fn Img(props: ImgProps) {
	html! (
		<details class="image-details">
			<summary class="image-details-summary">
				<img alt={props.alt.clone()} class="image-img" src={props.src.clone()} />
			</summary>
			<div class="image-viewer">
				<img alt={props.alt} class="image-viewer-img" src={props.src} />
			</div>
		</details>
	)
}
