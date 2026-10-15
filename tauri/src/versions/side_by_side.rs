//! The versions kept side by side, made known to the tools that look for
//! them (D-81), at start and after every change of what is installed:
//!
//! - **Java**: Gradle's toolchains find the kit's JDKs
//!   (`org.gradle.java.installations.paths` in `data\gradle\gradle.properties`,
//!   no detection of the PC's), Maven's in `~\.m2\toolchains.xml`; a project
//!   on Java 11 compiles with the kit's 11 while Gradle 9 itself runs on the
//!   newest JDK (`data\kit\bin\gradle.cmd`, `ATOLLO_GRADLE_JAVA`).
//! - **.NET**: every folder of a version gets the SDKs and runtimes of the
//!   others, as junctions in `sdk\` and `shared\<framework>\` (the layout
//!   dotnet itself uses side by side): a `net8.0` app runs under the 10 SDK,
//!   a `global.json` on 8.0 finds its SDK.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::fsx::write_atomic;
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
const KIT_MARK: &str = "Atollo";

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
            key != GRADLE_PATHS && key != GRADLE_DETECT && !l.contains("# Atollo: the kit's JDKs")
        })
        .map(str::to_owned)
        .collect();
    if !jdks.is_empty() {
        out.push("# Atollo: the kit's JDKs for toolchains, none of the PC's".into());
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
    let props = kit.data().join(r"gradle\gradle.properties");
    let old = fs::read_to_string(&props).unwrap_or_default();
    let dirs: Vec<PathBuf> = jdks.iter().map(|(_, d)| d.clone()).collect();
    let new = gradle_properties(&old, &dirs);
    if new.trim() != old.trim() {
        fs::create_dir_all(props.parent().unwrap_or(&props))?;
        write_atomic(&props, new.as_bytes()).with_context(|| props.display().to_string())?;
    }
    let toolchains = kit.home().join(r".m2\toolchains.xml");
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
    r"shared\Microsoft.NETCore.App",
    r"shared\Microsoft.AspNetCore.App",
    r"shared\Microsoft.WindowsDesktop.App",
];

fn is_junction(path: &Path) -> bool {
    junction::exists(path).unwrap_or(false)
}

/// Each root gets, as junctions, the versions the other roots have of
/// their own; junctions to versions gone are removed.
pub fn link_dotnet(roots: &[PathBuf]) -> Result<()> {
    for root in roots {
        for sub in DOTNET_SHARED {
            let dir = root.join(sub);
            for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if is_junction(&path) && !junction::get_target(&path).is_ok_and(|t| t.is_dir()) {
                    let _ = junction::delete(&path);
                    let _ = fs::remove_dir(&path);
                }
            }
        }
    }
    for root in roots {
        for other in roots.iter().filter(|o| *o != root) {
            for sub in DOTNET_SHARED {
                let from = other.join(sub);
                for entry in fs::read_dir(&from).into_iter().flatten().flatten() {
                    let source = entry.path();
                    if is_junction(&source) || !source.is_dir() {
                        continue;
                    }
                    let link = root.join(sub).join(entry.file_name());
                    if link.exists() {
                        continue;
                    }
                    fs::create_dir_all(root.join(sub))?;
                    junction::create(&source, &link).with_context(|| link.display().to_string())?;
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
mod tests {
    use super::*;
    use crate::kit::tests::TempDir;

    #[test]
    fn gradle_finds_the_kit_jdks_only() {
        let jdks = [
            PathBuf::from(r"D:\kit\java\25.0"),
            PathBuf::from(r"D:\kit\java\11.0"),
        ];
        let text = "org.gradle.jvmargs=-Xmx2g\norg.gradle.java.installations.paths=C:/old\n";
        let out = gradle_properties(text, &jdks);
        assert!(
            out.contains("org.gradle.jvmargs=-Xmx2g"),
            "the user's lines stay"
        );
        assert!(
            out.contains("org.gradle.java.installations.paths=D:/kit/java/25.0,D:/kit/java/11.0")
        );
        assert!(out.contains("auto-detect=false"));
        assert!(!out.contains("C:/old"));
        assert_eq!(gradle_properties(&out, &jdks), out, "stable");
        assert!(!gradle_properties(&out, &[]).contains("installations"));
        let xml = maven_toolchains(&[("11.0".into(), PathBuf::from(r"D:\kit\java\11.0"))]);
        assert!(xml.contains("<version>11</version>") && xml.contains(KIT_MARK));
    }

    #[test]
    fn dotnet_roots_share_their_versions() {
        let tmp = TempDir::new("dotnet-sbs");
        let ten = tmp.0.join("10.0");
        let eight = tmp.0.join("8.0");
        for (root, v) in [(&ten, "10.0.3"), (&eight, "8.0.21")] {
            let rt = root.join(r"shared\Microsoft.NETCore.App").join(v);
            fs::create_dir_all(&rt).unwrap();
            fs::write(rt.join("coreclr.dll"), b"x").unwrap();
        }
        fs::create_dir_all(eight.join(r"sdk\8.0.415")).unwrap();
        link_dotnet(&[ten.clone(), eight.clone()]).unwrap();
        assert!(ten
            .join(r"shared\Microsoft.NETCore.App\8.0.21\coreclr.dll")
            .is_file());
        assert!(ten.join(r"sdk\8.0.415").is_dir());
        assert!(eight
            .join(r"shared\Microsoft.NETCore.App\10.0.3\coreclr.dll")
            .is_file());
        // Again: nothing new, no error.
        link_dotnet(&[ten.clone(), eight.clone()]).unwrap();

        // Removing a version's folder leaves the others' files alone
        // (junctions are not followed), then its junctions go.
        crate::archive::remove_dir(&ten).unwrap();
        assert!(eight
            .join(r"shared\Microsoft.NETCore.App\8.0.21\coreclr.dll")
            .is_file());
        link_dotnet(std::slice::from_ref(&eight)).unwrap();
        assert!(!eight.join(r"shared\Microsoft.NETCore.App\10.0.3").exists());
        assert!(eight.join(r"sdk\8.0.415").is_dir());
    }
}
