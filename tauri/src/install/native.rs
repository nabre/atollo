//! Installers made of processes: Rust (rustup), pnpm and yarn (npm), R
//! (micromamba). Port of nabre/dev-kit `_prepare-rust.ps1` and
//! `_languages.ps1:254-429`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
#[cfg(windows)]
use windows::core::HSTRING;
#[cfg(windows)]
use windows::Win32::System::Registry::{
    RegDeleteTreeW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ,
};

use super::{fetch, target_of, work_dir, Ctx, Installer, Progress, Release};
use crate::fsx::{join_rel, write_atomic};
use crate::kit::Target;
use crate::net::first_sha256;
use crate::net::verify::Expect;
use crate::os::jobs;
use crate::settings::Settings;
use crate::setup::catalog::Component;

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

fn rustup_bin(rust: &Path, target: Target) -> PathBuf {
    join_rel(rust, &format!("cargo/bin/rustup{}", target.exe_suffix()))
}

fn rustup_installed(rust: &Path, target: Target) -> bool {
    let sfx = target.exe_suffix();
    join_rel(rust, &format!("cargo/bin/rustup{sfx}")).is_file()
        && join_rel(rust, &format!("cargo/bin/rustc{sfx}")).is_file()
}

#[derive(Debug, PartialEq, Eq)]
enum RustPlan {
    /// Installed on a kit with hard links: `rustup update` where it is.
    InPlace,
    /// rustup-init in the temp, result copied into the kit.
    Fresh,
    /// Kit without hard links: update a copy in the temp, copy it back.
    CopyUpdate,
}

/// Where rustup runs. In place only on a kit with hard links; otherwise in
/// the temp folder, which must have them (the probe runs only then). A temp
/// folder without hard links is the one case that still fails.
fn rust_plan(
    installed: bool,
    kit_links: bool,
    temp_links: impl FnOnce() -> bool,
) -> Result<RustPlan> {
    if installed && kit_links {
        return Ok(RustPlan::InPlace);
    }
    if !temp_links() {
        bail!("rustup needs a disk with hard links (NTFS) for its work folder: the temp folder of this PC has none");
    }
    Ok(if installed {
        RustPlan::CopyUpdate
    } else {
        RustPlan::Fresh
    })
}

// --- Rustup's registry entry ---------------------------------------------------
// rustup writes HKCU\...\Uninstall\Rustup at every install and self update,
// even with --no-modify-path. The kit puts it back as it was: removed if it
// was created now and points into the kit, old values restored otherwise.

#[cfg(windows)]
const RUSTUP_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Rustup";
#[cfg(windows)]
const RUSTUP_VALUES: [&str; 3] = ["UninstallString", "DisplayName", "DisplayVersion"];

#[cfg(windows)]
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
#[cfg(windows)]
fn rustup_key() -> Option<Vec<(String, Option<String>)>> {
    let values: Vec<_> = RUSTUP_VALUES
        .iter()
        .map(|n| ((*n).to_owned(), reg_get(n)))
        .collect();
    values.iter().any(|(_, v)| v.is_some()).then_some(values)
}

#[cfg(not(windows))]
fn rustup_key() -> Option<Vec<(String, Option<String>)>> {
    None
}

#[cfg(windows)]
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

#[cfg(not(windows))]
fn restore_rustup_key(_before: Option<Vec<(String, Option<String>)>>, _owned: &[&Path]) {}

/// Rust: rustup with the stable toolchain, rustfmt and clippy, all in
/// `rust/` (`RUSTUP_HOME`, `CARGO_HOME`). Host `x86_64-pc-windows-gnu` by
/// default on Windows x64 (own linker), `msvc` from the settings, or the
/// native Rust target triple on Windows ARM64, macOS and Linux.
pub struct Rust;

fn rust_host(ctx: &Ctx) -> String {
    if ctx.kit.target() != Target::WIN32_X64 {
        return ctx.kit.target().rust_triple().into();
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
        let target = target_of(ctx, component);
        let triple = target.rust_triple();
        let file_name = format!("rustup-init{}", target.exe_suffix());
        let url = format!("https://static.rust-lang.org/rustup/dist/{triple}/{file_name}");
        let sha = first_sha256(&ctx.net.text(&format!("{url}.sha256"))?)
            .ok_or_else(|| anyhow!("rustup-init: no SHA-256"))?;
        Ok(Release::single(
            version,
            url,
            file_name,
            Expect::Sha256(sha),
            crate::net::archive::Kind::File,
            crate::net::archive::Inner::Root,
        ))
    }

    fn custom(&self, _target: Target) -> bool {
        true
    }

    fn install_custom(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        if ctx.kit.target() != Target::detect() {
            bail!("Rust must be prepared on a machine of the kit's target (rustup runs the toolchain)");
        }
        let target = ctx.kit.target();
        let rust = ctx.kit.apps().join("rust");
        let installed = rustup_installed(&rust, target);
        // Hard links are probed where rustup runs: in place, the kit's own
        // disk; otherwise the work folder in the host temp (normally NTFS).
        let kit_links = installed && {
            fs::create_dir_all(ctx.kit.apps())?;
            can_hard_link(&ctx.kit.apps())
        };
        let in_place = rust_plan(installed, kit_links, || false).ok() == Some(RustPlan::InPlace);
        let host = rust_host(ctx);
        let owned = [ctx.kit.root()];
        let rustup = rustup_bin(&rust, target);
        let envs_for = |root: &Path| -> [(&'static str, PathBuf); 3] {
            [
                ("RUSTUP_HOME", root.join("rustup")),
                ("CARGO_HOME", root.join("cargo")),
                ("RUSTUP_INIT_SKIP_PATH_CHECK", PathBuf::from("yes")),
            ]
        };
        let before = rustup_key();
        let result = (|| -> Result<()> {
            if in_place {
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
            let guard = work_dir(ctx, component)?;
            let work = guard.path();
            let plan = rust_plan(installed, kit_links, || can_hard_link(work))?;
            let root = work.join("rust");
            if plan == RustPlan::CopyUpdate {
                progress.note("rustup update (copy in the temp folder)");
                crate::net::archive::check_space(&rust, work)?;
                crate::net::archive::copy_tree(&rust, &root)
                    .with_context(|| format!("copy {}", rust.display()))?;
                let envs = envs_for(&root);
                let envs: Vec<(&str, &Path)> =
                    envs.iter().map(|(k, v)| (*k, v.as_path())).collect();
                let tool = rustup_bin(&root, target);
                run(ctx, &tool, &["update", "stable"], &envs, progress)?;
                run(
                    ctx,
                    &tool,
                    &["set", "auto-self-update", "disable"],
                    &envs,
                    progress,
                )?;
                crate::net::archive::stage_into(&root, &rust)?;
                return Ok(());
            }
            let part = &release.parts[0];
            let init = fetch(
                ctx,
                &part.url,
                &part.file_name,
                &part.expect,
                work,
                progress,
            )?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(m) = fs::metadata(&init) {
                    let mut p = m.permissions();
                    p.set_mode(p.mode() | 0o755);
                    let _ = fs::set_permissions(&init, p);
                }
            }
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
                &rustup_bin(&root, target),
                &["set", "auto-self-update", "disable"],
                &envs,
                progress,
            )?;
            crate::net::archive::stage_into(&root, &rust)?;
            Ok(())
        })();
        restore_rustup_key(before, &owned);
        result
    }
}

/// pnpm and yarn, installed globally with the kit's npm in `data/npm-global`
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
            version: format!(
                "pnpm {}, corepack {}",
                version("pnpm")?,
                version("corepack")?
            ),
            parts: Vec::new(),
        })
    }

    fn custom(&self, _target: Target) -> bool {
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
        let node = crate::setup::catalog::kit_path(ctx.kit, "node/node.exe");
        let npm_cli = if ctx.kit.os().is_windows() {
            crate::setup::catalog::kit_path(ctx.kit, "node/node_modules/npm/bin/npm-cli.js")
        } else {
            crate::setup::catalog::kit_path(ctx.kit, "node/lib/node_modules/npm/bin/npm-cli.js")
        };
        if !node.is_file() || !npm_cli.is_file() {
            bail!("the kit's Node.js is needed for pnpm and yarn");
        }
        let npm = |args: &[&str], progress: &mut dyn Progress| -> Result<()> {
            let cli = npm_cli.display().to_string();
            let mut all = vec![cli.as_str(), "--global", "--no-fund", "--no-audit"];
            all.extend_from_slice(args);
            run(ctx, &node, &all, &[], progress)
        };
        let _ = npm(&["uninstall", "yarn", "corepack"], progress);
        npm(
            &["install", "--ignore-scripts", "corepack@latest"],
            progress,
        )?;
        let prefix = ctx.kit.data().join("npm-global");
        let corepack = if ctx.kit.os().is_windows() {
            join_rel(&prefix, "node_modules/corepack/dist/corepack.js")
        } else {
            let lib = join_rel(&prefix, "lib/node_modules/corepack/dist/corepack.js");
            if lib.is_file() {
                lib
            } else {
                join_rel(&prefix, "node_modules/corepack/dist/corepack.js")
            }
        };
        let bin_dir = if ctx.kit.os().is_windows() {
            prefix.clone()
        } else {
            prefix.join("bin")
        };
        fs::create_dir_all(&bin_dir)?;
        run(
            ctx,
            &node,
            &[
                &corepack.display().to_string(),
                "enable",
                "yarn",
                "--install-directory",
                &bin_dir.display().to_string(),
            ],
            &[],
            progress,
        )?;
        npm(
            &["install", "--force", "--allow-scripts=pnpm", "pnpm@latest"],
            progress,
        )
    }
}

/// R from conda-forge (OpenBLAS), installed by micromamba in the work
/// folder and moved into the kit; launchers `bin/R(.cmd)`, `bin/Rscript(.cmd)`
/// put R's libraries on the PATH of R only. Unavailable on `win32-arm64`.
pub struct R;

/// The description of the conda-forge package `pkg`: every file of every
/// version, with its `attrs.subdir`.
pub(super) fn conda_info(ctx: &Ctx, pkg: &str) -> Result<Value> {
    ctx.net.json(&format!(
        "https://api.anaconda.org/package/conda-forge/{pkg}"
    ))
}

/// The release versions of a conda-forge package (`info` from `conda_info`)
/// built for `subdir` (`win-64`, `osx-arm64`, `linux-64`, ...), newest
/// first; pre-releases (`4.7.0rc1`) are left out.
pub fn conda_versions(info: &Value, subdir: &str) -> Vec<String> {
    let mut out: Vec<String> = info
        .get("files")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|f| f.pointer("/attrs/subdir").and_then(Value::as_str) == Some(subdir))
        .filter_map(|f| f.get("version").and_then(Value::as_str))
        .filter(|v| super::lang::is_release(v))
        .map(str::to_owned)
        .collect();
    out.sort_by_key(|v| std::cmp::Reverse(super::lang::version_key(v)));
    out.dedup();
    out
}

/// `version` of a conda package with the newest micromamba that installs
/// it: the release's one part is micromamba itself (`micromamba_create`).
pub(super) fn micromamba_release(ctx: &Ctx, version: &str) -> Result<Release> {
    let target = ctx.kit.target();
    let asset = target.micromamba_asset();
    let mm = ctx.net.github_latest("mamba-org/micromamba-releases")?;
    let base = format!(
        "https://github.com/mamba-org/micromamba-releases/releases/download/{}",
        mm.tag_name
    );
    let sha = first_sha256(&ctx.net.text(&format!("{base}/{asset}.sha256"))?)
        .ok_or_else(|| anyhow!("micromamba: no SHA-256"))?;
    Ok(Release::single(
        // The package only: a new micromamba alone does not rebuild it.
        version.to_owned(),
        format!("{base}/{asset}"),
        format!("micromamba{}", target.exe_suffix()),
        Expect::Sha256(sha),
        crate::net::archive::Kind::File,
        crate::net::archive::Inner::Root,
    ))
}

/// The arguments of `micromamba create` for an environment at `prefix` with
/// `specs` (`git=2.51.0`), from conda-forge only and without any `.condarc`.
pub fn micromamba_create_args(prefix: &Path, specs: &[&str]) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "create".to_owned(),
        "--no-rc".to_owned(),
        "-y".to_owned(),
        "-q".to_owned(),
        "-p".to_owned(),
        prefix.display().to_string(),
        "-c".to_owned(),
        "conda-forge".to_owned(),
        "--override-channels".to_owned(),
    ];
    args.extend(specs.iter().map(|s| (*s).to_owned()));
    args
}

/// Creates the conda environment `prefix` with `specs`, by the micromamba
/// of `release` (`micromamba_release`), put into `tools`. Any old `prefix`
/// goes first. Its home, root prefix and package cache are in the work
/// folder, so nothing is written outside the kit and the temporary folder.
/// The environment is created where it stays: conda writes `prefix` into
/// some of its files, so it cannot be moved (D-92).
pub(super) fn micromamba_create(
    ctx: &Ctx,
    component: &Component,
    release: &Release,
    prefix: &Path,
    specs: &[&str],
    progress: &mut dyn Progress,
) -> Result<()> {
    let part = release
        .parts
        .first()
        .ok_or_else(|| anyhow!("{}: no micromamba to download", component.id))?;
    let guard = work_dir(ctx, component)?;
    let work = guard.path();
    let result = (|| -> Result<()> {
        let mm_new = fetch(
            ctx,
            &part.url,
            &part.file_name,
            &part.expect,
            work,
            progress,
        )?;
        let tools = ctx.kit.apps().join("tools");
        fs::create_dir_all(&tools)?;
        let mm = tools.join(format!("micromamba{}", ctx.kit.os().exe_suffix()));
        crate::net::archive::replace_file(&mm_new, &mm)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(m) = fs::metadata(&mm) {
                let mut p = m.permissions();
                p.set_mode(p.mode() | 0o755);
                let _ = fs::set_permissions(&mm, p);
            }
        }
        let home = work.join("mamba-home");
        // The same line again (a patch): the old folder goes first.
        if prefix.exists() {
            crate::net::archive::remove_dir(prefix)?;
        }
        let envs = [
            ("HOME", home.clone()),
            ("USERPROFILE", home.clone()),
            ("APPDATA", join_rel(&home, "AppData/Roaming")),
            ("LOCALAPPDATA", join_rel(&home, "AppData/Local")),
            ("MAMBA_ROOT_PREFIX", home.join("mamba")),
        ];
        let envs: Vec<(&str, &Path)> = envs.iter().map(|(k, v)| (*k, v.as_path())).collect();
        progress.note(&format!("micromamba: {}", specs.join(" ")));
        let args = micromamba_create_args(prefix, specs);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        run(ctx, &mm, &args, &envs, progress)?;
        // The package cache is in the work folder; a `pkgs` left in the
        // environment is only downloaded archives.
        let _ = fs::remove_dir_all(prefix.join("pkgs"));
        Ok(())
    })();
    drop(guard);
    result
}

/// Packages compiled for one minor version of R do not load in another:
/// one folder per line (versioned, D-81), `renv.lock` declares R's version.
impl Installer for R {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        let subdir = ctx.kit.target().conda_subdir();
        let versions = conda_versions(&conda_info(ctx, "r-base")?, subdir);
        let version = versions
            .first()
            .ok_or_else(|| anyhow!("r-base: no version for {subdir}"))?;
        micromamba_release(ctx, version)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        let subdir = ctx.kit.target().conda_subdir();
        let mut lines: Vec<String> = Vec::new();
        for v in conda_versions(&conda_info(ctx, "r-base")?, subdir) {
            if let Some(line) = crate::versions::line_of(&v) {
                if !lines.contains(&line) {
                    lines.push(line);
                }
            }
        }
        Ok(lines)
    }

    fn release_line(&self, ctx: &Ctx, _component: &Component, line: &str) -> Result<Release> {
        let subdir = ctx.kit.target().conda_subdir();
        let versions = conda_versions(&conda_info(ctx, "r-base")?, subdir);
        let version = versions
            .iter()
            .find(|v| crate::versions::line_of(v).as_deref() == Some(line))
            .ok_or_else(|| anyhow!("R {line} not available for {subdir}"))?;
        micromamba_release(ctx, version)
    }

    fn custom(&self, _target: Target) -> bool {
        true
    }

    fn install_custom(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        let target = crate::versions::install_dir(ctx.kit, component, &release.version);
        self.install_custom_at(ctx, component, release, &target, progress)
    }

    /// The conda environment is created where it stays: conda writes its
    /// folder into some files (Makeconf, pkgconfig), so a moved one points at
    /// a folder that no longer exists.
    fn install_custom_at(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        target: &Path,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        if ctx.kit.target() == Target::WIN32_ARM64 {
            bail!("conda-forge has no R for Windows ARM64");
        }
        let spec = format!("r-base={}", release.version);
        micromamba_create(
            ctx,
            component,
            release,
            target,
            &[&spec, "libblas=*=*openblas"],
            progress,
        )?;
        if ctx.kit.os().is_windows() {
            if !join_rel(target, "lib/R/bin/x64/R.exe").is_file() {
                bail!("R.exe not found after the installation");
            }
            let bin = target.join("bin");
            fs::create_dir_all(&bin)?;
            for exe in ["R", "Rscript"] {
                let cmd = format!(
                        "@echo off\r\nrem Dev-Kit: R with its DLLs (Library\\bin) in the PATH of R only\r\nsetlocal\r\nset \"PATH=%~dp0..\\lib\\R\\bin\\x64;%~dp0..\\Library\\bin;%PATH%\"\r\n\"%~dp0..\\lib\\R\\bin\\x64\\{exe}.exe\" %*\r\nexit /b %ERRORLEVEL%\r\n"
                    );
                write_atomic(&bin.join(format!("{exe}.cmd")), cmd.as_bytes())?;
                let sh = format!(
                        "#!/bin/sh\n# Dev-Kit: R with its DLLs (Library/bin) in the PATH of R only\nd=$(dirname \"$0\")/..\nPATH=\"$d/lib/R/bin/x64:$d/Library/bin:$PATH\" exec \"$d/lib/R/bin/x64/{exe}.exe\" \"$@\"\n"
                    );
                write_atomic(&bin.join(exe), sh.as_bytes())?;
            }
        } else if !join_rel(target, "bin/R").is_file() {
            bail!("bin/R not found after the installation");
        }
        let site = join_rel(target, "lib/R/etc/Rprofile.site");
        let mut text = fs::read_to_string(&site).unwrap_or_default();
        text.push_str("\n# Dev-Kit: CRAN without asking for a mirror\nlocal({ r <- getOption('repos'); if (is.null(r) || identical(unname(r['CRAN']), '@CRAN@')) options(repos = c(CRAN = 'https://cloud.r-project.org')) })\n");
        fs::write(&site, text)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../test/rust/install/native.rs"]
mod tests;
