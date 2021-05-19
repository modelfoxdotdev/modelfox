use pinwheel::prelude::*;
use tangram_ui as ui;
use tangram_www_layouts::{document::Document, page_layout::PageLayout};

#[derive(ComponentBuilder)]
pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let title = ui::H1::new().center(true).child("Pricing");
		let free_cta = ui::Button::new()
			.color("var(--green)".to_owned())
			.href("/docs/install".to_owned())
			.child("Install the CLI")
			.into_node();
		let team_cta = ui::Button::new()
			.color("var(--indigo)".to_owned())
			.href("https://app.tangram.xyz/login".to_owned())
			.child("Free Trial")
			.into_node();
		let enterprise_cta = ui::Button::new()
			.color("var(--blue)".to_owned())
			.href("mailto:hello@tangram.xyz".to_owned())
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
		let faqs = FAQs;
		let content = div()
			.class("pricing-grid")
			.child(ui::S1::new().child(title).child(cards).child(faqs));
		Document::new()
			.child(PageLayout::new().child(content))
			.into_node()
	}
}

#[derive(ComponentBuilder)]
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
			color: "var(--indigo)".to_owned(),
			cta: self.free_cta,
			features: vec![
				"Training with the CLI".to_owned(),
				"Prediction with the language libraries".to_owned(),
				"Run the app as a single user on your own hardware.".to_owned(),
			],
			price: "$0".to_owned(),
			price_subtitle: "forever".to_owned(),
			selected: false,
			title: "Free".to_owned(),
		};
		let team_card = PricingCard {
			color: "var(--green)".to_owned(),
			cta: self.team_cta,
			features: vec![
				"Everything in the Free plan".to_owned(),
				"Cloud-hosted".to_owned(),
			],
			price: "$199".to_owned(),
			price_subtitle: "per user per month".to_owned(),
			selected: false,
			title: "Team".to_owned(),
		};
		let enterprise_card = PricingCard {
			color: "var(--blue)".to_owned(),
			cta: self.enterprise_cta,
			features: vec![
				"Run the app on your own servers.".to_owned(),
				"Support for multiple users.".to_owned(),
			],
			price: "contact us".to_owned(),
			price_subtitle: "hello@tangram.xyz".to_owned(),
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
		let class = classes!(
			"pricing-card-grid",
			if self.selected {
				Some("pricing-card-grid-selected")
			} else {
				None
			},
		);
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
			.class(class)
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

struct FAQs;

impl Component for FAQs {
	fn into_node(self) -> Node {
		let what_is_tangram = "Tangram makes it easy to train and deploy machine learning models. Programmers can train a model on the command line and make predictions from any programming language. Programmers can then collaborate with product teams to understand how the model works, tune it, and monitor it in production.";
		let is_my_data_sent_to_tangram = "All training happens on the computer you run the CLI on and prediction happens in the same process as your code without any network calls. If you opt-in to logging, your production data will be sent to Tangram. If you are on the self hosted plan and run the app on your own servers, Tangram will never receive any of your production data.";
		let how_do_i_get_started = r#"Tangram is open source and free to use as a single user. As engineers, we were annoyed every time we wanted to try a new tool but were met with a "contact us". We worked very hard to make Tangram incredibly easy to download and run. If anything is challenging in the set-up process let us know."#;
		let do_i_need_to_be_a_machine_learning_engineer = "No! Tangram was designed to make machine learning accessible to all engineers. Just like you do not need to know how to implement a B-Tree map for a database, you do not need to know what one hot encoding is to create a machine learning model. There are some basic guidelines to keep in mind when choosing a dataset that will help you in making successful models. Our engineers are eager to help out so get in touch with any questions!";
		ui::S2::new()
			.child(ui::H2::new().child("What is Tangram?"))
			.child(ui::P::new().child(what_is_tangram.to_owned()))
			.child(ui::H2::new().child("Is my data sent to Tangram?"))
			.child(ui::P::new().child(is_my_data_sent_to_tangram.to_owned()))
			.child(ui::H2::new().child("How do I get started?"))
			.child(ui::P::new().child(how_do_i_get_started.to_owned()))
			.child(
				ui::H2::new().child("Do I need to be a machine learning engineer to use Tangram?"),
			)
			.child(ui::P::new().child(do_i_need_to_be_a_machine_learning_engineer.to_owned()))
			.into_node()
	}
}
