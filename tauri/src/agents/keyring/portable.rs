//! The pure parts of the keyring of macOS and Linux (names, go-keyring's
//! encodings, the sealing of the computer's secrets, file names), tested on
//! every system.
#![cfg_attr(not(any(unix, test)), allow(dead_code))]

use anyhow::{bail, Context, Result};
use base64::Engine;
use sha2::{Digest, Sha256};

use super::Item;

/// No credential store at all on this system (Linux without a Secret
/// Service), as opposed to a store that cannot be used now.
#[derive(Debug)]
pub struct NoStore(pub String);

impl std::fmt::Display for NoStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no Secret Service: {}", self.0)
    }
}

impl std::error::Error for NoStore {}

/// A Secret Service error that means "there is none" (nobody owns or can
/// start `org.freedesktop.secrets`, or no session bus), never a locked or
/// refused one.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub fn no_secret_service(error: &str) -> bool {
    [
        "org.freedesktop.secrets was not provided by any .service files",
        "org.freedesktop.DBus.Error.ServiceUnknown",
        "DBUS_SESSION_BUS_ADDRESS",
        "Unable to autolaunch a dbus-daemon",
    ]
    .iter()
    .any(|p| error.contains(p))
}

/// The go-keyring (service, user) pairs agy uses on macOS and Linux, where
/// the stores cannot be listed by a name pattern as on Windows. Verified on
/// agy 1.4.1 (17/10/2026): the Windows credential agy writes is
/// `gemini:antigravity` with user `antigravity` (go-keyring's target is
/// `<service>:<user>`), and the macOS and Linux builds of the same version
/// embed go-keyring (`zalando/go_keyring`, `macOSXKeychain`,
/// `secretServiceProvider`) and the strings `gemini` and `antigravity`.
pub const KNOWN: &[(&str, &str)] = &[("gemini", "antigravity")];

/// `(service, user)` of a go-keyring target `<service>:<user>`.
pub fn split_target(target: &str) -> Option<(&str, &str)> {
    target
        .split_once(':')
        .filter(|(s, u)| !s.is_empty() && !u.is_empty())
}

/// What go-keyring's `Set` stores on macOS: every password base64-encoded
/// behind a prefix (otherwise `security` returns non-ASCII text as hex).
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
pub const MAC_BASE64: &str = "go-keyring-base64:";
/// The older hex encoding go-keyring still reads on macOS.
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
pub const MAC_HEX: &str = "go-keyring-encoded:";

/// The value written in the Keychain for the secret `raw`, as go-keyring.
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
pub fn mac_encode(raw: &[u8]) -> String {
    format!(
        "{MAC_BASE64}{}",
        base64::engine::general_purpose::STANDARD.encode(raw)
    )
}

/// The secret of a Keychain value printed by `security -w`, as go-keyring's
/// `Get` reads it (a value without a prefix as it is).
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
pub fn mac_decode(value: &str) -> Result<Vec<u8>> {
    let value = value.trim();
    if let Some(rest) = value.strip_prefix(MAC_BASE64) {
        return base64::engine::general_purpose::STANDARD
            .decode(rest)
            .context("go-keyring base64 value");
    }
    if let Some(rest) = value.strip_prefix(MAC_HEX) {
        return hex_decode(rest).context("go-keyring hex value");
    }
    Ok(value.as_bytes().to_vec())
}

#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok())
        .collect()
}

/// A word `security -i` reads as it is (go-keyring's shellescape leaves
/// these unquoted): no quoting needed, nothing injected.
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
fn plain_word(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"@%+=:,./_-".contains(&b))
}

/// The line given on stdin to `security -i` to store `raw` (as go-keyring:
/// the secret never appears among a process's arguments).
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
pub fn mac_add_command(service: &str, user: &str, raw: &[u8]) -> Result<String> {
    let value = mac_encode(raw);
    if !(plain_word(service) && plain_word(user)) {
        bail!("{service}:{user}: unexpected characters for the Keychain");
    }
    Ok(format!(
        "add-generic-password -U -s {service} -a {user} -w {value}\n"
    ))
}

/// A stand-in for the last write time on macOS and Linux, whose stores do
/// not give one: the first 8 bytes of the secret's SHA-256.
pub fn fingerprint(blob: &str) -> i64 {
    let digest = Sha256::digest(blob.as_bytes());
    let mut first = [0u8; 8];
    first.copy_from_slice(&digest[..8]);
    i64::from_le_bytes(first)
}

/// An agy credential read from a store of macOS or Linux.
pub fn unix_item(service: &str, user: &str, raw: &[u8]) -> Item {
    let blob = base64::engine::general_purpose::STANDARD.encode(raw);
    Item {
        target: format!("{service}:{user}"),
        user: Some(user.to_owned()),
        written: Some(fingerprint(&blob)),
        blob: Some(blob),
        persist: None,
        kit: None,
    }
}

/// A secret sealed on macOS or Linux: `kc1:` + base64(nonce, data, tag).
pub const SEALED: &str = "kc1:";
pub const NONCE: usize = 16;
const TAG: usize = 32;

fn sha256(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

fn hmac_sha256(key: &[u8; 32], data: &[&[u8]]) -> [u8; 32] {
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for (i, k) in key.iter().enumerate() {
        ipad[i] ^= k;
        opad[i] ^= k;
    }
    let mut parts: Vec<&[u8]> = vec![&ipad];
    parts.extend_from_slice(data);
    let inner = sha256(&parts);
    sha256(&[&opad, &inner])
}

/// SHA-256 in counter mode as the key stream (encrypt and decrypt).
fn keystream_xor(key: &[u8; 32], nonce: &[u8], data: &mut [u8]) {
    let enc = sha256(&[b"atollo-agy-enc", key]);
    for (i, chunk) in data.chunks_mut(32).enumerate() {
        let block = sha256(&[&enc, nonce, &(i as u64).to_be_bytes()]);
        for (b, k) in chunk.iter_mut().zip(block) {
            *b ^= k;
        }
    }
}

fn mac_key(key: &[u8; 32]) -> [u8; 32] {
    sha256(&[b"atollo-agy-mac", key])
}

/// `blob` (base64) sealed with `key`: encrypt-then-MAC, SHA-256 in counter
/// mode and HMAC-SHA256 (no other crate needed).
pub fn seal(key: &[u8; 32], nonce: &[u8; NONCE], blob: &str) -> Result<String> {
    let mut data = base64::engine::general_purpose::STANDARD.decode(blob)?;
    keystream_xor(key, nonce, &mut data);
    let tag = hmac_sha256(&mac_key(key), &[nonce, &data]);
    let mut out = nonce.to_vec();
    out.extend_from_slice(&data);
    out.extend_from_slice(&tag);
    Ok(format!(
        "{SEALED}{}",
        base64::engine::general_purpose::STANDARD.encode(out)
    ))
}

/// The base64 secret sealed by [`seal`]; an altered value or another key
/// fail.
pub fn open_sealed(key: &[u8; 32], sealed: &str) -> Result<String> {
    let Some(rest) = sealed.strip_prefix(SEALED) else {
        bail!("not a sealed secret");
    };
    let raw = base64::engine::general_purpose::STANDARD.decode(rest)?;
    if raw.len() < NONCE + TAG {
        bail!("sealed secret too short");
    }
    let (nonce, rest) = raw.split_at(NONCE);
    let (data, tag) = rest.split_at(rest.len() - TAG);
    let expected = hmac_sha256(&mac_key(key), &[nonce, data]);
    // Constant time: every byte compared.
    if expected
        .iter()
        .zip(tag)
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        != 0
    {
        bail!("sealed secret: wrong key or altered");
    }
    let mut data = data.to_vec();
    keystream_xor(key, nonce, &mut data);
    Ok(base64::engine::general_purpose::STANDARD.encode(data))
}

/// A computer's name usable in a file name (`-` for anything else).
pub fn host_tag(name: &str) -> String {
    let tag: String = name
        .trim()
        .chars()
        .take(64)
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    if tag.is_empty() {
        "host".into()
    } else {
        tag
    }
}
