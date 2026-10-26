use super::super::tests::kept;
use super::*;
use crate::kit::tests::TempDir;
use crate::kit::Target;

/// An R line on disk as conda leaves it (`conda-meta`).
fn r_line(kit: &Kit, line: &str) {
    let dir = kit.apps().join("r").join(line);
    fs::create_dir_all(dir.join("conda-meta")).unwrap();
    fs::create_dir_all(dir.join("bin")).unwrap();
    fs::write(dir.join("bin").join("R"), b"#!/bin/sh\n").unwrap();
}

fn write_lock(kit: &Kit, name: &str, list: &str, record: Option<&str>) {
    let locks = locks_dir(kit);
    fs::create_dir_all(&locks).unwrap();
    fs::write(locks.join(format!("{name}.txt")), list).unwrap();
    if let Some(record) = record {
        fs::write(locks.join(format!("{name}.json")), record).unwrap();
    }
}

#[test]
fn sweep_removes_what_no_list_of_an_environment_names() {
    let tmp = TempDir::new("conda-sweep");
    let kit = Kit::at_target(tmp.0.clone(), Target::LINUX_X64);
    let pkgs = pkgs_dir(&kit);
    let (list45, rec45) = kept(
        &pkgs,
        "linux-64",
        &[("shared-1-0.conda", b"s"), ("r45-1-0.conda", b"r")],
    );
    let (list44, rec44) = kept(
        &pkgs,
        "linux-64",
        &[("shared-1-0.conda", b"s"), ("r44-1-0.conda", b"q")],
    );
    let (ruby, _) = kept(&pkgs, "linux-64", &[("ruby-1-0.conda", b"y")]);
    fs::write(pkgs.join("stray-1-0.conda"), b"x").unwrap();
    fs::write(pkgs.join("r45-1-0.part"), b"half").unwrap();
    fs::create_dir_all(pkgs.join("extracted-1-0").join("info")).unwrap();
    r_line(&kit, "4.5");
    write_lock(&kit, "r@4.5", &list45, Some(&rec45));
    // 4.4 removed from the Tools page: its environment is gone.
    write_lock(&kit, "r@4.4", &list44, Some(&rec44));
    // A list without its record: not usable, removed.
    write_lock(&kit, "ruby@3.4", &ruby, None);
    // Another system's folder is never looked at.
    let mac = Kit::at_target(tmp.0.clone(), Target::DARWIN_ARM64);
    fs::create_dir_all(pkgs_dir(&mac)).unwrap();
    fs::write(pkgs_dir(&mac).join("osx-1-0.conda"), b"m").unwrap();

    let swept = sweep(&kit);
    assert_eq!(swept.lists, vec!["r@4.4".to_owned(), "ruby@3.4".to_owned()]);
    let mut left: Vec<String> = fs::read_dir(&pkgs)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    assert_eq!(left, ["r45-1-0.conda", "shared-1-0.conda"]);
    assert_eq!(swept.archives, 5);
    assert!(locks_dir(&kit).join("r@4.5.txt").is_file());
    assert!(locks_dir(&kit).join("r@4.5.json").is_file());
    assert!(!locks_dir(&kit).join("r@4.4.json").exists());
    assert!(pkgs_dir(&mac).join("osx-1-0.conda").is_file());
    assert_eq!(status(&kit).lists, vec!["r@4.5".to_owned()]);
    assert!(status(&kit).applies);
    assert_eq!(status(&kit).bytes, tree_size(&root(&kit)));

    // Nothing more to remove the second time.
    assert_eq!(sweep(&kit), Swept::default());
}

#[test]
fn a_list_of_a_component_that_follows_the_kit_goes() {
    // On Windows R follows the kit (D-92): a list there is never used.
    let tmp = TempDir::new("conda-windows");
    let kit = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    let (list, rec) = kept(&pkgs_dir(&kit), "win-64", &[("r-1-0.conda", b"r")]);
    r_line(&kit, "4.5");
    write_lock(&kit, "r@4.5", &list, Some(&rec));
    let swept = sweep(&kit);
    assert_eq!(swept.lists, vec!["r@4.5".to_owned()]);
    assert_eq!(swept.archives, 1);
    assert!(!status(&kit).applies);
}

/// A component (or one line of it) removed: its lists go at once, also
/// while its environment is still on disk, and those of another component
/// stay; the archives are left to the sweep.
#[test]
fn forget_removes_the_lists_of_a_component() {
    let tmp = TempDir::new("conda-forget");
    let kit = Kit::at_target(tmp.0.clone(), Target::LINUX_X64);
    let pkgs = pkgs_dir(&kit);
    let (list45, rec45) = kept(
        &pkgs,
        "linux-64",
        &[("shared-1-0.conda", b"s"), ("r45-1-0.conda", b"r")],
    );
    let (list44, rec44) = kept(&pkgs, "linux-64", &[("r44-1-0.conda", b"q")]);
    let (rr, recrr) = kept(&pkgs, "linux-64", &[("shared-1-0.conda", b"s")]);
    r_line(&kit, "4.5");
    write_lock(&kit, "r@4.5", &list45, Some(&rec45));
    write_lock(&kit, "r@4.4", &list44, Some(&rec44));
    // A component whose id starts the same: not R's.
    write_lock(&kit, "rr@1.0", &rr, Some(&recrr));

    forget(&kit, "r", Some("4.4"));
    assert!(!locks_dir(&kit).join("r@4.4.txt").exists());
    assert!(!locks_dir(&kit).join("r@4.4.json").exists());
    assert!(locks_dir(&kit).join("r@4.5.txt").is_file());

    forget(&kit, "r", None);
    assert!(!locks_dir(&kit).join("r@4.5.txt").exists());
    assert!(!locks_dir(&kit).join("r@4.5.json").exists());
    assert!(locks_dir(&kit).join("rr@1.0.txt").is_file());
    assert!(locks_dir(&kit).join("rr@1.0.json").is_file());
}

#[test]
fn kept_packages_off_frees_everything() {
    let tmp = TempDir::new("conda-off");
    let kit = Kit::at_target(tmp.0.clone(), Target::LINUX_ARM64);
    let (list, rec) = kept(
        &pkgs_dir(&kit),
        "linux-aarch64",
        &[("r-1-0.conda", b"rrrr")],
    );
    r_line(&kit, "4.5");
    write_lock(&kit, "r@4.5", &list, Some(&rec));
    let settings = Settings {
        keep_conda_packages: false,
        ..Settings::default()
    };
    settings.save(&kit).unwrap();
    let swept = sweep(&kit);
    assert!(swept.bytes > 0);
    assert!(!root(&kit).exists());
    assert_eq!(status(&kit).bytes, 0);
}

#[test]
fn free_removes_the_target_folder_only() {
    let tmp = TempDir::new("conda-free");
    let kit = Kit::at_target(tmp.0.clone(), Target::LINUX_X64);
    let mac = Kit::at_target(tmp.0.clone(), Target::DARWIN_ARM64);
    kept(&pkgs_dir(&kit), "linux-64", &[("a-1-0.conda", b"12345")]);
    kept(&pkgs_dir(&mac), "osx-arm64", &[("a-1-0.conda", b"12345")]);
    assert_eq!(free(&kit).unwrap(), 5);
    assert!(!root(&kit).exists());
    assert!(root(&mac).exists());
    assert_eq!(free(&kit).unwrap(), 0);
}
