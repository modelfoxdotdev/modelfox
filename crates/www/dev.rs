use std::path::PathBuf;

#[tokio::main]
async fn main() {
	watchserve::run(watchserve::Config {
		host: "0.0.0.0".parse().unwrap(),
		port: 8080,
		child_host: "0.0.0.0".parse().unwrap(),
		child_port: 8081,
		watch_paths: vec![PathBuf::from(".")],
		ignore_paths: vec![
			PathBuf::from("target"),
			PathBuf::from("target_check"),
			PathBuf::from("languages"),
		],
		command: "cargo run --bin tangram_www -- serve".to_owned(),
	})
	.await;
}
