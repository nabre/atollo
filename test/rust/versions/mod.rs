use super::*;

#[test]
fn lines() {
    for (v, l) in [
        ("v22.11.0", Some("22.11")),
        ("3.13.10", Some("3.13")),
        ("8.4.13 (Composer 2.8.12)", Some("8.4")),
        ("21.0.5+11 (Maven 3.9.11, Gradle 9.1.0)", Some("21.0")),
        ("1.25", Some("1.25")),
        ("0.15.1", Some("0.15")),
        ("go1.25.3", Some("1.25")),
        ("22", None),
        ("", None),
    ] {
        assert_eq!(line_of(v).as_deref(), l, "{v}");
    }
    assert!(valid_line("22.11") && !valid_line("22.11.0") && !valid_line("x"));
    assert!(!valid_line(r"..\x") && !valid_line("1.2;3"));
}

#[test]
#[cfg_attr(
    not(windows),
    ignore = "install_fake writes the Windows layout; backslash paths"
)]
fn tools_have_a_folder_per_line() {
    let lined: Vec<&str> = lined().map(|c| c.id.as_str()).collect();
    for id in [
        "claude",
        "codex",
        "antigravity",
        "git",
        "gh",
        "uv",
        "pwsh",
        "cmake",
        "phpmyadmin",
        "sqlite",
        "node",
        "apache",
    ] {
        assert!(lined.contains(&id), "{id}");
    }
    // Packages added later live in their folders (rustup, sdkmanager,
    // tlmgr, npm); llvm-mingw has dates; the editor is the app's.
    for id in [
        "rust",
        "android",
        "tinytex",
        "pnpm",
        "cc",
        "vscodium-server",
    ] {
        assert!(!lined.contains(&id), "{id}");
    }

    let tmp = crate::kit::tests::TempDir::new("versions-lined");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64);
    let git = Catalog::get().find("git").unwrap();
    let dir = install_dir(&kit, git, "2.56.0.2");
    assert!(dir.ends_with(r"git\2.56"));
    install_fake(&kit, git, &dir);
    git.save_version(&kit, "2.56.0.2").unwrap();
    assert_eq!(
        crate::setup::catalog::kit_path(&kit, r"git\cmd\git.exe"),
        dir.join(r"cmd\git.exe")
    );
    assert!(saved(&kit, git).is_empty());
}

pub(super) fn install_fake(kit: &Kit, c: &Component, dir: &Path) {
    let rel = c.key.strip_prefix(c.folder.as_deref().unwrap()).unwrap();
    let key = dir.join(rel.trim_start_matches('/').replace('/', "\\"));
    fs::create_dir_all(key.parent().unwrap()).unwrap();
    fs::write(key, b"").unwrap();
    let _ = kit;
}
