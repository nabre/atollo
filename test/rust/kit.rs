use super::*;

/// A fresh folder under the system temp, removed on drop.
pub struct TempDir(pub PathBuf);
impl TempDir {
    pub fn new(name: &str) -> TempDir {
        let dir = std::env::temp_dir().join(format!(
            "atollo-test-{}-{}-{name}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn arch_follows_env_rule() {
    assert_eq!(Arch::from_env_values(Some("AMD64"), None), Arch::X64);
    assert_eq!(Arch::from_env_values(Some("ARM64"), None), Arch::Arm64);
    assert_eq!(
        Arch::from_env_values(Some("x86"), Some("arm64")),
        Arch::Arm64
    );
    assert_eq!(Arch::from_env_values(None, None), Arch::X64);
}

#[test]
fn root_is_exe_folder_with_odd_names() {
    for name in [
        "Atollo",
        "kit (2)",
        "a&b",
        "percent%20",
        "già è",
        r"x\very\deep\path\to\the\kit",
    ] {
        let exe = PathBuf::from(r"E:\").join(name).join("Atollo.exe");
        let kit = Kit::from_exe(&exe).unwrap();
        assert_eq!(kit.root(), exe.parent().unwrap());
        assert_eq!(kit.data(), exe.parent().unwrap().join("data"));
    }
}

/// T1.1.1: the root next to `Atollo.app` or to the AppImage, otherwise
/// the folder of the program; Windows keeps the folder of the exe.
#[test]
fn root_next_to_the_bundle_or_the_appimage() {
    let p = |s: &str| PathBuf::from(s);
    let root =
        |os, exe: &str, image: Option<&str>| root_from(os, Path::new(exe), image.map(Path::new));
    // macOS: the folder that holds the bundle, any case of `.app`.
    assert_eq!(
        root(
            Os::Darwin,
            "/Users/u/Kit/Atollo.app/Contents/MacOS/Atollo",
            None
        ),
        Some(p("/Users/u/Kit"))
    );
    assert_eq!(
        root(
            Os::Darwin,
            "/Volumes/KIT/A b.APP/Contents/MacOS/Atollo",
            None
        ),
        Some(p("/Volumes/KIT"))
    );
    // Not a bundle layout: the folder of the program.
    for exe in [
        "/Users/u/Kit/Atollo",
        "/Users/u/Kit/Atollo.app/Contents/Atollo",
        "/Users/u/Kit/Atollo/Contents/MacOS/Atollo",
    ] {
        assert_eq!(
            root(Os::Darwin, exe, None),
            Path::new(exe).parent().map(Path::to_path_buf),
            "{exe}"
        );
    }
    // Linux: the folder of the AppImage, not of its temporary mount.
    assert_eq!(
        root(
            Os::Linux,
            "/tmp/.mount_Atollox1/usr/bin/Atollo",
            Some("/media/u/KIT/Atollo.AppImage")
        ),
        Some(p("/media/u/KIT"))
    );
    assert_eq!(
        root(Os::Linux, "/media/u/KIT/Atollo", None),
        Some(p("/media/u/KIT"))
    );
    // A relative $APPIMAGE is not used.
    assert_eq!(
        root(Os::Linux, "/media/u/KIT/Atollo", Some("Atollo.AppImage")),
        Some(p("/media/u/KIT"))
    );
    // The bundle rule is macOS only, the AppImage Linux only.
    assert_eq!(
        root(Os::Linux, "/k/X.app/Contents/MacOS/Atollo", None),
        Some(p("/k/X.app/Contents/MacOS"))
    );
    assert_eq!(
        root(Os::Win32, "/k/Atollo.exe", Some("/media/u/x.AppImage")),
        Some(p("/k"))
    );
    #[cfg(windows)]
    assert_eq!(
        root(Os::Win32, r"E:\Kit\Atollo.exe", Some("/media/u/x.AppImage")),
        Some(p(r"E:\Kit"))
    );
    assert_eq!(root(Os::Win32, "Atollo.exe", None), None);
}

#[test]
fn appimage_only_from_inside_its_mount() {
    let os = |s: &str| Some(std::ffi::OsString::from(s));
    let exe = Path::new("/tmp/.mount_ab/usr/bin/Atollo");
    let (image, dir) = (os("/media/u/K/Atollo.AppImage"), os("/tmp/.mount_ab"));
    assert_eq!(
        appimage_of(exe, image.as_deref(), dir.as_deref()),
        Some(PathBuf::from("/media/u/K/Atollo.AppImage"))
    );
    // A plain binary started from inside another AppImage: not its image.
    let plain = Path::new("/media/u/K/Atollo");
    assert_eq!(appimage_of(plain, image.as_deref(), dir.as_deref()), None);
    assert_eq!(appimage_of(exe, image.as_deref(), None), None);
    assert_eq!(appimage_of(exe, os("").as_deref(), dir.as_deref()), None);
}

/// T1.1.2: a translocated bundle stops the start with the command to run.
#[test]
fn app_translocation_is_refused_on_macos() {
    let moved = Path::new(
        "/private/var/folders/xy/abc/T/AppTranslocation/0A1B-2C/d/Atollo.app/Contents/MacOS/Atollo",
    );
    assert_eq!(translocated_bundle(moved).as_deref(), Some("Atollo.app"));
    let text = launch_problem(Os::Darwin, moved, "it").unwrap();
    assert!(
        text.contains("xattr -dr com.apple.quarantine 'Atollo.app'"),
        "{text}"
    );
    // Always quoted, so that a crafted name cannot run anything.
    let crafted = Path::new(
        "/private/var/folders/xy/abc/T/AppTranslocation/0A1B-2C/d/$(id)'.app/Contents/MacOS/Atollo",
    );
    let text = launch_problem(Os::Darwin, crafted, "it").unwrap();
    assert!(text.contains(r"quarantine '$(id)'\''.app'"), "{text}");
    let plain = Path::new(
        "/private/var/folders/xy/abc/T/AppTranslocation/0A1B-2C/d/Kit.app/Contents/MacOS/Atollo",
    );
    let text = launch_problem(Os::Darwin, plain, "it").unwrap();
    assert!(text.contains("quarantine 'Kit.app'"), "{text}");
    let en = launch_problem(Os::Darwin, moved, "en").unwrap();
    assert!(en.contains("App Translocation") && en != text, "{en}");
    let normal = Path::new("/Users/u/Kit/Atollo.app/Contents/MacOS/Atollo");
    assert_eq!(translocated_bundle(normal), None);
    assert_eq!(launch_problem(Os::Darwin, normal, "it"), None);
    assert_eq!(launch_problem(Os::Linux, moved, "it"), None);
    assert_eq!(launch_problem(Os::Win32, moved, "it"), None);
}

/// T1.1.3: a read-only disk is named off Windows; Windows keeps its text.
#[test]
fn read_only_root_message() {
    let err = |kind| KitError::NotWritable {
        path: PathBuf::from("/media/u/KIT/projects"),
        source: io::Error::from(kind),
    };
    let ro = prepare_error_message(Os::Linux, &err(io::ErrorKind::ReadOnlyFilesystem), "it");
    assert!(
        ro.contains("sola lettura") && ro.contains("/media/u/KIT/projects"),
        "{ro}"
    );
    let denied = prepare_error_message(Os::Darwin, &err(io::ErrorKind::PermissionDenied), "it");
    assert!(
        !denied.contains("sola lettura") && !denied.contains("Atollo.exe"),
        "{denied}"
    );
    // Windows: the message of before, whatever the error.
    for kind in [
        io::ErrorKind::ReadOnlyFilesystem,
        io::ErrorKind::PermissionDenied,
    ] {
        assert_eq!(
            prepare_error_message(Os::Win32, &err(kind), "it"),
            "Impossibile scrivere nella cartella /media/u/KIT/projects.\n\nSposta Atollo.exe in una cartella dove puoi scrivere (per esempio una chiavetta o una cartella del tuo profilo) e riavvialo."
        );
    }
    let en = prepare_error_message(Os::Linux, &err(io::ErrorKind::ReadOnlyFilesystem), "en");
    assert_ne!(en, ro);
}

/// Without an AppImage the wrappers run this very program.
#[test]
fn app_exe_is_this_program() {
    assert_eq!(app_exe(), std::env::current_exe().ok());
}

#[test]
fn verbatim_prefix_is_removed() {
    let kit = Kit::from_exe(Path::new(r"\\?\D:\Kit\Atollo.exe")).unwrap();
    assert_eq!(kit.root(), Path::new(r"D:\Kit"));
}

#[test]
fn apps_folder_per_platform() {
    let kit = Kit::at(PathBuf::from(r"F:\K"), Arch::Arm64);
    assert_eq!(kit.apps(), PathBuf::from(r"F:\K\data\apps\win32-arm64"));

    for (target, slug, triple, conda, tl_bin) in [
        (
            Target::WIN32_X64,
            "win32-x64",
            "x86_64-pc-windows-msvc",
            "win-64",
            "windows",
        ),
        (
            Target::WIN32_ARM64,
            "win32-arm64",
            "aarch64-pc-windows-msvc",
            "win-arm64",
            "windows",
        ),
        (
            Target::DARWIN_ARM64,
            "darwin-arm64",
            "aarch64-apple-darwin",
            "osx-arm64",
            "universal-darwin",
        ),
        (
            Target::DARWIN_X64,
            "darwin-x64",
            "x86_64-apple-darwin",
            "osx-64",
            "universal-darwin",
        ),
        (
            Target::LINUX_X64,
            "linux-x64",
            "x86_64-unknown-linux-gnu",
            "linux-64",
            "x86_64-linux",
        ),
        (
            Target::LINUX_ARM64,
            "linux-arm64",
            "aarch64-unknown-linux-gnu",
            "linux-aarch64",
            "aarch64-linux",
        ),
    ] {
        let k = Kit::at_target(PathBuf::from(r"F:\K"), target);
        assert_eq!(k.target(), target);
        assert_eq!(Target::parse(slug), Some(target));
        assert_eq!(target.as_str(), slug);
        assert_eq!(target.rust_triple(), triple);
        assert_eq!(target.conda_subdir(), conda);
        assert_eq!(target.texlive_bin(), tl_bin);
        assert_eq!(k.apps(), PathBuf::from(r"F:\K\data\apps").join(slug));
        assert_eq!(
            k.versions_dir(),
            PathBuf::from(r"F:\K\data\app\versions").join(slug)
        );
    }
}

#[test]
fn id_ignores_case() {
    let a = Kit::at(PathBuf::from(r"E:\Kit"), Arch::X64);
    let b = Kit::at(PathBuf::from(r"e:\kit"), Arch::X64);
    assert_eq!(a.id(), b.id());
    assert_eq!(a.id().len(), 16);
}

#[test]
fn host_temp_and_disk() {
    let tmp = TempDir::new("with-host");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    assert_eq!(kit.system_temp(), None);
    assert_eq!(kit.project_data(), tmp.0.join(r"data\projects"));
    let host = kit
        .clone()
        .with_host(Some(Path::new(r"C:\Users\u\AppData\Local\Temp")));
    assert_eq!(
        host.system_temp().unwrap(),
        Path::new(r"C:\Users\u\AppData\Local\Temp\Atollo").join(kit.id())
    );
    assert!(host.volume().is_some_and(|v| v.free > 0));
    // started from the kit's own programs: never nested, never inside the kit
    let nested = host.system_temp().unwrap().join("system");
    assert_eq!(
        kit.clone().with_host(Some(&nested)).system_temp(),
        host.system_temp()
    );
    let inside = tmp.0.join(r"data\tmp\system");
    assert_eq!(kit.clone().with_host(Some(&inside)).system_temp(), None);
    // a relative TEMP is not used: everything stays in data\tmp
    assert_eq!(kit.with_host(Some(Path::new("Temp"))).system_temp(), None);
}

#[test]
fn prepare_creates_only_inside_root_and_detects_move() {
    let tmp = TempDir::new("prepare");
    let first = Kit::at(tmp.0.join("one (1) & co"), Arch::X64);
    assert_eq!(first.prepare().unwrap(), None);
    assert!(first.projects().is_dir() && first.logs().is_dir());
    assert_eq!(first.prepare().unwrap(), None, "same root: not moved");

    // Same data folder seen from another root (drive letter change).
    let moved_root = tmp.0.join("two");
    fs::rename(first.root(), &moved_root).unwrap();
    let moved = Kit::at(moved_root, Arch::X64);
    assert_eq!(moved.prepare().unwrap(), Some(first.root().to_path_buf()));
    let pending = fs::read_to_string(moved.app_dir().join("old-root.txt")).unwrap();
    assert_eq!(Path::new(&pending), first.root());
    assert_eq!(moved.prepare().unwrap(), None, "recorded: not moved again");

    let entries: Vec<_> = fs::read_dir(&tmp.0).unwrap().collect();
    assert_eq!(entries.len(), 1, "nothing created next to the kit");
}

/// D-92: a conda environment (Git, R off Windows) has its folder written
/// inside, so a moved kit marks it to be installed again; on Windows every
/// component moves with the kit.
#[test]
fn moving_the_kit_flags_the_conda_components_off_windows() {
    let catalog = crate::setup::catalog::Catalog::get();
    let flagged = |kit: &Kit| -> Vec<&str> {
        ["git", "r", "python", "node"]
            .into_iter()
            .filter(|id| catalog.find(id).unwrap().needs_reinstall(kit))
            .collect()
    };
    for (target, expected) in [
        (Target::LINUX_X64, vec!["git", "r"]),
        (Target::DARWIN_ARM64, vec!["git", "r"]),
        (Target::WIN32_X64, vec![]),
    ] {
        let tmp = TempDir::new(&format!("moved-{target}"));
        let first = Kit::at_target(tmp.0.join("one"), target);
        assert_eq!(first.prepare().unwrap(), None);
        // Git, R and Python on disk; Node is not.
        for rel in ["git/2.51/bin/git", "r/4.5/bin/R", "python/3.13/bin/python3"] {
            let file = crate::fsx::join_rel(first.apps(), rel);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, b"x").unwrap();
        }
        assert_eq!(first.prepare().unwrap(), None);
        assert!(flagged(&first).is_empty(), "{target}: not moved yet");

        let moved_root = tmp.0.join("two");
        fs::rename(first.root(), &moved_root).unwrap();
        let moved = Kit::at_target(moved_root, target);
        assert_eq!(moved.prepare().unwrap(), Some(first.root().to_path_buf()));
        assert_eq!(flagged(&moved), expected, "{target}");

        // Reinstalled (install::install clears it): not flagged again at
        // the next start from the same root.
        for id in &expected {
            catalog.find(id).unwrap().clear_reinstall(&moved);
        }
        assert_eq!(moved.prepare().unwrap(), None);
        assert!(flagged(&moved).is_empty(), "{target}: same root");
    }
}

/// The old root that `check_moved` saves is the file `agents::relocate`
/// reads, on every OS (macOS and Linux used to save `old-root-<os>.txt`
/// while relocate read only `old-root.txt`).
#[test]
fn old_root_file_is_the_one_check_moved_writes() {
    let root = std::env::temp_dir().join("atollo-old-root");
    let names = [
        (Target::WIN32_X64, "old-root.txt"),
        (Target::DARWIN_ARM64, "old-root-darwin.txt"),
        (Target::LINUX_X64, "old-root-linux.txt"),
    ];
    for (target, name) in names {
        let kit = Kit::at_target(root.clone(), target);
        assert_eq!(kit.old_root_file(), kit.app_dir().join(name));
        assert_eq!(kit.root_files().1, kit.old_root_file());
    }
}

/// F7a: off Windows a root that changed only in case is another folder
/// (Linux, and macOS volumes that respect case): the kit moved. On Windows
/// it is the same folder.
#[test]
fn case_only_move_is_a_move_off_windows() {
    assert!(same_root(Os::Win32, r"E:\Kit", r"e:\kit"));
    assert!(!same_root(Os::Linux, "/media/u/Kit", "/media/u/kit"));
    assert!(!same_root(Os::Darwin, "/Volumes/U/Kit", "/Volumes/U/kit"));
    assert!(same_root(Os::Linux, "/media/u/kit", "/media/u/kit"));

    let catalog = crate::setup::catalog::Catalog::get();
    for (target, moves) in [(Target::LINUX_X64, true), (Target::WIN32_X64, false)] {
        let tmp = TempDir::new(&format!("case-{target}"));
        let first = Kit::at_target(tmp.0.join("Kit"), target);
        assert_eq!(first.prepare().unwrap(), None);
        let file = crate::fsx::join_rel(first.apps(), "git/2.51/bin/git");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, b"x").unwrap();
        // Renamed in two steps: also where the file system ignores case.
        fs::rename(first.root(), tmp.0.join("tmp")).unwrap();
        fs::rename(tmp.0.join("tmp"), tmp.0.join("kit")).unwrap();
        let moved = Kit::at_target(tmp.0.join("kit"), target);
        let expected = moves.then(|| first.root().to_path_buf());
        assert_eq!(moved.prepare().unwrap(), expected, "{target}");
        let git = catalog.find("git").unwrap();
        assert_eq!(git.needs_reinstall(&moved), moves, "{target}");
    }
}

/// M3/F7b: a saved line of R (another conda environment next to the
/// default one) is marked too, and the start reinstalls it at its line.
#[test]
fn saved_r_line_is_marked_when_the_kit_moves() {
    let catalog = crate::setup::catalog::Catalog::get();
    let r = catalog.find("r").unwrap();
    let tmp = TempDir::new("moved-r-line");
    let first = Kit::at_target(tmp.0.join("one"), Target::LINUX_X64);
    assert_eq!(first.prepare().unwrap(), None);
    for rel in ["r/4.5/bin/R", "r/4.4/bin/R"] {
        let file = crate::fsx::join_rel(first.apps(), rel);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, b"x").unwrap();
    }
    r.save_version(&first, "4.5.1").unwrap();
    let saved: Vec<String> = crate::versions::saved(&first, r)
        .into_iter()
        .map(|l| l.line)
        .collect();
    assert_eq!(saved, ["4.4"]);
    assert!(r.lines_to_reinstall(&first).is_empty());

    let moved_root = tmp.0.join("two");
    fs::rename(first.root(), &moved_root).unwrap();
    let moved = Kit::at_target(moved_root, Target::LINUX_X64);
    assert_eq!(moved.prepare().unwrap(), Some(first.root().to_path_buf()));
    assert!(r.needs_reinstall(&moved));
    assert_eq!(r.lines_to_reinstall(&moved), ["4.4"]);
    assert!(crate::setup::task::outdated_app(&moved).contains(&"r".to_owned()));

    // The default reinstalled: the saved line still keeps R in the start.
    r.clear_reinstall(&moved);
    assert!(crate::setup::task::outdated_app(&moved).contains(&"r".to_owned()));
    // The line reinstalled (install::install_line clears it): nothing left.
    r.clear_line_reinstall(&moved, "4.4");
    assert!(!crate::setup::task::outdated_app(&moved).contains(&"r".to_owned()));

    // A marked line removed from the Tools page loses its mark.
    r.mark_line_reinstall(&moved, "4.4", "/old").unwrap();
    crate::versions::remove(&moved, "r", "4.4").unwrap();
    assert!(r.lines_to_reinstall(&moved).is_empty());
    assert!(!moved.versions_dir().join("r@4.4.reinstall").exists());

    // A marked line that became the default marks the component.
    let file = crate::fsx::join_rel(moved.apps(), "r/4.4/bin/R");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, b"x").unwrap();
    r.mark_line_reinstall(&moved, "4.4", "/old").unwrap();
    r.save_version(&moved, "4.4.3").unwrap();
    assert!(!r.needs_reinstall(&moved));
    // 4.5 is a saved line now, not marked; 4.4's mark moves to R.
    assert!(r.lines_to_reinstall(&moved).is_empty());
    assert!(r.needs_reinstall(&moved));
    assert!(!moved.versions_dir().join("r@4.4.reinstall").exists());

    // Removing R forgets every mark.
    r.mark_line_reinstall(&moved, "4.5", "/old").unwrap();
    r.forget_version(&moved);
    assert!(!r.needs_reinstall(&moved));
    assert!(!moved.versions_dir().join("r@4.5.reinstall").exists());
}

#[test]
fn folders_that_break_the_editor_watcher() {
    assert_eq!(watcher_unsafe_folder(Path::new(r"E:\Atollo")), None);
    assert_eq!(
        watcher_unsafe_folder(Path::new(r"E:\Atollo (1) & co")),
        None
    );
    assert_eq!(
        watcher_unsafe_folder(Path::new(r"C:\Users\Mario\code\kit")),
        None
    );
    assert_eq!(
        watcher_unsafe_folder(Path::new(r"D:\2026\kit")).as_deref(),
        Some("2026")
    );
    assert_eq!(
        watcher_unsafe_folder(Path::new(r"D:\x-kit")).as_deref(),
        Some("x-kit")
    );
    assert_eq!(
        watcher_unsafe_folder(Path::new(r"C:\users\a")).as_deref(),
        Some("users")
    );
}

#[test]
fn project_names() {
    for ok in [
        "app",
        "my app",
        "Progetto è",
        "a.b",
        "con1",
        "comx",
        "x".repeat(100).as_str(),
    ] {
        assert_eq!(validate_project_name(ok), Ok(()), "{ok}");
    }
    assert_eq!(validate_project_name(""), Err(NameError::Empty));
    assert_eq!(
        validate_project_name(&"x".repeat(101)),
        Err(NameError::TooLong)
    );
    for bad in [
        "a/b", r"a\b", "a:b", "a*", "a?", "a\"b", "<a>", "a|b", "a\u{7}",
    ] {
        assert_eq!(
            validate_project_name(bad),
            Err(NameError::ForbiddenChar),
            "{bad}"
        );
    }
    for bad in [".git", "a.", "a "] {
        assert_eq!(validate_project_name(bad), Err(NameError::BadEdge), "{bad}");
    }
    for bad in ["CON", "nul", "Aux.txt", "COM1", "lpt9.log"] {
        assert_eq!(
            validate_project_name(bad),
            Err(NameError::Reserved),
            "{bad}"
        );
    }
}
