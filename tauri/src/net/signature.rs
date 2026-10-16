//! Authenticode signatures (port of nabre/dev-kit `Assert-Signature`,
//! `_prepare-helpers.ps1:110-130`).
//!
//! `WinVerifyTrust` checks the signature and the certificate chain against
//! the Windows trust store, with revocation of the whole chain; the subject
//! of the signer certificate must match the expected publisher. When the
//! revocation check cannot complete (offline, proxy) and the publisher is
//! right, the result is a warning, unless the signature is the only check of
//! that file (`required`).

use std::path::Path;

use anyhow::{bail, Result};
use regex::Regex;
use windows::core::{GUID, HSTRING, PCWSTR};
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::Security::Cryptography::{
    CertNameToStrW, CERT_X500_NAME_STR, X509_ASN_ENCODING,
};
use windows::Win32::Security::WinTrust::{
    WTHelperGetProvSignerFromChain, WTHelperProvDataFromStateData, WinVerifyTrust,
    WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_DATA_0, WINTRUST_FILE_INFO,
    WTD_CHOICE_FILE, WTD_REVOKE_WHOLECHAIN, WTD_STATEACTION_CLOSE, WTD_STATEACTION_VERIFY,
    WTD_UI_NONE,
};

/// HRESULTs of a revocation check that could not complete.
const REVOCATION_UNKNOWN: [u32; 2] = [
    0x8009_2013, // CRYPT_E_REVOCATION_OFFLINE
    0x800B_010E, // CERT_E_REVOCATION_FAILURE
];

#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Valid {
        subject: String,
    },
    /// Signed by the right publisher, revocation not verifiable.
    RevocationUnknown {
        subject: String,
    },
}

/// Verifies the signature of `path` and that its signer matches `publisher`.
pub fn check(path: &Path, publisher: &Regex, required: bool) -> Result<Verdict> {
    let file = HSTRING::from(path.as_os_str());
    let mut file_info = WINTRUST_FILE_INFO {
        cbStruct: size_of::<WINTRUST_FILE_INFO>() as u32,
        pcwszFilePath: PCWSTR(file.as_ptr()),
        hFile: HANDLE::default(),
        pgKnownSubject: std::ptr::null_mut(),
    };
    let mut data = WINTRUST_DATA {
        cbStruct: size_of::<WINTRUST_DATA>() as u32,
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: WTD_REVOKE_WHOLECHAIN,
        dwUnionChoice: WTD_CHOICE_FILE,
        Anonymous: WINTRUST_DATA_0 {
            pFile: &mut file_info,
        },
        dwStateAction: WTD_STATEACTION_VERIFY,
        ..Default::default()
    };
    let mut action: GUID = WINTRUST_ACTION_GENERIC_VERIFY_V2;
    // SAFETY: `data` and `file_info` live until the CLOSE call below.
    let status = unsafe {
        WinVerifyTrust(
            HWND::default(),
            &mut action,
            std::ptr::from_mut(&mut data).cast(),
        )
    } as u32;
    let subject = signer_subject(data.hWVTStateData);
    data.dwStateAction = WTD_STATEACTION_CLOSE;
    // SAFETY: releases the state allocated by the VERIFY call.
    unsafe {
        WinVerifyTrust(
            HWND::default(),
            &mut action,
            std::ptr::from_mut(&mut data).cast(),
        )
    };

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let publisher_ok = subject.as_deref().is_some_and(|s| publisher.is_match(s));
    match (status, publisher_ok) {
        (0, true) => Ok(Verdict::Valid {
            subject: subject.unwrap_or_default(),
        }),
        (s, true) if REVOCATION_UNKNOWN.contains(&s) && !required => {
            Ok(Verdict::RevocationUnknown {
                subject: subject.unwrap_or_default(),
            })
        }
        (0, false) => bail!(
            "{name}: signed by an unexpected publisher ({})",
            subject.unwrap_or_default()
        ),
        (s, _) => bail!("{name}: signature not valid (0x{s:08X})"),
    }
}

/// Subject of the first signer certificate, as `CN=…, O=…`.
fn signer_subject(state: HANDLE) -> Option<String> {
    // SAFETY: the provider data belongs to `state`, still open; every pointer
    // is checked before use and nothing is kept after the function.
    unsafe {
        let prov = WTHelperProvDataFromStateData(state);
        if prov.is_null() {
            return None;
        }
        let signer = WTHelperGetProvSignerFromChain(prov, 0, false, 0);
        if signer.is_null() || (*signer).csCertChain == 0 || (*signer).pasCertChain.is_null() {
            return None;
        }
        let cert = (*(*signer).pasCertChain).pCert;
        if cert.is_null() || (*cert).pCertInfo.is_null() {
            return None;
        }
        let name = &(*(*cert).pCertInfo).Subject;
        let len = CertNameToStrW(X509_ASN_ENCODING, name, CERT_X500_NAME_STR, None);
        if len <= 1 {
            return None;
        }
        let mut buf = vec![0u16; len as usize];
        CertNameToStrW(X509_ASN_ENCODING, name, CERT_X500_NAME_STR, Some(&mut buf));
        Some(String::from_utf16_lossy(&buf[..len as usize - 1]))
    }
}

#[cfg(test)]
#[path = "../../../test/rust/net/signature.rs"]
mod tests;
