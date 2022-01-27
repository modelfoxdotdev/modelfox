pub use ctrl_c::*;

#[cfg(unix)]
mod ctrl_c {

	use anyhow::Result;
	use modelfox_kill_chip::KillChip;

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
	use modelfox_kill_chip::KillChip;
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
