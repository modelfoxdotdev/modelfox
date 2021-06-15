use pinwheel::prelude::*;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use tangram_app_ui::page_heading::{PageHeading, PageHeadingButtons};
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct Page {
	pub app_layout_info: AppLayoutInfo,
	pub repos_table: Option<ReposTable>,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let heading = PageHeading::new()
			.child(ui::H1::new().child("Repositories"))
			.child(
				PageHeadingButtons::new().child(
					ui::Button::new()
						.href("/repos/new".to_owned())
						.child("Create Repo"),
				),
			);
		let body = if let Some(repos_table) = self.repos_table {
			repos_table.into_node()
		} else {
			ui::Card::new()
				.child(ui::P::new().child("You do not have any repositories."))
				.into_node()
		};
		let content = ui::S1::new().child(heading).child(body);
		let layout = AppLayout::new(self.app_layout_info).child(content);
		Document::new().child(layout).into_node()
	}
}

pub struct ReposTable {
	pub rows: Vec<ReposTableRow>,
}

pub struct ReposTableRow {
	pub id: String,
	pub title: String,
	pub owner_name: Option<String>,
}

impl Component for ReposTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new().child(
						ui::TableHeaderCell::new()
							.width("100%".to_owned())
							.child("Name"),
					),
				),
			)
			.child(
				ui::TableBody::new().children(self.rows.into_iter().map(|repo| {
					let href = format!("/repos/{}/", repo.id);
					let owner_slash = repo.owner_name.map(|owner_name| format!("{}/", owner_name));
					let link_text =
						format!("{}{}", owner_slash.as_deref().unwrap_or(""), repo.title,);
					ui::TableRow::new().child(
						ui::TableCell::new().child(ui::Link::new().href(href).child(link_text)),
					)
				})),
			)
			.into_node()
	}
}
