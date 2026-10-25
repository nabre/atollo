//! Where the command line's output goes on macOS and Linux
//! (`platform::console`).

use std::io::{self, Write};

/// A Unix program always has its standard output.
pub fn output() -> Box<dyn Write> {
    Box::new(io::stdout())
}
