//! Lines on disk changing outside a choice: removed from the Tools page,
//! deleted by hand, moved from an older kit's layout.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::{
    check, default_dir, default_line, find, has_key, install_dir, line_of, lined, project_users,
    saved, saved_dir, store, valid_line, version_in, versioned, write_version, Line,
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
        crate::net::archive::remove_dir(&saved_dir(kit, c, line))?;
        c.clear_line_reinstall(kit, line);
        return Ok(());
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
    let target = kit.target();
    // What the chosen ones need on this target only (Zig for C and C++ on
    // Linux) is not in the saved choice, so `Choice::set` cannot follow it:
    // the chosen components that need it on this target go off here.
    let kept = catalog.needs_only_on(&before.chosen_ids(catalog), target);
    for c in &gone {
        c.forget_version(kit);
        if catalog.is_locked(&c.id) {
            continue;
        }
        choice.set(catalog, &c.id, false);
        if kept.contains(c.id.as_str()) {
            for d in catalog.all() {
                if choice.is_chosen(&d.id)
                    && !catalog.is_locked(&d.id)
                    && d.id != c.id
                    && catalog
                        .with_needs_on(&[&d.id], target)
                        .contains(c.id.as_str())
                {
                    choice.set(catalog, &d.id, false);
                }
            }
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
/// saved `versions\<id>\<line>` into `node\<line>`, `tools\gh.exe` into
/// `gh\<line>\bin`, a folder per patch (`apache\2.4.69`) into its line. The
/// saved lines of components no longer versioned go, and the old lines of
/// the tools always at their latest version. A folder in use stays as it is
/// (the kit works with it) and is moved at the next start.
pub fn migrate(kit: &Kit) {
    for c in lined() {
        if let Err(e) = migrate_default(kit, c) {
            crate::log::warn(&format!("{}: default folder not moved ({e:#})", c.id));
        }
        if let Err(e) = migrate_from_tools(kit, c) {
            crate::log::warn(&format!("{}: not moved from tools ({e:#})", c.id));
        }
        if let Err(e) = migrate_patch_folders(kit, c) {
            crate::log::warn(&format!("{}: version folders not renamed ({e:#})", c.id));
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
    for c in lined() {
        drop_old_lines(kit, c);
    }
    drop_retired(kit);
    // The databases' data, one folder per version too.
    crate::db::services::migrate_data(kit);
    if let Err(e) = crate::install::adopt_composer(kit) {
        crate::log::warn(&format!("Composer of the older kit: {e:#}"));
    }
    if let Err(e) = crate::install::migrate_php_ini(kit) {
        crate::log::warn(&format!("php.ini per line of PHP: {e:#}"));
    }
    if let Err(e) = crate::projects::apache::fix_kit_conf(kit) {
        crate::log::warn(&format!("Apache kit.conf: {e:#}"));
    }
}

/// Components no longer in the catalog: Windows Terminal (D-95), Adminer
/// (phpMyAdmin in its place, D-107).
const RETIRED: [&str; 2] = ["terminal", "adminer"];

/// What an older kit kept of a component no longer in the catalog: its
/// folder in `data\apps`, its version files and its files in `data\kit`. A
/// folder in use stays and goes at the next start.
fn drop_retired(kit: &Kit) {
    for id in RETIRED {
        let dir = kit.apps().join(id);
        if dir.exists() {
            match crate::net::archive::remove_dir(&dir) {
                Ok(()) => crate::log::info(&format!("{id}: no longer in the catalog, removed")),
                Err(e) => crate::log::warn(&format!("{id}: not removed ({e:#})")),
            }
        }
        let prefix = format!("{id}@");
        for e in fs::read_dir(kit.versions_dir())
            .into_iter()
            .flatten()
            .flatten()
        {
            let name = e.file_name().to_string_lossy().into_owned();
            if name == format!("{id}.txt")
                || name == format!("{id}.reinstall")
                || (name.starts_with(&prefix) && name.ends_with(".reinstall"))
            {
                let _ = fs::remove_file(e.path());
            }
        }
        let _ = fs::remove_file(kit.versions_root().join(format!("{id}.txt")));
    }
    // The start script of Windows Terminal's consoles.
    let _ = fs::remove_file(kit.kit_files().join("console.ps1"));
    // Adminer's router, sessions and log.
    let _ = fs::remove_file(kit.kit_files().join("adminer.php"));
    let _ = fs::remove_dir_all(kit.app_dir().join("adminer"));
    let _ = fs::remove_file(kit.logs().join("adminer.log"));
}

/// `node\node.exe` → `node\<line>\node.exe`, through `node.devkit-move`
/// (left by an interrupted start: taken back first).
fn migrate_default(kit: &Kit, c: &Component) -> Result<()> {
    if c.folder.is_none() {
        return Ok(());
    }
    let store = store(kit, c);
    let mut moving = store.clone().into_os_string();
    moving.push(".devkit-move");
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

/// The lines of a `minor_folder` component other than the installed one's,
/// left by an update to a new line. One in use stays, for the next start.
pub fn drop_old_lines(kit: &Kit, c: &Component) {
    if !c.minor_folder || c.versioned {
        return;
    }
    let Some(line) = default_line(kit, c) else {
        return;
    };
    if !has_key(c, &saved_dir(kit, c, &line)) {
        return;
    }
    for entry in fs::read_dir(store(kit, c)).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == line || !valid_line(&name) || !entry.path().is_dir() {
            continue;
        }
        match crate::net::archive::remove_dir(&entry.path()) {
            Ok(()) => crate::log::info(&format!("{}: {name} removed, {line} replaces it", c.id)),
            Err(e) => crate::log::warn(&format!("{}: {name} not removed ({e:#})", c.id)),
        }
    }
}

/// The executables of a kit that kept them in `tools\` (GitHub CLI, uv)
/// move into the folder of their line, where the key is (`gh\2.102\bin`):
/// the `tools/` files of `remove`, when the key's file is one of them.
fn migrate_from_tools(kit: &Kit, c: &Component) -> Result<()> {
    let (Some(folder), Some(version)) = (&c.folder, c.installed_version(kit)) else {
        return Ok(());
    };
    let key = c.key_for(kit.target());
    let rel = key
        .strip_prefix(folder.as_str())
        .unwrap_or(&key)
        .trim_start_matches('/');
    let (sub, exe) = rel.rsplit_once('/').unwrap_or(("", rel));
    let tools = kit.apps().join("tools");
    let files: Vec<PathBuf> = c
        .remove
        .iter()
        .map(|r| crate::setup::catalog::translate_rel(kit.target(), r))
        .filter_map(|r| r.strip_prefix("tools/").map(str::to_owned))
        .map(|name| tools.join(name))
        .filter(|p| p.is_file())
        .collect();
    let dir = install_dir(kit, c, &version);
    if has_key(c, &dir) || !files.iter().any(|p| p.ends_with(exe)) {
        return Ok(());
    }
    let into = crate::fsx::join_rel(&dir, sub);
    fs::create_dir_all(&into)?;
    for from in &files {
        let to = into.join(from.file_name().unwrap_or_default());
        fs::rename(from, &to).with_context(|| to.display().to_string())?;
    }
    write_version(&dir, &version)?;
    crate::log::info(&format!("{}: moved from tools to {}", c.id, dir.display()));
    Ok(())
}

/// A folder per patch, from a kit that kept them (Apache: `apache\2.4.69`):
/// the installed version's takes the name of its line (`apache\2.4`), the
/// other ones of that line go.
fn migrate_patch_folders(kit: &Kit, c: &Component) -> Result<()> {
    let installed = c.installed_version(kit);
    let mut folders: Vec<(String, PathBuf, Option<String>)> = fs::read_dir(store(kit, c))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let line = line_of(&name)?;
            let patch = line != name && name.chars().all(|ch| ch.is_ascii_digit() || ch == '.');
            (patch && has_key(c, &e.path())).then(|| (name, e.path(), version_in(&e.path())))
        })
        .collect();
    // The installed version first, then the newest.
    folders.sort_by_key(|(name, _, version)| {
        (
            version.is_none() || *version != installed,
            std::cmp::Reverse(crate::install::version_key(name)),
        )
    });
    for (name, path, _) in folders {
        let Some(line) = line_of(&name) else { continue };
        let to = saved_dir(kit, c, &line);
        if to.exists() {
            crate::net::archive::remove_dir(&path)?;
            crate::log::info(&format!("{}: {name} removed, {line} replaces it", c.id));
        } else {
            fs::rename(&path, &to).with_context(|| to.display().to_string())?;
            crate::log::info(&format!("{}: {name} renamed to {line}", c.id));
        }
    }
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
