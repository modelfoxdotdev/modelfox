use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_layouts::{document::Document, page_layout::PageLayout};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let title = ui::H1::new().center(true).child("Pricing");
		let subtitle = ui::H2::new()
			.center(true)
			.child("Contact us at ")
			.child(
				ui::Link::new()
					.href("mailto:hello@tangram.dev".to_owned())
					.child("hello@tangram.dev"),
			)
			.child(".");
		let free_cta = ui::Button::new()
			.color(ui::colors::RED.to_owned())
			.href("/docs/install".to_owned())
			.child("Install the CLI")
			.into_node();
		let team_cta = ui::Button::new()
			.color(ui::colors::GREEN.to_owned())
			.href("mailto:hello@tangram.dev".to_owned())
			.child("Contact Us")
			.into_node();
		let enterprise_cta = ui::Button::new()
			.color(ui::colors::BLUE.to_owned())
			.href("mailto:hello@tangram.dev".to_owned())
			.child("Contact Us")
			.into_node();
		let cards = PricingCards {
			free_cta,
			free_selected: false,
			team_cta,
			team_selected: false,
			enterprise_cta,
			enterprise_selected: false,
		};
		let table = PricingTable;
		let p = ui::P::new()
			.child("The cloud hosted and self hosted apps are free to use by an individual for evaluation and testing, but require a license to use as part of a team and in production.");
		let content = div().class("pricing-grid").child(
			ui::S1::new()
				.child(title)
				.child(subtitle)
				.child(cards)
				.child(table)
				.child(p),
		);
		Document::new()
			.child(PageLayout::new().child(content))
			.into_node()
	}
}

pub struct PricingCards {
	pub free_cta: Node,
	pub free_selected: bool,
	pub team_cta: Node,
	pub team_selected: bool,
	pub enterprise_cta: Node,
	pub enterprise_selected: bool,
}

impl Component for PricingCards {
	fn into_node(self) -> Node {
		let free_card = PricingCard {
			color: ui::colors::RED.to_owned(),
			cta: self.free_cta,
			features: vec![
				"Training with the CLI.".to_owned(),
				"Prediction with the language libraries.".to_owned(),
			],
			price: "$0".to_owned(),
			price_subtitle: "forever".to_owned(),
			selected: false,
			title: "Free".to_owned(),
		};
		let team_card = PricingCard {
			color: ui::colors::GREEN.to_owned(),
			cta: self.team_cta,
			features: vec![
				"Everything in the Free plan.".to_owned(),
				"Use the cloud-hosted app.".to_owned(),
			],
			price: "contact us".to_owned(),
			price_subtitle: "hello@tangram.dev".to_owned(),
			selected: false,
			title: "Team".to_owned(),
		};
		let enterprise_card = PricingCard {
			color: ui::colors::BLUE.to_owned(),
			cta: self.enterprise_cta,
			features: vec![
				"Everything in the Team plan.".to_owned(),
				"Use the self-hosted app.".to_owned(),
			],
			price: "contact us".to_owned(),
			price_subtitle: "hello@tangram.dev".to_owned(),
			selected: false,
			title: "Enterprise".to_owned(),
		};
		div()
			.class("pricing-cards-grid")
			.child(free_card)
			.child(team_card)
			.child(enterprise_card)
			.into_node()
	}
}

pub struct PricingCard {
	color: String,
	cta: Node,
	features: Vec<String>,
	price: String,
	price_subtitle: String,
	selected: bool,
	title: String,
}

impl Component for PricingCard {
	fn into_node(self) -> Node {
		let selected_class = if self.selected {
			Some("pricing-card-grid-selected")
		} else {
			None
		};
		let token = div().child(ui::Token::new().color(self.color).child(self.title));
		let price = div()
			.class("pricing-card-price-wrapper")
			.child(div().class("pricing-card-price").child(self.price))
			.child(
				div()
					.class("pricing-card-subtitle")
					.child(self.price_subtitle),
			);
		let features_list = div().class("pricing-card-features-list").children(
			self.features
				.into_iter()
				.map(|feature| div().class("pricing-card-feature").child(feature)),
		);
		div()
			.class("pricing-card-grid")
			.class(selected_class)
			.child(
				div()
					.class("pricing-card-content-grid")
					.child(token)
					.child(price)
					.child(features_list),
			)
			.child(self.cta)
			.into_node()
	}
}

pub struct PricingTable;

impl Component for PricingTable {
	fn into_node(self) -> Node {
		let head = ui::TableHeader::new().child(
			ui::TableRow::new()
				.child(ui::TableHeaderCell::new())
				.child(
					ui::TableHeaderCell::new().width("33%".to_owned()).child(
						ui::Token::new()
							.color(ui::colors::RED.to_owned())
							.child("Free"),
					),
				)
				.child(
					ui::TableHeaderCell::new().width("33%".to_owned()).child(
						ui::Token::new()
							.color(ui::colors::GREEN.to_owned())
							.child("Team"),
					),
				)
				.child(
					ui::TableHeaderCell::new().width("33%".to_owned()).child(
						ui::Token::new()
							.color(ui::colors::BLUE.to_owned())
							.child("Enterprise"),
					),
				),
		);
		let body = ui::TableBody::new()
			.child(
				ui::TableRow::new()
					.child(ui::TableHeaderCell::new().child("Training"))
					.child(ui::TableCell::new().child("✅"))
					.child(ui::TableCell::new().child("✅"))
					.child(ui::TableCell::new().child("✅")),
			)
			.child(
				ui::TableRow::new()
					.child(ui::TableHeaderCell::new().child("Prediction"))
					.child(ui::TableCell::new().child("✅"))
					.child(ui::TableCell::new().child("✅"))
					.child(ui::TableCell::new().child("✅")),
			)
			.child(
				ui::TableRow::new()
					.child(ui::TableHeaderCell::new().child("GitHub support"))
					.child(ui::TableCell::new().child("✅"))
					.child(ui::TableCell::new().child("✅"))
					.child(ui::TableCell::new().child("✅")),
			)
			.child(
				ui::TableRow::new()
					.child(ui::TableHeaderCell::new().child("Cloud-hosted app"))
					.child(ui::TableCell::new().child(""))
					.child(ui::TableCell::new().child("✅"))
					.child(ui::TableCell::new().child("✅")),
			)
			.child(
				ui::TableRow::new()
					.child(ui::TableHeaderCell::new().child("Email support"))
					.child(ui::TableCell::new().child(""))
					.child(ui::TableCell::new().child("✅"))
					.child(ui::TableCell::new().child("✅")),
			)
			.child(
				ui::TableRow::new()
					.child(ui::TableHeaderCell::new().child("Self-hosted app"))
					.child(ui::TableCell::new().child(""))
					.child(ui::TableCell::new().child(""))
					.child(ui::TableCell::new().child("✅")),
			)
			.child(
				ui::TableRow::new()
					.child(ui::TableHeaderCell::new().child("Slack support"))
					.child(ui::TableCell::new().child(""))
					.child(ui::TableCell::new().child(""))
					.child(ui::TableCell::new().child("✅")),
			);
		ui::Table::new()
			.width("100%".to_owned())
			.child(head)
			.child(body)
			.into_node()
	}
}
