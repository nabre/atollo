//! A preparation run: what to install, update or remove, in catalog order
//! (port of nabre/atollo `Prepare-Atollo.ps1`, DECISIONS.md D-11).
//!
//! Rules kept from the kit: a component fails alone (the others go on);
//! without "update all" an installed component is left as it is; "only"
//! touches the listed components and removes nothing; one run at a time per
//! kit (named mutex on Windows, `flock` on macOS and Linux). Cancellation
//! stops at once during a download and otherwise at the end of the current
//! step, never in the middle of a swap.

use std::collections::HashSet;
use std::sync::atomic::Ordering;

#[cfg(windows)]
use anyhow::Context;
use anyhow::{bail, Result};
use serde::Serialize;
#[cfg(windows)]
use windows::core::HSTRING;
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
#[cfg(windows)]
use windows::Win32::System::Threading::CreateMutexW;

use super::catalog::{Catalog, Choice, Component};
use crate::install::{self, Ctx, Progress};
use crate::kit::{Kit, Target};
use crate::net::archive::{self, InUse};
use crate::net::Cancelled;

#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Target platform to prepare; defaults to `ctx.kit.target()`.
    pub target: Option<Target>,
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

/// One run per kit on macOS and Linux: the kit's id in a set of this
/// process, and an exclusive `flock` on `data/app/prepare.lock` for the
/// other processes (another window, `prepare` from a terminal). The system
/// drops the `flock` when the file closes, also after a crash.
#[cfg(not(windows))]
#[derive(Debug)]
struct RunLock {
    id: String,
    #[allow(dead_code)]
    file: Option<std::fs::File>,
}

#[cfg(not(windows))]
impl Drop for RunLock {
    fn drop(&mut self) {
        if let Ok(mut set) = active_locks().lock() {
            set.remove(&self.id);
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
fn lock(kit: &Kit) -> Result<RunLock> {
    let name = HSTRING::from(format!(r"Local\Atollo-Prepare-{}", kit.id()));
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
fn lock(kit: &Kit) -> Result<RunLock> {
    let id = kit.id();
    {
        let mut set = active_locks()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !set.insert(id.clone()) {
            return Err(Busy.into());
        }
    }
    // From here the guard takes the id out of the set again, also on error.
    let mut guard = RunLock { id, file: None };
    guard.file = Some(prepare_flock(kit)?);
    Ok(guard)
}

/// The exclusive `flock` on `data/app/prepare.lock`, without waiting:
/// `Busy` when another process holds it. Any other error (a file system
/// without `flock`, some network shares) is returned: the run does not
/// start unguarded.
#[cfg(unix)]
fn prepare_flock(kit: &Kit) -> Result<std::fs::File> {
    use anyhow::Context;
    use std::os::fd::AsRawFd;
    let dir = kit.app_dir();
    std::fs::create_dir_all(&dir).with_context(|| dir.display().to_string())?;
    let path = dir.join("prepare.lock");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .with_context(|| path.display().to_string())?;
    // SAFETY: a valid descriptor owned by `file`.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        let e = std::io::Error::last_os_error();
        if e.kind() == std::io::ErrorKind::WouldBlock {
            return Err(Busy.into());
        }
        // No silent fallback: as `os::instance::acquire`, a file system
        // without `flock` stops the run instead of preparing unguarded.
        return Err(e).with_context(|| format!("flock {}", path.display()));
    }
    Ok(file)
}

/// What a run would do, before doing it (shown by the interface).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Plan {
    pub handle: Vec<String>,
    pub remove: Vec<String>,
}

/// Nothing unavailable on the kit's target (e.g. `noArm64` on `win32-arm64`,
/// PHP on macOS/Linux, Android on Linux ARM64, an x64 build on a Mac
/// without Rosetta 2) is ever planned.
pub fn plan(ctx: &Ctx, catalog: &Catalog, choice: &Choice, opts: &Options) -> Result<Plan> {
    for id in opts.only.iter().chain(&opts.skip) {
        if catalog.find(id).is_none() {
            bail!("unknown component: {id}");
        }
    }
    let target = ctx.kit.target();
    let can_run = crate::router::target::can_execute_on_host(target);
    let installed: HashSet<&str> = catalog
        .all()
        .iter()
        .filter(|c| {
            c.is_installed(ctx.kit) || (!can_run && crate::router::bootstrap::is_staged(ctx.kit, c))
        })
        .map(|c| c.id.as_str())
        .collect();
    let wanted = if opts.only.is_empty() {
        chosen_without_skipped(catalog, choice, &opts.skip, &installed, target)
    } else {
        only_with_needs(catalog, &opts.only, &installed, target)?
    };
    // What the chosen components need on this target only (Zig for C and
    // C++ on macOS and Linux): not in the choice, yet not removed while
    // they are chosen, also in a run that skips them.
    let kept = catalog.needs_only_on(&choice.chosen_ids(catalog), target);
    let mut plan = Plan::default();
    // In the order of the Tools page: the progress goes down the list.
    for c in catalog.in_page_order() {
        if wanted.contains(c.id.as_str()) {
            plan.handle.push(c.id.clone());
        } else if opts.only.is_empty()
            && !choice.is_chosen(&c.id)
            && !kept.contains(c.id.as_str())
            && has_trace(ctx.kit, c)
        {
            // Also when only part of it is there (an installation that
            // failed halfway), or only what was pre-downloaded for it.
            plan.remove.push(c.id.clone());
        }
    }
    Ok(plan)
}

/// The components to remove from the kit's other systems in the same run:
/// `components.txt` is one for every system, so a component taken out of
/// the choice goes from each `data/apps/<target>` that still has some of
/// it, not only from the one prepared. What a chosen one needs on that
/// system only (Zig next to C and C++) stays. Nothing with `only` (a run
/// that removes nothing).
pub fn remove_elsewhere(
    kit: &Kit,
    catalog: &Catalog,
    choice: &Choice,
    opts: &Options,
) -> Vec<(Target, String)> {
    if !opts.only.is_empty() {
        return Vec::new();
    }
    let chosen = choice.chosen_ids(catalog);
    let mut out = Vec::new();
    for target in Target::ALL {
        if target == kit.target() {
            continue;
        }
        let other = kit.for_target(target);
        let kept = catalog.needs_only_on(&chosen, target);
        for c in catalog.in_page_order() {
            if !choice.is_chosen(&c.id) && !kept.contains(c.id.as_str()) && has_trace(&other, c) {
                out.push((target, c.id.clone()));
            }
        }
    }
    out
}

/// `only`: the listed components plus, transitively, what they need on `target` and is
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
        if !crate::router::target::installable_on(c, target) {
            let platform = if c.is_available_on(target) {
                // Available, but this Mac has no Rosetta 2 for its x64 build.
                "this Mac without Rosetta 2"
            } else if target == Target::WIN32_ARM64 {
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
        for need in c.needs_on(target).filter_map(|n| catalog.find(n)) {
            if !installed.contains(need.id.as_str()) {
                todo.push((need, Some(c.id.as_str())));
            }
        }
    }
    Ok(wanted)
}

/// The choice, with what it needs on `target` only (never saved: see
/// [`Choice`]), without `skip` (system components and what they need cannot
/// be skipped) and without what is unavailable on `target`; then,
/// transitively, without every component not installed yet (locked ones
/// aside) that needs one neither planned nor installed. A need of this
/// target skipped drops what needs it for this run only.
fn chosen_without_skipped<'a>(
    catalog: &'a Catalog,
    choice: &Choice,
    skip: &[String],
    installed: &HashSet<&str>,
    target: Target,
) -> HashSet<&'a str> {
    let locked = catalog.locked_on(target);
    let can_plan = |c: &Component| {
        crate::router::target::installable_on(c, target)
            && (locked.contains(c.id.as_str()) || !skip.contains(&c.id))
    };
    let mut wanted: HashSet<&str> = catalog
        .all()
        .iter()
        .filter(|c| choice.is_chosen(&c.id) && can_plan(c))
        .map(|c| c.id.as_str())
        .collect();
    let planned: Vec<&str> = wanted.iter().copied().collect();
    for id in catalog.needs_only_on(&planned, target) {
        if catalog.find(id).is_some_and(can_plan) {
            wanted.insert(id);
        }
    }
    loop {
        let missing_need: Vec<&str> = catalog
            .all()
            .iter()
            .filter(|c| wanted.contains(c.id.as_str()) && !locked.contains(c.id.as_str()))
            .filter(|c| !installed.contains(c.id.as_str()))
            .filter(|c| {
                c.needs_on(target)
                    .any(|n| !wanted.contains(n) && !installed.contains(n))
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
    let _lock = lock(ctx.kit)?;
    let catalog = Catalog::get();
    if !opts.lines.is_empty() {
        let results = run_lines(ctx, catalog, &opts.lines, sink);
        install::conda::sweep(ctx.kit);
        return Ok(results);
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
    let elsewhere = remove_elsewhere(ctx.kit, catalog, &choice, opts);
    let update_all = opts.update_all || !opts.only.is_empty();
    // "Update all" also brings every saved line to its latest patch.
    let mut patches = if opts.update_all {
        saved_lines(ctx, catalog, &plan.handle)
    } else {
        Vec::new()
    };
    // Saved lines marked when the kit moved (conda environments, D-92):
    // installed again at their line, even at the same version.
    for (id, line) in marked_lines(ctx, catalog, &plan.handle) {
        match patches.iter_mut().find(|(i, l, _)| *i == id && *l == line) {
            Some(patch) => patch.2 = None,
            None => patches.push((id, line, None)),
        }
    }
    let named = |id: &String| catalog.find(id).map(|c| (c.id.clone(), c.name.clone()));
    sink.emit(Event::Queue {
        steps: plan
            .remove
            .iter()
            .chain(&plan.handle)
            .filter_map(named)
            .chain(elsewhere.iter().filter_map(|(target, id)| {
                catalog
                    .find(id)
                    .map(|c| (c.id.clone(), elsewhere_name(c, *target)))
            }))
            .chain(patches.iter().filter_map(|(id, line, _)| {
                catalog
                    .find(id)
                    .map(|c| (c.id.clone(), format!("{} {line}", c.name)))
            }))
            .collect(),
    });
    let total = plan.handle.len() + plan.remove.len() + elsewhere.len() + patches.len();
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
                forget(ctx.kit, c);
                Outcome::Removed
            }
        };
        sink.emit(Event::Done {
            id: c.id.clone(),
            outcome: outcome.clone(),
        });
        results.push((c.id.clone(), outcome));
    }

    for (target, id) in &elsewhere {
        let Some(c) = catalog.find(id) else { continue };
        index += 1;
        sink.emit(Event::Step {
            index,
            total,
            id: c.id.clone(),
            name: elsewhere_name(c, *target),
        });
        let other = ctx.kit.for_target(*target);
        let there = Ctx {
            kit: &other,
            ..*ctx
        };
        let outcome = match remove_component(&there, c) {
            Err(e) if e.is::<InUse>() => Outcome::InUse,
            Err(e) => Outcome::failed(&e),
            Ok(()) => {
                forget(&other, c);
                Outcome::Removed
            }
        };
        sink.emit(Event::Done {
            id: c.id.clone(),
            outcome: outcome.clone(),
        });
        results.push((c.id.clone(), outcome));
    }
    // The end of the run sweeps the prepared system only: the kept conda
    // packages of the other systems that lost a component go now (D-118).
    let mut swept: Vec<Target> = Vec::new();
    for (target, _) in &elsewhere {
        if !swept.contains(target) {
            swept.push(*target);
            install::conda::sweep(&ctx.kit.for_target(*target));
        }
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
    // The kept conda packages no list names any more (D-118): after the
    // installations and removals of the run, under its lock.
    install::conda::sweep(ctx.kit);
    Ok(results)
}

/// `f` while no preparation runs on the kit, under the same lock
/// (`Busy` otherwise): the start's sweep of the kept conda packages and
/// «Libera» of the Settings page (D-118).
pub fn while_idle<T>(kit: &Kit, f: impl FnOnce() -> T) -> Result<T> {
    let _lock = lock(kit)?;
    Ok(f())
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

/// The saved lines of the components a run handles that are marked to be
/// installed again because the kit moved (`Component::lines_to_reinstall`).
pub(crate) fn marked_lines(ctx: &Ctx, catalog: &Catalog, ids: &[String]) -> Vec<(String, String)> {
    ids.iter()
        .filter_map(|id| catalog.find(id))
        .flat_map(|c| {
            c.lines_to_reinstall(ctx.kit)
                .into_iter()
                .map(|line| (c.id.clone(), line))
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
        // A line marked when the kit moved: from the kept packages first
        // (D-118), at its own version.
        if current.is_none() && c.lines_to_reinstall(ctx.kit).iter().any(|l| l == line) {
            let mut progress = StepProgress {
                id: &c.id,
                sink: &mut *sink,
                last_percent: None,
            };
            if let Some(version) = install::conda::reinstall(ctx, c, Some(line), &mut progress) {
                return Ok(Outcome::Installed { version });
            }
        }
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
        Err(e) => failed(ctx, c, &e),
    };
    sink.emit(Event::Done {
        id: c.id.clone(),
        outcome: outcome.clone(),
    });
    outcome
}

/// The name of a step of [`remove_elsewhere`]: the component and its system
/// (`Python (linux-x64)`), apart from the steps of the system prepared.
fn elsewhere_name(c: &Component, target: Target) -> String {
    format!("{} ({target})", c.name)
}

/// Something of `c` is on `kit`'s system: a part or its folder
/// (`Component::is_present`), or only what was pre-downloaded for it there
/// (bootstrap-staged for another system, its folder still empty).
fn has_trace(kit: &Kit, c: &Component) -> bool {
    c.is_present(kit)
        || crate::router::bootstrap::is_staged(kit, c)
        || crate::router::bootstrap::has_staged(kit, &c.id)
}

/// After a removal: its version and marks, the lists of its kept conda
/// packages (the archives go with the sweep) and the files added by hand
/// for it, on `kit`'s system.
fn forget(kit: &Kit, c: &Component) {
    c.forget_version(kit);
    install::conda::forget(kit, &c.id, None);
    crate::setup::downloads::forget(kit, &c.id);
}

/// [`Outcome::failed`]; a download to do in the browser is noted as
/// wanted by `c`, so removing `c` also removes the file added for it.
fn failed(ctx: &Ctx, c: &Component, e: &anyhow::Error) -> Outcome {
    if let Some(by_hand) = e.downcast_ref::<crate::net::ByHand>() {
        crate::setup::downloads::want(ctx.kit, &c.id, &by_hand.expect);
    }
    Outcome::failed(e)
}

/// Removes the component's folder, the other files it owns and what was
/// pre-downloaded for it (`.bootstrap`). Folders are renamed first, so one
/// in use is left untouched (`InUse`).
fn remove_component(ctx: &Ctx, c: &Component) -> Result<()> {
    for path in c.owned_paths(ctx.kit) {
        // A link (`bin/pnpm` of npm on macOS and Linux) goes as a link,
        // also when what it points to went first.
        #[cfg(unix)]
        if std::fs::symlink_metadata(&path).is_ok_and(|m| m.file_type().is_symlink()) {
            std::fs::remove_file(&path).map_err(|_| InUse(path.clone()))?;
            continue;
        }
        if path.is_dir() {
            archive::remove_dir(&path)?;
        } else if path.exists() {
            std::fs::remove_file(&path).map_err(|_| InUse(path.clone()))?;
        }
    }
    crate::router::bootstrap::remove(ctx.kit, c)
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
    let staged = crate::router::bootstrap::is_staged(ctx.kit, c);
    let can_run = crate::router::target::can_execute_on_host(ctx.kit.target());
    let current = c.installed_version(ctx.kit);
    // Marked when the kit moved (a conda environment, D-92) or bootstrap-staged
    // on another OS and now running on its target OS: installed again even at
    // the same version, a versioned one at its own line.
    let reinstall = (installed && c.needs_reinstall(ctx.kit)) || (staged && can_run);
    let force = force || reinstall;
    let keep_line = current
        .as_deref()
        .and_then(crate::versions::line_of)
        .filter(|_| reinstall && c.versioned);
    let ready_for_target = installed || (staged && !can_run);
    // A part of the app at another version than this release's: installed
    // again without "update all", and never newer than the fixed one.
    if ready_for_target && !update_all && !force && !c.outdated(ctx.kit) {
        return Outcome::UpToDate { version: current };
    }
    let Some(installer) = install::installer(&c.id) else {
        return if ready_for_target {
            Outcome::UpToDate { version: current }
        } else {
            Outcome::Unsupported
        };
    };
    // Moved: a conda environment is created again from the packages kept
    // in the kit, without the network; from the network as before when
    // they are missing or changed (D-118).
    if installed && c.needs_reinstall(ctx.kit) {
        let mut progress = StepProgress {
            id: &c.id,
            sink: &mut *sink,
            last_percent: None,
        };
        if let Some(version) = install::conda::reinstall(ctx, c, None, &mut progress) {
            return Outcome::Installed { version };
        }
    }
    let result = (|| -> Result<Outcome> {
        let release = match (at, &keep_line) {
            (Some(line), _) if !installed => install::release_at(ctx, c, line)?,
            (_, Some(line)) if installer.has_lines() => install::release_at(ctx, c, line)?,
            _ => installer.latest(ctx, c)?,
        };
        if ready_for_target && !force && current.as_deref() == Some(release.version.as_str()) {
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
        Err(e) => failed(ctx, c, &e),
    }
}

#[cfg(test)]
#[path = "../../../test/rust/setup/runner.rs"]
mod tests;
