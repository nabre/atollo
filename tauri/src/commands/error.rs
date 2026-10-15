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
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn serializes_as_the_plain_string() {
        let e = CmdError::from("Progetto non trovato: «x»\n\"y\"");
        assert_eq!(
            serde_json::to_string(&e).unwrap(),
            serde_json::to_string("Progetto non trovato: «x»\n\"y\"").unwrap()
        );
        // What Tauri sends for an `Err`: the same as for `Err(String)`.
        let sent = |e: tauri::ipc::InvokeError| e.0;
        assert_eq!(
            sent(CmdError::from("forbidden").into()),
            sent(String::from("forbidden").into())
        );
        assert_eq!(
            sent(CmdError::from("forbidden").into()),
            serde_json::json!("forbidden")
        );
    }

    #[test]
    fn conversions_keep_the_text() {
        assert_eq!(CmdError::from("a").to_string(), "a");
        assert_eq!(CmdError::from(String::from("b")).to_string(), "b");
        let e = anyhow::anyhow!("root").context("outer");
        assert_eq!(CmdError::from(e).to_string(), "outer: root");
        let q = || -> CmdResult<()> {
            Err(anyhow::anyhow!("disk full")).context("writing")?;
            Ok(())
        };
        assert_eq!(q().unwrap_err().to_string(), "writing: disk full");
    }

    #[test]
    fn or_fail_sends_the_short_or_whole_text() {
        let chain = || Err::<(), _>(anyhow::anyhow!("root").context("outer"));
        assert_eq!(chain().or_fail("x").unwrap_err().to_string(), "outer");
        assert_eq!(
            chain().or_fail_chain("x").unwrap_err().to_string(),
            "outer: root"
        );
        let io = Err::<(), _>(std::io::Error::other("denied"));
        assert_eq!(io.or_fail("x").unwrap_err().to_string(), "denied");
        assert_eq!(
            fail("ctx", "unknown server x").to_string(),
            "unknown server x"
        );
        assert_eq!(Ok::<u8, String>(1).or_fail("x"), Ok(1));
    }
}
