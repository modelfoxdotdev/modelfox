use pinwheel::prelude::*;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use tangram_ui as ui;

#[derive(ComponentBuilder)]
pub struct Page {
	pub app_layout_info: AppLayoutInfo,
	pub inner: Inner,
}

pub enum Inner {
	Auth(Auth),
	NoAuth(NoAuth),
}

#[derive(ComponentBuilder)]
pub struct Auth {
	pub details_section: DetailsSection,
	pub organizations_section: OrganizationsSection,
	pub repos_section: ReposSection,
}

#[derive(ComponentBuilder)]
pub struct NoAuth {
	pub repos_section: ReposSection,
}

impl Component for Page {
	fn into_node(self) -> Node {
		let inner = match self.inner {
			Inner::Auth(inner) => inner.into_node(),
			Inner::NoAuth(inner) => inner.into_node(),
		};
		Document::new()
			.child(AppLayout::new(self.app_layout_info).child(inner))
			.into_node()
	}
}

impl Component for NoAuth {
	fn into_node(self) -> Node {
		let text = "You are using the free version of tangram that does not support user accounts or organizations. Checkout out the different plans that allow you to collaborate with your team.";
		ui::S1::new().child(ui::P::new().child(text)).into_node()
	}
}

impl Component for Auth {
	fn into_node(self) -> Node {
		let Auth {
			organizations_section,
			repos_section,
			details_section,
		} = self;
		ui::S1::new()
			.child(Header)
			.child(details_section)
			.child(organizations_section)
			.child(repos_section)
			.into_node()
	}
}

struct Header;

impl Component for Header {
	fn into_node(self) -> Node {
		ui::SpaceBetween::new()
			.child(ui::H1::new().child("User"))
			.child(
				ui::Form::new()
					.post(Some(true))
					.child(
						input()
							.attribute("name", "action")
							.attribute("type", "hidden")
							.attribute("value", "logout"),
					)
					.child(
						ui::Button::new()
							.color(ui::colors::RED.to_owned())
							.button_type(Some(ui::ButtonType::Submit))
							.child("Logout"),
					),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct DetailsSection {
	pub email: String,
}

impl Component for DetailsSection {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(
				ui::Form::new().child(
					ui::TextField::new()
						.label("Email".to_owned())
						.readonly(Some(true))
						.value(Some(self.email)),
				),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ReposSection {
	pub repos_table: Option<ReposTable>,
}

impl Component for ReposSection {
	fn into_node(self) -> Node {
		let repos_table_or_empty_message = if let Some(repos_table) = self.repos_table {
			repos_table.into_node()
		} else {
			ui::Card::new()
				.child(ui::P::new().child("You do not have any repos."))
				.into_node()
		};
		ui::S2::new()
			.child(
				ui::SpaceBetween::new()
					.child(ui::H2::new().child("User Repos"))
					.child(
						ui::Button::new()
							.href("/repos/new".to_owned())
							.id(None)
							.child("Create New Repo"),
					),
			)
			.child(repos_table_or_empty_message)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct ReposTable {
	pub rows: Vec<ReposTableRow>,
}

pub struct ReposTableRow {
	pub id: String,
	pub title: String,
}

impl Component for ReposTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new().child(ui::TableHeaderCell::new().child("Repo Title")),
				),
			)
			.child(
				ui::TableBody::new().children(self.rows.into_iter().map(|row| {
					let href = format!("/repos/{}/", row.id);
					{
						ui::TableRow::new().child(
							ui::TableCell::new().child(ui::Link::new().href(href).child(row.title)),
						)
					}
				})),
			)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct OrganizationsSection {
	pub organizations_table: Option<OrganizationsTable>,
}

impl Component for OrganizationsSection {
	fn into_node(self) -> Node {
		let organizations_table_or_empty_message =
			if let Some(organizations_table) = self.organizations_table {
				organizations_table.into_node()
			} else {
				ui::Card::new()
					.child(ui::P::new().child("You do not have any organizations."))
					.into_node()
			};
		ui::S2::new()
			.child(
				ui::SpaceBetween::new()
					.child(ui::H2::new().child("Organizations"))
					.child(
						ui::Button::new()
							.href("/organizations/new".to_owned())
							.child("Create New Organization"),
					),
			)
			.child(organizations_table_or_empty_message)
			.into_node()
	}
}

#[derive(ComponentBuilder)]
pub struct OrganizationsTable {
	pub rows: Vec<OrganizationsTableRow>,
}

pub struct OrganizationsTableRow {
	pub id: String,
	pub name: String,
}

impl Component for OrganizationsTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(ui::TableHeader::new().child(
				ui::TableRow::new().child(ui::TableHeaderCell::new().child("Organization Name")),
			))
			.child(ui::TableBody::new().children(self.rows.iter().map(|row| {
				let href = format!("/organizations/{}/", row.id);
				{
					ui::TableRow::new().child(
						ui::TableCell::new()
							.child(ui::Link::new().href(href).child(row.name.clone())),
					)
				}
			})))
			.into_node()
	}
}
