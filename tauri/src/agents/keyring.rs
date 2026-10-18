//! Antigravity's Google login kept inside the kit, with no trace left in the
//! system's credential store (port of nabre/atollo `agy-keyring.ps1`).
//!
//! `agy` stores its login with go-keyring, with no option to use a file:
//! the Credential Manager on Windows, the Keychain on macOS, the Secret
//! Service on Linux. Around every `agy` run (`data\kit\bin\agy.cmd`, `agy`
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
//! macOS and Linux (WP-4.1): agy uses go-keyring's service `gemini` and user
//! `antigravity` (the Windows target `gemini:antigravity`; see [`KNOWN`]),
//! so `keyring.json` is the same file on every system. On macOS the items
//! are read and written with `/usr/bin/security`, as go-keyring does, so
//! that agy reads them without a Keychain prompt; on Linux through the
//! Secret Service. Without a Secret Service (WSL, minimal desktops) nothing
//! is moved: agy keeps its login in its own file inside the kit's home, and
//! the kit's files stay owner-only (600) with a warning. The "before" files
//! carry the computer's name on macOS and Linux, so a kit moved after a
//! crash never mixes two computers' credentials.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[cfg(unix)]
use anyhow::bail;
use anyhow::{Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use windows::core::{HSTRING, PCWSTR, PWSTR};
#[cfg(windows)]
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, LocalFree, ERROR_ALREADY_EXISTS, HLOCAL, WAIT_ABANDONED,
    WAIT_OBJECT_0,
};
#[cfg(windows)]
use windows::Win32::Security::Credentials::{
    CredDeleteW, CredEnumerateW, CredFree, CredWriteW, CREDENTIALW, CRED_FLAGS, CRED_PERSIST,
    CRED_PERSIST_SESSION, CRED_TYPE_GENERIC,
};
#[cfg(windows)]
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};

/// "Logon session" persistence: never on disk, gone at logoff.
#[cfg(windows)]
const PERSIST_SESSION: u32 = CRED_PERSIST_SESSION.0;
#[cfg(not(windows))]
const PERSIST_SESSION: u32 = 1;

use crate::kit::Kit;

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
/// (Antigravity's code name) or antigravity.
#[cfg_attr(not(any(windows, test)), allow(dead_code))]
pub fn is_antigravity(target: &str) -> bool {
    let lower = target.to_ascii_lowercase();
    let Some((service, user)) = lower.split_once(':') else {
        return false;
    };
    let starts = |s: &str| s.starts_with("jetski") || s.starts_with("antigravity");
    starts(service) || starts(user)
}

mod portable;
#[cfg_attr(not(any(unix, test)), allow(unused_imports))]
pub use portable::*;

const DPAPI: &str = "dpapi:";

/// Where agy keeps its login on this system. Behind a trait so that the
/// moves in and out of the kit are tested with a fake store.
pub trait Store {
    /// agy's credentials in the store now. An error: no store on this
    /// system (Linux without a Secret Service).
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
}

/// The store of this system.
pub struct System {
    #[cfg_attr(windows, allow(dead_code))]
    seal_user: String,
}

impl System {
    pub fn of(kit: &Kit) -> System {
        System {
            seal_user: format!("antigravity-before-{}", kit.id()),
        }
    }
}

#[cfg(windows)]
impl Store for System {
    fn list(&self) -> Result<Vec<Item>> {
        Ok(list())
    }
    fn write(&self, item: &Item, persist: u32) -> Result<()> {
        write(item, persist)
    }
    fn delete(&self, target: &str) {
        delete(target);
    }
    fn protect(&self, blob: &str) -> Result<String> {
        protect(blob)
    }
    fn unprotect(&self, blob: &str) -> Result<String> {
        unprotect(blob)
    }
}

/// The service of the kit's own sealing key in the Keychain or the
/// Secret Service (one per kit, only while keyring-before.json exists).
#[cfg(unix)]
const SEAL_SERVICE: &str = "Atollo";

#[cfg(unix)]
impl System {
    fn key_entry(&self) -> Result<keyring::Entry> {
        Ok(keyring::Entry::new(SEAL_SERVICE, &self.seal_user)?)
    }

    /// The sealing key, created on first use.
    fn key(&self, create: bool) -> Result<[u8; 32]> {
        let entry = self.key_entry()?;
        let text = match entry.get_password() {
            Ok(t) => t,
            Err(keyring::Error::NoEntry) if create => {
                let key: [u8; 32] = random()?;
                let text = base64::engine::general_purpose::STANDARD.encode(key);
                entry.set_password(&text)?;
                text
            }
            Err(e) => return Err(e.into()),
        };
        let bytes = base64::engine::general_purpose::STANDARD.decode(text.trim())?;
        bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("sealing key of the wrong size"))
    }
}

#[cfg(unix)]
fn random<const N: usize>() -> Result<[u8; N]> {
    use std::io::Read;
    let mut out = [0u8; N];
    fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut out))
        .context("/dev/urandom")?;
    Ok(out)
}

#[cfg(unix)]
impl Store for System {
    fn list(&self) -> Result<Vec<Item>> {
        unix::list()
    }
    fn write(&self, item: &Item, _persist: u32) -> Result<()> {
        unix::write(item)
    }
    fn delete(&self, target: &str) {
        unix::delete(target);
    }
    fn protect(&self, blob: &str) -> Result<String> {
        match self.key(true) {
            Ok(key) => seal(&key, &random()?, blob),
            // The file stays owner-only (600): the secret as it is.
            Err(e) => {
                crate::log::warn(&format!(
                    "antigravity keyring: no sealing key ({e:#}): the computer's login stays only in a private file"
                ));
                Ok(blob.to_owned())
            }
        }
    }
    fn unprotect(&self, blob: &str) -> Result<String> {
        if blob.starts_with(DPAPI) {
            bail!("DPAPI is not available on this system");
        }
        if !blob.starts_with(SEALED) {
            return Ok(blob.to_owned());
        }
        open_sealed(&self.key(false)?, blob)
    }
    fn forget_key(&self) {
        if let Ok(entry) = self.key_entry() {
            let _ = entry.delete_credential();
        }
    }
}

/// macOS: go-keyring calls `/usr/bin/security` (so do we: items it writes
/// are then read by agy without a Keychain prompt). Linux: the Secret
/// Service, through the `keyring` crate.
#[cfg(unix)]
mod unix {
    #[cfg(not(target_os = "macos"))]
    use anyhow::Context;
    use anyhow::Result;
    #[cfg(target_os = "macos")]
    use anyhow::{bail, Context};

    use super::{split_target, unix_item, Item, KNOWN};

    #[cfg(target_os = "macos")]
    const SECURITY: &str = "/usr/bin/security";

    #[cfg(target_os = "macos")]
    fn security() -> std::process::Command {
        let mut cmd = std::process::Command::new(SECURITY);
        cmd.stdin(std::process::Stdio::null());
        cmd
    }

    #[cfg(target_os = "macos")]
    pub fn list() -> Result<Vec<Item>> {
        let mut out = Vec::new();
        for (service, user) in KNOWN {
            let o = security()
                .args(["find-generic-password", "-s", service, "-a", user, "-w"])
                .output()
                .context(SECURITY)?;
            // 44: no such item.
            if !o.status.success() {
                continue;
            }
            let raw = super::mac_decode(&String::from_utf8_lossy(&o.stdout))?;
            out.push(unix_item(service, user, &raw));
        }
        Ok(out)
    }

    #[cfg(target_os = "macos")]
    pub fn write(item: &Item) -> Result<()> {
        use std::io::Write;
        use std::process::Stdio;
        let Some((service, user)) = split_target(&item.target) else {
            bail!("{}: not a go-keyring name", item.target);
        };
        let raw = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            item.blob.as_deref().unwrap_or_default(),
        )
        .context("credential blob")?;
        let line = super::mac_add_command(service, user, &raw)?;
        let mut child = std::process::Command::new(SECURITY)
            .arg("-i")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context(SECURITY)?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(line.as_bytes()).context(SECURITY)?;
        }
        let status = child.wait().context(SECURITY)?;
        let found = list()?.iter().any(|c| c.target == item.target);
        if !status.success() || !found {
            bail!("{}: not written in the Keychain", item.target);
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    pub fn delete(target: &str) {
        let Some((service, user)) = split_target(target) else {
            return;
        };
        // Every copy (at most a few): `security` deletes one per call.
        for _ in 0..8 {
            let gone = security()
                .args(["delete-generic-password", "-s", service, "-a", user])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map_or(true, |s| !s.success());
            if gone {
                break;
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    use keyring::credential::CredentialApi;
    #[cfg(not(target_os = "macos"))]
    use keyring::secret_service::{
        delete_item, get_item_attributes, get_item_secret, SsCredential,
    };

    /// go-keyring's items carry only `service` and `username`; the ones the
    /// kit writes through the crate also `application` = `rust-keyring`.
    /// agy's own copy wins when both are there.
    #[cfg(not(target_os = "macos"))]
    pub fn list() -> Result<Vec<Item>> {
        let mut out = Vec::new();
        for (service, user) in KNOWN {
            let cred = SsCredential::new_with_no_target(service, user)?;
            let found = match cred.map_matching_items(
                |i| Ok((get_item_attributes(i)?, get_item_secret(i)?)),
                false,
            ) {
                Ok(found) => found,
                Err(keyring::Error::NoEntry) => Vec::new(),
                Err(e) => return Err(e).context("Secret Service"),
            };
            let ours = |a: &std::collections::HashMap<String, String>| {
                a.get("application").is_some_and(|v| v == "rust-keyring")
            };
            let pick = found
                .iter()
                .find(|(a, _)| !ours(a))
                .or_else(|| found.first());
            if let Some((_, raw)) = pick {
                out.push(unix_item(service, user, raw));
            }
        }
        Ok(out)
    }

    #[cfg(not(target_os = "macos"))]
    pub fn write(item: &Item) -> Result<()> {
        let Some((service, user)) = split_target(&item.target) else {
            anyhow::bail!("{}: not a go-keyring name", item.target);
        };
        let raw = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            item.blob.as_deref().unwrap_or_default(),
        )
        .context("credential blob")?;
        delete(&item.target);
        SsCredential::new_with_target(None, service, user)?
            .set_secret(&raw)
            .with_context(|| format!("Secret Service {}", item.target))
    }

    #[cfg(not(target_os = "macos"))]
    pub fn delete(target: &str) {
        let Some((service, user)) = split_target(target) else {
            return;
        };
        if let Ok(cred) = SsCredential::new_with_no_target(service, user) {
            let _ = cred.map_matching_items(delete_item, false);
        }
    }
}

#[cfg(windows)]
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

#[cfg(windows)]
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

#[cfg(windows)]
fn delete(target: &str) {
    // SAFETY: valid string; a missing credential is not an error here.
    let _ = unsafe { CredDeleteW(&HSTRING::from(target), CRED_TYPE_GENERIC, None) };
}

struct Files {
    store: PathBuf,
    seen: PathBuf,
    starting: PathBuf,
}

/// The kit's files in `dir` (`data\antigravity`). `host`: the computer's
/// name on macOS and Linux, in the names of the files of a run (Windows:
/// `None`, the names of nabre/atollo).
fn files_in(dir: &Path, host: Option<&str>) -> Files {
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

#[cfg(unix)]
fn host() -> Option<String> {
    let mut buf = [0u8; 256];
    // SAFETY: the buffer and its length; the name is NUL-terminated within.
    let ok = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) } == 0;
    let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
    Some(host_tag(if ok {
        std::str::from_utf8(&buf[..end]).unwrap_or_default()
    } else {
        ""
    }))
}

#[cfg(windows)]
fn host() -> Option<String> {
    None
}

fn files(kit: &Kit) -> Files {
    files_in(&kit.data().join("antigravity"), host().as_deref())
}

fn read(path: &std::path::Path) -> Vec<Item> {
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

/// DPAPI for the current user: a secret of the PC copied into the kit is
/// unreadable on any other PC or account.
#[cfg(windows)]
fn protect(blob: &str) -> Result<String> {
    use windows::Win32::Security::Cryptography::{CryptProtectData, CRYPT_INTEGER_BLOB};
    let data = base64::engine::general_purpose::STANDARD.decode(blob)?;
    let input = CRYPT_INTEGER_BLOB {
        cbData: u32::try_from(data.len())?,
        pbData: data.as_ptr().cast_mut(),
    };
    let mut out = CRYPT_INTEGER_BLOB::default();
    // SAFETY: `input` points to `data`, alive for the call; the output is
    // allocated by DPAPI and freed with LocalFree after the copy.
    let bytes = unsafe {
        CryptProtectData(&input, PCWSTR::null(), None, None, None, 0, &mut out)
            .context("CryptProtectData")?;
        let bytes = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        let _ = LocalFree(Some(HLOCAL(out.pbData.cast())));
        bytes
    };
    Ok(format!(
        "{DPAPI}{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

/// The secret saved by `protect` (a blob without the prefix as it is).
#[cfg(windows)]
fn unprotect(blob: &str) -> Result<String> {
    use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};
    let Some(rest) = blob.strip_prefix(DPAPI) else {
        return Ok(blob.to_owned());
    };
    let data = base64::engine::general_purpose::STANDARD.decode(rest)?;
    let input = CRYPT_INTEGER_BLOB {
        cbData: u32::try_from(data.len())?,
        pbData: data.as_ptr().cast_mut(),
    };
    let mut out = CRYPT_INTEGER_BLOB::default();
    // SAFETY: as in `protect`.
    let bytes = unsafe {
        CryptUnprotectData(&input, None, None, None, None, 0, &mut out)
            .context("CryptUnprotectData")?;
        let bytes = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        let _ = LocalFree(Some(HLOCAL(out.pbData.cast())));
        bytes
    };
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

/// The kit's copy: on macOS and Linux owner-only (600, in a 700 folder),
/// since it holds the login in clear when no store can seal it.
fn save_file(path: &std::path::Path, items: &[Item]) -> Result<()> {
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

/// An `agy.exe` (`agy` on macOS and Linux) of this kit is running.
pub fn kit_agy_running(kit: &Kit) -> bool {
    crate::os::processes::running_under(&kit.apps())
        .iter()
        .any(|p| p.name.eq_ignore_ascii_case("agy.exe") || p.name == "agy")
}

/// The kit's lock: an exclusive `flock` on `data/antigravity/keyring.lock`,
/// waited for at most 30 s like on Windows.
#[cfg(unix)]
struct Lock(#[allow(dead_code)] fs::File);

#[cfg(unix)]
impl Lock {
    fn take(kit: &Kit) -> Result<Lock> {
        use std::os::fd::AsRawFd;
        let dir = kit.data().join("antigravity");
        fs::create_dir_all(&dir)?;
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(dir.join("keyring.lock"))
            .context("keyring.lock")?;
        let start = std::time::Instant::now();
        // SAFETY: a valid descriptor owned by `file`.
        while unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            if start.elapsed() > Duration::from_secs(30) {
                crate::log::warn("antigravity keyring: lock busy for 30 s, going on");
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Ok(Lock(file))
    }
}

/// The kit's lock: `Local\Atollo-agy-keyring-<kit id>` (same as the kit).
#[cfg(windows)]
struct Lock(windows::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl Lock {
    fn take(kit: &Kit) -> Result<Lock> {
        let name = HSTRING::from(format!(r"Local\Atollo-agy-keyring-{}", kit.id()));
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

#[cfg(windows)]
impl Drop for Lock {
    fn drop(&mut self) {
        // SAFETY: handle from CreateMutexW; released and closed once.
        unsafe {
            let _ = ReleaseMutex(self.0);
            let _ = CloseHandle(self.0);
        }
    }
}

fn save_login(kit: &Kit, f: &Files, store: &dyn Store) -> Result<()> {
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
    // Signed out inside agy: a kit credential no longer there is forgotten.
    keep.retain(|k| {
        !before.iter().any(|o| {
            o.target == k.target && o.kit == Some(true) && !now.iter().any(|c| c.target == k.target)
        })
    });
    // The kit's copy first; the system is cleaned only once it is written.
    save_file(&f.store, &keep)?;
    for t in to_delete {
        store.delete(&t);
    }
    for mut o in to_restore {
        o.blob = o.blob.as_deref().map(|b| store.unprotect(b)).transpose()?;
        store.write(&o, o.persist.unwrap_or(PERSIST_SESSION))?;
    }
    fs::remove_file(&f.seen)?;
    store.forget_key();
    Ok(())
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
/// (In nabre/atollo a hidden watcher process did it later.)
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

/// Before agy starts: the kit's login into the system's store.
pub fn restore(kit: &Kit) -> Result<()> {
    restore_with(kit, &System::of(kit))
}

/// Told when the system has no store for agy (Linux without a Secret
/// Service): nothing is moved and agy keeps its login in its own file.
pub const NO_STORE: &str = "no Secret Service on this system: agy keeps its login in its own file inside the kit, and the kit's copy stays owner-only (600)";

pub fn restore_with(kit: &Kit, store: &dyn Store) -> Result<()> {
    let f = files(kit);
    let _lock = Lock::take(kit)?;
    if f.seen.exists() && kit_agy_running(kit) {
        // Another agy of the kit runs: the login is already in place.
        fs::write(&f.starting, b"")?;
        return Ok(());
    }
    save_login(kit, &f, store)?;
    let existing = match store.list() {
        Ok(items) => items,
        Err(e) => {
            crate::log::warn(&format!("antigravity keyring: {NO_STORE} ({e:#})"));
            eprintln!("  [!] Antigravity: {NO_STORE}.");
            if f.store.exists() {
                // Owner-only even when it came from another system.
                let _ = crate::router::fs::restrict_to_owner(&f.store);
            }
            return Ok(());
        }
    };
    let mut before: Vec<Item> = existing
        .iter()
        .map(|c| {
            Ok(Item {
                kit: Some(false),
                blob: c.blob.as_deref().map(|b| store.protect(b)).transpose()?,
                ..c.clone()
            })
        })
        .collect::<Result<_>>()?;
    for c in read(&f.store) {
        // A credential with the same name already on this computer is not
        // the kit's.
        if existing.iter().any(|e| e.target == c.target) {
            continue;
        }
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
    save_file(&f.seen, &before)?;
    fs::write(&f.starting, b"")?;
    Ok(())
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
#[path = "../../../test/rust/agents/keyring.rs"]
mod tests;
