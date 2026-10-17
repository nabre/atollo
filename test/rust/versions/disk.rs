use super::*;
use crate::kit::{tests::TempDir, Arch};
use crate::versions::tests::install_fake;
use crate::versions::{dir_name, VERSION_FILE};

#[test]
#[cfg_attr(not(windows), ignore = "install_fake writes the Windows layout")]
fn deleted_by_hand_is_no_longer_chosen() {
    let tmp = TempDir::new("versions-sync");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let catalog = Catalog::get();
    let node = catalog.find("node").unwrap();
    let main = install_dir(&kit, node, "v22.11.0");
    install_fake(&kit, node, &main);
    node.save_version(&kit, "v22.11.0").unwrap();
    let mut choice = crate::setup::catalog::Choice::defaults(catalog);
    choice.set(catalog, "pnpm", true);
    choice.write(&kit, catalog).unwrap();
    assert!(sync_with_disk(&kit).unwrap().is_empty(), "all there");

    // The user deletes Node.js: it and pnpm (which needs it) go off.
    fs::remove_dir_all(node.folder_path(&kit).unwrap()).unwrap();
    let off = sync_with_disk(&kit).unwrap();
    assert!(off.contains(&"Node.js".to_owned()) && off.contains(&"pnpm, yarn".to_owned()));
    let now = crate::setup::catalog::Choice::read(&kit, catalog).unwrap();
    assert!(!now.is_chosen("node") && !now.is_chosen("pnpm"));
    assert!(node.installed_version(&kit).is_none());
    assert!(sync_with_disk(&kit).unwrap().is_empty(), "once");

    // Chosen and never installed (waiting for Apply): left alone.
    let mut choice = crate::setup::catalog::Choice::read(&kit, catalog).unwrap();
    choice.set(catalog, "go", true);
    choice.write(&kit, catalog).unwrap();
    assert!(sync_with_disk(&kit).unwrap().is_empty());
}

#[test]
#[cfg_attr(not(windows), ignore = "Windows layout: .exe keys and backslash paths")]
fn folders_of_an_older_kit_move() {
    let tmp = TempDir::new("versions-migrate");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let node = Catalog::get().find("node").unwrap();
    // Older kit: node\node.exe the default, versions\node\20.18 saved,
    // versions\typst\0.13 of a component no longer versioned.
    let store = node.folder_path(&kit).unwrap();
    install_fake(&kit, node, &store);
    node.save_version(&kit, "v22.11.0").unwrap();
    install_fake(&kit, node, &kit.apps().join(r"versions\node\20.18"));
    fs::create_dir_all(kit.apps().join(r"versions\typst\0.13")).unwrap();
    assert_eq!(default_dir(&kit, node), store, "works before moving");
    assert!(node.is_installed(&kit));
    let conf = kit.data().join(r"apache\kit.conf");
    fs::create_dir_all(conf.parent().unwrap()).unwrap();
    fs::write(&conf, "LoadFile \"${CP_APPS_FWD}/php/php8ts.dll\"\r\n").unwrap();

    migrate(&kit);
    let main = store.join("22.11");
    assert_eq!(default_dir(&kit, node), main);
    assert!(main.join("node.exe").is_file() && !store.join("node.exe").exists());
    assert_eq!(
        fs::read_to_string(main.join(VERSION_FILE)).unwrap(),
        "v22.11.0"
    );
    assert_eq!(saved(&kit, node).len(), 1);
    assert!(has_key(node, &store.join("20.18")));
    assert!(!kit.apps().join("versions").exists());
    // The PHP path follows the line, and a .php without a module is refused.
    let fixed = fs::read_to_string(&conf).unwrap();
    assert_eq!(
        fixed,
        crate::projects::apache::migrate_kit_conf("LoadFile \"${CP_PHP_FWD}/php8ts.dll\"\r\n")
    );
    assert!(fixed.starts_with("LoadFile \"${CP_PHP_FWD}/php8ts.dll\"\r\n<IfModule !php_module>"));
    // Nothing left to do the second time.
    migrate(&kit);
    assert_eq!(fs::read_to_string(&conf).unwrap(), fixed);
    assert_eq!(default_dir(&kit, node), main);
    // The default removed by hand: the newest line left takes over.
    fs::remove_dir_all(&main).unwrap();
    migrate(&kit);
    assert_eq!(default_dir(&kit, node), store.join("20.18"));
    assert!(node.is_installed(&kit));

    // The older kit's Composer, one in tools\: into each folder of PHP.
    let php = Catalog::get().find("php").unwrap();
    let version = "8.4.13 (Composer 2.8.12)";
    let php_dir = install_dir(&kit, php, version);
    install_fake(&kit, php, &php_dir);
    php.save_version(&kit, version).unwrap();
    let tools = kit.apps().join("tools");
    fs::create_dir_all(&tools).unwrap();
    fs::write(tools.join("composer.phar"), b"phar").unwrap();
    assert!(!php.is_installed(&kit), "no Composer in its folder");
    migrate(&kit);
    assert!(php_dir.join("composer.phar").is_file() && php_dir.join("composer.cmd").is_file());
    assert!(!tools.join("composer.phar").exists());
    assert!(php.is_installed(&kit));

    // Interrupted after the first rename: taken back, then moved.
    let python = Catalog::get().find("python").unwrap();
    let py = python.folder_path(&kit).unwrap();
    let mut moving = py.clone().into_os_string();
    moving.push(".devkit-move");
    install_fake(&kit, python, &PathBuf::from(&moving));
    python.save_version(&kit, "3.13.10").unwrap();
    migrate(&kit);
    assert!(has_key(python, &py.join("3.13")));
    assert!(!PathBuf::from(moving).exists());

    assert_eq!(dir_name("20251104 build"), "20251104_build");
}

#[test]
#[cfg_attr(not(windows), ignore = "install_fake writes the Windows layout")]
fn apache_folders_take_their_line() {
    let tmp = TempDir::new("versions-patch");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    let apache = Catalog::get().find("apache").unwrap();
    let store = apache.folder_path(&kit).unwrap();
    // One folder per patch, from the kit of before: the installed one and
    // an older build.
    for v in ["2.4.66-251206", "2.4.69-261002"] {
        let dir = store.join(&v[..6]);
        install_fake(&kit, apache, &dir);
        fs::write(dir.join(VERSION_FILE), v).unwrap();
    }
    apache.save_version(&kit, "2.4.69-261002").unwrap();
    migrate(&kit);
    assert!(has_key(apache, &store.join("2.4")));
    assert_eq!(
        fs::read_to_string(store.join("2.4").join(VERSION_FILE)).unwrap(),
        "2.4.69-261002"
    );
    assert!(!store.join("2.4.66").exists() && !store.join("2.4.69").exists());
    assert_eq!(default_line(&kit, apache).as_deref(), Some("2.4"));
    assert!(saved(&kit, apache).is_empty());
    assert_eq!(dir_name("2.4.70-261201"), "2.4");
}

#[test]
#[cfg_attr(not(windows), ignore = "install_fake writes the Windows layout")]
fn tools_keep_only_their_new_line() {
    let tmp = TempDir::new("versions-minor");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    let git = Catalog::get().find("git").unwrap();
    let store = git.folder_path(&kit).unwrap();
    // An older kit's `git\`, without lines: into `git\2.56`.
    install_fake(&kit, git, &store);
    git.save_version(&kit, "2.56.0.2").unwrap();
    migrate(&kit);
    assert!(has_key(git, &store.join("2.56")));

    // A new patch goes into the same folder, a new line next to it; the
    // old line goes.
    assert_eq!(install_dir(&kit, git, "2.56.1.1"), store.join("2.56"));
    let next = install_dir(&kit, git, "2.57.0.1");
    install_fake(&kit, git, &next);
    git.save_version(&kit, "2.57.0.1").unwrap();
    drop_old_lines(&kit, git);
    assert!(has_key(git, &next));
    assert!(!store.join("2.56").exists());

    // A versioned language keeps its lines.
    let node = Catalog::get().find("node").unwrap();
    for v in ["v22.11.0", "v24.1.0"] {
        install_fake(&kit, node, &install_dir(&kit, node, v));
        node.save_version(&kit, v).unwrap();
    }
    drop_old_lines(&kit, node);
    assert!(has_key(node, &install_dir(&kit, node, "v22.11.0")));
}

#[test]
#[cfg_attr(
    not(windows),
    ignore = "Windows layout: .exe programs and backslash paths"
)]
fn github_cli_and_uv_leave_tools() {
    let tmp = TempDir::new("versions-tools");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    let tools = kit.apps().join("tools");
    fs::create_dir_all(&tools).unwrap();
    for name in ["gh.exe", "uv.exe", "uvx.exe", "uvw.exe", "ninja.exe"] {
        fs::write(tools.join(name), b"").unwrap();
    }
    let catalog = Catalog::get();
    let (gh, uv) = (catalog.find("gh").unwrap(), catalog.find("uv").unwrap());
    gh.save_version(&kit, "2.102.0").unwrap();
    uv.save_version(&kit, "0.9.5").unwrap();
    assert!(!gh.is_installed(&kit));
    migrate(&kit);
    assert!(kit.apps().join(r"gh\2.102\bin\gh.exe").is_file());
    for name in ["uv.exe", "uvx.exe", "uvw.exe"] {
        assert!(kit.apps().join(r"uv\0.9").join(name).is_file(), "{name}");
        assert!(!tools.join(name).exists(), "{name}");
    }
    assert!(gh.is_installed(&kit) && uv.is_installed(&kit));
    // CMake's Ninja stays in tools.
    assert!(tools.join("ninja.exe").is_file() && !tools.join("gh.exe").exists());
}

/// Windows Terminal is no longer in the catalog (D-95): what an older kit
/// kept of it goes at start, nothing else.
#[test]
#[cfg_attr(not(windows), ignore = "folders created with backslash paths")]
fn retired_windows_terminal_goes() {
    let tmp = TempDir::new("versions-retired");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let wt = kit.apps().join(r"terminal\1.25\settings");
    fs::create_dir_all(&wt).unwrap();
    fs::write(wt.join("settings.json"), "{}").unwrap();
    fs::create_dir_all(kit.versions_dir()).unwrap();
    for f in [
        "terminal.txt",
        "terminal.reinstall",
        "terminal@1.24.reinstall",
        "pwsh.txt",
    ] {
        fs::write(kit.versions_dir().join(f), "1").unwrap();
    }
    let legacy = kit.app_dir().join(r"versions\terminal.txt");
    fs::write(&legacy, "1.25.2609.30003").unwrap();
    fs::create_dir_all(kit.kit_files()).unwrap();
    let script = kit.kit_files().join("console.ps1");
    fs::write(&script, "# pipe").unwrap();

    migrate(&kit);

    assert!(!kit.apps().join("terminal").exists());
    assert!(!legacy.exists() && !script.exists());
    let left: Vec<String> = fs::read_dir(kit.versions_dir())
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left, ["pwsh.txt"]);
}

/// Adminer is no longer in the catalog (phpMyAdmin, D-107): its folder,
/// version, router, sessions and log go at start.
#[test]
fn retired_adminer_goes() {
    let tmp = TempDir::new("versions-retired-adminer");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let line = kit.apps().join("adminer").join("6.1");
    fs::create_dir_all(&line).unwrap();
    fs::write(line.join("adminer.php"), "<?php").unwrap();
    fs::create_dir_all(kit.versions_dir()).unwrap();
    fs::write(kit.versions_dir().join("adminer.txt"), "6.1.2").unwrap();
    fs::create_dir_all(kit.kit_files()).unwrap();
    let router = kit.kit_files().join("adminer.php");
    fs::write(&router, "<?php").unwrap();
    let sessions = kit.app_dir().join("adminer");
    fs::create_dir_all(&sessions).unwrap();
    fs::write(sessions.join("sess_x"), "").unwrap();
    fs::create_dir_all(kit.logs()).unwrap();
    let log = kit.logs().join("adminer.log");
    fs::write(&log, "started").unwrap();

    migrate(&kit);

    assert!(!kit.apps().join("adminer").exists());
    assert!(!kit.versions_dir().join("adminer.txt").exists());
    assert!(!router.exists() && !sessions.exists() && !log.exists());
}
