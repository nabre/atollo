//! Antigravity's Google login kept inside the kit, with no trace left in the
//! system's credential store.
//!
//! `agy` stores its login with go-keyring, with no option to use a file:
//! the Credential Manager on Windows, the Keychain on macOS, the Secret
//! Service on Linux. Around every `agy` run (`data\kit\agents\agy.cmd`, `agy`
//! on macOS and Linux):
//! - `restore`: the login saved in `data\antigravity\keyring.json` goes
//!   into the system's store (on Windows with "logon session" persistence:
//!   never on disk, gone at logoff); the Antigravity credentials already on
//!   the computer are recorded in `keyring-before.json`, their secrets
//!   sealed for this computer (DPAPI for the current Windows user; on macOS
//!   and Linux with a key kept in the Keychain or the Secret Service only
//!   while the file exists), unreadable anywhere else; the file is deleted
//!   at the next save;
//! - `save`: credentials created or changed during the run go to the kit and
//!   are deleted from the system; the computer's own ones, if agy overwrote
//!   or removed them, are put back as they were. Skipped while an agy of the
//!   kit runs (the last one to end saves).
//!
//! The app saves as well at start and when an agy of the kit ends, so a
//! window closed abruptly does not leave the login on the computer.
//!
//! macOS and Linux (D-115): agy uses go-keyring's service `gemini` and user
//! `antigravity` (the Windows target `gemini:antigravity`; see [`KNOWN`]),
//! so `keyring.json` is the same file on every system. On macOS the items
//! are read and written with `/usr/bin/security`, as go-keyring does, so
//! that agy reads them without a Keychain prompt; on Linux through the
//! Secret Service, into its `login` collection (the only one go-keyring
//! reads). Fail closed: a store that exists but cannot be used (locked,
//! denied, no `login` collection), or a sealing key that cannot be read or
//! created, moves nothing and writes no secret of the computer into the
//! kit. Only on
//! Linux with no Secret Service at all (WSL, minimal desktops), and only on
//! a disk that really keeps the 600 mode, the sign-in stays unencrypted in
//! the kit (keyring.json, 600), with a warning; on a disk without
//! permissions (exFAT, FAT) the kit refuses. Only the names the store can
//! list back ([`KNOWN`]) are moved; any other name in keyring.json (saved
//! on Windows) stays there untouched. The "before" files carry a tag of the
//! machine's id and the user's uid on macOS and Linux ([`machine_tag`]), so
//! a kit moved after a crash never mixes two computers' (or two accounts')
//! credentials, and a host name changed by the network does not orphan
//! them.
//!
//! Here the credentials, the trait of the store and the steps around a
//! run; the kit's files in `files`, its lock in `lock`, the save and the
//! restore in `moves`, the parts every system tests in `portable`. The
//! store of each system implements [`Store`] for [`System`] in `os/`:
//! the Credential Manager in `os/windows/agy_store.rs`, the Keychain and
//! the Secret Service in `os/unix/agy_store.rs` (over
//! `os/macos/agy_keyring.rs` and `os/linux/agy_keyring.rs`).

pub(crate) mod files;
mod lock;
mod moves;
mod portable;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::kit::Kit;
use files::{files, read};
use lock::Lock;
use moves::{mode_after_600, restore_checked, save_login};

/// "Logon session" persistence: never on disk, gone at logoff (Windows'
/// `CRED_PERSIST_SESSION`, checked in `os/windows/agy_store.rs`; the other
/// stores ignore it).
pub(crate) const PERSIST_SESSION: u32 = 1;

/// One credential. `blob`: base64 of the secret (as in the kit's files).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    pub target: String,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub blob: Option<String>,
    /// FILETIME of the last write on Windows; a fingerprint of the secret
    /// on macOS and Linux (only in keyring-before.json).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub written: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persist: Option<u32>,
    /// Restored by the kit (true) or the computer's own (false).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kit: Option<bool>,
}

/// Never the secret in a log or a panic message.
impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Item")
            .field("target", &self.target)
            .field("user", &self.user)
            .field("blob", &self.blob.as_ref().map(|_| "<secret>"))
            .field("written", &self.written)
            .field("persist", &self.persist)
            .field("kit", &self.kit)
            .finish()
    }
}

/// go-keyring names `<service>:<user>`: one part starts with jetski
/// (Antigravity's code name) or antigravity. The store of Windows lists by
/// it; tested on every system.
#[allow(dead_code)]
pub fn is_antigravity(target: &str) -> bool {
    let lower = target.to_ascii_lowercase();
    let Some((service, user)) = lower.split_once(':') else {
        return false;
    };
    let starts = |s: &str| s.starts_with("jetski") || s.starts_with("antigravity");
    starts(service) || starts(user)
}

// Used by the stores of macOS and Linux in `os/`.
#[allow(unused_imports)]
pub use portable::*;

/// The prefix of a secret sealed with DPAPI (Windows).
pub(crate) const DPAPI: &str = "dpapi:";

/// Where agy keeps its login on this system. Behind a trait so that the
/// moves in and out of the kit are tested with a fake store.
pub trait Store {
    /// agy's credentials in the store now. A [`NoStore`] error: no store
    /// at all on this system (Linux without a Secret Service); any other
    /// error: a store that cannot be used now (locked, denied).
    fn list(&self) -> Result<Vec<Item>>;
    fn write(&self, item: &Item, persist: u32) -> Result<()>;
    /// A missing credential is not an error.
    fn delete(&self, target: &str);
    /// A secret of this computer sealed for keyring-before.json.
    fn protect(&self, blob: &str) -> Result<String>;
    /// The secret saved by `protect` (a blob without a prefix as it is).
    fn unprotect(&self, blob: &str) -> Result<String>;
    /// keyring-before.json is gone: a sealing key kept for it goes too.
    fn forget_key(&self) {}
    /// `target` is one `list` can return. A kit credential it cannot list
    /// is never moved in: the save would not see it and it would stay in
    /// the store. Windows: every Antigravity name is listed.
    fn listable(&self, target: &str) -> bool {
        let _ = target;
        true
    }
    /// A restore that fails partway deletes the kit's credentials it had
    /// already written (macOS and Linux; on Windows they are left as they
    /// are, as earlier kits did).
    fn undo_partial(&self) -> bool {
        false
    }
}

/// The store of this system (`Store`, implemented in `os/`).
pub struct System {
    /// The user of the kit's sealing key (macOS and Linux).
    #[allow(dead_code)]
    pub(crate) seal_user: String,
    /// [`SEAL_ID_FILE`] and the id it holds (or will hold).
    #[allow(dead_code)]
    pub(crate) seal_id: (PathBuf, String),
}

impl System {
    pub fn of(kit: &Kit) -> System {
        let file = kit.data().join("antigravity").join(SEAL_ID_FILE);
        let id = seal_id(fs::read_to_string(&file).ok().as_deref(), &kit.id());
        System {
            seal_user: format!("antigravity-before-{id}"),
            seal_id: (file, id),
        }
    }
}

/// An `agy.exe` (`agy` on macOS and Linux) of this kit is running.
pub fn kit_agy_running(kit: &Kit) -> bool {
    crate::platform::processes::running_under(&kit.apps())
        .iter()
        .any(|p| p.name.eq_ignore_ascii_case("agy.exe") || p.name == "agy")
}

/// Saves the login of an agy run that ended (or did not end cleanly).
pub fn save(kit: &Kit) -> Result<()> {
    save_with(kit, &System::of(kit))
}

pub fn save_with(kit: &Kit, store: &dyn Store) -> Result<()> {
    let f = files(kit);
    if !f.seen.exists() {
        return Ok(());
    }
    let _lock = Lock::take(kit)?;
    save_login(kit, &f, store)
}

/// After an agy run (`agy.cmd`): the "starting" mark of this run is
/// dropped when no agy of the kit runs any more, then the login is saved.
pub fn save_after_run(kit: &Kit) -> Result<()> {
    save_after_run_with(kit, &System::of(kit))
}

pub fn save_after_run_with(kit: &Kit, store: &dyn Store) -> Result<()> {
    let f = files(kit);
    if !f.seen.exists() {
        return Ok(());
    }
    let _lock = Lock::take(kit)?;
    if !kit_agy_running(kit) {
        let _ = fs::remove_file(&f.starting);
    }
    save_login(kit, &f, store)
}

/// Something still to save (an agy run that did not end cleanly).
pub fn pending(kit: &Kit) -> bool {
    files(kit).seen.exists()
}

/// Which pending run: the latest change of its "before" file and of its
/// "starting" mark (a new agy run touches the mark), `None` when nothing
/// is pending. The watcher's retries are counted per value (BUG-V6-17).
pub fn pending_stamp(kit: &Kit) -> Option<std::time::SystemTime> {
    let f = files(kit);
    let changed = |p: &Path| fs::metadata(p).and_then(|m| m.modified()).ok();
    let seen = changed(&f.seen)?;
    Some(changed(&f.starting).map_or(seen, |s| s.max(seen)))
}

/// Before agy starts: the kit's login into the system's store.
pub fn restore(kit: &Kit) -> Result<()> {
    restore_with(kit, &System::of(kit))
}

/// Told when the system has no store for agy (Linux without a Secret
/// Service) and the kit's disk keeps the 600 mode: nothing is moved.
pub const NO_STORE: &str = "no Secret Service on this system: the Antigravity sign-in is stored UNENCRYPTED in the kit (data/antigravity/keyring.json, owner-only 600): whoever copies the kit gets it";

/// Told when there is no store and the kit's disk does not keep the 600
/// mode (exFAT, FAT, NTFS through FUSE), or on macOS.
pub const NO_STORE_REFUSED: &str = "no Secret Service on this system and the kit's disk does not keep owner-only permissions (600): the kit does not move the Antigravity sign-in; anything agy stores here is readable by whoever can read or copy the kit";

/// What the kit does without a store: the unencrypted fallback of D-115
/// only on Linux, and only on a disk that really keeps the 600 mode
/// (`mode`: the permissions read back after setting 600).
pub fn plaintext_allowed(os: crate::kit::Os, mode: Option<u32>) -> bool {
    os.is_linux() && mode.map(|m| m & 0o777) == Some(0o600)
}

pub fn restore_with(kit: &Kit, store: &dyn Store) -> Result<()> {
    restore_checked(kit, store, crate::kit::Os::current(), mode_after_600)
}

/// The secrets (base64) of the login saved in `dir` (`data\antigravity`),
/// read by `stats.rs` for the quotas.
pub fn saved_secrets(dir: &std::path::Path) -> Vec<String> {
    read(&dir.join("keyring.json"))
        .into_iter()
        .filter_map(|i| i.blob)
        .collect()
}

/// Logged in with a Google account (a saved login in the kit).
pub fn has_login(kit: &Kit) -> bool {
    !read(&files(kit).store).is_empty()
}

#[cfg(test)]
#[path = "../../../../test/rust/agents/keyring/mod.rs"]
pub(crate) mod tests;
