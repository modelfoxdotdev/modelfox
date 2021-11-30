use clap::{self, ArgEnum, Parser};

#[derive(Parser)]
struct Args {
	#[clap(long, arg_enum, multiple_values = true)]
	datasets: Vec<Dataset>,
	#[clap(long, arg_enum, multiple_values = true)]
	libraries: Vec<Library>,
}

#[derive(ArgEnum, Clone)]
enum Dataset {
	#[clap(name = "allstate")]
	Allstate,
	#[clap(name = "boston")]
	Boston,
	#[clap(name = "census")]
	Census,
	#[clap(name = "flights")]
	Flights,
	#[clap(name = "heart_disease")]
	HeartDisease,
	#[clap(name = "higgs")]
	Higgs,
	#[clap(name = "iris")]
	Iris,
}

impl std::fmt::Display for Dataset {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Dataset::Allstate => write!(f, "allstate"),
			Dataset::Boston => write!(f, "boston"),
			Dataset::Census => write!(f, "census"),
			Dataset::Flights => write!(f, "flights"),
			Dataset::HeartDisease => write!(f, "heart_disease"),
			Dataset::Higgs => write!(f, "higgs"),
			Dataset::Iris => write!(f, "iris"),
		}
	}
}

#[derive(ArgEnum, Clone, PartialEq)]
enum Library {
	#[clap(name = "pytorch")]
	PyTorch,
	#[clap(name = "sklearn")]
	SkLearn,
	#[clap(name = "tangram")]
	Tangram,
}

impl std::fmt::Display for Library {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Library::PyTorch => write!(f, "pytorch"),
			Library::SkLearn => write!(f, "sklearn"),
			Library::Tangram => write!(f, "tangram"),
		}
	}
}

#[derive(PartialEq, Eq)]
enum Task {
	Regression,
	BinaryClassification,
	MulticlassClassification,
}

impl Dataset {
	fn task(&self) -> Task {
		match self {
			Dataset::Allstate | Dataset::Boston => Task::Regression,
			Dataset::Census | Dataset::Flights | Dataset::HeartDisease | Dataset::Higgs => {
				Task::BinaryClassification
			}
			Dataset::Iris => Task::MulticlassClassification,
		}
	}
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum BenchmarkOutput {
	Regression(RegressionBenchmarkOutput),
	BinaryClassification(BinaryClassificationBenchmarkOutput),
	MulticlassClassification(MulticlassClassificationBenchmarkOutput),
}

impl BenchmarkOutput {
	fn memory(&mut self, memory: String) {
		match self {
			BenchmarkOutput::Regression(output) => {
				output.memory = Some(memory);
			}
			BenchmarkOutput::BinaryClassification(output) => {
				output.memory = Some(memory);
			}
			BenchmarkOutput::MulticlassClassification(output) => {
				output.memory = Some(memory);
			}
		}
	}
}

#[derive(serde::Deserialize, Debug)]
pub struct RegressionBenchmarkOutput {
	mse: f32,
	memory: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct BinaryClassificationBenchmarkOutput {
	auc_roc: f32,
	memory: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct MulticlassClassificationBenchmarkOutput {
	accuracy: f32,
	memory: Option<String>,
}

fn main() {
	let args = Args::parse();
	let regression_datasets = args
		.datasets
		.iter()
		.cloned()
		.filter(|dataset| dataset.task() == Task::Regression)
		.collect::<Vec<_>>();
	let binary_classification_datasets = args
		.datasets
		.iter()
		.cloned()
		.filter(|dataset| dataset.task() == Task::BinaryClassification)
		.collect::<Vec<_>>();
	let multiclass_classification_datasets = args
		.datasets
		.iter()
		.cloned()
		.filter(|dataset| dataset.task() == Task::MulticlassClassification)
		.collect::<Vec<_>>();

	// Build the tangram benchmarks.
	for dataset in args.datasets.iter() {
		build_tangram_benchmark(dataset);
	}

	// Test the regression datasets.
	if !regression_datasets.is_empty() {
		println!("# Regression");
		run_benchmarks(&args.libraries, &regression_datasets);
		println!();
	}

	// Test the binary classification datasets.
	if !binary_classification_datasets.is_empty() {
		println!("# Binary Classification");
		run_benchmarks(&args.libraries, &binary_classification_datasets);
		println!();
	}

	// Test the multiclass classification datasets.
	if !multiclass_classification_datasets.is_empty() {
		println!("# Multiclass Classification");
		run_benchmarks(&args.libraries, &multiclass_classification_datasets);
		println!();
	}
}

fn build_tangram_benchmark(dataset: &Dataset) {
	let status = std::process::Command::new("cargo")
		.arg("build")
		.arg("--release")
		.arg("-p")
		.arg(format!("tangram_linear_benchmark_{}", dataset))
		.spawn()
		.unwrap()
		.wait()
		.unwrap();
	assert!(status.success());
}

fn run_benchmarks(libraries: &[Library], datasets: &[Dataset]) {
	for dataset in datasets.iter() {
		println!("## {}", dataset);
		for library in libraries.iter() {
			let start = std::time::Instant::now();
			let output = run_benchmark(dataset, library);
			let duration = start.elapsed();
			match output {
				BenchmarkOutput::Regression(output) => println!(
					"library {} mse {} duration {:?} memory {:?}",
					library, output.mse, duration, output.memory
				),
				BenchmarkOutput::BinaryClassification(output) => println!(
					"library {} auc_roc {} duration {:?} memory {:?}",
					library, output.auc_roc, duration, output.memory
				),
				BenchmarkOutput::MulticlassClassification(output) => println!(
					"library {} accuracy {} duration {:?} memory {:?}",
					library, output.accuracy, duration, output.memory
				),
			};
		}
	}
}

fn run_benchmark(dataset: &Dataset, library: &Library) -> BenchmarkOutput {
	let output = match library {
		Library::Tangram => std::process::Command::new("time")
			.arg("-f")
			.arg("%M")
			.arg(format!(
				"target/release/tangram_linear_benchmark_{}",
				dataset
			))
			.output()
			.unwrap(),
		_ => std::process::Command::new("time")
			.arg("-f")
			.arg("%M")
			.arg("python")
			.arg(format!("crates/linear/benchmarks/{}.py", dataset))
			.arg("--library")
			.arg(format!("{}", library))
			.output()
			.unwrap(),
	};
	assert!(output.status.success());
	let stderr = String::from_utf8(output.stderr).unwrap();
	let memory = stderr.lines().last().unwrap().to_owned();
	let stdout = String::from_utf8(output.stdout).unwrap();
	let mut output: BenchmarkOutput = serde_json::from_str(stdout.lines().last().unwrap()).unwrap();
	output.memory(memory);
	output
}
