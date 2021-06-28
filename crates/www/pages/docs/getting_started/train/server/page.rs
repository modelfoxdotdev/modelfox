use crate::dataset_preview::DatasetPreview;
use pinwheel::prelude::*;
use std::borrow::Cow;
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage},
	document::Document,
};

pub struct Page;

impl Component for Page {
	fn into_node(self) -> Node {
		let install_p = ui::P::new().child("If you haven't already, ").child(
			ui::Link::new()
				.href("/docs/install".to_owned())
				.child("install the Tangram CLI."),
		);
		let install = ui::S2::new()
			.child(ui::H2::new().child("Install the Tangram CLI"))
			.child(install_p);
		let data_p1 = ui::P::new().child(
			ui::Link::new()
				.href("/heart_disease.csv".to_owned())
				.child("download heart_disease.csv"),
		);
		let data_p2 = ui::P::new()
			.child("The heart disease dataset contains information from cardiac patients such as their age, cholesterol, and stress test results. Below are some example rows.");
		let data_p3 = ui::P::new()
			.child("The last column, called ")
			.child(ui::InlineCode::new("diagnosis"))
			.child(", is either ")
			.child(ui::InlineCode::new("Positive"))
			.child(" if the patient has heart disease or ")
			.child(ui::InlineCode::new("Negative"))
			.child(" if they don't.");
		let data = ui::S2::new()
			.child(ui::H2::new().child("Get the data"))
			.child(data_p1)
			.child(data_p2)
			.child(DatasetPreview)
			.child(data_p3);
		let train_p1 = ui::P::new()
			.child("We can train a model to predict the ")
			.child(ui::InlineCode::new("diagnosis"))
			.child(" column using the ")
			.child(ui::InlineCode::new("tangram train"))
			.child(" command, passing in the path to the CSV file and the name of the column we want to predict, called the ")
			.child(ui::InlineCode::new("target"))
			.child(" column.");
		let train_window = ui::Window::new().child(
			ui::Code::new()
				.code(Cow::Borrowed(
					"$ tangram train --file heart_disease.csv --target diagnosis",
				))
				.hide_line_numbers(true),
		);
		let train_p2 = ui::P::new()
			.child("The CLI automatically transforms the data into features, trains a number of models to predict the target column, and writes the best model to a ")
			.child(ui::InlineCode::new(".tangram"))
			.child(" file. We can use this file to make predictions from our code.");
		let train = ui::S2::new()
			.child(ui::H2::new().child("Train"))
			.child(train_p1)
			.child(train_window)
			.child(train_p2);
		let prev_next_buttons = div()
			.class("docs-prev-next-buttons")
			.child(
				ui::Link::new()
					.href("./".to_owned())
					.child("< Previous: Overview"),
			)
			.child(
				ui::Link::new()
					.href("predict/".to_owned())
					.child("Next: Make a Prediction. >"),
			);
		let content = ui::S1::new()
			.child(ui::H1::new().child("Train"))
			.child(install)
			.child(data)
			.child(train)
			.child(prev_next_buttons);
		let layout = DocsLayout::new()
			.selected_page(DocsPage::GettingStarted(GettingStartedPage::Train))
			.child(content);
		Document::new().child(layout).into_node()
	}
}
