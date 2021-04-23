use html::{component, html, style};
use indoc::indoc;
use tangram_serve::client;
use tangram_ui as ui;
use tangram_www_layouts::{
	docs_app_layout::DocsAppLayout,
	docs_layout::{DocsLayout, DocsPage, GettingStartedPage},
	document::{Document, DocumentProps},
};

#[component]
pub fn Page() {
	let document_props = DocumentProps {
		client_wasm_js_src: Some(client!()),
	};
	html! {
		<Document {document_props}>
			<DocsLayout selected_page={DocsPage::GettingStarted(GettingStartedPage::Inspect)} headings={None}>
				<ui::S1>
					<ui::H1>{"Inspect"}</ui::H1>
					<ui::S2>
						<ui::P>
							{"Run "}
							<ui::InlineCode>{"tangram app"}</ui::InlineCode>
							{" and open your browser to "}
							<ui::Link href="http://localhost:8080">
								{"http://localhost:8080"}
							</ui::Link>
							{"."}
						</ui::P>
						<ui::P>
							{"Create a new repo and upload the "}
							<ui::InlineCode>{".tangram"}</ui::InlineCode>
							{" file we just trained."}
						</ui::P>
						<ui::Callout
							title="Repo"
							level={ui::Level::Info}
						>
							{"A repo is where we can compare multiple versions of the same model."}
						</ui::Callout>
						<ui::P>
							{"Click on 'Training Metrics' and have a look at the confusion matrix."}
						</ui::P>
						<TrainingMetrics />
						<ui::P>
							{"It looks like false negatives are a bit high. This means we are predicting people are healthy when they actually aren't. It would be better if the model had fewer false negatives, even if it means more false positives, because doctors can rule out heart disease with further testing. Let's make that change by going to the 'Tuning' page."}
						</ui::P>
						<ui::P>
							{"Turn down the threshold, and see how it affects precision and recall."}
						</ui::P>
						<Tuning />
						<ui::P>
							{"Once you've chosen a threshold, you can go back to your prediction code to use it."}
						</ui::P>
						<TuningCode />
					</ui::S2>
					<div class="docs-prev-next-buttons">
						<ui::Link href="predict">
							{"< Previous: Make a prediction."}
						</ui::Link>
						<ui::Link href="monitor">
							{"Next: Monitor your model in production. >"}
						</ui::Link>
					</div>
				</ui::S1>
			</DocsLayout>
		</Document>
	}
}

#[component]
pub fn TrainingMetrics() {
	html! {
		<DocsAppLayout>
			<ui::H1>{"Training Metrics"}</ui::H1>
			<ui::ConfusionMatrix
				class_label="positive"
				false_negatives={Some(20)}
				false_positives={Some(19)}
				true_negatives={Some(299)}
				true_positives={Some(400)}
			/>
		</DocsAppLayout>
	}
}

#[component]
pub fn Tuning() {
	let threshold_index: usize = 9;
	let accuracy = 0.8567;
	let precision = 0.7891;
	let recall = 0.5658;
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
		<DocsAppLayout>
			<ui::H1>{"Tuning"}</ui::H1>
			<ui::Slider
				id="tuning-slider"
				max={18.0}
				min={0.0}
				value={threshold_index}
			/>
			<div class="docs-inspect-tuning-number-chart-grid">
				<div style={accuracy_style}>
					<ui::NumberCard
						id?="docs-inspect-tuning-accuracy"
						title="Accuracy"
						value={ui::format_percent(accuracy)}
					/>
				</div>
				<div style={precision_style}>
					<ui::NumberCard
						id?="docs-inspect-tuning-precision"
						title="Precision"
						value={ui::format_percent(precision)}
					/>
				</div>
				<div style={recall_style}>
					<ui::NumberCard
						id?="docs-inspect-tuning-recall"
						title="Recall"
						value={ui::format_percent(recall)}
					/>
				</div>
			</div>
		</DocsAppLayout>
	}
}

#[component]
pub fn TuningCode() {
	let code_for_language = ui::highlight_code_for_language(ui::CodeForLanguage {
		elixir: indoc! {
			r#"
				predict_options = %Tangram.PredictOptions{
					threshold: 0.5,
					compute_feature_contributions: false
				}
				output = Tangram.predict(model, input, predict_options)
			"#
		}
		.into(),
		go: indoc! {
			r#"
				predictOptions := tangram.PredictOptions{
					Threshold:                   0.5,
					ComputeFeatureContributions: false,
				}
				output := model.PredictOne(input, &predictOptions)
			"#
		}
		.into(),
		javascript: indoc! {
			r#"
				options = {
					threshold: 0.5,
					computeFeatureContributions: true
				}
				let output = model.predictSync(input, options)
			"#
		}
		.into(),
		python: indoc! {
			r#"
				predict_options = tangram.PredictOptions(
						threshold=0.5,
						compute_feature_contributions=True
				)
				output = model.predict(input, predict_options)
			"#
		}
		.into(),
		ruby: indoc! {
			r#"
				options = Tangram::PredictOptions.new(
					threshold: 0.5,
					compute_feature_contributions: true
				)
				output = model.predict(input, options: options)
			"#
		}
		.into(),
		rust: indoc! {
			r#"
				let options = tangram::PredictOptions {
					threshold: Some(0.5),
					compute_feature_contributions: Some(true),
				};
				let output = model.predict_one(input.clone(), Some(options.clone()));
			"#
		}
		.into(),
	});
	html! {
		<ui::Window padding={Some(true)}>
			<ui::CodeSelect
				id="predict-threshold"
				code_for_language={code_for_language}
			/>
		</ui::Window>
	}
}
