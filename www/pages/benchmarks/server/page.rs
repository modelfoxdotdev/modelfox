use html::{component, html, Props};
use tangram_charts::{bar_chart::BarChartPoint, bar_chart::BarChartSeries, components::BarChart};
use tangram_serve::{self, client};
use tangram_ui as ui;
use tangram_www_layouts::{
	document::{Document, DocumentProps},
	layout::Layout,
};

#[component]
pub fn Page() {
	let document_props = DocumentProps {
		client_wasm_js_src: Some(client!()),
	};
	let dataset_select_field_options = Some(vec![
		ui::SelectFieldOption {
			text: "allstate".to_owned(),
			value: "allstate".to_owned(),
		},
		ui::SelectFieldOption {
			text: "flights".to_owned(),
			value: "flights".to_owned(),
		},
		ui::SelectFieldOption {
			text: "higgs".to_owned(),
			value: "higgs".to_owned(),
		},
	]);
	let cpu_select_field_options = Some(vec![
		ui::SelectFieldOption {
			text: "Apple M1".to_owned(),
			value: "m1".to_owned(),
		},
		ui::SelectFieldOption {
			text: "AMD Ryzen 7 2700".to_owned(),
			value: "ryzen".to_owned(),
		},
	]);
	html! {
		<Document {document_props}>
			<Layout>
				<div class="index-grid">
					<ui::S1>
						<ui::H1>{"Tangram Tree Benchmarks"}</ui::H1>
						<ui::SelectField
							id?="cpu-select"
							label?="Select CPU"
							options?={cpu_select_field_options}
							value?="m1"
						/>
						<ui::SelectField
							id?="dataset-select"
							label?="Select Dataset"
							options?={dataset_select_field_options}
							value?="flights"
						/>
						<div id="m1">
							<Benchmarks cpu={Cpu::M1} />
						</div>
						<div id="ryzen">
							<Benchmarks cpu={Cpu::Ryzen} />
						</div>
					</ui::S1>
				</div>
			</Layout>
		</Document>
	}
}

#[derive(Props)]
struct BenchmarksProps {
	cpu: Cpu,
}

#[component]
fn Benchmarks(props: BenchmarksProps) {
	html! {
		<>
			<div id={format!("{}-flights", props.cpu)}>
				<ui::S2>
					<ui::H2>{"Flights"}</ui::H2>
					<ui::P>{"The flights dataset contains 9 columns. There are 10,000,000 rows in the train dataset and 100,000 rows in the test dataset. The target column is "}<ui::InlineCode>{"dep_delayed_15min"}</ui::InlineCode>{" and it is a binary classification task."}</ui::P>
					<div class="benchmarks-grid-item duration">
						<DurationTable cpu={props.cpu} dataset={Dataset::Flights} />
						<DurationChart cpu={props.cpu} dataset={Dataset::Flights} />
					</div>
					<div class="benchmarks-grid-item memory">
						<MemoryTable cpu={props.cpu} dataset={Dataset::Flights} />
						<MemoryChart cpu={props.cpu} dataset={Dataset::Flights} />
					</div>
					<div class="benchmarks-grid-item metric">
						<AUCTable cpu={props.cpu} dataset={Dataset::Flights} />
						<AUCChart cpu={props.cpu} dataset={Dataset::Flights} />
					</div>
				</ui::S2>
			</div>
			<div id={format!("{}-higgs", props.cpu)}>
				<ui::S2>
					<ui::H2>{"Higgs"}</ui::H2>
					<ui::P>{"The higgs dataset contains 28 numeric columns and the target column "} <ui::InlineCode>{"signal"}</ui::InlineCode>{". There are 10,500,000 rows in the train dataset and 500,000 rows in the test dataset. It is a binary classifiation task."}</ui::P>
					<div class="benchmarks-grid-item duration">
						<DurationTable cpu={props.cpu} dataset={Dataset::Higgs} />
						<DurationChart cpu={props.cpu} dataset={Dataset::Higgs} />
					</div>
					<div class="benchmarks-grid-item memory">
						<MemoryTable cpu={props.cpu} dataset={Dataset::Higgs} />
						<MemoryChart cpu={props.cpu} dataset ={Dataset::Higgs} />
					</div>
					<div class="benchmarks-grid-item metric">
						<AUCTable cpu={props.cpu} dataset={Dataset::Higgs} />
						<AUCChart cpu={props.cpu} dataset={Dataset::Higgs} />
					</div>
				</ui::S2>
			</div>
			<div id={format!("{}-allstate", props.cpu)}>
				<ui::S2>
					<ui::H2>{"Allstate"}</ui::H2>
					<ui::P>{"The allstate dataset contains 35 columns. There are 10,547,432 rows in the train dataset and 2,636,858 rows in the test dataset. The target column is "}<ui::InlineCode>{"claim_amount"}</ui::InlineCode>{" and it is a regression task."}</ui::P>
					<div class="benchmarks-grid-item duration">
						<DurationTable cpu={props.cpu} dataset={Dataset::Allstate} />
						<DurationChart cpu={props.cpu} dataset={Dataset::Allstate} />
					</div>
					<div class="benchmarks-grid-item memory">
						<MemoryTable cpu={props.cpu} dataset={Dataset::Allstate} />
						<MemoryChart cpu={props.cpu} dataset={Dataset::Allstate} />
					</div>
					<div class="benchmarks-grid-item metric">
						<MSETable cpu={props.cpu} dataset={Dataset::Allstate} />
						<MSEChart cpu={props.cpu} dataset={Dataset::Allstate} />
					</div>
				</ui::S2>
			</div>
		</>
	}
}

#[derive(Props)]
struct DurationTableProps {
	dataset: Dataset,
	cpu: Cpu,
}

#[component]
fn DurationTable(props: DurationTableProps) {
	let tangram_entry = benchmark_data(props.cpu, props.dataset, Library::Tangram);
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>{"Library"}</ui::TableHeaderCell>
					<ui::TableHeaderCell>{"Duration"}</ui::TableHeaderCell>
					<ui::TableHeaderCell>{"v. Tangram"}</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{LIBRARIES.iter().cloned().map(|library| {
					let entry = benchmark_data(props.cpu, props.dataset, library);
					let color = if library == Library::Tangram { Some(ui::colors::BLUE.to_owned()) } else { None };
					let text_color = if library == Library::Tangram { Some("var(--fun-text-color)".to_owned()) } else { None };
					let duration = format_duration(entry.duration);
					let duration_delta = format_delta(entry.duration / tangram_entry.duration);
					html! {
						<ui::TableRow color?={color} text_color?={text_color}>
							<ui::TableCell>{library.to_string()}</ui::TableCell>
							<ui::TableCell>{duration}</ui::TableCell>
							<ui::TableCell>{duration_delta}</ui::TableCell>
						</ui::TableRow>
					}
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}

#[derive(Props)]
struct DurationChartProps {
	dataset: Dataset,
	cpu: Cpu,
}

#[component]
fn DurationChart(props: DurationChartProps) {
	let chart_data: Vec<BarChartSeries> = LIBRARIES
		.iter()
		.cloned()
		.map(|library| {
			let dataset = props.dataset;
			let data = vec![BarChartPoint {
				label: dataset.to_string(),
				x: 0.0,
				y: Some(benchmark_data(props.cpu, dataset, library).duration),
			}];
			BarChartSeries {
				color: color_for_library(library).to_owned(),
				data,
				title: Some(library.to_string()),
			}
		})
		.collect();
	html! {
		<BarChart
			id?={Some(format!("{}_{}_duration_chart", props.cpu, props.dataset))}
			group_gap?={Some(10.0)}
			series?={Some(chart_data)}
			title?="Training Time (lower is better)"
			x_axis_title?="Dataset"
			y_axis_title?="Training Time (seconds)"
			y_min?={Some(0.0)}
		/>
	}
}
#[derive(Props)]
struct MemoryTableProps {
	dataset: Dataset,
	cpu: Cpu,
}

#[component]
fn MemoryTable(props: MemoryTableProps) {
	let tangram_entry = benchmark_data(props.cpu, props.dataset, Library::Tangram);
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>{"Library"}</ui::TableHeaderCell>
					<ui::TableHeaderCell>{"Memory"}</ui::TableHeaderCell>
					<ui::TableHeaderCell>{"v. Tangram"}</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{LIBRARIES.iter().cloned().map(|library| {
					let entry = benchmark_data(props.cpu,props.dataset, library);
					let color = if library == Library::Tangram { Some(ui::colors::BLUE.to_owned()) } else { None };
					let text_color = if library == Library::Tangram { Some("var(--fun-text-color)".to_owned()) } else { None };
					let duration = format_memory(entry.memory);
					let duration_delta = format_delta(entry.memory / tangram_entry.memory);
					html! {
						<ui::TableRow color?={color} text_color?={text_color}>
							<ui::TableCell>{library.to_string()}</ui::TableCell>
							<ui::TableCell>{duration}</ui::TableCell>
							<ui::TableCell>{duration_delta}</ui::TableCell>
						</ui::TableRow>
					}
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}

#[derive(Props)]
struct MemoryChartProps {
	dataset: Dataset,
	cpu: Cpu,
}

#[component]
fn MemoryChart(props: MemoryChartProps) {
	let chart_data: Vec<BarChartSeries> = LIBRARIES
		.iter()
		.cloned()
		.map(|library| {
			let dataset = props.dataset;
			let data = vec![BarChartPoint {
				label: dataset.to_string(),
				x: 0.0,
				y: Some(benchmark_data(props.cpu, dataset, library).memory),
			}];
			BarChartSeries {
				color: color_for_library(library).to_owned(),
				data,
				title: Some(library.to_string()),
			}
		})
		.collect();
	html! {
		<BarChart
			id?={Some(format!("{}_{}_memory_chart", props.cpu, props.dataset))}
			group_gap?={Some(10.0)}
			series?={Some(chart_data)}
			title?="Memory Usage (lower is better)"
			x_axis_title?="Dataset"
			y_axis_title?="Memory Usage (GB)"
			y_min?={Some(0.0)}
		/>
	}
}
#[derive(Props)]
struct MseTableProps {
	dataset: Dataset,
	cpu: Cpu,
}

#[component]
fn MSETable(props: MseTableProps) {
	let tangram_entry = benchmark_data(props.cpu, props.dataset, Library::Tangram);
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>{"Library"}</ui::TableHeaderCell>
					<ui::TableHeaderCell>{"MSE"}</ui::TableHeaderCell>
					<ui::TableHeaderCell>{"v. Tangram"}</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{LIBRARIES.iter().cloned().map(|library| {
					let entry = benchmark_data(props.cpu,props.dataset, library);
					let color = if library == Library::CatBoost { Some(ui::colors::RED.to_owned()) } else { None };
					let text_color = if library == Library::CatBoost { Some("var(--fun-text-color)".to_owned()) } else { None };
					let mse = ui::format_float_with_digits(entry.metric, 6);
					let mse_delta = format_delta(entry.metric / tangram_entry.metric);
					html! {
						<ui::TableRow color?={color} text_color?={text_color}>
							<ui::TableCell>{library.to_string()}</ui::TableCell>
							<ui::TableCell>{mse}</ui::TableCell>
							<ui::TableCell>{mse_delta}</ui::TableCell>
						</ui::TableRow>
					}
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}

#[derive(Props)]
struct AucChartProps {
	dataset: Dataset,
	cpu: Cpu,
}

#[component]
fn AUCChart(props: AucChartProps) {
	let chart_data: Vec<BarChartSeries> = LIBRARIES
		.iter()
		.cloned()
		.map(|library| {
			let dataset = props.dataset;
			let data = vec![BarChartPoint {
				label: dataset.to_string(),
				x: 0.0,
				y: Some(benchmark_data(props.cpu, dataset, library).metric),
			}];
			BarChartSeries {
				color: color_for_library(library).to_owned(),
				data,
				title: Some(library.to_string()),
			}
		})
		.collect();
	html! {
		<BarChart
			id?={Some(format!("{}_{}_metric_chart", props.cpu, props.dataset))}
			group_gap?={Some(10.0)}
			series?={Some(chart_data)}
			title?="AUC (higher is better)"
			x_axis_title?="Dataset"
			y_axis_title?="AUC"
			y_min?={Some(0.0)}
		/>
	}
}

#[derive(Props)]
struct AucTableProps {
	dataset: Dataset,
	cpu: Cpu,
}

#[component]
fn AUCTable(props: AucTableProps) {
	let tangram_entry = benchmark_data(props.cpu, props.dataset, Library::Tangram);
	html! {
		<ui::Table width?="100%">
			<ui::TableHeader>
				<ui::TableRow>
					<ui::TableHeaderCell>{"Library"}</ui::TableHeaderCell>
					<ui::TableHeaderCell>{"AUC"}</ui::TableHeaderCell>
					<ui::TableHeaderCell>{"v. Tangram"}</ui::TableHeaderCell>
				</ui::TableRow>
			</ui::TableHeader>
			<ui::TableBody>
				{LIBRARIES.iter().cloned().map(|library| {
					let entry = benchmark_data(props.cpu, props.dataset, library);
					let color = if library == Library::Tangram { Some(ui::colors::BLUE.to_owned()) } else { None };
					let text_color = if library == Library::Tangram { Some("var(--fun-text-color)".to_owned()) } else { None };
					let auc = ui::format_float_with_digits(entry.metric, 4);
					let auc_delta = format_delta(entry.metric / tangram_entry.metric);
					html! {
						<ui::TableRow color?={color} text_color?={text_color}>
							<ui::TableCell>{library.to_string()}</ui::TableCell>
							<ui::TableCell>{auc}</ui::TableCell>
							<ui::TableCell>{auc_delta}</ui::TableCell>
						</ui::TableRow>
					}
				}).collect::<Vec<_>>()}
			</ui::TableBody>
		</ui::Table>
	}
}

#[derive(Props)]
struct MseChartProps {
	dataset: Dataset,
	cpu: Cpu,
}

#[component]
fn MSEChart(props: MseChartProps) {
	let chart_data: Vec<BarChartSeries> = LIBRARIES
		.iter()
		.cloned()
		.map(|library| {
			let dataset = props.dataset;
			let data = vec![BarChartPoint {
				label: dataset.to_string(),
				x: 0.0,
				y: Some(benchmark_data(props.cpu, dataset, library).metric),
			}];
			BarChartSeries {
				color: color_for_library(library).to_owned(),
				data,
				title: Some(library.to_string()),
			}
		})
		.collect();
	html! {
		<BarChart
			id?={Some(format!("{}_{}_metric_chart", props.cpu, props.dataset))}
			group_gap?={Some(10.0)}
			series?={Some(chart_data)}
			title?="Mean Squared Error (lower is better)"
			x_axis_title?="Dataset"
			y_axis_title?="MSE"
			y_min?={Some(0.0)}
		/>
	}
}

fn format_duration(value: f64) -> String {
	format!("{} sec", ui::format_float_with_digits(value, 4))
}

fn format_memory(value: f64) -> String {
	format!("{} GB", ui::format_float_with_digits(value, 4))
}

fn format_delta(value: f64) -> String {
	format!("{}x", ui::format_float_with_digits(value, 4))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cpu {
	M1,
	Ryzen,
}

impl std::fmt::Display for Cpu {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Cpu::M1 => write!(f, "m1"),
			Cpu::Ryzen => write!(f, "ryzen"),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Dataset {
	Allstate,
	Flights,
	Higgs,
}

impl std::fmt::Display for Dataset {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Dataset::Allstate => write!(f, "allstate"),
			Dataset::Flights => write!(f, "flights"),
			Dataset::Higgs => write!(f, "higgs"),
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Library {
	CatBoost,
	LightGbm,
	SkLearn,
	Tangram,
	XgBoost,
}

const LIBRARIES: &[Library] = &[
	Library::CatBoost,
	Library::LightGbm,
	Library::SkLearn,
	Library::Tangram,
	Library::XgBoost,
];

impl std::fmt::Display for Library {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Library::CatBoost => write!(f, "catboost"),
			Library::LightGbm => write!(f, "lightgbm"),
			Library::SkLearn => write!(f, "sklearn"),
			Library::Tangram => write!(f, "tangram"),
			Library::XgBoost => write!(f, "xgboost"),
		}
	}
}

struct BenchmarkEntry {
	duration: f64,
	memory: f64,
	metric: f64,
}

fn benchmark_data(cpu: Cpu, dataset: Dataset, library: Library) -> BenchmarkEntry {
	match (cpu, dataset, library) {
		// Allstate
		(Cpu::Ryzen, Dataset::Allstate, Library::Tangram) => BenchmarkEntry {
			duration: 56.4639,
			memory: 4.829028,
			metric: 1587.9972,
		},
		(Cpu::Ryzen, Dataset::Allstate, Library::LightGbm) => BenchmarkEntry {
			duration: 80.766,
			memory: 12.210420,
			metric: 1587.0221,
		},
		(Cpu::Ryzen, Dataset::Allstate, Library::XgBoost) => BenchmarkEntry {
			duration: 82.463,
			memory: 12.424112,
			metric: 1581.0436,
		},
		(Cpu::Ryzen, Dataset::Allstate, Library::CatBoost) => BenchmarkEntry {
			duration: 871.858,
			memory: 19.086404,
			metric: 1579.6266,
		},
		(Cpu::Ryzen, Dataset::Allstate, Library::SkLearn) => BenchmarkEntry {
			duration: 78.3966,
			memory: 10.934560,
			metric: 1583.9514,
		},
		// Flights
		(Cpu::Ryzen, Dataset::Flights, Library::Tangram) => BenchmarkEntry {
			duration: 38.256,
			memory: 1.137140,
			metric: 0.78150725,
		},
		(Cpu::Ryzen, Dataset::Flights, Library::LightGbm) => BenchmarkEntry {
			duration: 44.9627,
			memory: 1.997200,
			metric: 0.7807312,
		},
		(Cpu::Ryzen, Dataset::Flights, Library::XgBoost) => BenchmarkEntry {
			duration: 49.71078,
			memory: 2.417792,
			metric: 0.75779957,
		},
		(Cpu::Ryzen, Dataset::Flights, Library::CatBoost) => BenchmarkEntry {
			duration: 343.025076746,
			memory: 10.096476,
			metric: 0.7357335,
		},
		(Cpu::Ryzen, Dataset::Flights, Library::SkLearn) => BenchmarkEntry {
			duration: 59.461891259,
			memory: 2.589488,
			metric: 0.7589289,
		},
		// Higgs
		(Cpu::Ryzen, Dataset::Higgs, Library::Tangram) => BenchmarkEntry {
			duration: 84.701200426,
			memory: 2.456480,
			metric: 0.83177507,
		},
		(Cpu::Ryzen, Dataset::Higgs, Library::LightGbm) => BenchmarkEntry {
			duration: 111.403243358,
			memory: 11.622168,
			metric: 0.83145106,
		},
		(Cpu::Ryzen, Dataset::Higgs, Library::XgBoost) => BenchmarkEntry {
			duration: 100.569081359,
			memory: 12.743680,
			metric: 0.81292254,
		},
		(Cpu::Ryzen, Dataset::Higgs, Library::CatBoost) => BenchmarkEntry {
			duration: 298.102982833,
			memory: 13.222584,
			metric: 0.81350523,
		},
		(Cpu::Ryzen, Dataset::Higgs, Library::SkLearn) => BenchmarkEntry {
			duration: 201.186598547,
			memory: 9.298212,
			metric: 0.83165807,
		},
		// Allstate
		(Cpu::M1, Dataset::Allstate, Library::Tangram) => BenchmarkEntry {
			duration: 43.170980041,
			memory: 4.944788608,
			metric: 1587.9972,
		},
		(Cpu::M1, Dataset::Allstate, Library::LightGbm) => BenchmarkEntry {
			duration: 73.575861458,
			memory: 12.682184152,
			metric: 1587.0221,
		},
		(Cpu::M1, Dataset::Allstate, Library::XgBoost) => BenchmarkEntry {
			duration: 70.211878666,
			memory: 10.403035136,
			metric: 1581.0436,
		},
		(Cpu::M1, Dataset::Allstate, Library::CatBoost) => BenchmarkEntry {
			duration: 772.200986708,
			memory: 14.583577280,
			metric: 1579.6266,
		},
		(Cpu::M1, Dataset::Allstate, Library::SkLearn) => BenchmarkEntry {
			duration: 121.718725333,
			memory: 11.265747640,
			metric: 1583.809,
		},
		// Flights
		(Cpu::M1, Dataset::Flights, Library::Tangram) => BenchmarkEntry {
			duration: 35.630228,
			memory: 1.437882560,
			metric: 0.78150725,
		},
		(Cpu::M1, Dataset::Flights, Library::LightGbm) => BenchmarkEntry {
			duration: 43.468511541,
			memory: 2.612205648,
			metric: 0.7807312,
		},
		(Cpu::M1, Dataset::Flights, Library::XgBoost) => BenchmarkEntry {
			duration: 46.839800208,
			memory: 2.880443040,
			metric: 0.75779957,
		},
		(Cpu::M1, Dataset::Flights, Library::CatBoost) => BenchmarkEntry {
			duration: 329.038912958,
			memory: 7.290871200,
			metric: 0.7357335,
		},
		(Cpu::M1, Dataset::Flights, Library::SkLearn) => BenchmarkEntry {
			duration: 90.404242333,
			memory: 2.779639776,
			metric: 0.7580008,
		},
		// Higgs
		(Cpu::M1, Dataset::Higgs, Library::Tangram) => BenchmarkEntry {
			duration: 93.630773958,
			memory: 2.898570856,
			metric: 0.83177507,
		},
		(Cpu::M1, Dataset::Higgs, Library::LightGbm) => BenchmarkEntry {
			duration: 126.413343625,
			memory: 12.031269640,
			metric: 0.83151484,
		},
		(Cpu::M1, Dataset::Higgs, Library::XgBoost) => BenchmarkEntry {
			duration: 104.966819291,
			memory: 10.523970680,
			metric: 0.81292254,
		},
		(Cpu::M1, Dataset::Higgs, Library::CatBoost) => BenchmarkEntry {
			duration: 281.496573125,
			memory: 9.002074280,
			metric: 0.8133961,
		},
		(Cpu::M1, Dataset::Higgs, Library::SkLearn) => BenchmarkEntry {
			duration: 270.634199166,
			memory: 9.197352312,
			metric: 0.8316483,
		},
	}
}

fn color_for_library(library: Library) -> &'static str {
	match library {
		Library::Tangram => ui::colors::BLUE,
		Library::CatBoost => ui::colors::RED,
		Library::LightGbm => ui::colors::PURPLE,
		Library::SkLearn => ui::colors::ORANGE,
		Library::XgBoost => ui::colors::GREEN,
	}
}
