use super::*;

/// A fresh folder under the system temp, removed on drop.
pub struct TempDir(pub PathBuf);
impl TempDir {
    pub fn new(name: &str) -> TempDir {
        let dir = std::env::temp_dir().join(format!(
            "devkit-test-{}-{}-{name}",
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
        "Dev Kit",
        "kit (2)",
        "a&b",
        "percent%20",
        "già è",
        r"x\very\deep\path\to\the\kit",
    ] {
        let exe = PathBuf::from(r"E:\").join(name).join("Dev-Kit.exe");
        let kit = Kit::from_exe(&exe).unwrap();
        assert_eq!(kit.root(), exe.parent().unwrap());
        assert_eq!(kit.data(), exe.parent().unwrap().join("data"));
    }
}

#[test]
fn verbatim_prefix_is_removed() {
    let kit = Kit::from_exe(Path::new(r"\\?\D:\Kit\Dev-Kit.exe")).unwrap();
    assert_eq!(kit.root(), Path::new(r"D:\Kit"));
}

#[test]
fn apps_folder_per_platform() {
    let kit = Kit::at(PathBuf::from(r"F:\K"), Arch::Arm64);
    assert_eq!(kit.apps(), PathBuf::from(r"F:\K\data\apps\win32-arm64"));
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
        Path::new(r"C:\Users\u\AppData\Local\Temp\Dev-Kit").join(kit.id())
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

#[test]
fn folders_that_break_the_editor_watcher() {
    assert_eq!(watcher_unsafe_folder(Path::new(r"E:\Dev-Kit")), None);
    assert_eq!(
        watcher_unsafe_folder(Path::new(r"E:\Dev Kit (1) & co")),
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
