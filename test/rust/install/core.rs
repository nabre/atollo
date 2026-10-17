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

/// The line of the Python actually in the tree, not the one of the settings
/// (BUG-24b): `lib/python3.N` on macOS and Linux, `python3NN.dll` on Windows.
#[test]
fn python_minor_in_tree() {
    let tmp = crate::kit::tests::TempDir::new("python-minor");
    let unix = tmp.0.join("unix");
    fs::create_dir_all(unix.join("lib").join("python3.14").join("site-packages")).unwrap();
    fs::create_dir_all(unix.join("lib").join("pkgconfig")).unwrap();
    fs::write(unix.join("lib").join("libpython3.14.so"), b"").unwrap();
    assert_eq!(python_minor_in(&unix, false).as_deref(), Some("3.14"));
    assert_eq!(python_minor_in(&unix, true), None);

    let win = tmp.0.join("win");
    fs::create_dir_all(win.join("Lib").join("site-packages")).unwrap();
    for f in [
        "python.exe",
        "python3.dll",
        "python314.dll",
        "vcruntime140.dll",
    ] {
        fs::write(win.join(f), b"").unwrap();
    }
    assert_eq!(python_minor_in(&win, true).as_deref(), Some("3.14"));
    assert_eq!(python_minor_in(&win, false), None);

    // A file named like the folder is not the folder; nothing there: None.
    let odd = tmp.0.join("odd");
    fs::create_dir_all(odd.join("lib")).unwrap();
    fs::write(odd.join("lib").join("python3.12"), b"").unwrap();
    assert_eq!(python_minor_in(&odd, false), None);
    assert_eq!(python_minor_in(&tmp.0.join("missing"), false), None);
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

/// The newest LTS, not the newest release (T0.4.6); a line asked for gives
/// its newest; a newer major already installed is not taken back.
#[test]
fn node_pick_is_the_newest_lts() {
    let index: Vec<NodeEntry> = serde_json::from_value(serde_json::json!([
        {"version": "v25.1.0", "files": ["win-x64-zip", "linux-arm64"], "lts": false},
        {"version": "v25.0.0", "files": ["win-x64-zip"], "lts": false},
        {"version": "v24.11.1", "files": ["linux-arm64"], "lts": "Krypton"},
        {"version": "v24.11.0", "files": ["win-x64-zip", "linux-arm64"], "lts": "Krypton"},
        {"version": "v24.10.0", "files": ["win-x64-zip"], "lts": false},
        {"version": "v22.21.1", "files": ["win-x64-zip"], "lts": "Jod"},
        {"version": "v21.7.3", "files": ["osx-arm64-tar"], "lts": false}
    ]))
    .unwrap();
    let pick = |wanted: &str, line: Option<&str>, installed: Option<&str>| {
        node_pick(&index, wanted, line, installed).map(|e| e.version.as_str())
    };
    assert_eq!(pick("win-x64-zip", None, None), Some("v24.11.0"));
    assert_eq!(pick("linux-arm64", None, None), Some("v24.11.1"));
    assert_eq!(
        pick("win-x64-zip", None, Some("v22.21.1")),
        Some("v24.11.0")
    );
    assert_eq!(
        pick("win-x64-zip", None, Some("v24.11.0")),
        Some("v24.11.0")
    );
    // A "Current" installed before: its major, never back to 24.
    assert_eq!(pick("win-x64-zip", None, Some("v25.0.0")), Some("v25.1.0"));
    assert_eq!(pick("win-x64-zip", Some("25.0"), None), Some("v25.0.0"));
    assert_eq!(pick("win-x64-zip", Some("22.21"), None), Some("v22.21.1"));
    assert_eq!(pick("win-x64-zip", Some("23.11"), None), None);
    // No LTS for that archive: the newest there is.
    assert_eq!(pick("osx-arm64-tar", None, None), Some("v21.7.3"));
    assert_eq!(pick("osx-x64-tar", None, None), None);
    // The major installed has no build here: the LTS.
    assert_eq!(pick("linux-arm64", None, Some("v26.0.0")), Some("v24.11.1"));
}

#[test]
fn custom_by_target() {
    for t in Target::ALL {
        // Git: an archive everywhere (PortableGit, dugite-native), and it
        // follows the kit everywhere (WP-2.1).
        assert!(!Git.custom(t), "git on {t}");
        assert!(Git.relocatable(t), "git on {t}");
        // Archives everywhere.
        assert!(!Python.custom(t), "python on {t}");
        // Processes everywhere.
        let r = super::super::installer("r").map(|i| i.custom(t));
        assert_eq!(r, Some(true), "r on {t}");
    }
}

/// WP-2.1: the dugite-native asset of each Unix target (names of the
/// v2.53.0-4 release, checked on GitHub), never the 32-bit builds;
/// PortableGit on Windows.
#[test]
fn dugite_asset_per_target() {
    let assets = [
        "dugite-native-v2.53.0-4098283-macOS-arm64.tar.gz",
        "dugite-native-v2.53.0-4098283-macOS-arm64.tar.gz.sha256",
        "dugite-native-v2.53.0-4098283-macOS-arm64.lzma",
        "dugite-native-v2.53.0-4098283-macOS-x64.tar.gz",
        "dugite-native-v2.53.0-4098283-ubuntu-arm.tar.gz",
        "dugite-native-v2.53.0-4098283-ubuntu-arm64.tar.gz",
        "dugite-native-v2.53.0-4098283-ubuntu-x64.tar.gz",
        "dugite-native-v2.53.0-4098283-ubuntu-x86.tar.gz",
        "dugite-native-v2.53.0-4098283-windows-arm64.tar.gz",
        "dugite-native-v2.53.0-4098283-windows-x64.tar.gz",
    ];
    for (t, want) in [
        (Target::DARWIN_ARM64, Some("macOS-arm64.tar.gz")),
        (Target::DARWIN_X64, Some("macOS-x64.tar.gz")),
        (Target::LINUX_X64, Some("ubuntu-x64.tar.gz")),
        (Target::LINUX_ARM64, Some("ubuntu-arm64.tar.gz")),
        (Target::WIN32_X64, None),
        (Target::WIN32_ARM64, None),
    ] {
        let Some(re) = dugite_asset_pattern(t) else {
            assert_eq!(want, None, "{t}");
            continue;
        };
        let re = regex::Regex::new(&re).unwrap();
        let found: Vec<&str> = assets.iter().copied().filter(|a| re.is_match(a)).collect();
        assert_eq!(found.len(), 1, "{t}: {found:?}");
        assert!(found[0].ends_with(want.unwrap()), "{t}: {found:?}");
    }
    assert_eq!(dugite_version("v2.53.0-4"), "2.53.0-4");
    assert_eq!(
        crate::versions::line_of(&dugite_version("v2.53.0-4")).as_deref(),
        Some("2.53")
    );
}

/// T2.1.4: the credential helper goes into dugite's `etc/gitconfig` once,
/// next to what is there; a helper already set stays.
#[test]
fn unix_gitconfig_gets_the_credential_helper_once() {
    assert_eq!(unix_credential_helper(true), "osxkeychain");
    assert_eq!(unix_credential_helper(false), "cache --timeout=3600");
    let dugite = "[include]\n  path=/etc/gitconfig\n\n[credential \"https://dev.azure.com\"]\n  useHttpPath = true\n";
    let new = gitconfig_with_helper(dugite, "cache --timeout=3600").unwrap();
    assert!(new.starts_with(dugite), "{new}");
    assert!(
        new.ends_with("[credential]\n\thelper = cache --timeout=3600\n"),
        "{new}"
    );
    assert_eq!(gitconfig_with_helper(&new, "osxkeychain"), None);
    // A helper of the user: kept, also with other spacing and case.
    assert_eq!(
        gitconfig_with_helper("[Credential]\n    Helper=store\n", "cache"),
        None
    );
    // A helper only for one host is not the default one.
    assert!(gitconfig_with_helper(
        "[credential \"https://example.com\"]\n\thelper = store",
        "cache"
    )
    .is_some_and(|t| t.contains("example.com\"]\n\thelper = store\n#")));
    assert!(gitconfig_with_helper("", "cache").is_some_and(|t| t.starts_with("# Dev-Kit")));
}

/// T2.1.3: the conda-forge Git of an older Dev-Kit (the only one that
/// cannot move) is marked once to be installed again; dugite's folder and
/// Windows are left alone.
#[test]
fn conda_git_is_marked_once_for_reinstall() {
    let tmp = crate::kit::tests::TempDir::new("conda-git");
    let git = crate::setup::catalog::Catalog::get().find("git").unwrap();
    for target in [Target::LINUX_X64, Target::DARWIN_ARM64, Target::WIN32_X64] {
        let kit = crate::kit::Kit::at_target(tmp.0.join(target.as_str()), target);
        let store = git.folder_path(&kit).unwrap();
        // The installed line 2.51, with the key of `target`.
        let line = store.join("2.51");
        let key = git.key_for(target);
        let key_file = crate::fsx::join_rel(&line, key.strip_prefix("git/").unwrap());
        fs::create_dir_all(key_file.parent().unwrap()).unwrap();
        fs::write(&key_file, b"x").unwrap();
        fs::create_dir_all(line.join("libexec").join("git-core")).unwrap();
        git.save_version(&kit, "2.51.0").unwrap();
        Git.relocate(&kit, git).unwrap();
        assert!(!git.needs_reinstall(&kit), "{target}: dugite's layout");
        // An old conda line not removed yet (in use) does not count.
        fs::create_dir_all(store.join("2.50").join("conda-meta")).unwrap();
        Git.relocate(&kit, git).unwrap();
        assert!(!git.needs_reinstall(&kit), "{target}: old line");
        fs::create_dir_all(line.join("conda-meta")).unwrap();
        assert!(has_conda_git(&line));
        Git.relocate(&kit, git).unwrap();
        assert_eq!(git.needs_reinstall(&kit), !target.is_windows(), "{target}");
        // Installed again (install::install clears it) with dugite.
        git.clear_reinstall(&kit);
        fs::remove_dir_all(line.join("conda-meta")).unwrap();
        Git.relocate(&kit, git).unwrap();
        assert!(!git.needs_reinstall(&kit), "{target}: replaced");
    }
    assert!(!has_conda_git(&tmp.0.join("missing")));
    // The layout before lines: `git` itself is the environment.
    let flat = tmp.0.join("flat").join("git");
    fs::create_dir_all(flat.join("conda-meta")).unwrap();
    assert!(has_conda_git(&flat));
}
