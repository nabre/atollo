//! The kit of each sandbox (D-169), the source of truth of what makes it:
//! written by Atollo in `data\app\sandboxes\<sandbox>\`, never fetched.
//!
//! - `spec.yaml`: a kit v2 `mixin` of Docker Sandboxes, given to `sbx` with
//!   `--kit` when the sandbox is made; a change later goes in with
//!   `sbx kit add`, which makes its container again and keeps its disk
//!   (home, `/tmp`, the VM's Docker: tried with sbx 0.49.2);
//! - `atollo.json`: whose sandbox it is (a project, a kit terminal and its
//!   folder), for the list of the VMs and their removal with their owner;
//! - `.applied`: the fingerprint of the kit `sbx` has.
//!
//! What a kit cannot say stays with Atollo: the folders mounted (`sbx run`'s
//! arguments), the daemon's settings, and what changes at every session
//! (the tools' `PATH` and lines, given by `sbx exec -e`).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::kit::Kit;

/// Whose sandbox it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Owner {
    /// A project of the kit.
    Project { project: String },
    /// A kit terminal (its id) in a folder.
    Terminal { terminal: String, dir: String },
}

impl Owner {
    fn kind(&self) -> &'static str {
        match self {
            Owner::Project { .. } => "project",
            Owner::Terminal { .. } => "terminal",
        }
    }

    /// The name shown: the project, or the terminal and its folder.
    pub fn label(&self) -> String {
        match self {
            Owner::Project { project } => project.clone(),
            Owner::Terminal { terminal, dir } => format!("{terminal} · {dir}"),
        }
    }
}

/// The folders of the sandboxes' kits: `data\app\sandboxes`.
pub fn root(kit: &Kit) -> PathBuf {
    kit.app_dir().join("sandboxes")
}

/// The folder of the kit of `sandbox`.
pub fn dir(kit: &Kit, sandbox: &str) -> PathBuf {
    root(kit).join(sandbox)
}

/// The kit's name in the spec: `atollo-` and 24 hex digits of the sandbox's
/// name (the spec takes at most 64 characters, `[a-z0-9-]`).
pub fn kit_name(sandbox: &str) -> String {
    let hash = Sha256::digest(sandbox.as_bytes());
    let hex: String = hash[..12].iter().map(|b| format!("{b:02x}")).collect();
    format!("atollo-{hex}")
}

/// A YAML scalar that is only ever text: JSON's quoting, which YAML reads.
fn quoted(text: &str) -> String {
    serde_json::Value::String(text.to_owned()).to_string()
}

/// The `spec.yaml` of `sandbox` for `owner` (LF only: the same bytes, the
/// same fingerprint, on every system).
pub fn spec_yaml(sandbox: &str, owner: &Owner) -> String {
    let mut lines = vec![
        "schemaVersion: \"2\"".to_owned(),
        "kind: mixin".to_owned(),
        format!("name: {}", kit_name(sandbox)),
        format!(
            "displayName: {}",
            quoted(&format!("Atollo: {}", owner.label()))
        ),
        format!(
            "description: {}",
            quoted("Written by Atollo (D-169): what makes this sandbox.")
        ),
        "environment:".to_owned(),
        "  variables:".to_owned(),
        format!("    ATOLLO_SANDBOX: {}", quoted(sandbox)),
        format!("    ATOLLO_OWNER: {}", quoted(owner.kind())),
    ];
    if let Owner::Project { project } = owner {
        lines.push(format!("    ATOLLO_PROJECT: {}", quoted(project)));
    }
    lines.join("\n") + "\n"
}

/// The fingerprint of a kit's text.
pub fn fingerprint(spec: &str) -> String {
    let hash = Sha256::digest(spec.as_bytes());
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

/// Writes the kit of `sandbox` for `owner` (`spec.yaml`, `atollo.json`);
/// its folder and fingerprint.
pub fn write(kit: &Kit, sandbox: &str, owner: &Owner) -> Result<(PathBuf, String)> {
    let dir = dir(kit, sandbox);
    std::fs::create_dir_all(&dir).with_context(|| dir.display().to_string())?;
    let spec = spec_yaml(sandbox, owner);
    write_if_changed(&dir.join("spec.yaml"), &spec)?;
    write_if_changed(
        &dir.join("atollo.json"),
        &(serde_json::to_string_pretty(owner)? + "\n"),
    )?;
    Ok((dir, fingerprint(&spec)))
}

fn write_if_changed(path: &Path, text: &str) -> Result<()> {
    if std::fs::read_to_string(path).ok().as_deref() != Some(text) {
        std::fs::write(path, text).with_context(|| path.display().to_string())?;
    }
    Ok(())
}

/// The fingerprint of the kit `sbx` has for `sandbox`, if any.
pub fn applied(kit: &Kit, sandbox: &str) -> Option<String> {
    std::fs::read_to_string(dir(kit, sandbox).join(".applied"))
        .ok()
        .map(|s| s.trim().to_owned())
}

/// Notes that `sbx` has the kit of fingerprint `print` for `sandbox`.
pub fn mark_applied(kit: &Kit, sandbox: &str, print: &str) -> Result<()> {
    let path = dir(kit, sandbox).join(".applied");
    std::fs::write(&path, format!("{print}\n")).with_context(|| path.display().to_string())
}

/// The folders `sbx` mounted in `sandbox` when it was made (one per line:
/// the workspace, then the read-only ones), if Atollo noted them: they
/// cannot change after (`sbx run`'s arguments, not the kit's).
pub fn mounts(kit: &Kit, sandbox: &str) -> Option<String> {
    std::fs::read_to_string(dir(kit, sandbox).join(".mounts")).ok()
}

/// Notes the folders mounted in `sandbox` at its making.
pub fn mark_mounts(kit: &Kit, sandbox: &str, mounts: &str) -> Result<()> {
    let path = dir(kit, sandbox).join(".mounts");
    std::fs::write(&path, mounts).with_context(|| path.display().to_string())
}

/// The owner of `sandbox`, from its kit, if Atollo wrote one.
pub fn owner(kit: &Kit, sandbox: &str) -> Option<Owner> {
    let text = std::fs::read_to_string(dir(kit, sandbox).join("atollo.json")).ok()?;
    serde_json::from_str(&text).ok()
}

/// Every sandbox with a kit of Atollo, with its owner.
pub fn all(kit: &Kit) -> Vec<(String, Owner)> {
    let mut out: Vec<(String, Owner)> = std::fs::read_dir(root(kit))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            Some((name.clone(), owner(kit, &name)?))
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Removes the kit of `sandbox` (after the sandbox).
pub fn remove(kit: &Kit, sandbox: &str) -> Result<()> {
    let dir = dir(kit, sandbox);
    match std::fs::remove_dir_all(&dir) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(e).with_context(|| dir.display().to_string())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
#[path = "../../../test/rust/isolation/kit.rs"]
mod tests;
