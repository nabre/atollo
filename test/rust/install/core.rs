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
