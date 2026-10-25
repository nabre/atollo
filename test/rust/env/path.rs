use super::*;
use crate::env::tests::*;
use crate::env::*;
use crate::kit::tests::TempDir;

#[test]
fn path_is_kit_then_system_only() {
    let windows = cfg!(windows);
    let (_tmp, kit) = fake_kit(
        "env-path",
        &[
            "git/bin/bash.exe",
            "node/node.exe",
            "python/python.exe",
            "pwsh/pwsh.exe",
            "go/bin/go.exe",
        ],
    );
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    let entries = env.path_entries();
    let system: Vec<String> = if windows {
        vec![
            r"C:\Windows\System32".into(),
            r"C:\Windows\System32\Wbem".into(),
            r"C:\Windows\System32\WindowsPowerShell\v1.0".into(),
        ]
    } else {
        system_path(kit.os())
            .iter()
            .map(|s| s.to_string())
            .collect()
    };
    let n = entries.len() - system.len();
    assert_eq!(&entries[n..], system);
    for entry in &entries[..n] {
        assert!(Path::new(entry).starts_with(kit.data()), "{entry}");
    }
    let apps = kit.apps();
    let pos = |p: PathBuf| {
        entries
            .iter()
            .position(|e| Path::new(e) == p)
            .unwrap_or_else(|| panic!("{} missing", p.display()))
    };
    // Node and Python have their programs in `bin` off Windows; Git is
    // `cmd` on Windows, `bin` elsewhere.
    let bin = |d: &str| {
        if windows {
            apps.join(d)
        } else {
            apps.join(d).join("bin")
        }
    };
    let git = apps.join("git").join(if windows { "cmd" } else { "bin" });
    assert!(pos(apps.join("claude")) < pos(bin("python")));
    assert!(pos(bin("node")) < pos(apps.join("go").join("bin")));
    assert!(pos(apps.join("go").join("bin")) < pos(git));
    assert!(
        entries
            .iter()
            .all(|e| !Path::new(e).starts_with(apps.join("java"))),
        "java not installed"
    );
    let bash = Path::new("git").join("bin").join("bash.exe");
    assert_eq!(
        env.get("CLAUDE_CODE_GIT_BASH_PATH")
            .map(|p| Path::new(p).ends_with(&bash)),
        windows.then_some(true)
    );
    assert!(env.get("JAVA_HOME").is_none());
}

/// Backlog (a): on macOS and Linux `python` runs the kit's `python3`,
/// through a script in a folder that only the Unix PATH has, right after
/// the kit's Python; Windows has no such folder.
#[test]
fn python_on_the_unix_path() {
    use crate::kit::Target;

    let tmp = TempDir::new("env-python-shim");
    let kit = Kit::at_target(tmp.0.join("Kit"), Target::LINUX_X64);
    let python = crate::common::fsx::join_rel(kit.apps(), "python/bin/python3");
    fs::create_dir_all(python.parent().unwrap()).unwrap();
    fs::write(&python, b"").unwrap();
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    let shims = kit.kit_files().join("shims");
    let entries = env.path_entries();
    let at = |p: &Path| entries.iter().position(|e| Path::new(e) == p);
    let bin = crate::common::fsx::join_rel(kit.apps(), "python/bin");
    assert!(at(&bin).is_some(), "{entries:?}");
    assert_eq!(at(&shims), at(&bin).map(|i| i + 1), "{entries:?}");
    env.create_dirs().unwrap();
    let script = fs::read_to_string(shims.join("python")).unwrap();
    assert_eq!(script, PYTHON_SHIM);
    assert!(script.starts_with("#!/bin/sh\n") && !script.contains('\r'));
    assert!(script.contains("exec python3 \"$@\""));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(shims.join("python"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o755);
    }
    // Unchanged: left as it is.
    env.create_dirs().unwrap();
    // The same script without its exec bits (a copy that drops modes):
    // they come back.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file = shims.join("python");
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
        env.create_dirs().unwrap();
        let mode = fs::metadata(&file).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o755);
    }

    // Without the kit's Python: no shim on the PATH.
    let bare = Kit::at_target(tmp.0.join("Bare"), Target::LINUX_X64);
    let env = KitEnv::build(&bare, &Settings::default(), "en", &host());
    let bare_shims = bare.kit_files().join("shims");
    assert!(!env
        .path_entries()
        .iter()
        .any(|e| Path::new(e) == bare_shims));

    // Windows: python.exe itself, never the shim folder.
    let (_tmp, win) = fake_kit("env-python-win", &["python/python.exe"]);
    let win = Kit::at_target(win.root().to_path_buf(), Target::WIN32_X64);
    let env = KitEnv::build(&win, &Settings::default(), "en", &host());
    let win_shims = win.kit_files().join("shims");
    assert!(!env.path_entries().iter().any(|e| Path::new(e) == win_shims));
}

/// D-125: a conda environment puts its `kit-bin` on the PATH, the
/// launchers of the component's programs, never its `bin` with those of
/// its dependencies; `bin` while it has no `kit-bin` (installed before).
#[test]
fn conda_environments_give_only_their_programs() {
    use crate::kit::Target;

    let tmp = TempDir::new("env-conda-launchers");
    let kit = Kit::at_target(tmp.0.join("Kit"), Target::LINUX_X64);
    touch(
        &kit,
        &[
            "ruby/4.0/bin/ruby",
            "ruby/4.0/kit-bin/ruby",
            "mongodb/8.0/bin/mongod",
            "mongodb/8.0/kit-bin/mongod",
            "r/4.6/bin/R",
        ],
    );
    let catalog = crate::setup::Catalog::get();
    for (id, version) in [("ruby", "4.0.7"), ("mongodb", "8.0.23"), ("r", "4.6.1")] {
        catalog
            .find(id)
            .unwrap()
            .save_version(&kit, version)
            .unwrap();
    }
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    let entries = env.path_entries();
    let has = |rel: &str| {
        let p = crate::common::fsx::join_rel(kit.apps(), rel);
        entries.iter().any(|e| Path::new(e) == p)
    };
    assert!(
        has("ruby/4.0/kit-bin") && !has("ruby/4.0/bin"),
        "{entries:?}"
    );
    assert!(
        has("mongodb/8.0/kit-bin") && !has("mongodb/8.0/bin"),
        "{entries:?}"
    );
    assert!(has("r/4.6/bin"), "R without launchers yet: {entries:?}");
}
