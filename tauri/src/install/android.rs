//! The Android SDK (D-84): the command-line tools of developer.android.com
//! in `android\cmdline-tools\latest`, so sdkmanager finds the SDK by itself
//! (`android\`), then platform-tools, the newest platform and build-tools
//! through sdkmanager, Google's own installer. Its licenses are accepted
//! when the component is chosen (its description says so); they are files
//! in `android\licenses`, nothing on the PC. No emulator.
//!
//! The SHA-256 of the zip is the one on the download page: Google's
//! repository XML has only SHA-1.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{anyhow, bail, Context, Result};

use super::{fetch, target_of, work_dir, Ctx, Installer, Progress, Release};
use crate::fsx::join_rel;
use crate::kit::{Kit, Os, Target};
use crate::net::archive::{self, Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::{translate_rel, Catalog, Component};

pub struct Android;

const STUDIO: &str = "https://developer.android.com/studio";
const REPOSITORY: &str = "https://dl.google.com/android/repository/";

fn os_tag_of(target: Target) -> &'static str {
    match target.os {
        Os::Win32 => "win",
        Os::Darwin => "mac",
        Os::Linux => "linux",
    }
}

/// The Windows zip of the command-line tools on the download page: its
/// name, build number and SHA-256.
#[cfg(test)]
pub fn tools_on_page(page: &str) -> Option<(String, String, String)> {
    tools_on_page_for(page, "win")
}

/// The zip of the command-line tools on the download page for `os_tag`
/// (`"win"`, `"mac"`, or `"linux"`): its name, build number and SHA-256.
pub fn tools_on_page_for(page: &str, os_tag: &str) -> Option<(String, String, String)> {
    let re = regex::Regex::new(&format!(
        r#">\s*(commandlinetools-{os_tag}-(\d+)_latest\.zip)\s*</button>\s*</td>\s*<td>[^<]*</td>\s*<td>\s*([0-9a-f]{{64}})\s*</td>"#
    ))
    .ok()?;
    let c = re.captures(page)?;
    Some((c[1].to_owned(), c[2].to_owned(), c[3].to_owned()))
}

/// The newest stable platform and build-tools in `sdkmanager --list`.
pub fn newest_packages(list: &str) -> (Option<String>, Option<String>) {
    let platform = regex::Regex::new(r"^platforms;android-(\d+)$").ok();
    let build = regex::Regex::new(r"^build-tools;(\d+)\.(\d+)\.(\d+)$").ok();
    let mut best_platform: Option<(u32, String)> = None;
    let mut best_build: Option<((u32, u32, u32), String)> = None;
    for line in list.lines() {
        let name = line.split('|').next().unwrap_or("").trim();
        if let Some(c) = platform.as_ref().and_then(|re| re.captures(name)) {
            let n: u32 = c[1].parse().unwrap_or(0);
            if best_platform.as_ref().is_none_or(|(b, _)| n > *b) {
                best_platform = Some((n, name.to_owned()));
            }
        }
        if let Some(c) = build.as_ref().and_then(|re| re.captures(name)) {
            let v = (
                c[1].parse().unwrap_or(0),
                c[2].parse().unwrap_or(0),
                c[3].parse().unwrap_or(0),
            );
            if best_build.as_ref().is_none_or(|(b, _)| v > *b) {
                best_build = Some((v, name.to_owned()));
            }
        }
    }
    (best_platform.map(|(_, n)| n), best_build.map(|(_, n)| n))
}

/// The kit's newest JDK, if it is 17 or newer (sdkmanager's minimum).
fn java_home(kit: &Kit) -> Option<PathBuf> {
    let java = Catalog::get().find("java")?;
    let line = crate::versions::newest_line(kit, java)?;
    let major: u32 = line.split('.').next()?.parse().ok()?;
    if major < 17 {
        return None;
    }
    let dir = crate::versions::dir_for(kit, java, Some(&line))?;
    let java_bin = translate_rel(kit.target(), r"bin\java.exe");
    join_rel(&dir, &java_bin).is_file().then_some(dir)
}

/// Runs sdkmanager with fixed arguments (package names checked by
/// `newest_packages`), `input` on its standard input; returns its output.
fn sdkmanager(
    ctx: &Ctx,
    sdk: &Path,
    java: &Path,
    args: &[&str],
    input: &str,
    progress: &mut dyn Progress,
) -> Result<String> {
    let rel_script = translate_rel(ctx.kit.target(), r"cmdline-tools\latest\bin\sdkmanager.bat");
    let script = join_rel(sdk, &rel_script);
    let quoted: Vec<String> = args.iter().map(|a| format!("\"{a}\"")).collect();
    let quoted: Vec<&str> = quoted.iter().map(String::as_str).collect();
    let mut cmd = if ctx.kit.os().is_windows() {
        crate::os::jobs::batch_command(&script, &quoted, ctx.env)
    } else {
        let mut c = crate::os::jobs::kit_command(&script, ctx.env);
        c.args(args);
        c
    };
    cmd.env("JAVA_HOME", java)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().context("sdkmanager")?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input.as_bytes());
    }
    let out = child.wait_with_output().context("sdkmanager")?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let err = String::from_utf8_lossy(&out.stderr);
    // The progress bars are lines ending in \r: only what is not one.
    for line in text.lines().chain(err.lines()) {
        let line = line.rsplit('\r').next().unwrap_or(line).trim();
        if !line.is_empty() && !line.starts_with('[') {
            progress.note(line);
        }
    }
    if !out.status.success() {
        let last = err
            .lines()
            .chain(text.lines())
            .rev()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or("")
            .to_owned();
        bail!(
            "sdkmanager {} failed ({}): {last}",
            args.join(" "),
            out.status
        );
    }
    Ok(text)
}

impl Installer for Android {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let os_tag = os_tag_of(target);
        let page = ctx.net.text(STUDIO)?;
        let (name, build, sha) = tools_on_page_for(&page, os_tag).ok_or_else(|| {
            anyhow!("Android command-line tools ({os_tag}): not found on {STUDIO}")
        })?;
        Ok(Release::single(
            build,
            format!("{REPOSITORY}{name}"),
            name,
            Expect::Sha256(sha),
            Kind::Zip,
            Inner::Single,
        ))
    }

    fn custom(&self) -> bool {
        true
    }

    /// The new tools replace `cmdline-tools\latest` only: the packages
    /// already there stay (no new download), sdkmanager adds what is
    /// missing and updates the rest. A first installation that fails
    /// leaves nothing behind.
    fn install_custom(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        let target = target_of(ctx, component);
        let sdk = component
            .folder_path(ctx.kit)
            .ok_or_else(|| anyhow!("android: no folder"))?;
        let java = java_home(ctx.kit)
            .ok_or_else(|| anyhow!("the Android SDK needs Java 17 or newer: install Java first"))?;
        let guard = work_dir(ctx, component)?;
        let work = guard.path();
        let first = !sdk.exists();
        let result = (|| -> Result<()> {
            let part = &release.parts[0];
            let file = fetch(
                ctx,
                &part.url,
                &part.file_name,
                &part.expect,
                work,
                progress,
            )?;
            let extracted = work.join("x");
            archive::extract(&file, Kind::Zip, &extracted)?;
            let tree = archive::pick_inner(&extracted, &Inner::Single)?;
            let sdk_bin = translate_rel(target, r"bin\sdkmanager.bat");
            if !join_rel(&tree, &sdk_bin).is_file() {
                bail!("the archive does not contain {sdk_bin}");
            }
            fs::create_dir_all(sdk.join("cmdline-tools"))?;
            archive::stage_into(&tree, &join_rel(&sdk, "cmdline-tools/latest"))?;

            progress.note("sdkmanager: licenses");
            sdkmanager(
                ctx,
                &sdk,
                &java,
                &["--licenses"],
                &"y\n".repeat(64),
                progress,
            )?;
            let list = sdkmanager(ctx, &sdk, &java, &["--list"], "", progress)?;
            let (platform, build) = newest_packages(&list);
            let mut packages = vec!["platform-tools".to_owned()];
            packages.extend(platform);
            packages.extend(build);
            progress.note(&format!("sdkmanager: {}", packages.join(", ")));
            let mut args = vec!["--install"];
            args.extend(packages.iter().map(String::as_str));
            sdkmanager(ctx, &sdk, &java, &args, &"y\n".repeat(8), progress)?;
            if !first {
                sdkmanager(ctx, &sdk, &java, &["--update"], &"y\n".repeat(8), progress)?;
            }
            let adb_rel = translate_rel(target, r"platform-tools\adb.exe");
            if !join_rel(&sdk, &adb_rel).is_file() {
                bail!("{adb_rel} not found after the installation");
            }
            Ok(())
        })();
        drop(guard);
        if result.is_err() && first {
            let _ = archive::remove_dir(&sdk);
        }
        result
    }
}

#[cfg(test)]
#[path = "../../../test/rust/install/android.rs"]
mod tests;
