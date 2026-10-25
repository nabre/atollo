//! Authenticode signatures (`platform::signature`): macOS and Linux cannot
//! check them.

use std::path::Path;

use anyhow::Result;

use crate::platform::signature::Verdict;

/// Not checked here.
pub fn check(_path: &Path, _publisher: &str, _required: bool) -> Result<Option<Verdict>> {
    Ok(None)
}
