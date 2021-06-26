use pinwheel::prelude::*;
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct Logo {
	#[optional]
	pub color_scheme: Option<LogoColorScheme>,
	#[optional]
	pub class: Option<String>,
	#[optional]
	pub color: Option<String>,
}

#[derive(PartialEq)]
pub enum LogoColorScheme {
	Solid,
	Multi,
}

impl Component for Logo {
	fn into_node(self) -> Node {
		struct ShapesColors {
			trapezoid: String,
			square: String,
			medium_triangle: String,
			small_triangle1: String,
			small_triangle2: String,
			large_triangle1: String,
			large_triangle2: String,
		}
		let color_scheme = self.color_scheme.unwrap_or(LogoColorScheme::Solid);
		let shapes_colors = match color_scheme {
			LogoColorScheme::Solid => {
				let color = self.color.unwrap_or_else(|| ui::colors::ACCENT.to_owned());
				ShapesColors {
					trapezoid: color.clone(),
					square: color.clone(),
					medium_triangle: color.clone(),
					small_triangle1: color.clone(),
					small_triangle2: color.clone(),
					large_triangle1: color.clone(),
					large_triangle2: color,
				}
			}
			LogoColorScheme::Multi => ShapesColors {
				trapezoid: ui::colors::PINK.to_owned(),
				square: ui::colors::YELLOW.to_owned(),
				medium_triangle: ui::colors::TEAL.to_owned(),
				small_triangle1: ui::colors::PURPLE.to_owned(),
				small_triangle2: ui::colors::INDIGO.to_owned(),
				large_triangle1: ui::colors::BLUE.to_owned(),
				large_triangle2: ui::colors::GREEN.to_owned(),
			},
		};
		svg()
			.class(self.class)
			.attribute("width", "100%")
			.attribute("height", "100%")
			.attribute("fill", "none")
			.attribute("viewBox", "0 0 200 200")
			.child(svg::desc().child("tangram"))
			.child(
				svg::path()
					.attribute("fill", shapes_colors.trapezoid)
					.attribute("d", "M0 0L50 50V150L0 100V0Z"),
			)
			.child(
				svg::path()
					.attribute("fill", shapes_colors.square)
					.attribute("d", "M100 100L150 150L100 200L50 150L100 100Z"),
			)
			.child(
				svg::path()
					.attribute("fill", shapes_colors.medium_triangle)
					.attribute("d", "M0 100L100 200H0V100Z"),
			)
			.child(
				svg::path()
					.attribute("fill", shapes_colors.small_triangle2)
					.attribute("d", "M150 150L200 200H100L150 150Z"),
			)
			.child(
				svg::path()
					.attribute("fill", shapes_colors.small_triangle1)
					.attribute("d", "M50 50L100 100L50 150V50Z"),
			)
			.child(
				svg::path()
					.attribute("fill", shapes_colors.large_triangle2)
					.attribute("d", "M200 0V200L100 100L200 0Z"),
			)
			.child(
				svg::path()
					.attribute("fill", shapes_colors.large_triangle1)
					.attribute("d", "M200 0L100 100L0 0H200Z"),
			)
			.into_node()
	}
}
