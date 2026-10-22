//! The ports of the projects' actions: kept from one start to the next in
//! each project's `ports.json`, or the one the user locked.

use std::collections::{BTreeMap, HashSet};
use std::sync::PoisonError;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::i18n::t as tr;
use crate::kit::Kit;
use crate::ports::is_port_free;
use crate::projects::state::{self, ProjectState};

use super::{key_of, Runner};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Ports {
    pub ports: BTreeMap<String, u16>,
    /// Ports chosen by the user: never moved to another one.
    pub locked: Vec<String>,
}

/// `ports.json` of one project (`data\projects\<project>\`): its actions'
/// ports, by action id, and those locked. Keys of a newer version stay.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct ProjectPorts {
    ports: BTreeMap<String, u16>,
    locked: Vec<String>,
    #[serde(flatten)]
    other: serde_json::Map<String, serde_json::Value>,
}

/// The ports files of all projects, in memory under `project|action` keys,
/// and as they are on disk.
pub(super) fn load_ports(kit: &Kit) -> (Ports, BTreeMap<String, ProjectPorts>) {
    let mut ports = Ports::default();
    let mut saved = BTreeMap::new();
    for project in state::all(kit) {
        let Some(file) = ProjectState::of(kit, &project).map(|s| s.ports_file()) else {
            continue;
        };
        let Some(p) = crate::fsx::read_json::<ProjectPorts>(&file) else {
            continue;
        };
        for (action, &port) in &p.ports {
            ports.ports.insert(key_of(&project, action), port);
        }
        ports
            .locked
            .extend(p.locked.iter().map(|a| key_of(&project, a)));
        saved.insert(project, p);
    }
    (ports, saved)
}

/// The ports of the projects' actions as their `ports.json` keep them,
/// under `project|action` keys (for [`crate::ports`]).
pub(crate) fn saved_ports(kit: &Kit) -> BTreeMap<String, u16> {
    load_ports(kit).0.ports
}

/// `project|action` keys split by project: `(project, action)`.
pub(super) fn split_key(key: &str) -> Option<(&str, &str)> {
    key.split_once('|')
}

impl Runner {
    pub fn ports(&self) -> Ports {
        self.ports
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Writes the `ports.json` of the projects whose ports changed since the
    /// last save, each atomically; the others are not touched. The first
    /// failure is returned after trying them all.
    pub(super) fn save_ports(&self) -> Result<()> {
        // `saved` is held while writing, so saves never overtake each other
        // and always write the ports as they are now.
        let mut saved = self.saved.lock().unwrap_or_else(PoisonError::into_inner);
        let ports = self.ports();
        let mut now: BTreeMap<String, ProjectPorts> = BTreeMap::new();
        for (key, &port) in &ports.ports {
            if let Some((project, action)) = split_key(key) {
                now.entry(project.to_owned())
                    .or_default()
                    .ports
                    .insert(action.to_owned(), port);
            }
        }
        for key in &ports.locked {
            if let Some((project, action)) = split_key(key) {
                now.entry(project.to_owned())
                    .or_default()
                    .locked
                    .push(action.to_owned());
            }
        }
        // a project that lost all its ports is written empty
        for project in saved.keys() {
            now.entry(project.clone()).or_default();
        }
        let mut result = Ok(());
        for (project, mut p) in now {
            let before = saved.get(&project);
            p.other = before.map(|b| b.other.clone()).unwrap_or_default();
            if before == Some(&p) || (before.is_none() && p == ProjectPorts::default()) {
                continue;
            }
            let Some(state) = ProjectState::of(&self.kit, &project) else {
                continue;
            };
            // nothing left to say and no folder: not created just for that
            if p.ports.is_empty() && p.locked.is_empty() && !state.dir().exists() {
                saved.remove(&project);
                continue;
            }
            match crate::fsx::write_json(&state.ports_file(), &p) {
                Ok(()) => {
                    saved.insert(project, p);
                }
                Err(e) => {
                    crate::log::warn(&format!("ports of {project}: {e:#}"));
                    if result.is_ok() {
                        result = Err(e);
                    }
                }
            }
        }
        result
    }

    /// The port of a web action: the one it had (if free), or the first free
    /// one in the range. A locked port in use is an error, never moved.
    pub fn assign_port(&self, key: &str, lang: &str) -> Result<u16> {
        let ports = self.ports();
        if let Some(&p) = ports.ports.get(key) {
            if is_port_free(p) {
                return Ok(p);
            }
            if ports.locked.iter().any(|k| k == key) {
                bail!(
                    "{}",
                    tr(lang, "La porta {0} è occupata da un altro programma. Chiudilo o cambia la porta.", &[&p.to_string()])
                );
            }
        }
        let mut used: HashSet<u16> = ports.ports.values().copied().collect();
        used.extend(self.entries().values().filter_map(|e| e.info.port));
        // Nor the port of a pool or a database version, even one not
        // running now (BUG-V5-07): the projects' own are those above.
        used.extend(
            crate::ports::ports_taken(&self.kit)
                .into_iter()
                .filter(|(_, o)| !matches!(o, crate::ports::Owner::Project { .. }))
                .map(|(p, _)| p),
        );
        let (from, to) = self.port_range;
        for n in from..=to {
            if used.contains(&n) {
                continue;
            }
            if !self
                .pending
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(n)
            {
                continue;
            }
            let free = is_port_free(n);
            if free {
                let mut ports = self.ports.lock().unwrap_or_else(PoisonError::into_inner);
                ports.ports.insert(key.to_owned(), n);
                drop(ports);
                // a failure is already in the log
                let _ = self.save_ports();
            }
            self.pending
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .remove(&n);
            if free {
                return Ok(n);
            }
        }
        bail!(
            "{}",
            tr(
                lang,
                "Nessuna porta libera tra {0} e {1}",
                &[&from.to_string(), &to.to_string()]
            )
        )
    }

    /// The user's port for an action (locked), or `None` to go back to an
    /// automatic one.
    pub fn set_port(&self, key: &str, port: Option<u16>, lang: &str) -> Result<Ports> {
        if !key.contains('|') {
            bail!("invalid key");
        }
        if let Some(n) = port {
            if n < 1024 {
                bail!("{}", tr(lang, "Porta tra {0} e {1}", &["1024", "65535"]));
            }
            // Not the port of a pool, of a database version or the editor's
            // (BUG-V5-05); the projects' own below, from memory.
            crate::ports::refuse_taken(
                &self.kit,
                n,
                |o| matches!(o, crate::ports::Owner::Project { .. }),
                lang,
            )?;
        }
        let mut ports = self.ports.lock().unwrap_or_else(PoisonError::into_inner);
        ports.locked.retain(|k| k != key);
        match port {
            None => {
                ports.ports.remove(key);
            }
            Some(n) => {
                if let Some((other, _)) = ports.ports.iter().find(|(k, v)| **v == n && *k != key) {
                    bail!(
                        "{}",
                        tr(
                            lang,
                            "Porta già assegnata a {0}",
                            &[&other.replace('|', " → ")]
                        )
                    );
                }
                ports.ports.insert(key.to_owned(), n);
                ports.locked.push(key.to_owned());
            }
        }
        let snapshot = ports.clone();
        drop(ports);
        self.save_ports()?;
        Ok(snapshot)
    }
}
