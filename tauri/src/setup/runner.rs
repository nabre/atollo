//! A preparation run: what to install, update or remove, in catalog order
//! (port of nabre/atollo `Prepare-Atollo.ps1`, DECISIONS.md D-11).
//!
//! Rules kept from the kit: a component fails alone (the others go on);
//! without "update all" an installed component is left as it is; "only"
//! touches the listed components and removes nothing; one run at a time per
//! kit (named mutex). Cancellation stops at once during a download and
//! otherwise at the end of the current step, never in the middle of a swap.

use std::collections::HashSet;
use std::sync::atomic::Ordering;

use anyhow::{bail, Context, Result};
use serde::Serialize;
#[cfg(windows)]
use windows::core::HSTRING;
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
#[cfg(windows)]
use windows::Win32::System::Threading::CreateMutexW;

use super::catalog::{Catalog, Choice, Component};
use crate::install::{self, Ctx, Progress};
use crate::kit::Target;
use crate::net::archive::{self, InUse};
use crate::net::Cancelled;

#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Only these components (installed or updated); nothing removed.
    pub only: Vec<String>,
    /// Not these, on top of the choice (required components cannot be skipped).
    pub skip: Vec<String>,
    /// Also update what is already installed.
    pub update_all: bool,
    /// Download again even when up to date.
    pub force: bool,
    /// Only these lines (`major.minor`) of versioned components, saved next
    /// to the default folder (`versions/mod.rs`); nothing else is done.
    pub lines: Vec<(String, String)>,
    /// Versioned components installed for the first time at this line
    /// (`major.minor`), not at the latest; ignored for the installed ones.
    pub at: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Outcome {
    Installed {
        version: String,
    },
    UpToDate {
        version: Option<String>,
    },
    Removed,
    InUse,
    Unsupported,
    Cancelled,
    Failed {
        message: String,
        #[serde(rename = "byHand")]
        by_hand: Option<ByHand>,
    },
}

/// Where to download in the browser the file that failed (`net::ByHand`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ByHand {
    pub url: String,
    pub file_name: String,
}

impl Outcome {
    pub fn is_failure(&self) -> bool {
        matches!(self, Outcome::Failed { .. } | Outcome::InUse)
    }

    pub fn failed(e: &anyhow::Error) -> Outcome {
        Outcome::Failed {
            message: format!("{e:#}"),
            by_hand: e.downcast_ref::<crate::net::ByHand>().map(|b| ByHand {
                url: b.url.clone(),
                file_name: b.file_name.clone(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum Event {
    /// The steps of the run, `(id, name)` as `Step` will say them, before
    /// the first: the interface shows them waiting.
    Queue {
        steps: Vec<(String, String)>,
    },
    Step {
        index: usize,
        total: usize,
        id: String,
        name: String,
    },
    Note {
        id: String,
        text: String,
    },
    Bytes {
        id: String,
        done: u64,
        total: Option<u64>,
    },
    Done {
        id: String,
        outcome: Outcome,
    },
}

pub trait Sink {
    fn emit(&mut self, event: Event);
}

/// One run per kit: `Local\Atollo-Prepare-<kit id>`.
#[cfg(windows)]
#[derive(Debug)]
struct RunLock(HANDLE);

#[cfg(windows)]
impl Drop for RunLock {
    fn drop(&mut self) {
        // SAFETY: handle from CreateMutexW, closed once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

#[cfg(not(windows))]
#[derive(Debug)]
struct RunLock(String);

#[cfg(not(windows))]
impl Drop for RunLock {
    fn drop(&mut self) {
        if let Ok(mut set) = active_locks().lock() {
            set.remove(&self.0);
        }
    }
}

#[cfg(not(windows))]
fn active_locks() -> &'static std::sync::Mutex<HashSet<String>> {
    static LOCKS: std::sync::OnceLock<std::sync::Mutex<HashSet<String>>> =
        std::sync::OnceLock::new();
    LOCKS.get_or_init(|| std::sync::Mutex::new(HashSet::new()))
}

#[derive(Debug, thiserror::Error)]
#[error("another preparation is already running in this kit")]
pub struct Busy;

#[cfg(windows)]
fn lock(kit_id: &str) -> Result<RunLock> {
    let name = HSTRING::from(format!(r"Local\Atollo-Prepare-{kit_id}"));
    // SAFETY: valid name, default security.
    let handle = unsafe { CreateMutexW(None, false, &name) }.context("CreateMutexW")?;
    // SAFETY: last error of CreateMutexW on this thread.
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        // SAFETY: handle from CreateMutexW.
        let _ = unsafe { CloseHandle(handle) };
        return Err(Busy.into());
    }
    Ok(RunLock(handle))
}

#[cfg(not(windows))]
fn lock(kit_id: &str) -> Result<RunLock> {
    let mut set = active_locks()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !set.insert(kit_id.to_owned()) {
        return Err(Busy.into());
    }
    Ok(RunLock(kit_id.to_owned()))
}

/// What a run would do, before doing it (shown by the interface).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Plan {
    pub handle: Vec<String>,
    pub remove: Vec<String>,
}

/// Nothing unavailable on the kit's target (e.g. `noArm64` on `win32-arm64`,
/// or `terminal` on macOS/Linux) is ever planned.
pub fn plan(ctx: &Ctx, catalog: &Catalog, choice: &Choice, opts: &Options) -> Result<Plan> {
    for id in opts.only.iter().chain(&opts.skip) {
        if catalog.find(id).is_none() {
            bail!("unknown component: {id}");
        }
    }
    let target = ctx.kit.target();
    let installed: HashSet<&str> = catalog
        .all()
        .iter()
        .filter(|c| c.is_installed(ctx.kit))
        .map(|c| c.id.as_str())
        .collect();
    let wanted = if opts.only.is_empty() {
        chosen_without_skipped(catalog, choice, &opts.skip, &installed, target)
    } else {
        only_with_needs(catalog, &opts.only, &installed, target)?
    };
    let mut plan = Plan::default();
    // In the order of the Tools page: the progress goes down the list.
    for c in catalog.in_page_order() {
        if wanted.contains(c.id.as_str()) {
            plan.handle.push(c.id.clone());
        } else if opts.only.is_empty() && !choice.is_chosen(&c.id) && c.is_present(ctx.kit) {
            // Also when only part of it is there (an installation that
            // failed halfway).
            plan.remove.push(c.id.clone());
        }
    }
    Ok(plan)
}

/// `only`: the listed components plus, transitively, what they need and is
/// not installed yet. A component unavailable on `target` is an error.
fn only_with_needs<'a>(
    catalog: &'a Catalog,
    only: &[String],
    installed: &HashSet<&str>,
    target: Target,
) -> Result<HashSet<&'a str>> {
    let mut wanted = HashSet::new();
    let mut todo: Vec<(&Component, Option<&str>)> = only
        .iter()
        .filter_map(|id| catalog.find(id))
        .map(|c| (c, None))
        .collect();
    while let Some((c, needed_by)) = todo.pop() {
        if !c.is_available_on(target) {
            let platform = if target == Target::WIN32_ARM64 {
                "ARM64"
            } else {
                target.as_str()
            };
            match needed_by {
                None => bail!("{} is not available on {platform}", c.id),
                Some(by) => bail!("{by} needs {}, which is not available on {platform}", c.id),
            }
        }
        if !wanted.insert(c.id.as_str()) {
            continue;
        }
        for need in c.needs.iter().filter_map(|n| catalog.find(n)) {
            if !installed.contains(need.id.as_str()) {
                todo.push((need, Some(c.id.as_str())));
            }
        }
    }
    Ok(wanted)
}

/// The choice without `skip` (system components and what they need cannot be
/// skipped) and without what is unavailable on `target`; then,
/// transitively, without every component not installed yet (locked ones
/// aside) that needs one neither planned nor installed.
fn chosen_without_skipped<'a>(
    catalog: &'a Catalog,
    choice: &Choice,
    skip: &[String],
    installed: &HashSet<&str>,
    target: Target,
) -> HashSet<&'a str> {
    let locked = catalog.locked();
    let mut wanted: HashSet<&str> = catalog
        .all()
        .iter()
        .filter(|c| choice.is_chosen(&c.id) && c.is_available_on(target))
        .filter(|c| locked.contains(c.id.as_str()) || !skip.contains(&c.id))
        .map(|c| c.id.as_str())
        .collect();
    loop {
        let missing_need: Vec<&str> = catalog
            .all()
            .iter()
            .filter(|c| wanted.contains(c.id.as_str()) && !locked.contains(c.id.as_str()))
            .filter(|c| !installed.contains(c.id.as_str()))
            .filter(|c| {
                c.needs
                    .iter()
                    .any(|n| !wanted.contains(n.as_str()) && !installed.contains(n.as_str()))
            })
            .map(|c| c.id.as_str())
            .collect();
        if missing_need.is_empty() {
            return wanted;
        }
        for id in missing_need {
            wanted.remove(id);
        }
    }
}

struct StepProgress<'a> {
    id: &'a str,
    sink: &'a mut dyn Sink,
    last_percent: Option<u64>,
}

impl Progress for StepProgress<'_> {
    fn note(&mut self, text: &str) {
        self.sink.emit(Event::Note {
            id: self.id.to_owned(),
            text: text.to_owned(),
        });
    }
    fn bytes(&mut self, done: u64, total: Option<u64>) {
        // Throttled: one event per percent (or per 4 MB without a size).
        let mark = match total {
            Some(t) if t > 0 => done * 100 / t,
            _ => done / (4 * 1024 * 1024),
        };
        if self.last_percent != Some(mark) {
            self.last_percent = Some(mark);
            self.sink.emit(Event::Bytes {
                id: self.id.to_owned(),
                done,
                total,
            });
        }
    }
}

/// Runs a preparation; returns the outcome of every component touched.
pub fn run(ctx: &Ctx, opts: &Options, sink: &mut dyn Sink) -> Result<Vec<(String, Outcome)>> {
    let _lock = lock(&ctx.kit.id())?;
    let catalog = Catalog::get();
    if !opts.lines.is_empty() {
        return Ok(run_lines(ctx, catalog, &opts.lines, sink));
    }
    let choice = match Choice::read(ctx.kit, catalog) {
        Some(choice) => choice,
        None => {
            let choice = Choice::defaults(catalog);
            choice.write(ctx.kit, catalog)?;
            choice
        }
    };
    let plan = plan(ctx, catalog, &choice, opts)?;
    let update_all = opts.update_all || !opts.only.is_empty();
    // "Update all" also brings every saved line to its latest patch.
    let patches = if opts.update_all {
        saved_lines(ctx, catalog, &plan.handle)
    } else {
        Vec::new()
    };
    let named = |id: &String| catalog.find(id).map(|c| (c.id.clone(), c.name.clone()));
    sink.emit(Event::Queue {
        steps: plan
            .remove
            .iter()
            .chain(&plan.handle)
            .filter_map(named)
            .chain(patches.iter().filter_map(|(id, line, _)| {
                catalog
                    .find(id)
                    .map(|c| (c.id.clone(), format!("{} {line}", c.name)))
            }))
            .collect(),
    });
    let total = plan.handle.len() + plan.remove.len() + patches.len();
    let mut results = Vec::new();
    let mut index = 0;

    for id in &plan.remove {
        let Some(c) = catalog.find(id) else { continue };
        index += 1;
        sink.emit(Event::Step {
            index,
            total,
            id: c.id.clone(),
            name: c.name.clone(),
        });
        let outcome = match remove_component(ctx, c) {
            Err(e) if e.is::<InUse>() => Outcome::InUse,
            Err(e) => Outcome::failed(&e),
            Ok(()) => {
                c.forget_version(ctx.kit);
                Outcome::Removed
            }
        };
        sink.emit(Event::Done {
            id: c.id.clone(),
            outcome: outcome.clone(),
        });
        results.push((c.id.clone(), outcome));
    }

    for id in &plan.handle {
        let Some(c) = catalog.find(id) else { continue };
        index += 1;
        if ctx.cancel.load(Ordering::Relaxed) {
            results.push((c.id.clone(), Outcome::Cancelled));
            continue;
        }
        sink.emit(Event::Step {
            index,
            total,
            id: c.id.clone(),
            name: c.name.clone(),
        });
        let at = opts
            .at
            .iter()
            .find(|(id, _)| *id == c.id)
            .map(|(_, line)| line.as_str());
        let outcome = step(ctx, c, update_all, opts.force, at, sink);
        sink.emit(Event::Done {
            id: c.id.clone(),
            outcome: outcome.clone(),
        });
        results.push((c.id.clone(), outcome));
    }

    for (id, line, current) in &patches {
        let Some(c) = catalog.find(id) else { continue };
        index += 1;
        // Become the default with the update of the default: done.
        if !crate::versions::saved(ctx.kit, c)
            .iter()
            .any(|l| &l.line == line)
        {
            continue;
        }
        let current = current.as_deref().filter(|_| !opts.force);
        let outcome = line_step(ctx, c, line, current, (index, total), sink);
        results.push((c.id.clone(), outcome));
    }
    Ok(results)
}

/// Downloads saved lines of versioned components, one step each.
fn run_lines(
    ctx: &Ctx,
    catalog: &Catalog,
    lines: &[(String, String)],
    sink: &mut dyn Sink,
) -> Vec<(String, Outcome)> {
    sink.emit(Event::Queue {
        steps: lines
            .iter()
            .filter_map(|(id, line)| {
                catalog
                    .find(id)
                    .map(|c| (c.id.clone(), format!("{} {line}", c.name)))
            })
            .collect(),
    });
    let mut results = Vec::new();
    for (index, (id, line)) in lines.iter().enumerate() {
        let Some(c) = catalog.find(id) else { continue };
        let outcome = line_step(ctx, c, line, None, (index + 1, lines.len()), sink);
        results.push((c.id.clone(), outcome));
    }
    results
}

/// The saved lines "Update all" brings to their latest patch, each in its
/// own folder: those of the versioned components it handles whose installer
/// knows the lines, with the version they have now.
fn saved_lines(
    ctx: &Ctx,
    catalog: &Catalog,
    ids: &[String],
) -> Vec<(String, String, Option<String>)> {
    ids.iter()
        .filter_map(|id| catalog.find(id))
        .filter(|c| c.versioned && c.is_installed(ctx.kit))
        .filter(|c| install::installer(&c.id).is_some_and(|i| i.has_lines()))
        .flat_map(|c| {
            crate::versions::saved(ctx.kit, c)
                .into_iter()
                .map(|l| (c.id.clone(), l.line, l.version))
        })
        .collect()
}

/// Installs the newest build of `line` of `c` as a saved line; with
/// `current` (the version on disk) only when the source has a newer one.
fn line_step(
    ctx: &Ctx,
    c: &Component,
    line: &str,
    current: Option<&str>,
    (index, total): (usize, usize),
    sink: &mut dyn Sink,
) -> Outcome {
    if ctx.cancel.load(Ordering::Relaxed) {
        return Outcome::Cancelled;
    }
    sink.emit(Event::Step {
        index,
        total,
        id: c.id.clone(),
        name: format!("{} {line}", c.name),
    });
    let result = (|| -> Result<Outcome> {
        let Some(installer) = install::installer(&c.id) else {
            return Ok(Outcome::Unsupported);
        };
        let release = installer.release_line(ctx, c, line)?;
        if current == Some(release.version.as_str()) {
            return Ok(Outcome::UpToDate {
                version: Some(release.version),
            });
        }
        let mut progress = StepProgress {
            id: &c.id,
            sink: &mut *sink,
            last_percent: None,
        };
        install::install_line(ctx, c, &release, &mut progress)?;
        Ok(Outcome::Installed {
            version: release.version,
        })
    })();
    let outcome = match result {
        Ok(outcome) => outcome,
        Err(e) if e.is::<Cancelled>() => Outcome::Cancelled,
        Err(e) if e.is::<InUse>() => Outcome::InUse,
        Err(e) => Outcome::failed(&e),
    };
    sink.emit(Event::Done {
        id: c.id.clone(),
        outcome: outcome.clone(),
    });
    outcome
}

/// Removes the component's folder and the other files it owns. Folders
/// are renamed first, so one in use is left untouched (`InUse`).
fn remove_component(ctx: &Ctx, c: &Component) -> Result<()> {
    for path in c.owned_paths(ctx.kit) {
        if path.is_dir() {
            archive::remove_dir(&path)?;
        } else if path.exists() {
            std::fs::remove_file(&path).map_err(|_| InUse(path.clone()))?;
        }
    }
    Ok(())
}

fn step(
    ctx: &Ctx,
    c: &Component,
    update_all: bool,
    force: bool,
    at: Option<&str>,
    sink: &mut dyn Sink,
) -> Outcome {
    let installed = c.is_installed(ctx.kit);
    let current = c.installed_version(ctx.kit);
    // Marked when the kit moved (a conda environment, D-92): installed
    // again even at the same version, a versioned one at its own line.
    let reinstall = installed && c.needs_reinstall(ctx.kit);
    let force = force || reinstall;
    let keep_line = current
        .as_deref()
        .and_then(crate::versions::line_of)
        .filter(|_| reinstall && c.versioned);
    // A part of the app at another version than this release's: installed
    // again without "update all", and never newer than the fixed one.
    if installed && !update_all && !force && !c.outdated(ctx.kit) {
        return Outcome::UpToDate { version: current };
    }
    let Some(installer) = install::installer(&c.id) else {
        return if installed {
            Outcome::UpToDate { version: current }
        } else {
            Outcome::Unsupported
        };
    };
    let result = (|| -> Result<Outcome> {
        let release = match (at, &keep_line) {
            (Some(line), _) if !installed => install::release_at(ctx, c, line)?,
            (_, Some(line)) if installer.has_lines() => install::release_at(ctx, c, line)?,
            _ => installer.latest(ctx, c)?,
        };
        if installed && !force && current.as_deref() == Some(release.version.as_str()) {
            return Ok(Outcome::UpToDate {
                version: current.clone(),
            });
        }
        let mut progress = StepProgress {
            id: &c.id,
            sink: &mut *sink,
            last_percent: None,
        };
        install::install(ctx, c, &release, &mut progress)?;
        Ok(Outcome::Installed {
            version: release.version,
        })
    })();
    match result {
        Ok(outcome) => outcome,
        Err(e) if e.is::<Cancelled>() => Outcome::Cancelled,
        Err(e) if e.is::<InUse>() => Outcome::InUse,
        Err(e) => Outcome::failed(&e),
    }
}

#[cfg(test)]
#[path = "../../../test/rust/setup/runner.rs"]
mod tests;
