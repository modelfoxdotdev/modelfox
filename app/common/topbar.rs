use super::logo::{Logo, LogoScheme};
use html::{component, html, Props};
use tangram_ui as ui;

#[derive(Props)]
pub struct TopbarProps {
	pub topbar_avatar: Option<TopbarAvatar>,
}

pub struct TopbarAvatar {
	pub avatar_url: Option<String>,
}

#[component]
pub fn Topbar(props: TopbarProps) {
	let items = if let Some(topbar_avatar) = props.topbar_avatar {
		let avatar_item = ui::TopbarItem {
			element: Some(html! {
				<ui::Link href="/user">
					<ui::Avatar src={topbar_avatar.avatar_url} />
				</ui::Link>
			}),
			href: "/user".to_owned(),
			title: "Settings".to_owned(),
		};
		Some(vec![avatar_item])
	} else {
		None
	};
	let logo = Some(html! {
		<Logo color_scheme={LogoScheme::Multi} />
	});
	html! {
		<ui::Topbar
			background_color="var(--header-color)"
			dropdown_background_color="var(--surface-color)"
			items={items}
			logo_href="/"
			logo_img_url={None}
			logo={logo}
			title="tangram"
		/>
	}
}
