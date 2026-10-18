//! Nginx is no longer in the kit (D-119): its pools and sites become
//! Caddy's, at start before anything reads them.
//!
//! ```text
//! data\app\servers.json  profile "engine": "nginx" → "caddy", no serverLine
//!                        port of "nginx-1.27_php-8.4" → "caddy-<line>_php-8.4"
//! projects\<p>\panel.json  "nginx": "public"  → "caddy": "public"
//!                          "server": "nginx-1.27_php-8.4" → Caddy's pool
//! ```
//!
//! - a file is rewritten only after its copy in `data\app\pre-caddy\`
//!   (`servers.json`, `projects\<p>\panel.json`), made once: a later try
//!   keeps the first copy, the way back by hand;
//! - every step can run again, and does at every start: a file with no
//!   Nginx left is not touched, one that could not be written is tried
//!   again next time; nothing is ever deleted;
//! - the ports wait for Caddy: an automatic pool is named after its line
//!   (SC-2), so a port of Nginx's moves to the pool of the newest Caddy
//!   installed ([`nginx_ports`], at every scan), and stays under its old id
//!   (never pruned) until then. A site that named an automatic pool of
//!   Nginx's names Caddy's, or none (the pool of its project's lines) while
//!   Caddy is not installed;
//! - a file the migration did not get to (a read-only project) still works:
//!   `nginx` of a site and `"engine": "nginx"` of a profile read as Caddy.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{Map, Value};

use super::pools::{auto_pool_id, store_path, StoredServers};
use super::WebServerEngine;
use crate::fsx::write_atomic;
use crate::kit::Kit;

fn copies_dir(kit: &Kit) -> PathBuf {
    kit.app_dir().join("pre-caddy")
}

/// Migrates what is left of Nginx; never panics, logs what it did not do.
pub fn run(kit: &Kit) {
    let caddy = crate::db::services::server_lines(kit, "caddy")
        .into_iter()
        .next();
    let mut failed = Vec::new();
    match migrate_store(kit) {
        Ok(true) => crate::log::info("servers.json: the profiles of Nginx are Caddy's"),
        Ok(false) => {}
        Err(e) => failed.push(format!("servers.json: {e:#}")),
    }
    for dir in crate::projects::folders(kit) {
        match migrate_panel(kit, &dir, caddy.as_deref()) {
            Ok(true) => crate::log::info(&format!(
                "{}: the sites of Nginx are Caddy's",
                dir.display()
            )),
            Ok(false) => {}
            Err(e) => failed.push(format!("{}: {e:#}", dir.display())),
        }
    }
    if !failed.is_empty() {
        crate::log::warn(&format!(
            "Nginx to Caddy: tried again at the next start ({})",
            failed.join("; ")
        ));
    }
}

/// `src` copied to `dest` once: a copy already there is kept.
fn copy_once(src: &Path, dest: &Path) -> Result<()> {
    if dest.exists() {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dest).with_context(|| format!("copy to {}", dest.display()))?;
    Ok(())
}

/// The profiles of `servers.json` with the engine `nginx`: Caddy's, with
/// no server line (a line of Nginx is not one of Caddy). The raw JSON, so
/// that nothing it holds is lost. True if it was rewritten.
fn migrate_store(kit: &Kit) -> Result<bool> {
    let path = store_path(kit);
    let Ok(text) = crate::fsx::read_text(&path) else {
        return Ok(false);
    };
    let Ok(Value::Object(mut store)) = serde_json::from_str::<Value>(&text) else {
        return Ok(false);
    };
    let mut changed = false;
    if let Some(Value::Array(profiles)) = store.get_mut("profiles") {
        for p in profiles.iter_mut().filter_map(Value::as_object_mut) {
            if p.get("engine").and_then(Value::as_str) == Some("nginx") {
                p.insert("engine".into(), Value::String("caddy".into()));
                p.remove("serverLine");
                changed = true;
            }
        }
    }
    if !changed {
        return Ok(false);
    }
    copy_once(&path, &copies_dir(kit).join("servers.json"))?;
    let json = serde_json::to_string_pretty(&Value::Object(store))?;
    write_atomic(&path, json.as_bytes())?;
    Ok(true)
}

/// The sites of the project in `dir` (its `panel.json`, or the old
/// `pannello.json`): `nginx` becomes `caddy`, «Nginx» in the label
/// «Caddy», an automatic pool of Nginx in `server` Caddy's pool of the same
/// PHP (`caddy`, the newest Caddy installed) or none. True if rewritten.
fn migrate_panel(kit: &Kit, dir: &Path, caddy: Option<&str>) -> Result<bool> {
    let Some(path) = crate::detect::OVERRIDE_FILES
        .iter()
        .map(|f| dir.join(f))
        .find(|p| crate::os::safe_path::is_file(dir, p))
    else {
        return Ok(false);
    };
    if crate::os::safe_path::is_link(&path) {
        return Ok(false);
    }
    let Some(text) = crate::os::safe_path::read_text(dir, &path) else {
        return Ok(false);
    };
    let Ok(Value::Object(mut panel)) = serde_json::from_str::<Value>(&text) else {
        return Ok(false);
    };
    let mut changed = false;
    for key in ["actions", "azioni"] {
        if let Some(Value::Array(actions)) = panel.get_mut(key) {
            for a in actions.iter_mut().filter_map(Value::as_object_mut) {
                changed |= migrate_action(a, caddy);
            }
        }
    }
    if !changed {
        return Ok(false);
    }
    let (Some(project), Some(file)) = (dir.file_name(), path.file_name()) else {
        return Ok(false);
    };
    copy_once(
        &path,
        &copies_dir(kit).join("projects").join(project).join(file),
    )?;
    let json = serde_json::to_string_pretty(&Value::Object(panel))? + "\n";
    write_atomic(&path, json.as_bytes())?;
    Ok(true)
}

/// One action of a `panel.json` ([`migrate_panel`]). True if changed.
pub(super) fn migrate_action(a: &mut Map<String, Value>, caddy: Option<&str>) -> bool {
    let Some(folder) = a.remove("nginx") else {
        return false;
    };
    let has_caddy = a
        .get("caddy")
        .and_then(Value::as_str)
        .is_some_and(|c| !c.is_empty());
    if !has_caddy && folder.as_str().is_some_and(|f| !f.is_empty()) {
        a.insert("caddy".into(), folder);
    }
    for key in ["label", "etichetta"] {
        if let Some(Value::String(label)) = a.get_mut(key) {
            *label = label.replace("Nginx", "Caddy");
        }
    }
    let php = a
        .get("server")
        .and_then(Value::as_str)
        .and_then(parse_nginx_pool);
    if let Some(php) = php {
        match caddy {
            Some(line) => {
                let id = auto_pool_id(WebServerEngine::Caddy, Some(line), php.as_deref());
                a.insert("server".into(), Value::String(id));
            }
            None => {
                a.remove("server");
            }
        }
    }
    true
}

/// The PHP line (`None`: no PHP) of the id of an automatic pool of Nginx,
/// as written by SC-2 (`nginx-1.27_php-8.4`) or before (`nginx-1.27-php-8.4`).
/// `None` for any other id.
pub(super) fn parse_nginx_pool(id: &str) -> Option<Option<String>> {
    let rest = id.strip_prefix("nginx-")?;
    let (srv, php) = match rest
        .split_once("_php-")
        .or_else(|| rest.split_once("-php-"))
    {
        Some((srv, php)) => (srv, Some(php)),
        None => (rest, None),
    };
    if srv != "default" && !crate::versions::valid_line(srv) {
        return None;
    }
    match php {
        Some(p) if !crate::versions::valid_line(p) => None,
        p => Some(p.map(str::to_owned)),
    }
}

/// An automatic pool of Nginx: its port waits for Caddy ([`nginx_ports`]).
pub(super) fn is_nginx_pool(id: &str) -> bool {
    parse_nginx_pool(id).is_some()
}

/// The ports of the automatic pools of Nginx moved to Caddy's pool of the
/// same PHP on `caddy`, the newest Caddy installed (none: they wait). A
/// port already there for that pool stays. True if any moved.
pub(super) fn nginx_ports(store: &mut StoredServers, caddy: Option<&str>) -> bool {
    let Some(line) = caddy else {
        return false;
    };
    let moves: Vec<(String, String)> = store
        .ports
        .keys()
        .filter(|id| !store.profiles.iter().any(|p| &p.id == *id))
        .filter_map(|id| {
            let php = parse_nginx_pool(id)?;
            let new = auto_pool_id(WebServerEngine::Caddy, Some(line), php.as_deref());
            Some((id.clone(), new))
        })
        .collect();
    let mut moved = false;
    for (old, new) in moves {
        if let Some(port) = store.ports.remove(&old) {
            store.ports.entry(new).or_insert(port);
            moved = true;
        }
    }
    moved
}

#[cfg(test)]
#[path = "../../../test/rust/servers/migrate.rs"]
mod tests;
