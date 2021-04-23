use html::{component, html, style};
use tangram_ui as ui;

#[component]
pub fn Tuning() {
	let threshold_index: usize = 9;
	let accuracy = 0.8567;
	let precision = 0.7891;
	let recall = 0.5658;
	let code = format!("// Update your code to use the selected threshold.\nmodel.predict(input, {{ threshold: {:.2} }})", 0.5);
	let accuracy_style = style! {
		"grid-area" => "accuracy",
	};
	let precision_style = style! {
		"grid-area" => "precision",
	};
	let recall_style = style! {
		"grid-area" => "recall",
	};
	html! {
		<div class="index-step">
			<div>
				<div class="index-step-title">{"Tune your model to get the best performance."}</div>
				<div class="index-step-text">
					{"Tune binary classification models to your preferred tradeoff between precision and recall. To use your selected threshold, update the "}
					<ui::InlineCode>{"predict"}</ui::InlineCode>
					{" call in your code."}
				</div>
			</div>
			<ui::Window padding={Some(true)}>
				<div class="tuning-grid">
					<ui::Slider
						id="tuning-slider"
						max={18.0}
						min={0.0}
						value={threshold_index}
					/>
					<div class="tuning-number-chart-grid">
						<div style={accuracy_style}>
							<ui::NumberCard
								id?="tuning-accuracy"
								title="Accuracy"
								value={ui::format_percent(accuracy)}
							/>
						</div>
						<div style={precision_style}>
							<ui::NumberCard
								id?="tuning-precision"
								title="Precision"
								value={ui::format_percent(precision)}
							/>
						</div>
						<div style={recall_style}>
							<ui::NumberCard
								id?="tuning-recall"
								title="Recall"
								value={ui::format_percent(recall)}
							/>
						</div>
					</div>
					<ui::Card>
						<ui::Code
							id?="tuning-code"
							code={code}
						/>
					</ui::Card>
				</div>
			</ui::Window>
		</div>
	}
}
