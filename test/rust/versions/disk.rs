use super::*;
use crate::kit::{tests::TempDir, Arch};
use crate::versions::tests::install_fake;
use crate::versions::{dir_name, VERSION_FILE};

#[test]
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
    assert_eq!(
        fs::read_to_string(&conf).unwrap(),
        "LoadFile \"${CP_PHP_FWD}/php8ts.dll\"\r\n"
    );
    // Nothing left to do the second time.
    migrate(&kit);
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

    assert_eq!(dir_name(python, "20251104 build"), "20251104_build");
}

#[test]
fn apache_folders_take_their_patch() {
    let tmp = TempDir::new("versions-patch");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    let apache = Catalog::get().find("apache").unwrap();
    let store = apache.folder_path(&kit).unwrap();
    // The default and a saved line of before, named `major.minor`.
    install_fake(&kit, apache, &store.join("2.4"));
    fs::write(store.join("2.4").join(VERSION_FILE), "2.4.66-251206").unwrap();
    apache.save_version(&kit, "2.4.66-251206").unwrap();
    migrate(&kit);
    assert!(has_key(apache, &store.join("2.4.66")));
    assert!(!store.join("2.4").exists());
    assert_eq!(default_line(&kit, apache).as_deref(), Some("2.4.66"));
    assert_eq!(dir_name(apache, "2.4.69-261002"), "2.4.69");

    // A new build installs next to it; the older one stays, saved.
    let next = install_dir(&kit, apache, "2.4.69-261002");
    install_fake(&kit, apache, &next);
    write_version(&next, "2.4.69-261002").unwrap();
    apache.save_version(&kit, "2.4.69-261002").unwrap();
    let saved: Vec<String> = saved(&kit, apache).into_iter().map(|l| l.line).collect();
    assert_eq!(saved, ["2.4.66"]);
}
