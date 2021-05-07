use html::{component, html, Props};
use tangram_app_common::{
	topbar::{Topbar, TopbarAvatar},
	Context,
};
use tangram_error::Result;

#[derive(Props)]
pub struct AppLayoutProps {
	pub topbar_avatar: Option<TopbarAvatar>,
}

#[component]
pub fn AppLayout(props: AppLayoutProps) {
	let topbar_avatar = props.topbar_avatar.map(|topbar_avatar| TopbarAvatar {
		avatar_url: topbar_avatar.avatar_url,
	});
	html! {
		<div class="app-layout-topbar-grid">
			<Topbar topbar_avatar={topbar_avatar} />
			<div class="app-layout">{children}</div>
		</div>
	}
}

pub async fn get_app_layout_props(context: &Context) -> Result<AppLayoutProps> {
	let topbar_avatar = if context.options.auth_enabled() {
		Some(TopbarAvatar { avatar_url: None })
	} else {
		None
	};
	Ok(AppLayoutProps { topbar_avatar })
}
