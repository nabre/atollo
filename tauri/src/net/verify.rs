//! Checksums of downloaded files (port of nabre/atollo
//! `_prepare-helpers.ps1:60-130`): SHA-256, SHA-512, npm `sha512-<base64>`,
//! and SHA3-256 (what sqlite.org publishes, D-73).
//! A file that does not match is deleted, so it can never be installed later.
//! Authenticode signatures (Claude Code, VC++ runtime) are in `net/signature.rs`.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use anyhow::{bail, Context, Result};
use base64::Engine;
use sha2::{Digest, Sha256, Sha512};
use sha3::Sha3_256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expect {
    Sha256(String),
    Sha512(String),
    /// npm `dist.integrity`, e.g. `sha512-AbC…==`.
    NpmIntegrity(String),
    Sha3_256(String),
}

fn hash_file<D: Digest>(path: &Path) -> Result<Vec<u8>> {
    let mut file = File::open(path).with_context(|| path.display().to_string())?;
    let mut hasher = D::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_vec())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn sha256_hex(path: &Path) -> Result<String> {
    hash_file::<Sha256>(path).map(|h| hex(&h))
}

/// Checks `path`; deletes it and fails when it does not match.
pub fn check(path: &Path, expect: &Expect) -> Result<()> {
    if !matches(path, expect)? {
        let _ = fs::remove_file(path);
        bail!(
            "{}: the checksum does not match, the file was deleted",
            path.file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default()
        );
    }
    Ok(())
}

/// Whether `path` has the checksum `expect`; the file is left as it is.
pub fn matches(path: &Path, expect: &Expect) -> Result<bool> {
    Ok(match expect {
        Expect::Sha256(want) => hex(&hash_file::<Sha256>(path)?).eq_ignore_ascii_case(want.trim()),
        Expect::Sha512(want) => hex(&hash_file::<Sha512>(path)?).eq_ignore_ascii_case(want.trim()),
        Expect::Sha3_256(want) => {
            hex(&hash_file::<Sha3_256>(path)?).eq_ignore_ascii_case(want.trim())
        }
        Expect::NpmIntegrity(want) => match want.trim().strip_prefix("sha512-") {
            Some(b64) => {
                base64::engine::general_purpose::STANDARD.encode(hash_file::<Sha512>(path)?) == b64
            }
            None => bail!("unsupported npm integrity: {want}"),
        },
    })
}

#[cfg(test)]
#[path = "../../../test/rust/net/verify.rs"]
mod tests;
