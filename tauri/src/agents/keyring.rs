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

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
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
    /// already written (macOS and Linux; Windows as nabre/atollo).
    fn undo_partial(&self) -> bool {
        false
    }
}

/// The store of this system.
pub struct System {
    #[cfg_attr(windows, allow(dead_code))]
    seal_user: String,
    /// [`SEAL_ID_FILE`] and the id it holds (or will hold).
    #[cfg_attr(windows, allow(dead_code))]
    seal_id: (PathBuf, String),
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

    /// The id in the key's name saved with the kit, if it is not yet: the
    /// name then survives a move of the kit (PORT-V6-06).
    fn keep_seal_id(&self) -> Result<()> {
        let (file, id) = &self.seal_id;
        if !file.exists() {
            if let Some(dir) = file.parent() {
                fs::create_dir_all(dir)?;
            }
            fs::write(file, id).with_context(|| file.display().to_string())?;
        }
        Ok(())
    }

    /// The sealing key, created on first use. Missing when it is only
    /// read: [`SealKeyMissing`].
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
            Err(keyring::Error::NoEntry) => return Err(SealKeyMissing.into()),
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
        if !known_target(&item.target) {
            bail!("{}: not a name agy uses here", item.target);
        }
        unix::write(item)
    }
    fn delete(&self, target: &str) {
        unix::delete(target);
    }
    /// Fails closed, like DPAPI: without the sealing key (store locked,
    /// access denied, any error) the computer's secret never reaches the
    /// kit.
    fn protect(&self, blob: &str) -> Result<String> {
        self.keep_seal_id()
            .context("sealing key: its name in the kit")?;
        let key = self
            .key(true)
            .context("sealing key in the Keychain or the Secret Service")?;
        seal(&key, &random()?, blob)
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
    fn listable(&self, target: &str) -> bool {
        known_target(target)
    }
    fn undo_partial(&self) -> bool {
        true
    }
}

/// macOS: go-keyring calls `/usr/bin/security` (so do we: items it writes
/// are then read by agy without a Keychain prompt). Linux: the Secret
/// Service, through the `keyring` crate; written into its `login`
/// collection, the only one go-keyring reads ([`LOGIN_COLLECTION`]).
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
            // Only 44 means "no such item"; a locked or refused Keychain
            // is an error, so restore and save stop before touching anything.
            if !super::mac_found(o.status.code())? {
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
                Err(e) => {
                    let text = format!("{e:#}");
                    if super::no_secret_service(&text) {
                        return Err(anyhow::Error::new(super::NoStore(text)));
                    }
                    return Err(e).context("Secret Service");
                }
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

    /// Into the `login` collection only, whatever `default` is: go-keyring
    /// reads nothing else. Without it (KeePassXC, some KWallet setups)
    /// nothing is written and the restore fails closed with
    /// [`super::NO_LOGIN_COLLECTION`].
    #[cfg(not(target_os = "macos"))]
    pub fn write(item: &Item) -> Result<()> {
        use dbus_secret_service::{EncryptionType, SecretService};

        let Some((service, user)) = split_target(&item.target) else {
            anyhow::bail!("{}: not a go-keyring name", item.target);
        };
        let raw = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            item.blob.as_deref().unwrap_or_default(),
        )
        .context("credential blob")?;
        // The same session the `keyring` crate opens (no crypto feature).
        let ss = SecretService::connect(EncryptionType::Plain).context("Secret Service")?;
        let all = ss.get_all_collections().context("Secret Service")?;
        let login = &all[super::login_collection(all.iter().map(|c| &*c.path))?];
        login
            .ensure_unlocked()
            .context("Secret Service: the login collection is locked")?;
        delete(&item.target);
        // The attributes and label the crate gives its items (`application`
        // = `rust-keyring`: `list` prefers agy's own copy to these).
        let cred = SsCredential::new_with_target(Some("login"), service, user)?;
        let attributes = cred
            .attributes
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        login
            .create_item(&cred.label, attributes, &raw, true, "text/plain")
            .with_context(|| format!("Secret Service {}", item.target))?;
        Ok(())
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

/// The kit's files in `dir` (`data\antigravity`). `host`: the tag of this
/// computer and account on macOS and Linux ([`machine_tag`]), in the names
/// of the files of a run (Windows: `None`, the names of nabre/atollo).
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

/// An agy of the kit started less than 20 s ago (the starting mark), and
/// may not show as running yet.
fn starting_is_fresh(f: &Files) -> bool {
    fs::metadata(&f.starting)
        .and_then(|m| m.modified())
        .map(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(20))
        .unwrap_or(false)
}

fn save_login(kit: &Kit, f: &Files, store: &dyn Store) -> Result<()> {
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
        crate::log::warn(&format!(
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

/// The permissions a file in `dir` really has after being set to 600.
#[cfg(unix)]
fn mode_after_600(dir: &Path) -> Option<u32> {
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
fn mode_after_600(_dir: &Path) -> Option<u32> {
    None
}

/// No store at all: the kit moves nothing. Allowed (with the warning that
/// the sign-in is unencrypted in the kit) only where [`plaintext_allowed`].
fn without_store(f: &Files, detail: &str, os: crate::kit::Os, mode: Option<u32>) -> Result<()> {
    if !plaintext_allowed(os, mode) {
        crate::log::warn(&format!(
            "antigravity keyring: {NO_STORE_REFUSED} ({detail})"
        ));
        bail!("{NO_STORE_REFUSED}");
    }
    crate::log::warn(&format!("antigravity keyring: {NO_STORE} ({detail})"));
    eprintln!("  [!] Antigravity: {NO_STORE}.");
    if f.store.exists() {
        // Owner-only even when it came from another system.
        let _ = crate::router::fs::restrict_to_owner(&f.store);
    }
    Ok(())
}

pub fn restore_with(kit: &Kit, store: &dyn Store) -> Result<()> {
    restore_checked(kit, store, crate::kit::Os::current(), mode_after_600)
}

/// `restore_with`, with the system and the check of the 600 mode given
/// (tests).
fn restore_checked(
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
            crate::log::warn(&format!(
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
            crate::log::warn(&format!(
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
