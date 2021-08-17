use clap::Clap;
use std::path::PathBuf;

#[derive(Clap)]
pub struct Args {
	#[clap(long, default_value = "0.0.0.0")]
	host: std::net::IpAddr,
	#[clap(long, default_value = "8080")]
	port: u16,
	#[clap(long, default_value = "0.0.0.0")]
	child_host: std::net::IpAddr,
	#[clap(long, default_value = "8081")]
	child_port: u16,
	#[clap(long = "watch", multiple_occurrences = true)]
	watch_paths: Vec<PathBuf>,
	#[clap(long = "ignore", multiple_occurrences = true)]
	ignore_paths: Vec<PathBuf>,
	#[clap(long)]
	command: String,
}

#[tokio::main]
pub async fn main() {
	let Args {
		host,
		port,
		child_host,
		child_port,
		watch_paths,
		ignore_paths,
		command,
	} = Args::parse();
	let config = sunfish::watchserve::Config {
		host,
		port,
		child_host,
		child_port,
		watch_paths,
		ignore_paths,
		command,
	};
	sunfish::watchserve::run(config).await;
}
