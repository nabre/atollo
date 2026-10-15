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
use windows::core::HSTRING;
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;

use super::catalog::{Catalog, Choice, Component};
use crate::install::{self, Ctx, Progress};
use crate::kit::Arch;
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
#[derive(Debug)]
struct RunLock(HANDLE);

impl Drop for RunLock {
    fn drop(&mut self) {
        // SAFETY: handle from CreateMutexW, closed once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

#[derive(Debug, thiserror::Error)]
#[error("another preparation is already running in this kit")]
pub struct Busy;

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

/// What a run would do, before doing it (shown by the interface).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Plan {
    pub handle: Vec<String>,
    pub remove: Vec<String>,
}

/// Nothing marked `noArm64` is ever planned on an ARM64 kit.
pub fn plan(ctx: &Ctx, catalog: &Catalog, choice: &Choice, opts: &Options) -> Result<Plan> {
    for id in opts.only.iter().chain(&opts.skip) {
        if catalog.find(id).is_none() {
            bail!("unknown component: {id}");
        }
    }
    let arm64 = ctx.kit.arch() == Arch::Arm64;
    let installed: HashSet<&str> = catalog
        .all()
        .iter()
        .filter(|c| c.is_installed(ctx.kit))
        .map(|c| c.id.as_str())
        .collect();
    let wanted = if opts.only.is_empty() {
        chosen_without_skipped(catalog, choice, &opts.skip, &installed, arm64)
    } else {
        only_with_needs(catalog, &opts.only, &installed, arm64)?
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
/// not installed yet. On ARM64 a component marked `noArm64` is an error.
fn only_with_needs<'a>(
    catalog: &'a Catalog,
    only: &[String],
    installed: &HashSet<&str>,
    arm64: bool,
) -> Result<HashSet<&'a str>> {
    let mut wanted = HashSet::new();
    let mut todo: Vec<(&Component, Option<&str>)> = only
        .iter()
        .filter_map(|id| catalog.find(id))
        .map(|c| (c, None))
        .collect();
    while let Some((c, needed_by)) = todo.pop() {
        if arm64 && c.no_arm64 {
            match needed_by {
                None => bail!("{} is not available on ARM64", c.id),
                Some(by) => bail!("{by} needs {}, which is not available on ARM64", c.id),
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
/// skipped) and, on ARM64, without what is marked `noArm64`; then,
/// transitively, without every component not installed yet (locked ones
/// aside) that needs one neither planned nor installed.
fn chosen_without_skipped<'a>(
    catalog: &'a Catalog,
    choice: &Choice,
    skip: &[String],
    installed: &HashSet<&str>,
    arm64: bool,
) -> HashSet<&'a str> {
    let locked = catalog.locked();
    let mut wanted: HashSet<&str> = catalog
        .all()
        .iter()
        .filter(|c| choice.is_chosen(&c.id) && !(arm64 && c.no_arm64))
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
        let outcome = step(ctx, c, update_all, opts.force, sink);
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

fn step(ctx: &Ctx, c: &Component, update_all: bool, force: bool, sink: &mut dyn Sink) -> Outcome {
    let installed = c.is_installed(ctx.kit);
    let current = c.installed_version(ctx.kit);
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
        let release = installer.latest(ctx, c)?;
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
mod tests {
    use super::*;
    use crate::env::KitEnv;
    use crate::kit::{tests::TempDir, Arch, Kit};
    use crate::net::Net;
    use crate::settings::Settings;
    use std::fs;
    use std::sync::atomic::AtomicBool;

    fn with_ctx(name: &str, f: impl FnOnce(&Ctx)) {
        with_arch_ctx(name, Arch::X64, f);
    }

    fn with_arch_ctx(name: &str, arch: Arch, f: impl FnOnce(&Ctx)) {
        let tmp = TempDir::new(name);
        let kit = Kit::at(tmp.0.clone(), arch);
        kit.prepare().unwrap();
        let net = Net::new(&Settings::default(), None).unwrap();
        let env = KitEnv::default();
        let cancel = AtomicBool::new(false);
        f(&Ctx {
            kit: &kit,
            net: &net,
            env: &env,
            cancel: &cancel,
        });
    }

    fn fake_install(ctx: &Ctx, id: &str) {
        let c = Catalog::get().find(id).unwrap();
        for key in c.key_paths(ctx.kit) {
            fs::create_dir_all(key.parent().unwrap()).unwrap();
            fs::write(&key, b"").unwrap();
        }
        c.save_version(ctx.kit, c.version.as_deref().unwrap_or("1.0"))
            .unwrap();
    }

    #[derive(Default)]
    struct Collect(Vec<Event>);
    impl Sink for Collect {
        fn emit(&mut self, e: Event) {
            self.0.push(e);
        }
    }

    #[test]
    fn plan_follows_choice_skip_and_only() {
        with_ctx("plan", |ctx| {
            let catalog = Catalog::get();
            let mut choice = Choice::defaults(catalog);
            choice.set(catalog, "go", true);
            choice.set(catalog, "php", false);
            fake_install(ctx, "php");

            let p = plan(
                ctx,
                catalog,
                &choice,
                &Options {
                    skip: vec!["codex".into()],
                    ..Options::default()
                },
            )
            .unwrap();
            assert!(p.handle.contains(&"go".into()) && p.handle.contains(&"pwsh".into()));
            assert!(!p.handle.contains(&"codex".into()), "skipped");
            assert!(
                !p.handle.contains(&"apache".into()),
                "apache follows php off"
            );
            assert_eq!(
                p.remove,
                vec!["php".to_owned()],
                "turned off and installed: removed"
            );

            let only = plan(
                ctx,
                catalog,
                &choice,
                &Options {
                    only: vec!["pwsh".into()],
                    ..Options::default()
                },
            )
            .unwrap();
            assert_eq!(only.handle, vec!["pwsh".to_owned()]);
            assert!(only.remove.is_empty(), "only never removes");

            assert!(plan(
                ctx,
                catalog,
                &choice,
                &Options {
                    only: vec!["nope".into()],
                    ..Options::default()
                }
            )
            .is_err());
        });
    }

    fn plan_with(ctx: &Ctx, choice: &Choice, only: &[&str], skip: &[&str]) -> Result<Plan> {
        let ids = |ids: &[&str]| ids.iter().map(|&id| id.to_owned()).collect();
        plan(
            ctx,
            Catalog::get(),
            choice,
            &Options {
                only: ids(only),
                skip: ids(skip),
                ..Options::default()
            },
        )
    }

    #[test]
    fn only_adds_what_is_needed_and_missing() {
        with_ctx("plan-only-needs", |ctx| {
            let choice = Choice::defaults(Catalog::get());
            let p = plan_with(ctx, &choice, &["kotlin"], &[]).unwrap();
            assert_eq!(p.handle, ["java", "kotlin"], "catalog order");
            assert!(p.remove.is_empty());
            let p = plan_with(ctx, &choice, &["apache"], &[]).unwrap();
            assert_eq!(p.handle, ["php", "apache"]);

            fake_install(ctx, "java");
            let p = plan_with(ctx, &choice, &["kotlin"], &[]).unwrap();
            assert_eq!(p.handle, ["kotlin"], "java already installed");
        });
    }

    #[test]
    fn skip_also_skips_what_needs_it() {
        with_ctx("plan-skip-needs", |ctx| {
            let catalog = Catalog::get();
            let mut choice = Choice::defaults(catalog);
            choice.set(catalog, "pnpm", true);
            assert!(choice.is_chosen("php") && choice.is_chosen("apache"));

            let p = plan_with(ctx, &choice, &[], &["php"]).unwrap();
            assert!(!p.handle.contains(&"php".into()));
            assert!(!p.handle.contains(&"apache".into()), "apache needs php");
            assert!(p.remove.is_empty(), "skipped, not removed");

            let p = plan_with(ctx, &choice, &[], &["gh"]).unwrap();
            assert!(
                p.handle.contains(&"gh".into()),
                "required ones are never skipped"
            );
            assert!(choice.is_chosen("node"), "pnpm needs node");
            let p = plan_with(ctx, &choice, &[], &["node"]).unwrap();
            assert!(
                !p.handle.contains(&"node".into()) && !p.handle.contains(&"pnpm".into()),
                "node is not required any more: skipped, and pnpm with it"
            );

            fake_install(ctx, "php");
            let p = plan_with(ctx, &choice, &[], &["php"]).unwrap();
            assert!(!p.handle.contains(&"php".into()));
            assert!(
                p.handle.contains(&"apache".into()),
                "php is there: apache can be installed"
            );
        });
    }

    #[test]
    fn half_installed_components_are_removed() {
        with_ctx("plan-half", |ctx| {
            let catalog = Catalog::get();
            let java = catalog.find("java").unwrap();
            let key = &java.key_paths(ctx.kit)[0];
            fs::create_dir_all(key.parent().unwrap()).unwrap();
            fs::write(key, b"").unwrap();
            assert!(!java.is_installed(ctx.kit) && java.is_present(ctx.kit));
            let choice = Choice::defaults(catalog);
            assert!(!choice.is_chosen("java"));
            let p = plan_with(ctx, &choice, &[], &[]).unwrap();
            assert!(p.remove.contains(&"java".into()), "maven missing, removed");
        });
    }

    #[test]
    fn no_arm64_components_are_never_planned_on_arm64() {
        let catalog = Catalog::get();
        assert!(catalog.find("r").unwrap().no_arm64);
        let mut choice = Choice::defaults(catalog);
        choice.set(catalog, "r", true);
        with_arch_ctx("plan-arm64", Arch::Arm64, |ctx| {
            let p = plan_with(ctx, &choice, &[], &[]).unwrap();
            assert!(!p.handle.contains(&"r".into()), "not from the choice");
            assert!(p.handle.contains(&"git".into()));
            let e = plan_with(ctx, &choice, &["r"], &[]).unwrap_err();
            assert_eq!(e.to_string(), "r is not available on ARM64");
        });
        with_ctx("plan-x64", |ctx| {
            let p = plan_with(ctx, &choice, &[], &[]).unwrap();
            assert!(p.handle.contains(&"r".into()), "x64: planned");
            let p = plan_with(ctx, &choice, &["r"], &[]).unwrap();
            assert_eq!(p.handle, ["r"]);
        });
    }

    /// No network: everything chosen is already installed.
    #[test]
    fn installed_components_are_left_alone_without_update_all() {
        with_ctx("run-uptodate", |ctx| {
            let catalog = Catalog::get();
            let required: Vec<String> = catalog
                .all()
                .iter()
                .filter(|c| catalog.is_locked(&c.id))
                .map(|c| c.id.clone())
                .collect();
            for id in &required {
                fake_install(ctx, id);
            }
            let opts = Options {
                skip: catalog.all().iter().map(|c| c.id.clone()).collect(),
                ..Options::default()
            };
            let mut sink = Collect::default();
            let results = run(ctx, &opts, &mut sink).unwrap();
            let mut ids: Vec<&String> = results.iter().map(|(id, _)| id).collect();
            ids.sort();
            let mut want: Vec<&String> = required.iter().collect();
            want.sort();
            assert_eq!(ids, want, "required ones are never skipped");
            assert!(results.iter().all(|(id, o)| *o
                == Outcome::UpToDate {
                    version: Some(
                        catalog
                            .find(id)
                            .and_then(|c| c.version.clone())
                            .unwrap_or_else(|| "1.0".into())
                    )
                }));
            assert!(
                ctx.kit.app_dir().join("components.txt").is_file(),
                "first run saves the choice"
            );
        });
    }

    #[test]
    fn removal_and_cancel() {
        with_ctx("run-remove", |ctx| {
            let catalog = Catalog::get();
            let mut choice = Choice::defaults(catalog);
            choice.set(catalog, "zig", false);
            choice.write(ctx.kit, catalog).unwrap();
            fake_install(ctx, "zig");
            ctx.cancel.store(true, Ordering::Relaxed);
            let mut sink = Collect::default();
            let results = run(ctx, &Options::default(), &mut sink).unwrap();
            assert_eq!(results.first(), Some(&("zig".to_owned(), Outcome::Removed)));
            assert!(!catalog.find("zig").unwrap().is_installed(ctx.kit));
            assert!(
                results[1..].iter().all(|(_, o)| *o == Outcome::Cancelled),
                "cancelled before the next step"
            );
        });
    }

    #[test]
    fn one_run_at_a_time() {
        let first = lock("test-lock-kit").unwrap();
        assert!(lock("test-lock-kit").unwrap_err().is::<Busy>());
        drop(first);
        assert!(lock("test-lock-kit").is_ok());
    }
}
