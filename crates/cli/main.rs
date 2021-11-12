//! This module contains the main entrypoint to the tangram cli.

use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;
use tracing_subscriber::prelude::*;

#[cfg(feature = "tangram_app")]
mod app;
#[cfg(feature = "tangram_app")]
mod migrate;
#[cfg(feature = "train")]
mod predict;
#[cfg(feature = "serve")]
mod serve;
#[cfg(feature = "train")]
mod train;

#[derive(Parser)]
#[clap(
	version = concat!(env!("CARGO_PKG_VERSION")),
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
	#[cfg(feature = "tangram_app")]
	#[clap(name = "app")]
	App(Box<AppArgs>),
	#[cfg(feature = "tangram_app")]
	#[clap(name = "migrate")]
	Migrate(Box<MigrateArgs>),
	#[cfg(feature = "serve")]
	#[clap(name = "serve")]
	Serve(Box<ServeArgs>),
}

#[cfg(feature = "train")]
#[derive(Parser)]
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
#[derive(Parser)]
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

#[cfg(feature = "tangram_app")]
#[derive(Parser)]
#[clap(about = "Run the app.", long_about = "Run the app.")]
pub struct AppArgs {
	#[clap(short, long = "config")]
	config: Option<PathBuf>,
}

#[cfg(feature = "tangram_app")]
#[derive(Parser)]
#[clap(
	about = "Migrate your app database.",
	long_about = "Migrate your app database to the latest version."
)]
pub struct MigrateArgs {
	#[clap(long, env = "DATABASE_URL")]
	database_url: Option<String>,
}

#[cfg(feature = "serve")]
#[derive(Parser)]
#[clap(
	about = "Serve predictions via HTTP",
	long_about = "Create HTTP server exposing an endpoint for running predictions against a Tangram model"
)]
pub struct ServeArgs {
	#[clap(
		short,
		long,
		default_value = "127.0.0.1",
		about = "Host IP at which to bind the server"
	)]
	address: String,
	#[clap(
		short,
		long,
		about = "Path to the `.tangram` file containing the model to serve"
	)]
	model: PathBuf,
	#[clap(short, long, default_value = "8080", about = "Port to listen on")]
	port: u16,
}

fn main() {
	setup_tracing();
	let args = Args::parse();
	let result = match args {
		#[cfg(feature = "train")]
		Args::Train(args) => self::train::train(*args),
		#[cfg(feature = "train")]
		Args::Predict(args) => self::predict::predict(*args),
		#[cfg(feature = "tangram_app")]
		Args::App(args) => self::app::app(*args),
		#[cfg(feature = "tangram_app")]
		Args::Migrate(args) => self::migrate::migrate(*args),
		#[cfg(feature = "serve")]
		Args::Serve(args) => self::serve::serve(*args),
	};
	if let Err(error) = result {
		eprintln!("{}: {}", "error".red().bold(), error);
		std::process::exit(1);
	}
}

fn setup_tracing() {
	std::env::set_var("RUST_BACKTRACE", "1");
	let env_layer = tracing_subscriber::EnvFilter::try_from_env("TANGRAM_TRACING");
	let env_layer = if cfg!(debug_assertions) {
		Some(env_layer.unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("[]=info")))
	} else {
		env_layer.ok()
	};
	if let Some(env_layer) = env_layer {
		if cfg!(debug_assertions) {
			let format_layer = tracing_subscriber::fmt::layer().pretty();
			let subscriber = tracing_subscriber::registry()
				.with(env_layer)
				.with(format_layer);
			subscriber.init();
		} else {
			let json_layer = tracing_subscriber::fmt::layer().json();
			let subscriber = tracing_subscriber::registry()
				.with(env_layer)
				.with(json_layer);
			subscriber.init();
		}
	}
}
