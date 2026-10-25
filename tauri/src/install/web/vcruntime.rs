//! The Visual C++ runtime of PHP, Apache and the database servers on
//! Windows: its DLLs extracted from Microsoft's `vc_redist.x64.exe` and
//! copied next to the programs.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{anyhow, bail, Context, Result};

use crate::common::fsx::join_rel;
use crate::install::{pattern, Ctx, Progress};

/// A progress sink that only logs, for steps that run inside hooks.
pub(super) struct LogProgress;

impl Progress for LogProgress {
    fn note(&mut self, text: &str) {
        crate::common::log::info(text);
    }
    fn bytes(&mut self, _done: u64, _total: Option<u64>) {}
}

fn expand_cab(ctx: &Ctx, cab: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    let system_root = ctx.env.get("SystemRoot").unwrap_or(r"C:\Windows");
    let status = crate::platform::jobs::kit_command(
        Path::new(system_root).join(r"System32\expand.exe"),
        ctx.env,
    )
    .arg(cab)
    .arg("-F:*")
    .arg(dest)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .context("expand.exe")?;
    if !status.success() {
        bail!("expand.exe failed on {} ({status})", cab.display());
    }
    Ok(())
}

/// Offsets and sizes of the CAB archives inside `bytes` (header `MSCF`,
/// version 1.3, size within the file).
pub fn find_cabs(bytes: &[u8]) -> Vec<(usize, usize)> {
    const MAGIC: &[u8] = b"MSCF\0\0\0\0";
    let mut out = Vec::new();
    let mut pos = 0;
    while pos + 26 <= bytes.len() {
        match bytes[pos..].windows(MAGIC.len()).position(|w| w == MAGIC) {
            None => break,
            Some(i) => {
                let at = pos + i;
                if at + 26 > bytes.len() {
                    break;
                }
                let size = u32::from_le_bytes([
                    bytes[at + 8],
                    bytes[at + 9],
                    bytes[at + 10],
                    bytes[at + 11],
                ]) as usize;
                if size > 64
                    && at + size <= bytes.len()
                    && bytes[at + 24] == 3
                    && bytes[at + 25] == 1
                {
                    out.push((at, size));
                    pos = at + size;
                } else {
                    pos = at + 4;
                }
            }
        }
    }
    out
}

/// The organization (`O=`) that signs the Visual C++ runtime.
const MICROSOFT: &str = "Microsoft Corporation";

/// The DLLs of the Visual C++ runtime in `tools\vcruntime`, extracted once.
pub fn vc_runtime(ctx: &Ctx, force: bool) -> Result<PathBuf> {
    let folder = join_rel(ctx.kit.apps(), "tools/vcruntime");
    if folder.join("vcruntime140.dll").is_file() && !force {
        return Ok(folder);
    }
    // In the host's temporary folder (D-90), removed when `guard` is dropped.
    let guard = crate::platform::temp::TempWork::new(ctx.kit, "setup/vcruntime")?;
    let work = guard.path();
    let result = (|| -> Result<()> {
        let exe = work.join("vc_redist.x64.exe");
        ctx.net.download(
            "https://aka.ms/vc14/vc_redist.x64.exe",
            &exe,
            ctx.cancel,
            &mut |_, _| {},
        )?;
        // The signer's organization, exactly: the only check of this file.
        crate::platform::signature::check(&exe, MICROSOFT, true)?;
        let bytes = fs::read(&exe)?;
        let cabs = find_cabs(&bytes);
        if cabs.len() < 2 {
            bail!("vc_redist.x64.exe: unknown format");
        }
        for (n, (at, size)) in cabs.iter().enumerate() {
            let cab = work.join(format!("c{n}.cab"));
            fs::write(&cab, &bytes[*at..*at + *size])?;
            expand_cab(ctx, &cab, &work.join(format!("c{n}")))?;
        }
        // The Burn manifest names the cab of the "Minimum" runtime.
        let manifest = walk(work)
            .into_iter()
            .filter(|p| p.file_name().is_some_and(|n| n == "0"))
            .find_map(|p| {
                fs::read_to_string(&p)
                    .ok()
                    .filter(|t| t.contains("BurnManifest"))
            })
            .ok_or_else(|| anyhow!("vc_redist.x64.exe: manifest not found"))?;
        let re = pattern(r#"<Payload\b[^>]*>"#)?;
        let source = re
            .find_iter(&manifest)
            .map(|m| m.as_str())
            .find(|tag| tag.contains(r"vcRuntimeMinimum_amd64\cab1.cab"))
            .and_then(|tag| {
                pattern(r#"SourcePath="([^"]+)""#)
                    .ok()?
                    .captures(tag)
                    .map(|c| c[1].to_owned())
            })
            .ok_or_else(|| anyhow!("vc_redist.x64.exe: no vcRuntimeMinimum x64"))?;
        let dll_cab = walk(work)
            .into_iter()
            .find(|p| p.file_name().is_some_and(|n| n.to_string_lossy() == source))
            .ok_or_else(|| anyhow!("vc_redist.x64.exe: {source} not found"))?;
        let dlls = work.join("dll");
        expand_cab(ctx, &dll_cab, &dlls)?;
        for entry in fs::read_dir(&dlls)?.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            match name.strip_suffix("_amd64").filter(|n| n.ends_with(".dll")) {
                Some(dll) => fs::rename(entry.path(), dlls.join(dll))?,
                None => fs::remove_file(entry.path())?,
            }
        }
        let key = dlls.join("vcruntime140.dll");
        if !key.is_file() {
            bail!("vcruntime140.dll not found in the Microsoft package");
        }
        crate::platform::signature::check(&key, MICROSOFT, true)?;
        crate::net::archive::stage_into(&dlls, &folder)
    })();
    drop(guard);
    result.map(|()| folder)
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).into_iter().flatten().flatten() {
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(e.path());
            } else {
                out.push(e.path());
            }
        }
    }
    out
}

pub fn copy_vc_runtime(ctx: &Ctx, dest: &Path) -> Result<()> {
    if !ctx.kit.os().is_windows() {
        return Ok(());
    }
    let vc = vc_runtime(ctx, false)?;
    for entry in fs::read_dir(vc)?.flatten() {
        if entry
            .path()
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("dll"))
        {
            fs::copy(entry.path(), dest.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/rust/install/web/vcruntime.rs"]
mod tests;
