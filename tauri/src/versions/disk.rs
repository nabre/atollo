//! Lines on disk changing outside a choice: removed from the Tools page,
//! deleted by hand, moved from an older kit's layout.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::{
    check, default_dir, default_line, find, has_key, install_dir, project_users, saved, saved_dir,
    store, valid_line, versioned, write_version, Line,
};
use crate::kit::Kit;
use crate::setup::catalog::{Catalog, Component};

/// Removes a line. The default goes only while another one stays, which
/// becomes the default (the newest); the last one goes with the component
/// (the Tools page, which follows what needs it).
pub fn remove(kit: &Kit, id: &str, line: &str) -> Result<()> {
    check(id, Some(line))?;
    let Some(c) = find(id) else { return Ok(()) };
    // Locked: the projects that use it keep it.
    if let Some(users) = project_users(kit).get(&(id.to_owned(), line.to_owned())) {
        bail!("{id} {line}: used by {}", users.join(", "));
    }
    if default_line(kit, c).as_deref() != Some(line) {
        return crate::net::archive::remove_dir(&saved_dir(kit, c, line));
    }
    let Some(next) = saved(kit, c).into_iter().next() else {
        bail!("{id} {line}: the only version, it goes with the component");
    };
    let dir = default_dir(kit, c);
    // Never the component's folder (an older kit's layout, not moved yet):
    // it holds every version.
    if dir != saved_dir(kit, c, line) {
        bail!("{id} {line}: not in its own folder yet");
    }
    crate::net::archive::remove_dir(&dir)?;
    promote(kit, c, &next)
}

/// `line` becomes the default: the installed version is its own.
fn promote(kit: &Kit, c: &Component, line: &Line) -> Result<()> {
    let version = line.version.clone().unwrap_or_else(|| line.line.clone());
    c.save_version(kit, &version)?;
    crate::log::info(&format!("{}: {version} is the default", c.id));
    Ok(())
}

/// What was deleted by hand from `data\apps` (D-81), when the Tools page
/// opens: a component installed before (its version is known) of which
/// nothing is on disk is no longer chosen (with what needs it) and its
/// version is forgotten; a versioned one whose default folder went takes
/// the newest line left. Required components stay chosen: they come back
/// at the next start. Returns the names no longer chosen.
pub fn sync_with_disk(kit: &Kit) -> Result<Vec<String>> {
    let catalog = Catalog::get();
    for c in versioned() {
        if let Err(e) = repair_default(kit, c) {
            crate::log::warn(&format!("{}: no default version ({e:#})", c.id));
        }
    }
    let Some(mut choice) = crate::setup::catalog::Choice::read(kit, catalog) else {
        return Ok(Vec::new());
    };
    let gone: Vec<&Component> = catalog
        .all()
        .iter()
        .filter(|c| c.installed_version(kit).is_some() && !c.is_present(kit))
        .collect();
    let before = choice.clone();
    for c in &gone {
        c.forget_version(kit);
        if !catalog.is_locked(&c.id) {
            choice.set(catalog, &c.id, false);
        }
    }
    if choice == before {
        return Ok(Vec::new());
    }
    choice.write(kit, catalog)?;
    let off: Vec<String> = catalog
        .all()
        .iter()
        .filter(|c| before.is_chosen(&c.id) && !choice.is_chosen(&c.id))
        .map(|c| c.name.clone())
        .collect();
    crate::log::info(&format!(
        "deleted by hand, no longer chosen: {}",
        off.join(", ")
    ));
    Ok(off)
}

/// The default's folder is gone (removed by hand) and other lines are
/// there: the newest becomes the default.
fn repair_default(kit: &Kit, c: &Component) -> Result<()> {
    if c.installed_version(kit).is_none() || has_key(c, &default_dir(kit, c)) {
        return Ok(());
    }
    match saved(kit, c).into_iter().next() {
        Some(next) => promote(kit, c, &next),
        None => Ok(()),
    }
}

/// Brings the folders of an older kit to one folder per version, at start
/// before anything uses them: the default `node\` into `node\<line>`, the
/// saved `versions\<id>\<line>` into `node\<line>`. The saved lines of
/// components no longer versioned go. A folder in use stays as it is (the
/// kit works with it) and is moved at the next start.
pub fn migrate(kit: &Kit) {
    for c in versioned() {
        if let Err(e) = migrate_default(kit, c) {
            crate::log::warn(&format!("{}: default folder not moved ({e:#})", c.id));
        }
    }
    let old = kit.apps().join("versions");
    for entry in fs::read_dir(&old).into_iter().flatten().flatten() {
        let id = entry.file_name().to_string_lossy().into_owned();
        let result = match find(&id) {
            Some(c) => migrate_saved(kit, c, &entry.path()),
            None => crate::net::archive::remove_dir(&entry.path()),
        };
        if let Err(e) = result {
            crate::log::warn(&format!("{id}: saved versions not moved ({e:#})"));
        }
    }
    let _ = fs::remove_dir(&old);
    for c in versioned() {
        if let Err(e) = repair_default(kit, c) {
            crate::log::warn(&format!("{}: no default version ({e:#})", c.id));
        }
    }
    // The databases' data, one folder per version too.
    crate::db::services::migrate_data(kit);
    if let Err(e) = crate::install::adopt_composer(kit) {
        crate::log::warn(&format!("Composer of the older kit: {e:#}"));
    }
    if let Err(e) = crate::projects::apache::fix_kit_conf(kit) {
        crate::log::warn(&format!("Apache kit.conf: {e:#}"));
    }
}

/// `node\node.exe` → `node\<line>\node.exe`, through `node.atollo-move`
/// (left by an interrupted start: taken back first).
fn migrate_default(kit: &Kit, c: &Component) -> Result<()> {
    if c.folder.is_none() {
        return Ok(());
    }
    let store = store(kit, c);
    let mut moving = store.clone().into_os_string();
    moving.push(".atollo-move");
    let moving = PathBuf::from(moving);
    if moving.exists() && !store.exists() {
        fs::rename(&moving, &store).context("taking back the folder being moved")?;
    }
    if !has_key(c, &store) {
        return Ok(());
    }
    let Some(version) = c.installed_version(kit) else {
        return Ok(()); // no version to name its folder: it stays
    };
    let to = install_dir(kit, c, &version);
    fs::rename(&store, &moving).with_context(|| store.display().to_string())?;
    let moved = fs::create_dir_all(&store).and_then(|()| fs::rename(&moving, &to));
    if let Err(e) = moved {
        let _ = fs::remove_dir(&store);
        let _ = fs::rename(&moving, &store);
        return Err(e.into());
    }
    write_version(&to, &version)?;
    crate::log::info(&format!("{}: {version} moved to {}", c.id, to.display()));
    Ok(())
}

/// `versions\<id>\<line>` → `<folder>\<line>`; a line already there wins.
fn migrate_saved(kit: &Kit, c: &Component, old: &Path) -> Result<()> {
    for entry in fs::read_dir(old)?.flatten() {
        let line = entry.file_name().to_string_lossy().into_owned();
        let to = saved_dir(kit, c, &line);
        if !valid_line(&line) || has_key(c, &to) {
            crate::net::archive::remove_dir(&entry.path())?;
            continue;
        }
        fs::create_dir_all(store(kit, c))?;
        fs::rename(entry.path(), &to).with_context(|| to.display().to_string())?;
    }
    fs::remove_dir(old)?;
    Ok(())
}

#[cfg(test)]
mod tests {
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
        moving.push(".atollo-move");
        install_fake(&kit, python, &PathBuf::from(&moving));
        python.save_version(&kit, "3.13.10").unwrap();
        migrate(&kit);
        assert!(has_key(python, &py.join("3.13")));
        assert!(!PathBuf::from(moving).exists());

        assert_eq!(dir_name("20251104 build"), "20251104_build");
    }
}
