use anyhow::Result;
use pinwheel::prelude::*;
use sqlx::prelude::*;
use std::sync::Arc;
use tangram_app_common::{
	error::{bad_request, not_found, redirect_to_login, service_unavailable},
	repos::get_model_version_ids,
	user::{authorize_user, authorize_user_for_model},
	Context,
};
use tangram_app_ui::topbar::{Topbar, TopbarAvatar};
use tangram_id::Id;
use tangram_ui as ui;

pub struct ModelLayoutInfo {
	pub model_id: Id,
	pub model_tag: Option<String>,
	pub model_version_ids: Vec<Id>,
	pub owner: Option<Owner>,
	pub repo_id: String,
	pub repo_title: String,
	pub selected_item: ModelNavItem,
	pub topbar_avatar: Option<TopbarAvatar>,
}

pub enum Owner {
	User { id: Id, email: String },
	Organization { id: Id, name: String },
}

#[derive(children)]
pub struct ModelLayout {
	pub info: ModelLayoutInfo,
	pub children: Vec<Node>,
}

impl ModelLayout {
	pub fn new(info: ModelLayoutInfo) -> ModelLayout {
		ModelLayout {
			info,
			children: Vec::new(),
		}
	}
}

pub async fn model_layout_info(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	context: &Context,
	model_id: Id,
	selected_item: ModelNavItem,
) -> Result<ModelLayoutInfo> {
	let row = sqlx::query(
		"
			select
				repos.id,
				repos.title,
				models.tag,
				repos.user_id,
				users.email,
				repos.organization_id,
				organizations.name
			from repos
			join models
				on models.repo_id = repos.id
			left join users
				on users.id = repos.user_id
			left join organizations
				on organizations.id = repos.organization_id
			where models.id = $1
		",
	)
	.bind(&model_id.to_string())
	.fetch_one(&mut *db)
	.await?;
	let repo_id: String = row.get(0);
	let repo_id: Id = repo_id.parse()?;
	let repo_title: String = row.get(1);
	let model_tag: Option<String> = row.get(2);
	let owner_user_id: Option<String> = row.get(3);
	let owner_user_email: Option<String> = row.get(4);
	let owner_organization_id: Option<String> = row.get(5);
	let owner_organization_name: Option<String> = row.get(6);
	let model_version_ids = get_model_version_ids(db, repo_id).await?;
	#[allow(clippy::manual_map)]
	let owner = if let Some(owner_user_id) = owner_user_id {
		Some(Owner::User {
			id: owner_user_id.parse().unwrap(),
			email: owner_user_email.unwrap(),
		})
	} else if let Some(owner_organization_id) = owner_organization_id {
		Some(Owner::Organization {
			id: owner_organization_id.parse().unwrap(),
			name: owner_organization_name.unwrap(),
		})
	} else {
		None
	};
	let topbar_avatar = if context.options.auth_enabled() {
		Some(TopbarAvatar { avatar_url: None })
	} else {
		None
	};
	Ok(ModelLayoutInfo {
		model_id,
		model_tag,
		model_version_ids,
		owner,
		repo_id: repo_id.to_string(),
		repo_title,
		topbar_avatar,
		selected_item,
	})
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action")]
enum Action {
	#[serde(rename = "delete_model")]
	DeleteModel,
	#[serde(rename = "download_model")]
	DownloadModel,
}

pub async fn post(
	request: &mut http::Request<hyper::Body>,
	model_id: &str,
) -> Result<http::Response<hyper::Body>> {
	let context = request.extensions().get::<Arc<Context>>().unwrap();
	let mut db = match context.database_pool.begin().await {
		Ok(db) => db,
		Err(_) => return Ok(service_unavailable()),
	};
	let user = match authorize_user(request, &mut db, context.options.auth_enabled()).await? {
		Ok(user) => user,
		Err(_) => return Ok(redirect_to_login()),
	};
	let model_id: Id = match model_id.parse() {
		Ok(model_id) => model_id,
		Err(_) => return Ok(bad_request()),
	};
	if !authorize_user_for_model(&mut db, &user, model_id).await? {
		return Ok(not_found());
	}
	let response = delete_model(&mut db, model_id).await?;
	db.commit().await?;
	Ok(response)
}

async fn delete_model(
	db: &mut sqlx::Transaction<'_, sqlx::Any>,
	model_id: Id,
) -> Result<http::Response<hyper::Body>> {
	sqlx::query(
		"
		delete from models
		where
			models.id = $1
	",
	)
	.bind(&model_id.to_string())
	.execute(&mut *db)
	.await?;
	let response = http::Response::builder()
		.status(http::StatusCode::SEE_OTHER)
		.header(http::header::LOCATION, "/")
		.body(hyper::Body::empty())
		.unwrap();
	Ok(response)
}

#[derive(PartialEq)]
pub enum ModelNavItem {
	Overview,
	TrainingGrid,
	TrainingStats,
	TrainingMetrics,
	Playground,
	Tuning,
	ProductionPredictions,
	ProductionStats,
	ProductionMetrics,
}

impl Component for ModelLayout {
	fn into_node(self) -> Node {
		let selected_model_version_id = self
			.info
			.model_version_ids
			.iter()
			.find(|model_version_id| *model_version_id == &self.info.model_id)
			.unwrap();
		let topbar_avatar = self
			.info
			.topbar_avatar
			.as_ref()
			.map(|topbar_avatar| TopbarAvatar {
				avatar_url: topbar_avatar.avatar_url.to_owned(),
			});
		let topbar = div()
			.style(style::GRID_AREA, "topbar")
			.child(Topbar::new().topbar_avatar(topbar_avatar));
		let top = ModelLayoutTop {
			model_id: self.info.model_id,
			model_tag: self.info.model_tag,
			model_version_ids: self.info.model_version_ids.clone(),
			owner: self.info.owner,
			repo_id: self.info.repo_id.clone(),
			repo_title: self.info.repo_title,
			selected_model_version_id: selected_model_version_id.to_string(),
		};
		let left = div().class("model-layout-left").child(ModelNav {
			repo_id: self.info.repo_id.to_string(),
			model_id: self.info.model_id.to_string(),
			selected_item: self.info.selected_item,
		});
		let center = div().class("model-layout-center").child(self.children);
		let right = div().class("model-layout-right");
		div()
			.class("model-layout")
			.child(topbar)
			.child(top)
			.child(left)
			.child(center)
			.child(right)
			.into_node()
	}
}

pub struct ModelLayoutTop {
	pub model_id: Id,
	pub model_tag: Option<String>,
	pub model_version_ids: Vec<Id>,
	pub owner: Option<Owner>,
	pub repo_id: String,
	pub repo_title: String,
	pub selected_model_version_id: String,
}

impl Component for ModelLayoutTop {
	fn into_node(self) -> Node {
		let repo_id = self.repo_id;
		let model_id = self.model_id;
		let model_heading = self.model_tag.unwrap_or_else(|| model_id.to_string());
		struct OwnerInfo {
			title: String,
			href: String,
		}
		let owner_info = self.owner.map(|owner| match owner {
			Owner::User { email, .. } => OwnerInfo {
				title: email,
				href: "/user".to_owned(),
			},
			Owner::Organization { name, id } => OwnerInfo {
				title: name,
				href: format!("/organizations/{}/", id),
			},
		});
		let owner_segment = owner_info.map(|owner_info| {
			a().class("model-layout-top-title-segment")
				.attribute("href", owner_info.href)
				.attribute("title", "owner")
				.child(owner_info.title)
		});
		let owner_slash = owner_segment
			.as_ref()
			.map(|_| span().class("model-layout-top-title-slash").child("/"));
		let repo_segment = a()
			.class("model-layout-top-title-segment")
			.attribute("title", "repo")
			.href(format!("/repos/{}/", repo_id))
			.child(self.repo_title.clone());
		let repo_slash = span().class("model-layout-top-title-slash").child("/");
		let model_segment = a()
			.class("model-layout-top-title-segment")
			.attribute("title", "repo")
			.href(format!(
				"/repos/{}/models/{}/",
				repo_id,
				self.model_id.to_string()
			))
			.child(model_heading);
		let title = div()
			.class("model-layout-top-title-wrapper")
			.child(owner_segment)
			.child(owner_slash)
			.child(repo_segment)
			.child(repo_slash)
			.child(model_segment);
		let buttons = div()
			.class("model-layout-top-buttons-wrapper")
			.child(
				ui::Button::new()
					.color(ui::colors::GRAY.to_owned())
					.href(format!("/repos/{}/models/{}/edit", repo_id, self.model_id))
					.child("Edit"),
			)
			.child(
				ui::Button::new()
					.href(format!(
						"/repos/{}/models/{}/download",
						repo_id, self.model_id
					))
					.download(format!("{}.tangram", self.repo_title))
					.child("Download"),
			);
		div()
			.class("model-layout-top")
			.child(title)
			.child(buttons)
			.into_node()
	}
}

pub struct ModelNav {
	repo_id: String,
	model_id: String,
	selected_item: ModelNavItem,
}

impl Component for ModelNav {
	fn into_node(self) -> Node {
		let overview = ui::NavSection::new("Overview".to_owned()).child(
			ui::NavItem::new()
				.title("Overview".to_owned())
				.href(format!("/repos/{}/models/{}/", self.repo_id, self.model_id))
				.selected(self.selected_item == ModelNavItem::Overview),
		);
		let training = ui::NavSection::new("Training".to_owned())
			.child(
				ui::NavItem::new()
					.title("Grid".to_owned())
					.href(format!(
						"/repos/{}/models/{}/training_grid/",
						self.repo_id, self.model_id
					))
					.selected(self.selected_item == ModelNavItem::TrainingGrid),
			)
			.child(
				ui::NavItem::new()
					.title("Stats".to_owned())
					.href(format!(
						"/repos/{}/models/{}/training_stats/",
						self.repo_id, self.model_id
					))
					.selected(self.selected_item == ModelNavItem::TrainingStats),
			)
			.child(
				ui::NavItem::new()
					.title("Metrics".to_owned())
					.href(format!(
						"/repos/{}/models/{}/training_metrics/",
						self.repo_id, self.model_id
					))
					.selected(self.selected_item == ModelNavItem::TrainingMetrics),
			);
		let playground = ui::NavSection::new("Playground".to_owned()).child(
			ui::NavItem::new()
				.title("Playground".to_owned())
				.href(format!(
					"/repos/{}/models/{}/playground",
					self.repo_id, self.model_id
				))
				.selected(self.selected_item == ModelNavItem::Playground),
		);
		let tuning = ui::NavSection::new("Tuning".to_owned()).child(
			ui::NavItem::new()
				.title("Tuning".to_owned())
				.href(format!(
					"/repos/{}/models/{}/tuning",
					self.repo_id, self.model_id
				))
				.selected(self.selected_item == ModelNavItem::Tuning),
		);
		let production = ui::NavSection::new("Production".to_owned())
			.child(
				ui::NavItem::new()
					.title("Predictions".to_owned())
					.href(format!(
						"/repos/{}/models/{}/production_predictions/",
						self.repo_id, self.model_id
					))
					.selected(self.selected_item == ModelNavItem::ProductionPredictions),
			)
			.child(
				ui::NavItem::new()
					.title("Stats".to_owned())
					.href(format!(
						"/repos/{}/models/{}/production_stats/",
						self.repo_id, self.model_id
					))
					.selected(self.selected_item == ModelNavItem::ProductionStats),
			)
			.child(
				ui::NavItem::new()
					.title("Metrics".to_owned())
					.href(format!(
						"/repos/{}/models/{}/production_metrics/",
						self.repo_id, self.model_id
					))
					.selected(self.selected_item == ModelNavItem::ProductionMetrics),
			);
		ui::Nav::new()
			.title("Pages".to_owned())
			.child(overview)
			.child(training)
			.child(playground)
			.child(tuning)
			.child(production)
			.into_node()
	}
}
