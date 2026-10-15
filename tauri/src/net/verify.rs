//! Checksums of downloaded files (port of nabre/dev-kit
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
mod tests {
    use super::*;
    use crate::kit::tests::TempDir;

    #[test]
    fn known_hashes() {
        let tmp = TempDir::new("verify");
        let file = tmp.0.join("abc.txt");
        fs::write(&file, b"abc").unwrap();
        let sha256 = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        let sha512 = "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f";
        assert_eq!(sha256_hex(&file).unwrap(), sha256);
        check(&file, &Expect::Sha256(sha256.to_uppercase())).unwrap();
        check(&file, &Expect::Sha512(sha512.into())).unwrap();
        let sha3 = "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532";
        check(&file, &Expect::Sha3_256(sha3.into())).unwrap();
        let integrity = format!(
            "sha512-{}",
            base64::engine::general_purpose::STANDARD.encode(hash_file::<Sha512>(&file).unwrap())
        );
        check(&file, &Expect::NpmIntegrity(integrity)).unwrap();
    }

    #[test]
    fn mismatch_deletes_the_file() {
        let tmp = TempDir::new("verify-bad");
        let file = tmp.0.join("x.zip");
        fs::write(&file, b"tampered").unwrap();
        assert!(check(&file, &Expect::Sha256("00".repeat(32))).is_err());
        assert!(!file.exists());
    }
}
