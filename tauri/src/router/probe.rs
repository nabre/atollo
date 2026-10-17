//! Multi-target disk probing and health verification across all 6 targets
//! (`win32-x64`, `win32-arm64`, `darwin-arm64`, `darwin-x64`, `linux-x64`,
//! `linux-arm64`).

use serde::Serialize;

use super::target::{can_execute_on_host, profile};
use crate::env::KitEnv;
use crate::kit::{Arch, Kit, Os, Target};
use crate::setup::catalog::{kit_path, Catalog, Choice, Component};
use crate::setup::checkup::{self, Check};

/// Status of one catalog component on a specific [`Target`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetToolStatus {
    pub id: String,
    pub name: String,
    pub available: bool,
    pub emulated: bool,
    pub chosen: bool,
    pub installed: bool,
    pub present: bool,
    pub version: Option<String>,
    pub needs_reinstall: bool,
    pub bootstrap_staged: bool,
}

/// Summary of a [`Target`]'s tools on disk inside `data/apps/<target>` and
/// `data/app/versions/<target>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetSummary {
    pub target: String,
    pub os: Os,
    pub arch: Arch,
    pub is_host: bool,
    pub executable_on_host: bool,
    pub emulated_on_host: bool,
    pub available_count: usize,
    pub installed_count: usize,
    pub tools: Vec<TargetToolStatus>,
}

/// Probes the on-disk state of every catalog component for `target` without
/// spawning any child processes.
pub fn probe_target(kit: &Kit, target: Target) -> TargetSummary {
    let scoped = kit.for_target(target);
    let catalog = Catalog::get();
    let choice = Choice::read(&scoped, catalog).unwrap_or_else(|| Choice::defaults(catalog));
    let prof = profile(target);

    let mut available_count = 0;
    let mut installed_count = 0;
    let mut tools = Vec::with_capacity(catalog.all().len());

    for c in catalog.all() {
        let available = super::target::installable_on(c, target);
        let emulated = c.is_emulated_on(target);
        let chosen = available && choice.is_chosen(&c.id);
        let installed = available && c.is_installed(&scoped);
        let present = c.is_present(&scoped);
        let version = c.installed_version(&scoped);
        let needs_reinstall = c.needs_reinstall(&scoped);
        let bootstrap_staged = available && super::bootstrap::is_staged(&scoped, c);

        if available {
            available_count += 1;
        }
        if installed {
            installed_count += 1;
        }

        tools.push(TargetToolStatus {
            id: c.id.clone(),
            name: c.name.clone(),
            available,
            emulated,
            chosen,
            installed,
            present,
            version,
            needs_reinstall,
            bootstrap_staged,
        });
    }

    TargetSummary {
        target: target.as_str().to_owned(),
        os: target.os,
        arch: target.arch,
        is_host: prof.is_host,
        executable_on_host: prof.executable_on_host,
        emulated_on_host: prof.emulated_on_host,
        available_count,
        installed_count,
        tools,
    }
}

/// Probes all 6 supported targets (`Target::ALL`) on the kit's disk.
pub fn probe_all_targets(kit: &Kit) -> Vec<TargetSummary> {
    Target::ALL
        .into_iter()
        .map(|t| probe_target(kit, t))
        .collect()
}

/// Static disk verification of one component when `kit.target()` cannot be
/// executed on the current host OS/CPU.
pub fn check_static(kit: &Kit, c: &Component) -> Option<Check> {
    if super::bootstrap::is_staged(kit, c) {
        let ver = c
            .installed_version(kit)
            .unwrap_or_else(|| "staged".to_owned());
        return Some(Check {
            id: c.id.clone(),
            name: c.name.clone(),
            ok: true,
            answer: format!("{ver} (bootstrap {})", kit.target()),
        });
    }
    let (program, _) = c.check.split_first()?;
    if !c.is_present(kit) {
        return None;
    }
    let result = match c.missing_part(kit) {
        Some(part) => Err(format!("{} not found", part.display())),
        None => {
            let exe = kit_path(kit, program);
            if !exe.is_file() {
                Err(format!("{} not found", exe.display()))
            } else {
                let ver = c
                    .installed_version(kit)
                    .unwrap_or_else(|| "installed".to_owned());
                Ok(format!("{ver} ({})", kit.target()))
            }
        }
    };
    Some(Check {
        id: c.id.clone(),
        name: c.name.clone(),
        ok: result.is_ok(),
        answer: match result {
            Ok(a) | Err(a) => a,
        },
    })
}

/// Verifies every installed component on `kit`: runs live version commands
/// when `kit.target()` is executable on the host, or verifies key files and
/// recorded versions on disk when probing a foreign target.
pub fn probe_health(kit: &Kit, env: &KitEnv) -> Vec<Check> {
    if can_execute_on_host(kit.target()) {
        checkup::check_all(kit, env)
    } else {
        Catalog::get()
            .all()
            .iter()
            .filter_map(|c| check_static(kit, c))
            .collect()
    }
}
