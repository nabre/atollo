//! Windows: agy's credentials in the Credential Manager, this computer's own sealed with DPAPI.

use anyhow::{Context, Result};
use base64::Engine;
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Credentials::{
    CredDeleteW, CredEnumerateW, CredFree, CredWriteW, CREDENTIALW, CRED_FLAGS, CRED_PERSIST,
    CRED_TYPE_GENERIC,
};

use crate::agents::keyring::{is_antigravity, Item, Store, System, DPAPI, PERSIST_SESSION};

// The kit's "logon session" persistence is Windows' own value.
const _: () =
    assert!(PERSIST_SESSION == windows::Win32::Security::Credentials::CRED_PERSIST_SESSION.0);

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

/// DPAPI for the current user: a secret of the PC copied into the kit is
/// unreadable on any other PC or account.
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

#[cfg(test)]
#[path = "../../../../test/rust/os/windows/agy_store.rs"]
mod tests;
