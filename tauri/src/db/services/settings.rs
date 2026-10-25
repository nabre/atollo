//! `data\app\services.json`: each server's port, version and automatic start, each version's port.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{bail, Result};

use super::{check_line, default_port, Services, Setting};
use crate::db::DbEngine;
use crate::kit::Kit;
use crate::setup::Catalog;

/// The version of `id` the server runs: the one in its settings while it is
/// on disk, otherwise the newest on disk.
pub fn server_line(kit: &Kit, id: &str) -> Option<String> {
    let c = Catalog::get().find(id)?;
    load_settings(kit)
        .get(id)
        .and_then(|s| s.line.clone())
        .filter(|l| crate::versions::is_present(kit, c, l))
        .or_else(|| crate::versions::newest_line(kit, c))
}

/// The installed versions of `id`, newest first.
pub fn server_lines(kit: &Kit, id: &str) -> Vec<String> {
    let Some(c) = Catalog::get().find(id) else {
        return Vec::new();
    };
    let mut lines: Vec<String> = crate::versions::default_line(kit, c)
        .into_iter()
        .chain(crate::versions::saved(kit, c).into_iter().map(|l| l.line))
        .collect();
    lines.sort_by_key(|l| std::cmp::Reverse(crate::install::version_key(l)));
    lines
}

/// The port of every version in `lines` (BUG-V5-02): each on its stored
/// port; `main` (the version the server runs), when it has none, on the
/// server's port; the others on the first free one after it (3307,
/// 3308…), in the order of `lines`. So choosing another version for the
/// server moves no version that has a port already. A stored port that
/// another version already has (`main` first) is given again.
pub fn line_ports(
    setting: &Setting,
    id: &str,
    main: Option<&str>,
    lines: &[String],
) -> BTreeMap<String, u16> {
    let port = setting.port.unwrap_or_else(|| default_port(id));
    let mut taken: Vec<u16> = Vec::new();
    let mut out = BTreeMap::new();
    let all: Vec<&str> = main
        .into_iter()
        .chain(
            lines
                .iter()
                .map(String::as_str)
                .filter(|l| Some(*l) != main),
        )
        .collect();
    for line in &all {
        if let Some(p) = setting.ports.get(*line).filter(|p| !taken.contains(p)) {
            taken.push(*p);
            out.insert((*line).to_owned(), *p);
        }
    }
    if let Some(m) = main.filter(|m| !out.contains_key(*m) && !taken.contains(&port)) {
        out.insert(m.to_owned(), port);
    }
    // The server's port is never the first free one of another version.
    taken.push(port);
    let mut next = port;
    for line in all {
        if out.contains_key(line) {
            continue;
        }
        loop {
            next = next.checked_add(1).unwrap_or(1024).max(1024);
            if !taken.contains(&next) {
                break;
            }
        }
        taken.push(next);
        out.insert(line.to_owned(), next);
    }
    out
}

/// The port of version `line` of `id` (None: the version the server runs).
pub fn line_port(kit: &Kit, id: &str, line: Option<&str>) -> u16 {
    let setting = load_settings(kit).get(id).cloned().unwrap_or_default();
    let main = server_line(kit, id);
    let server = setting.port.unwrap_or_else(|| default_port(id));
    match line.map(str::to_owned).or(main.clone()) {
        Some(l) => line_ports(&setting, id, main.as_deref(), &server_lines(kit, id))
            .get(&l)
            .copied()
            .unwrap_or(server),
        None => server,
    }
}

/// One turn at a time on `services.json`.
static SETTINGS_LOCK: Mutex<()> = Mutex::new(());

/// Reads `services.json`, lets `change` modify it and writes it when it
/// changed, with no other change in between (BUG-V5-02, as
/// `Settings::update`): two starts storing their port lose neither. An
/// error from `change` leaves the file as it is, and so does a file that is
/// there but cannot be read (SEC-V6-01): never written again as if empty,
/// the ports and versions of the other servers lost.
fn update_settings<T>(
    kit: &Kit,
    change: impl FnOnce(&mut BTreeMap<String, Setting>) -> Result<T>,
) -> Result<T> {
    let _one = SETTINGS_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let before: BTreeMap<String, Setting> =
        crate::common::fsx::try_read_json(&settings_path(kit))?.unwrap_or_default();
    let mut all = before.clone();
    let out = change(&mut all)?;
    if all != before {
        crate::common::fsx::write_json(&settings_path(kit), &all)?;
    }
    Ok(out)
}

/// An error naming the owner of `port` when a pool, a project, the editor
/// or another server has it (BUG-V5-05); the versions of `engine` itself
/// are checked by the caller, which may move them. Called before
/// [`update_settings`], never inside it.
fn refuse_other_owner(kit: &Kit, engine: DbEngine, port: u16, lang: &str) -> Result<()> {
    crate::common::ports::refuse_taken(
        kit,
        port,
        |o| matches!(o, crate::common::ports::Owner::Db { engine: e, .. } if *e == engine),
        lang,
    )
}

fn settings_path(kit: &Kit) -> PathBuf {
    kit.app_dir().join("services.json")
}

pub fn load_settings(kit: &Kit) -> BTreeMap<String, Setting> {
    crate::common::fsx::read_json(&settings_path(kit)).unwrap_or_default()
}

impl Services {
    /// The port given to a version at its first start, the version the
    /// server runs too, stored so that versions installed later, or
    /// another version chosen for the server, do not move it (BUG-V5-02).
    pub(super) fn keep_port(&self, id: &str, line: &str, port: u16) -> Result<()> {
        update_settings(&self.kit, |all| {
            let entry = all.entry(id.to_owned()).or_default();
            entry.ports.insert(line.to_owned(), port);
            Ok(())
        })
    }

    /// Port (None: default) and "start with Atollo" of `id`. A new port is
    /// the one of the version the server runs (its stored one dropped),
    /// unless another version has it, or a pool, a project or the editor
    /// (BUG-V5-05, in `lang`).
    pub fn set(&self, id: &str, port: Option<u16>, autostart: bool, lang: &str) -> Result<()> {
        let engine = DbEngine::parse(id)?;
        if port.is_some_and(|p| p < 1024) {
            bail!("port below 1024");
        }
        let main = server_line(&self.kit, id);
        let lines = server_lines(&self.kit, id);
        let new = port.unwrap_or_else(|| engine.default_port());
        if new != line_port(&self.kit, id, None) {
            refuse_other_owner(&self.kit, engine, new, lang)?;
        }
        update_settings(&self.kit, |all| {
            let entry = all.entry(id.to_owned()).or_default();
            if let Some(m) = &main {
                let ports = line_ports(entry, id, Some(m), &lines);
                if ports.get(m) != Some(&new) {
                    if let Some((other, _)) =
                        entry.ports.iter().find(|(l, q)| **q == new && *l != m)
                    {
                        bail!("port {new} is the one of {id} {other}");
                    }
                    entry.ports.remove(m);
                }
            }
            entry.port = port.filter(|p| *p != engine.default_port());
            entry.autostart = autostart;
            Ok(())
        })
    }

    /// The port of another version of `id` (None: the first free one after
    /// the server's), from its next start. Not the version the server runs
    /// (its port is the server's, [`Services::set`]) nor a port another
    /// version has, nor one of a pool, a project or the editor (BUG-V5-05,
    /// in `lang`).
    pub fn set_line_port(&self, id: &str, line: &str, port: Option<u16>, lang: &str) -> Result<()> {
        let engine = DbEngine::parse(id)?;
        let lines = server_lines(&self.kit, id);
        if !lines.iter().any(|x| x == line) {
            bail!("{id} {line} is not installed");
        }
        let main = server_line(&self.kit, id);
        if main.as_deref() == Some(line) {
            bail!("{id} {line} is the version the server runs: its port is the server's");
        }
        if port.is_some_and(|p| p < 1024) {
            bail!("port below 1024");
        }
        if let Some(p) = port.filter(|p| *p != line_port(&self.kit, id, Some(line))) {
            refuse_other_owner(&self.kit, engine, p, lang)?;
        }
        update_settings(&self.kit, |all| {
            let entry = all.entry(id.to_owned()).or_default();
            if let Some(p) = port {
                let ports = line_ports(entry, id, main.as_deref(), &lines);
                if let Some((other, _)) = ports.iter().find(|(l, q)| **q == p && *l != line) {
                    bail!("port {p} is the one of {id} {other}");
                }
                entry.ports.insert(line.to_owned(), p);
            } else {
                entry.ports.remove(line);
            }
            Ok(())
        })
    }

    /// The version `id` runs (None: the newest on disk), while it is
    /// stopped: each version has its own data (D-81). Every installed
    /// version keeps the port it has (stored now, BUG-V5-02): the `.env`
    /// of the projects on them stay right.
    pub fn set_line(&self, id: &str, line: Option<&str>) -> Result<()> {
        let engine = DbEngine::parse(id)?;
        // No start in between the check and the change.
        let _turn = self.op(engine);
        if let Some(l) = line {
            check_line(&self.kit, id, l)?;
        }
        // Any version, not only the one the server runs: another one on its
        // own port would become the server while every caller takes the
        // server's port for it.
        if self.running_server(engine).is_ok() || !self.running_lines(engine).is_empty() {
            bail!("{id} is running: stop it first");
        }
        let main = server_line(&self.kit, id);
        let lines = server_lines(&self.kit, id);
        update_settings(&self.kit, |all| {
            let entry = all.entry(id.to_owned()).or_default();
            let ports = line_ports(entry, id, main.as_deref(), &lines);
            entry.ports.extend(ports);
            entry.line = line.map(str::to_owned);
            Ok(())
        })
    }
}
