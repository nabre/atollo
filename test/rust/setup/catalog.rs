use super::*;
use crate::kit::{tests::TempDir, Target};
use crate::test_support::prepared_kit;
use std::collections::HashSet;

#[test]
fn catalog_is_valid() {
    let catalog = Catalog::get();
    assert!(catalog.all().len() >= 29, "catalog did not load");
    let mut ids = HashSet::new();
    for c in catalog.all() {
        assert!(ids.insert(c.id.as_str()), "duplicate id {}", c.id);
        assert!(
            !c.key.is_empty() && !c.desc.is_empty() && c.home.starts_with("https://"),
            "{}",
            c.id
        );
        assert!(
            c.keys.iter().all(|k| !k.is_empty() && *k != c.key),
            "{}: keys",
            c.id
        );
        if let Some(folder) = &c.folder {
            assert!(
                c.key == *folder || c.key.starts_with(&format!("{folder}/")),
                "{}: key outside folder",
                c.id
            );
        }
    }
    for c in catalog.all() {
        for n in &c.needs {
            assert!(ids.contains(n.as_str()), "{} needs unknown {n}", c.id);
        }
        for (t, needs) in &c.needs_by_target {
            assert!(Target::parse(t).is_some(), "{}: needsByTarget {t}", c.id);
            for n in needs {
                assert!(
                    ids.contains(n.as_str()),
                    "{} needs unknown {n} on {t}",
                    c.id
                );
                assert!(!c.needs.contains(n), "{}: {n} twice", c.id);
            }
        }
    }
    // Always installed: the kit's shell, GitHub CLI and Docker Sandboxes
    // (D-167); the rest comes when something needs it (D-59).
    let required: Vec<&str> = catalog
        .all()
        .iter()
        .filter(|c| c.group == Group::Required)
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(required, ["gh", "pwsh", "sandboxes"]);
    for id in ["git", "node", "uv"] {
        let c = catalog.find(id).unwrap();
        assert!(c.group == Group::Languages && !c.def, "{id}");
    }
    assert_eq!(catalog.find("claude").unwrap().needs, ["git"]);
    // Only the parts of the app, the kit's own mongo-express build (D-147)
    // and llama.cpp's rolling builds (D-151, SHA-256 per target) have a
    // fixed version, and all of them do; mongo-express's SHA-256 is pinned
    // once its release is published.
    for c in catalog.all() {
        let app = c.group == Group::App;
        let pinned = app || c.id == "mongo-express" || c.id == "llama";
        assert_eq!(c.version.is_some(), pinned, "{}: version", c.id);
        if c.id != "mongo-express" {
            assert_eq!(c.sha256.is_some(), app, "{}: sha256", c.id);
        }
    }
    assert_eq!(
        catalog.find("vscodium-server").map(|c| c.group),
        Some(Group::App)
    );
    // Saved lines are copies of the component's own folder: languages,
    // and the database servers (each version with its data, D-81).
    for c in catalog.all().iter().filter(|c| c.versioned) {
        assert!(
            c.folder.is_some()
                && (c.group == Group::Languages
                    || crate::db::services::SERVERS.contains(&c.id.as_str())),
            "{}: versioned",
            c.id
        );
    }
    assert!(catalog.find("node").unwrap().versioned);
}

#[test]
fn choice_file_compatible_with_atollo() {
    let catalog = Catalog::get();
    // As written by nabre/atollo Save-ComponentChoice (plus a BOM and LF).
    let text =
        "\u{feff}# comment\napache=1\nphp=0\nrust=1\ngh=0\nunknown=1\nbad line\nkotlin = 1\nvscodium=1\n";
    // Rust stays on: it needs nothing on any target.
    let choice = Choice::parse(text, catalog);
    assert!(!choice.is_chosen("php"));
    assert!(!choice.is_chosen("apache"), "apache needs php");
    assert!(choice.is_chosen("rust"));
    assert!(choice.is_chosen("gh"), "required stays on");
    assert!(
        choice.is_chosen("claude") && choice.is_chosen("git"),
        "a default comes with what it needs"
    );
    assert!(
        !choice.is_chosen("kotlin"),
        "kotlin needs java, which is off by default"
    );
    assert!(choice.is_chosen("python"), "not mentioned: default");

    let rendered = choice.render(catalog);
    assert!(rendered.ends_with("\r\n") && !rendered.starts_with('\u{feff}'));
    assert!(
        !rendered.contains("unknown=") && !rendered.contains("\nvscodium="),
        "ids no longer in the catalog (VSCodium desktop) are dropped"
    );
    assert_eq!(Choice::parse(&rendered, catalog), choice);
}

#[test]
fn dependencies_follow_both_ways() {
    let catalog = Catalog::get();
    let mut choice = Choice::defaults(catalog);
    choice.set(catalog, "kotlin", true);
    assert!(choice.is_chosen("java"));
    choice.set(catalog, "java", false);
    assert!(!choice.is_chosen("kotlin"));
    choice.set(catalog, "gh", false);
    assert!(choice.is_chosen("gh"), "required cannot be turned off");
    // Git is not locked (GitHub CLI works without it, D-81): turned off,
    // what needs it (Claude Code) goes with it.
    choice.set(catalog, "git", false);
    assert!(!choice.is_chosen("git") && !choice.is_chosen("claude"));
    assert!(!catalog.is_locked("git") && catalog.is_locked("pwsh"));
    assert!(!catalog.is_locked("node"));
}

/// What a component needs on each target (D-108): C and C++ are
/// `zig cc` on macOS and Linux, Rust links with llvm-mingw on Windows
/// ARM64. Elsewhere nothing changes.
#[test]
fn needs_follow_the_target() {
    let catalog = Catalog::get();
    let needs =
        |id: &str, t: Target| -> Vec<&str> { catalog.find(id).unwrap().needs_on(t).collect() };
    let only = |ids: &[&str], t: Target| -> Vec<&str> {
        catalog.needs_only_on(ids, t).into_iter().collect()
    };
    for t in Target::ALL {
        let unix = t.is_unix();
        assert_eq!(
            needs("cc", t),
            if unix { vec!["zig"] } else { vec![] },
            "{t}"
        );
        assert!(needs("rust", t).is_empty(), "{t}");
        assert!(catalog.find("rust").unwrap().needs_list_on(t).is_empty());
        // The same everywhere for the others.
        assert_eq!(needs("kotlin", t), ["java"], "{t}");
        assert_eq!(needs("claude", t), ["git"], "{t}");
        let with: Vec<&str> = catalog.with_needs_on(&["rust"], t).into_iter().collect();
        assert_eq!(with, vec!["rust"], "{t}");
        // What a run adds to the choice: the needs of this target only,
        // never one of every target, nor the ids themselves.
        assert_eq!(
            only(&["cc"], t),
            if unix { vec!["zig"] } else { vec![] },
            "{t}"
        );
        assert_eq!(
            only(&["cc", "zig"], t),
            only(&["cc"], t),
            "a chosen Zig is still a need of C and C++ on {t}"
        );
        assert!(only(&["kotlin", "claude"], t).is_empty(), "{t}");
        assert!(only(&["zig"], t).is_empty(), "{t}");
        assert!(only(&["rust"], t).is_empty(), "{t}");
        // No cycle on any target: every chain ends.
        for c in catalog.all() {
            assert!(
                !catalog
                    .with_needs_on(&c.needs_on(t).collect::<Vec<_>>(), t)
                    .contains(c.id.as_str()),
                "{} needs itself on {t}",
                c.id
            );
        }
        // The system components need nothing per target: locked as before.
        assert_eq!(catalog.locked_on(t), catalog.locked(), "{t}");
    }
}

/// `components.txt` is one for every system the kit runs on (review of
/// D-108): a need of one target never turns a component on or off in
/// it. A Windows kit with C and C++ on and Zig off, read for Linux or
/// macOS, keeps C and C++ on; ticking C and C++ does not save Zig.
#[test]
fn choice_ignores_the_needs_of_one_target() {
    let catalog = Catalog::get();
    let read = Choice::parse("cc=1\nzig=0\nrust=1\n", catalog);
    assert!(read.is_chosen("cc") && !read.is_chosen("zig") && read.is_chosen("rust"));
    assert_eq!(Choice::parse(&read.render(catalog), catalog), read);
    let read = Choice::parse("cc=0\nrust=1\n", catalog);
    assert!(read.is_chosen("rust") && !read.is_chosen("cc"));

    let mut choice = Choice::parse("cc=0\nzig=0\nrust=0\n", catalog);
    choice.set(catalog, "cc", true);
    assert!(!choice.is_chosen("zig"), "cc does not save zig");
    choice.set(catalog, "zig", true);
    choice.set(catalog, "zig", false);
    assert!(choice.is_chosen("cc"), "zig off keeps cc");
    choice.set(catalog, "rust", true);
    choice.set(catalog, "cc", false);
    assert!(choice.is_chosen("rust"), "cc off keeps rust");
    assert!(choice.chosen_ids(catalog).contains(&"rust"));
    assert!(!choice.chosen_ids(catalog).contains(&"cc"));
    // Dependencies of every target still follow both ways.
    choice.set(catalog, "kotlin", true);
    assert!(choice.is_chosen("java"));
}

/// A kit's choice is read as written, also on the first start.
#[test]
fn read_or_defaults_reads_the_kit() {
    let tmp = TempDir::new("choice-target");
    let catalog = Catalog::get();
    for t in Target::ALL {
        let kit = Kit::at_target(tmp.0.join(t.as_str()), t);
        kit.prepare().unwrap();
        assert_eq!(
            Choice::read_or_defaults(&kit, catalog),
            Choice::defaults(catalog)
        );
        let mut choice = Choice::defaults(catalog);
        choice.set(catalog, "rust", true);
        choice.write(&kit, catalog).unwrap();
        assert_eq!(Choice::read_or_defaults(&kit, catalog), choice, "{t}");
    }
}

#[test]
fn page_order_is_by_section_then_name() {
    let catalog = Catalog::get();
    let order = catalog.in_page_order();
    assert_eq!(order.len(), catalog.all().len(), "every component once");
    let pos = |id: &str| order.iter().position(|c| c.id == id).unwrap();
    // Sections in the order of the page; inside one, by name.
    for w in order.windows(2) {
        let (a, b) = (w[0], w[1]);
        assert!(a.page_section() <= b.page_section(), "{} / {}", a.id, b.id);
        if a.page_section() == b.page_section() {
            assert!(
                a.name.to_lowercase() <= b.name.to_lowercase(),
                "{} / {}",
                a.name,
                b.name
            );
        }
    }
    // pnpm is installed with the kit's Node.js: languages come first.
    assert!(pos("node") < pos("pnpm"));
    assert!(pos("python") < pos("uv"), "languages before tools");
}

#[test]
fn installed_and_versions() {
    let (_tmp, kit) = prepared_kit("catalog");
    let catalog = Catalog::get();
    let pwsh = catalog.find("pwsh").unwrap();
    assert!(!pwsh.is_installed(&kit));
    let exe = pwsh.key_path(&kit);
    fs::create_dir_all(exe.parent().unwrap()).unwrap();
    fs::write(&exe, b"").unwrap();
    assert!(pwsh.is_installed(&kit));

    let folder: Component = serde_json::from_value(serde_json::json!({
        "id": "f", "name": "F", "group": "languages", "def": false, "mb": 1,
        "desc": "f", "home": "https://f", "key": "f"
    }))
    .unwrap();
    fs::create_dir_all(folder.key_path(&kit)).unwrap();
    assert!(!folder.is_installed(&kit), "empty folder is not installed");
    fs::write(folder.key_path(&kit).join("x"), b"").unwrap();
    assert!(folder.is_installed(&kit));

    let pnpm = catalog.find("pnpm").unwrap();
    assert!(pnpm
        .key_path(&kit)
        .starts_with(kit.data().join("npm-global")));

    pwsh.save_version(&kit, "7.6.7").unwrap();
    assert_eq!(pwsh.installed_version(&kit).as_deref(), Some("7.6.7"));
    pwsh.forget_version(&kit);
    assert_eq!(pwsh.installed_version(&kit), None);

    // Below the oldest version Atollo works with, or of unknown version:
    // installed again (Docker Sandboxes follows its latest release).
    let sbx = catalog.find("sandboxes").unwrap();
    let min = sbx.min_version.clone().expect("a minimum for sbx");
    assert!(sbx.outdated(&kit), "unknown version");
    sbx.save_version(&kit, "0.1.0").unwrap();
    assert!(sbx.outdated(&kit), "older than {min}");
    sbx.save_version(&kit, &min).unwrap();
    assert!(!sbx.outdated(&kit));
    sbx.save_version(&kit, "99.0.0").unwrap();
    assert!(!sbx.outdated(&kit), "newer");
    sbx.forget_version(&kit);

    assert!(Choice::read(&kit, catalog).is_none());
    Choice::defaults(catalog).write(&kit, catalog).unwrap();
    assert_eq!(Choice::read(&kit, catalog), Some(Choice::defaults(catalog)));
}

fn touch(path: &std::path::Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"").unwrap();
}

#[test]
fn every_part_must_be_installed() {
    let (_tmp, kit) = prepared_kit("catalog-parts");
    let catalog = Catalog::get();
    for (id, parts) in [
        ("java", &["maven/bin/mvn.cmd", "gradle/bin/gradle.bat"][..]),
        ("cmake", &["tools/ninja.exe"]),
        ("php", &["php/composer.phar"]),
        ("pnpm", &["@data/npm-global/yarn.cmd"]),
        ("vscodium-server", &["vscodium-server/node.exe"]),
    ] {
        let c = catalog.find(id).unwrap();
        assert_eq!(c.keys, parts, "{id}");
        touch(&c.key_path(&kit));
        assert!(!c.is_installed(&kit), "{id}: main file only");
        for p in &c.key_paths(&kit)[1..] {
            touch(p);
        }
        assert!(c.is_installed(&kit), "{id}: every part");
    }
    let pnpm = catalog.find("pnpm").unwrap();
    assert!(pnpm.key_paths(&kit)[1].starts_with(kit.data().join("npm-global")));
}

/// The server's layout on each target: `bin\codium-server.cmd` and
/// `node.exe` on Windows, `bin/codium-server` and `node` elsewhere.
#[test]
fn editor_installed_means_the_files_started() {
    use crate::kit::Target;

    let tmp = TempDir::new("catalog-editor");
    for target in Target::ALL {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        kit.prepare().unwrap();
        let (launcher, node) = if target.is_windows() {
            ("codium-server.cmd", "node.exe")
        } else {
            ("codium-server", "node")
        };
        let dir = crate::editor::server_dir(&kit);
        touch(&dir.join("bin").join(launcher));
        assert!(!crate::editor::is_installed(&kit), "{target}");
        touch(&dir.join("out").join("server-main.js"));
        assert!(
            !crate::editor::is_installed(&kit),
            "{target}: {node} missing"
        );
        touch(&dir.join(node));
        assert!(crate::editor::is_installed(&kit), "{target}");
        assert!(Catalog::get()
            .find("vscodium-server")
            .unwrap()
            .is_installed(&kit));
    }
}

#[test]
fn shared_paths_go_with_the_last_owner() {
    let tmp = TempDir::new("catalog-shared");
    // The shared VC++ runtime is Windows only.
    let kit = Kit::at_target(tmp.0.clone(), crate::kit::Target::WIN32_X64);
    kit.prepare().unwrap();
    let catalog = Catalog::get();
    let (php, apache) = (
        catalog.find("php").unwrap(),
        catalog.find("apache").unwrap(),
    );
    let vc = kit.apps().join("tools").join("vcruntime");
    for c in [php, apache] {
        for p in c.key_paths(&kit) {
            touch(&p);
        }
    }
    assert!(!php.owned_paths(&kit).contains(&vc), "Apache still uses it");
    assert!(!apache.owned_paths(&kit).contains(&vc), "PHP still uses it");
    assert!(php.owned_paths(&kit).contains(&kit.apps().join("php")));
    // Half of PHP still there: it still counts as an owner.
    fs::remove_file(php.key_path(&kit)).unwrap();
    assert!(
        !apache.owned_paths(&kit).contains(&vc),
        "composer.phar left"
    );
    // PHP removed (what its removal deletes, vcruntime aside).
    for p in php.owned_paths(&kit) {
        if p.is_dir() {
            fs::remove_dir_all(&p).unwrap();
        } else if p.exists() {
            fs::remove_file(&p).unwrap();
        }
    }
    assert!(apache.owned_paths(&kit).contains(&vc), "last owner");
    assert!(apache
        .owned_paths(&kit)
        .contains(&kit.apps().join("apache")));
}

/// The web stack is on every target: PHP (static-php-cli on macOS and
/// Linux, D-112), phpMyAdmin on it, Caddy (D-111) and Apache (Apache Lounge
/// on Windows, the kit's build elsewhere, D-116); Nginx is no longer in the
/// catalog (D-120). Git stays everywhere: Claude Code needs it.
#[test]
fn web_stack_is_everywhere() {
    use crate::kit::Target;

    let catalog = Catalog::get();
    for id in ["php", "phpmyadmin", "caddy", "apache"] {
        let c = catalog.find(id).unwrap();
        for target in Target::ALL {
            assert!(c.is_available_on(target), "{id} on {target}");
        }
    }
    assert!(catalog.find("nginx").is_none());
    for id in ["git", "claude"] {
        let c = catalog.find(id).unwrap();
        for target in Target::ALL {
            assert!(c.is_available_on(target), "{id} on {target}");
        }
    }
}

/// Asking on Linux for a component without a build there names the system,
/// not "ARM64" (Android on Linux ARM64; Apache before D-116, phpMyAdmin
/// before D-112).
#[test]
fn unavailable_on_linux_names_the_system() {
    use crate::kit::Target;
    use crate::setup::runner::{plan, Options};

    let tmp = TempDir::new("catalog-android-linux");
    let kit = Kit::at_target(tmp.0.clone(), Target::LINUX_ARM64);
    kit.prepare().unwrap();
    let net = crate::net::Net::new(&crate::common::settings::Settings::default(), None).unwrap();
    let env = crate::env::KitEnv::default();
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let ctx = crate::install::Ctx {
        kit: &kit,
        net: &net,
        env: &env,
        cancel: &cancel,
    };
    let catalog = Catalog::get();
    let opts = Options {
        only: vec!["android".into()],
        ..Options::default()
    };
    let e = plan(&ctx, catalog, &Choice::defaults(catalog), &opts)
        .err()
        .unwrap()
        .to_string();
    assert_eq!(e, "android is not available on linux-arm64");
    let msg = crate::ipc::unavailable_msg("en", "Android SDK", Target::LINUX_ARM64);
    assert_eq!(msg, "Android SDK is not available for linux-arm64.");
}

#[test]
fn multi_platform_availability_and_paths_across_targets() {
    use crate::kit::Target;

    let catalog = Catalog::get();
    let php = catalog.find("php").unwrap();
    assert!(php.is_available_on(Target::WIN32_X64));
    assert!(php.is_available_on(Target::DARWIN_ARM64));
    assert!(php.is_available_on(Target::LINUX_X64));
    let apache = catalog.find("apache").unwrap();
    assert!(apache.is_available_on(Target::DARWIN_ARM64));
    assert!(apache.is_available_on(Target::LINUX_X64));
    assert!(!apache.is_emulated_on(Target::LINUX_ARM64));

    let tex = catalog.find("tinytex").unwrap();
    assert!(tex.is_available_on(Target::DARWIN_ARM64));
    assert!(tex.is_available_on(Target::LINUX_ARM64));
    assert_eq!(
        tex.key_for(Target::DARWIN_ARM64),
        "tinytex/bin/universal-darwin/tlmgr"
    );
    assert_eq!(
        tex.key_for(Target::LINUX_X64),
        "tinytex/bin/x86_64-linux/tlmgr"
    );

    let r = catalog.find("r").unwrap();
    assert!(r.is_available_on(Target::WIN32_X64));
    assert!(r.is_available_on(Target::DARWIN_ARM64));
    assert!(!r.is_emulated_on(Target::DARWIN_ARM64));
    assert_eq!(r.key_for(Target::LINUX_X64), "r/bin/R");

    let php = catalog.find("php").unwrap();
    assert!(!php.is_emulated_on(Target::DARWIN_ARM64));
    assert_eq!(php.key_for(Target::DARWIN_ARM64), "php/bin/php");

    let node = catalog.find("node").unwrap();
    assert_eq!(node.key_for(Target::WIN32_X64), "node/node.exe");
    assert_eq!(node.key_for(Target::DARWIN_ARM64), "node/bin/node");
    assert_eq!(node.key_for(Target::LINUX_ARM64), "node/bin/node");

    let vsc = catalog.find("vscodium-server").unwrap();
    assert_eq!(
        vsc.keys_for(Target::DARWIN_ARM64),
        vec!["vscodium-server/node".to_string()]
    );
    assert!(vsc.sha256_for(Target::WIN32_X64).is_some());

    // Per-target version files stay isolated in data/app/versions/<target>/<id>.txt
    let tmp = TempDir::new("catalog-targets");
    let kit_win = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    let kit_mac = Kit::at_target(tmp.0.clone(), Target::DARWIN_ARM64);
    kit_win.prepare().unwrap();
    kit_mac.prepare().unwrap();
    node.save_version(&kit_win, "22.11.0").unwrap();
    node.save_version(&kit_mac, "24.1.0").unwrap();
    assert_eq!(node.installed_version(&kit_win).as_deref(), Some("22.11.0"));
    assert_eq!(node.installed_version(&kit_mac).as_deref(), Some("24.1.0"));
}

/// D-108: a target with no build says why (Android on Linux ARM64; SQLite
/// was there too until D-112), and nothing changes for the others.
#[test]
fn unavailable_targets_have_a_reason() {
    use crate::kit::Target;

    let catalog = Catalog::get();
    let sqlite = catalog.find("sqlite").unwrap();
    for t in Target::ALL {
        assert!(sqlite.is_available_on(t), "sqlite on {t}");
    }
    for id in ["android"] {
        let c = catalog.find(id).unwrap();
        assert!(!c.is_available_on(Target::LINUX_ARM64), "{id}");
        let reason = c.unavailable_reason(Target::LINUX_ARM64).unwrap();
        assert!(reason.contains("Linux ARM64"), "{id}: {reason}");
        for t in Target::ALL
            .into_iter()
            .filter(|t| *t != Target::LINUX_ARM64)
        {
            assert!(c.is_available_on(t), "{id} on {t}");
            assert_eq!(c.unavailable_reason(t), None, "{id} on {t}");
        }
    }
    // Outside `targets`: not available, no reason of its own.
    let windows: Component = serde_json::from_value(serde_json::json!({
        "id": "w", "name": "W", "group": "languages", "def": false, "mb": 1,
        "desc": "w", "home": "https://w", "key": "w/w.exe",
        "targets": ["win32-x64"]
    }))
    .unwrap();
    assert!(!windows.is_available_on(Target::LINUX_X64));
    assert!(windows.is_available_on(Target::WIN32_X64));
    assert_eq!(windows.unavailable_reason(Target::LINUX_X64), None);
    // A removed system (D-176) is never one of a component, even listed.
    let old: Component = serde_json::from_value(serde_json::json!({
        "id": "o", "name": "O", "group": "languages", "def": false, "mb": 1,
        "desc": "o", "home": "https://o", "key": "o/o.exe",
        "targets": ["win32-x64", "win32-arm64"]
    }))
    .unwrap();
    let windows_arm = Target::new(crate::kit::Os::Win32, crate::kit::Arch::Arm64);
    assert!(!old.is_available_on(windows_arm));
    assert!(old.is_available_on(Target::WIN32_X64));

    // Every key a target, every reason translated.
    let i18n: BTreeMap<String, BTreeMap<String, String>> =
        serde_json::from_str(include_str!("../../../tauri/i18n/catalog.json")).unwrap();
    for c in catalog.all() {
        for (t, reason) in &c.unavailable {
            assert!(Target::parse(t).is_some(), "{}: unknown target {t}", c.id);
            assert!(!reason.trim().is_empty(), "{}: empty reason", c.id);
            for lang in ["en", "fr", "de"] {
                assert!(
                    i18n.get(reason).and_then(|tr| tr.get(lang)).is_some(),
                    "{}: {lang} missing for {reason:?}",
                    c.id
                );
            }
        }
        for t in c
            .targets
            .iter()
            .chain(c.sha256_by_target.keys())
            .chain(c.needs_by_target.keys())
        {
            assert!(Target::parse(t).is_some(), "{}: unknown target {t}", c.id);
        }
    }
}

/// `rosetta` comes only with a reason to: no catalog entry needs it today
/// (every component has an Apple silicon build).
#[test]
fn rosetta_is_off_in_the_catalog() {
    for c in Catalog::get().all() {
        assert!(!c.rosetta, "{}: check the Apple Silicon build first", c.id);
    }
}

/// `removeUnix` follows the rules of `remove` with Unix names, and takes its
/// place on macOS and Linux only; Windows keeps `remove`.
#[test]
fn remove_unix_on_macos_and_linux() {
    use crate::kit::Target;

    let catalog = Catalog::get();
    for c in catalog.all() {
        for r in &c.remove_unix {
            let windows_name = [".exe", ".cmd", ".bat", ".ps1"]
                .iter()
                .any(|e| r.ends_with(e));
            assert!(!r.contains('\\') && !windows_name, "{}: {r}", c.id);
        }
        assert_eq!(
            c.remove_for(Target::WIN32_X64),
            c.remove.as_slice(),
            "{}",
            c.id
        );
        let unix = if c.remove_unix.is_empty() {
            &c.remove
        } else {
            &c.remove_unix
        };
        for t in [Target::DARWIN_ARM64, Target::LINUX_X64, Target::LINUX_ARM64] {
            assert_eq!(c.remove_for(t), unix.as_slice(), "{}", c.id);
        }
    }
    for (id, part) in [
        ("pnpm", "@data/npm-global/lib/node_modules/pnpm"),
        ("pnpm", "@data/npm-global/lib/node_modules/corepack"),
        ("pnpm", "@data/npm-global/bin/pnpm"),
        ("uv", "tools/uv"),
        ("cmake", "tools/ninja"),
        ("r", "tools/micromamba"),
        ("ruby", "tools/micromamba"),
        ("mongodb", "tools/micromamba"),
        ("mongodb", "mongosh"),
        ("sqlite", "tools/micromamba"),
    ] {
        let c = catalog.find(id).unwrap();
        assert!(c.remove_unix.iter().any(|r| r == part), "{id}: {part}");
    }
    // corepack comes with pnpm and yarn on Windows too: it goes with them.
    let pnpm_win = &catalog.find("pnpm").unwrap().remove;
    for part in [
        "@data/npm-global/corepack.cmd",
        "@data/npm-global/node_modules/corepack",
    ] {
        assert!(pnpm_win.iter().any(|r| r == part), "pnpm: {part}");
    }
    // Git is dugite-native off Windows (D-112): micromamba is not its own.
    let git_unix = &catalog.find("git").unwrap().remove_unix;
    assert!(!git_unix.iter().any(|r| r == "tools/micromamba"));

    // On Linux: the Unix files, and micromamba goes with the last of Ruby and R.
    let tmp = TempDir::new("catalog-remove-unix");
    let kit = Kit::at_target(tmp.0.clone(), Target::LINUX_X64);
    kit.prepare().unwrap();
    let npm_global = kit.data().join("npm-global");
    let pnpm = catalog.find("pnpm").unwrap();
    let owned = pnpm.owned_paths(&kit);
    assert!(owned.contains(&npm_global.join("bin").join("pnpm")));
    assert!(owned.contains(&npm_global.join("lib").join("node_modules").join("pnpm")));
    assert!(
        !owned.iter().any(|p| p.to_string_lossy().ends_with(".cmd")),
        "{owned:?}"
    );
    let mm = kit.apps().join("tools").join("micromamba");
    let (ruby, r) = (catalog.find("ruby").unwrap(), catalog.find("r").unwrap());
    for c in [ruby, r] {
        touch(&c.folder_path(&kit).unwrap().join("bin").join("x"));
    }
    assert!(!r.owned_paths(&kit).contains(&mm), "Ruby still uses it");
    fs::remove_dir_all(ruby.folder_path(&kit).unwrap()).unwrap();
    assert!(r.owned_paths(&kit).contains(&mm), "last owner");

    // The same kit on Windows: the Windows names, as before.
    let win = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    assert!(pnpm
        .owned_paths(&win)
        .contains(&win.data().join("npm-global").join("pnpm.cmd")));
    assert!(r
        .owned_paths(&win)
        .contains(&win.apps().join("tools").join("micromamba.exe")));
}
