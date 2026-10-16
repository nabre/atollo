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
#[path = "../../../test/rust/versions/disk.rs"]
mod tests;
