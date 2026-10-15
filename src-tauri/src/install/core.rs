//! Git, Python, Node.js, uv, GitHub CLI (port of nabre/dev-kit
//! `_prepare-core.ps1:109-335` and `_prepare-finish.ps1:10-38`).

use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::{arch_of, github_single, pattern, triple_arch, Ctx, Dest, Installer, Release};
use crate::archive::{Inner, Kind};
use crate::catalog::Component;
use crate::jobs;
use crate::kit::write_atomic;
use crate::net::{github_digest, hash_in_list};
use crate::settings::Settings;
use crate::verify::Expect;

/// Git for Windows Portable: a self-extracting 7z, read directly (no 7-Zip
/// window), then its post-install script.
pub struct Git;

impl Installer for Git {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
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
    /// (main branch; any owner, for FAT32/exFAT sticks).
    fn finish(&self, ctx: &Ctx, _component: &Component, dest: &Path) -> Result<()> {
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
        let triple = format!("{}-pc-windows-msvc", triple_arch(ctx, component));
        let (file, version, sha) = pick_python(&sums, minor, &triple)
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
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        Python::release(ctx, component, &python_minor(ctx))
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let (_, sums) = python_sums(ctx)?;
        let triple = format!("{}-pc-windows-msvc", triple_arch(ctx, component));
        Ok(python_lines(&sums, &triple))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        Python::release(ctx, component, line)
    }

    /// Keeps the user's packages: the list is saved in
    /// `data\python-packages-<old version>.txt`; with the same minor version,
    /// `Lib\site-packages` and `Scripts` (except pip) move to the new tree.
    fn before_swap(&self, ctx: &Ctx, component: &Component, tree: &Path, old: &Path) -> Result<()> {
        let old_python = old.join("python.exe");
        if !old_python.is_file() {
            return Ok(());
        }
        let old_version = component
            .installed_version(ctx.kit)
            .unwrap_or_else(|| "old".into());
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
        // Packages are compatible only within the same minor version.
        let minor = |v: &str| v.split('.').take(2).collect::<Vec<_>>().join(".");
        if minor(&old_version) != python_minor(ctx) {
            return Ok(());
        }
        let is_pip = |name: &str| {
            let n = name.to_ascii_lowercase();
            n == "pip" || n.starts_with("pip-") || n.starts_with("pip.") || n.starts_with("pip3")
        };
        for sub in [r"Lib\site-packages", "Scripts"] {
            let from = old.join(sub);
            let to = tree.join(sub);
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
        let python = dest.join("python.exe");
        fs::copy(&python, dest.join("python3.exe")).context("python3.exe")?;
        let bundled = dest.join(r"Lib\ensurepip\_bundled");
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

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    if from.is_dir() {
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
    #[serde(default)]
    lts: Value,
}

/// Node.js: the newest LTS with a Windows zip for this platform.
pub struct Node;

/// The lines of Node.js with a Windows zip for `arch` (`win-x64-zip`) in
/// `index.json` (newest first, as the index): the even majors (LTS lines)
/// from 18 on, the others only for their newest major.
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
        let arch = arch_of(ctx, component);
        let index: Vec<NodeEntry> = ctx.net.json("https://nodejs.org/dist/index.json")?;
        let wanted = format!("win-{arch}-zip");
        let entry = index
            .iter()
            .find(|e| {
                e.files.contains(&wanted)
                    && match line {
                        Some(l) => crate::versions::line_of(&e.version).as_deref() == Some(l),
                        None => !matches!(e.lts, Value::Bool(false) | Value::Null),
                    }
            })
            .ok_or_else(|| anyhow!("no Node.js {} for win-{arch}", line.unwrap_or("LTS")))?;
        let v = &entry.version;
        let file_name = format!("node-{v}-win-{arch}.zip");
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
            Kind::Zip,
            Inner::Named(format!("node-{v}-win-{arch}")),
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
            &format!("win-{}-zip", arch_of(ctx, component)),
        ))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        Node::release(ctx, component, Some(line))
    }
}

/// A GitHub asset whose executables go to `tools\` (uv, gh).
fn github_tools(ctx: &Ctx, repo: &str, re: &str, files: Vec<&'static str>) -> Result<Release> {
    let mut release = github_single(ctx, repo, re, Kind::Zip, Inner::Root)?;
    for part in &mut release.parts {
        part.dest = Dest::Tools(files.clone());
    }
    Ok(release)
}

/// uv, uvx, uvw in `tools\`.
pub struct Uv;

impl Installer for Uv {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let triple = format!("{}-pc-windows-msvc", triple_arch(ctx, component));
        github_tools(
            ctx,
            "astral-sh/uv",
            &format!(r"^uv-{}\.zip$", regex::escape(&triple)),
            vec!["uv.exe", "uvx.exe", "uvw.exe"],
        )
    }
}

/// GitHub CLI in `tools\`.
pub struct Gh;

impl Installer for Gh {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = if arch_of(ctx, component) == "arm64" {
            "arm64"
        } else {
            "amd64"
        };
        github_tools(
            ctx,
            "cli/cli",
            &format!(r"^gh_[\d.]+_windows_{arch}\.zip$"),
            vec!["gh.exe"],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_build_is_the_newest_patch() {
        let h = "a".repeat(64);
        let sums = format!(
            "{h}  cpython-3.13.2+20260101-x86_64-pc-windows-msvc-install_only_stripped.tar.gz\n\
             {h}  cpython-3.13.10+20260901-x86_64-pc-windows-msvc-install_only_stripped.tar.gz\n\
             {h}  cpython-3.13.10+20260901-x86_64-pc-windows-msvc-install_only.tar.gz\n\
             {h}  cpython-3.14.0+20260901-x86_64-pc-windows-msvc-install_only_stripped.tar.gz\n\
             {h}  cpython-3.13.10+20260901-aarch64-pc-windows-msvc-install_only_stripped.tar.gz\n"
        );
        let (file, version, _) = pick_python(&sums, "3.13", "x86_64-pc-windows-msvc").unwrap();
        assert_eq!(version, "3.13.10");
        assert!(file.contains("x86_64") && file.ends_with("install_only_stripped.tar.gz"));
        assert_eq!(pick_python(&sums, "3.12", "x86_64-pc-windows-msvc"), None);
        assert_eq!(
            python_lines(&sums, "x86_64-pc-windows-msvc"),
            ["3.14", "3.13"]
        );
        assert_eq!(python_lines(&sums, "aarch64-pc-windows-msvc"), ["3.13"]);
    }

    #[test]
    fn node_lines_are_lts_majors_and_the_newest() {
        let index: Vec<NodeEntry> = serde_json::from_value(serde_json::json!([
            {"version": "v25.1.0", "files": ["win-x64-zip"], "lts": false},
            {"version": "v25.0.0", "files": ["win-x64-zip"], "lts": false},
            {"version": "v24.11.0", "files": ["win-x64-zip"], "lts": "Krypton"},
            {"version": "v24.10.0", "files": ["win-x64-zip"], "lts": false},
            {"version": "v23.11.1", "files": ["win-x64-zip"], "lts": false},
            {"version": "v22.21.1", "files": ["win-x64-zip"], "lts": "Jod"},
            {"version": "v22.21.0", "files": ["win-x64-zip"], "lts": "Jod"},
            {"version": "v20.19.5", "files": ["win-arm64-zip"], "lts": "Iron"},
            {"version": "v16.20.2", "files": ["win-x64-zip"], "lts": "Gallium"}
        ]))
        .unwrap();
        assert_eq!(
            node_lines(&index, "win-x64-zip"),
            ["25.1", "25.0", "24.11", "24.10", "22.21"]
        );
        assert_eq!(node_lines(&index, "win-arm64-zip"), ["20.19"]);
    }
}
