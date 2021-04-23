use clap::Clap;
use notify::Watcher;
use std::{path::PathBuf, sync::mpsc::channel, time::Duration};
use tangram_error::Result;
use which::which;

#[derive(Clap)]
#[clap(
	setting = clap::AppSettings::TrailingVarArg,
 )]
pub struct Args {
	#[clap(arg_enum)]
	target: Target,
	args: Vec<String>,
}

#[derive(Clap)]
enum Target {
	#[clap(name = "app")]
	App,
	#[clap(name = "www")]
	Www,
}

pub fn main() -> Result<()> {
	let args = Args::parse();
	let workspace_dir = std::env::current_dir()?;
	let target_dir = workspace_dir.join("target");
	let target_wasm_dir = workspace_dir.join("target_wasm");
	let languages_dir = workspace_dir.join("languages");
	let watch_paths = vec![workspace_dir];
	let ignore_paths = vec![target_dir, target_wasm_dir, languages_dir];
	let (cmd, cmd_args) = match args.target {
		Target::App => {
			let cmd = which("cargo")?.as_os_str().to_str().unwrap().to_owned();
			let mut cmd_args = vec![
				"run".to_owned(),
				"--bin".to_owned(),
				"tangram".to_owned(),
				"--".to_owned(),
				"app".to_owned(),
			];
			cmd_args.extend(args.args);
			(cmd, cmd_args)
		}
		Target::Www => {
			let cmd = which("cargo")?.as_os_str().to_str().unwrap().to_owned();
			let mut cmd_args = vec![
				"run".to_owned(),
				"--bin".to_owned(),
				"tangram_www".to_owned(),
				"--".to_owned(),
			];
			cmd_args.extend(args.args);
			(cmd, cmd_args)
		}
	};
	watch(watch_paths, ignore_paths, cmd, cmd_args)?;
	Ok(())
}

pub fn watch(
	watch_paths: Vec<PathBuf>,
	ignore_paths: Vec<PathBuf>,
	cmd: String,
	args: Vec<String>,
) -> Result<()> {
	let mut process = ChildProcess::new(cmd, args);
	process.start()?;
	let (tx, rx) = channel();
	let mut watcher = notify::watcher(tx, Duration::from_secs_f32(0.1)).unwrap();
	for path in watch_paths.iter() {
		watcher.watch(path, notify::RecursiveMode::Recursive)?;
	}
	loop {
		let event = rx.recv()?;
		let paths = match event {
			notify::DebouncedEvent::NoticeWrite(path) => vec![path],
			notify::DebouncedEvent::NoticeRemove(path) => vec![path],
			notify::DebouncedEvent::Create(path) => vec![path],
			notify::DebouncedEvent::Write(path) => vec![path],
			notify::DebouncedEvent::Chmod(path) => vec![path],
			notify::DebouncedEvent::Remove(path) => vec![path],
			notify::DebouncedEvent::Rename(path_a, path_b) => vec![path_a, path_b],
			notify::DebouncedEvent::Rescan => Vec::new(),
			notify::DebouncedEvent::Error(_, path) => {
				path.map(|path| vec![path]).unwrap_or_else(Vec::new)
			}
		};
		let should_restart = paths.iter().any(|path| {
			!ignore_paths
				.iter()
				.any(|ignore_path| path.starts_with(ignore_path))
		});
		if should_restart {
			process.restart()?;
		}
	}
}

struct ChildProcess {
	cmd: String,
	args: Vec<String>,
	process: Option<std::process::Child>,
}

impl ChildProcess {
	pub fn new(cmd: String, args: Vec<String>) -> ChildProcess {
		ChildProcess {
			cmd,
			args,
			process: None,
		}
	}

	pub fn start(&mut self) -> Result<()> {
		let process = std::process::Command::new(&self.cmd)
			.args(&self.args)
			.spawn()?;
		self.process.replace(process);
		Ok(())
	}

	pub fn stop(&mut self) -> Result<()> {
		if let Some(mut process) = self.process.take() {
			process.kill()?;
			process.wait()?;
		}
		Ok(())
	}

	pub fn restart(&mut self) -> Result<()> {
		self.stop()?;
		self.start()?;
		Ok(())
	}
}

impl Drop for ChildProcess {
	fn drop(&mut self) {
		self.stop().unwrap();
	}
}
