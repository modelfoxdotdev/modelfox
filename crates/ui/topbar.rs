use crate as ui;
use pinwheel::prelude::*;

#[derive(builder, Default, new)]
#[new(default)]
pub struct Topbar {
	#[builder]
	pub background_color: Option<String>,
	#[builder]
	pub dropdown_background_color: Option<String>,
	#[builder]
	pub items: Option<Vec<TopbarItem>>,
	#[builder]
	pub logo_href: Option<String>,
	#[builder]
	pub logo_img_url: Option<String>,
	#[builder]
	pub logo: Option<Node>,
	#[builder]
	pub title: Option<String>,
}

pub struct TopbarItem {
	pub element: Option<Node>,
	pub href: String,
	pub title: String,
}

impl Component for Topbar {
	fn into_node(self) -> Node {
		let dropdown = self.items.as_ref().map(|items| {
			let items: Vec<TopbarDropdownItem> = items
				.iter()
				.map(|item| TopbarDropdownItem {
					href: item.href.clone(),
					title: item.title.clone(),
				})
				.collect();
			details()
				.class("topbar-details")
				.child(summary().child(TopbarHamburger))
				.child(
					TopbarDropdown::new(items)
						.background_color(self.dropdown_background_color.clone()),
				)
		});
		let items = self.items.map(|items| {
			TopbarItemsWrapper::new().children(items.into_iter().map(|item| {
				if let Some(element) = item.element {
					element
				} else {
					a().class("topbar-link")
						.attribute("href", item.href)
						.child(item.title)
						.into_node()
				}
			}))
		});
		div()
			.class("topbar-wrapper")
			.child(TopbarBrand {
				logo_element: self.logo,
				logo_href: self.logo_href,
				logo_img_url: self.logo_img_url,
				title: self.title,
			})
			.child(items)
			.child(dropdown)
			.into_node()
	}
}

pub struct TopbarBrand {
	logo_element: Option<Node>,
	logo_href: Option<String>,
	logo_img_url: Option<String>,
	title: Option<String>,
}

impl Component for TopbarBrand {
	fn into_node(self) -> Node {
		let logo = if let Some(logo_img_url) = self.logo_img_url {
			img()
				.class("topbar-brand-img")
				.attribute("srcset", format!("{} 3x", logo_img_url))
				.into_node()
		} else {
			div()
				.class("topbar-brand-svg")
				.child(self.logo_element)
				.into_node()
		};
		a().class("topbar-link")
			.attribute("href", self.logo_href.unwrap_or_else(|| "/".to_owned()))
			.child(
				div().class("topbar-brand-wrapper").child(logo).child(
					self.title
						.map(|title| div().class("topbar-brand-title").child(title)),
				),
			)
			.into_node()
	}
}

#[derive(children, Default, new)]
#[new(default)]
struct TopbarItemsWrapper {
	pub children: Vec<Node>,
}

impl Component for TopbarItemsWrapper {
	fn into_node(self) -> Node {
		nav()
			.class("topbar-items-wrapper")
			.child(self.children)
			.into_node()
	}
}

struct TopbarHamburger;

impl Component for TopbarHamburger {
	fn into_node(self) -> Node {
		div()
			.class("topbar-hamburger")
			.child(
				svg()
					.class("topbar-hamburger-icon")
					.attribute("height", "15px")
					.attribute("overflow", "visible")
					.attribute("viewBox", "0 0 1 1")
					.attribute("width", "15px")
					.children({
						[0.0, 0.5, 1.0].iter().map(|y| {
							svg::line()
								.attribute("stroke", "currentColor")
								.attribute("stroke-linecap", "round")
								.attribute("stroke-width", "0.2")
								.attribute("x1", "0")
								.attribute("x2", "1")
								.attribute("y1", y.to_string())
								.attribute("y2", y.to_string())
						})
					}),
			)
			.child(
				svg()
					.class("topbar-x-icon")
					.attribute("height", "15px")
					.attribute("overflow", "visible")
					.attribute("viewBox", "0 0 1 1")
					.attribute("width", "15px")
					.child(
						svg::line()
							.attribute("stroke", "currentColor")
							.attribute("stroke-linecap", "round")
							.attribute("stroke-width", "0.2")
							.attribute("x1", "0")
							.attribute("x2", "1")
							.attribute("y1", "0")
							.attribute("y2", "1"),
					)
					.child(
						svg::line()
							.attribute("stroke", "currentColor")
							.attribute("stroke-linecap", "round")
							.attribute("stroke-width", "0.2")
							.attribute("x1", "1")
							.attribute("x2", "0")
							.attribute("y1", "0")
							.attribute("y2", "1"),
					),
			)
			.into_node()
	}
}

#[derive(builder, new)]
pub struct TopbarDropdown {
	items: Vec<TopbarDropdownItem>,
	#[builder]
	#[new(default)]
	background_color: Option<String>,
	#[builder]
	#[new(default)]
	cta: Option<TopbarItem>,
}

pub struct TopbarDropdownItem {
	title: String,
	href: String,
}

impl Component for TopbarDropdown {
	fn into_node(self) -> Node {
		div()
			.class("topbar-dropdown-wrapper")
			.style(style::BACKGROUND_COLOR, self.background_color)
			.children(self.items.into_iter().map(|item| {
				a().class("topbar-dropdown-link")
					.attribute("href", item.href)
					.child(div().class("topbar-dropdown-item").child(item.title))
			}))
			.child(self.cta.map(|cta| {
				div()
					.class("topbar-dropdown-item")
					.child(ui::Button::new().href(cta.href))
					.child(cta.title)
			}))
			.into_node()
	}
}
