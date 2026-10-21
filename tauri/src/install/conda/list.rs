//! The explicit list of a line's environment and its record, and the check
//! of the kept archives against them before an installation without the
//! network.

use std::fmt;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use super::CHANNEL;
use crate::net::verify::{self, Expect};

/// One archive of an explicit list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive {
    pub url: String,
    /// Its name in `pkgs/`, the last part of the URL.
    pub file: String,
    /// Lowercase hex.
    pub sha256: String,
}

impl Archive {
    /// The conda subdir of its URL (`linux-64`, `noarch`).
    pub fn subdir(&self) -> Option<&str> {
        self.url.strip_prefix(CHANNEL)?.split('/').next()
    }
}

/// The record of a list (`locks/<id>@<line>.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    /// The component's version, as saved.
    pub version: String,
    pub target: String,
    /// Unix seconds.
    pub created: u64,
    /// What `micromamba --version` said, for the diagnosis only: the
    /// explicit format does not change with it.
    pub micromamba: Option<String>,
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub file: String,
    pub sha256: String,
    pub size: u64,
}

pub(super) fn is_sha256(hash: &str) -> bool {
    hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit())
}

/// A file name of conda-forge that can only be a file of `pkgs/`: no
/// separator, not hidden, an archive's extension.
pub fn safe_file(name: &str) -> bool {
    !name.starts_with('.')
        && (name.ends_with(".conda") || name.ends_with(".tar.bz2"))
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._+-!".contains(c))
}

fn safe_subdir(subdir: &str) -> bool {
    !subdir.is_empty()
        && subdir
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// The archives of an explicit list: `@EXPLICIT`, then one `url#sha256`
/// per line; comments (`#`) and blank lines are skipped. Only conda-forge's
/// https URLs and plain file names are taken, and only SHA-256: an MD5
/// (what `micromamba env export` writes) is refused, since the list is what
/// the archives are checked against.
pub fn parse_explicit(text: &str) -> Result<Vec<Archive>> {
    let mut explicit = false;
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "@EXPLICIT" {
            explicit = true;
            continue;
        }
        if !explicit {
            bail!("not an explicit list: {line} before @EXPLICIT");
        }
        let (url, hash) = line
            .split_once('#')
            .ok_or_else(|| anyhow!("{line}: no hash"))?;
        let hash = hash.strip_prefix("sha256:").unwrap_or(hash);
        if !is_sha256(hash) {
            bail!("{url}: not a SHA-256");
        }
        let (subdir, file) = url
            .strip_prefix(CHANNEL)
            .and_then(|rest| rest.split_once('/'))
            .ok_or_else(|| anyhow!("{url}: not a package of conda-forge"))?;
        if !safe_subdir(subdir) || !safe_file(file) {
            bail!("{url}: not a package of conda-forge");
        }
        out.push(Archive {
            url: url.to_owned(),
            file: file.to_owned(),
            sha256: hash.to_ascii_lowercase(),
        });
    }
    if !explicit {
        bail!("not an explicit list: no @EXPLICIT");
    }
    if out.is_empty() {
        bail!("an explicit list without packages");
    }
    Ok(out)
}

/// The text of an explicit list of `archives` for `subdir`.
pub fn explicit_text(archives: &[Archive], subdir: &str) -> String {
    let mut out = format!(
        "# Atollo: the packages of a conda environment, kept to reinstall it without the network (D-118)\n# platform: {subdir}\n@EXPLICIT\n"
    );
    for a in archives {
        out.push_str(&format!("{}#{}\n", a.url, a.sha256));
    }
    out
}

/// Why an environment is installed from the network after a move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fallback {
    /// No list for the line: installed before D-118, freed, or kept
    /// packages turned off.
    NoList,
    /// The list or its record cannot be read.
    Unreadable(String),
    /// The list and its record name another archive or hash.
    Disagree(String),
    /// An archive of another system.
    OtherTarget(String),
    /// An archive is not there, or has another size.
    Missing(String),
    /// An archive has another SHA-256: never used.
    Changed(String),
    /// micromamba is not in `tools`.
    NoMicromamba,
}

impl fmt::Display for Fallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fallback::NoList => write!(f, "no kept packages"),
            Fallback::Unreadable(e) => {
                write!(f, "the list of the kept packages cannot be read ({e})")
            }
            Fallback::Disagree(file) => write!(f, "the list and its record disagree on {file}"),
            Fallback::OtherTarget(file) => write!(f, "{file} is a package of another system"),
            Fallback::Missing(file) => write!(f, "{file} is missing from the kept packages"),
            Fallback::Changed(file) => write!(f, "{file} does not have the SHA-256 of the list"),
            Fallback::NoMicromamba => write!(f, "micromamba is not in the kit"),
        }
    }
}

/// The decision before an installation without the network: the archives
/// of `list` when it and its `record` agree, every archive is for `subdir`
/// (or `noarch`) and every file of `pkgs` has the size and the SHA-256 of
/// the list; otherwise why not. The reference is the list, whose hashes
/// came from conda-forge's metadata; the record must say the same, so a
/// record changed alone does not let another archive through.
pub fn check(
    list: Option<&str>,
    record: Option<&str>,
    subdir: &str,
    pkgs: &Path,
) -> Result<Vec<Archive>, Fallback> {
    let (Some(list), Some(record)) = (list, record) else {
        return Err(Fallback::NoList);
    };
    let archives = parse_explicit(list).map_err(|e| Fallback::Unreadable(format!("{e:#}")))?;
    let record: Record =
        serde_json::from_str(record).map_err(|e| Fallback::Unreadable(format!("record: {e}")))?;
    if let Some(a) = archives
        .iter()
        .find(|a| !matches!(a.subdir(), Some(s) if s == subdir || s == "noarch"))
    {
        return Err(Fallback::OtherTarget(a.file.clone()));
    }
    if record.packages.len() != archives.len() {
        return Err(Fallback::Disagree(format!(
            "{} packages instead of {}",
            record.packages.len(),
            archives.len()
        )));
    }
    let mut sizes = Vec::with_capacity(archives.len());
    for a in &archives {
        let package = record
            .packages
            .iter()
            .find(|p| p.file == a.file)
            .filter(|p| p.sha256.eq_ignore_ascii_case(&a.sha256))
            .ok_or_else(|| Fallback::Disagree(a.file.clone()))?;
        sizes.push(package.size);
    }
    for (a, size) in archives.iter().zip(sizes) {
        match fs::metadata(pkgs.join(&a.file)) {
            Ok(m) if m.is_file() && m.len() == size => {}
            _ => return Err(Fallback::Missing(a.file.clone())),
        }
    }
    for a in &archives {
        let path = pkgs.join(&a.file);
        if !verify::matches(&path, &Expect::Sha256(a.sha256.clone())).unwrap_or(false) {
            return Err(Fallback::Changed(a.file.clone()));
        }
    }
    Ok(archives)
}

/// A `file://` URL of `path` for an explicit list: forward slashes, every
/// byte but the unreserved ones and `/` `:` percent-encoded (spaces and
/// parentheses of a kit's folder).
pub fn file_url(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let mut out = String::from("file://");
    if !text.starts_with('/') {
        out.push('/');
    }
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || b"/:-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
#[path = "../../../../test/rust/install/conda/list.rs"]
mod tests;
