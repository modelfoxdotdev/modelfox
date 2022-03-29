use futures_signals::map_ref;
use modelfox_charts::{bar_chart::BarChartPoint, bar_chart::BarChartSeries, components::BarChart};
use modelfox_ui as ui;
use pinwheel::prelude::*;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Benchmarks {
	dataset: Mutable<Dataset>,
	cpu: Mutable<Cpu>,
}

impl Benchmarks {
	pub fn new() -> Benchmarks {
		Benchmarks {
			dataset: Mutable::new(Dataset::Flights),
			cpu: Mutable::new(Cpu::M1),
		}
	}
}

impl Default for Benchmarks {
	fn default() -> Self {
		Self::new()
	}
}

impl Component for Benchmarks {
	fn into_node(self) -> Node {
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
		let benchmark = map_ref! {
			let dataset = self.dataset.signal(),
			let cpu = self.cpu.signal() =>
			Benchmark { dataset: *dataset, cpu: *cpu }
		};
		let modelfox_tree_benchmarks_description = ui::Markdown::new("ModelFox Tree is a pure Rust implementation of Gradient Boosted Decision Trees. It has the smallest memory footprint of the leading GBDT implementations and achieves state of the art speed and accuracy. Check it out on [GitHub](https://github.com/modelfoxdotdev/modelfox/blob/main/crates/tree).");
		ui::S1::new()
			.child(ui::H1::new("ModelFox Tree Benchmarks"))
			.child(modelfox_tree_benchmarks_description)
			.child_signal(self.dataset.signal().map({
				let dataset = self.dataset.clone();
				clone!(dataset_select_field_options);
				move |value| {
					ui::SelectField::new()
						.label("Select Dataset".to_owned())
						.options(dataset_select_field_options.clone())
						.value(value.to_string())
						.on_change({
							clone!(dataset);
							Some(Box::new(move |value: String| {
								dataset.set(value.parse().unwrap());
							}) as Box<dyn Fn(_)>)
						})
				}
			}))
			.child_signal(self.cpu.signal().map({
				let cpu = self.cpu.clone();
				clone!(cpu_select_field_options);
				move |value| {
					ui::SelectField::new()
						.label("Select CPU".to_owned())
						.options(cpu_select_field_options.clone())
						.value(value.to_string())
						.on_change({
							clone!(cpu);
							Some(Box::new(move |value: String| {
								cpu.set(value.parse().unwrap());
							}) as Box<dyn Fn(_)>)
						})
				}
			}))
			.child_signal(benchmark)
			.into_node()
	}
}

pub struct Benchmark {
	dataset: Dataset,
	cpu: Cpu,
}

impl Component for Benchmark {
	fn into_node(self) -> Node {
		match self.dataset {
			Dataset::Allstate => {
				let description = ui::Markdown::new(ui::doc!(
					r#"
						The allstate dataset contains 35 columns. There are 10,547,432 rows in the train dataset and 2,636,858 rows in the test dataset. The target column is `claim_amount` and it is a regression task.
					"#
				));
				let duration = ui::S2::new()
					.child(ui::H2::new("Training Time (lower is better)"))
					.child(
						div()
							.class("benchmarks-table-chart-grid")
							.child(DurationTable {
								dataset: Dataset::Allstate,
								cpu: self.cpu,
							})
							.child(DurationChart {
								dataset: Dataset::Allstate,
								cpu: self.cpu,
							}),
					);
				let memory = ui::S2::new()
					.child(ui::H2::new("Memory Usage (lower is better)"))
					.child(
						div()
							.class("benchmarks-table-chart-grid")
							.child(MemoryTable {
								dataset: Dataset::Allstate,
								cpu: self.cpu,
							})
							.child(MemoryChart {
								cpu: self.cpu,
								dataset: Dataset::Allstate,
							}),
					);
				let mse = ui::S2::new()
					.child(ui::H2::new("Mean Squared Error (lower is better)"))
					.child(
						div()
							.class("benchmarks-table-chart-grid")
							.child(MseTable {
								cpu: self.cpu,
								dataset: Dataset::Allstate,
							})
							.child(MseChart {
								cpu: self.cpu,
								dataset: Dataset::Allstate,
							}),
					);
				fragment()
					.child(description)
					.child(duration)
					.child(memory)
					.child(mse)
					.into_node()
			}
			Dataset::Flights => {
				let description = ui::Markdown::new(ui::doc!(
					r#"
						The flights dataset contains 9 columns. There are 10,000,000 rows in the train dataset and 100,000 rows in the test dataset. The target column is `dep_delayed_15min` and it is a binary classification task.
					"#
				));
				let duration = ui::S2::new()
					.child(ui::H2::new("Training Time (lower is better)"))
					.child(
						div()
							.class("benchmarks-table-chart-grid")
							.child(DurationTable {
								dataset: Dataset::Flights,
								cpu: self.cpu,
							})
							.child(DurationChart {
								dataset: Dataset::Flights,
								cpu: self.cpu,
							}),
					);
				let memory = ui::S2::new()
					.child(ui::H2::new("Memory Usage (lower is better)"))
					.child(
						div()
							.class("benchmarks-table-chart-grid")
							.child(MemoryTable {
								dataset: Dataset::Flights,
								cpu: self.cpu,
							})
							.child(MemoryChart {
								cpu: self.cpu,
								dataset: Dataset::Flights,
							}),
					);
				let auc = ui::S2::new()
					.child(ui::H2::new("AUC (higher is better)".to_owned()))
					.child(
						div()
							.class("benchmarks-table-chart-grid")
							.child(AucTable {
								cpu: self.cpu,
								dataset: Dataset::Flights,
							})
							.child(AucChart {
								cpu: self.cpu,
								dataset: Dataset::Flights,
							}),
					);
				fragment()
					.child(description)
					.child(duration)
					.child(memory)
					.child(auc)
					.into_node()
			}
			Dataset::Higgs => {
				let description = ui::Markdown::new(ui::doc!(
					r#"
						The higgs dataset contains 28 numeric columns and the target column `signal`. There are 10,500,000 rows in the train dataset and 500,000 rows in the test dataset. It is a binary classifiation task.
					"#
				));
				let duration = ui::S2::new()
					.child(ui::H2::new("Training Time (lower is better)"))
					.child(
						div()
							.class("benchmarks-table-chart-grid")
							.child(DurationTable {
								dataset: Dataset::Higgs,
								cpu: self.cpu,
							})
							.child(DurationChart {
								dataset: Dataset::Higgs,
								cpu: self.cpu,
							}),
					);
				let memory = ui::S2::new()
					.child(ui::H2::new("Memory Usage (lower is better)"))
					.child(
						div()
							.class("benchmarks-table-chart-grid")
							.child(MemoryTable {
								dataset: Dataset::Higgs,
								cpu: self.cpu,
							})
							.child(MemoryChart {
								cpu: self.cpu,
								dataset: Dataset::Higgs,
							}),
					);
				let auc = ui::S2::new()
					.child(ui::H2::new("AUC (higher is better)".to_owned()))
					.child(
						div()
							.class("benchmarks-table-chart-grid")
							.child(AucTable {
								dataset: Dataset::Higgs,
								cpu: self.cpu,
							})
							.child(AucChart {
								dataset: Dataset::Higgs,
								cpu: self.cpu,
							}),
					);
				fragment()
					.child(description)
					.child(duration)
					.child(memory)
					.child(auc)
					.into_node()
			}
		}
	}
}

struct DurationTable {
	dataset: Dataset,
	cpu: Cpu,
}

impl Component for DurationTable {
	fn into_node(self) -> Node {
		let modelfox_entry = benchmark_data(self.cpu, self.dataset, Library::ModelFox);
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Library"))
						.child(ui::TableHeaderCell::new().child("Duration"))
						.child(ui::TableHeaderCell::new().child("v. ModelFox")),
				),
			)
			.child(
				ui::TableBody::new().children(LIBRARIES.iter().cloned().map(|library| {
					let entry = benchmark_data(self.cpu, self.dataset, library);
					let color = if library == Library::ModelFox {
						Some(ui::colors::BLUE.to_owned())
					} else {
						None
					};
					let text_color = if library == Library::ModelFox {
						Some(ui::colors::FUN_TEXT.to_owned())
					} else {
						None
					};
					let duration = format_duration(entry.duration);
					let duration_delta = format_delta(entry.duration / modelfox_entry.duration);
					ui::TableRow::new()
						.color(color)
						.text_color(text_color)
						.child(ui::TableCell::new().child(library.to_string()))
						.child(ui::TableCell::new().child(duration))
						.child(ui::TableCell::new().child(duration_delta))
				})),
			)
			.into_node()
	}
}

struct DurationChart {
	dataset: Dataset,
	cpu: Cpu,
}

impl Component for DurationChart {
	fn into_node(self) -> Node {
		let chart_data: Vec<BarChartSeries> = LIBRARIES
			.iter()
			.cloned()
			.map(|library| {
				let dataset = self.dataset;
				let data = vec![BarChartPoint {
					label: dataset.to_string(),
					x: 0.0,
					y: Some(benchmark_data(self.cpu, dataset, library).duration),
				}];
				BarChartSeries {
					color: color_for_library(library).to_owned(),
					data,
					title: Some(library.to_string()),
				}
			})
			.collect();
		BarChart::new()
			.group_gap(10.0)
			.series(chart_data)
			.x_axis_title("Dataset".to_owned())
			.y_axis_title("Training Time (seconds)".to_owned())
			.y_min(0.0)
			.into_node()
	}
}

struct MemoryTable {
	dataset: Dataset,
	cpu: Cpu,
}

impl Component for MemoryTable {
	fn into_node(self) -> Node {
		let modelfox_entry = benchmark_data(self.cpu, self.dataset, Library::ModelFox);
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Library"))
						.child(ui::TableHeaderCell::new().child("Memory"))
						.child(ui::TableHeaderCell::new().child("v. ModelFox")),
				),
			)
			.child(
				ui::TableBody::new().children(LIBRARIES.iter().cloned().map(|library| {
					let entry = benchmark_data(self.cpu, self.dataset, library);
					let color = if library == Library::ModelFox {
						Some(ui::colors::BLUE.to_owned())
					} else {
						None
					};
					let text_color = if library == Library::ModelFox {
						Some(ui::colors::FUN_TEXT.to_owned())
					} else {
						None
					};
					let duration = format_memory(entry.memory);
					let duration_delta = format_delta(entry.memory / modelfox_entry.memory);
					ui::TableRow::new()
						.color(color)
						.text_color(text_color)
						.child(ui::TableCell::new().child(library.to_string()))
						.child(ui::TableCell::new().child(duration))
						.child(ui::TableCell::new().child(duration_delta))
				})),
			)
			.into_node()
	}
}

struct MemoryChart {
	dataset: Dataset,
	cpu: Cpu,
}

impl Component for MemoryChart {
	fn into_node(self) -> Node {
		let chart_data: Vec<BarChartSeries> = LIBRARIES
			.iter()
			.cloned()
			.map(|library| {
				let dataset = self.dataset;
				let data = vec![BarChartPoint {
					label: dataset.to_string(),
					x: 0.0,
					y: Some(benchmark_data(self.cpu, dataset, library).memory),
				}];
				BarChartSeries {
					color: color_for_library(library).to_owned(),
					data,
					title: Some(library.to_string()),
				}
			})
			.collect();
		BarChart::new()
			.group_gap(10.0)
			.series(chart_data)
			.x_axis_title("Dataset".to_owned())
			.y_axis_title("Memory Usage (GB)".to_owned())
			.y_min(0.0)
			.into_node()
	}
}

struct MseTable {
	cpu: Cpu,
	dataset: Dataset,
}

impl Component for MseTable {
	fn into_node(self) -> Node {
		let modelfox_entry = benchmark_data(self.cpu, self.dataset, Library::ModelFox);
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Library"))
						.child(ui::TableHeaderCell::new().child("MSE"))
						.child(ui::TableHeaderCell::new().child("v. ModelFox")),
				),
			)
			.child(
				ui::TableBody::new().children(LIBRARIES.iter().cloned().map(|library| {
					let entry = benchmark_data(self.cpu, self.dataset, library);
					let color = if library == Library::CatBoost {
						Some(ui::colors::RED.to_owned())
					} else {
						None
					};
					let text_color = if library == Library::CatBoost {
						Some(ui::colors::FUN_TEXT.to_owned())
					} else {
						None
					};
					let mse = ui::format_float_with_digits(entry.metric, 6);
					let mse_delta = format_delta(entry.metric / modelfox_entry.metric);
					ui::TableRow::new()
						.color(color)
						.text_color(text_color)
						.child(ui::TableCell::new().child(library.to_string()))
						.child(ui::TableCell::new().child(mse))
						.child(ui::TableCell::new().child(mse_delta))
				})),
			)
			.into_node()
	}
}

struct AucChart {
	dataset: Dataset,
	cpu: Cpu,
}

impl Component for AucChart {
	fn into_node(self) -> Node {
		let chart_data: Vec<BarChartSeries> = LIBRARIES
			.iter()
			.cloned()
			.map(|library| {
				let dataset = self.dataset;
				let data = vec![BarChartPoint {
					label: dataset.to_string(),
					x: 0.0,
					y: Some(benchmark_data(self.cpu, dataset, library).metric),
				}];
				BarChartSeries {
					color: color_for_library(library).to_owned(),
					data,
					title: Some(library.to_string()),
				}
			})
			.collect();
		BarChart::new()
			.group_gap(10.0)
			.series(chart_data)
			.x_axis_title("Dataset".to_owned())
			.y_axis_title("AUC".to_owned())
			.y_min(0.0)
			.into_node()
	}
}

struct AucTable {
	dataset: Dataset,
	cpu: Cpu,
}

impl Component for AucTable {
	fn into_node(self) -> Node {
		let modelfox_entry = benchmark_data(self.cpu, self.dataset, Library::ModelFox);
		ui::Table::new()
			.width("100%".to_owned())
			.child(
				ui::TableHeader::new().child(
					ui::TableRow::new()
						.child(ui::TableHeaderCell::new().child("Library"))
						.child(ui::TableHeaderCell::new().child("AUC"))
						.child(ui::TableHeaderCell::new().child("v. ModelFox")),
				),
			)
			.child(
				ui::TableBody::new().children(LIBRARIES.iter().cloned().map(|library| {
					let entry = benchmark_data(self.cpu, self.dataset, library);
					let color = if library == Library::ModelFox {
						Some(ui::colors::BLUE.to_owned())
					} else {
						None
					};
					let text_color = if library == Library::ModelFox {
						Some(ui::colors::FUN_TEXT.to_owned())
					} else {
						None
					};
					let auc = ui::format_float_with_digits(entry.metric, 4);
					let auc_delta = format_delta(entry.metric / modelfox_entry.metric);
					ui::TableRow::new()
						.color(color)
						.text_color(text_color)
						.child(ui::TableCell::new().child(library.to_string()))
						.child(ui::TableCell::new().child(auc))
						.child(ui::TableCell::new().child(auc_delta))
				})),
			)
			.into_node()
	}
}

struct MseChart {
	dataset: Dataset,
	cpu: Cpu,
}

impl Component for MseChart {
	fn into_node(self) -> Node {
		let chart_data: Vec<BarChartSeries> = LIBRARIES
			.iter()
			.cloned()
			.map(|library| {
				let dataset = self.dataset;
				let data = vec![BarChartPoint {
					label: dataset.to_string(),
					x: 0.0,
					y: Some(benchmark_data(self.cpu, dataset, library).metric),
				}];
				BarChartSeries {
					color: color_for_library(library).to_owned(),
					data,
					title: Some(library.to_string()),
				}
			})
			.collect();
		BarChart::new()
			.group_gap(10.0)
			.series(chart_data)
			.x_axis_title("Dataset".to_owned())
			.y_axis_title("MSE".to_owned())
			.y_min(0.0)
			.into_node()
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

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Dataset {
	#[serde(rename = "allstate")]
	Allstate,
	#[serde(rename = "flights")]
	Flights,
	#[serde(rename = "higgs")]
	Higgs,
}

impl std::str::FromStr for Dataset {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"allstate" => Ok(Dataset::Allstate),
			"flights" => Ok(Dataset::Flights),
			"higgs" => Ok(Dataset::Higgs),
			_ => Err(()),
		}
	}
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

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Cpu {
	#[serde(rename = "m1")]
	M1,
	#[serde(rename = "ryzen")]
	Ryzen,
}

impl std::str::FromStr for Cpu {
	type Err = ();
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"m1" => Ok(Cpu::M1),
			"ryzen" => Ok(Cpu::Ryzen),
			_ => Err(()),
		}
	}
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
enum Library {
	CatBoost,
	LightGbm,
	SkLearn,
	ModelFox,
	XgBoost,
}

const LIBRARIES: &[Library] = &[
	Library::CatBoost,
	Library::LightGbm,
	Library::SkLearn,
	Library::ModelFox,
	Library::XgBoost,
];

impl std::fmt::Display for Library {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Library::CatBoost => write!(f, "catboost"),
			Library::LightGbm => write!(f, "lightgbm"),
			Library::SkLearn => write!(f, "sklearn"),
			Library::ModelFox => write!(f, "modelfox"),
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
		(Cpu::Ryzen, Dataset::Allstate, Library::ModelFox) => BenchmarkEntry {
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
		(Cpu::Ryzen, Dataset::Flights, Library::ModelFox) => BenchmarkEntry {
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
		(Cpu::Ryzen, Dataset::Higgs, Library::ModelFox) => BenchmarkEntry {
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
		(Cpu::M1, Dataset::Allstate, Library::ModelFox) => BenchmarkEntry {
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
		(Cpu::M1, Dataset::Flights, Library::ModelFox) => BenchmarkEntry {
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
		(Cpu::M1, Dataset::Higgs, Library::ModelFox) => BenchmarkEntry {
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
		Library::ModelFox => ui::colors::BLUE,
		Library::CatBoost => ui::colors::RED,
		Library::LightGbm => ui::colors::PURPLE,
		Library::SkLearn => ui::colors::ORANGE,
		Library::XgBoost => ui::colors::GREEN,
	}
}
