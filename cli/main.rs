//! This module contains the main entrypoint to the tangram cli.

use clap::Clap;
use colored::Colorize;
use std::path::PathBuf;

#[cfg(feature = "app")]
mod app;
#[cfg(feature = "app")]
mod migrate;
#[cfg(feature = "train")]
mod predict;
#[cfg(feature = "train")]
mod train;

#[derive(Clap)]
#[clap(
	version = env!("CARGO_PKG_VERSION"),
	about = "Train and deploy a machine learning model in minutes.",
	setting = clap::AppSettings::DisableHelpSubcommand,
)]
enum Args {
	#[cfg(feature = "train")]
	#[clap(name = "train")]
	Train(Box<TrainArgs>),
	#[cfg(feature = "train")]
	#[clap(name = "predict")]
	Predict(Box<PredictArgs>),
	#[cfg(feature = "app")]
	#[clap(name = "app")]
	App(Box<AppArgs>),
	#[cfg(feature = "app")]
	#[clap(name = "migrate")]
	Migrate(Box<MigrateArgs>),
}

#[cfg(feature = "train")]
#[derive(Clap)]
#[clap(
	about = "Train a model.",
	long_about = "Train a model from a csv file."
)]
pub struct TrainArgs {
	#[clap(
		short,
		long,
		about = "the path to your .csv file",
		conflicts_with_all=&["file-train", "file-test"],
	)]
	file: Option<PathBuf>,
	#[clap(
		long,
		about = "the path to your .csv file used for training",
		requires = "file-test"
	)]
	file_train: Option<PathBuf>,
	#[clap(
		long,
		about = "the path to your .csv file used for testing",
		requires = "file-train"
	)]
	file_test: Option<PathBuf>,
	#[clap(short, long, about = "the name of the column to predict")]
	target: String,
	#[clap(short, long, about = "the path to a config file")]
	config: Option<PathBuf>,
	#[clap(short, long, about = "the path to write the .tangram file to")]
	output: Option<PathBuf>,
	#[clap(
		long = "no-progress",
		about = "disable the cli progress view",
		parse(from_flag = std::ops::Not::not),
	)]
	progress: bool,
}

#[cfg(feature = "train")]
#[derive(Clap)]
#[clap(
	about = "Make predictions with a model.",
	long_about = "Make predictions with a model on the command line from a csv file."
)]
pub struct PredictArgs {
	#[clap(
		short,
		long,
		about = "the path to read examples from, defaults to stdin"
	)]
	file: Option<PathBuf>,
	#[clap(short, long, about = "the path to the model to make predictions with")]
	model: PathBuf,
	#[clap(
		short,
		long,
		about = "the path to write the predictions to, defaults to stdout"
	)]
	output: Option<PathBuf>,
	#[clap(
		short,
		long,
		about = "output probabilities instead of class labels, only relevant for classifier models"
	)]
	probabilities: Option<bool>,
}

#[cfg(feature = "app")]
#[derive(Clap)]
#[clap(about = "Run the app.", long_about = "Run the app.")]
pub struct AppArgs {
	#[clap(short, long = "config")]
	config: Option<PathBuf>,
}

#[cfg(feature = "app")]
#[derive(Clap)]
#[clap(
	about = "Migrate your app database.",
	long_about = "Migrate your app database to the latest version."
)]
pub struct MigrateArgs {
	#[clap(long, env = "DATABASE_URL")]
	database_url: Option<String>,
}

fn main() {
	let args = Args::parse();
	let result = match args {
		#[cfg(feature = "train")]
		Args::Train(args) => self::train::train(*args),
		#[cfg(feature = "train")]
		Args::Predict(args) => self::predict::predict(*args),
		#[cfg(feature = "app")]
		Args::App(args) => self::app::app(*args),
		#[cfg(feature = "app")]
		Args::Migrate(args) => self::migrate::migrate(*args),
	};
	if let Err(error) = result {
		eprintln!("{}: {}", "error".red().bold(), error);
		std::process::exit(1);
	}
}
