//! macOS: go-keyring calls `/usr/bin/security` (so do we: items it writes
//! are then read by agy without a Keychain prompt). Linux: the Secret
//! Service, through the `keyring` crate; written into its `login`
//! collection, the only one go-keyring reads ([`LOGIN_COLLECTION`]).

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
use keyring::secret_service::{delete_item, get_item_attributes, get_item_secret, SsCredential};

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
