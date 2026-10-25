//! Where the command line's output goes (`app/cli.rs`): the exe is a GUI
//! program on Windows, so there it goes to the redirected handle when there
//! is one, otherwise to the console of the parent (cmd, PowerShell);
//! elsewhere a program always has its standard output.

use std::io::Write;

/// Where command output goes.
pub fn output() -> Box<dyn Write> {
    crate::os::current::console::output()
}
