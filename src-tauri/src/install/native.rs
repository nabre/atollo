//! Installers made of processes: Rust (rustup), pnpm and yarn (npm), R
//! (micromamba). Port of nabre/atollo `_prepare-rust.ps1` and
//! `_languages.ps1:254-429`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use windows::core::HSTRING;
use windows::Win32::System::Registry::{
    RegDeleteTreeW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ,
};

use super::{fetch, triple_arch, work_dir, Ctx, Installer, Progress, Release};
use crate::catalog::Component;
use crate::jobs;
use crate::kit::write_atomic;
use crate::net::first_sha256;
use crate::settings::Settings;
use crate::verify::Expect;

/// Runs a kit program, sending its output lines to the progress notes.
fn run(
    ctx: &Ctx,
    program: &Path,
    args: &[&str],
    envs: &[(&str, &Path)],
    progress: &mut dyn Progress,
) -> Result<()> {
    let mut cmd = jobs::kit_command(program, ctx.env);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = cmd
        .output()
        .with_context(|| program.display().to_string())?;
    for line in String::from_utf8_lossy(&out.stdout)
        .lines()
        .chain(String::from_utf8_lossy(&out.stderr).lines())
    {
        if !line.trim().is_empty() {
            progress.note(line.trim());
        }
    }
    if !out.status.success() {
        bail!(
            "{} failed ({})",
            program.file_name().unwrap_or_default().to_string_lossy(),
            out.status
        );
    }
    Ok(())
}

fn can_hard_link(dir: &Path) -> bool {
    let a = dir.join(".hardlink-test-a");
    let b = dir.join(".hardlink-test-b");
    let _ = fs::write(&a, b"x");
    let ok = fs::hard_link(&a, &b).is_ok();
    let _ = fs::remove_file(&b);
    let _ = fs::remove_file(&a);
    ok
}

// --- Rustup's registry entry ---------------------------------------------------
// rustup writes HKCU\...\Uninstall\Rustup at every install and self update,
// even with --no-modify-path. The kit puts it back as it was: removed if it
// was created now and points into the kit, old values restored otherwise.

const RUSTUP_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Rustup";
const RUSTUP_VALUES: [&str; 3] = ["UninstallString", "DisplayName", "DisplayVersion"];

fn reg_get(name: &str) -> Option<String> {
    let key = HSTRING::from(RUSTUP_KEY);
    let value = HSTRING::from(name);
    let mut buf = vec![0u16; 2048];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: buffer and size describe the same memory; read-only query.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            &key,
            &value,
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    r.is_ok()
        .then(|| String::from_utf16_lossy(&buf[..(size as usize / 2).saturating_sub(1)]))
}

/// `None` when the key does not exist.
fn rustup_key() -> Option<Vec<(String, Option<String>)>> {
    let values: Vec<_> = RUSTUP_VALUES
        .iter()
        .map(|n| ((*n).to_owned(), reg_get(n)))
        .collect();
    values.iter().any(|(_, v)| v.is_some()).then_some(values)
}

fn restore_rustup_key(before: Option<Vec<(String, Option<String>)>>, owned: &[&Path]) {
    let key = HSTRING::from(RUSTUP_KEY);
    let result = (|| -> Result<()> {
        let Some(now) = rustup_key() else {
            return Ok(());
        };
        match before {
            None => {
                let uninstall = now
                    .iter()
                    .find(|(n, _)| n == "UninstallString")
                    .and_then(|(_, v)| v.clone())
                    .unwrap_or_default();
                let ours = owned.iter().any(|p| {
                    uninstall
                        .to_lowercase()
                        .contains(&p.to_string_lossy().to_lowercase())
                });
                if uninstall.is_empty() || ours {
                    // SAFETY: deletes the subkey rustup created just now.
                    unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, &key) }.ok()?;
                }
            }
            Some(old) => {
                for (name, value) in old {
                    let Some(value) = value else { continue };
                    if reg_get(&name).as_deref() == Some(value.as_str()) {
                        continue;
                    }
                    let wide: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
                    // SAFETY: REG_SZ data with its terminating NUL, size in bytes.
                    unsafe {
                        RegSetKeyValueW(
                            HKEY_CURRENT_USER,
                            &key,
                            &HSTRING::from(name.as_str()),
                            REG_SZ.0,
                            Some(wide.as_ptr().cast()),
                            (wide.len() * 2) as u32,
                        )
                    }
                    .ok()?;
                }
            }
        }
        Ok(())
    })();
    if let Err(e) = result {
        crate::log::warn(&format!(
            "rustup wrote HKCU\\{RUSTUP_KEY}; remove it by hand ({e:#})"
        ));
    }
}

/// Rust: rustup with the stable toolchain, rustfmt and clippy, all in
/// `rust\` (`RUSTUP_HOME`, `CARGO_HOME`). Host `x86_64-pc-windows-gnu` by
/// default (own linker), `msvc` from the settings, always `msvc` on ARM64.
pub struct Rust;

fn rust_host(ctx: &Ctx) -> String {
    if ctx.kit.arch() == crate::kit::Arch::Arm64 {
        return "aarch64-pc-windows-msvc".into();
    }
    match Settings::load(ctx.kit).settings.rust_toolchain.as_deref() {
        Some("msvc") => "x86_64-pc-windows-msvc".into(),
        _ => "x86_64-pc-windows-gnu".into(),
    }
}

impl Installer for Rust {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let channel = ctx
            .net
            .text("https://static.rust-lang.org/dist/channel-rust-stable.toml")?;
        let version = super::pattern(r#"(?m)^\[pkg\.rustc\]\r?\nversion = "(\S+)"#)?
            .captures(&channel)
            .map(|c| c[1].to_owned())
            .ok_or_else(|| anyhow!("no stable version in channel-rust-stable.toml"))?;
        let triple = format!("{}-pc-windows-msvc", triple_arch(ctx, component));
        let url = format!("https://static.rust-lang.org/rustup/dist/{triple}/rustup-init.exe");
        let sha = first_sha256(&ctx.net.text(&format!("{url}.sha256"))?)
            .ok_or_else(|| anyhow!("rustup-init: no SHA-256"))?;
        Ok(Release::single(
            version,
            url,
            "rustup-init.exe".into(),
            Expect::Sha256(sha),
            crate::archive::Kind::File,
            crate::archive::Inner::Root,
        ))
    }

    fn custom(&self) -> bool {
        true
    }

    fn install_custom(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        if ctx.kit.arch() != crate::kit::Arch::detect() {
            bail!("Rust must be prepared on a PC of the kit's architecture (rustup runs the toolchain)");
        }
        let rust = ctx.kit.apps().join("rust");
        fs::create_dir_all(ctx.kit.data().join("tmp"))?;
        if !can_hard_link(&ctx.kit.data().join("tmp")) {
            bail!("rustup needs a disk with hard links (NTFS): this kit is on FAT32/exFAT");
        }
        let host = rust_host(ctx);
        let owned = [ctx.kit.root()];
        let rustup = rust.join(r"cargo\bin\rustup.exe");
        let envs_for = |root: &Path| -> [(&'static str, PathBuf); 3] {
            [
                ("RUSTUP_HOME", root.join("rustup")),
                ("CARGO_HOME", root.join("cargo")),
                ("RUSTUP_INIT_SKIP_PATH_CHECK", PathBuf::from("yes")),
            ]
        };
        let before = rustup_key();
        let result = (|| -> Result<()> {
            if rustup.is_file() && rust.join(r"cargo\bin\rustc.exe").is_file() {
                // In place: keeps the crates registry and the `cargo install` tools.
                progress.note("rustup update");
                let envs = envs_for(&rust);
                let envs: Vec<(&str, &Path)> =
                    envs.iter().map(|(k, v)| (*k, v.as_path())).collect();
                run(ctx, &rustup, &["update", "stable"], &envs, progress)?;
                run(
                    ctx,
                    &rustup,
                    &["set", "auto-self-update", "disable"],
                    &envs,
                    progress,
                )?;
                return Ok(());
            }
            let work = work_dir(ctx, component)?;
            let part = &release.parts[0];
            let init = fetch(
                ctx,
                &part.url,
                &part.file_name,
                &part.expect,
                &work,
                progress,
            )?;
            let root = work.join("rust");
            let envs = envs_for(&root);
            let envs: Vec<(&str, &Path)> = envs.iter().map(|(k, v)| (*k, v.as_path())).collect();
            progress.note(&format!("rustup-init: stable {host}"));
            run(
                ctx,
                &init,
                &[
                    "-y",
                    "--no-modify-path",
                    "--profile",
                    "minimal",
                    "--default-host",
                    &host,
                    "--default-toolchain",
                    "stable",
                    "-c",
                    "rustfmt",
                    "-c",
                    "clippy",
                ],
                &envs,
                progress,
            )?;
            run(
                ctx,
                &root.join(r"cargo\bin\rustup.exe"),
                &["set", "auto-self-update", "disable"],
                &envs,
                progress,
            )?;
            crate::archive::replace_dir(&root, &rust)?;
            let _ = fs::remove_dir_all(&work);
            Ok(())
        })();
        restore_rustup_key(before, &owned);
        result
    }
}

/// pnpm and yarn, installed globally with the kit's npm in `data\npm-global`
/// (their install scripts only print messages: not run).
pub struct Pnpm;

impl Installer for Pnpm {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        let version = |p: &str| -> Result<String> {
            let meta: Value = ctx
                .net
                .json(&format!("https://registry.npmjs.org/{p}/latest"))?;
            meta.get("version")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| anyhow!("{p}: no version"))
        };
        Ok(Release {
            version: format!("pnpm {}, yarn {}", version("pnpm")?, version("yarn")?),
            parts: Vec::new(),
        })
    }

    fn custom(&self) -> bool {
        true
    }

    fn install_custom(
        &self,
        ctx: &Ctx,
        _component: &Component,
        _release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        // npm-cli.js with the kit's node, no cmd.exe: its quoting breaks on
        // kit paths with `&` or parentheses (found in a test kit).
        let node = ctx.kit.apps().join(r"node\node.exe");
        let npm_cli = ctx.kit.apps().join(r"node\node_modules\npm\bin\npm-cli.js");
        if !node.is_file() || !npm_cli.is_file() {
            bail!("the kit's Node.js is needed for pnpm and yarn");
        }
        run(
            ctx,
            &node,
            &[
                &npm_cli.display().to_string(),
                "install",
                "--global",
                "--no-fund",
                "--no-audit",
                "--ignore-scripts",
                "pnpm@latest",
                "yarn@latest",
            ],
            &[],
            progress,
        )
    }
}

/// R from conda-forge (OpenBLAS), installed by micromamba in the work
/// folder and moved into the kit; launchers `bin\R.cmd`, `bin\Rscript.cmd`
/// put R's DLLs on the PATH of R only. x64 only (no conda-forge R for ARM64).
pub struct R;

impl Installer for R {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        let info: Value = ctx
            .net
            .json("https://api.anaconda.org/package/conda-forge/r-base")?;
        let version = info
            .get("latest_version")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("r-base: no version"))?;
        let mm = ctx.net.github_latest("mamba-org/micromamba-releases")?;
        let base = format!(
            "https://github.com/mamba-org/micromamba-releases/releases/download/{}",
            mm.tag_name
        );
        let sha = first_sha256(&ctx.net.text(&format!("{base}/micromamba-win-64.sha256"))?)
            .ok_or_else(|| anyhow!("micromamba: no SHA-256"))?;
        Ok(Release::single(
            // r-base only: a new micromamba alone does not rebuild R.
            version.to_owned(),
            format!("{base}/micromamba-win-64"),
            "micromamba.exe".into(),
            Expect::Sha256(sha),
            crate::archive::Kind::File,
            crate::archive::Inner::Root,
        ))
    }

    fn custom(&self) -> bool {
        true
    }

    fn install_custom(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        if ctx.kit.arch() != crate::kit::Arch::X64 {
            bail!("conda-forge has no R for Windows ARM64");
        }
        let work = work_dir(ctx, component)?;
        let result = (|| -> Result<()> {
            let part = &release.parts[0];
            let mm_new = fetch(
                ctx,
                &part.url,
                &part.file_name,
                &part.expect,
                &work,
                progress,
            )?;
            let tools = ctx.kit.apps().join("tools");
            fs::create_dir_all(&tools)?;
            let mm = tools.join("micromamba.exe");
            crate::archive::replace_file(&mm_new, &mm)?;
            let home = work.join("mamba-home");
            let target = work.join("r");
            let envs = [
                ("HOME", home.clone()),
                ("USERPROFILE", home.clone()),
                ("APPDATA", home.join(r"AppData\Roaming")),
                ("LOCALAPPDATA", home.join(r"AppData\Local")),
                ("MAMBA_ROOT_PREFIX", home.join("mamba")),
            ];
            let envs: Vec<(&str, &Path)> = envs.iter().map(|(k, v)| (*k, v.as_path())).collect();
            progress.note(&format!("micromamba: r-base {}", release.version));
            let spec = format!("r-base={}", release.version);
            run(
                ctx,
                &mm,
                &[
                    "create",
                    "--no-rc",
                    "-y",
                    "-q",
                    "-p",
                    &target.display().to_string(),
                    "-c",
                    "conda-forge",
                    "--override-channels",
                    &spec,
                    "libblas=*=*openblas",
                ],
                &envs,
                progress,
            )?;
            if !target.join(r"lib\R\bin\x64\R.exe").is_file() {
                bail!("R.exe not found after the installation");
            }
            let bin = target.join("bin");
            fs::create_dir_all(&bin)?;
            for exe in ["R", "Rscript"] {
                let cmd = format!(
                    "@echo off\r\nrem Atollo: R with its DLLs (Library\\bin) in the PATH of R only\r\nsetlocal\r\nset \"PATH=%~dp0..\\lib\\R\\bin\\x64;%~dp0..\\Library\\bin;%PATH%\"\r\n\"%~dp0..\\lib\\R\\bin\\x64\\{exe}.exe\" %*\r\nexit /b %ERRORLEVEL%\r\n"
                );
                write_atomic(&bin.join(format!("{exe}.cmd")), cmd.as_bytes())?;
                let sh = format!(
                    "#!/bin/sh\n# Atollo: R with its DLLs (Library/bin) in the PATH of R only\nd=$(dirname \"$0\")/..\nPATH=\"$d/lib/R/bin/x64:$d/Library/bin:$PATH\" exec \"$d/lib/R/bin/x64/{exe}.exe\" \"$@\"\n"
                );
                write_atomic(&bin.join(exe), sh.as_bytes())?;
            }
            let site = target.join(r"lib\R\etc\Rprofile.site");
            let mut text = fs::read_to_string(&site).unwrap_or_default();
            text.push_str("\n# Atollo: CRAN without asking for a mirror\nlocal({ r <- getOption('repos'); if (is.null(r) || identical(unname(r['CRAN']), '@CRAN@')) options(repos = c(CRAN = 'https://cloud.r-project.org')) })\n");
            fs::write(&site, text)?;
            let _ = fs::remove_dir_all(target.join("pkgs"));
            crate::archive::replace_dir(&target, &ctx.kit.apps().join("r"))
        })();
        let _ = fs::remove_dir_all(&work);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::tests::TempDir;

    #[test]
    fn hard_links_on_the_test_disk() {
        let tmp = TempDir::new("hardlink");
        // The test folder is on the system disk (NTFS).
        assert!(can_hard_link(&tmp.0));
        assert!(
            fs::read_dir(&tmp.0).unwrap().next().is_none(),
            "probe files removed"
        );
    }
}
