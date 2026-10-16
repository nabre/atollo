//! The lines each source offers, kept in `data\app\version-lines.json`,
//! and whether a chosen line has to be downloaded.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::{check, find, is_present, valid_line};
use crate::kit::Kit;

/// The lines a source offered for a component, and when (Unix seconds).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offered {
    pub lines: Vec<String>,
    pub checked: u64,
}

pub(crate) fn offered_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("version-lines.json")
}

/// Every component's offered lines (only valid ones, newest first).
pub fn offered(kit: &Kit) -> BTreeMap<String, Offered> {
    let mut all: BTreeMap<String, Offered> =
        crate::fsx::read_json(&offered_file(kit)).unwrap_or_default();
    all.retain(|id, _| find(id).is_some());
    for o in all.values_mut() {
        o.lines.retain(|l| valid_line(l));
        o.lines
            .sort_by_key(|l| std::cmp::Reverse(crate::install::version_key(l)));
        o.lines.dedup();
    }
    all
}

/// How long a list of lines is good for (as on the Tools page).
const STALE_SECS: u64 = 24 * 60 * 60;

/// The versioned components whose lines can be downloaded and whose list is
/// missing or older than a day.
pub fn stale_offers(kit: &Kit) -> Vec<String> {
    let all = offered(kit);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    super::versioned()
        .filter(|c| crate::install::installer(&c.id).is_some_and(|i| i.has_lines()))
        .filter(|c| {
            all.get(&c.id)
                .is_none_or(|o| now.saturating_sub(o.checked) > STALE_SECS)
        })
        .map(|c| c.id.clone())
        .collect()
}

/// Keeps what the source of `id` offers now.
pub fn save_offered(kit: &Kit, id: &str, lines: &[String]) -> Result<()> {
    // Lists of several components can be asked for at the same time.
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut all = offered(kit);
    let checked = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    all.insert(
        id.to_owned(),
        Offered {
            lines: lines.iter().filter(|l| valid_line(l)).cloned().collect(),
            checked,
        },
    );
    crate::fsx::write_json(&offered_file(kit), &all)
}

/// `line` of `id` has to be downloaded before it can be used: the component
/// is installed, the line is not on disk and its installer can fetch it.
pub fn to_download(kit: &Kit, id: &str, line: &str) -> Result<bool> {
    check(id, Some(line))?;
    let Some(c) = find(id) else { return Ok(false) };
    if !c.is_installed(kit) || is_present(kit, c, line) {
        return Ok(false);
    }
    if !crate::install::installer(id).is_some_and(|i| i.has_lines()) {
        bail!("{id} {line}: not saved and it cannot be downloaded");
    }
    Ok(true)
}

#[cfg(test)]
#[path = "../../../test/rust/versions/offered.rs"]
mod tests;
