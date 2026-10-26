use super::*;
use crate::kit::{tests::TempDir, Os};
use crate::test_support::prepared_kit;

#[test]
fn windows_runtimes_only_on_windows() {
    for id in ["vcruntime", "webview2"] {
        assert!(checks::runtime_on(id, Os::Win32), "{id}");
        assert!(!checks::runtime_on(id, Os::Linux), "{id}");
        assert!(!checks::runtime_on(id, Os::Darwin), "{id}");
    }
    assert!(checks::runtime_on("something-else", Os::Linux));
    // Every runtime of the knowledge file is named there.
    for r in &known().runtimes {
        assert!(
            ["vcruntime", "webview2"].contains(&r.id.as_str()),
            "{}: say on which systems it exists (checks::runtime_on)",
            r.id
        );
    }
}

/// A project on `>=3.12.4` with the kit's 3.12.3: the full version says no,
/// whatever later patches of the line would say; the line alone decides
/// only when the version is unknown.
#[test]
fn a_known_version_outside_the_range_is_not_admitted() {
    let range = Range::parse(">=3.12.4", super::super::ranges::Bare::Exact).unwrap();
    let have = |version: Option<&str>| Have {
        installed: true,
        lines: vec![("3.12".into(), version.map(str::to_owned))],
        kit: Some("3.12".into()),
        offered: Vec::new(),
    };
    assert!(!admits(&range, &have(Some("3.12.3")), "3.12"));
    assert!(admits(&range, &have(Some("3.12.7")), "3.12"));
    assert!(admits(&range, &have(None), "3.12"), "unknown: any patch");
}

/// `id` installed at `version`: its key files, in the folder of its line.
fn fake(kit: &Kit, id: &str, version: &str) {
    let c = Catalog::get().find(id).unwrap();
    c.save_version(kit, version).unwrap();
    for k in c.key_paths(kit) {
        fs::create_dir_all(k.parent().unwrap()).unwrap();
        fs::write(&k, b"").unwrap();
    }
}

#[test]
fn links_and_projects() {
    let (_tmp, kit) = prepared_kit("relations");
    fake(&kit, "java", "11.0.25+9 (Maven 3.9.11, Gradle 9.1.0)");
    fake(&kit, "node", "v20.19.5");
    let node = Catalog::get().find("node").unwrap();
    let saved = crate::versions::saved_dir(&kit, node, "22.21");
    crate::versions::tests::install_fake(&kit, node, &saved);
    crate::versions::write_version(&saved, "v22.21.1").unwrap();
    crate::versions::save_offered(
        &kit,
        "node",
        &["24.11".into(), "22.21".into(), "20.19".into()],
    )
    .unwrap();

    let app = kit.projects().join("app");
    fs::create_dir_all(&app).unwrap();
    fs::write(app.join("package.json"), r#"{"engines": {"node": ">=22"}}"#).unwrap();
    fs::write(app.join(".nvmrc"), "24\n").unwrap();
    fs::write(app.join("go.mod"), "module x\n\ngo 1.22.1\n").unwrap();
    // The project was made with Node.js 20.19 (the kit now uses 22.21).
    crate::versions::set_project_choice(&kit, "app", "node", Some("20.19")).unwrap();

    let m = build(&kit, "it");
    let gradle = m.links.iter().find(|l| l.from == "java/gradle").unwrap();
    assert_eq!(
        gradle.status,
        LinkStatus::Incompatible,
        "Gradle 9 on Java 11"
    );
    assert!(gradle.outside.is_empty(), "the kit's JDK only counts");
    let kotlin = m.links.iter().find(|l| l.from == "kotlin").unwrap();
    assert_eq!(kotlin.status, LinkStatus::Inactive);
    let pnpm = m.links.iter().find(|l| l.from == "pnpm").unwrap();
    assert_eq!(pnpm.status, LinkStatus::Inactive);

    let needs = &m
        .projects
        .iter()
        .find(|p| p.project == "app")
        .unwrap()
        .needs;
    let engines = needs
        .iter()
        .find(|n| n.source == "package.json engines.node")
        .unwrap();
    assert_eq!(
        (engines.status, engines.line.as_deref()),
        (NeedStatus::Choose, Some("22.21"))
    );
    let nvmrc = needs.iter().find(|n| n.source == ".nvmrc").unwrap();
    assert_eq!(
        (nvmrc.status, nvmrc.line.as_deref()),
        (NeedStatus::Download, Some("24.11"))
    );
    let go = needs.iter().find(|n| n.component == "go").unwrap();
    assert_eq!(go.status, NeedStatus::NotInstalled);

    // A project just cloned: what it declares is chosen, the first
    // declaration of a component wins, nothing to download here.
    let cloned = kit.projects().join("cloned");
    fs::create_dir_all(&cloned).unwrap();
    fs::write(
        cloned.join("package.json"),
        r#"{"engines": {"node": "^20.11"}}"#,
    )
    .unwrap();
    fs::write(cloned.join(".nvmrc"), "24\n").unwrap();
    assert!(settle(&kit, "it", "cloned").is_empty());
    assert_eq!(
        crate::versions::project_choice(&kit, "cloned")
            .get("node")
            .map(String::as_str),
        Some("20.19")
    );
    fs::write(cloned.join("go.mod"), "module x\n").unwrap();
    let cloned2 = kit.projects().join("cloned2");
    fs::create_dir_all(&cloned2).unwrap();
    fs::write(cloned2.join(".nvmrc"), "24\n").unwrap();
    assert_eq!(
        settle(&kit, "it", "cloned2"),
        [("node".to_owned(), "24.11".to_owned())]
    );

    // The project chooses 22.21: engines fit.
    crate::versions::set_project_choice(&kit, "app", "node", Some("22.21")).unwrap();
    let m2 = matrix(&kit, "it", false).unwrap();
    let engines = m2.projects[0]
        .needs
        .iter()
        .find(|n| n.source == "package.json engines.node")
        .unwrap();
    assert_eq!(engines.status, NeedStatus::Ok);
    // Saved, and the same while nothing changes.
    let again = matrix(&kit, "it", false).unwrap();
    assert_eq!(again.generated, m2.generated);
    assert_eq!(again.fingerprint, m2.fingerprint);
    // `tool-versions.json` is no one's file since D-80: not in the fingerprint.
    fs::write(kit.app_dir().join("tool-versions.json"), "{}").unwrap();
    assert_eq!(fingerprint(&kit, "it"), m2.fingerprint);
    fs::write(app.join(".nvmrc"), "22.21.0\n").unwrap();
    assert_ne!(
        fingerprint(&kit, "it"),
        m2.fingerprint,
        "a project file changed"
    );
}

/// The matrix of a target chosen on the Tools page (D-121): only the links
/// of components that target can have, its own name, its own file.
#[test]
fn matrix_follows_the_target() {
    let tmp = TempDir::new("relations-target");
    let catalog = Catalog::get();
    let mut counts = Vec::new();
    for target in crate::kit::Target::ALL {
        let kit = Kit::at_target(tmp.0.clone(), target);
        kit.prepare().unwrap();
        let m = build(&kit, "en");
        assert_eq!(m.platform, target.as_str());
        for l in &m.links {
            for id in [split(&l.from).0, l.to.as_str()] {
                let c = catalog.find(id).unwrap();
                assert!(
                    c.is_available_on(target),
                    "{} > {} on {target}",
                    l.from,
                    l.to
                );
            }
        }
        let name = file(&kit)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        if target == crate::kit::Target::detect() {
            assert_eq!(name, "relations.json");
        } else {
            assert_eq!(name, format!("relations-{target}.json"));
        }
        counts.push(m.links.len());
    }
    // The same links everywhere: Android, the only component missing on
    // some system (Linux ARM64), has none; Apache and R had some before
    // they came to every target (D-116).
    assert!(counts.iter().all(|n| *n == counts[0]), "{counts:?}");
}
