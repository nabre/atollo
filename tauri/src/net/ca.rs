//! The extra certificates of the settings (`network.extraCaCerts`, the CA of
//! a firewall that inspects HTTPS) for the app and for the kit's tools
//! (D-137).
//!
//! - The app's downloads ([`super::Net`]) trust the system's roots plus the
//!   certificates of that PEM file.
//! - `data/app/ca-bundle.pem` holds the same roots and certificates for the
//!   tools that read one bundle instead of the system's store (OpenSSL's
//!   `SSL_CERT_FILE`, pip, requests, curl, Git, Cargo): it is written when
//!   the kit's environment is built, removed when the setting is emptied.
//!   Node.js gets only the PEM file (`NODE_EXTRA_CA_CERTS` adds to its own).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use base64::Engine;
use ureq::tls::{parse_pem, Certificate, PemItem};

use crate::kit::Kit;
use crate::settings::Settings;

/// The variables of the tools that read one bundle of roots.
pub const BUNDLE_VARS: &[&str] = &[
    "SSL_CERT_FILE",
    "REQUESTS_CA_BUNDLE",
    "PIP_CERT",
    "CURL_CA_BUNDLE",
    "GIT_SSL_CAINFO",
    "CARGO_HTTP_CAINFO",
];

pub fn bundle_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("ca-bundle.pem")
}

/// The certificates of the PEM file `file`; an error when it has none.
pub fn extra_certs(file: &Path) -> Result<Vec<Certificate<'static>>> {
    let pem = fs::read(file).with_context(|| file.display().to_string())?;
    let certs: Vec<_> = parse_pem(&pem)
        .filter_map(|item| match item {
            Ok(PemItem::Certificate(c)) => Some(c),
            _ => None,
        })
        .collect();
    if certs.is_empty() {
        bail!("no certificate in {}", file.display());
    }
    Ok(certs)
}

/// The system's roots and the certificates of `file`.
pub fn roots_with(file: &Path) -> Result<Vec<Certificate<'static>>> {
    let system = rustls_native_certs::load_native_certs();
    for e in &system.errors {
        crate::log::warn(&format!("system certificates: {e}"));
    }
    let mut all: Vec<Certificate<'static>> = system
        .certs
        .iter()
        .map(|der| Certificate::from_der(der.as_ref()).to_owned())
        .collect();
    all.extend(extra_certs(file)?);
    Ok(all)
}

/// `certs` as PEM text.
fn pem(certs: &[Certificate<'_>]) -> String {
    let mut out = String::new();
    for c in certs {
        let b64 = base64::engine::general_purpose::STANDARD.encode(c.der());
        out.push_str("-----BEGIN CERTIFICATE-----\n");
        for line in b64.as_bytes().chunks(64) {
            out.push_str(&String::from_utf8_lossy(line));
            out.push('\n');
        }
        out.push_str("-----END CERTIFICATE-----\n");
    }
    out
}

/// Writes `data/app/ca-bundle.pem` for the extra file of `settings`, or
/// removes it when there is none or it cannot be read (the tools then keep
/// their own roots; the Settings page already refused a file without
/// certificates).
pub fn refresh(kit: &Kit, settings: &Settings) {
    let file = bundle_file(kit);
    let extra = settings
        .network
        .extra_ca_certs
        .as_deref()
        .filter(|f| !f.is_empty());
    let written = extra.map(|extra| {
        roots_with(Path::new(extra)).and_then(|certs| {
            crate::fsx::write_atomic(&file, pem(&certs).as_bytes()).map_err(Into::into)
        })
    });
    match written {
        Some(Ok(())) => {}
        Some(Err(e)) => {
            crate::log::warn(&format!("certificate bundle: {e:#}"));
            let _ = fs::remove_file(&file);
        }
        None => {
            let _ = fs::remove_file(&file);
        }
    }
}

#[cfg(test)]
#[path = "../../../test/rust/net/ca.rs"]
mod tests;
