use super::*;

use crate::kit::tests::TempDir;

fn tree(dir: &Path) {
    fs::create_dir_all(dir.join("bin")).unwrap();
    fs::create_dir_all(dir.join("lib").join("site-packages").join("x")).unwrap();
    fs::create_dir_all(dir.join("lib").join("__pycache__")).unwrap();
    fs::write(dir.join("bin").join("tool.exe"), "program").unwrap();
    fs::write(dir.join("bin").join("my lib.dll"), "library").unwrap();
    fs::write(dir.join("README"), "read me").unwrap();
    fs::write(
        dir.join("lib").join("site-packages").join("x").join("a.py"),
        "a",
    )
    .unwrap();
    fs::write(dir.join("lib").join("__pycache__").join("m.pyc"), "c").unwrap();
    fs::write(dir.join(".atollo-version"), "1.2.3").unwrap();
}

/// Missing or different files are damage; added files and those changed
/// on purpose are not.
#[test]
fn a_damaged_folder_is_found() {
    let tmp = TempDir::new("integrity");
    let dir = tmp.0.join("tool").join("1.2");
    tree(&dir);
    assert!(verify(&tmp.0, &dir).is_none(), "no manifest yet");
    let tool = tmp.0.join("tools").join("tool.exe");
    fs::create_dir_all(tool.parent().unwrap()).unwrap();
    fs::write(&tool, "shared").unwrap();
    write(
        &tmp.0,
        &dir,
        &[tool.clone(), tmp.0.join("tools").join("absent.exe")],
    )
    .unwrap();
    let text = fs::read_to_string(dir.join(MANIFEST)).unwrap();
    assert!(text.starts_with("# Atollo manifest 1"));
    assert!(text.contains(" 7 bin/my lib.dll\n"), "{text}");
    assert!(
        !text.contains("site-packages")
            && !text.contains(".pyc")
            && !text.contains(".atollo-version")
    );
    // A part in a shared folder, from the kit's root; a missing one is left
    // out.
    assert!(text.contains(" 6 @tools/tool.exe\n"), "{text}");
    assert!(!text.contains("absent"));
    let ok = verify(&tmp.0, &dir).unwrap().unwrap();
    assert_eq!(ok.checked, 4);
    assert!(!ok.is_damaged());
    fs::write(&tool, "shaRed").unwrap();
    assert_eq!(
        verify(&tmp.0, &dir).unwrap().unwrap().changed,
        ["@tools/tool.exe"]
    );
    fs::write(&tool, "shared").unwrap();

    // What the programs do: packages, compiled Python, new files.
    fs::write(
        dir.join("lib").join("site-packages").join("x").join("a.py"),
        "b",
    )
    .unwrap();
    fs::write(dir.join("lib").join("__pycache__").join("m.pyc"), "d").unwrap();
    fs::write(dir.join("bin").join("new.txt"), "n").unwrap();
    assert!(!verify(&tmp.0, &dir).unwrap().unwrap().is_damaged());

    // A byte changed with the same size, a file gone.
    fs::write(dir.join("bin").join("tool.exe"), "progrem").unwrap();
    fs::remove_file(dir.join("README")).unwrap();
    let bad = verify(&tmp.0, &dir).unwrap().unwrap();
    assert_eq!(bad.changed, ["bin/tool.exe"]);
    assert_eq!(bad.missing, ["README"]);

    // A damaged manifest is an error, not "all well".
    fs::write(dir.join(MANIFEST), "# x\nnot a line\n").unwrap();
    assert!(verify(&tmp.0, &dir).unwrap().is_err());
}

#[test]
fn files_changed_on_purpose() {
    for rel in [
        "conf/httpd.conf",
        "config.inc.php",
        "lib/R/etc/x64/Makeconf",
        "etc/gitconfig",
        "composer.phar",
        "Lib/site-packages/pip/__init__.py",
        "Scripts/pip.exe",
        "lib/ruby/gems/3.3.0/x.rb",
        "a/__pycache__/b.cpython-313.pyc",
        "claude.exe.old",
        "claude.exe.old-1712",
        "logs/error.log",
        ".atollo-version",
        MANIFEST,
        "._x",
    ] {
        assert!(changes_on_purpose(rel), "{rel}");
    }
    for rel in [
        "bin/php.exe",
        "lib/libssl.so.3",
        "httpd.conf.bak",
        "share/gems.txt",
    ] {
        assert!(!changes_on_purpose(rel), "{rel}");
    }
}

/// Repair marks the default version as the component, a saved one as its
/// line.
#[test]
fn repair_marks_the_folder() {
    let tmp = TempDir::new("integrity-kit");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64);
    kit.prepare().unwrap();
    let python = Catalog::get().find("python").unwrap();
    python.save_version(&kit, "3.13.1").unwrap();
    let ids = mark_for_repair(
        &kit,
        &[
            // A component without a folder per version, or its default.
            ("python".into(), None),
            ("python".into(), Some("3.12".into())),
        ],
    )
    .unwrap();
    assert_eq!(ids, ["python"]);
    assert!(python.needs_reinstall(&kit));
    assert!(kit.versions_dir().join("python@3.12.reinstall").is_file());
    assert!(mark_for_repair(&kit, &[("nothing".into(), None)]).is_err());
}
