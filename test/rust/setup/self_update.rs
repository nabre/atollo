use super::*;
use crate::kit::{tests::TempDir, Arch, Target};
use std::io::Write as _;

fn release(tag: &str, asset: &str) -> Release {
    Release {
        tag_name: tag.into(),
        draft: false,
        prerelease: false,
        assets: vec![GhAsset {
            name: asset.into(),
            browser_download_url: String::new(),
            digest: None,
        }],
    }
}

#[test]
fn versions_and_choice() {
    assert_eq!(parse_version("app-v1.2.3"), Some((1, 2, 3)));
    assert_eq!(parse_version("1.10.0"), Some((1, 10, 0)));
    assert_eq!(parse_version("app-v1.2"), None);
    assert_eq!(parse_version("v1.2.3.4"), None);
    let mut pre = release("app-v9.0.0", "Atollo-x64.exe");
    pre.prerelease = true;
    let list = vec![
        release("app-v1.0.1", "Atollo-x64.exe"),
        release("app-v1.2.0", "Atollo-x64.exe"),
        release("app-v1.3.0", "Atollo-arm64.exe"),
        release("node-v1", "Atollo-x64.exe"),
        pre,
    ];
    assert_eq!(
        newest(&list, "1.0.0", "Atollo-x64.exe").map(|r| r.tag_name.as_str()),
        Some("app-v1.2.0")
    );
    assert!(newest(&list, "1.2.0", "Atollo-x64.exe").is_none());
    assert_eq!(
        newest(&list, "1.0.0", "Atollo-arm64.exe").map(|r| r.tag_name.as_str()),
        Some("app-v1.3.0")
    );
}

/// The release asset of each of the six targets: no `.exe` off Windows.
#[test]
fn asset_names_per_target() {
    let names: Vec<String> = Target::ALL
        .iter()
        .map(|t| asset_name(&Kit::at_target(PathBuf::from("kit"), *t)))
        .collect();
    for (target, name) in Target::ALL.iter().zip(&names) {
        let expected = match target.as_str() {
            "win32-x64" => "Atollo-x64.exe",
            "win32-arm64" => "Atollo-arm64.exe",
            "darwin-arm64" => "Atollo-darwin-arm64.app.zip",
            "darwin-x64" => "Atollo-darwin-x64.app.zip",
            "linux-x64" => "Atollo-linux-x64.AppImage",
            _ => "Atollo-linux-arm64.AppImage",
        };
        assert_eq!(name, expected);
        assert_eq!(name.ends_with(".exe"), target.is_windows());
    }
}

/// The swap plan of the three systems, from any of them.
#[test]
fn swap_plan_per_system() {
    let root = PathBuf::from(if cfg!(windows) {
        r"D:\kit"
    } else {
        "/media/u/kit"
    });
    let update = root.join("data").join("app").join("update");

    let exe = root.join("Atollo.exe");
    assert_eq!(
        swap_plan(Os::Win32, &exe, None, &update),
        Some(SwapPlan {
            form: Form::Exe,
            current: exe.clone(),
            staged: update.join("Atollo.exe"),
            old: root.join("Atollo.old.exe"),
        })
    );

    let bundle = root.join("Atollo.app");
    let inner = bundle.join("Contents").join("MacOS").join("Atollo");
    assert_eq!(
        swap_plan(Os::Darwin, &inner, None, &update),
        Some(SwapPlan {
            form: Form::Bundle,
            current: bundle.clone(),
            staged: root.join("Atollo.app.new"),
            old: root.join("Atollo.app.old"),
        })
    );
    // A renamed bundle keeps its name.
    let renamed = root
        .join("Atollo.app")
        .join("Contents")
        .join("MacOS")
        .join("Atollo");
    let plan = swap_plan(Os::Darwin, &renamed, None, &update).unwrap();
    assert_eq!(plan.staged, root.join("Atollo.app.new"));
    assert_eq!(plan.old, root.join("Atollo.app.old"));
    // A plain binary (cargo run, no bundle) does not replace itself.
    assert_eq!(
        swap_plan(Os::Darwin, &root.join("Atollo"), None, &update),
        None
    );

    let image = root.join("Atollo.AppImage");
    let mounted = PathBuf::from("/tmp/.mount_AtolloXYZ/usr/bin/Atollo");
    assert_eq!(
        swap_plan(Os::Linux, &mounted, Some(&image), &update),
        Some(SwapPlan {
            form: Form::AppImage,
            current: image.clone(),
            staged: root.join("Atollo.AppImage.new"),
            old: root.join("Atollo.AppImage.old"),
        })
    );
    // The tar.gz binary, or a relative $APPIMAGE: nothing to swap.
    assert_eq!(
        swap_plan(Os::Linux, &root.join("Atollo"), None, &update),
        None
    );
    assert_eq!(
        swap_plan(
            Os::Linux,
            &mounted,
            Some(Path::new("Atollo.AppImage")),
            &update
        ),
        None
    );
    // No .exe name off Windows.
    for os in [Os::Darwin, Os::Linux] {
        let exe = if os.is_macos() { &inner } else { &mounted };
        let plan = swap_plan(os, exe, Some(&image), &update).unwrap();
        for p in [&plan.current, &plan.staged, &plan.old] {
            assert!(!p.to_string_lossy().ends_with(".exe"), "{}", p.display());
        }
    }
}

#[test]
fn staged_names_stay_in_the_kit_root() {
    let kit = Kit::at_target(PathBuf::from("kit"), Target::new(Os::Linux, Arch::X64));
    let info = |file: Option<&str>| Staged {
        version: "9.9.9".into(),
        sha256: String::new(),
        file: file.map(Into::into),
    };
    assert_eq!(staged_path(&kit, &info(None)), Some(staged_exe(&kit)));
    assert_eq!(
        staged_path(&kit, &info(Some("Atollo.AppImage.new"))),
        Some(kit.root().join("Atollo.AppImage.new"))
    );
    for bad in [
        "../x.new",
        "a/b.new",
        r"a\b.new",
        "Atollo.AppImage",
        ".new",
        "",
    ] {
        assert_eq!(staged_path(&kit, &info(Some(bad))), None, "{bad}");
    }
    // update.json of earlier versions (no `file`) still reads.
    let old: Staged = serde_json::from_str(r#"{"version":"1.0.0","sha256":"ab"}"#).unwrap();
    assert!(old.file.is_none());
}

fn write_info(kit: &Kit, version: &str, sha256: String, file: Option<&str>) {
    let info = Staged {
        version: version.into(),
        sha256,
        file: file.map(Into::into),
    };
    fs::create_dir_all(update_dir(kit)).unwrap();
    fs::write(staged_info(kit), serde_json::to_string(&info).unwrap()).unwrap();
}

/// Windows, unchanged: `Atollo.exe` → `Atollo.old.exe`, the staged exe in
/// its place.
#[test]
fn swap_and_cleanup_windows() {
    let tmp = TempDir::new("self-update");
    let kit = Kit::at_target(tmp.0.clone(), Target::new(Os::Win32, Arch::X64));
    let exe = tmp.0.join("Atollo.exe");
    fs::write(&exe, b"old").unwrap();
    let plan = swap_plan(Os::Win32, &exe, None, &update_dir(&kit)).unwrap();
    // Nothing staged: nothing changes.
    assert!(!apply_plan(&kit, &plan).unwrap());
    fs::create_dir_all(update_dir(&kit)).unwrap();
    fs::write(staged_exe(&kit), b"new").unwrap();
    let hash = verify::sha256_hex(&staged_exe(&kit)).unwrap();
    write_info(&kit, "999.0.0", hash.clone(), None);
    assert_eq!(
        Updater::new(kit.clone()).status().staged.as_deref(),
        Some("999.0.0")
    );
    assert!(apply_plan(&kit, &plan).unwrap());
    assert_eq!(fs::read(&exe).unwrap(), b"new");
    assert_eq!(fs::read(tmp.0.join("Atollo.old.exe")).unwrap(), b"old");
    assert!(!staged_exe(&kit).exists() && !staged_info(&kit).exists());
    remove_old_of(&plan);
    assert!(!tmp.0.join("Atollo.old.exe").exists());

    // A staged file changed after the check is never used.
    fs::write(staged_exe(&kit), b"evil").unwrap();
    write_info(&kit, "999.0.0", hash, None);
    assert!(apply_plan(&kit, &plan).is_err());
    assert_eq!(fs::read(&exe).unwrap(), b"new");
    assert!(!staged_exe(&kit).exists());
}

/// The same flow through `apply_staged` on the system running the tests,
/// where it is Windows (elsewhere a test binary is not a bundle or an
/// AppImage, so nothing is swapped and what was staged is dropped).
#[test]
fn apply_staged_on_this_system() {
    let tmp = TempDir::new("self-update-host");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    let exe = tmp.0.join("Atollo.exe");
    fs::write(&exe, b"old").unwrap();
    fs::create_dir_all(update_dir(&kit)).unwrap();
    fs::write(staged_exe(&kit), b"new").unwrap();
    let hash = verify::sha256_hex(&staged_exe(&kit)).unwrap();
    write_info(&kit, "999.0.0", hash, None);
    let applied = apply_staged(&kit, &exe).unwrap();
    assert_eq!(applied, cfg!(windows));
    let expected: &[u8] = if cfg!(windows) { b"new" } else { b"old" };
    assert_eq!(fs::read(&exe).unwrap(), expected);
    assert!(!staged_info(&kit).exists());
}

/// A zip like the macOS release asset: `Atollo.app/` with its program.
fn app_zip(path: &Path, program: &[u8]) {
    let mut zip = zip::ZipWriter::new(fs::File::create(path).unwrap());
    let stored =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.add_directory("Atollo.app/Contents/MacOS/", stored)
        .unwrap();
    zip.start_file(
        "Atollo.app/Contents/MacOS/Atollo",
        stored.unix_permissions(0o755),
    )
    .unwrap();
    zip.write_all(program).unwrap();
    zip.start_file(
        "Atollo.app/Contents/Info.plist",
        stored.unix_permissions(0o644),
    )
    .unwrap();
    zip.write_all(b"<plist/>").unwrap();
    zip.finish().unwrap();
}

/// macOS: the zip's bundle staged as `Atollo.app.new` next to the running
/// one, swapped at exit, `Atollo.app.old` removed at the next start.
#[test]
fn swap_and_cleanup_macos_bundle() {
    let tmp = TempDir::new("self-update-mac");
    let kit = Kit::at_target(tmp.0.clone(), Target::new(Os::Darwin, Arch::Arm64));
    let bundle = tmp.0.join("Atollo.app");
    let macos = bundle.join("Contents").join("MacOS");
    fs::create_dir_all(&macos).unwrap();
    fs::write(macos.join("Atollo"), b"old").unwrap();
    let plan = swap_plan(Os::Darwin, &macos.join("Atollo"), None, &update_dir(&kit)).unwrap();

    fs::create_dir_all(update_dir(&kit)).unwrap();
    let download = update_dir(&kit).join("Atollo-darwin-arm64.app.zip");
    app_zip(&download, b"new");
    let digest = stage(&plan, &download, &update_dir(&kit).join("unpacked")).unwrap();
    assert!(!download.exists() && !update_dir(&kit).join("unpacked").exists());
    let staged_program = plan.staged.join("Contents").join("MacOS").join("Atollo");
    assert_eq!(fs::read(&staged_program).unwrap(), b"new");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&staged_program).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0o111, "{mode:o}");
    }
    assert_eq!(digest, tree_sha256(&plan.staged).unwrap());
    write_info(&kit, "999.0.0", digest.clone(), Some("Atollo.app.new"));
    assert_eq!(
        Updater::new(kit.clone()).status().staged.as_deref(),
        Some("999.0.0")
    );
    // A sidecar file macOS adds on exFAT does not count as a change.
    fs::write(plan.staged.join("Contents").join("._Info.plist"), b"x").unwrap();

    assert!(apply_plan(&kit, &plan).unwrap());
    assert_eq!(fs::read(macos.join("Atollo")).unwrap(), b"new");
    assert_eq!(
        fs::read(plan.old.join("Contents").join("MacOS").join("Atollo")).unwrap(),
        b"old"
    );
    assert!(!plan.staged.exists() && !staged_info(&kit).exists());
    remove_old_of(&plan);
    assert!(!plan.old.exists());
    assert!(bundle.is_dir());

    // A bundle changed after staging is never used, and the app stays.
    fs::create_dir_all(plan.staged.join("Contents").join("MacOS")).unwrap();
    fs::write(
        plan.staged.join("Contents").join("MacOS").join("Atollo"),
        b"evil",
    )
    .unwrap();
    write_info(&kit, "999.0.0", digest, Some("Atollo.app.new"));
    assert!(apply_plan(&kit, &plan).is_err());
    assert_eq!(fs::read(macos.join("Atollo")).unwrap(), b"new");
    assert!(!plan.staged.exists() && !staged_info(&kit).exists());
}

#[test]
fn a_zip_without_one_bundle_is_refused() {
    let tmp = TempDir::new("self-update-zip");
    let zip_path = tmp.0.join("x.zip");
    let mut zip = zip::ZipWriter::new(fs::File::create(&zip_path).unwrap());
    zip.start_file("README.txt", zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"no app").unwrap();
    zip.finish().unwrap();
    let plan = SwapPlan {
        form: Form::Bundle,
        current: tmp.0.join("Atollo.app"),
        staged: tmp.0.join("Atollo.app.new"),
        old: tmp.0.join("Atollo.app.old"),
    };
    assert!(stage(&plan, &zip_path, &tmp.0.join("unpacked")).is_err());
    assert!(!plan.staged.exists());
}

/// Linux: the AppImage staged as `Atollo.AppImage.new` with mode 755,
/// swapped at exit.
#[test]
fn swap_and_cleanup_linux_appimage() {
    let tmp = TempDir::new("self-update-linux");
    let kit = Kit::at_target(tmp.0.clone(), Target::new(Os::Linux, Arch::X64));
    let image = tmp.0.join("Atollo.AppImage");
    fs::write(&image, b"old").unwrap();
    let mounted = PathBuf::from("/tmp/.mount_Atollo/usr/bin/Atollo");
    let plan = swap_plan(Os::Linux, &mounted, Some(&image), &update_dir(&kit)).unwrap();

    fs::create_dir_all(update_dir(&kit)).unwrap();
    let download = update_dir(&kit).join("Atollo-linux-x64.AppImage");
    fs::write(&download, b"new").unwrap();
    let digest = stage(&plan, &download, &update_dir(&kit).join("unpacked")).unwrap();
    assert!(!download.exists());
    assert_eq!(fs::read(&plan.staged).unwrap(), b"new");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&plan.staged).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o755, "{mode:o}");
    }
    write_info(&kit, "999.0.0", digest, Some("Atollo.AppImage.new"));

    // Staged for a copy with another name: dropped, nothing swapped.
    let other = SwapPlan {
        current: tmp.0.join("Other.AppImage"),
        staged: tmp.0.join("Other.AppImage.new"),
        old: tmp.0.join("Other.AppImage.old"),
        ..plan.clone()
    };
    assert!(!apply_plan(&kit, &other).unwrap());
    assert!(!plan.staged.exists() && !staged_info(&kit).exists());
    assert_eq!(fs::read(&image).unwrap(), b"old");

    // Staged again, for this copy.
    fs::write(&download, b"new").unwrap();
    let digest = stage(&plan, &download, &update_dir(&kit).join("unpacked")).unwrap();
    write_info(&kit, "999.0.0", digest, Some("Atollo.AppImage.new"));
    assert!(apply_plan(&kit, &plan).unwrap());
    assert_eq!(fs::read(&image).unwrap(), b"new");
    assert_eq!(fs::read(&plan.old).unwrap(), b"old");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&image).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o755, "{mode:o}");
    }
    remove_old_of(&plan);
    assert!(!plan.old.exists() && image.is_file());
}

#[test]
fn tree_digest_sees_content_and_names() {
    let tmp = TempDir::new("self-update-tree");
    let a = tmp.0.join("a");
    fs::create_dir_all(a.join("x")).unwrap();
    fs::write(a.join("x").join("f"), b"1").unwrap();
    let first = tree_sha256(&a).unwrap();
    assert_eq!(first, tree_sha256(&a).unwrap());
    fs::write(a.join(".DS_Store"), b"finder").unwrap();
    assert_eq!(first, tree_sha256(&a).unwrap());
    fs::write(a.join("x").join("f"), b"2").unwrap();
    assert_ne!(first, tree_sha256(&a).unwrap());
    fs::write(a.join("x").join("f"), b"1").unwrap();
    fs::rename(a.join("x").join("f"), a.join("x").join("g")).unwrap();
    assert_ne!(first, tree_sha256(&a).unwrap());
}

/// The bundle configuration (D-113): Info.plist repeats tauri.conf.json,
/// the icons and the entitlements it names are there, and the bundles are
/// the macOS app and the AppImage only (Windows stays one exe).
#[test]
fn bundle_config_agrees() {
    let conf: serde_json::Value =
        serde_json::from_str(include_str!("../../../tauri/tauri.conf.json")).unwrap();
    let plist = include_str!("../../../tauri/Info.plist");
    let entitlements = include_str!("../../../tauri/entitlements.plist");
    let value = |key: &str| {
        let at = plist.find(&format!("<key>{key}</key>"))?;
        let rest = &plist[at..];
        let start = rest.find("<string>")? + "<string>".len();
        let end = rest.find("</string>")?;
        Some(rest[start..end].to_owned())
    };
    let product = conf["productName"].as_str().unwrap();
    assert_eq!(value("CFBundleName").as_deref(), Some(product));
    assert_eq!(value("CFBundleDisplayName").as_deref(), Some(product));
    assert_eq!(
        value("CFBundleIdentifier").as_deref(),
        conf["identifier"].as_str()
    );
    let mac = &conf["bundle"]["macOS"];
    assert_eq!(mac["minimumSystemVersion"], "12.0");
    assert_eq!(value("LSMinimumSystemVersion").as_deref(), Some("12.0"));
    assert_eq!(mac["signingIdentity"], "-");
    assert_eq!(mac["entitlements"], "entitlements.plist");
    for key in [
        "com.apple.security.cs.allow-jit",
        "com.apple.security.cs.disable-library-validation",
    ] {
        assert!(entitlements.contains(&format!("<key>{key}</key>")), "{key}");
    }
    assert_eq!(conf["bundle"]["active"], true);
    assert_eq!(
        conf["bundle"]["targets"],
        serde_json::json!(["app", "appimage"])
    );
    let tauri = Path::new(env!("CARGO_MANIFEST_DIR"));
    let icons: Vec<&str> = conf["bundle"]["icon"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i.as_str().unwrap())
        .collect();
    // Windows keeps its icon first (the exe's resource).
    assert_eq!(icons.first(), Some(&"icons/icon.ico"));
    assert!(icons.contains(&"icons/icon.icns"));
    for icon in icons {
        assert!(tauri.join(icon).is_file(), "{icon}");
    }
    // The release assets and the swap use the binary's name.
    assert_eq!(conf["mainBinaryName"], "Atollo");
}

/// A check that finds a newer version always reports it: staged where the
/// copy replaces itself, to download by hand for a plain binary (the
/// tar.gz, cargo run), never as a failed check.
#[test]
fn check_results_set_the_status() {
    let mut s = Status {
        current: "0.4.0".into(),
        checking: true,
        error: Some("earlier".into()),
        ..Status::default()
    };
    record(&mut s, Ok(Found::Manual("0.5.0".into())));
    assert_eq!(s.latest.as_deref(), Some("0.5.0"));
    assert!(s.manual);
    assert_eq!(s.error, None);
    assert_eq!(s.staged, None);
    assert!(!s.checking);

    s.checking = true;
    record(&mut s, Ok(Found::Staged("0.5.0".into())));
    assert_eq!(s.staged.as_deref(), Some("0.5.0"));
    assert!(!s.manual);

    record(&mut s, Ok(Found::UpToDate("0.5.0".into())));
    assert_eq!(s.latest.as_deref(), Some("0.5.0"));
    // A version staged earlier stays staged.
    assert_eq!(s.staged.as_deref(), Some("0.5.0"));

    s.checking = true;
    record(&mut s, Err(anyhow!("offline")));
    assert_eq!(s.error.as_deref(), Some("offline"));
    assert_eq!(s.latest.as_deref(), Some("0.5.0"));
    assert!(!s.checking);

    let json = serde_json::to_value(&s).unwrap();
    assert_eq!(json["manual"], false);
}
