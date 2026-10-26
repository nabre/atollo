use std::fs;
use std::path::Path;

use super::list::{explicit_text, Archive, Package, Record};
use super::*;
use crate::install::installer;
use crate::kit::tests::TempDir;
use crate::kit::Target;
use crate::setup::Catalog;

/// The SHA-256 of `bytes`, lowercase hex.
pub(super) fn sha(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn archive(subdir: &str, file: &str, sha256: &str) -> Archive {
    Archive {
        url: format!("{CHANNEL}{subdir}/{file}"),
        file: file.to_owned(),
        sha256: sha256.to_owned(),
    }
}

/// `files` (name, content) in `pkgs` with their list and record.
pub(super) fn kept(pkgs: &Path, subdir: &str, files: &[(&str, &[u8])]) -> (String, String) {
    fs::create_dir_all(pkgs).unwrap();
    let mut archives = Vec::new();
    let mut packages = Vec::new();
    for (name, content) in files {
        fs::write(pkgs.join(name), content).unwrap();
        archives.push(archive(subdir, name, &sha(content)));
        packages.push(Package {
            file: (*name).to_owned(),
            sha256: sha(content),
            size: content.len() as u64,
        });
    }
    let record = Record {
        version: "3.54.1".into(),
        target: "linux-x64".into(),
        created: 1,
        micromamba: Some("2.9.0".into()),
        packages,
    };
    (
        explicit_text(&archives, subdir),
        serde_json::to_string(&record).unwrap(),
    )
}

#[test]
fn kept_packages_have_one_folder_per_target() {
    let tmp = TempDir::new("conda-root");
    let linux = Kit::at_target(tmp.0.clone(), Target::LINUX_X64);
    let mac = Kit::at_target(tmp.0.clone(), Target::DARWIN_ARM64);
    assert_eq!(
        root(&linux),
        tmp.0
            .join("data")
            .join("app")
            .join("conda")
            .join("linux-x64")
    );
    assert_ne!(root(&linux), root(&mac));
    assert_eq!(lock_name("ruby", "3.4"), "ruby@3.4");
    assert_eq!(
        lock_name("cc", &crate::versions::dir_name("zig cc, make 4.4.1")),
        "cc@4.4"
    );
    assert_eq!(
        lock_name(
            "mongodb",
            &crate::versions::dir_name("8.0.23+mongosh2.5.9+tools100.19.1")
        ),
        "mongodb@8.0"
    );
}

#[test]
fn conda_environments_by_component_and_target() {
    let dir = Path::new("/k/data/apps/linux-x64/cc");
    let cc = installer("cc").unwrap();
    assert_eq!(
        cc.conda_prefix(Target::LINUX_X64, dir),
        Some(dir.join("make"))
    );
    assert_eq!(cc.conda_prefix(Target::WIN32_X64, dir), None);
    let ruby = installer("ruby").unwrap();
    assert_eq!(
        ruby.conda_prefix(Target::DARWIN_ARM64, dir),
        Some(dir.to_path_buf())
    );
    assert_eq!(ruby.conda_prefix(Target::WIN32_X64, dir), None);
    let mongo = installer("mongodb").unwrap();
    assert!(mongo.conda_prefix(Target::LINUX_ARM64, dir).is_some());
    assert!(mongo.conda_prefix(Target::DARWIN_ARM64, dir).is_none());
    let sqlite = installer("sqlite").unwrap();
    assert!(sqlite.conda_prefix(Target::LINUX_ARM64, dir).is_some());
    assert!(sqlite.conda_prefix(Target::LINUX_X64, dir).is_none());
    // Every component that cannot follow the kit says where its environment is.
    for t in [Target::LINUX_X64, Target::LINUX_ARM64, Target::DARWIN_ARM64] {
        for c in Catalog::get().all() {
            let inst = installer(&c.id).unwrap();
            if !inst.relocatable(t) {
                assert!(inst.conda_prefix(t, dir).is_some(), "{} on {t}", c.id);
            }
        }
    }
}
