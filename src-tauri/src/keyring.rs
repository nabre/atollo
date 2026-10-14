//! Antigravity's Google login kept inside the kit, with no trace left in the
//! Windows Credential Manager (port of nabre/dev-kit `agy-keyring.ps1`).
//!
//! `agy` stores its login in the Credential Manager (go-keyring) with no
//! option to use a file. Around every `agy` run (`data\kit\bin\agy.cmd`):
//! - `restore`: the login saved in `data\antigravity\keyring.json` goes back
//!   into the Credential Manager with "logon session" persistence (never on
//!   disk, gone at logoff); the Antigravity credentials already on the PC are
//!   recorded in `keyring-before.json`;
//! - `save`: credentials created or changed during the run go to the kit and
//!   are deleted from Windows; the PC's own ones, if agy overwrote or removed
//!   them, are put back as they were. Skipped while an agy of the kit runs
//!   (the last one to end saves).
//!
//! The app saves as well at start and when an agy of the kit ends, so a
//! window closed abruptly does not leave the login in Windows.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, WAIT_ABANDONED, WAIT_OBJECT_0,
};
use windows::Win32::Security::Credentials::{
    CredDeleteW, CredEnumerateW, CredFree, CredWriteW, CREDENTIALW, CRED_FLAGS, CRED_PERSIST,
    CRED_PERSIST_SESSION, CRED_TYPE_GENERIC,
};
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};

use crate::kit::{write_atomic, Kit};

/// One credential. `blob`: base64 of the secret (as in the kit's files).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    pub target: String,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub blob: Option<String>,
    /// FILETIME of the last write (only in keyring-before.json).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub written: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persist: Option<u32>,
    /// Restored by the kit (true) or the PC's own (false).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kit: Option<bool>,
}

/// go-keyring names `<service>:<user>`: one part starts with jetski
/// (Antigravity's code name) or antigravity.
pub fn is_antigravity(target: &str) -> bool {
    let lower = target.to_ascii_lowercase();
    let Some((service, user)) = lower.split_once(':') else {
        return false;
    };
    let starts = |s: &str| s.starts_with("jetski") || s.starts_with("antigravity");
    starts(service) || starts(user)
}

fn list() -> Vec<Item> {
    let mut out = Vec::new();
    let mut count = 0u32;
    let mut creds: *mut *mut CREDENTIALW = std::ptr::null_mut();
    // SAFETY: CredEnumerateW allocates `creds`, freed with CredFree below;
    // every pointer read is inside that allocation.
    unsafe {
        if CredEnumerateW(PCWSTR::null(), None, &mut count, &mut creds).is_err() {
            return out;
        }
        for i in 0..count as usize {
            let c = &**creds.add(i);
            if c.Type != CRED_TYPE_GENERIC || c.TargetName.is_null() {
                continue;
            }
            let target = c.TargetName.to_string().unwrap_or_default();
            if !is_antigravity(&target) {
                continue;
            }
            let blob = if c.CredentialBlobSize > 0 && !c.CredentialBlob.is_null() {
                std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize).to_vec()
            } else {
                Vec::new()
            };
            let written = (i64::from(c.LastWritten.dwHighDateTime) << 32)
                | i64::from(c.LastWritten.dwLowDateTime);
            out.push(Item {
                target,
                user: (!c.UserName.is_null()).then(|| c.UserName.to_string().unwrap_or_default()),
                blob: Some(base64::engine::general_purpose::STANDARD.encode(blob)),
                written: Some(written),
                persist: Some(c.Persist.0),
                kit: None,
            });
        }
        CredFree(creds.cast());
    }
    out
}

fn write(item: &Item, persist: u32) -> Result<()> {
    let blob = base64::engine::general_purpose::STANDARD
        .decode(item.blob.as_deref().unwrap_or_default())
        .context("credential blob")?;
    let mut target: Vec<u16> = item.target.encode_utf16().chain(Some(0)).collect();
    let mut user: Vec<u16> = item
        .user
        .as_deref()
        .unwrap_or_default()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut blob = blob;
    let cred = CREDENTIALW {
        Flags: CRED_FLAGS(0),
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(target.as_mut_ptr()),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_mut_ptr(),
        Persist: CRED_PERSIST(persist),
        UserName: if item.user.is_some() {
            PWSTR(user.as_mut_ptr())
        } else {
            PWSTR::null()
        },
        ..Default::default()
    };
    // SAFETY: every buffer outlives the call.
    unsafe { CredWriteW(&cred, 0) }.with_context(|| format!("CredWrite {}", item.target))
}

fn delete(target: &str) {
    // SAFETY: valid string; a missing credential is not an error here.
    let _ = unsafe { CredDeleteW(&HSTRING::from(target), CRED_TYPE_GENERIC, None) };
}

struct Files {
    store: PathBuf,
    seen: PathBuf,
    starting: PathBuf,
}

fn files(kit: &Kit) -> Files {
    let dir = kit.data().join("antigravity");
    Files {
        store: dir.join("keyring.json"),
        seen: dir.join("keyring-before.json"),
        starting: dir.join("keyring-starting"),
    }
}

fn read(path: &std::path::Path) -> Vec<Item> {
    fs::read_to_string(path)
        .ok()
        .map(|t| t.trim_start_matches('\u{feff}').trim().to_owned())
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

fn save_file(path: &std::path::Path, items: &[Item]) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    write_atomic(path, serde_json::to_string_pretty(items)?.as_bytes())
        .with_context(|| path.display().to_string())
}

/// An `agy.exe` of this kit is running.
pub fn kit_agy_running(kit: &Kit) -> bool {
    crate::processes::running_under(&kit.apps())
        .iter()
        .any(|p| p.name.eq_ignore_ascii_case("agy.exe"))
}

/// The kit's lock: `Local\Dev-Kit-agy-keyring-<kit id>` (same as the kit).
struct Lock(windows::Win32::Foundation::HANDLE);

impl Lock {
    fn take(kit: &Kit) -> Result<Lock> {
        let name = HSTRING::from(format!(r"Local\Dev-Kit-agy-keyring-{}", kit.id()));
        // SAFETY: valid name; waits at most 30 s like the kit.
        unsafe {
            let h = CreateMutexW(None, false, &name).context("CreateMutexW")?;
            let _ = GetLastError() == ERROR_ALREADY_EXISTS;
            let r = WaitForSingleObject(h, 30_000);
            if r != WAIT_OBJECT_0 && r != WAIT_ABANDONED {
                crate::log::warn("antigravity keyring: lock busy for 30 s, going on");
            }
            Ok(Lock(h))
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        // SAFETY: handle from CreateMutexW; released and closed once.
        unsafe {
            let _ = ReleaseMutex(self.0);
            let _ = CloseHandle(self.0);
        }
    }
}

fn save_login(kit: &Kit, f: &Files) -> Result<()> {
    if !f.seen.exists() || kit_agy_running(kit) {
        return Ok(());
    }
    if let Ok(age) = fs::metadata(&f.starting)
        .and_then(|m| m.modified())
        .map(|t| t.elapsed().unwrap_or_default())
    {
        if age < Duration::from_secs(20) {
            return Ok(());
        }
        let _ = fs::remove_file(&f.starting);
    }
    let before = read(&f.seen);
    let now = list();
    let mut keep: Vec<Item> = read(&f.store);
    let mut to_delete = Vec::new();
    let mut to_restore = Vec::new();
    let find = |items: &[Item], t: &str| items.iter().find(|i| i.target == t).cloned();
    for c in &now {
        let old = find(&before, &c.target);
        // Existed before, unchanged, not restored by the kit: not ours.
        if old
            .as_ref()
            .is_some_and(|o| o.kit != Some(true) && o.written == c.written)
        {
            continue;
        }
        keep.retain(|k| k.target != c.target);
        keep.push(Item {
            target: c.target.clone(),
            user: c.user.clone(),
            blob: c.blob.clone(),
            written: None,
            persist: None,
            kit: None,
        });
        match old {
            Some(o) if o.kit != Some(true) => to_restore.push(o),
            _ => to_delete.push(c.target.clone()),
        }
    }
    for o in before.iter().filter(|o| o.kit != Some(true)) {
        if !now.iter().any(|c| c.target == o.target) {
            to_restore.push(o.clone());
        }
    }
    // Signed out inside agy: a kit credential no longer there is forgotten.
    keep.retain(|k| {
        !before.iter().any(|o| {
            o.target == k.target && o.kit == Some(true) && !now.iter().any(|c| c.target == k.target)
        })
    });
    // The kit's copy first; Windows is cleaned only once it is written.
    save_file(&f.store, &keep)?;
    for t in to_delete {
        delete(&t);
    }
    for o in to_restore {
        write(&o, o.persist.unwrap_or(CRED_PERSIST_SESSION.0))?;
    }
    fs::remove_file(&f.seen)?;
    Ok(())
}

/// Saves the login of an agy run that ended (or did not end cleanly).
pub fn save(kit: &Kit) -> Result<()> {
    let f = files(kit);
    if !f.seen.exists() {
        return Ok(());
    }
    let _lock = Lock::take(kit)?;
    save_login(kit, &f)
}

/// After an agy run (`agy.cmd`): the "starting" mark of this run is
/// dropped when no agy of the kit runs any more, then the login is saved.
/// (In nabre/dev-kit a hidden watcher process did it later.)
pub fn save_after_run(kit: &Kit) -> Result<()> {
    let f = files(kit);
    if !f.seen.exists() {
        return Ok(());
    }
    let _lock = Lock::take(kit)?;
    if !kit_agy_running(kit) {
        let _ = fs::remove_file(&f.starting);
    }
    save_login(kit, &f)
}

/// Something still to save (an agy run that did not end cleanly).
pub fn pending(kit: &Kit) -> bool {
    files(kit).seen.exists()
}

/// Before agy starts: the kit's login into the Credential Manager.
pub fn restore(kit: &Kit) -> Result<()> {
    let f = files(kit);
    let _lock = Lock::take(kit)?;
    if f.seen.exists() && kit_agy_running(kit) {
        // Another agy of the kit runs: the login is already in place.
        fs::write(&f.starting, b"")?;
        return Ok(());
    }
    save_login(kit, &f)?;
    let existing = list();
    let mut before: Vec<Item> = existing
        .iter()
        .map(|c| Item {
            kit: Some(false),
            ..c.clone()
        })
        .collect();
    for c in read(&f.store) {
        // A credential with the same name already on this PC is not the kit's.
        if existing.iter().any(|e| e.target == c.target) {
            continue;
        }
        write(&c, CRED_PERSIST_SESSION.0)?;
        let written = list()
            .into_iter()
            .find(|x| x.target == c.target)
            .and_then(|x| x.written);
        before.push(Item {
            target: c.target.clone(),
            user: None,
            blob: None,
            written,
            persist: None,
            kit: Some(true),
        });
    }
    save_file(&f.seen, &before)?;
    fs::write(&f.starting, b"")?;
    Ok(())
}

/// Logged in with a Google account (a saved login in the kit).
pub fn has_login(kit: &Kit) -> bool {
    !read(&files(kit).store).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn names() {
        assert!(is_antigravity("gemini:antigravity"));
        assert!(is_antigravity("jetski-auth:user"));
        assert!(is_antigravity("Antigravity:x"));
        assert!(!is_antigravity("git:https://github.com"));
        assert!(!is_antigravity("myantigravity:x"));
        assert!(!is_antigravity("antigravity"), "no colon: not go-keyring");
    }

    /// The real Credential Manager, with a test-only name: restore puts the
    /// kit's login in Windows (session only), save takes it back.
    #[test]
    fn restore_and_save_round_trip() {
        let tmp = TempDir::new("keyring");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let target = format!("antigravity-devkit-test-{}:user", std::process::id());
        let f = files(&kit);
        let blob = base64::engine::general_purpose::STANDARD.encode(b"token-1");
        save_file(
            &f.store,
            &[Item {
                target: target.clone(),
                user: Some("u".into()),
                blob: Some(blob),
                written: None,
                persist: None,
                kit: None,
            }],
        )
        .unwrap();

        restore(&kit).unwrap();
        let in_windows = list()
            .into_iter()
            .find(|c| c.target == target)
            .expect("restored");
        assert_eq!(
            in_windows.persist,
            Some(CRED_PERSIST_SESSION.0),
            "never on disk"
        );
        assert!(f.seen.exists());

        // agy refreshes the token meanwhile.
        std::thread::sleep(Duration::from_millis(50));
        let new_blob = base64::engine::general_purpose::STANDARD.encode(b"token-2");
        write(
            &Item {
                target: target.clone(),
                user: Some("u".into()),
                blob: Some(new_blob.clone()),
                written: None,
                persist: None,
                kit: None,
            },
            CRED_PERSIST_SESSION.0,
        )
        .unwrap();
        let _ = fs::remove_file(&f.starting);
        save(&kit).unwrap();
        assert!(
            list().iter().all(|c| c.target != target),
            "gone from Windows"
        );
        assert!(!f.seen.exists());
        let stored = read(&f.store);
        assert_eq!(stored.len(), 1);
        assert_eq!(
            stored[0].blob.as_deref(),
            Some(new_blob.as_str()),
            "new token kept in the kit"
        );
        delete(&target);
    }
}
