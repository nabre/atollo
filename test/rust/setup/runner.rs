use super::*;
use crate::common::settings::Settings;
use crate::env::KitEnv;
use crate::kit::{tests::TempDir, Arch, Kit};
use crate::net::Net;
use crate::test_support::prepared_kit;
use std::fs;
use std::sync::atomic::AtomicBool;

fn with_ctx(name: &str, f: impl FnOnce(&Ctx)) {
    with_arch_ctx(name, Arch::X64, f);
}

/// A Windows kit on every host: the plans below follow the catalog of
/// Windows (Apache and PHP, R without an ARM64 build), and a plan reads the
/// kit's target only.
fn with_arch_ctx(name: &str, arch: Arch, f: impl FnOnce(&Ctx)) {
    with_target_ctx(
        name,
        crate::kit::Target::new(crate::kit::Os::Win32, arch),
        f,
    );
}

/// A kit for `target` on any host: a plan is pure over the catalog and the
/// kit's target.
fn with_target_ctx(name: &str, target: crate::kit::Target, f: impl FnOnce(&Ctx)) {
    let tmp = TempDir::new(name);
    let kit = Kit::at_target(tmp.0.clone(), target);
    kit.prepare().unwrap();
    let net = Net::new(&Settings::default(), None).unwrap();
    let env = KitEnv::default();
    let cancel = AtomicBool::new(false);
    f(&Ctx {
        kit: &kit,
        net: &net,
        env: &env,
        cancel: &cancel,
        guest: None,
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

/// With the VM on (D-173): the PC's run prepares the system components and
/// the parts of the app only, leaves the chosen tools as they are, and
/// still removes what is not chosen any more; the run for the VM's Linux
/// prepares the chosen tools only, never the system components nor what
/// only finishes on its own system, and removes nothing.
#[test]
fn the_vm_splits_the_system_and_the_tools() {
    let catalog = Catalog::get();
    let mut choice = Choice::defaults(catalog);
    choice.set(catalog, "go", true);
    choice.set(catalog, "node", true);
    choice.set(catalog, "php", false);
    let scoped = |scope| Options {
        scope,
        ..Options::default()
    };
    with_ctx("plan-system", |ctx| {
        fake_install(ctx, "php");
        fake_install(ctx, "node");
        let p = plan(ctx, catalog, &choice, &scoped(Scope::System)).unwrap();
        assert!(p.handle.contains(&"pwsh".into()) && p.handle.contains(&"gh".into()));
        assert!(p.handle.contains(&"sandboxes".into()));
        assert!(!p.handle.contains(&"go".into()), "a tool goes to the VM");
        assert!(
            !p.handle.contains(&"node".into()),
            "a tool on the PC is left"
        );
        assert_eq!(p.remove, ["php"], "not chosen any more: removed");
    });
    with_target_ctx("plan-tools", crate::kit::Target::LINUX_X64, |ctx| {
        fake_install(ctx, "php");
        let p = plan(ctx, catalog, &choice, &scoped(Scope::Tools)).unwrap();
        assert!(p.handle.contains(&"go".into()) && p.handle.contains(&"node".into()));
        for system in ["pwsh", "gh", "sandboxes", "vscodium-server"] {
            assert!(!p.handle.contains(&system.into()), "{system}");
        }
        assert!(
            !p.handle.contains(&"rust".into()),
            "finishes on its own system only"
        );
        assert!(p.remove.is_empty(), "the tools' run removes nothing");
        let tools = Options {
            scope: Scope::Tools,
            ..Options::default()
        };
        assert!(remove_elsewhere(ctx.kit, catalog, &choice, &tools).is_empty());
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
        assert_eq!(p.handle, ["apache", "php"]);

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

/// R is planned on Windows, the x64 build.
#[test]
fn r_is_planned_on_windows() {
    let catalog = Catalog::get();
    let mut choice = Choice::defaults(catalog);
    choice.set(catalog, "r", true);
    with_ctx("plan-x64", |ctx| {
        let p = plan_with(ctx, &choice, &[], &[]).unwrap();
        assert!(p.handle.contains(&"r".into()), "x64: planned");
        let p = plan_with(ctx, &choice, &["r"], &[]).unwrap();
        assert_eq!(p.handle, ["r"]);
    });
}

/// The plan of a Linux or macOS kit follows that system's catalog: what has
/// no build there is never planned (asking for it is an error naming the
/// target), and a component turned off is removed through the key paths of
/// that system.
#[test]
fn unix_plans_follow_the_unix_catalog() {
    use crate::kit::Target;
    let catalog = Catalog::get();
    for target in [Target::LINUX_ARM64, Target::DARWIN_ARM64] {
        let mut choice = Choice::defaults(catalog);
        for c in catalog.all() {
            choice.set(catalog, &c.id, true);
        }
        with_target_ctx(&format!("plan-{}", target.as_str()), target, |ctx| {
            let p = plan_with(ctx, &choice, &[], &[]).unwrap();
            assert!(p.handle.contains(&"git".into()), "{}", target.as_str());
            for id in &p.handle {
                let c = catalog.find(id).unwrap();
                assert!(c.is_available_on(target), "{id} on {}", target.as_str());
            }
            let missing: Vec<_> = catalog
                .all()
                .iter()
                .filter(|c| !c.is_available_on(target))
                .collect();
            // Linux ARM64 has no Android SDK; macOS has everything since
            // Apache came there (D-116), but what is not needed there (the
            // kit's ConPTY, D-148, D-177).
            let windows_only = |c: &&Component| c.not_needed_reason(target).is_some();
            assert_eq!(
                missing.iter().all(windows_only),
                target != Target::LINUX_ARM64,
                "{}: components without a build",
                target.as_str()
            );
            for c in missing {
                assert!(!p.handle.contains(&c.id), "{} on {}", c.id, target.as_str());
                let e = plan_with(ctx, &choice, &[&c.id], &[]).unwrap_err();
                assert_eq!(
                    e.to_string(),
                    format!("{} is not available on {}", c.id, target.as_str())
                );
            }
        });
        let choice = Choice::defaults(catalog);
        assert!(!choice.is_chosen("java"));
        with_target_ctx(&format!("plan-rm-{}", target.as_str()), target, |ctx| {
            fake_install(ctx, "java");
            let java = catalog.find("java").unwrap();
            assert!(java.is_present(ctx.kit));
            let p = plan_with(ctx, &choice, &[], &[]).unwrap();
            assert!(p.remove.contains(&"java".into()), "{}", target.as_str());
        });
    }
}

/// The needs of the kit's target: `--only cc` brings Zig on macOS and
/// Linux (first, in the order of the page), `--only rust` brings nothing
/// else; a chosen C and C++ whose Zig is skipped is not planned there.
/// Windows x64 as before.
#[test]
fn plans_follow_the_needs_of_the_target() {
    use crate::kit::Target;
    for target in Target::ALL {
        let catalog = Catalog::get();
        with_target_ctx(&format!("plan-needs-{}", target.as_str()), target, |ctx| {
            let choice = Choice::defaults(catalog);
            let p = plan_with(ctx, &choice, &["cc"], &[]).unwrap();
            let want: &[&str] = if target.is_unix() {
                &["zig", "cc"]
            } else {
                &["cc"]
            };
            assert_eq!(p.handle, want, "{target}");
            let p = plan_with(ctx, &choice, &["rust"], &[]).unwrap();
            assert_eq!(p.handle, ["rust"], "{target}");
            // Installed already: not again.
            fake_install(ctx, "zig");
            let p = plan_with(ctx, &choice, &["cc"], &[]).unwrap();
            assert_eq!(p.handle, ["cc"], "{target}");
            fs::remove_dir_all(ctx.kit.apps().join("zig")).unwrap();

            // The choice of the review: C and C++ on, Zig off (saved on
            // Windows). A run here adds Zig where C and C++ need it, and
            // removes neither.
            let choice = Choice::parse("cc=1\nzig=0\n", catalog);
            assert!(choice.is_chosen("cc") && !choice.is_chosen("zig"));
            let p = plan_with(ctx, &choice, &[], &[]).unwrap();
            let unix = target.is_unix();
            assert!(p.handle.contains(&"cc".into()), "{target}");
            assert_eq!(p.handle.contains(&"zig".into()), unix, "{target}");
            assert!(!p.remove.contains(&"cc".into()), "{target}");
            // Not installed yet: a skipped Zig drops C and C++ for this run
            // only, the choice is not touched.
            let p = plan_with(ctx, &choice, &[], &["zig"]).unwrap();
            assert_eq!(p.handle.contains(&"cc".into()), !unix, "{target}");
            assert!(!p.remove.contains(&"cc".into()), "{target}");
            assert!(choice.is_chosen("cc"));
            // C and C++ skipped: its Zig is not planned either.
            let p = plan_with(ctx, &choice, &[], &["cc"]).unwrap();
            assert!(!p.handle.contains(&"zig".into()), "{target}");
            // Zig present: kept where C and C++ need it, removed elsewhere
            // as before (not chosen).
            fake_install(ctx, "zig");
            fake_install(ctx, "cc");
            let p = plan_with(ctx, &choice, &[], &[]).unwrap();
            assert_eq!(p.remove.contains(&"zig".into()), !unix, "{target}");
            assert!(!p.remove.contains(&"cc".into()), "{target}");
            // Zig skipped: C and C++ already installed stay planned, Zig is
            // not removed for this run.
            let p = plan_with(ctx, &choice, &[], &["zig"]).unwrap();
            assert!(p.handle.contains(&"cc".into()), "{target}");
            assert!(!p.handle.contains(&"zig".into()), "{target}");
            assert_eq!(p.remove.contains(&"zig".into()), !unix, "{target}");
            // Rust brings no C and C++ anywhere.
            let rust = Choice::parse("rust=1\ncc=0\n", catalog);
            let p = plan_with(ctx, &rust, &[], &[]).unwrap();
            assert!(!p.handle.contains(&"cc".into()), "{target}");
        });
    }
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
    let (_tmp, kit) = prepared_kit("runner-lock");
    let first = lock(&kit).unwrap();
    assert!(lock(&kit).unwrap_err().is::<Busy>());
    // The same root for another target is the same kit.
    let other = kit.for_target(crate::kit::Target::LINUX_ARM64);
    assert!(lock(&other).unwrap_err().is::<Busy>());
    drop(first);
    assert!(lock(&kit).is_ok());
}

/// The start's sweep in the background holds the kit's lock while it
/// runs: a run of the app waits for its end instead of finding the kit
/// busy, also when the work panics.
#[test]
fn a_run_waits_for_the_work_in_the_background() {
    use std::sync::mpsc;
    use std::time::Duration;
    let (_tmp, kit) = prepared_kit("runner-background");
    let (go_tx, go_rx) = mpsc::channel::<()>();
    let (held_tx, held_rx) = mpsc::channel();
    let probe = kit.clone();
    while_idle_in_background(&kit, move || {
        let _ = held_tx.send(lock(&probe).is_err());
        let _ = go_rx.recv_timeout(Duration::from_secs(10));
    });
    assert!(
        held_rx.recv_timeout(Duration::from_secs(10)).unwrap(),
        "under the kit's lock"
    );
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        wait_for_background();
        let _ = done_tx.send(());
    });
    // «Libera» of the Settings page waits too, instead of `Busy`.
    let (freed_tx, freed_rx) = mpsc::channel();
    let free_kit = kit.clone();
    std::thread::spawn(move || {
        let _ = freed_tx.send(while_idle(&free_kit, || 7).map_err(|e| e.to_string()));
    });
    assert!(
        done_rx.recv_timeout(Duration::from_millis(200)).is_err(),
        "a run waits"
    );
    assert!(freed_rx.try_recv().is_err(), "«Libera» waits");
    go_tx.send(()).unwrap();
    done_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(
        freed_rx.recv_timeout(Duration::from_secs(10)).unwrap(),
        Ok(7)
    );
    assert!(lock(&kit).is_ok(), "the lock let go");

    while_idle_in_background(&kit, || panic!("sweep bug"));
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        wait_for_background();
        let _ = done_tx.send(());
    });
    done_rx.recv_timeout(Duration::from_secs(10)).unwrap();
}

/// macOS and Linux: another process holding `data/app/prepare.lock` makes
/// this one busy (D-105); its id is not left in the set.
#[cfg(unix)]
#[test]
fn another_process_holds_the_lock() {
    use std::os::fd::AsRawFd;
    let (tmp, kit) = prepared_kit("runner-flock");
    // A descriptor of its own: to `flock` it is another holder, as another
    // process would be.
    let held = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(kit.app_dir().join("prepare.lock"))
        .unwrap();
    // SAFETY: a valid descriptor owned by `held`.
    assert_eq!(
        unsafe { libc::flock(held.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
        0
    );
    assert!(lock(&kit).unwrap_err().is::<Busy>());
    drop(held);
    let ok = lock(&kit).unwrap();
    drop(ok);

    // A real second process: this test binary, running only the test below.
    let marker = tmp.0.join("child-ran");
    let child = |expect: &str| {
        let _ = fs::remove_file(&marker);
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "setup::runner::tests::prepare_lock_child",
                "--ignored",
                "--test-threads=1",
            ])
            .env("ATOLLO_TEST_PREPARE_LOCK_ROOT", &tmp.0)
            .env("ATOLLO_TEST_PREPARE_LOCK_EXPECT", expect)
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "child expecting {expect}");
        assert!(marker.is_file(), "the child test did not run");
    };
    let mine = lock(&kit).unwrap();
    child("busy");
    drop(mine);
    child("free");
}

/// The other process of `another_process_holds_the_lock`; nothing when run
/// alone.
#[cfg(unix)]
#[test]
#[ignore = "run by another_process_holds_the_lock as a second process"]
fn prepare_lock_child() {
    let Some(root) = std::env::var_os("ATOLLO_TEST_PREPARE_LOCK_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let kit = Kit::at(root.clone(), crate::test_support::kit_arch());
    let result = lock(&kit);
    match std::env::var("ATOLLO_TEST_PREPARE_LOCK_EXPECT").as_deref() {
        Ok("busy") => assert!(result.unwrap_err().is::<Busy>()),
        _ => assert!(result.is_ok()),
    }
    fs::write(root.join("child-ran"), b"").unwrap();
}

/// One choice for every system: a component taken out of it goes from each
/// system that still has it, in the same run, not only from the one
/// prepared. What a chosen one needs there only (Zig for C and C++ on
/// Linux) stays.
#[test]
fn removal_reaches_every_system() {
    with_ctx("run-remove-everywhere", |ctx| {
        let catalog = Catalog::get();
        let mut choice = Choice::defaults(catalog);
        choice.set(catalog, "zig", false);
        choice.set(catalog, "typst", false);
        choice.write(ctx.kit, catalog).unwrap();
        let linux = ctx.kit.for_target(crate::kit::Target::LINUX_X64);
        let mac = ctx.kit.for_target(crate::kit::Target::DARWIN_ARM64);
        for kit in [ctx.kit, &linux, &mac] {
            let there = Ctx { kit, ..*ctx };
            fake_install(&there, "typst");
        }
        fake_install(
            &Ctx {
                kit: &linux,
                ..*ctx
            },
            "zig",
        );
        let typst = catalog.find("typst").unwrap();
        let zig = catalog.find("zig").unwrap();

        let elsewhere = remove_elsewhere(ctx.kit, catalog, &choice, &Options::default());
        assert!(elsewhere.contains(&(crate::kit::Target::LINUX_X64, "typst".into())));
        assert!(elsewhere.contains(&(crate::kit::Target::DARWIN_ARM64, "typst".into())));
        let only = Options {
            only: vec!["git".into()],
            ..Options::default()
        };
        assert!(remove_elsewhere(ctx.kit, catalog, &choice, &only).is_empty());
        // C and C++ chosen: their Zig stays on Linux.
        assert_eq!(
            choice.is_chosen("cc"),
            !elsewhere.contains(&(crate::kit::Target::LINUX_X64, "zig".into()))
        );

        ctx.cancel.store(true, Ordering::Relaxed);
        let mut sink = Collect::default();
        run(ctx, &Options::default(), &mut sink).unwrap();
        for kit in [ctx.kit, &linux, &mac] {
            assert!(!typst.is_present(kit), "{}", kit.target());
        }
        assert_eq!(zig.is_present(&linux), choice.is_chosen("cc"));
        let names: Vec<String> = sink
            .0
            .iter()
            .filter_map(|e| match e {
                Event::Step { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect();
        assert!(
            names.contains(&format!("{} (linux-x64)", typst.name)),
            "{names:?}"
        );
    });
}

/// A component taken out of the choice whose only trace is what was
/// pre-downloaded for it (bootstrap-staged for its first start there) is
/// removed too, here and on the other systems, with its staged files, the
/// lists of its kept conda packages and the archives no other list names.
#[test]
fn removal_takes_what_was_pre_downloaded() {
    with_ctx("run-remove-staged", |ctx| {
        let catalog = Catalog::get();
        let mut choice = Choice::defaults(catalog);
        choice.set(catalog, "rust", false);
        choice.write(ctx.kit, catalog).unwrap();
        let rust = catalog.find("rust").unwrap();
        // Here: staged with its version and mark, nothing installed.
        let here = crate::router::bootstrap::staged_dir(ctx.kit, "rust", None);
        fs::create_dir_all(&here).unwrap();
        fs::write(here.join("rustup-init.exe"), b"init").unwrap();
        rust.save_version(ctx.kit, "1.91.0").unwrap();
        rust.mark_reinstall(ctx.kit, "bootstrap:win32-x64").unwrap();
        // On Linux: only the staged file, no version.
        let linux = ctx.kit.for_target(crate::kit::Target::LINUX_X64);
        let there = crate::router::bootstrap::staged_dir(&linux, "rust", None);
        fs::create_dir_all(&there).unwrap();
        fs::write(there.join("rustup-init"), b"init").unwrap();
        assert!(!rust.is_present(ctx.kit) && !rust.is_present(&linux));
        // Kept conda packages on Linux: a list of the component next to one
        // of another component.
        let conda = install::conda::root(&linux);
        let (locks, pkgs) = (conda.join("locks"), conda.join("pkgs"));
        fs::create_dir_all(&locks).unwrap();
        fs::create_dir_all(&pkgs).unwrap();
        for name in ["rust@1.91.txt", "rust@1.91.json", "r@4.5.txt", "r@4.5.json"] {
            fs::write(locks.join(name), b"").unwrap();
        }
        install::conda::forget(&linux, "rust", None);
        assert!(!locks.join("rust@1.91.txt").exists());
        assert!(!locks.join("rust@1.91.json").exists());
        assert!(
            locks.join("r@4.5.txt").is_file(),
            "another component's list stays"
        );
        fs::write(locks.join("rust@1.91.txt"), b"").unwrap();
        fs::write(pkgs.join("rust-1-0.conda"), b"x").unwrap();

        let plan = plan(ctx, catalog, &choice, &Options::default()).unwrap();
        assert!(plan.remove.contains(&"rust".to_owned()), "{plan:?}");
        let elsewhere = remove_elsewhere(ctx.kit, catalog, &choice, &Options::default());
        assert!(elsewhere.contains(&(crate::kit::Target::LINUX_X64, "rust".into())));

        ctx.cancel.store(true, Ordering::Relaxed);
        let mut sink = Collect::default();
        let results = run(ctx, &Options::default(), &mut sink).unwrap();
        assert_eq!(
            results
                .iter()
                .filter(|(id, o)| id == "rust" && *o == Outcome::Removed)
                .count(),
            2,
            "{results:?}"
        );
        for kit in [ctx.kit, &linux] {
            assert!(!kit.apps().join(".bootstrap").exists(), "{}", kit.target());
            assert!(!has_trace(kit, rust), "{}", kit.target());
        }
        assert_eq!(rust.installed_version(ctx.kit), None);
        assert!(!rust.needs_reinstall(ctx.kit));
        assert!(!locks.join("rust@1.91.txt").exists());
        assert!(!pkgs.join("rust-1-0.conda").exists(), "swept on Linux too");
    });
}

/// The files added by hand for a component removed go with it; another
/// file, or one another component waits for with the same checksum, stays.
#[test]
fn removal_takes_the_files_added_for_it() {
    with_ctx("run-remove-downloads", |ctx| {
        let catalog = Catalog::get();
        let mut choice = Choice::defaults(catalog);
        choice.set(catalog, "zig", false);
        choice.write(ctx.kit, catalog).unwrap();
        fake_install(ctx, "zig");
        let sha = |b: &[u8]| {
            crate::net::verify::Expect::Sha256(
                <sha2::Sha256 as sha2::Digest>::digest(b)
                    .iter()
                    .map(|x| format!("{x:02x}"))
                    .collect(),
            )
        };
        let downloads = crate::setup::downloads::dir(ctx.kit);
        fs::create_dir_all(&downloads).unwrap();
        fs::write(downloads.join("zig (1).zip"), b"zig").unwrap();
        fs::write(downloads.join("shared.zip"), b"shared").unwrap();
        fs::write(downloads.join("other.zip"), b"other").unwrap();
        crate::setup::downloads::want(ctx.kit, "zig", &sha(b"zig"));
        crate::setup::downloads::want(ctx.kit, "zig", &sha(b"shared"));
        crate::setup::downloads::want(ctx.kit, "uv", &sha(b"shared"));

        ctx.cancel.store(true, Ordering::Relaxed);
        let mut sink = Collect::default();
        let results = run(ctx, &Options::default(), &mut sink).unwrap();
        assert!(results.contains(&("zig".to_owned(), Outcome::Removed)));
        assert!(!downloads.join("zig (1).zip").exists());
        assert!(downloads.join("shared.zip").is_file(), "uv waits for it");
        assert!(downloads.join("other.zip").is_file());
    });
}

/// The choice with every agent chosen.
fn all_agents() -> Choice {
    let catalog = Catalog::get();
    let mut choice = Choice::defaults(catalog);
    for id in ["claude", "codex", "pi", "antigravity"] {
        choice.set(catalog, id, true);
    }
    choice
}

fn vm(vm: Option<crate::levels::VmState>) -> Options {
    Options {
        vm,
        ..Options::default()
    }
}

/// LEVELS.md §7 (I6): without the VM no agent is planned, for any system,
/// in a run of the whole choice or of `only`.
#[test]
fn agents_are_never_planned_without_the_vm() {
    let off = crate::levels::VmState::Off {
        blockers: vec![crate::isolation::Blocker::HypervisorOff],
    };
    for target in crate::kit::Target::ALL {
        with_target_ctx("plan-agents-off", target, |ctx| {
            let catalog = Catalog::get();
            for opts in [vm(None), vm(Some(off.clone()))] {
                let p = plan(ctx, catalog, &all_agents(), &opts).unwrap();
                for id in ["claude", "codex", "pi", "antigravity"] {
                    assert!(!p.handle.contains(&id.into()), "{target} {id}");
                }
                let only = Options {
                    only: vec!["pi".into()],
                    ..opts
                };
                let p = plan(ctx, catalog, &all_agents(), &only).unwrap();
                assert!(!p.handle.contains(&"pi".into()), "{target}");
            }
        });
    }
}

/// Q11: with the VM on, Pi and Antigravity only for the VM's Linux;
/// Claude Code and Codex never downloaded (the `sbx` images bring them).
#[test]
fn agents_only_for_the_vms_linux() {
    let on = Some(crate::levels::VmState::On);
    for target in crate::kit::Target::ALL {
        with_target_ctx("plan-agents-on", target, |ctx| {
            let catalog = Catalog::get();
            let tools = Options {
                scope: Scope::Tools,
                ..vm(on.clone())
            };
            for opts in [vm(on.clone()), tools] {
                let p = plan(ctx, catalog, &all_agents(), &opts).unwrap();
                for id in ["claude", "codex"] {
                    assert!(!p.handle.contains(&id.into()), "{target} {id}");
                }
                for id in ["pi", "antigravity"] {
                    let available = catalog.find(id).unwrap().is_available_on(target);
                    assert_eq!(
                        p.handle.contains(&id.into()),
                        target.is_linux() && available,
                        "{target} {id}"
                    );
                }
            }
        });
    }
}

/// An agent an earlier Atollo installed for the PC is removed by the next
/// run that removes, chosen or not; Pi's build for the VM's Linux stays
/// while the VM is off.
#[test]
fn an_agent_for_the_pc_is_removed() {
    with_ctx("plan-agents-leftover", |ctx| {
        let catalog = Catalog::get();
        fake_install(ctx, "claude");
        fake_install(ctx, "pi");
        let p = plan(ctx, catalog, &all_agents(), &vm(None)).unwrap();
        assert!(p.remove.contains(&"claude".into()) && p.remove.contains(&"pi".into()));
        let system = Options {
            scope: Scope::System,
            ..vm(Some(crate::levels::VmState::On))
        };
        let p = plan(ctx, catalog, &all_agents(), &system).unwrap();
        assert!(p.remove.contains(&"pi".into()), "with the VM on too");
        let only = Options {
            only: vec!["git".into()],
            ..Options::default()
        };
        assert!(plan(ctx, catalog, &all_agents(), &only)
            .unwrap()
            .remove
            .is_empty());
    });
    with_target_ctx("plan-agents-linux", crate::kit::Target::LINUX_X64, |ctx| {
        fake_install(ctx, "pi");
        let p = plan(ctx, Catalog::get(), &all_agents(), &vm(None)).unwrap();
        assert!(!p.remove.contains(&"pi".into()), "the VM's build stays");
    });
}
