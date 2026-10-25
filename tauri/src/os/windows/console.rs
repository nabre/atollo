//! Where the command line's output goes on Windows (`platform::console`).

use std::fs::OpenOptions;
use std::io::{self, Write};

use windows::Win32::System::Console::{
    AttachConsole, GetStdHandle, ATTACH_PARENT_PROCESS, STD_OUTPUT_HANDLE,
};

/// The redirected handle, else the parent's console, else nowhere.
pub fn output() -> Box<dyn Write> {
    // SAFETY: plain handle query.
    let redirected =
        unsafe { GetStdHandle(STD_OUTPUT_HANDLE) }.is_ok_and(|h| !h.is_invalid() && !h.0.is_null());
    if redirected {
        return Box::new(io::stdout());
    }
    // SAFETY: attaches to the console of the parent (cmd, PowerShell) if any.
    if unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }.is_ok() {
        if let Ok(console) = OpenOptions::new().write(true).open("CONOUT$") {
            return Box::new(console);
        }
    }
    Box::new(io::sink())
}
