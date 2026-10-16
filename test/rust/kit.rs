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
