//! The kit's files in `data\antigravity`: the sign-in, the "before" file and starting mark of a run.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::Item;
#[cfg(unix)]
use super::{host_tag, machine_tag};
use crate::kit::Kit;

pub(super) struct Files {
    pub(super) store: PathBuf,
    pub(super) seen: PathBuf,
    pub(super) starting: PathBuf,
}

/// The kit's files in `dir` (`data\antigravity`). `host`: the tag of this
/// computer and account on macOS and Linux ([`machine_tag`]), in the names
/// of the files of a run (Windows: `None`, the names of nabre/atollo).
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
#[cfg(unix)]
fn host() -> Option<String> {
    // SAFETY: getuid has no failure mode.
    let uid = unsafe { libc::getuid() };
    Some(match machine_id() {
        Some(id) => machine_tag(&id, uid),
        None => format!("{}-{uid}", host_tag(&host_name())),
    })
}

#[cfg(unix)]
fn host_name() -> String {
    let mut buf = [0u8; 256];
    // SAFETY: the buffer and its length; the name is NUL-terminated within.
    let ok = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) } == 0;
    let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
    if ok {
        String::from_utf8_lossy(&buf[..end]).into_owned()
    } else {
        String::new()
    }
}

/// macOS: `kern.uuid` (the hardware's IOPlatformUUID, the same `ioreg`
/// prints), fixed for the life of the Mac.
#[cfg(target_os = "macos")]
fn machine_id() -> Option<String> {
    let mut buf = [0u8; 64];
    let mut len = buf.len();
    // SAFETY: a NUL-terminated name, the buffer and its length; nothing is
    // written (new value null).
    let ok = unsafe {
        libc::sysctlbyname(
            c"kern.uuid".as_ptr(),
            buf.as_mut_ptr().cast(),
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    } == 0;
    let end = buf[..len.min(buf.len())]
        .iter()
        .position(|b| *b == 0)
        .unwrap_or(len.min(buf.len()));
    let id = std::str::from_utf8(&buf[..end]).ok()?.trim().to_owned();
    (ok && !id.is_empty()).then_some(id)
}

/// Linux: systemd's machine id, or D-Bus's copy of it.
#[cfg(all(unix, not(target_os = "macos")))]
fn machine_id() -> Option<String> {
    ["/etc/machine-id", "/var/lib/dbus/machine-id"]
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .map(|t| t.trim().to_owned())
        .find(|t| !t.is_empty())
}

#[cfg(windows)]
fn host() -> Option<String> {
    None
}

pub(super) fn files(kit: &Kit) -> Files {
    files_in(&kit.data().join("antigravity"), host().as_deref())
}

pub(super) fn read(path: &std::path::Path) -> Vec<Item> {
    crate::fsx::read_text(path)
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
pub(super) fn save_file(path: &std::path::Path, items: &[Item]) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(items)?;
    #[cfg(unix)]
    return crate::router::fs::write_private(path, &text)
        .with_context(|| path.display().to_string());
    #[cfg(not(unix))]
    crate::fsx::write_atomic(path, text.as_bytes()).with_context(|| path.display().to_string())
}
