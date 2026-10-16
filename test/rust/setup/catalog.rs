use super::*;
use crate::kit::{tests::TempDir, Arch};
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
        assert!(!(c.x64_only && c.no_arm64), "{}: x64Only or noArm64", c.id);
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
    }
    // Always installed: the kit's shell and GitHub CLI only; the rest
    // comes when something needs it (D-59).
    let required: Vec<&str> = catalog
        .all()
        .iter()
        .filter(|c| c.group == Group::Required)
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(required, ["gh", "pwsh"]);
    for id in ["git", "node", "uv"] {
        let c = catalog.find(id).unwrap();
        assert!(c.group == Group::Languages && !c.def, "{id}");
    }
    assert_eq!(catalog.find("claude").unwrap().needs, ["git"]);
    // Only the parts of the app have a fixed version, and all of them do.
    for c in catalog.all() {
        let app = c.group == Group::App;
        assert_eq!(c.version.is_some(), app, "{}: version", c.id);
        assert_eq!(c.sha256.is_some(), app, "{}: sha256", c.id);
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
    let tmp = TempDir::new("catalog");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
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
    let tmp = TempDir::new("catalog-parts");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
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

#[test]
fn editor_installed_means_the_files_started() {
    let tmp = TempDir::new("catalog-editor");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let dir = crate::editor::server_dir(&kit);
    touch(&dir.join(r"bin\codium-server.cmd"));
    assert!(!crate::editor::is_installed(&kit));
    touch(&dir.join(r"out\server-main.js"));
    assert!(!crate::editor::is_installed(&kit), "node.exe missing");
    touch(&dir.join("node.exe"));
    assert!(crate::editor::is_installed(&kit));
    assert!(Catalog::get()
        .find("vscodium-server")
        .unwrap()
        .is_installed(&kit));
}

#[test]
fn shared_paths_go_with_the_last_owner() {
    let tmp = TempDir::new("catalog-shared");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let catalog = Catalog::get();
    let (php, apache) = (
        catalog.find("php").unwrap(),
        catalog.find("apache").unwrap(),
    );
    let vc = kit.apps().join(r"tools\vcruntime");
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

/// Option C (BUG-23b, BUG-24a): PHP, Apache and Nginx are Windows only
/// (Windows builds, php.ini and httpd.conf of Windows), so Adminer, which
/// needs PHP, is too. Git stays everywhere: Claude Code needs it.
#[test]
fn web_stack_is_windows_only() {
    use crate::kit::Target;

    let catalog = Catalog::get();
    for id in ["php", "apache", "nginx", "adminer"] {
        let c = catalog.find(id).unwrap();
        for target in [Target::WIN32_X64, Target::WIN32_ARM64] {
            assert!(c.is_available_on(target), "{id} on {target}");
        }
        for target in [
            Target::DARWIN_ARM64,
            Target::DARWIN_X64,
            Target::LINUX_X64,
            Target::LINUX_ARM64,
        ] {
            assert!(!c.is_available_on(target), "{id} on {target}");
        }
    }
    for id in ["git", "claude"] {
        let c = catalog.find(id).unwrap();
        for target in Target::ALL {
            assert!(c.is_available_on(target), "{id} on {target}");
        }
    }
}

/// Asking for Adminer on Linux names the system, not ARM64.
#[test]
fn adminer_on_linux_names_the_system() {
    use crate::kit::Target;
    use crate::setup::runner::{plan, Options};

    let tmp = TempDir::new("catalog-adminer-linux");
    let kit = Kit::at_target(tmp.0.clone(), Target::LINUX_X64);
    kit.prepare().unwrap();
    let net = crate::net::Net::new(&crate::settings::Settings::default(), None).unwrap();
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
        only: vec!["adminer".into()],
        ..Options::default()
    };
    let e = plan(&ctx, catalog, &Choice::defaults(catalog), &opts)
        .err()
        .unwrap()
        .to_string();
    assert!(e.contains("linux-x64") && !e.contains("ARM64"), "{e}");
    let msg = crate::commands::unavailable_msg("en", "Adminer", Target::LINUX_X64);
    assert_eq!(msg, "Adminer is not available for linux-x64.");
}

#[test]
fn multi_platform_availability_and_paths_across_targets() {
    use crate::kit::Target;

    let catalog = Catalog::get();
    let php = catalog.find("php").unwrap();
    assert!(php.is_available_on(Target::WIN32_X64));
    assert!(php.is_available_on(Target::WIN32_ARM64));
    assert!(!php.is_available_on(Target::DARWIN_ARM64));
    assert!(!php.is_available_on(Target::LINUX_X64));

    let tex = catalog.find("tinytex").unwrap();
    assert!(tex.is_emulated_on(Target::WIN32_ARM64));
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
    assert!(!r.is_available_on(Target::WIN32_ARM64));
    assert!(r.is_available_on(Target::DARWIN_ARM64));
    assert_eq!(r.key_for(Target::LINUX_X64), "r/bin/R");

    let php = catalog.find("php").unwrap();
    assert!(php.is_emulated_on(Target::WIN32_ARM64));
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
    assert!(vsc.sha256_for(Target::WIN32_ARM64).is_some());

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
