//! Antigravity's sign-in in the Keychain (`agents::keyring`, D-115):
//! go-keyring calls `/usr/bin/security`, and so do we: items it writes are
//! then read by agy without a Keychain prompt.

use anyhow::{bail, Context, Result};

use crate::agents::keyring::{
    mac_add_command, mac_decode, mac_found, split_target, unix_item, Item, KNOWN,
};

const SECURITY: &str = "/usr/bin/security";

fn security() -> std::process::Command {
    let mut cmd = std::process::Command::new(SECURITY);
    cmd.stdin(std::process::Stdio::null());
    cmd
}

pub fn list() -> Result<Vec<Item>> {
    let mut out = Vec::new();
    for (service, user) in KNOWN {
        let o = security()
            .args(["find-generic-password", "-s", service, "-a", user, "-w"])
            .output()
            .context(SECURITY)?;
        // Only 44 means "no such item"; a locked or refused Keychain
        // is an error, so restore and save stop before touching anything.
        if !mac_found(o.status.code())? {
            continue;
        }
        let raw = mac_decode(&String::from_utf8_lossy(&o.stdout))?;
        out.push(unix_item(service, user, &raw));
    }
    Ok(out)
}

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
    let line = mac_add_command(service, user, &raw)?;
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
