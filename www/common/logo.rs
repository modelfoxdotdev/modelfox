use html::{component, html, Props};

#[derive(Props)]
pub struct LogoProps {
	pub class: Option<String>,
	pub color: Option<String>,
	pub color_scheme: LogoScheme,
}

#[derive(PartialEq)]
pub enum LogoScheme {
	Multi,
	Solid,
}

#[component]
pub fn Logo(props: LogoProps) {
	struct ShapesColors {
		trapezoid: String,
		square: String,
		medium_triangle: String,
		small_triangle1: String,
		small_triangle2: String,
		large_triangle1: String,
		large_triangle2: String,
	}
	let shapes_colors = if props.color_scheme == LogoScheme::Multi {
		ShapesColors {
			trapezoid: "var(--pink)".to_owned(),
			square: "var(--yellow)".to_owned(),
			medium_triangle: "var(--teal)".to_owned(),
			small_triangle1: "var(--purple)".to_owned(),
			small_triangle2: "var(--indigo)".to_owned(),
			large_triangle1: "var(--blue)".to_owned(),
			large_triangle2: "var(--green)".to_owned(),
		}
	} else {
		let color = props
			.color
			.unwrap_or_else(|| "var(--accent-color)".to_owned());
		ShapesColors {
			trapezoid: color.clone(),
			square: color.clone(),
			medium_triangle: color.clone(),
			small_triangle1: color.clone(),
			small_triangle2: color.clone(),
			large_triangle1: color.clone(),
			large_triangle2: color,
		}
	};
	html! {
		<svg
			class={props.class}
			fill="none"
			height="100%"
			viewBox="0 0 200 200"
			width="100%"
			xmlns="http://www.w3.org/2000/svg"
		>
			<desc>{"tangram"}</desc>
			<path d="M0 0L50 50V150L0 100V0Z" fill={shapes_colors.trapezoid} />
			<path d="M100 100L150 150L100 200L50 150L100 100Z" fill={shapes_colors.square} />
			<path d="M0 100L100 200H0V100Z" fill={shapes_colors.medium_triangle} />
			<path d="M150 150L200 200H100L150 150Z" fill={shapes_colors.small_triangle2} />
			<path d="M50 50L100 100L50 150V50Z" fill={shapes_colors.small_triangle1} />
			<path d="M200 0V200L100 100L200 0Z" fill={shapes_colors.large_triangle2} />
			<path d="M200 0L100 100L0 0H200Z" fill={shapes_colors.large_triangle1} />
		</svg>
	}
}
