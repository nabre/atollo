//! The kit's files in `data\antigravity`: the sign-in, the "before" file and starting mark of a run.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::{host_tag, machine_tag, Item};
use crate::kit::Kit;

pub(crate) struct Files {
    pub(crate) store: PathBuf,
    pub(crate) seen: PathBuf,
    pub(crate) starting: PathBuf,
}

/// The kit's files in `dir` (`data\antigravity`). `host`: the tag of this
/// computer and account on macOS and Linux ([`machine_tag`]), in the names
/// of the files of a run (Windows: `None`, the names earlier kits used).
pub(super) fn files_in(dir: &Path, host: Option<&str>) -> Files {
    let (seen, starting) = match host {
        Some(h) => (
            format!("keyring-before-{h}.json"),
            format!("keyring-starting-{h}"),
        ),
        None => ("keyring-before.json".into(), "keyring-starting".into()),
    };
    Files {
        store: dir.join("keyring.json"),
        seen: dir.join(seen),
        starting: dir.join(starting),
    }
}

/// This computer and account ([`machine_tag`]); the host name and the uid
/// only where no machine id can be read (a Linux without systemd nor
/// D-Bus, which has no Secret Service either: nothing is moved there).
/// `None` on Windows, where no user id is (the names earlier kits used).
fn host() -> Option<String> {
    use crate::platform::host;
    let uid = host::user_id()?;
    Some(match host::machine_id() {
        Some(id) => machine_tag(&id, uid),
        None => format!("{}-{uid}", host_tag(&host::host_name())),
    })
}

pub(crate) fn files(kit: &Kit) -> Files {
    files_in(&kit.data().join("antigravity"), host().as_deref())
}

pub(crate) fn read(path: &std::path::Path) -> Vec<Item> {
    crate::common::fsx::read_text(path)
        .ok()
        .map(|t| t.trim().to_owned())
        .filter(|t| !t.is_empty())
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .map(|v| {
            if v.is_array() {
                v
            } else {
                serde_json::Value::Array(vec![v])
            }
        })
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// The kit's copy: on macOS and Linux owner-only (600, in a 700 folder),
/// since it holds the login in clear when no store can seal it.
pub(crate) fn save_file(path: &std::path::Path, items: &[Item]) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(items)?;
    crate::platform::files::write_secret(path, &text).with_context(|| path.display().to_string())
}
