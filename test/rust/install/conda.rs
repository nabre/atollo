use super::*;
use crate::kit::tests::TempDir;
use crate::kit::Target;

const SHA_A: &str = "f027973cd4f9396ce907f95a7853a33bc9a38127ff473f8bf29eb129a14ce841";

fn sha(bytes: &[u8]) -> String {
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
fn kept(pkgs: &Path, subdir: &str, files: &[(&str, &[u8])]) -> (String, String) {
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
fn explicit_list_of_conda_forge_with_sha256() {
    let text = format!(
        "# platform: linux-64\n@EXPLICIT\n\n{CHANNEL}linux-64/sqlite-3.54.1-h6e79ffa_100.conda#{SHA_A}\n{CHANNEL}noarch/tzdata-2026a-h0c530f3_0.conda#sha256:{}\n",
        SHA_A.to_uppercase()
    );
    let list = parse_explicit(&text).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].file, "sqlite-3.54.1-h6e79ffa_100.conda");
    assert_eq!(list[0].subdir(), Some("linux-64"));
    assert_eq!(list[1].subdir(), Some("noarch"));
    assert_eq!(list[1].sha256, SHA_A, "hashes are kept lowercase");
    // What the kit writes, it reads back the same.
    assert_eq!(
        parse_explicit(&explicit_text(&list, "linux-64")).unwrap(),
        list
    );
}

#[test]
fn explicit_list_refuses_md5_other_channels_and_paths() {
    let ok = format!("{CHANNEL}linux-64/a-1-0.conda#{SHA_A}");
    for bad in [
        // micromamba env export's MD5: not what the archives are checked against.
        format!("@EXPLICIT\n{CHANNEL}linux-64/a-1-0.conda#d904c7ddd95e026254b4b80808ee9e12"),
        format!("@EXPLICIT\n{CHANNEL}linux-64/a-1-0.conda"),
        ok.clone(),
        format!("@EXPLICIT\nhttps://example.org/conda-forge/linux-64/a-1-0.conda#{SHA_A}"),
        format!("@EXPLICIT\nhttp://conda.anaconda.org/conda-forge/linux-64/a-1-0.conda#{SHA_A}"),
        format!("@EXPLICIT\nfile:///tmp/a-1-0.conda#{SHA_A}"),
        format!("@EXPLICIT\n{CHANNEL}linux-64/../../a-1-0.conda#{SHA_A}"),
        format!("@EXPLICIT\n{CHANNEL}linux-64/sub/a-1-0.conda#{SHA_A}"),
        format!("@EXPLICIT\n{CHANNEL}linux-64/.hidden.conda#{SHA_A}"),
        format!("@EXPLICIT\n{CHANNEL}linux-64/a-1-0.zip#{SHA_A}"),
        "@EXPLICIT\n".to_owned(),
        String::new(),
    ] {
        assert!(parse_explicit(&bad).is_err(), "accepted: {bad}");
    }
    assert!(parse_explicit(&format!("@EXPLICIT\n{ok}")).is_ok());
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
fn offline_only_with_every_archive_and_its_hash() {
    let tmp = TempDir::new("conda-check");
    let pkgs = tmp.0.join("pkgs");
    let (list, record) = kept(
        &pkgs,
        "linux-64",
        &[("a-1-0.conda", b"aaa"), ("b-2-0.tar.bz2", b"bbbb")],
    );
    let ok = check(Some(&list), Some(&record), "linux-64", &pkgs).unwrap();
    assert_eq!(ok.len(), 2);

    // No list (installed before D-118, freed): the network.
    assert_eq!(
        check(None, Some(&record), "linux-64", &pkgs),
        Err(Fallback::NoList)
    );
    assert_eq!(
        check(Some(&list), None, "linux-64", &pkgs),
        Err(Fallback::NoList)
    );
    assert!(matches!(
        check(Some("garbage"), Some(&record), "linux-64", &pkgs),
        Err(Fallback::Unreadable(_))
    ));
    assert!(matches!(
        check(Some(&list), Some("{"), "linux-64", &pkgs),
        Err(Fallback::Unreadable(_))
    ));

    // Same size, other bytes: never used.
    fs::write(pkgs.join("b-2-0.tar.bz2"), b"BBBB").unwrap();
    assert_eq!(
        check(Some(&list), Some(&record), "linux-64", &pkgs),
        Err(Fallback::Changed("b-2-0.tar.bz2".into()))
    );
    // A partial copy, then none.
    fs::write(pkgs.join("b-2-0.tar.bz2"), b"bb").unwrap();
    assert_eq!(
        check(Some(&list), Some(&record), "linux-64", &pkgs),
        Err(Fallback::Missing("b-2-0.tar.bz2".into()))
    );
    fs::remove_file(pkgs.join("b-2-0.tar.bz2")).unwrap();
    assert_eq!(
        check(Some(&list), Some(&record), "linux-64", &pkgs),
        Err(Fallback::Missing("b-2-0.tar.bz2".into()))
    );
}

#[test]
fn a_record_changed_alone_lets_nothing_through() {
    let tmp = TempDir::new("conda-record");
    let pkgs = tmp.0.join("pkgs");
    let (list, _) = kept(&pkgs, "linux-64", &[("a-1-0.conda", b"aaa")]);
    // The archive replaced and the record written for it: the list, from
    // conda-forge's metadata, still says the old hash.
    fs::write(pkgs.join("a-1-0.conda"), b"evil").unwrap();
    let forged = Record {
        version: "1".into(),
        target: "linux-x64".into(),
        created: 1,
        micromamba: None,
        packages: vec![Package {
            file: "a-1-0.conda".into(),
            sha256: sha(b"evil"),
            size: 4,
        }],
    };
    let forged = serde_json::to_string(&forged).unwrap();
    assert_eq!(
        check(Some(&list), Some(&forged), "linux-64", &pkgs),
        Err(Fallback::Disagree("a-1-0.conda".into()))
    );
    // One package fewer in the record.
    let (list2, _) = kept(
        &pkgs,
        "linux-64",
        &[("a-1-0.conda", b"aaa"), ("c-1-0.conda", b"c")],
    );
    let (_, one) = kept(&tmp.0.join("other"), "linux-64", &[("a-1-0.conda", b"aaa")]);
    assert!(matches!(
        check(Some(&list2), Some(&one), "linux-64", &pkgs),
        Err(Fallback::Disagree(_))
    ));
}

#[test]
fn never_an_archive_of_another_target() {
    let tmp = TempDir::new("conda-target");
    let pkgs = tmp.0.join("pkgs");
    let (list, record) = kept(&pkgs, "osx-arm64", &[("a-1-0.conda", b"aaa")]);
    assert_eq!(
        check(Some(&list), Some(&record), "linux-64", &pkgs),
        Err(Fallback::OtherTarget("a-1-0.conda".into()))
    );
    assert!(check(Some(&list), Some(&record), "osx-arm64", &pkgs).is_ok());
    let (noarch, record) = kept(&pkgs, "noarch", &[("t-1-0.conda", b"t")]);
    assert!(check(Some(&noarch), Some(&record), "linux-aarch64", &pkgs).is_ok());
}

#[test]
fn file_urls_of_a_kit_folder_with_spaces() {
    assert_eq!(
        file_url(Path::new("/media/u/My Kit (1)/tmp/kept/a-1-0.conda")),
        "file:///media/u/My%20Kit%20%281%29/tmp/kept/a-1-0.conda"
    );
    assert_eq!(
        file_url(Path::new(r"C:\Temp\Atollo\a b\x+1.conda")),
        "file:///C:/Temp/Atollo/a%20b/x%2B1.conda"
    );
}

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
    let mac = Kit::at_target(tmp.0.clone(), Target::DARWIN_X64);
    kept(&pkgs_dir(&kit), "linux-64", &[("a-1-0.conda", b"12345")]);
    kept(&pkgs_dir(&mac), "osx-64", &[("a-1-0.conda", b"12345")]);
    assert_eq!(free(&kit).unwrap(), 5);
    assert!(!root(&kit).exists());
    assert!(root(&mac).exists());
    assert_eq!(free(&kit).unwrap(), 0);
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
    assert_eq!(ruby.conda_prefix(Target::WIN32_ARM64, dir), None);
    let mongo = installer("mongodb").unwrap();
    assert!(mongo.conda_prefix(Target::LINUX_ARM64, dir).is_some());
    assert!(mongo.conda_prefix(Target::DARWIN_ARM64, dir).is_none());
    let sqlite = installer("sqlite").unwrap();
    assert!(sqlite.conda_prefix(Target::LINUX_ARM64, dir).is_some());
    assert!(sqlite.conda_prefix(Target::LINUX_X64, dir).is_none());
    // Every component that cannot follow the kit says where its environment is.
    for t in [
        Target::LINUX_X64,
        Target::LINUX_ARM64,
        Target::DARWIN_X64,
        Target::DARWIN_ARM64,
    ] {
        for c in Catalog::get().all() {
            let inst = installer(&c.id).unwrap();
            if !inst.relocatable(t) {
                assert!(inst.conda_prefix(t, dir).is_some(), "{} on {t}", c.id);
            }
        }
    }
}

struct Print;

impl crate::setup::runner::Sink for Print {
    fn emit(&mut self, event: crate::setup::runner::Event) {
        use crate::setup::runner::Event;
        match event {
            Event::Note { id, text } => eprintln!("  {id}: {text}"),
            Event::Done { id, outcome } => eprintln!("DONE {id}: {outcome:?}"),
            _ => {}
        }
    }
}

/// The program of `c` that answers, relative to its default folder.
fn answers(kit: &Kit, c: &Component) -> String {
    let dir = crate::versions::default_dir(kit, c);
    let (exe, arg) = match c.id.as_str() {
        "r" => ("bin/R", "--version"),
        "ruby" => ("bin/ruby", "-v"),
        "mongodb" => ("bin/mongod", "--version"),
        "sqlite" => ("sqlite3", "--version"),
        "cc" => ("make/bin/make", "--version"),
        other => panic!("{other}"),
    };
    let out = std::process::Command::new(crate::fsx::join_rel(&dir, exe))
        .arg(arg)
        .output()
        .unwrap_or_else(|e| panic!("{}: {e}", c.id));
    assert!(out.status.success(), "{} {exe} {arg}: {out:?}", c.id);
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.lines().next().unwrap_or_default().to_owned()
}

/// Files under `dir`, by relative path and size: the data that a
/// reinstallation must leave alone.
fn listing(dir: &Path) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(m) = fs::metadata(&p) {
                let rel = p.strip_prefix(dir).unwrap().to_string_lossy().into_owned();
                // MongoDB rewrites its diagnostics while running: not data.
                if !rel.contains("diagnostic.data") && !rel.ends_with(".lock") {
                    out.push((rel, m.len()));
                }
            }
        }
    }
    out.sort();
    out
}

/// D-118 with the real environments of the kit given by ATOLLO_TEST_KIT
/// (prepared with `prepare --only r,ruby,mongodb,sqlite`, plus `cc` where
/// it is conda): the kit folder is moved, the start marks them, a
/// preparation through a proxy that does not exist (and, run under
/// `unshare -rn`, without any network) creates them again from the kept
/// packages; the programs answer from the new folder, gems and MongoDB's
/// data are untouched. Then the same back to the first folder.
#[test]
fn real_offline_reinstall_after_a_move() {
    let Ok(first) = std::env::var("ATOLLO_TEST_KIT") else {
        eprintln!("ATOLLO_TEST_KIT not set: skipped");
        return;
    };
    let first = PathBuf::from(first);
    // No space in the new name: conda-forge's `bin/R` writes `SED=<prefix>/bin/sed`
    // without quotes, so R's shell script breaks in any folder with a space,
    // after a move or not.
    let moved = first.with_file_name(format!(
        "{}-moved-d118",
        first.file_name().unwrap().to_string_lossy()
    ));
    let target = Target::detect();
    let kit = Kit::at_target(first.clone(), target);
    let ids: Vec<String> = Catalog::get()
        .all()
        .iter()
        .filter(|c| c.is_installed(&kit))
        .filter(|c| installer(&c.id).is_some_and(|i| !i.relocatable(target)))
        .map(|c| c.id.clone())
        .collect();
    assert!(
        !ids.is_empty(),
        "no conda environment installed in ATOLLO_TEST_KIT"
    );
    let lists = status(&kit).lists;
    for id in &ids {
        assert!(
            lists.iter().any(|l| l.starts_with(&format!("{id}@"))),
            "{id}: no kept packages, install it again with this version of the app"
        );
    }
    eprintln!(
        "conda environments: {ids:?}, kept: {} MB",
        status(&kit).bytes >> 20
    );
    let data = |root: &Path| {
        let d = root.join("data");
        (
            listing(&d.join("ruby-gems")),
            listing(&d.join("db").join("mongodb")),
        )
    };
    let before = data(&first);
    let mut settings = Settings::default();
    settings.network.https_proxy = Some("http://127.0.0.1:9".into());
    settings.network.http_proxy = Some("http://127.0.0.1:9".into());
    let net = crate::net::Net::new(&settings, None).unwrap();
    let env = crate::env::KitEnv::default();
    let cancel = std::sync::atomic::AtomicBool::new(false);

    for (from, to) in [(&first, &moved), (&moved, &first)] {
        fs::rename(from, to).unwrap();
        let kit = Kit::at_target(to.clone(), target);
        kit.prepare().unwrap();
        for id in &ids {
            let c = Catalog::get().find(id).unwrap();
            assert!(c.needs_reinstall(&kit), "{id}: not marked after the move");
        }
        let ctx = Ctx {
            kit: &kit,
            net: &net,
            env: &env,
            cancel: &cancel,
        };
        let started = std::time::Instant::now();
        let opts = crate::setup::runner::Options {
            only: ids.clone(),
            ..Default::default()
        };
        let results = crate::setup::runner::run(&ctx, &opts, &mut Print).unwrap();
        eprintln!(
            "moved to {}: {} s",
            to.display(),
            started.elapsed().as_secs()
        );
        for id in &ids {
            let c = Catalog::get().find(id).unwrap();
            let outcome = results.iter().find(|(i, _)| i == id).map(|(_, o)| o);
            assert!(
                matches!(
                    outcome,
                    Some(crate::setup::runner::Outcome::Installed { .. })
                ),
                "{id}: {outcome:?}"
            );
            assert!(!c.needs_reinstall(&kit), "{id}: still marked");
            eprintln!("{id}: {}", answers(&kit, c));
        }
        if ids.iter().any(|i| i == "r") {
            let r = Catalog::get().find("r").unwrap();
            let home = std::process::Command::new(
                crate::versions::default_dir(&kit, r).join("bin").join("R"),
            )
            .arg("RHOME")
            .output()
            .unwrap();
            let home = String::from_utf8_lossy(&home.stdout).trim().to_owned();
            assert!(
                Path::new(&home).starts_with(to),
                "R_HOME {home} is not under {}",
                to.display()
            );
        }
        assert_eq!(data(to), before, "gems or MongoDB data changed");
    }
}

/// A conda environment in `prefix` whose `conda-meta` has `records`
/// (package name, version, files).
fn env_with(prefix: &Path, records: &[(&str, &str, &[&str])]) {
    let meta = prefix.join("conda-meta");
    fs::create_dir_all(&meta).unwrap();
    for (name, version, files) in records {
        let record = serde_json::json!({ "name": name, "version": version, "files": files });
        fs::write(
            meta.join(format!("{name}-{version}-h0_0.json")),
            record.to_string(),
        )
        .unwrap();
    }
}

/// D-121: the launchers are the programs of the component's package only,
/// from its record (Ruby 4.0.7 of conda-forge: 16 of the 30 of `bin`).
#[test]
fn launchers_only_for_the_programs_of_the_package() {
    let tmp = TempDir::new("conda-launchers");
    let prefix = tmp.0.join("ruby").join("4.0");
    env_with(
        &prefix,
        &[
            (
                "ruby",
                "4.0.7",
                &["bin/ruby", "bin/gem", "bin/irb", "lib/libruby.so", "bin/"],
            ),
            // Same start of the name, another package.
            ("ruby-dev", "1.0", &["bin/not-ruby"]),
            ("openssl", "3.5.0", &["bin/openssl", "bin/c_rehash"]),
            ("ncurses", "6.5", &["bin/tput", "bin/clear"]),
        ],
    );
    assert_eq!(
        package_programs(&prefix, "ruby").unwrap(),
        ["gem", "irb", "ruby"]
    );
    // An old launcher of a program the package no longer has goes.
    let dir = prefix.join(LAUNCHERS);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("old"), "").unwrap();
    write_launchers(&prefix, "ruby").unwrap();
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["gem", "irb", "ruby"]);
    let text = fs::read_to_string(dir.join("ruby")).unwrap();
    assert_eq!(text, launcher_text("ruby"));
    assert!(text.starts_with("#!/bin/sh\n") && text.contains(r#"exec "$d/bin/ruby" "$@""#));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(dir.join("ruby")).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0o111, "executable");
    }
    // No record of the package: an error.
    assert!(package_programs(&prefix, "mongodb").is_err());
    assert!(write_launchers(&tmp.0.join("empty"), "r-base").is_err());
}

/// The launcher runs the program of `bin` with that `bin` first in its
/// PATH, from any folder, also through a path with a space.
#[cfg(unix)]
#[test]
fn a_launcher_runs_the_program_with_its_bin_first() {
    let tmp = TempDir::new("conda-launcher-run");
    let prefix = tmp.0.join("k it").join("r");
    env_with(&prefix, &[("r-base", "4.6.1", &["bin/Rscript"])]);
    let bin = prefix.join("bin");
    crate::router::fs::write_executable(
        &bin.join("Rscript"),
        b"#!/bin/sh\necho \"args=$*\"\necho \"path=${PATH%%:*}\"\n",
    )
    .unwrap();
    write_launchers(&prefix, "r-base").unwrap();
    let out = std::process::Command::new(prefix.join(LAUNCHERS).join("Rscript"))
        .args(["-e", "1"])
        .current_dir(&tmp.0)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("args=-e 1"), "{text}");
    assert!(text.contains(&format!("path={}", bin.display())), "{text}");
}

/// At start: `kit-bin` for the lines installed before the kit wrote it,
/// on the targets where the component is a conda environment; a line that
/// has it already is left alone.
#[test]
fn launchers_for_older_installations() {
    let tmp = TempDir::new("conda-launchers-old");
    let ruby = Catalog::get().find("ruby").unwrap();
    let kit = Kit::at_target(tmp.0.join("Kit"), Target::LINUX_X64);
    for (line, version) in [("4.0", "4.0.7"), ("3.4", "3.4.9")] {
        let dir = crate::fsx::join_rel(kit.apps(), &format!("ruby/{line}"));
        env_with(&dir, &[("ruby", version, &["bin/ruby", "bin/gem"])]);
        fs::create_dir_all(dir.join("bin")).unwrap();
        fs::write(dir.join("bin").join("ruby"), "").unwrap();
        crate::versions::write_version(&dir, version).unwrap();
    }
    ruby.save_version(&kit, "4.0.7").unwrap();
    let kept = crate::fsx::join_rel(kit.apps(), "ruby/3.4").join(LAUNCHERS);
    fs::create_dir_all(&kept).unwrap();
    launchers_of_older_installations(&kit);
    let new = crate::fsx::join_rel(kit.apps(), "ruby/4.0").join(LAUNCHERS);
    assert!(new.join("ruby").is_file() && new.join("gem").is_file());
    assert!(
        !kept.join("ruby").exists(),
        "a line with them is left alone"
    );

    // Windows: Ruby is not a conda environment, nothing written.
    let win = Kit::at_target(tmp.0.join("Win"), Target::WIN32_X64);
    let dir = crate::fsx::join_rel(win.apps(), "ruby/4.0");
    env_with(&dir, &[("ruby", "4.0.7", &["bin/ruby"])]);
    fs::create_dir_all(dir.join("bin")).unwrap();
    fs::write(dir.join("bin").join("ruby.exe"), "").unwrap();
    ruby.save_version(&win, "4.0.7").unwrap();
    launchers_of_older_installations(&win);
    assert!(!dir.join(LAUNCHERS).exists());
}
