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

/// R is planned on Windows ARM64 too, its x64 build in emulation (it was
/// `noArm64`, never planned there); no component of the catalog is
/// `noArm64` any more.
#[test]
fn r_is_planned_on_arm64() {
    let catalog = Catalog::get();
    assert!(catalog.all().iter().all(|c| !c.no_arm64));
    assert!(catalog
        .find("r")
        .unwrap()
        .is_emulated_on(crate::kit::Target::WIN32_ARM64));
    let mut choice = Choice::defaults(catalog);
    choice.set(catalog, "r", true);
    with_arch_ctx("plan-arm64", Arch::Arm64, |ctx| {
        let p = plan_with(ctx, &choice, &[], &[]).unwrap();
        assert!(p.handle.contains(&"r".into()), "ARM64: planned");
        assert!(p.handle.contains(&"git".into()));
        let p = plan_with(ctx, &choice, &["r"], &[]).unwrap();
        assert_eq!(p.handle, ["r"]);
    });
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
            // Apache came there (D-116).
            assert_eq!(
                missing.is_empty(),
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
/// Linux (first, in the order of the page), `--only rust` brings C and C++
/// on Windows ARM64; a chosen C and C++ whose Zig is skipped is not planned
/// there. Windows x64 as before.
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
            let want: &[&str] = if target == Target::WIN32_ARM64 {
                &["cc", "rust"]
            } else {
                &["rust"]
            };
            assert_eq!(p.handle, want, "{target}");
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
            // Rust on Windows ARM64 brings C and C++ (and nothing else).
            let rust = Choice::parse("rust=1\ncc=0\n", catalog);
            let p = plan_with(ctx, &rust, &[], &[]).unwrap();
            assert_eq!(
                p.handle.contains(&"cc".into()),
                target == Target::WIN32_ARM64,
                "{target}"
            );
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
    let tmp = TempDir::new("runner-lock");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let first = lock(&kit).unwrap();
    assert!(lock(&kit).unwrap_err().is::<Busy>());
    // The same root for another target is the same kit.
    let other = kit.for_target(crate::kit::Target::LINUX_ARM64);
    assert!(lock(&other).unwrap_err().is::<Busy>());
    drop(first);
    assert!(lock(&kit).is_ok());
}

/// macOS and Linux: another process holding `data/app/prepare.lock` makes
/// this one busy (D-105); its id is not left in the set.
#[cfg(unix)]
#[test]
fn another_process_holds_the_lock() {
    use std::os::fd::AsRawFd;
    let tmp = TempDir::new("runner-flock");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
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
    let kit = Kit::at(root.clone(), Arch::X64);
    let result = lock(&kit);
    match std::env::var("ATOLLO_TEST_PREPARE_LOCK_EXPECT").as_deref() {
        Ok("busy") => assert!(result.unwrap_err().is::<Busy>()),
        _ => assert!(result.is_ok()),
    }
    fs::write(root.join("child-ran"), b"").unwrap();
}
