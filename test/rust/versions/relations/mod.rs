use super::*;
use crate::kit::{tests::TempDir, Arch, Os};

#[test]
fn windows_runtimes_only_on_windows() {
    for id in ["vcruntime", "webview2", "x64-emulation"] {
        assert!(checks::runtime_on(id, Os::Win32), "{id}");
        assert!(!checks::runtime_on(id, Os::Linux), "{id}");
        assert!(!checks::runtime_on(id, Os::Darwin), "{id}");
    }
    assert!(checks::runtime_on("something-else", Os::Linux));
    // Every runtime of the knowledge file is named there.
    for r in &known().runtimes {
        assert!(
            ["vcruntime", "webview2", "x64-emulation"].contains(&r.id.as_str()),
            "{}: say on which systems it exists (checks::runtime_on)",
            r.id
        );
    }
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
    let tmp = TempDir::new("relations");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
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
    fs::write(app.join(".nvmrc"), "22.21.0\n").unwrap();
    assert_ne!(
        fingerprint(&kit, "it"),
        m2.fingerprint,
        "a project file changed"
    );
}
