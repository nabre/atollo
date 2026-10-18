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

/// The newest LTS, not the newest release (D-108); a line asked for gives
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
        // follows the kit everywhere (D-112); off Windows it is staged
        // where its target cannot run (`git_staged`).
        assert_eq!(
            Git.custom(t),
            !t.is_windows() && !crate::router::target::can_execute_on_host(t),
            "git on {t}"
        );
        assert!(Git.relocatable(t), "git on {t}");
        // Archives everywhere.
        assert!(!Python.custom(t), "python on {t}");
        // Processes everywhere.
        let r = super::super::installer("r").map(|i| i.custom(t));
        assert_eq!(r, Some(true), "r on {t}");
    }
}

/// D-112: the dugite-native asset of each Unix target (names of the
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

/// D-112: the credential helper goes into dugite's `etc/gitconfig` once,
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
    assert!(gitconfig_with_helper("", "cache").is_some_and(|t| t.starts_with("# Atollo")));
}

/// Review of D-112: dugite-native for macOS or Linux prepared from a
/// host that cannot run it (Windows above all, D-114) is staged, not
/// unpacked there; PortableGit and the host's own target install directly.
#[test]
fn git_is_staged_where_its_target_cannot_run() {
    let between = |host: Target, target: Target, rosetta: bool| {
        git_staged(
            target,
            crate::router::target::can_execute_between(host, target, rosetta),
        )
    };
    for host in Target::ALL {
        for target in Target::ALL {
            let staged = between(host, target, false);
            // PortableGit is never staged (the 7z self-extractor unpacks
            // anywhere); from Windows every Unix Git is.
            if target.is_windows() {
                assert!(!staged, "{host} -> {target}: PortableGit");
            } else if host.is_windows() {
                assert!(staged, "{host} -> {target}: no links on Windows");
            }
            if host == target {
                assert!(!staged, "{host}: its own target");
            }
        }
    }
    // Another system or CPU without translation: staged.
    assert!(between(Target::LINUX_X64, Target::DARWIN_ARM64, false));
    assert!(between(Target::LINUX_X64, Target::LINUX_ARM64, false));
    assert!(between(Target::DARWIN_ARM64, Target::DARWIN_X64, false));
    // With Rosetta 2 an Apple Silicon Mac runs the x64 Git itself.
    assert!(!between(Target::DARWIN_ARM64, Target::DARWIN_X64, true));

    // `should_stage` on this host, for a kit of every target.
    let tmp = crate::kit::tests::TempDir::new("git-stage");
    let git = crate::setup::catalog::Catalog::get().find("git").unwrap();
    let net = crate::net::Net::new(&Settings::default(), None).unwrap();
    let env = crate::env::KitEnv::default();
    let cancel = std::sync::atomic::AtomicBool::new(false);
    for target in Target::ALL {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        let ctx = Ctx {
            kit: &kit,
            net: &net,
            env: &env,
            cancel: &cancel,
        };
        assert_eq!(
            crate::router::bootstrap::should_stage(&ctx, &Git, git),
            Git.custom(target),
            "{target}"
        );
        if cfg!(windows) && !target.is_windows() {
            assert!(
                crate::router::bootstrap::should_stage(&ctx, &Git, git),
                "{target} from Windows"
            );
        }
    }
}

/// Review of D-112: dugite's links (`git-remote-https` → `git-remote-http`,
/// `git-upload-pack` → `git`) are lost when its tarball is unpacked on
/// Windows; the Git left behind answers `--version` but cannot clone, so
/// `finish` refuses it (`missing_git_links`).
#[test]
fn dugite_links_lost_on_windows_are_noticed() {
    let tmp = crate::kit::tests::TempDir::new("git-links");
    let archive = tmp.0.join("dugite.tar.gz");
    {
        let gz = flate2::write::GzEncoder::new(
            fs::File::create(&archive).unwrap(),
            flate2::Compression::fast(),
        );
        let mut tar = tar::Builder::new(gz);
        for name in [
            "bin/git",
            "libexec/git-core/git",
            "libexec/git-core/git-remote-http",
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_size(2);
            header.set_mode(0o755);
            header.set_cksum();
            tar.append_data(&mut header, name, &b"#!"[..]).unwrap();
        }
        for (name, to) in [
            ("libexec/git-core/git-remote-https", "git-remote-http"),
            ("libexec/git-core/git-upload-pack", "git"),
            ("libexec/git-core/git-receive-pack", "git"),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_size(0);
            header.set_mode(0o777);
            tar.append_link(&mut header, name, to).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap();
    }
    let out = tmp.0.join("out");
    crate::net::archive::extract(&archive, Kind::TarGz, &out).unwrap();
    assert!(out.join("libexec").join("git-core").join("git").is_file());
    let missing = missing_git_links(&out);
    if cfg!(windows) {
        assert_eq!(
            missing,
            ["git-remote-https", "git-upload-pack", "git-receive-pack"]
        );
    } else {
        assert!(missing.is_empty(), "{missing:?}");
    }
    // A complete tree (regular files are as good as links).
    let full = tmp.0.join("full").join("libexec").join("git-core");
    fs::create_dir_all(&full).unwrap();
    for name in ["git-remote-https", "git-upload-pack", "git-receive-pack"] {
        fs::write(full.join(name), b"x").unwrap();
    }
    assert!(missing_git_links(&tmp.0.join("full")).is_empty());
    assert_eq!(missing_git_links(&tmp.0.join("none")).len(), 3);
}

/// D-112: the conda-forge Git of an older Atollo (the only one that
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

/// Why a fresh Linux kit got Python 3.15.0: with no line in the settings
/// `Python::latest` takes the first of `python_lines`, the newest final
/// minor (D-81, "ticked installs the newest"); 3.15.0 is final since
/// October 2026. Pre-releases never count. Same on every target.
#[test]
fn python_default_is_the_newest_final_line() {
    let h = "a".repeat(64);
    for t in Target::ALL {
        let triple = t.rust_triple();
        let sums = format!(
            "{h}  cpython-3.13.16+20261014-{triple}-install_only_stripped.tar.gz\n\
             {h}  cpython-3.14.4+20261014-{triple}-install_only_stripped.tar.gz\n\
             {h}  cpython-3.15.0+20261014-{triple}-install_only_stripped.tar.gz\n\
             {h}  cpython-3.16.0a1+20261014-{triple}-install_only_stripped.tar.gz\n"
        );
        assert_eq!(python_lines(&sums, triple)[0], "3.15", "{t}");
        let (_, version, _) = pick_python(&sums, "3.15", triple).unwrap();
        assert_eq!(version, "3.15.0", "{t}");
    }
    // Only when the list is empty.
    assert_eq!(PYTHON_DEFAULT, "3.13");
}
