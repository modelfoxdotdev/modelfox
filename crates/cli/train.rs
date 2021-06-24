use crate::TrainArgs;
use anyhow::{anyhow, Result};
use backtrace::Backtrace;
use num::ToPrimitive;
use once_cell::sync::Lazy;
use std::{
	borrow::Cow,
	io::Write,
	path::{Path, PathBuf},
	sync::{
		mpsc::{channel, Receiver, RecvTimeoutError, Sender},
		Mutex,
	},
	thread::{spawn, JoinHandle},
	time::{Duration, Instant},
};
use tangram_core::progress::{
	LoadProgressEvent, ProgressEvent, StatsProgressEvent, TrainGridItemProgressEvent,
	TrainProgressEvent,
};
use tangram_progress_counter::ProgressCounter;
use tortoise::{
	style::Color,
	terminal::{Clear, Terminal},
};

#[cfg(feature = "train")]
pub fn train(args: TrainArgs) -> Result<()> {
	// Start the progress view if enabled and train the model. However, we need to do some extra work to make panic messages display properly. The problem is that progress is written to the terminal from another thread, which may conflict with the default panic hook. To work around this, we create a custom panic hook to store the panic message, wrap the progress view and training with `catch_unwind`, and then print the panic message if `catch_unwind` returns an `Err`. This ensures that the progress manager will be dropped before the panic message is displayed.
	static PANIC_MESSAGE_AND_BACKTRACE: Lazy<Mutex<Option<(String, Backtrace)>>> =
		Lazy::new(|| Mutex::new(None));
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(|panic_info| {
		let value = (panic_info.to_string(), Backtrace::new());
		PANIC_MESSAGE_AND_BACKTRACE.lock().unwrap().replace(value);
	}));
	let result = std::panic::catch_unwind(|| {
		let mut progress_thread = if args.progress {
			let terminal = Terminal::new()?;
			let progress_thread = ProgressThread::start(terminal);
			Some(progress_thread)
		} else {
			None
		};
		let mut handle_progress_event = |progress_event| {
			if let Some(progress_thread) = progress_thread.as_mut() {
				progress_thread.send_progress_event(progress_event)
			}
		};
		// Load the dataset, compute stats, and prepare for training.
		let mut trainer = tangram_core::train::Trainer::prepare(
			tangram_id::Id::generate(),
			args.file.as_deref(),
			args.file_train.as_deref(),
			args.file_test.as_deref(),
			&args.target,
			args.config.as_deref(),
			&mut handle_progress_event,
		)?;
		if let Some(progress_thread) = progress_thread.as_mut() {
			progress_thread.send_progress_event(ProgressEvent::Info(
				"Press ctrl-c to stop early and save the best model trained so far.".to_owned(),
			))
		}
		let mut handle_progress_event = |progress_event| {
			if let Some(progress_thread) = progress_thread.as_mut() {
				progress_thread.send_progress_event(progress_event)
			}
		};
		let kill_chip = unsafe { ctrl_c::register_ctrl_c_handler()? };
		let train_grid_item_outputs = trainer.train_grid(kill_chip, &mut handle_progress_event)?;
		unsafe { ctrl_c::unregister_ctrl_c_handler()? };
		if kill_chip.is_activated() {
			if let Some(progress_thread) = progress_thread.as_mut() {
				progress_thread.send_progress_event(ProgressEvent::Info(
					"Testing and writing the best model. Press ctrl-c again to kill the process immediately.".to_owned(),
				))
			}
		}
		let mut handle_progress_event = |progress_event| {
			if let Some(progress_thread) = progress_thread.as_mut() {
				progress_thread.send_progress_event(progress_event)
			}
		};
		let model =
			trainer.test_and_assemble_model(train_grid_item_outputs, &mut handle_progress_event)?;
		Ok(model)
	});
	std::panic::set_hook(hook);
	let model = match result {
		Ok(result) => result,
		Err(_) => {
			let panic_info = PANIC_MESSAGE_AND_BACKTRACE.lock().unwrap();
			let (message, backtrace) = panic_info.as_ref().unwrap();
			Err(anyhow!("{}\n{:?}", message, backtrace))
		}
	}?;

	// Retrieve the output path from the command line arguments or generate a default.
	let output_path = match args.output {
		Some(output) => output,
		None => {
			let dir = std::env::current_dir()?;
			let csv_file_name = args
				.file
				.as_ref()
				.unwrap_or_else(|| args.file_train.as_ref().unwrap())
				.file_stem()
				.unwrap()
				.to_str()
				.unwrap();
			available_path(&dir, csv_file_name, "tangram")?
		}
	};

	// Write the model to the output path.
	model.to_path(&output_path)?;

	// Announce that everything worked!
	eprintln!("Your model was written to {}.", output_path.display());
	eprintln!(
		"For help making predictions in your code, read the docs at https://www.tangram.xyz/docs."
	);
	eprintln!(
		"To learn more about how your model works and set up production monitoring, run `tangram app`."
	);

	Ok(())
}

pub struct ProgressThread {
	thread: Option<JoinHandle<Result<()>>>,
	sender: Option<Sender<Option<ProgressEvent>>>,
}

impl ProgressThread {
	pub fn start(terminal: Terminal) -> ProgressThread {
		let (sender, receiver) = channel::<Option<ProgressEvent>>();
		let thread = Some(spawn(move || progress_thread_main(terminal, receiver)));
		ProgressThread {
			thread,
			sender: Some(sender),
		}
	}

	pub fn send_progress_event(&mut self, progress_event: ProgressEvent) {
		self.sender
			.as_ref()
			.unwrap()
			.send(Some(progress_event))
			.unwrap();
	}
}

impl Drop for ProgressThread {
	fn drop(&mut self) {
		self.sender.take().unwrap().send(None).unwrap();
		self.thread.take().unwrap().join().unwrap().unwrap();
	}
}

struct ProgressThreadState {
	progress_bar: Option<ProgressBar>,
	timer: Option<Timer>,
}

fn progress_thread_main(
	mut terminal: Terminal,
	receiver: Receiver<Option<ProgressEvent>>,
) -> Result<()> {
	let mut state = ProgressThreadState {
		progress_bar: None,
		timer: None,
	};
	loop {
		enum RecieveResult {
			Event(ProgressEvent),
			Timeout,
		}
		let receive_result = if state.progress_bar.is_some() {
			match receiver.recv_timeout(Duration::from_millis(15)) {
				Ok(Some(progress_event)) => RecieveResult::Event(progress_event),
				Err(RecvTimeoutError::Timeout) => RecieveResult::Timeout,
				Ok(None) | Err(RecvTimeoutError::Disconnected) => break,
			}
		} else {
			match receiver.recv() {
				Ok(Some(progress_event)) => RecieveResult::Event(progress_event),
				Ok(None) | Err(_) => break,
			}
		};
		match receive_result {
			RecieveResult::Event(progress_event) => {
				progress_thread_handle_progress_event(&mut terminal, &mut state, progress_event)?;
				terminal.flush()?;
			}
			RecieveResult::Timeout => {
				let progress_bar = state.progress_bar.as_mut().unwrap();
				progress_bar.clear(&mut terminal)?;
				progress_bar.draw(&mut terminal)?;
				terminal.flush()?;
			}
		};
	}
	Ok(())
}

fn progress_thread_handle_progress_event(
	terminal: &mut Terminal,
	state: &mut ProgressThreadState,
	progress_event: ProgressEvent,
) -> Result<()> {
	match progress_event {
		ProgressEvent::Info(message) => {
			terminal.set_foreground_color(Color::Blue)?;
			terminal.set_bold()?;
			write!(terminal, "info: ")?;
			terminal.reset_style()?;
			writeln!(terminal, "{}", message)?;
		}
		ProgressEvent::Warning(message) => {
			terminal.set_foreground_color(Color::Yellow)?;
			terminal.set_bold()?;
			write!(terminal, "warning: ")?;
			terminal.reset_style()?;
			writeln!(terminal, "{}", message)?;
		}
		ProgressEvent::Load(progress_event) => match progress_event {
			LoadProgressEvent::Train(progress_event) => match progress_event {
				tangram_table::LoadProgressEvent::InferStarted(progress_counter) => {
					let progress_bar = ProgressBar::new(
						"🤔",
						"Inferring train table columns.".into(),
						progress_counter,
						ProgressValueFormatter::Bytes,
					);
					start_progress_bar(terminal, state, progress_bar)?;
				}
				tangram_table::LoadProgressEvent::InferDone => {
					finish_progress_bar(terminal, state)?;
				}
				tangram_table::LoadProgressEvent::LoadStarted(progress_counter) => {
					let progress_bar = ProgressBar::new(
						"🚚",
						"Loading train table.".into(),
						progress_counter,
						ProgressValueFormatter::Bytes,
					);
					start_progress_bar(terminal, state, progress_bar)?;
				}
				tangram_table::LoadProgressEvent::LoadDone => {
					finish_progress_bar(terminal, state)?;
				}
			},
			LoadProgressEvent::Test(progress_event) => match progress_event {
				tangram_table::LoadProgressEvent::InferStarted(progress_counter) => {
					let progress_bar = ProgressBar::new(
						"🤔",
						"Infer train table columns.".into(),
						progress_counter,
						ProgressValueFormatter::Bytes,
					);
					start_progress_bar(terminal, state, progress_bar)?;
				}
				tangram_table::LoadProgressEvent::InferDone => {
					finish_progress_bar(terminal, state)?;
				}
				tangram_table::LoadProgressEvent::LoadStarted(progress_counter) => {
					let progress_bar = ProgressBar::new(
						"🚚",
						"Loading test table.".into(),
						progress_counter,
						ProgressValueFormatter::Bytes,
					);
					start_progress_bar(terminal, state, progress_bar)?;
				}
				tangram_table::LoadProgressEvent::LoadDone => {
					finish_progress_bar(terminal, state)?;
				}
			},
			LoadProgressEvent::Shuffle => {
				start(terminal, state, "🎰 Shuffling.".into())?;
			}
			LoadProgressEvent::ShuffleDone => {
				finish(terminal, state, "✅ Shuffling.".into())?;
			}
		},
		ProgressEvent::Stats(progress_event) => match progress_event {
			StatsProgressEvent::ComputeTrainStats(progress_counter) => {
				let progress_bar = ProgressBar::new(
					"🧮",
					"Computing train stats.".into(),
					progress_counter,
					ProgressValueFormatter::Normal,
				);
				start_progress_bar(terminal, state, progress_bar)?;
			}
			StatsProgressEvent::ComputeTrainStatsDone => {
				finish_progress_bar(terminal, state)?;
			}
			StatsProgressEvent::ComputeTestStats(progress_counter) => {
				let progress_bar = ProgressBar::new(
					"🧮",
					"Computing test stats.".into(),
					progress_counter,
					ProgressValueFormatter::Normal,
				);
				start_progress_bar(terminal, state, progress_bar)?;
			}
			StatsProgressEvent::ComputeTestStatsDone => {
				finish_progress_bar(terminal, state)?;
			}
			StatsProgressEvent::Finalize => {
				start(terminal, state, "🎀 Finalizing stats.".into())?;
			}
			StatsProgressEvent::FinalizeDone => {
				finish(terminal, state, "✅ Finalizing stats.".into())?;
			}
		},
		ProgressEvent::ComputeBaselineMetrics(progress_counter) => {
			let progress_bar = ProgressBar::new(
				"🏁",
				"Computing baseline metrics.".into(),
				progress_counter,
				ProgressValueFormatter::Normal,
			);
			start_progress_bar(terminal, state, progress_bar)?;
		}
		ProgressEvent::ComputeBaselineMetricsDone => {
			finish_progress_bar(terminal, state)?;
		}
		ProgressEvent::Train(progress_event) => {
			let TrainProgressEvent {
				grid_item_index,
				grid_item_count,
				grid_item_progress_event,
			} = progress_event;
			match grid_item_progress_event {
				TrainGridItemProgressEvent::ComputeFeatures(progress_counter) => {
					let progress_bar = ProgressBar::new(
						"🚧",
						"Computing features.".into(),
						progress_counter,
						ProgressValueFormatter::Normal,
					);
					start_progress_bar(terminal, state, progress_bar)?;
				}
				TrainGridItemProgressEvent::ComputeFeaturesDone => {
					finish_progress_bar(terminal, state)?;
				}
				TrainGridItemProgressEvent::TrainModel(progress_event) => {
					match progress_event {
						tangram_core::progress::ModelTrainProgressEvent::Linear(progress_event) => {
							match progress_event {
								tangram_core::progress::LinearTrainProgressEvent::Train(
									progress_counter,
								) => {
									let title = format!(
										"Training model {} of {}.",
										grid_item_index + 1,
										grid_item_count
									)
									.into();
									let progress_bar = ProgressBar::new(
										"🚂",
										title,
										progress_counter,
										ProgressValueFormatter::Normal,
									);
									start_progress_bar(terminal, state, progress_bar)?;
								}
								tangram_core::progress::LinearTrainProgressEvent::TrainDone => {
									finish_progress_bar(terminal, state)?;
								}
							}
						}
						tangram_core::progress::ModelTrainProgressEvent::Tree(progress_event) => {
							match progress_event {
								tangram_core::progress::TreeTrainProgressEvent::Initialize(
									progress_counter,
								) => {
									let title = format!(
										"Preparing model {} of {}.",
										grid_item_index + 1,
										grid_item_count
									)
									.into();
									let progress_bar = ProgressBar::new(
										"🔌",
										title,
										progress_counter,
										ProgressValueFormatter::Normal,
									);
									start_progress_bar(terminal, state, progress_bar)?;
								}
								tangram_core::progress::TreeTrainProgressEvent::InitializeDone => {
									finish_progress_bar(terminal, state)?;
								}
								tangram_core::progress::TreeTrainProgressEvent::Train(
									progress_counter,
								) => {
									let title = format!(
										"Training model {} of {}.",
										grid_item_index + 1,
										grid_item_count
									)
									.into();
									let progress_bar = ProgressBar::new(
										"🚂",
										title,
										progress_counter,
										ProgressValueFormatter::Normal,
									);
									start_progress_bar(terminal, state, progress_bar)?;
								}
								tangram_core::progress::TreeTrainProgressEvent::TrainDone => {
									finish_progress_bar(terminal, state)?;
								}
							}
						}
					};
				}
				TrainGridItemProgressEvent::ComputeModelComparisonMetrics(progress_event) => {
					match progress_event {
						tangram_core::progress::ModelTestProgressEvent::ComputeFeatures(
							progress_counter,
						) => {
							let progress_bar = ProgressBar::new(
								"🏭️",
								"Computing model comparison features.".into(),
								progress_counter,
								ProgressValueFormatter::Normal,
							);
							start_progress_bar(terminal, state, progress_bar)?;
						}
						tangram_core::progress::ModelTestProgressEvent::ComputeFeaturesDone => {
							finish_progress_bar(terminal, state)?;
						}
						tangram_core::progress::ModelTestProgressEvent::Test(progress_counter) => {
							let progress_bar = ProgressBar::new(
								"️⚖️",
								"Computing model comparison metric.".into(),
								progress_counter,
								ProgressValueFormatter::Normal,
							);
							start_progress_bar(terminal, state, progress_bar)?;
						}
						tangram_core::progress::ModelTestProgressEvent::TestDone => {
							finish_progress_bar(terminal, state)?;
						}
					}
				}
			}
		}
		ProgressEvent::Test(progress_event) => match progress_event {
			tangram_core::progress::ModelTestProgressEvent::ComputeFeatures(progress_counter) => {
				let progress_bar = ProgressBar::new(
					"🏭️",
					"Computing test features.".into(),
					progress_counter,
					ProgressValueFormatter::Normal,
				);
				start_progress_bar(terminal, state, progress_bar)?;
			}
			tangram_core::progress::ModelTestProgressEvent::ComputeFeaturesDone => {
				finish_progress_bar(terminal, state)?;
			}
			tangram_core::progress::ModelTestProgressEvent::Test(progress_counter) => {
				let progress_bar = ProgressBar::new(
					"🔬️",
					"Testing.".into(),
					progress_counter,
					ProgressValueFormatter::Normal,
				);
				start_progress_bar(terminal, state, progress_bar)?;
			}
			tangram_core::progress::ModelTestProgressEvent::TestDone => {
				finish_progress_bar(terminal, state)?;
			}
		},
		ProgressEvent::Finalize => {
			start(terminal, state, "🏆 Finalizing the best model.".into())?;
		}
		ProgressEvent::FinalizeDone => {
			finish(terminal, state, "✅ Finalizing the best model.".into())?;
		}
	};
	Ok(())
}

fn start(
	terminal: &mut Terminal,
	state: &mut ProgressThreadState,
	title: Cow<'static, str>,
) -> Result<()> {
	state.timer = Some(Timer::start());
	writeln!(terminal, "{}", title)?;
	Ok(())
}

fn finish(
	terminal: &mut Terminal,
	state: &mut ProgressThreadState,
	title: Cow<'static, str>,
) -> Result<()> {
	let duration = state.timer.take().unwrap().stop();
	terminal.cursor_up(1)?;
	terminal.clear(Clear::FromCursorToEndOfScreen)?;
	writeln!(terminal, "{} {}", title, DisplayDuration(duration))?;
	Ok(())
}

fn start_progress_bar(
	terminal: &mut Terminal,
	state: &mut ProgressThreadState,
	mut progress_bar: ProgressBar,
) -> Result<()> {
	state.timer = Some(Timer::start());
	progress_bar.draw(terminal)?;
	state.progress_bar = Some(progress_bar);
	Ok(())
}

fn finish_progress_bar(terminal: &mut Terminal, state: &mut ProgressThreadState) -> Result<()> {
	let mut progress_bar = state.progress_bar.take().unwrap();
	let duration = state.timer.take().unwrap().stop();
	progress_bar.clear(terminal)?;
	writeln!(
		terminal,
		"✅ {} {}",
		progress_bar.title,
		DisplayDuration(duration)
	)?;
	Ok(())
}

#[derive(Debug)]
struct ProgressBar {
	emoji: &'static str,
	title: Cow<'static, str>,
	progress_counter: ProgressCounter,
	formatter: ProgressValueFormatter,
	start: Instant,
	last_change: Instant,
	last_value: u64,
}

#[derive(Copy, Clone, Debug)]
pub enum ProgressValueFormatter {
	Normal,
	Bytes,
}

impl ProgressBar {
	pub fn new(
		emoji: &'static str,
		title: Cow<'static, str>,
		progress_counter: ProgressCounter,
		formatter: ProgressValueFormatter,
	) -> ProgressBar {
		let start = Instant::now();
		let value = progress_counter.get();
		ProgressBar {
			emoji,
			title,
			progress_counter,
			formatter,
			start,
			last_change: start,
			last_value: value,
		}
	}

	fn draw(&mut self, terminal: &mut Terminal) -> Result<()> {
		let total = self.progress_counter.total();
		let value = self.progress_counter.get();
		if self.last_value != value {
			self.last_value = value;
			self.last_change = Instant::now();
		}
		let fraction = value.to_f64().unwrap() / total.to_f64().unwrap();
		write!(terminal, "{} {}", self.emoji, self.title)?;
		let elapsed = self.start.elapsed();
		let eta = if fraction > std::f64::EPSILON {
			let current_elapsed = self.last_change.duration_since(self.start);
			let current_elapsed_secs = current_elapsed.as_secs().to_f64().unwrap()
				+ current_elapsed.subsec_nanos().to_f64().unwrap() / 1_000_000_000f64;
			let eta = ((current_elapsed_secs / fraction) - current_elapsed_secs).floor();
			let eta = eta.to_u64().unwrap();
			Some(Duration::from_secs(eta))
		} else {
			None
		};
		write!(terminal, " ")?;
		match &self.formatter {
			ProgressValueFormatter::Normal => {
				write!(terminal, "{} / {}", value, total)?;
			}
			ProgressValueFormatter::Bytes => {
				let value = DisplayBytes(value);
				let total = DisplayBytes(total);
				write!(terminal, "{} / {}", value, total)?;
			}
		};
		write!(terminal, " {:.0}%", fraction * 100.0)?;
		write!(terminal, " {} elapsed", DisplayDuration(elapsed))?;
		if let Some(eta) = eta {
			write!(terminal, " {} remaining", DisplayDuration(eta))?;
		}
		writeln!(terminal)?;
		// Draw the bar.
		let term_width = terminal.size()?.1.to_usize().unwrap();
		let n_chars = term_width.min(PROGRESS_BAR_MAX_WIDTH) - 2;
		let (n_fill_chars, draw_arrow_char, n_empty_chars) = if fraction == 0.0 {
			(0, false, n_chars)
		} else if fraction >= 1.0 {
			(n_chars, false, 0)
		} else {
			let n_fill_chars = (fraction * n_chars.to_f64().unwrap())
				.floor()
				.to_usize()
				.unwrap();
			(n_fill_chars, true, n_chars - n_fill_chars - 1)
		};
		write!(terminal, "{}", LEFT_CHAR).unwrap();
		for _ in 0..n_fill_chars {
			write!(terminal, "{}", FILL_CHAR)?;
		}
		if draw_arrow_char {
			write!(terminal, "{}", ARROW_CHAR)?;
		}
		for _ in 0..n_empty_chars {
			write!(terminal, "{}", EMPTY_CHAR)?;
		}
		write!(terminal, "{}", RIGHT_CHAR)?;
		writeln!(terminal)?;
		Ok(())
	}

	pub fn clear(&mut self, terminal: &mut Terminal) -> Result<()> {
		terminal.cursor_up(2)?;
		terminal.clear(Clear::FromCursorToEndOfScreen)?;
		Ok(())
	}
}

const PROGRESS_BAR_MAX_WIDTH: usize = 80;
const LEFT_CHAR: char = '[';
const RIGHT_CHAR: char = ']';
const EMPTY_CHAR: char = ' ';
const FILL_CHAR: char = '=';
const ARROW_CHAR: char = '>';

pub struct DisplayBytes(pub u64);

impl std::fmt::Display for DisplayBytes {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let value = self.0;
		if value >= 1_000_000_000_000 {
			write!(f, "{}TB", value / 1_000_000_000_000)
		} else if value >= 1_000_000_000 {
			write!(f, "{}GB", value / 1_000_000_000)
		} else if value >= 1_000_000 {
			write!(f, "{}MB", value / 1_000_000)
		} else if value >= 1_000 {
			write!(f, "{}KB", value / 1_000)
		} else {
			write!(f, "{}B", value)
		}
	}
}

pub struct DisplayDuration(pub Duration);

impl std::fmt::Display for DisplayDuration {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let milliseconds = self.0.as_millis();
		let seconds = self.0.as_secs();
		let minutes = seconds / 60;
		let hours = seconds / (60 * 60);
		let days = seconds / (24 * 60 * 60);
		if days >= 1 {
			write!(
				f,
				"{}d {}h",
				days,
				(seconds - days * (24 * 60 * 60)) / (60 * 60)
			)
		} else if hours >= 1 {
			write!(f, "{}h {}m", hours, (seconds - hours * (60 * 60)) / 60)
		} else if minutes >= 1 {
			write!(f, "{}m {}s", minutes, (seconds - minutes * 60))
		} else if seconds >= 1 {
			write!(f, "{}s", seconds)
		} else if milliseconds >= 1 {
			write!(f, "0s {}ms", milliseconds)
		} else {
			write!(f, "0ms")
		}
	}
}

#[cfg(unix)]
mod ctrl_c {

	use anyhow::Result;
	use tangram_kill_chip::KillChip;

	static mut KILL_CHIP: KillChip = KillChip::new();

	unsafe extern "C" fn kill_chip_handler(_: u32) {
		let previous_value = KILL_CHIP.activate();
		if previous_value {
			libc::_exit(1);
		}
	}

	pub unsafe fn register_ctrl_c_handler() -> Result<&'static KillChip> {
		let res = libc::signal(libc::SIGINT, kill_chip_handler as libc::sighandler_t);
		if res == libc::SIG_ERR {
			return Err(std::io::Error::last_os_error().into());
		}
		Ok(&KILL_CHIP)
	}

	pub unsafe fn unregister_ctrl_c_handler() -> Result<()> {
		let res = libc::signal(libc::SIGINT, libc::SIG_DFL);
		if res == libc::SIG_ERR {
			return Err(std::io::Error::last_os_error().into());
		}
		Ok(())
	}
}

#[cfg(windows)]
mod ctrl_c {

	use anyhow::Result;
	use tangram_kill_chip::KillChip;
	use winapi::{
		shared::minwindef::{BOOL, DWORD, FALSE, TRUE},
		um::{consoleapi::SetConsoleCtrlHandler, processthreadsapi::ExitProcess},
	};

	static mut KILL_CHIP: KillChip = KillChip::new();

	unsafe extern "system" fn kill_chip_handler(_: DWORD) -> BOOL {
		let previous_value = KILL_CHIP.activate();
		if previous_value {
			ExitProcess(1);
		}
		TRUE
	}

	pub unsafe fn register_ctrl_c_handler() -> Result<&'static KillChip> {
		let err = SetConsoleCtrlHandler(Some(kill_chip_handler), TRUE);
		if err == 0 {
			return Err(std::io::Error::last_os_error().into());
		}
		Ok(&KILL_CHIP)
	}

	pub unsafe fn unregister_ctrl_c_handler() -> Result<()> {
		let err = SetConsoleCtrlHandler(Some(kill_chip_handler), FALSE);
		if err == 0 {
			return Err(std::io::Error::last_os_error().into());
		}
		Ok(())
	}
}

pub struct Timer(Instant);

impl Timer {
	pub fn start() -> Timer {
		Timer(Instant::now())
	}

	pub fn stop(self) -> Duration {
		self.0.elapsed()
	}
}

/// This function checks if a file with the given name and extension already exists at the path `base`, and if it does, it appends " 1", " 2", etc. to it until it finds a name that will not overwrite an existing file.
fn available_path(dir: &Path, name: &str, extension: &str) -> Result<PathBuf> {
	let mut i = 0;
	loop {
		let mut path = PathBuf::from(dir);
		let mut filename = String::new();
		filename.push_str(name);
		if i > 0 {
			filename.push(' ');
			filename.push_str(&i.to_string());
		}
		filename.push('.');
		filename.push_str(extension);
		path.push(filename);
		match std::fs::metadata(&path) {
			// If a file at the path does not exist, return the path.
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
				return Ok(path);
			}
			Err(error) => return Err(error.into()),
			// If a file at the path exists, try the next number.
			Ok(_) => {
				i += 1;
				continue;
			}
		}
	}
}
