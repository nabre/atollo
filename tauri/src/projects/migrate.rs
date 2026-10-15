//! The kit's data of the projects from layout 1 to layout 2 (D-89,
//! PLAN-DATA.md WP-6), at start before anything reads it:
//!
//! ```text
//! data\app\projects\<p>\              → data\projects\<p>\
//! data\app\ports.json  "<p>|<action>" → data\projects\<p>\ports.json
//! data\app\db\secrets.json  projects  → data\projects\<p>\private\db.json
//! data\db\backup\<p>\                 → data\projects\<p>\private\backups\
//! ```
//!
//! - `data\app\layout.txt` = `2` once done: written last, before the old
//!   folders (emptied) and files go;
//! - nothing is ever overwritten: a file already at its new place wins, the
//!   old one stays where it was (logged), a different password stays in
//!   `secrets.json` (logged without the passwords);
//! - every step can run again: an error leaves layout 1 and the next start
//!   tries again. The steps do not depend on each other, except that the
//!   global files are rewritten only after their copy in `data\app\pre-v2\`
//!   (the way back to an older version of the app, by hand);
//! - no read falls back to the old paths: this runs before the window, the
//!   runner and the versions read anything, and what a failed step leaves
//!   behind waits, untouched, for the next start.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{Map, Value};

use super::state::{has_files, remove_empty_dirs, ProjectState};
use crate::db::auth;
use crate::fsx::{try_read_json, write_atomic, write_json};
use crate::kit::Kit;
use crate::os::safe_path::is_link;

const LAYOUT: &str = "2";

fn layout_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("layout.txt")
}
fn copies_dir(kit: &Kit) -> PathBuf {
    kit.app_dir().join("pre-v2")
}
fn old_ports(kit: &Kit) -> PathBuf {
    kit.app_dir().join("ports.json")
}
fn secrets_file(kit: &Kit) -> PathBuf {
    auth::dir(kit).join("secrets.json")
}
fn old_backups(kit: &Kit) -> PathBuf {
    kit.data().join(r"db\backup")
}

/// Migrates once; never panics, logs what it did not do.
pub fn run(kit: &Kit) {
    if fs::read_to_string(layout_file(kit)).is_ok_and(|t| t.trim() == LAYOUT) {
        return;
    }
    match migrate(kit) {
        Ok(()) => crate::log::info("data of the projects: layout 2 (data\\projects)"),
        Err(e) => crate::log::warn(&format!(
            "data of the projects: still layout 1, tried again at the next start ({e:#})"
        )),
    }
}

fn migrate(kit: &Kit) -> Result<()> {
    let mut failed: Vec<String> = Vec::new();
    let mut step = |name: &str, r: Result<()>| {
        if let Err(e) = r {
            failed.push(format!("{name}: {e:#}"));
        }
    };
    // T6.2 then T6.4 and T6.5: a global file is rewritten only once copied.
    let ports_copied = copy_ports(kit);
    let secrets_copied = copy_secrets(kit);
    step("states", move_states(kit));
    match ports_copied {
        Ok(()) => step("ports", split_ports(kit)),
        Err(e) => step("ports copy", Err(e)),
    }
    #[cfg(test)]
    if tests::INTERRUPT.get() {
        bail!("interrupted after the ports");
    }
    match secrets_copied {
        Ok(()) => step("passwords", split_passwords(kit)),
        Err(e) => step("secrets copy", Err(e)),
    }
    step("copies", move_backups(kit));
    if !failed.is_empty() {
        bail!("{}", failed.join("; "));
    }
    // T6.7: the layout first, then the old places emptied by the steps.
    write_atomic(&layout_file(kit), LAYOUT.as_bytes()).context("layout.txt")?;
    remove_empty_dirs(&kit.project_states());
    remove_empty_dirs(&old_backups(kit));
    let ports = old_ports(kit);
    if read_map(&ports).is_ok_and(|m| m.is_some_and(|m| is_empty_ports(&m))) {
        let _ = fs::remove_file(&ports);
    }
    Ok(())
}

/// A JSON object from `path`; `None` when missing or empty, an error when
/// it cannot be read or is not an object, so it is never overwritten.
fn read_map(path: &Path) -> Result<Option<Map<String, Value>>> {
    if is_link(path) {
        bail!("{}: a link, left alone", path.display());
    }
    try_read_json(path)
}

/// T6.2: `data\app\pre-v2\ports.json`, once (a later try keeps the first).
fn copy_ports(kit: &Kit) -> Result<()> {
    let (from, to) = (old_ports(kit), copies_dir(kit).join("ports.json"));
    if !from.is_file() || to.exists() {
        return Ok(());
    }
    fs::create_dir_all(copies_dir(kit))?;
    fs::copy(&from, &to).with_context(|| to.display().to_string())?;
    Ok(())
}

/// T6.2: `data\app\pre-v2\secrets.json`, private like the original, only
/// when it still has the projects' passwords (or cannot be read).
fn copy_secrets(kit: &Kit) -> Result<()> {
    let (from, to) = (secrets_file(kit), copies_dir(kit).join("secrets.json"));
    if !from.is_file() || to.exists() {
        return Ok(());
    }
    if read_map(&from).is_ok_and(|m| m.is_none_or(|m| !m.contains_key("projects"))) {
        return Ok(());
    }
    // Never logged: the text has passwords.
    let text = fs::read_to_string(&from).with_context(|| from.display().to_string())?;
    auth::write_private_text(&to, &text)
}

/// T6.3: each `data\app\projects\<p>\` to `data\projects\<p>\`.
fn move_states(kit: &Kit) -> Result<()> {
    let old = kit.project_states();
    if is_link(&old) {
        bail!("{}: a link, left alone", old.display());
    }
    for entry in read_dir(&old)? {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(state) = ProjectState::of(kit, &name).filter(|_| is_real_dir(&entry)) else {
            crate::log::warn(&format!("{}: not a project, left", entry.path().display()));
            continue;
        };
        if !state.dir().exists() {
            fs::create_dir_all(kit.project_data())?;
            fs::rename(entry.path(), state.dir())
                .with_context(|| state.dir().display().to_string())?;
        } else {
            move_entries(&entry.path(), state.dir(), false)?;
        }
    }
    Ok(())
}

/// T6.4: `{ "ports": {"<p>|<action>": port}, "locked": ["<p>|<action>"] }`
/// into each project's `ports.json`, keeping what it already has. The old
/// file keeps only what could not move (other keys included).
fn split_ports(kit: &Kit) -> Result<()> {
    let path = old_ports(kit);
    let mut old = match read_map(&path) {
        Ok(Some(old)) => old,
        Ok(None) => return Ok(()),
        // The ports are found again; the file and its copy stay for a hand.
        Err(e) => {
            crate::log::warn(&format!("ports not moved: {e:#}"));
            return Ok(());
        }
    };
    let ports = take_object(&mut old, "ports");
    let locked = take_array(&mut old, "locked");
    let mut by_project: std::collections::BTreeMap<String, (Map<String, Value>, Vec<Value>)> =
        Default::default();
    let (mut left_ports, mut left_locked) = (Map::new(), Vec::new());
    for (key, port) in ports {
        match key.split_once('|') {
            Some((p, action)) => {
                let entry = by_project.entry(p.to_owned()).or_default();
                entry.0.insert(action.to_owned(), port);
            }
            None => {
                left_ports.insert(key, port);
            }
        }
    }
    for key in locked {
        match key.as_str().and_then(|k| k.split_once('|')) {
            Some((p, action)) => {
                let entry = by_project.entry(p.to_owned()).or_default();
                entry.1.push(Value::from(action));
            }
            None => left_locked.push(key),
        }
    }
    let mut failed = Vec::new();
    for (project, (ports, locked)) in by_project {
        // An invalid name stays for good; a file that could not be written
        // is tried again at the next start.
        let merged = match ProjectState::of(kit, &project) {
            None => Err(anyhow!("not a project name")),
            Some(s) => merge_ports(&s.ports_file(), &ports, &locked).inspect_err(|_| {
                failed.push(project.clone());
            }),
        };
        if let Err(e) = merged {
            crate::log::warn(&format!("{project}: ports left in data\\app ({e:#})"));
            for (action, port) in ports {
                left_ports.insert(format!("{project}|{action}"), port);
            }
            for action in locked {
                let action = action.as_str().unwrap_or_default();
                left_locked.push(Value::from(format!("{project}|{action}")));
            }
        }
    }
    old.insert("ports".into(), Value::Object(left_ports));
    old.insert("locked".into(), Value::Array(left_locked));
    write_json(&path, &old)?;
    if !failed.is_empty() {
        bail!("ports of {} not moved", failed.join(", "));
    }
    Ok(())
}

fn take_object(map: &mut Map<String, Value>, key: &str) -> Map<String, Value> {
    match map.remove(key) {
        Some(Value::Object(m)) => m,
        Some(other) => {
            map.insert(key.to_owned(), other);
            Map::new()
        }
        None => Map::new(),
    }
}

fn take_array(map: &mut Map<String, Value>, key: &str) -> Vec<Value> {
    match map.remove(key) {
        Some(Value::Array(a)) => a,
        Some(other) => {
            map.insert(key.to_owned(), other);
            Vec::new()
        }
        None => Vec::new(),
    }
}

/// Nothing left but empty `ports` and `locked`.
fn is_empty_ports(map: &Map<String, Value>) -> bool {
    map.iter().all(|(k, v)| match (k.as_str(), v) {
        ("ports", Value::Object(m)) => m.is_empty(),
        ("locked", Value::Array(a)) => a.is_empty(),
        _ => false,
    })
}

/// The ports of one project added to its file; a port or lock already
/// there wins.
fn merge_ports(file: &Path, ports: &Map<String, Value>, locked: &[Value]) -> Result<()> {
    let mut now = read_map(file)?.unwrap_or_default();
    let before = now.clone();
    let Value::Object(have) = now
        .entry("ports")
        .or_insert_with(|| Value::Object(Map::new()))
    else {
        bail!("{}: ports is not an object", file.display());
    };
    for (action, port) in ports {
        have.entry(action.clone()).or_insert_with(|| port.clone());
    }
    let Value::Array(have) = now
        .entry("locked")
        .or_insert_with(|| Value::Array(Vec::new()))
    else {
        bail!("{}: locked is not a list", file.display());
    };
    for action in locked {
        if !have.contains(action) {
            have.push(action.clone());
        }
    }
    if now != before {
        write_json(file, &now)?;
    }
    Ok(())
}

/// T6.5: `projects` of `secrets.json` into each project's `private\db.json`,
/// a password already there kept; `secrets.json` rewritten only after every
/// project's file was written, with only what could not move.
fn split_passwords(kit: &Kit) -> Result<()> {
    let path = secrets_file(kit);
    let Some(mut secrets) = read_map(&path)? else {
        return Ok(());
    };
    let projects = match secrets.get("projects") {
        None => return Ok(()),
        Some(Value::Object(projects)) => projects.clone(),
        Some(_) => {
            crate::log::warn("secrets.json: projects is not an object, left");
            return Ok(());
        }
    };
    let mut kept = Map::new();
    for (project, engines) in &projects {
        let (Some(state), Value::Object(engines)) = (ProjectState::of(kit, project), engines)
        else {
            crate::log::warn(&format!(
                "{project}: passwords left in secrets.json (not a project name)"
            ));
            kept.insert(project.clone(), engines.clone());
            continue;
        };
        let conflicts = merge_passwords(&state, project, engines)?;
        if !conflicts.is_empty() {
            kept.insert(project.clone(), Value::Object(conflicts));
        }
    }
    if kept.is_empty() {
        secrets.remove("projects");
    } else {
        secrets.insert("projects".into(), Value::Object(kept));
    }
    auth::write_private_text(&path, &serde_json::to_string_pretty(&secrets)?)
}

/// The passwords of one project added to its `db.json` (its folder made
/// private first); returns those that differ from one already there.
fn merge_passwords(
    state: &ProjectState,
    project: &str,
    engines: &Map<String, Value>,
) -> Result<Map<String, Value>> {
    let file = state.db_file()?;
    let mut now = read_map(&file)?.unwrap_or_default();
    let mut conflicts = Map::new();
    let mut changed = false;
    for (engine, password) in engines {
        match now.get(engine) {
            None => {
                now.insert(engine.clone(), password.clone());
                changed = true;
            }
            Some(have) if have == password => {}
            Some(_) => {
                crate::log::warn(&format!(
                    "{project}: {engine}: another password already in db.json; \
                     the old one stays in secrets.json"
                ));
                conflicts.insert(engine.clone(), password.clone());
            }
        }
    }
    if changed {
        auth::write_private_text(&file, &serde_json::to_string_pretty(&now)?)?;
    }
    Ok(conflicts)
}

/// T6.6: `data\db\backup\<p>\` into `private\backups\`, made private
/// before the first copy arrives; each copy owner-only once there.
fn move_backups(kit: &Kit) -> Result<()> {
    let old = old_backups(kit);
    if is_link(&old) {
        bail!("{}: a link, left alone", old.display());
    }
    for entry in read_dir(&old)? {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(state) = ProjectState::of(kit, &name).filter(|_| is_real_dir(&entry)) else {
            crate::log::warn(&format!("{}: not a project, left", entry.path().display()));
            continue;
        };
        if has_files(&entry.path()) {
            move_entries(&entry.path(), &state.backups_dir()?, true)?;
        }
    }
    Ok(())
}

/// The entries of `dir`; nothing when it does not exist.
fn read_dir(dir: &Path) -> Result<Vec<fs::DirEntry>> {
    match fs::read_dir(dir) {
        Ok(entries) => Ok(entries.collect::<io::Result<_>>()?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(anyhow!("{}: {e}", dir.display())),
    }
}

/// A folder, not a link to one.
fn is_real_dir(entry: &fs::DirEntry) -> bool {
    entry.file_type().is_ok_and(|t| t.is_dir())
}

/// Moves what `from` holds into `to`, never over something already there
/// (left in `from`, logged). `private`: `to` is owner-only and so must be
/// each file moved in: a rename keeps the file's old DACL, so it is set
/// again; folders are made anew (they inherit) and links are left.
fn move_entries(from: &Path, to: &Path, private: bool) -> Result<()> {
    fs::create_dir_all(to).with_context(|| to.display().to_string())?;
    for entry in read_dir(from)? {
        let target = to.join(entry.file_name());
        let dir = is_real_dir(&entry);
        let free =
            matches!(fs::symlink_metadata(&target), Err(e) if e.kind() == io::ErrorKind::NotFound);
        if private && !dir && entry.file_type().is_ok_and(|t| t.is_symlink()) {
            crate::log::warn(&format!("{}: a link, left", entry.path().display()));
        } else if dir && (private || !free) {
            if free || (target.is_dir() && !is_link(&target)) {
                move_entries(&entry.path(), &target, private)?;
            } else {
                skipped(&entry.path(), &target);
            }
        } else if free {
            fs::rename(entry.path(), &target).with_context(|| target.display().to_string())?;
            if private {
                if let Err(e) = crate::editor::restrict_to_owner(&target) {
                    crate::log::warn(&format!("{}: permissions: {e:#}", target.display()));
                }
            }
        } else {
            skipped(&entry.path(), &target);
        }
    }
    Ok(())
}

fn skipped(from: &Path, to: &Path) {
    crate::log::warn(&format!(
        "{}: {} already exists, left in the old folder",
        from.display(),
        to.display()
    ));
}

#[cfg(test)]
pub(super) mod tests {
    use std::cell::Cell;
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;
    use crate::db::auth::tests::assert_owner_only;
    use crate::kit::{tests::TempDir, Arch};

    thread_local! {
        /// Stops the migration right after the ports (T6.4), as a crash would.
        pub(super) static INTERRUPT: Cell<bool> = const { Cell::new(false) };
    }

    const SECRETS: &str = r#"{"mariadbRoot":"r00t","mongodbAdmin":"adm1n","projects":{"shop":{"mariadb":"pw-shop-maria","mongodb":"pw-shop-mongo"},"blog":{"mariadb":"pw-blog"},".old":{"mariadb":"pw-old"}}}"#;
    const PORTS: &str = r#"{"ports":{"shop|web":8001,"shop|api":8002,"blog|web":8003,".old|web":8004},"locked":["shop|web",".old|web"]}"#;

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    fn json_of(path: &Path) -> Value {
        serde_json::from_str(&read(path)).unwrap()
    }

    /// A kit with every file of layout 1.
    fn v1_kit(tmp: &TempDir) -> Kit {
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let old = kit.project_states();
        write(
            &old.join(r"shop\dev-settings.json"),
            r#"{"agent":"claude"}"#,
        );
        write(&old.join(r"shop\apache\httpd.conf"), "Listen 8001\n");
        write(&old.join(r"shop\apache\conf.d\user.conf"), "# mine\n");
        write(&old.join(r"shop\versions.json"), r#"{"node":"22"}"#);
        write(&old.join(r"shop\versions.txt"), "a\tb\n");
        write(&old.join(r"blog\dev-settings.json"), r#"{"agent":"codex"}"#);
        write(&old.join(r".old\dev-settings.json"), "{}");
        write(&kit.app_dir().join("ports.json"), PORTS);
        write(&secrets_file(&kit), SECRETS);
        let backup = kit.data().join(r"db\backup");
        write(&backup.join(r"shop\shop-mariadb-1.sql"), "CREATE TABLE t;");
        write(&backup.join(r"shop\shop-mongodb-1.archive.gz"), "gz");
        fs::create_dir_all(backup.join("blog")).unwrap();
        kit
    }

    /// Every file under the kit's `data\` (but the logs) with its bytes.
    fn snapshot(kit: &Kit) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(dir: &Path, logs: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for e in fs::read_dir(dir).into_iter().flatten().flatten() {
                if e.path() == logs {
                    continue;
                }
                if e.file_type().unwrap().is_dir() {
                    walk(&e.path(), logs, out);
                } else {
                    out.insert(e.path(), fs::read(e.path()).unwrap());
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(&kit.data(), &kit.logs(), &mut out);
        out
    }

    /// What a complete migration of `v1_kit` gives.
    fn assert_migrated(kit: &Kit) {
        let new = kit.project_data();
        assert_eq!(read(&kit.app_dir().join("layout.txt")), "2");
        // states, Apache, versions files
        assert_eq!(
            read(&new.join(r"shop\dev-settings.json")),
            r#"{"agent":"claude"}"#
        );
        assert_eq!(read(&new.join(r"shop\apache\httpd.conf")), "Listen 8001\n");
        assert_eq!(read(&new.join(r"shop\apache\conf.d\user.conf")), "# mine\n");
        assert_eq!(read(&new.join(r"shop\versions.json")), r#"{"node":"22"}"#);
        assert_eq!(read(&new.join(r"shop\versions.txt")), "a\tb\n");
        assert_eq!(
            read(&new.join(r"blog\dev-settings.json")),
            r#"{"agent":"codex"}"#
        );
        // ports, locked included
        assert_eq!(
            json_of(&new.join(r"shop\ports.json")),
            json!({"ports": {"web": 8001, "api": 8002}, "locked": ["web"]})
        );
        assert_eq!(
            json_of(&new.join(r"blog\ports.json")),
            json!({"ports": {"web": 8003}, "locked": []})
        );
        // passwords, private
        let pw = |p, e| auth::project_password(kit, p, e);
        assert_eq!(pw("shop", "mariadb").as_deref(), Some("pw-shop-maria"));
        assert_eq!(pw("shop", "mongodb").as_deref(), Some("pw-shop-mongo"));
        assert_eq!(pw("blog", "mariadb").as_deref(), Some("pw-blog"));
        assert_owner_only(&new.join(r"shop\private\db.json"));
        // copies, owner-only after the move
        let backups = new.join(r"shop\private\backups");
        assert_eq!(read(&backups.join("shop-mariadb-1.sql")), "CREATE TABLE t;");
        assert_owner_only(&backups.join("shop-mariadb-1.sql"));
        assert_owner_only(&backups.join("shop-mongodb-1.archive.gz"));
        // copies of the global files, as they were; the secrets private
        let copies = kit.app_dir().join("pre-v2");
        assert_eq!(read(&copies.join("ports.json")), PORTS);
        assert_eq!(read(&copies.join("secrets.json")), SECRETS);
        assert_owner_only(&copies.join("secrets.json"));
        // the old places: emptied ones gone, an invalid name left
        let old = kit.project_states();
        assert!(!old.join("shop").exists() && !old.join("blog").exists());
        assert!(old.join(r".old\dev-settings.json").is_file());
        assert!(!kit.data().join(r"db\backup").exists());
        assert_eq!(
            json_of(&kit.app_dir().join("ports.json")),
            json!({"ports": {".old|web": 8004}, "locked": [".old|web"]})
        );
        assert!(!new.join(".old").exists());
    }

    #[test]
    fn v1_kit_to_v2_nothing_lost() {
        let tmp = TempDir::new("migrate-v2");
        let kit = v1_kit(&tmp);
        run(&kit);
        assert_migrated(&kit);
        let secrets = json_of(&secrets_file(&kit));
        assert_eq!(
            secrets,
            json!({"mariadbRoot": "r00t", "mongodbAdmin": "adm1n",
                   "projects": {".old": {"mariadb": "pw-old"}}}),
            "only what has no project left"
        );
        assert_owner_only(&secrets_file(&kit));

        // a second start does nothing; nor does a full run again
        let before = snapshot(&kit);
        run(&kit);
        assert_eq!(snapshot(&kit), before);
        fs::remove_file(layout_file(&kit)).unwrap();
        run(&kit);
        assert_eq!(snapshot(&kit), before);
    }

    #[test]
    fn a_new_kit_has_nothing_to_move() {
        let tmp = TempDir::new("migrate-new");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        write(&secrets_file(&kit), r#"{"mariadbRoot":"r00t"}"#);
        run(&kit);
        assert_eq!(read(&layout_file(&kit)), "2");
        assert!(!copies_dir(&kit).exists(), "no copy of a file of layout 2");
        assert_eq!(read(&secrets_file(&kit)), r#"{"mariadbRoot":"r00t"}"#);
        assert!(!kit.project_data().exists());
    }

    #[test]
    fn interrupted_after_the_ports_then_retried() {
        let tmp = TempDir::new("migrate-interrupt");
        let kit = v1_kit(&tmp);
        INTERRUPT.set(true);
        run(&kit);
        INTERRUPT.set(false);
        assert!(!layout_file(&kit).exists(), "still layout 1");
        let new = kit.project_data();
        assert!(new.join(r"shop\dev-settings.json").is_file());
        assert!(new.join(r"shop\ports.json").is_file());
        assert_eq!(read(&secrets_file(&kit)), SECRETS, "not touched yet");
        assert!(kit
            .data()
            .join(r"db\backup\shop\shop-mariadb-1.sql")
            .is_file());

        // the next start finishes it, the ports not doubled
        run(&kit);
        assert_migrated(&kit);
    }

    #[test]
    fn present_target_merged_and_password_conflict_kept() {
        let tmp = TempDir::new("migrate-merge");
        let kit = v1_kit(&tmp);
        let shop = ProjectState::of(&kit, "shop").unwrap();
        write(&shop.settings_file(), r#"{"agent":"newer"}"#);
        write(
            &shop.ports_file(),
            r#"{"ports":{"web":9000},"locked":[],"future":1}"#,
        );
        auth::set_project_password(&kit, "shop", "mariadb", "pw-newer").unwrap();
        write(
            &shop.backups_dir().unwrap().join("shop-mariadb-1.sql"),
            "newer",
        );
        run(&kit);

        assert_eq!(read(&layout_file(&kit)), "2");
        // what was there wins; the old file stays where it was
        assert_eq!(read(&shop.settings_file()), r#"{"agent":"newer"}"#);
        let old = kit.project_states().join("shop");
        assert_eq!(
            read(&old.join("dev-settings.json")),
            r#"{"agent":"claude"}"#
        );
        assert!(!old.join("apache").exists(), "moved: nothing was there");
        assert_eq!(read(&shop.apache_dir().join("httpd.conf")), "Listen 8001\n");
        assert_eq!(
            json_of(&shop.ports_file()),
            json!({"ports": {"web": 9000, "api": 8002}, "locked": ["web"], "future": 1})
        );
        // passwords: the newer kept, the missing added, the old one kept too
        let pw = |e| auth::project_password(&kit, "shop", e);
        assert_eq!(pw("mariadb").as_deref(), Some("pw-newer"));
        assert_eq!(pw("mongodb").as_deref(), Some("pw-shop-mongo"));
        assert_eq!(
            json_of(&secrets_file(&kit))["projects"],
            json!({"shop": {"mariadb": "pw-shop-maria"}, ".old": {"mariadb": "pw-old"}})
        );
        // copies: the one there wins, the other one moved
        let backups = shop.backups_dir().unwrap();
        assert_eq!(read(&backups.join("shop-mariadb-1.sql")), "newer");
        assert_eq!(read(&backups.join("shop-mongodb-1.archive.gz")), "gz");
        let left = kit.data().join(r"db\backup\shop\shop-mariadb-1.sql");
        assert_eq!(read(&left), "CREATE TABLE t;");
    }
}
