//! Authenticode signatures of the Windows programs the kit downloads
//! (`WinVerifyTrust`, `os/windows/signature.rs`): the signature and the
//! certificate chain against the Windows trust store, and the organization
//! (`O=`) of the signer exactly. Only Windows can check them: elsewhere
//! (a kit prepared for Windows from macOS or Linux) [`check`] answers
//! `None` and the file keeps its other checks (the SHA-256 of the catalog).

use std::path::Path;

use anyhow::Result;

use crate::os::current::signature as imp;

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)] // made on Windows only
pub enum Verdict {
    Valid {
        subject: String,
    },
    /// Signed by the right publisher, revocation not verifiable.
    RevocationUnknown {
        subject: String,
    },
}

/// Verifies the signature of `path` and that the organization of its
/// signer is `publisher` exactly; when the revocation check cannot
/// complete (offline, proxy) and the publisher is right, the verdict says
/// so, unless the signature is the only check of that file (`required`):
/// then an error. `None` where this system cannot check it.
pub fn check(path: &Path, publisher: &str, required: bool) -> Result<Option<Verdict>> {
    imp::check(path, publisher, required)
}
