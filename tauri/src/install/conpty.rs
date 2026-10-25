//! The kit's ConPTY (DECISIONS.md D-148): `conpty.dll` and
//! `OpenConsole.exe` of Microsoft's NuGet package
//! `Microsoft.Windows.Console.ConPTY`, the console host of Windows Terminal
//! and VS Code, for the app's terminal on Windows (`editor/conpty.rs`).
//! Part of the app: the version and the SHA-256 of the package are fixed in
//! the catalog for this release (the SHA-512 NuGet publishes checked when
//! it was fixed); both files must also be signed by Microsoft.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::{fetch, target_of, work_dir, Ctx, Installer, Progress, Release};
use crate::kit::Target;
use crate::net::archive::{self, Inner, Kind};
use crate::net::verify::Expect;
use crate::platform::signature::Verdict;
use crate::setup::Component;

/// The signer of both files.
const PUBLISHER: &str = "Microsoft Corporation";

pub struct ConPty;

/// The package of `version` on NuGet: its address and file name.
pub(crate) fn package(version: &str) -> (String, String) {
    let name = format!("microsoft.windows.console.conpty.{version}.nupkg");
    (
        format!("https://api.nuget.org/v3-flatcontainer/microsoft.windows.console.conpty/{version}/{name}"),
        name,
    )
}

/// The two files for `target` inside the package: where each is, and its
/// name in the component's folder (`OpenConsole.exe` next to `conpty.dll`,
/// where the DLL looks for it first).
pub(crate) fn files_for(target: Target) -> [(String, &'static str); 2] {
    let arch = target.arch.as_str();
    [
        (
            format!("runtimes/win-{arch}/native/conpty.dll"),
            "conpty.dll",
        ),
        (
            format!("build/native/runtimes/{arch}/OpenConsole.exe"),
            "OpenConsole.exe",
        ),
    ]
}

impl Installer for ConPty {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let (Some(version), Some(sha)) = (&component.version, component.sha256_for(target)) else {
            bail!("{}: no version fixed in the catalog", component.id);
        };
        let (url, name) = package(version);
        Ok(Release::single(
            version.clone(),
            url,
            name,
            Expect::Sha256(sha.to_owned()),
            Kind::Zip,
            Inner::Root,
        ))
    }

    /// Two files out of one package: picked by hand, not the whole tree.
    fn custom(&self, target: Target) -> bool {
        target.is_windows()
    }

    fn install_custom(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        let Some(part) = release.parts.first() else {
            bail!("{}: nothing to install", component.id);
        };
        let Some(folder) = component.folder_path(ctx.kit) else {
            bail!("{} has no folder of its own", component.id);
        };
        let work = work_dir(ctx, component)?;
        let file = fetch(
            ctx,
            &part.url,
            &part.file_name,
            &part.expect,
            work.path(),
            progress,
        )?;
        let extracted = work.path().join("x");
        archive::extract(&file, part.kind, &extracted)?;
        let tree = work.path().join("tree");
        fs::create_dir_all(&tree)?;
        for (from, name) in files_for(target_of(ctx, component)) {
            let src: PathBuf = crate::common::fsx::join_rel(&extracted, &from);
            if !src.is_file() {
                bail!("{from} not found in {}", part.file_name);
            }
            signed(&src, progress)?;
            fs::copy(&src, tree.join(name)).with_context(|| format!("copying {from}"))?;
        }
        archive::clean_leftovers(&ctx.kit.apps());
        archive::stage_into(&tree, &folder)?;
        drop(work);
        Ok(())
    }
}

/// Signed by Microsoft (Authenticode), checked where the system can.
fn signed(file: &Path, progress: &mut dyn Progress) -> Result<()> {
    match crate::platform::signature::check(file, PUBLISHER, false)? {
        Some(Verdict::Valid { subject }) => {
            progress.note(&format!("signature ok: {subject}"));
        }
        Some(Verdict::RevocationUnknown { subject }) => progress.note(&format!(
            "signature of {subject}: revocation not verifiable (offline?)"
        )),
        None => {}
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/install/conpty.rs"]
mod tests;
