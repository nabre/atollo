//! The moves around an agy run: the kit's sign-in saved out of the store, then restored into it.

use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context, Result};
#[cfg(windows)]
use windows::Win32::Security::Credentials::CRED_PERSIST_SESSION;

use super::files::{files, read, save_file, Files};
use super::lock::Lock;
use super::{
    kit_agy_running, plaintext_allowed, Item, NoStore, SealKeyMissing, Store, NO_STORE,
    NO_STORE_REFUSED,
};
use crate::kit::Kit;

/// "Logon session" persistence: never on disk, gone at logoff.
#[cfg(windows)]
pub(super) const PERSIST_SESSION: u32 = CRED_PERSIST_SESSION.0;
#[cfg(not(windows))]
pub(super) const PERSIST_SESSION: u32 = 1;

/// An agy of the kit started less than 20 s ago (the starting mark), and
/// may not show as running yet.
fn starting_is_fresh(f: &Files) -> bool {
    fs::metadata(&f.starting)
        .and_then(|m| m.modified())
        .map(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(20))
        .unwrap_or(false)
}

pub(super) fn save_login(kit: &Kit, f: &Files, store: &dyn Store) -> Result<()> {
    if !f.seen.exists() || kit_agy_running(kit) {
        return Ok(());
    }
    if starting_is_fresh(f) {
        return Ok(());
    }
    let _ = fs::remove_file(&f.starting);
    let before = read(&f.seen);
    let now = store.list()?;
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
    // Signed out inside agy: a kit credential no longer there is forgotten
    // (only one the store can list: any other was never moved in).
    keep.retain(|k| {
        !before.iter().any(|o| {
            o.target == k.target
                && o.kit == Some(true)
                && store.listable(&k.target)
                && !now.iter().any(|c| c.target == k.target)
        })
    });
    // This computer's secrets opened before anything is written: a file
    // sealed elsewhere (a wrong key, an altered file) changes nothing,
    // neither the kit's copy nor the store. A sealing key that is gone for
    // good ([`SealKeyMissing`]) would block every save for ever: those
    // secrets are dropped with a warning, what agy left in their place
    // leaves the store, and the kit's sign-in is saved (PORT-V6-06).
    let mut opened = Vec::new();
    let mut lost = Vec::new();
    for mut o in to_restore {
        match o.blob.as_deref().map(|b| store.unprotect(b)).transpose() {
            Ok(blob) => {
                o.blob = blob;
                opened.push(o);
            }
            Err(e) if e.downcast_ref::<SealKeyMissing>().is_some() => lost.push(o.target),
            Err(e) => {
                return Err(e.context(
                    "this computer's own Antigravity sign-in cannot be opened, nothing changed",
                ))
            }
        }
    }
    if !lost.is_empty() {
        crate::common::log::warn(&format!(
            "antigravity keyring: {SealKeyMissing}: this computer's own sign-in ({}) cannot be put back; the kit's sign-in is saved and the pending file dropped",
            lost.join(", ")
        ));
        for t in lost {
            if now.iter().any(|c| c.target == t) && !to_delete.contains(&t) {
                to_delete.push(t);
            }
        }
    }
    let to_restore = opened;
    // The kit's copy first; the system is cleaned only once it is written.
    save_file(&f.store, &keep)?;
    for t in to_delete {
        store.delete(&t);
    }
    for o in to_restore {
        store.write(&o, o.persist.unwrap_or(PERSIST_SESSION))?;
    }
    fs::remove_file(&f.seen)?;
    store.forget_key();
    Ok(())
}

/// The permissions a file in `dir` really has after being set to 600.
#[cfg(unix)]
pub(super) fn mode_after_600(dir: &Path) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    fs::create_dir_all(dir).ok()?;
    let probe = dir.join(".mode-probe");
    fs::write(&probe, b"").ok()?;
    let _ = fs::set_permissions(&probe, fs::Permissions::from_mode(0o600));
    let mode = fs::metadata(&probe).ok().map(|m| m.permissions().mode());
    let _ = fs::remove_file(&probe);
    mode
}

#[cfg(not(unix))]
pub(super) fn mode_after_600(_dir: &Path) -> Option<u32> {
    None
}

/// No store at all: the kit moves nothing. Allowed (with the warning that
/// the sign-in is unencrypted in the kit) only where [`plaintext_allowed`].
fn without_store(f: &Files, detail: &str, os: crate::kit::Os, mode: Option<u32>) -> Result<()> {
    if !plaintext_allowed(os, mode) {
        crate::common::log::warn(&format!(
            "antigravity keyring: {NO_STORE_REFUSED} ({detail})"
        ));
        bail!("{NO_STORE_REFUSED}");
    }
    crate::common::log::warn(&format!("antigravity keyring: {NO_STORE} ({detail})"));
    eprintln!("  [!] Antigravity: {NO_STORE}.");
    if f.store.exists() {
        // Owner-only even when it came from another system.
        let _ = crate::platform::files::restrict_to_owner(&f.store);
    }
    Ok(())
}

/// `restore_with`, with the system and the check of the 600 mode given
/// (tests).
pub(super) fn restore_checked(
    kit: &Kit,
    store: &dyn Store,
    os: crate::kit::Os,
    mode_of: impl Fn(&Path) -> Option<u32>,
) -> Result<()> {
    let f = files(kit);
    let _lock = Lock::take(kit)?;
    if f.seen.exists() && (kit_agy_running(kit) || starting_is_fresh(&f)) {
        // Another agy of the kit runs, or has just been started: the login
        // is already in place, and listing the store now would take the
        // kit's own login for this computer's.
        fs::write(&f.starting, b"")?;
        return Ok(());
    }
    save_login(kit, &f, store)?;
    let existing = match store.list() {
        Ok(items) => items,
        Err(e) if e.downcast_ref::<NoStore>().is_some() => {
            let dir = f.store.parent().map(Path::to_path_buf).unwrap_or_default();
            return without_store(&f, &format!("{e:#}"), os, mode_of(&dir));
        }
        // A store that cannot be used now (locked, denied): nothing moved.
        Err(e) => {
            crate::common::log::warn(&format!(
                "antigravity keyring: store not usable, sign-in not moved: {e:#}"
            ));
            return Err(e.context("the Antigravity sign-in was not moved"));
        }
    };
    let moved = move_in(&f, store, &existing);
    if moved.is_err() && !f.seen.exists() {
        // A sealing key made for a keyring-before.json never written.
        store.forget_key();
    }
    moved?;
    fs::write(&f.starting, b"")?;
    Ok(())
}

/// This computer's own sign-in sealed into keyring-before.json, the kit's
/// written into the store.
fn move_in(f: &Files, store: &dyn Store, existing: &[Item]) -> Result<()> {
    // Every secret of this computer sealed first: if one cannot be (no
    // sealing key), nothing is written anywhere (fail closed, as DPAPI).
    let mut before: Vec<Item> = existing
        .iter()
        .map(|c| {
            Ok(Item {
                kit: Some(false),
                blob: c.blob.as_deref().map(|b| store.protect(b)).transpose()?,
                ..c.clone()
            })
        })
        .collect::<Result<_>>()
        .inspect_err(|e| {
            crate::common::log::warn(&format!(
                "antigravity keyring: this computer's sign-in cannot be sealed, the kit's sign-in is not moved: {e:#}"
            ));
        })
        .context("the Antigravity sign-in was not moved: this computer's own sign-in cannot be sealed")?;
    let mut moved: Vec<String> = Vec::new();
    let result = (|| {
        for c in read(&f.store) {
            // A name the store cannot list back stays in keyring.json only;
            // a credential with the same name already on this computer is
            // not the kit's.
            if !store.listable(&c.target) || existing.iter().any(|e| e.target == c.target) {
                continue;
            }
            moved.push(c.target.clone());
            store.write(&c, PERSIST_SESSION)?;
            let written = store
                .list()?
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
        save_file(&f.seen, &before)
    })();
    if result.is_err() && store.undo_partial() {
        // No "before" file records them: the kit's credentials written so
        // far would later look like this computer's own.
        for t in &moved {
            store.delete(t);
        }
    }
    result
}

#[cfg(test)]
#[path = "../../../../test/rust/agents/keyring/moves.rs"]
mod tests;
