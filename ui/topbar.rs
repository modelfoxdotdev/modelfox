use super::Button;
use html::{component, html, style, Props};

#[derive(Props)]
pub struct TopbarProps {
	pub background_color: String,
	pub dropdown_background_color: String,
	pub items: Option<Vec<TopbarItem>>,
	pub logo: Option<html::Node>,
	pub logo_href: Option<String>,
	pub logo_img_url: Option<String>,
	pub title: Option<String>,
}

pub struct TopbarItem {
	pub element: Option<html::Node>,
	pub href: String,
	pub title: String,
}

#[component]
pub fn Topbar(props: TopbarProps) {
	let dropdown_background_color = &props.dropdown_background_color;
	let dropdown = props.items.as_ref().map(|items| {
		let items: Vec<TopbarDropdownItem> = items
			.iter()
			.map(|item| TopbarDropdownItem {
				href: item.href.clone(),
				title: item.title.clone(),
			})
			.collect();
		html! {
			<details class="topbar-details">
				<summary>
					<TopbarHamburger />
				</summary>
				<TopbarDropdown
					background_color={dropdown_background_color.to_owned()}
					cta={None}
					items={items}
				/>
			</details>
		}
	});
	let items = props.items.map(|items| {
		html! {
			<TopbarItemsWrapper>
				{items.into_iter().map(|item| {
					if let Some(element) = item.element {
						element
					} else {
						html! {
							<a class="topbar-link" href={item.href}>
								{item.title}
							</a>
						}
					}
				}).collect::<Vec<_>>()}
			</TopbarItemsWrapper>
		}
	});
	let wrapper_style = style! {
		"background-color" => props.background_color,
	};
	html! {
		<div class="topbar-wrapper" style={wrapper_style}>
			<TopbarBrand
				logo_element={props.logo}
				logo_href={props.logo_href}
				logo_img_url={props.logo_img_url}
				title={props.title}
			/>
			{items}
			{dropdown}
		</div>
	}
}

#[derive(Props)]
pub struct TopbarBrandProps {
	logo_element: Option<html::Node>,
	logo_href: Option<String>,
	logo_img_url: Option<String>,
	title: Option<String>,
}

#[component]
fn TopbarBrand(props: TopbarBrandProps) {
	html! {
		<a class="topbar-link" href={props.logo_href.unwrap_or_else(|| "/".to_owned())}>
			<div class="topbar-brand-wrapper">
				{if let Some(logo_img_url) = props.logo_img_url {
					html! {
						<img class="topbar-brand-img" srcset={format!("{} 3x", logo_img_url)} />
					}
				} else {
					html! {
						<div class="topbar-brand-svg">{props.logo_element}</div>
					}
				}}
				{props.title.map(|title| html! {
					<div class="topbar-brand-title">
						{title}
					</div>
				})}
			</div>
		</a>
	}
}

#[component]
fn TopbarItemsWrapper() {
	html! { <nav class="topbar-items-wrapper">{children}</nav> }
}

#[component]
fn TopbarHamburger() {
	html! {
		<div class="topbar-hamburger">
			<svg
				class="topbar-hamburger-icon"
				height="15px"
				overflow="visible"
				viewBox="0 0 1 1"
				width="15px"
			>
				{[0.0, 0.5, 1.0].iter().map(|y| html! {
					<line
						stroke="currentColor"
						stroke-linecap="round"
						stroke-width="0.2"
						x1="0"
						x2="1"
						y1={y.to_string()}
						y2={y.to_string()}
					/>
				}).collect::<Vec<_>>()}
			</svg>
			<svg
				class="topbar-x-icon"
				height="15px"
				overflow="visible"
				viewBox="0 0 1 1"
				width="15px"
			>
				<line
					stroke="currentColor"
					stroke-linecap="round"
					stroke-width="0.2"
					x1="0"
					x2="1"
					y1="0"
					y2="1"
				/>
				<line
					stroke="currentColor"
					stroke-linecap="round"
					stroke-width="0.2"
					x1="1"
					x2="0"
					y1="0"
					y2="1"
				/>
			</svg>
		</div>
	}
}

#[derive(Props)]
pub struct TopbarDropdownProps {
	background_color: String,
	cta: Option<TopbarItem>,
	items: Vec<TopbarDropdownItem>,
}

pub struct TopbarDropdownItem {
	title: String,
	href: String,
}

#[component]
fn TopbarDropdown(props: TopbarDropdownProps) {
	let wrapper_style = style! {
		"background-color" => props.background_color,
	};
	html! {
		<div class="topbar-dropdown-wrapper" style={wrapper_style}>
			{props.items.into_iter().map(|item| html! {
				<a class="topbar-dropdown-link" href={item.href}>
					<div class="topbar-dropdown-item">
						{item.title}
					</div>
				</a>
			}).collect::<Vec<_>>()}
			{props.cta.map(|cta| html! {
				<div class="topbar-dropdown-item">
					<Button href?={Some(cta.href)}>
						{cta.title}
					</Button>
				</div>
			})}
		</div>
	}
}
