//! The error of the commands: the short text the interface shows.
//!
//! It reaches the interface as a JSON string, as `Err(String)` did. Details
//! go to the log with [`fail`] or [`OrFail`]; `?` on an `anyhow::Error`
//! sends its whole chain (`{e:#}`) without logging it.

use std::fmt;

use serde::{Serialize, Serializer};

/// A command's refusal or failure, as the text for the interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdError(String);

pub type CmdResult<T> = Result<T, CmdError>;

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for CmdError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl From<String> for CmdError {
    fn from(text: String) -> Self {
        Self(text)
    }
}

impl From<&str> for CmdError {
    fn from(text: &str) -> Self {
        Self(text.to_owned())
    }
}

impl From<anyhow::Error> for CmdError {
    fn from(e: anyhow::Error) -> Self {
        Self(format!("{e:#}"))
    }
}

/// Logs `context: e` and gives `e` to the interface.
pub(super) fn fail(context: &str, e: impl fmt::Display) -> CmdError {
    crate::log::error(&format!("{context}: {e}"));
    CmdError(e.to_string())
}

/// [`fail`] on the error of a `Result`, so a command can use `?`.
pub(super) trait OrFail<T> {
    /// The error as `{e}`.
    fn or_fail(self, context: &str) -> CmdResult<T>;
    /// The error as `{e:#}`: the whole chain of an `anyhow::Error`.
    fn or_fail_chain(self, context: &str) -> CmdResult<T>;
}

impl<T, E: fmt::Display> OrFail<T> for Result<T, E> {
    fn or_fail(self, context: &str) -> CmdResult<T> {
        self.map_err(|e| fail(context, e))
    }

    fn or_fail_chain(self, context: &str) -> CmdResult<T> {
        self.map_err(|e| fail(context, format!("{e:#}")))
    }
}

#[cfg(test)]
#[path = "../../../test/rust/ipc/error.rs"]
mod tests;
