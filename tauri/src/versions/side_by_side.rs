//! The versions kept side by side, made known to the tools that look for
//! them (D-81), at start and after every change of what is installed:
//!
//! - **Java**: Gradle's toolchains find the kit's JDKs
//!   (`org.gradle.java.installations.paths` in `data\gradle\gradle.properties`,
//!   no detection of the PC's), Maven's in `~\.m2\toolchains.xml`; a project
//!   on Java 11 compiles with the kit's 11 while Gradle 9 itself runs on the
//!   newest JDK (`data\kit\bin\gradle.cmd`, `DEVKIT_GRADLE_JAVA`).
//! - **.NET**: every folder of a version gets the SDKs and runtimes of the
//!   others, as junctions in `sdk\` and `shared\<framework>\` (the layout
//!   dotnet itself uses side by side): a `net8.0` app runs under the 10 SDK,
//!   a `global.json` on 8.0 finds its SDK.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::fsx::{join_rel, write_atomic};
use crate::kit::Kit;
use crate::setup::catalog::{Catalog, Component};

/// Every folder of a versioned component on disk (the default first).
fn line_dirs(kit: &Kit, c: &Component) -> Vec<(String, PathBuf)> {
    crate::versions::default_line(kit, c)
        .into_iter()
        .chain(crate::versions::saved(kit, c).into_iter().map(|l| l.line))
        .filter_map(|l| Some((l.clone(), crate::versions::dir_for(kit, c, Some(&l))?)))
        .collect()
}

pub fn refresh(kit: &Kit) {
    if let Err(e) = java(kit) {
        crate::log::warn(&format!("JDKs for Gradle and Maven: {e:#}"));
    }
    if let Err(e) = dotnet(kit) {
        crate::log::warn(&format!(".NET side by side: {e:#}"));
    }
    // `make` for a C/C++ installed before the kit wrote it.
    let cc_bin = crate::setup::catalog::kit_path(kit, "cc/bin");
    if let Err(e) = crate::install::make_launchers(&cc_bin) {
        crate::log::warn(&format!("make for C/C++: {e:#}"));
    }
}

const GRADLE_PATHS: &str = "org.gradle.java.installations.paths";
const GRADLE_DETECT: &str = "org.gradle.java.installations.auto-detect";
const KIT_MARK: &str = "Dev-Kit";

fn fwd(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

/// `gradle.properties` with the kit's JDKs as the only local ones (the
/// user's other lines stay).
pub fn gradle_properties(text: &str, jdks: &[PathBuf]) -> String {
    let mut out: Vec<String> = text
        .lines()
        .filter(|l| {
            let key = l.split('=').next().unwrap_or("").trim();
            key != GRADLE_PATHS && key != GRADLE_DETECT && !l.contains("# Dev-Kit: the kit's JDKs")
        })
        .map(str::to_owned)
        .collect();
    if !jdks.is_empty() {
        out.push("# Dev-Kit: the kit's JDKs for toolchains, none of the PC's".into());
        let list: Vec<String> = jdks.iter().map(|p| fwd(p)).collect();
        out.push(format!("{GRADLE_PATHS}={}", list.join(",")));
        out.push(format!("{GRADLE_DETECT}=false"));
    }
    let mut text = out.join("\n");
    text.push('\n');
    text
}

/// Maven's toolchains: one JDK per line on disk.
pub fn maven_toolchains(jdks: &[(String, PathBuf)]) -> String {
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!-- {KIT_MARK}: written by the kit, the JDKs it has; a file without this line is left alone. -->\n<toolchains>\n"
    );
    for (line, dir) in jdks {
        let version = line.split('.').next().unwrap_or(line);
        xml.push_str(&format!(
            "  <toolchain>\n    <type>jdk</type>\n    <provides><version>{version}</version><vendor>temurin</vendor></provides>\n    <configuration><jdkHome>{}</jdkHome></configuration>\n  </toolchain>\n",
            dir.display()
        ));
    }
    xml.push_str("</toolchains>\n");
    xml
}

fn java(kit: &Kit) -> Result<()> {
    let Some(c) = Catalog::get().find("java") else {
        return Ok(());
    };
    let jdks = line_dirs(kit, c);
    let props = join_rel(kit.data(), "gradle/gradle.properties");
    let old = fs::read_to_string(&props).unwrap_or_default();
    let dirs: Vec<PathBuf> = jdks.iter().map(|(_, d)| d.clone()).collect();
    let new = gradle_properties(&old, &dirs);
    if new.trim() != old.trim() {
        fs::create_dir_all(props.parent().unwrap_or(&props))?;
        write_atomic(&props, new.as_bytes()).with_context(|| props.display().to_string())?;
    }
    let toolchains = join_rel(kit.home(), ".m2/toolchains.xml");
    let mine = fs::read_to_string(&toolchains).map_or(true, |t| t.contains(KIT_MARK));
    if mine {
        if jdks.is_empty() {
            let _ = fs::remove_file(&toolchains);
        } else {
            fs::create_dir_all(toolchains.parent().unwrap_or(&toolchains))?;
            write_atomic(&toolchains, maven_toolchains(&jdks).as_bytes())?;
        }
    }
    Ok(())
}

/// The folders of a .NET root another root shares: its SDKs and runtimes.
const DOTNET_SHARED: [&str; 4] = [
    "sdk",
    "shared/Microsoft.NETCore.App",
    "shared/Microsoft.AspNetCore.App",
    "shared/Microsoft.WindowsDesktop.App",
];

#[cfg(windows)]
fn is_dir_link(path: &Path) -> bool {
    junction::exists(path).unwrap_or(false)
}

#[cfg(not(windows))]
fn is_dir_link(path: &Path) -> bool {
    path.is_symlink()
}

#[cfg(windows)]
fn dir_link_target_ok(path: &Path) -> bool {
    junction::get_target(path).is_ok_and(|t| t.is_dir())
}

#[cfg(not(windows))]
fn dir_link_target_ok(path: &Path) -> bool {
    fs::read_link(path).is_ok_and(|t| t.is_dir())
}

#[cfg(windows)]
fn remove_dir_link(path: &Path) {
    let _ = junction::delete(path);
    let _ = fs::remove_dir(path);
}

#[cfg(not(windows))]
fn remove_dir_link(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(windows)]
fn create_dir_link(source: &Path, link: &Path) -> Result<()> {
    junction::create(source, link).with_context(|| link.display().to_string())
}

#[cfg(not(windows))]
fn create_dir_link(source: &Path, link: &Path) -> Result<()> {
    link_or_copy(source, link, |s, l| std::os::unix::fs::symlink(s, l))
}

/// In a folder copied where a link could not be made: the folder it copies,
/// relative to the copy (`../../../8.0/sdk/8.0.415`), so a kit on a disk
/// mounted elsewhere (`/media/a/KIT`, `/Volumes/KIT`) still finds it.
const COPY_MARK: &str = ".devkit-copy-of";

/// Suffix of the folder a copy is made in before it takes its name: an
/// interrupted copy leaves only that, removed by [`link_dotnet`].
const STAGED: &str = ".devkit-new";

/// The link `link -> source`, made by `make`; on a disk without links
/// (exFAT, FAT: `archive::links_unsupported`) a copy of `source` marked
/// with [`COPY_MARK`], so it goes like a link when `source` goes.
#[cfg_attr(windows, allow(dead_code))]
fn link_or_copy(
    source: &Path,
    link: &Path,
    make: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
) -> Result<()> {
    match make(source, link) {
        Ok(()) => Ok(()),
        Err(e) if crate::net::archive::links_unsupported(&e) => {
            crate::net::archive::check_fits(source, link)?;
            crate::net::archive::check_space(source, link)?;
            copy_marked(source, link, crate::net::archive::copy_tree)
                .with_context(|| format!("copying {} (no links on this disk)", link.display()))
        }
        Err(e) => Err(e).with_context(|| link.display().to_string()),
    }
}

/// `<link>.devkit-new`, where the copy of [`copy_marked`] is made.
fn staged_path(link: &Path) -> PathBuf {
    let mut name = link.file_name().unwrap_or_default().to_os_string();
    name.push(STAGED);
    link.with_file_name(name)
}

/// Copies `source` (with `copy`) into `<link>.devkit-new`, marks it, then
/// renames it to `link`: `link` is a complete, marked copy or nothing, and
/// a copy that fails (a full disk) removes what it made.
#[cfg_attr(windows, allow(dead_code))]
fn copy_marked(
    source: &Path,
    link: &Path,
    copy: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let staged = staged_path(link);
    if staged.exists() {
        fs::remove_dir_all(&staged)?;
    }
    let made = copy(source, &staged)
        .and_then(|()| fs::write(staged.join(COPY_MARK), copy_mark(source, link)))
        .and_then(|()| fs::rename(&staged, link));
    if made.is_err() {
        let _ = fs::remove_dir_all(&staged);
    }
    made
}

/// A folder [`copy_marked`] is still making, or left by a copy interrupted.
fn is_staged(path: &Path) -> bool {
    !is_dir_link(path)
        && path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().ends_with(STAGED))
}

/// The text of [`COPY_MARK`]: `source` relative to `link`, with `/`; the
/// absolute path when they share no root.
fn copy_mark(source: &Path, link: &Path) -> String {
    let from: Vec<_> = link.components().collect();
    let to: Vec<_> = source.components().collect();
    let common = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
    if common == 0 {
        return source.display().to_string();
    }
    let ups = std::iter::repeat_n("..".to_owned(), from.len() - common);
    let downs = to[common..]
        .iter()
        .map(|c| c.as_os_str().to_string_lossy().into_owned());
    ups.chain(downs).collect::<Vec<_>>().join("/")
}

/// A copy made by [`link_or_copy`] whose source is gone. The mark is read
/// against the copy (an absolute one, of an older copy, as it is).
fn stale_copy(path: &Path) -> bool {
    fs::read_to_string(path.join(COPY_MARK)).is_ok_and(|s| !copy_source(path, s.trim()).is_dir())
}

/// The folder a [`COPY_MARK`] text names, read against the copy `path`.
fn copy_source(path: &Path, mark: &str) -> PathBuf {
    if Path::new(mark).is_absolute() {
        PathBuf::from(mark)
    } else {
        join_rel(path, mark)
    }
}

/// Each root gets, as junctions (Windows) or symlinks (Unix), the versions the
/// other roots have of their own; links to versions gone are removed. On a
/// Unix disk without links they are copies, removed the same way.
pub fn link_dotnet(roots: &[PathBuf]) -> Result<()> {
    for root in roots {
        for sub in DOTNET_SHARED {
            let dir = join_rel(root, sub);
            for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if is_staged(&path) {
                    // A copy interrupted (the app closed): made again below.
                    let _ = fs::remove_dir_all(&path);
                } else if is_dir_link(&path) && !dir_link_target_ok(&path) {
                    remove_dir_link(&path);
                } else if !is_dir_link(&path) && stale_copy(&path) {
                    let _ = fs::remove_dir_all(&path);
                }
            }
        }
    }
    for root in roots {
        for other in roots.iter().filter(|o| *o != root) {
            for sub in DOTNET_SHARED {
                let from = join_rel(other, sub);
                for entry in fs::read_dir(&from).into_iter().flatten().flatten() {
                    let source = entry.path();
                    // Only a root's own versions: not its links, nor copies.
                    if is_dir_link(&source)
                        || !source.is_dir()
                        || source.join(COPY_MARK).is_file()
                        || is_staged(&source)
                    {
                        continue;
                    }
                    let sub_dir = join_rel(root, sub);
                    let link = sub_dir.join(entry.file_name());
                    if link.exists() || is_dir_link(&link) {
                        continue;
                    }
                    fs::create_dir_all(&sub_dir)?;
                    create_dir_link(&source, &link)?;
                }
            }
        }
    }
    Ok(())
}

fn dotnet(kit: &Kit) -> Result<()> {
    let Some(c) = Catalog::get().find("dotnet") else {
        return Ok(());
    };
    let roots: Vec<PathBuf> = line_dirs(kit, c).into_iter().map(|(_, d)| d).collect();
    if roots.len() < 2 {
        return Ok(());
    }
    link_dotnet(&roots)
}

#[cfg(test)]
#[path = "../../../test/rust/versions/side_by_side.rs"]
mod tests;
