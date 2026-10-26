//! The error of the commands: the short text the interface shows.
//!
//! It reaches the interface as a JSON string, as `Err(String)` did. Details
//! go to the log with [`fail`] or [`OrFail`]; `?` on an `anyhow::Error`
//! sends its whole chain (`{e:#}`) without logging it.
//!
//! A refusal of the levels (D-179, LEVELS.md §4.1) carries its reason and
//! its fix too, so the interface shows the right button without reading
//! the text: then it reaches the interface as `{ message, refusal, fix }`
//! (`RefusedError` of `ui/src/ipc/levels.ts`).

use std::fmt;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::levels::{Fix, Refusal};

/// A command's refusal or failure, as the text for the interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmdError {
    message: String,
    /// Set for a refusal of `levels::decide` only, with its fix.
    refusal: Option<(Refusal, Fix)>,
}

pub type CmdResult<T> = Result<T, CmdError>;

impl CmdError {
    /// The refusal `reason` of `levels::decide`, with its `fix`, said in
    /// `lang` (`levels::labels::refusal_text`).
    pub(crate) fn refused(reason: Refusal, fix: Fix, lang: &str) -> Self {
        Self {
            message: crate::levels::labels::refusal_text(&reason, lang),
            refusal: Some((reason, fix)),
        }
    }

    /// The reason and the fix of a refusal; `None` for any other error.
    #[cfg(test)]
    pub(crate) fn refusal(&self) -> Option<(&Refusal, Fix)> {
        self.refusal.as_ref().map(|(r, f)| (r, *f))
    }
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

/// The plain string as before for every error; `{ message, refusal, fix }`
/// for a refusal of the levels.
impl Serialize for CmdError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let Some((refusal, fix)) = &self.refusal else {
            return s.serialize_str(&self.message);
        };
        let mut out = s.serialize_struct("RefusedError", 3)?;
        out.serialize_field("message", &self.message)?;
        out.serialize_field("refusal", refusal)?;
        out.serialize_field("fix", fix)?;
        out.end()
    }
}

impl From<String> for CmdError {
    fn from(message: String) -> Self {
        Self {
            message,
            refusal: None,
        }
    }
}

impl From<&str> for CmdError {
    fn from(text: &str) -> Self {
        Self::from(text.to_owned())
    }
}

impl From<anyhow::Error> for CmdError {
    fn from(e: anyhow::Error) -> Self {
        Self::from(format!("{e:#}"))
    }
}

/// Logs `context: e` and gives `e` to the interface.
pub(super) fn fail(context: &str, e: impl fmt::Display) -> CmdError {
    crate::common::log::error(&format!("{context}: {e}"));
    CmdError::from(e.to_string())
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
