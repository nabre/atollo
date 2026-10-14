//! Git and Node.js (port of nabre/dev-kit `_prepare-core.ps1:109-158,
//! 239-268` and `_prepare-finish.ps1:10-38`).

use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde::Deserialize;

use super::{arch_of, Ctx, Installer, Release};
use crate::archive::{Inner, Kind};
use crate::catalog::Component;
use crate::jobs;
use crate::kit::write_atomic;
use crate::net::{github_digest, hash_in_list};
use crate::verify::Expect;

/// Git for Windows Portable: a self-extracting 7z, read directly (no 7-Zip
/// window), then its post-install script. Without a GitHub checksum it is not installed: the signature
/// check that the kit used as fallback arrives with phase 4.
pub struct Git;

impl Installer for Git {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let rel = ctx.net.github_latest("git-for-windows/git")?;
        let suffix = if arch_of(ctx, component) == "arm64" {
            "arm64"
        } else {
            "64-bit"
        };
        let re =
            Regex::new(&format!(r"^PortableGit-(.+)-{suffix}\.7z\.exe$")).context("pattern")?;
        let asset = rel
            .assets
            .iter()
            .find(|a| re.is_match(&a.name))
            .ok_or_else(|| anyhow!("no PortableGit for {suffix} in {}", rel.tag_name))?;
        let version = re
            .captures(&asset.name)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_owned())
            .unwrap_or_else(|| rel.tag_name.clone());
        Ok(Release {
            version,
            url: asset.browser_download_url.clone(),
            file_name: asset.name.clone(),
            expect: Expect::Sha256(github_digest(asset)?),
            kind: Kind::SevenZip,
            inner: Inner::Root,
        })
    }

    /// `Set-KitGitConfig`: the system credential helper of the kit's Git is
    /// `manager` (not the selector window), and the portable home gets a
    /// `.gitconfig` once (main branch; any owner, for FAT32/exFAT sticks).
    fn finish(&self, ctx: &Ctx, _component: &Component, dest: &Path) -> Result<()> {
        // What the self-extractor would run after extracting: devices, mtab,
        // DLL copies, all inside the Git folder; the script deletes itself.
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
        let re = Regex::new(r"(?i)helper-selector|git-credential-manager(\.exe)?$")
            .context("pattern")?;
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

#[derive(Deserialize)]
struct NodeEntry {
    version: String,
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    lts: serde_json::Value,
}

/// Node.js: the newest LTS with a Windows zip for this platform.
pub struct Node;

impl Installer for Node {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = arch_of(ctx, component);
        let index: Vec<NodeEntry> = ctx.net.json("https://nodejs.org/dist/index.json")?;
        let wanted = format!("win-{arch}-zip");
        let entry = index
            .iter()
            .find(|e| {
                !matches!(
                    e.lts,
                    serde_json::Value::Bool(false) | serde_json::Value::Null
                ) && e.files.contains(&wanted)
            })
            .ok_or_else(|| anyhow!("no Node.js LTS for win-{arch}"))?;
        let v = &entry.version;
        let file_name = format!("node-{v}-win-{arch}.zip");
        let sums = ctx
            .net
            .text(&format!("https://nodejs.org/dist/{v}/SHASUMS256.txt"))?;
        let sha = hash_in_list(&sums, &file_name)
            .ok_or_else(|| anyhow!("{file_name} missing in SHASUMS256.txt"))?;
        Ok(Release {
            version: v.clone(),
            url: format!("https://nodejs.org/dist/{v}/{file_name}"),
            inner: Inner::Named(format!("node-{v}-win-{arch}")),
            file_name,
            expect: Expect::Sha256(sha),
            kind: Kind::Zip,
        })
    }
}
