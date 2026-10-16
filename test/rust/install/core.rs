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
