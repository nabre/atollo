//! Which part of Atollo has each port (BUG-V5-05): the shared web server
//! pools (`servers.json`), every version of each database server
//! (`services.json`, [`line_ports`]), the actions of the projects
//! (`ports.json` of each) and the editor's range. A port chosen by hand
//! for one of them is refused when another has it, with its owner named:
//! never a pool that cannot start because a MySQL version listens there.
//! A new pool's automatic port is none of them either.
//!
//! Only files are read: no lock of the pools or of the database servers is
//! taken, so any of them can ask while holding its own.

use std::collections::BTreeMap;

use anyhow::{bail, Result};

use crate::db::services::{line_ports, load_settings, server_line, server_lines};
use crate::db::DbEngine;
use crate::i18n::t;
use crate::kit::Kit;
use crate::setup::catalog::Catalog;

/// What a port is kept for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Owner {
    /// A web server pool, by id.
    Pool(String),
    /// A version of a database server.
    Db { engine: DbEngine, line: String },
    /// An action of a project.
    Project { project: String, action: String },
    /// [`crate::editor::PORTS`].
    Editor,
}

impl Owner {
    /// Why `port` cannot be taken, in `lang`.
    pub fn refusal(&self, port: u16, lang: &str) -> String {
        let p = port.to_string();
        match self {
            Owner::Pool(id) => t(
                lang,
                "La porta {0} è già del server {1}: scegline un'altra.",
                &[&p, id],
            ),
            Owner::Db { engine, line } => {
                let name = Catalog::get()
                    .find(engine.id())
                    .map_or_else(|| engine.id().to_owned(), |c| c.name.clone());
                t(
                    lang,
                    "La porta {0} è già del database {1}: scegline un'altra.",
                    &[&p, &format!("{name} {line}")],
                )
            }
            Owner::Project { project, action } => t(
                lang,
                "La porta {0} è già del progetto {1}: scegline un'altra.",
                &[&p, &format!("{project} → {action}")],
            ),
            Owner::Editor => {
                let (from, to) = (crate::editor::PORTS.start(), crate::editor::PORTS.end());
                t(
                    lang,
                    "Le porte da {0} a {1} sono dell'editor",
                    &[&from.to_string(), &to.to_string()],
                )
            }
        }
    }
}

/// Every port kept and its owner, in the order pools, databases, projects,
/// editor; a port two of them have (from before this check) more than once.
fn owners(kit: &Kit) -> Vec<(u16, Owner)> {
    let mut out: Vec<(u16, Owner)> = crate::servers::pool_ports(kit)
        .into_iter()
        .map(|(id, p)| (p, Owner::Pool(id)))
        .collect();
    let settings = load_settings(kit);
    for engine in DbEngine::ALL {
        let id = engine.id();
        let setting = settings.get(id).cloned().unwrap_or_default();
        // The installed versions on the port they start on, and those
        // uninstalled since on the one they keep for their return.
        let mut ports = line_ports(
            &setting,
            id,
            server_line(kit, id).as_deref(),
            &server_lines(kit, id),
        );
        for (line, p) in &setting.ports {
            ports.entry(line.clone()).or_insert(*p);
        }
        out.extend(
            ports
                .into_iter()
                .map(|(line, p)| (p, Owner::Db { engine, line })),
        );
    }
    for (key, p) in crate::projects::run::saved_ports(kit) {
        if let Some((project, action)) = key.split_once('|') {
            let (project, action) = (project.to_owned(), action.to_owned());
            out.push((p, Owner::Project { project, action }));
        }
    }
    out.extend(crate::editor::PORTS.map(|p| (p, Owner::Editor)));
    out
}

/// The ports of Atollo and who has each (the first, when two have one).
pub fn ports_taken(kit: &Kit) -> BTreeMap<u16, Owner> {
    let mut taken = BTreeMap::new();
    for (p, owner) in owners(kit) {
        taken.entry(p).or_insert(owner);
    }
    taken
}

/// An error naming the owner of `port` when it is kept for anything but
/// what `own` accepts (the entry saving it: a pool its own port, a version
/// the one it has).
pub fn refuse_taken(kit: &Kit, port: u16, own: impl Fn(&Owner) -> bool, lang: &str) -> Result<()> {
    if let Some((_, owner)) = owners(kit)
        .into_iter()
        .find(|(p, owner)| *p == port && !own(owner))
    {
        bail!("{}", owner.refusal(port, lang));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../test/rust/ports.rs"]
mod tests;
