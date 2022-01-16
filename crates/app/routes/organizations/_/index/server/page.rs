use pinwheel::prelude::*;
use tangram_app_layouts::{
	app_layout::{AppLayout, AppLayoutInfo},
	document::Document,
};
use tangram_id::Id;
use tangram_ui as ui;

pub struct Page {
	pub app_layout_info: AppLayoutInfo,
	pub details_section: DetailsSection,
	pub id: String,
	pub members_section: MembersSection,
	pub name: String,
	pub repos_section: ReposSection,
	pub can_delete: bool,
}

impl Component for Page {
	fn into_node(self) -> Node {
		Document::new()
			.child(
				AppLayout::new(self.app_layout_info).child(
					ui::S1::new()
						.child(ui::H1::new(self.name))
						.child(self.details_section)
						.child(self.members_section)
						.child(self.repos_section)
						.child(if self.can_delete {
							Some(DangerZoneSection)
						} else {
							None
						}),
				),
			)
			.into_node()
	}
}

pub struct DetailsSection {
	pub organization_id: String,
	pub organization_name: String,
	pub can_edit: bool,
}

impl Component for DetailsSection {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(
				ui::SpaceBetween::new().child(ui::H2::new("Details")).child(
					ui::Button::new()
						.href(format!("/organizations/{}/edit", self.organization_id))
						.color(ui::colors::GRAY.to_owned())
						.disabled(false)
						.child("Edit"),
				),
			)
			.child(
				ui::TextField::new()
					.disabled(true)
					.value(self.organization_name)
					.label("Organization Name".to_owned())
					.readonly(true),
			)
			.into_node()
	}
}

pub struct MembersSection {
	pub organization_id: Id,
	pub members_table: MembersTable,
}

impl Component for MembersSection {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(
				ui::SpaceBetween::new().child(ui::H2::new("Members")).child(
					ui::Button::new()
						.href(format!(
							"/organizations/{}/members/new",
							self.organization_id,
						))
						.child("Invite Team Member"),
				),
			)
			.child(self.members_table)
			.into_node()
	}
}

pub struct MembersTable {
	pub user_id: String,
	pub can_edit: bool,
	pub rows: Vec<MembersTableRow>,
}

pub struct MembersTableRow {
	pub id: Id,
	pub email: String,
	pub is_admin: bool,
}

impl Component for MembersTable {
	fn into_node(self) -> Node {
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Email"))
						.child(ui::TableHeaderCell::new().child("Role")),
				),
			)
			.child(ui::TableBody::new().children(self.rows.iter().map(|row| {
				let member_cell = if self.can_edit {
					ui::Link::new()
						.href(format!("members/{}", row.id))
						.child(row.email.clone())
						.into_node()
				} else {
					row.email.clone().into_node()
				};
				ui::TableRow::new()
					.child(ui::TableCell::new().child(member_cell))
					.child(ui::TableCell::new().child(if row.is_admin {
						"Admin"
					} else {
						"Member"
					}))
			})))
			.into_node()
	}
}

pub struct MemberDeleteForm {
	member_id: String,
}

impl Component for MemberDeleteForm {
	fn into_node(self) -> Node {
		ui::Form::new()
			.post(true)
			.child(
				input()
					.attribute("name", "action")
					.attribute("type", "hidden")
					.attribute("value", "delete_member"),
			)
			.child(
				input()
					.attribute("name", "member_id")
					.attribute("type", "hidden")
					.attribute("value", self.member_id),
			)
			.child(
				ui::Button::new()
					.button_type(ui::ButtonType::Submit)
					.color(ui::colors::RED.to_owned())
					.child("Remove"),
			)
			.into_node()
	}
}

pub struct ReposSection {
	pub repos_table: Option<ReposTable>,
}

impl Component for ReposSection {
	fn into_node(self) -> Node {
		let repos_table_or_empty_message = self
			.repos_table
			.map(|repos_table| repos_table.into_node())
			.unwrap_or_else(|| {
				ui::Card::new()
					.child(ui::P::new().child("This organization does not have any repos."))
					.into_node()
			});
		ui::S2::new()
			.child(
				ui::SpaceBetween::new().child(ui::H2::new("Repos")).child(
					ui::Button::new()
						.href("/repos/new".to_owned())
						.child("Create New Repo"),
				),
			)
			.child(repos_table_or_empty_message)
			.into_node()
	}
}

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
					ui::TableRow::new().child(
						ui::TableCell::new().child(
							ui::Link::new()
								.href(format!("/repos/{}/", row.id))
								.child(row.title),
						),
					)
				})),
			)
			.into_node()
	}
}

struct DangerZoneSection;

impl Component for DangerZoneSection {
	fn into_node(self) -> Node {
		ui::S2::new()
			.child(ui::H2::new("Danger Zone"))
			.child(
				ui::Form::new()
					.post(true)
					.onsubmit("return confirm(\"Are you sure?\")".to_owned())
					.child(
						input()
							.attribute("name", "action")
							.attribute("type", "hidden")
							.attribute("value", "delete_organization"),
					)
					.child(
						ui::Button::new()
							.button_type(ui::ButtonType::Submit)
							.color(ui::colors::RED.to_owned())
							.child("Delete Organization"),
					),
			)
			.into_node()
	}
}
