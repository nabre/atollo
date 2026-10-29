//! The components with a new version (report v6/04, N-9, DECISIONS.md
//! D-161): a badge on Tools and a list with what changes, instead of
//! «Aggiorna tutto» blindly.
//!
//! - each installed component's latest version, asked to its source (the
//!   installer's `latest`, as an update does), kept in
//!   `data\app\updates.json` with the time;
//! - asked at the start at most once a day, only with «Aggiornamento
//!   automatico» on and without `ATOLLO_NO_UPDATE` (the privacy settings);
//!   «Controlla ora» asks at once;
//! - the notes: the catalog's `notes` (with `{version}`), else the
//!   component's site.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::env::KitEnv;
use crate::kit::Kit;
use crate::setup::{Catalog, Component};

/// How long the versions asked are good for.
const STALE_SECS: u64 = 24 * 60 * 60;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Latest {
    /// When they were asked (Unix seconds); 0: never.
    pub checked: u64,
    /// Component → its latest version.
    pub versions: BTreeMap<String, String>,
}

fn file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("updates.json")
}

use crate::common::time::now_secs as now;

pub fn load(kit: &Kit) -> Latest {
    crate::common::fsx::read_json(&file(kit)).unwrap_or_default()
}

/// Never asked, or more than a day ago.
pub fn stale(kit: &Kit) -> bool {
    now().saturating_sub(load(kit).checked) > STALE_SECS
}

/// `latest` is a newer version than `installed` (numbers compared one by
/// one: 2.10 after 2.9).
pub fn newer(latest: &str, installed: &str) -> bool {
    crate::install::version_key(latest) > crate::install::version_key(installed)
}

/// The components it asks for: installed, with an installer, not a part of
/// the app (those follow Atollo's own version), not an agent the kit no
/// longer keeps on this system (LEVELS.md §7, Q11).
fn watched(kit: &Kit) -> impl Iterator<Item = &'static Component> + '_ {
    Catalog::get().all().iter().filter(move |c| {
        c.version.is_none()
            && crate::install::installer(&c.id).is_some()
            && !super::agents::obsolete(c, kit.target())
            && c.is_installed(kit)
    })
}

/// Asks every watched component's source for its latest version and keeps
/// them. A source that does not answer keeps its last version.
pub fn check(kit: &Kit, env: &KitEnv, token: Option<String>) -> Result<Latest> {
    let token = token.or_else(|| crate::github::token(kit, env));
    let settings = crate::common::settings::Settings::load(kit).settings;
    let net = crate::net::Net::new(&settings, token)?
        .with_github_cache(kit.data().join("cache").join("github"));
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let ctx = crate::install::Ctx {
        kit,
        net: &net,
        env,
        cancel: &cancel,
        guest: None,
    };
    let mut latest = load(kit);
    for c in watched(kit) {
        let Some(installer) = crate::install::installer(&c.id) else {
            continue;
        };
        match installer.latest(&ctx, c) {
            Ok(release) => {
                latest.versions.insert(c.id.clone(), release.version);
            }
            Err(e) => crate::common::log::warn(&format!("latest version of {}: {e:#}", c.id)),
        }
    }
    latest.checked = now();
    crate::common::fsx::write_json(&file(kit), &latest)?;
    Ok(latest)
}

/// One component with a new version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Update {
    pub id: String,
    pub name: String,
    pub installed: String,
    pub latest: String,
    /// Its release notes, or else its site.
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Updates {
    /// When the versions were asked (Unix seconds), if ever.
    pub checked: Option<u64>,
    pub items: Vec<Update>,
}

/// The notes of `version` of `c`: the catalog's `notes` with the version,
/// or its site.
pub fn notes_url(c: &Component, version: &str) -> String {
    c.notes
        .as_deref()
        .map_or_else(|| c.home.clone(), |n| n.replace("{version}", version))
}

/// The watched components whose latest version is newer than the one
/// installed, in the catalog's order; an agent only while the agents are
/// `enabled` (the VM on, LEVELS.md §7). Nothing is asked.
pub fn pending(kit: &Kit, enabled: bool) -> Updates {
    let latest = load(kit);
    let items = watched(kit)
        .filter(|c| super::agents::installable(c, kit.target(), enabled))
        .filter_map(|c| {
            let installed = c.installed_version(kit)?;
            let new = latest.versions.get(&c.id)?;
            newer(new, &installed).then(|| Update {
                id: c.id.clone(),
                name: c.name.clone(),
                installed,
                latest: new.clone(),
                notes: notes_url(c, new),
            })
        })
        .collect();
    Updates {
        checked: (latest.checked > 0).then_some(latest.checked),
        items,
    }
}

#[cfg(test)]
#[path = "../../../test/rust/setup/updates.rs"]
mod tests;
