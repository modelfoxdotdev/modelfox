use html::{component, html};
use tangram_ui as ui;

#[component]
pub fn Train() {
	html! {
		<div class="index-step">
			<div>
				<div class="index-step-title">{"Train a model on the command line."}</div>
				<div class="index-step-text">
					{"Train a machine learning model by running "}
					<ui::InlineCode>{"tangram train"}</ui::InlineCode>
					{" with the path to a CSV file and the name of the column you want to predict."}
				</div>
				<br />
				<div class="index-step-text">
					{"The CLI automatically transforms your data into features, trains a number of models to predict the target column, and writes the best model to a "}
					<ui::InlineCode>{".tangram"}</ui::InlineCode>
					{" file."}
				</div>
				<br />
				<div class="index-step-text">
					{"If you want more control, you can provide a config file."}
				</div>
			</div>
			<ui::Window padding={Some(true)}>
				<ui::Code code={include_str!("./train.txt").to_owned()} hide_line_numbers?={Some(true)} />
			</ui::Window>
		</div>
	}
}
