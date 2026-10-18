//! Git, Python, Node.js, uv, GitHub CLI (port of nabre/atollo
//! `_prepare-core.ps1:109-335` and `_prepare-finish.ps1:10-38`).

use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::{arch_of, github_single, pattern, target_of, Ctx, Installer, Release};
use crate::fsx::{join_rel, write_atomic};
use crate::kit::{Arch, Kit, Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::net::{github_digest, hash_in_list};
use crate::os::jobs;
use crate::settings::Settings;
use crate::setup::catalog::Component;

/// Git: Git for Windows Portable (`.7z.exe`) on Windows; on macOS and Linux
/// the relocatable build of GitHub Desktop (`desktop/dugite-native`, D-112):
/// `bin/git`, `libexec/git-core`, `share/git-core/templates` and its own
/// `etc/gitconfig`. It finds them through the kit's environment
/// (`GIT_EXEC_PATH`, `GIT_TEMPLATE_DIR`, `GIT_CONFIG_SYSTEM`, env.rs), so the
/// folder follows the kit when it moves, like PortableGit.
pub struct Git;

/// The dugite-native asset of `target` (`...-ubuntu-arm64.tar.gz`); `None`
/// on Windows, which keeps PortableGit. Only the 64-bit builds: never
/// `ubuntu-arm` (32-bit ARM) or `ubuntu-x86`.
pub fn dugite_asset_pattern(target: Target) -> Option<String> {
    let suffix = match (target.os, target.arch) {
        (Os::Win32, _) => return None,
        (Os::Darwin, Arch::Arm64) => "macOS-arm64",
        (Os::Darwin, Arch::X64) => "macOS-x64",
        (Os::Linux, Arch::X64) => "ubuntu-x64",
        (Os::Linux, Arch::Arm64) => "ubuntu-arm64",
    };
    Some(format!(
        r"^dugite-native-v[\d.]+-[0-9a-f]+-{suffix}\.tar\.gz$"
    ))
}

/// The version of a dugite-native release tag (`v2.53.0-4` → `2.53.0-4`):
/// Git's version with dugite's build number, so a new build of the same
/// Git is noticed as an update.
pub fn dugite_version(tag: &str) -> String {
    tag.trim_start_matches('v').to_owned()
}

/// The credential helper the kit's Git uses on macOS and Linux for hosts
/// other than GitHub (GitHub goes through `gh`, github/mod.rs): the
/// keychain when the build has `git-credential-osxkeychain`, otherwise
/// Git's memory cache for an hour. dugite-native ships neither osxkeychain
/// nor a configured Git Credential Manager (verified on v2.53.0-4), so
/// today it is the cache on both systems.
pub fn unix_credential_helper(has_osxkeychain: bool) -> &'static str {
    if has_osxkeychain {
        "osxkeychain"
    } else {
        "cache --timeout=3600"
    }
}

/// `text` (dugite's `etc/gitconfig`, the system config of the kit's Git
/// through `GIT_CONFIG_SYSTEM`) with `helper` as the credential helper;
/// `None` when a `[credential]` section already sets one (the user's
/// choice, or an earlier installation).
pub fn gitconfig_with_helper(text: &str, helper: &str) -> Option<String> {
    let mut in_credential = false;
    for line in text.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_credential = l.eq_ignore_ascii_case("[credential]");
            continue;
        }
        let key = l.split('=').next().unwrap_or_default().trim();
        if in_credential && key.eq_ignore_ascii_case("helper") {
            return None;
        }
    }
    let mut out = text.to_owned();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&format!(
        "# Atollo: credentials of hosts other than GitHub\n[credential]\n\thelper = {helper}\n"
    ));
    Some(out)
}

/// True when `dir`, the folder of the installed Git (its line, or `git`
/// itself in the layout before lines), is a conda environment
/// (`conda-meta`): the Git that Atollo installed off Windows before
/// dugite-native, which cannot follow the kit.
pub fn has_conda_git(dir: &Path) -> bool {
    dir.join("conda-meta").is_dir()
}

/// Git for `target` is staged when prepared on a host that cannot run it
/// (`runs_here` false, `can_execute_on_host`): dugite-native off Windows.
/// Its tarball has well over a hundred symbolic links
/// (`git-remote-https`, `git-upload-pack`, the builtins) that a Windows
/// host cannot make, and dropping them leaves a Git whose `--version`
/// works but that cannot clone over HTTPS. So the archive is only
/// downloaded and verified, and the first start on the target unpacks it
/// with its links (bootstrap.rs), as the conda-forge Git was before.
pub const fn git_staged(target: Target, runs_here: bool) -> bool {
    !target.is_windows() && !runs_here
}

/// The programs of dugite-native's `libexec/git-core` that are symbolic
/// links in its tarball, missing from `dest`: an archive unpacked where
/// links could not be made. `git --version` works without them, HTTPS
/// clones and local clones or pushes do not.
pub fn missing_git_links(dest: &Path) -> Vec<&'static str> {
    const LINKS: [&str; 3] = ["git-remote-https", "git-upload-pack", "git-receive-pack"];
    let core = join_rel(dest, "libexec/git-core");
    LINKS
        .into_iter()
        .filter(|name| !core.join(name).exists())
        .collect()
}

impl Installer for Git {
    /// Off Windows, on a host that cannot run the target's Git: staged
    /// (`git_staged`), never unpacked there. Everywhere else the shared
    /// path; `install_custom` is never reached, since `should_stage` is
    /// true exactly when this is.
    fn custom(&self, target: Target) -> bool {
        git_staged(target, crate::router::target::can_execute_on_host(target))
    }

    fn install_custom(
        &self,
        _ctx: &Ctx,
        _component: &Component,
        _release: &Release,
        _progress: &mut dyn super::Progress,
    ) -> Result<()> {
        bail!("git: dugite-native is staged for its target, never unpacked on this system")
    }

    /// A conda-forge Git left by an older Atollo on macOS or Linux is
    /// marked once to be installed again (the start replaces it with
    /// dugite-native, whose folder has no `conda-meta`): it is the only Git
    /// that cannot move with the kit. Only the installed line counts: an
    /// old line still in use is removed later (`drop_old_lines`) and does
    /// not mark it again. Nothing on Windows.
    fn relocate(&self, kit: &Kit, component: &Component) -> Result<()> {
        if kit.os().is_windows() || component.needs_reinstall(kit) {
            return Ok(());
        }
        if has_conda_git(&crate::versions::default_dir(kit, component)) {
            component.mark_reinstall(kit, "conda-forge git")?;
        }
        Ok(())
    }

    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        if let Some(re) = dugite_asset_pattern(target) {
            let rel = ctx.net.github_latest("desktop/dugite-native")?;
            let asset = rel
                .asset(&pattern(&re)?)
                .ok_or_else(|| anyhow!("dugite-native {}: no Git for {target}", rel.tag_name))?;
            // GitHub's digest, or the `.sha256` published next to the asset.
            return Ok(Release::single(
                dugite_version(&rel.tag_name),
                asset.browser_download_url.clone(),
                asset.name.clone(),
                Expect::Sha256(ctx.net.release_sha256(&rel, asset)?),
                Kind::TarGz,
                Inner::Root,
            ));
        }
        let rel = ctx.net.github_latest("git-for-windows/git")?;
        let suffix = if arch_of(ctx, component) == "arm64" {
            "arm64"
        } else {
            "64-bit"
        };
        let re = pattern(&format!(r"^PortableGit-(.+)-{suffix}\.7z\.exe$"))?;
        let asset = rel
            .assets
            .iter()
            .find(|a| re.is_match(&a.name))
            .ok_or_else(|| anyhow!("no PortableGit for {suffix} in {}", rel.tag_name))?;
        let version = re
            .captures(&asset.name)
            .and_then(|c| c.get(1))
            .map_or_else(|| rel.tag_name.clone(), |m| m.as_str().to_owned());
        Ok(Release::single(
            version,
            asset.browser_download_url.clone(),
            asset.name.clone(),
            Expect::Sha256(github_digest(asset)?),
            Kind::SevenZip,
            Inner::Root,
        ))
    }

    /// The self-extractor's post-install script, then `Set-KitGitConfig`:
    /// the system credential helper of the kit's Git is `manager` (not the
    /// selector window), and the portable home gets a `.gitconfig` once
    /// (main branch; any owner, for FAT32/exFAT sticks). On macOS and Linux
    /// the helper is written into the build's `etc/gitconfig` (no process:
    /// also when the kit is prepared from another system).
    fn finish(&self, ctx: &Ctx, component: &Component, dest: &Path) -> Result<()> {
        if !target_of(ctx, component).is_windows() {
            // Unpacked without its links: fail now, not at the first clone.
            let missing = missing_git_links(dest);
            if !missing.is_empty() {
                bail!(
                    "git: libexec/git-core without {} (symbolic links not unpacked)",
                    missing.join(", ")
                );
            }
            let etc = join_rel(dest, "etc/gitconfig");
            let text = fs::read_to_string(&etc).unwrap_or_default();
            let keychain = join_rel(dest, "libexec/git-core/git-credential-osxkeychain").is_file();
            if let Some(new) = gitconfig_with_helper(&text, unix_credential_helper(keychain)) {
                if let Some(dir) = etc.parent() {
                    fs::create_dir_all(dir)?;
                }
                write_atomic(&etc, new.as_bytes()).with_context(|| etc.display().to_string())?;
            }
        }
        if ctx.kit.os().is_windows() && crate::router::target::can_execute_on_host(ctx.kit.target())
        {
            if dest.join("post-install.bat").is_file() {
                let system_root = ctx.env.get("SystemRoot").unwrap_or(r"C:\Windows");
                let cmd = Path::new(system_root).join("System32").join("cmd.exe");
                let status = jobs::kit_command(&cmd, ctx.env)
                    .args(["/d", "/c", "post-install.bat"])
                    .current_dir(dest)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .context("Git post-install.bat")?;
                // The script deletes itself as its last line, so cmd always ends
                // with "batch file not found" (exit 1): success = script gone.
                if dest.join("post-install.bat").exists() {
                    bail!("Git post-install.bat failed ({status})");
                }
            }
            let git = dest.join("cmd").join("git.exe");
            let current = jobs::kit_command(&git, ctx.env)
                .args(["config", "--system", "--get", "credential.helper"])
                .output()
                .context("git config --get")?;
            let helper = String::from_utf8_lossy(&current.stdout).trim().to_owned();
            let re = pattern(r"(?i)helper-selector|git-credential-manager(\.exe)?$")?;
            if helper.is_empty() || re.is_match(&helper) {
                let status = jobs::kit_command(&git, ctx.env)
                    .args(["config", "--system", "credential.helper", "manager"])
                    .status()
                    .context("git config --system")?;
                if !status.success() {
                    bail!("git config --system credential.helper failed");
                }
            }
        }
        let gitconfig = ctx.kit.home().join(".gitconfig");
        if !gitconfig.exists() {
            fs::create_dir_all(ctx.kit.home())?;
            write_atomic(
                &gitconfig,
                b"[init]\n\tdefaultBranch = main\n[safe]\n\tdirectory = *\n",
            )?;
        }
        Ok(())
    }
}

/// Default minor version of Python (`-Python 3.13` of the kit).
pub const PYTHON_DEFAULT: &str = "3.13";

/// Python: python-build-standalone `install_only_stripped` (full CPython with
/// pip and venv), the newest patch of the chosen minor version.
pub struct Python;

fn python_minor(ctx: &Ctx) -> String {
    Settings::load(ctx.kit)
        .settings
        .python_version
        .filter(|v| pattern(r"^3\.\d+$").is_ok_and(|re| re.is_match(v)))
        .unwrap_or_else(|| PYTHON_DEFAULT.to_owned())
}

/// The minor line of the Python in `tree` (`3.14`), read from its files:
/// `python314.dll` on Windows, `lib/python3.14` elsewhere. The line chosen
/// in the settings may differ from the one installed (a project's line, a
/// change not applied yet). `None` when neither is there.
pub fn python_minor_in(tree: &Path, windows: bool) -> Option<String> {
    let (dir, re) = if windows {
        (tree.to_path_buf(), r"(?i)^python3(\d+)\.dll$")
    } else {
        (tree.join("lib"), r"^python3\.(\d+)$")
    };
    let re = pattern(re).ok()?;
    fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir() != windows))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            re.captures(&name)?.get(1)?.as_str().parse::<u32>().ok()
        })
        .max()
        .map(|n| format!("3.{n}"))
}

/// The newest `cpython-<minor>.N+<date>-<triple>-install_only_stripped.tar.gz`
/// in a SHA256SUMS file: (file name, patch version, hash).
pub fn pick_python(sums: &str, minor: &str, triple: &str) -> Option<(String, String, String)> {
    let re = regex::Regex::new(&format!(
        r"^([0-9a-f]{{64}})\s+(cpython-({}\.(\d+))\+\d+-{}-install_only_stripped\.tar\.gz)$",
        regex::escape(minor),
        regex::escape(triple)
    ))
    .ok()?;
    sums.lines()
        .filter_map(|l| re.captures(l.trim()))
        .max_by_key(|c| c[4].parse::<u32>().unwrap_or(0))
        .map(|c| (c[2].to_owned(), c[3].to_owned(), c[1].to_owned()))
}

/// The newest python-build-standalone release: its URL prefix and SHA256SUMS.
fn python_sums(ctx: &Ctx) -> Result<(String, String)> {
    let info: Value = ctx
        .net
        .json("https://raw.githubusercontent.com/astral-sh/python-build-standalone/latest-release/latest-release.json")?;
    let prefix = info
        .get("asset_url_prefix")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("python-build-standalone: no asset_url_prefix"))?
        .trim_end_matches('/')
        .to_owned();
    let sums = ctx.net.text(&format!("{prefix}/SHA256SUMS"))?;
    Ok((prefix, sums))
}

/// The minor versions with a build for `triple` in a SHA256SUMS file,
/// newest first.
pub fn python_lines(sums: &str, triple: &str) -> Vec<String> {
    let Ok(re) = regex::Regex::new(&format!(
        r"cpython-(\d+\.\d+)\.\d+\+\d+-{}-install_only_stripped\.tar\.gz$",
        regex::escape(triple)
    )) else {
        return Vec::new();
    };
    let mut lines: Vec<String> = sums
        .lines()
        .filter_map(|l| re.captures(l.trim()).map(|c| c[1].to_owned()))
        .collect();
    lines.sort_by_key(|l| std::cmp::Reverse(super::version_key(l)));
    lines.dedup();
    lines
}

impl Python {
    fn release(ctx: &Ctx, component: &Component, minor: &str) -> Result<Release> {
        let (prefix, sums) = python_sums(ctx)?;
        let triple = target_of(ctx, component).rust_triple();
        let (file, version, sha) = pick_python(&sums, minor, triple)
            .ok_or_else(|| anyhow!("no Python {minor} for {triple}"))?;
        Ok(Release::single(
            version,
            format!("{prefix}/{}", file.replace('+', "%2B")),
            file,
            Expect::Sha256(sha),
            Kind::TarGz,
            Inner::Named("python".into()),
        ))
    }
}

impl Installer for Python {
    /// The newest minor (the line marked "latest"), unless one is chosen in
    /// the settings: a ticked versioned language installs its newest line
    /// (D-81), so a fresh kit gets 3.15 once python-build-standalone has
    /// it. `PYTHON_DEFAULT` is only the fallback when no line is listed.
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let chosen = Settings::load(ctx.kit)
            .settings
            .python_version
            .filter(|v| pattern(r"^3\.\d+$").is_ok_and(|re| re.is_match(v)));
        let minor = match chosen {
            Some(m) => m,
            None => {
                let (_, sums) = python_sums(ctx)?;
                let triple = target_of(ctx, component).rust_triple();
                python_lines(&sums, triple)
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| PYTHON_DEFAULT.to_owned())
            }
        };
        Python::release(ctx, component, &minor)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let (_, sums) = python_sums(ctx)?;
        let triple = target_of(ctx, component).rust_triple();
        Ok(python_lines(&sums, triple))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        Python::release(ctx, component, line)
    }

    /// Keeps the user's packages: the list is saved in
    /// `data\python-packages-<old version>.txt`; with the same minor version,
    /// `Lib\site-packages` and `Scripts` (except pip) move to the new tree.
    fn before_swap(&self, ctx: &Ctx, component: &Component, tree: &Path, old: &Path) -> Result<()> {
        let old_python = if ctx.kit.os().is_windows() {
            old.join("python.exe")
        } else {
            old.join("bin").join("python3")
        };
        if !old_python.is_file() {
            return Ok(());
        }
        let old_version = component
            .installed_version(ctx.kit)
            .unwrap_or_else(|| "old".into());
        if crate::router::target::can_execute_on_host(ctx.kit.target()) {
            if let Ok(out) = jobs::kit_command(&old_python, ctx.env)
                .args([
                    "-m",
                    "pip",
                    "freeze",
                    "--all",
                    "--disable-pip-version-check",
                ])
                .output()
            {
                if out.status.success() {
                    let list = ctx
                        .kit
                        .data()
                        .join(format!("python-packages-{old_version}.txt"));
                    write_atomic(&list, &out.stdout)?;
                }
            }
        }
        // Packages are compatible only within the same minor version.
        let minor = |v: &str| v.split('.').take(2).collect::<Vec<_>>().join(".");
        let m = minor(&old_version);
        let new =
            python_minor_in(tree, ctx.kit.os().is_windows()).unwrap_or_else(|| python_minor(ctx));
        if m != new {
            return Ok(());
        }
        let is_pip = |name: &str| {
            let n = name.to_ascii_lowercase();
            n == "pip" || n.starts_with("pip-") || n.starts_with("pip.") || n.starts_with("pip3")
        };
        let subs: Vec<String> = if ctx.kit.os().is_windows() {
            vec!["Lib/site-packages".into(), "Scripts".into()]
        } else {
            vec![format!("lib/python{m}/site-packages"), "bin".into()]
        };
        for sub in &subs {
            let from = join_rel(old, sub);
            let to = join_rel(tree, sub);
            fs::create_dir_all(&to)?;
            for entry in fs::read_dir(&from).into_iter().flatten().flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let target = to.join(&name);
                if is_pip(&name) || target.exists() {
                    continue;
                }
                copy_tree(&entry.path(), &target)?;
            }
        }
        Ok(())
    }

    /// `python3.exe` next to `python.exe`, and pip from the bundled wheel.
    fn finish(&self, ctx: &Ctx, _component: &Component, dest: &Path) -> Result<()> {
        let (python, bundled) = if ctx.kit.os().is_windows() {
            let p = dest.join("python.exe");
            fs::copy(&p, dest.join("python3.exe")).context("python3.exe")?;
            (p, join_rel(dest, "Lib/ensurepip/_bundled"))
        } else {
            let p = dest.join("bin").join("python3");
            let m = python_minor_in(dest, false).unwrap_or_else(|| python_minor(ctx));
            (
                p,
                join_rel(dest, &format!("lib/python{m}/ensurepip/_bundled")),
            )
        };
        if !crate::router::target::can_execute_on_host(ctx.kit.target()) {
            return Ok(());
        }
        let wheel = fs::read_dir(&bundled)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .find(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("pip-"))
            });
        if let Some(wheel) = wheel {
            let status = jobs::kit_command(&python, ctx.env)
                .args([
                    "-m",
                    "pip",
                    "install",
                    "--no-index",
                    "--no-deps",
                    "--force-reinstall",
                    "--no-warn-script-location",
                    "--disable-pip-version-check",
                    "-q",
                ])
                .arg(&wheel)
                .status()
                .context("pip")?;
            if !status.success() {
                crate::log::warn(&format!(
                    "python: pip from the bundled wheel failed ({status})"
                ));
            }
        }
        Ok(())
    }
}

/// The packages kept from the old line. Links (and junctions) are skipped:
/// following one would copy what it points to outside the tree, and a cycle
/// would never end; the new line has its own `bin/python3`-style links.
fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(from).with_context(|| from.display().to_string())?;
    if meta.file_type().is_symlink() {
        crate::log::warn(&format!(
            "python: link not carried to the new line: {}",
            from.display()
        ));
        return Ok(());
    }
    if meta.is_dir() {
        fs::create_dir_all(to)?;
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
    } else {
        fs::copy(from, to).with_context(|| from.display().to_string())?;
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct NodeEntry {
    version: String,
    #[serde(default)]
    files: Vec<String>,
    /// The codename of an LTS release (`"Krypton"`), `false` for the others.
    #[serde(default)]
    lts: Value,
}

impl NodeEntry {
    fn is_lts(&self) -> bool {
        !matches!(self.lts, Value::Bool(false) | Value::Null)
    }

    fn major(&self) -> Option<u64> {
        super::lang::version_key(&self.version).first().copied()
    }
}

/// The release of `index` (newest first, as nodejs.org lists it) to
/// install with the archive `wanted`: the newest of `line` when one is
/// asked; otherwise the newest LTS (`lts` not `false`), or the newest
/// release when the index has no LTS for it. When the version installed
/// (`installed`) is newer than that LTS (a "Current" release installed by
/// an older Atollo), the newest of its major instead: an update never takes
/// the kit back to an older major.
pub fn node_pick<'a>(
    index: &'a [NodeEntry],
    wanted: &str,
    line: Option<&str>,
    installed: Option<&str>,
) -> Option<&'a NodeEntry> {
    let mut with_file = index.iter().filter(|e| e.files.iter().any(|f| f == wanted));
    if let Some(l) = line {
        return with_file.find(|e| crate::versions::line_of(&e.version).as_deref() == Some(l));
    }
    let all: Vec<&NodeEntry> = with_file.collect();
    let pick = all
        .iter()
        .find(|e| e.is_lts())
        .or_else(|| all.first())
        .copied()?;
    let key = super::lang::version_key;
    match installed {
        Some(v) if key(v) > key(&pick.version) => {
            let major = key(v).first().copied();
            all.iter()
                .find(|e| e.major() == major)
                .copied()
                .or(Some(pick))
        }
        _ => Some(pick),
    }
}

/// Node.js: the newest LTS with an archive for this target.
pub struct Node;

/// The `files` identifier used in `https://nodejs.org/dist/index.json` for `target`.
pub fn node_wanted_file(target: Target) -> String {
    match target.os {
        Os::Win32 => format!("win-{}-zip", target.arch.as_str()),
        Os::Darwin => format!("osx-{}-tar", target.arch.as_str()),
        Os::Linux => format!("linux-{}", target.arch.as_str()),
    }
}

/// The lines of Node.js with `wanted` (`win-x64-zip`, `osx-arm64-tar`,
/// `linux-x64`, ...) in `index.json` (newest first, as the index): the even
/// majors (LTS lines) from 18 on, the others only for their newest major.
pub fn node_lines(index: &[NodeEntry], wanted: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for e in index.iter().filter(|e| e.files.iter().any(|f| f == wanted)) {
        let Some(line) = crate::versions::line_of(&e.version) else {
            continue;
        };
        if !out.contains(&line) {
            out.push(line);
        }
    }
    let newest_major = out
        .first()
        .and_then(|l| l.split('.').next())
        .map(str::to_owned);
    out.retain(|l| {
        let major: u32 = l
            .split('.')
            .next()
            .and_then(|m| m.parse().ok())
            .unwrap_or(0);
        (major >= 18 && major.is_multiple_of(2))
            || newest_major.as_deref() == Some(&major.to_string())
    });
    out
}

impl Node {
    fn release(ctx: &Ctx, component: &Component, line: Option<&str>) -> Result<Release> {
        let target = target_of(ctx, component);
        let arch = target.arch.as_str();
        let node_os = target.node_os();
        let wanted = node_wanted_file(target);
        let index: Vec<NodeEntry> = ctx.net.json("https://nodejs.org/dist/index.json")?;
        let installed = component.installed_version(ctx.kit);
        let entry = node_pick(&index, &wanted, line, installed.as_deref())
            .ok_or_else(|| anyhow!("no Node.js {} for {node_os}-{arch}", line.unwrap_or("LTS")))?;
        let v = &entry.version;
        let (ext, kind) = if target.is_windows() {
            ("zip", Kind::Zip)
        } else {
            ("tar.gz", Kind::TarGz)
        };
        let stem = format!("node-{v}-{node_os}-{arch}");
        let file_name = format!("{stem}.{ext}");
        let sums = ctx
            .net
            .text(&format!("https://nodejs.org/dist/{v}/SHASUMS256.txt"))?;
        let sha = hash_in_list(&sums, &file_name)
            .ok_or_else(|| anyhow!("{file_name} missing in SHASUMS256.txt"))?;
        Ok(Release::single(
            v.clone(),
            format!("https://nodejs.org/dist/{v}/{file_name}"),
            file_name,
            Expect::Sha256(sha),
            kind,
            Inner::Named(stem),
        ))
    }
}

impl Installer for Node {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        Node::release(ctx, component, None)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let index: Vec<NodeEntry> = ctx.net.json("https://nodejs.org/dist/index.json")?;
        Ok(node_lines(
            &index,
            &node_wanted_file(target_of(ctx, component)),
        ))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        Node::release(ctx, component, Some(line))
    }
}

/// uv, uvx, uvw in `uv/<line>`.
pub struct Uv;

impl Installer for Uv {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let triple = target.rust_triple();
        let (ext, kind) = if target.is_windows() {
            (r"\.zip", Kind::Zip)
        } else {
            (r"\.tar\.gz", Kind::TarGz)
        };
        github_single(
            ctx,
            "astral-sh/uv",
            &format!(r"^uv-{}{ext}$", regex::escape(triple)),
            kind,
            Inner::Auto,
        )
    }
}

/// GitHub CLI in `gh/<line>` (`bin/gh(.exe)`).
pub struct Gh;

impl Installer for Gh {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let arch = target.go_arch();
        let (os_tag, ext, kind) = match target.os {
            Os::Win32 => ("windows", r"\.zip", Kind::Zip),
            Os::Darwin => ("macOS", r"\.zip", Kind::Zip),
            Os::Linux => ("linux", r"\.tar\.gz", Kind::TarGz),
        };
        github_single(
            ctx,
            "cli/cli",
            &format!(r"^gh_[\d.]+_{os_tag}_{arch}{ext}$"),
            kind,
            Inner::Auto,
        )
    }
}

#[cfg(test)]
#[path = "../../../test/rust/install/core.rs"]
mod tests;
