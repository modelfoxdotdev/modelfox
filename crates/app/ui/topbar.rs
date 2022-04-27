use crate::logo::Logo;
use modelfox_ui as ui;
use pinwheel::prelude::*;

#[derive(builder, Default, new)]
#[new(default)]
pub struct Topbar {
	#[builder]
	pub topbar_avatar: Option<TopbarAvatar>,
}

pub struct TopbarAvatar {
	pub avatar_url: Option<String>,
}

impl Component for Topbar {
	fn into_node(self) -> Node {
		let items = if let Some(topbar_avatar) = self.topbar_avatar {
			let avatar_item = ui::TopbarItem {
				element: Some(
					ui::Link::new()
						.href("/user".to_owned())
						.child(ui::Avatar::new().src(topbar_avatar.avatar_url))
						.into_node(),
				),
				href: "/user".to_owned(),
				title: "Settings".to_owned(),
			};
			Some(vec![avatar_item])
		} else {
			None
		};
		let logo = Some(Logo::new().into_node());
		ui::Topbar::new()
			.background_color(ui::colors::HEADER.to_owned())
			.dropdown_background_color(ui::colors::SURFACE.to_owned())
			.items(items)
			.logo_href("/".to_owned())
			.logo(logo)
			.title("ModelFox".to_owned())
			.into_node()
	}
}
