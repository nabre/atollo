use super::super::status;
use super::*;
use crate::kit::Target;
use crate::settings::Settings;
use crate::setup::Catalog;

struct Print;

impl crate::setup::runner::Sink for Print {
    fn emit(&mut self, event: crate::setup::runner::Event) {
        use crate::setup::runner::Event;
        match event {
            Event::Note { id, text } => eprintln!("  {id}: {text}"),
            Event::Done { id, outcome } => eprintln!("DONE {id}: {outcome:?}"),
            _ => {}
        }
    }
}

/// The program of `c` that answers, relative to its default folder.
fn answers(kit: &Kit, c: &Component) -> String {
    let dir = crate::versions::default_dir(kit, c);
    let (exe, arg) = match c.id.as_str() {
        "r" => ("bin/R", "--version"),
        "ruby" => ("bin/ruby", "-v"),
        "mongodb" => ("bin/mongod", "--version"),
        "sqlite" => ("sqlite3", "--version"),
        "cc" => ("make/bin/make", "--version"),
        other => panic!("{other}"),
    };
    let out = std::process::Command::new(crate::fsx::join_rel(&dir, exe))
        .arg(arg)
        .output()
        .unwrap_or_else(|e| panic!("{}: {e}", c.id));
    assert!(out.status.success(), "{} {exe} {arg}: {out:?}", c.id);
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.lines().next().unwrap_or_default().to_owned()
}

/// Files under `dir`, by relative path and size: the data that a
/// reinstallation must leave alone.
fn listing(dir: &Path) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(m) = fs::metadata(&p) {
                let rel = p.strip_prefix(dir).unwrap().to_string_lossy().into_owned();
                // MongoDB rewrites its diagnostics while running: not data.
                if !rel.contains("diagnostic.data") && !rel.ends_with(".lock") {
                    out.push((rel, m.len()));
                }
            }
        }
    }
    out.sort();
    out
}

/// D-118 with the real environments of the kit given by ATOLLO_TEST_KIT
/// (prepared with `prepare --only r,ruby,mongodb,sqlite`, plus `cc` where
/// it is conda): the kit folder is moved, the start marks them, a
/// preparation through a proxy that does not exist (and, run under
/// `unshare -rn`, without any network) creates them again from the kept
/// packages; the programs answer from the new folder, gems and MongoDB's
/// data are untouched. Then the same back to the first folder.
#[test]
fn real_offline_reinstall_after_a_move() {
    let Ok(first) = std::env::var("ATOLLO_TEST_KIT") else {
        eprintln!("ATOLLO_TEST_KIT not set: skipped");
        return;
    };
    let first = PathBuf::from(first);
    // No space in the new name: kept from before `plain_r_sed`, when
    // conda-forge's `bin/R` broke in any folder with a space.
    let moved = first.with_file_name(format!(
        "{}-moved-d118",
        first.file_name().unwrap().to_string_lossy()
    ));
    let target = Target::detect();
    let kit = Kit::at_target(first.clone(), target);
    let ids: Vec<String> = Catalog::get()
        .all()
        .iter()
        .filter(|c| c.is_installed(&kit))
        .filter(|c| installer(&c.id).is_some_and(|i| !i.relocatable(target)))
        .map(|c| c.id.clone())
        .collect();
    // CI's Windows kit has none: R follows the kit there (D-92).
    if ids.is_empty() {
        eprintln!("no conda environment installed in ATOLLO_TEST_KIT: skipped");
        return;
    }
    let lists = status(&kit).lists;
    for id in &ids {
        assert!(
            lists.iter().any(|l| l.starts_with(&format!("{id}@"))),
            "{id}: no kept packages, install it again with this version of the app"
        );
    }
    eprintln!(
        "conda environments: {ids:?}, kept: {} MB",
        status(&kit).bytes >> 20
    );
    let data = |root: &Path| {
        let d = root.join("data");
        (
            listing(&d.join("ruby-gems")),
            listing(&d.join("db").join("mongodb")),
        )
    };
    let before = data(&first);
    let mut settings = Settings::default();
    settings.network.https_proxy = Some("http://127.0.0.1:9".into());
    settings.network.http_proxy = Some("http://127.0.0.1:9".into());
    let net = crate::net::Net::new(&settings, None).unwrap();
    let env = crate::env::KitEnv::default();
    let cancel = std::sync::atomic::AtomicBool::new(false);

    for (from, to) in [(&first, &moved), (&moved, &first)] {
        fs::rename(from, to).unwrap();
        let kit = Kit::at_target(to.clone(), target);
        kit.prepare().unwrap();
        for id in &ids {
            let c = Catalog::get().find(id).unwrap();
            assert!(c.needs_reinstall(&kit), "{id}: not marked after the move");
        }
        let ctx = Ctx {
            kit: &kit,
            net: &net,
            env: &env,
            cancel: &cancel,
        };
        let started = std::time::Instant::now();
        let opts = crate::setup::runner::Options {
            only: ids.clone(),
            ..Default::default()
        };
        let results = crate::setup::runner::run(&ctx, &opts, &mut Print).unwrap();
        eprintln!(
            "moved to {}: {} s",
            to.display(),
            started.elapsed().as_secs()
        );
        for id in &ids {
            let c = Catalog::get().find(id).unwrap();
            let outcome = results.iter().find(|(i, _)| i == id).map(|(_, o)| o);
            assert!(
                matches!(
                    outcome,
                    Some(crate::setup::runner::Outcome::Installed { .. })
                ),
                "{id}: {outcome:?}"
            );
            assert!(!c.needs_reinstall(&kit), "{id}: still marked");
            eprintln!("{id}: {}", answers(&kit, c));
        }
        if ids.iter().any(|i| i == "r") {
            let r = Catalog::get().find("r").unwrap();
            let home = std::process::Command::new(
                crate::versions::default_dir(&kit, r).join("bin").join("R"),
            )
            .arg("RHOME")
            .output()
            .unwrap();
            let home = String::from_utf8_lossy(&home.stdout).trim().to_owned();
            assert!(
                Path::new(&home).starts_with(to),
                "R_HOME {home} is not under {}",
                to.display()
            );
        }
        assert_eq!(data(to), before, "gems or MongoDB data changed");
    }
}
