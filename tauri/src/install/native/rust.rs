//! Rust: rustup with the stable toolchain in `rust/`, run in place or in
//! the temp folder by where hard links work, with rustup's registry entry
//! put back as it was.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
#[cfg(windows)]
use windows::core::HSTRING;
#[cfg(windows)]
use windows::Win32::System::Registry::{
    RegDeleteTreeW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ,
};

use super::run;
use crate::common::fsx::join_rel;
use crate::common::settings::Settings;
use crate::install::{fetch, pattern, target_of, work_dir, Ctx, Installer, Progress, Release};
use crate::kit::Target;
use crate::net::first_sha256;
use crate::net::verify::Expect;
use crate::setup::Component;

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
        crate::common::log::warn(&format!(
            "rustup wrote HKCU\\{RUSTUP_KEY}; remove it by hand ({e:#})"
        ));
    }
}

#[cfg(not(windows))]
fn restore_rustup_key(_before: Option<Vec<(String, Option<String>)>>, _owned: &[&Path]) {}

/// Rust: rustup with the stable toolchain, rustfmt and clippy, all in
/// `rust/` (`RUSTUP_HOME`, `CARGO_HOME`). Host `x86_64-pc-windows-gnu` by
/// default on Windows x64 (own linker), `aarch64-pc-windows-gnullvm` on
/// Windows ARM64 (linked by the clang of llvm-mingw, the `cc` component:
/// `CARGO_TARGET_AARCH64_PC_WINDOWS_GNULLVM_LINKER` in env/unix.rs), `msvc` from
/// the settings on both, the native Rust target triple on macOS and Linux.
pub struct Rust;

/// The host toolchain of Rust on `target` with the "Compilatore di Rust"
/// setting (`toolchain`: `msvc`, `gnu` or none). MSVC needs Visual Studio
/// on the PC; GNU is all in the kit.
pub(crate) fn rust_host_for(target: Target, toolchain: Option<&str>) -> &'static str {
    let msvc = toolchain == Some("msvc");
    match target {
        Target::WIN32_X64 if msvc => "x86_64-pc-windows-msvc",
        Target::WIN32_X64 => "x86_64-pc-windows-gnu",
        Target::WIN32_ARM64 if msvc => "aarch64-pc-windows-msvc",
        Target::WIN32_ARM64 => "aarch64-pc-windows-gnullvm",
        _ => target.rust_triple(),
    }
}

/// The `gnullvm` host links with llvm-mingw (the `cc` component): without
/// it `cargo build` stops at the link (D-112).
pub(crate) fn host_needs_cc(host: &str) -> bool {
    host.ends_with("-gnullvm")
}

fn rust_host(ctx: &Ctx) -> String {
    let target = ctx.kit.target();
    if !target.is_windows() {
        return target.rust_triple().into();
    }
    let settings = Settings::load(ctx.kit).settings;
    rust_host_for(target, settings.rust_toolchain.as_deref()).into()
}

/// The arguments of `rustup-init` for a first installation with `host`:
/// stable, minimal profile plus rustfmt and clippy, PATH left alone.
pub(crate) fn rustup_init_args(host: &str) -> Vec<&str> {
    vec![
        "-y",
        "--no-modify-path",
        "--profile",
        "minimal",
        "--default-host",
        host,
        "--default-toolchain",
        "stable",
        "-c",
        "rustfmt",
        "-c",
        "clippy",
    ]
}

impl Installer for Rust {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let channel = ctx
            .net
            .text("https://static.rust-lang.org/dist/channel-rust-stable.toml")?;
        let version = pattern(r#"(?m)^\[pkg\.rustc\]\r?\nversion = "(\S+)"#)?
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
        if host_needs_cc(&host) && !crate::setup::kit_path(ctx.kit, "cc/bin/clang.exe").is_file() {
            progress.note("C, C++ is not installed: Rust on Windows ARM64 links with its clang");
        }
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
            run(ctx, &init, &rustup_init_args(&host), &envs, progress)?;
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

#[cfg(test)]
#[path = "../../../../test/rust/install/native/rust.rs"]
mod tests;
