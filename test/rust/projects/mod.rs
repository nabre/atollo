use super::*;
use crate::kit::{tests::TempDir, Arch};

/// A junction in projects\ is not a project, and a junction inside a
/// project is not read by the detection (D-77).
#[test]
fn junctions_are_not_projects_nor_read() {
    let tmp = TempDir::new("proj-links");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let real = kit.projects().join("real");
    fs::create_dir_all(&real).unwrap();
    fs::write(real.join("index.html"), "<h1>x</h1>").unwrap();
    let outside = tmp.0.join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(
        outside.join("package.json"),
        r#"{"dependencies":{"mongoose":"8"}}"#,
    )
    .unwrap();
    fs::write(outside.join(".env"), "DB_CONNECTION=mysql\n").unwrap();
    crate::os::safe_path::tests::junction(&kit.projects().join("linked"), &outside);
    let names: Vec<String> = folders(&kit)
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["real"]);
    // Inside a project: a junction named like a manifest folder.
    crate::os::safe_path::tests::junction(&real.join("server"), &outside);
    let req = crate::detect::requires(&real);
    assert!(
        !req.iter().any(|r| r == "mongodb" || r == "mysql"),
        "{req:?}"
    );
    let p = crate::detect::detect(&real, "it");
    assert!(!p.reason.contains("server"), "{}", p.reason);
    // The detection of the linked folder itself finds nothing.
    assert!(crate::detect::requires(&kit.projects().join("linked")).is_empty());
}

#[test]
fn git_state_from_files() {
    let tmp = TempDir::new("gitinfo");
    let d = tmp.0.join("p");
    let g = d.join(".git");
    let heads = g.join("refs").join("heads");
    let remotes = g.join("refs").join("remotes").join("origin");
    fs::create_dir_all(&heads).unwrap();
    fs::create_dir_all(&remotes).unwrap();
    fs::write(g.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    // A command in the repository's config must never run: it is only read.
    fs::write(
        g.join("config"),
        "[core]\n\tfsmonitor = calc.exe\n[remote \"origin\"]\n\turl = https://github.com/a/b.git\n\
             [branch \"main\"]\n\tremote = origin\n\tmerge = refs/heads/main\n",
    )
    .unwrap();
    fs::write(heads.join("main"), "aaa\n").unwrap();
    fs::write(
        g.join("packed-refs"),
        "# pack-refs\nbbb refs/remotes/origin/main\n",
    )
    .unwrap();
    let info = git_info(&d, &tmp.0).unwrap();
    assert_eq!(info.branch.as_deref(), Some("main"));
    assert_eq!(info.upstream.as_deref(), Some("origin/main"));
    assert_eq!(info.remote.as_deref(), Some("https://github.com/a/b.git"));
    assert_eq!(info.synced, Some(false));
    fs::write(remotes.join("main"), "aaa\n").unwrap();
    assert_eq!(git_info(&d, &tmp.0).unwrap().synced, Some(true));
    fs::write(g.join("HEAD"), "0123abcd\n").unwrap();
    assert_eq!(git_info(&d, &tmp.0).unwrap().branch, None);
    assert!(git_info(&tmp.0, &tmp.0).is_none(), "not a repository");
    // A .git file may point only inside the projects folder, never to a share.
    let wt = tmp.0.join("wt");
    fs::create_dir_all(&wt).unwrap();
    fs::write(
        wt.join(".git"),
        "gitdir: ../p/.git
",
    )
    .unwrap();
    assert!(git_info(&wt, &tmp.0).is_some(), "inside: followed");
    for bad in [
        r"gitdir: \\attacker\share\x",
        "gitdir: //attacker/share/x",
        r"gitdir: \\?\C:\Windows",
        "gitdir: ../../../Windows",
        "gitdir: C:x",
        "gitdir: ../p/.git:stream",
    ] {
        fs::write(wt.join(".git"), bad).unwrap();
        assert!(git_info(&wt, &tmp.0).is_none(), "{bad}");
    }
    // A junction (no admin rights needed) is never followed, whatever
    // it points to: here the real repository of `p`.
    let linked = tmp.0.join("linked");
    fs::create_dir_all(&linked).unwrap();
    crate::os::safe_path::tests::junction(&linked.join(".git"), &g);
    assert!(git_dir(&linked, &tmp.0).is_none(), "junction .git refused");
    assert!(git_info(&linked, &tmp.0).is_none());
}

#[test]
fn sources() {
    assert_eq!(
        Source::parse("nabre/atollo"),
        Some(Source::GitHub("nabre/atollo".into()))
    );
    assert_eq!(
        Source::parse(" a/b.git "),
        Some(Source::GitHub("a/b".into()))
    );
    assert_eq!(
        Source::parse("https://gitlab.com/x/y.git"),
        Some(Source::Url("https://gitlab.com/x/y.git".into()))
    );
    for bad in [
        "-o/x",
        "a/-b",
        "../x",
        "http://x/y",
        "https://x/y z",
        "ssh://x",
        "a/b/c",
        "file:///c/x",
        "--upload-pack=x",
    ] {
        assert_eq!(Source::parse(bad), None, "{bad}");
    }
    assert_eq!(
        Source::parse("https://gitlab.com/x/y.git")
            .unwrap()
            .default_name(),
        "y"
    );
    assert_eq!(
        Source::parse("nabre/atollo").unwrap().default_name(),
        "atollo"
    );
}

#[test]
fn create_list_delete() {
    let tmp = TempDir::new("projects");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    create(&kit, "site", "it").unwrap();
    assert!(create(&kit, "site", "it").is_err());
    assert!(create(&kit, "..", "it").is_err());
    assert!(create(&kit, "con", "it").is_err());
    fs::write(kit.projects().join("site").join("index.html"), "x").unwrap();
    create(&kit, "empty", "it").unwrap();
    fs::create_dir_all(kit.projects().join(".hidden")).unwrap();
    let l = list(&kit, "it");
    assert_eq!(l.len(), 2);
    let site = l.iter().find(|p| p.id == "site").unwrap();
    assert!(!site.empty);
    assert_eq!(site.project.default.as_deref(), Some("static"));
    let none = FakeDbs::new(&[]);
    assert!(delete(&kit, "site", "it", &none, false).is_err());
    delete(&kit, "empty", "it", &none, false).unwrap();
    assert!(!kit.projects().join("empty").exists());
    assert!(delete(&kit, r"..\data", "it", &none, false).is_err());
    assert!(none.calls().is_empty());
}

/// The kit's servers, faked: which are installed, which drop fails, the
/// drops asked (and the servers "started" for them).
struct FakeDbs {
    installed: Vec<&'static str>,
    failing: Vec<&'static str>,
    calls: std::sync::Mutex<Vec<String>>,
}

impl FakeDbs {
    fn new(installed: &[&'static str]) -> FakeDbs {
        FakeDbs {
            installed: installed.to_vec(),
            failing: Vec::new(),
            calls: std::sync::Mutex::new(Vec::new()),
        }
    }
    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

impl ProjectDatabases for FakeDbs {
    fn installed(&self, engine: &str) -> bool {
        self.installed.contains(&engine)
    }
    fn drop_db(&self, engine: &str, database: &str) -> Result<()> {
        // as `Services::db_drop`: the server starts first
        self.calls.lock().unwrap().push(format!("start {engine}"));
        if self.failing.contains(&engine) {
            bail!("{engine} did not start");
        }
        self.calls
            .lock()
            .unwrap()
            .push(format!("drop {engine} {database}"));
        Ok(())
    }
}

/// Two projects whose databases have the same name (made before the
/// names were stored: `my-app` and `my_app` both `my_app`): deleting one
/// would drop the other's, so it is refused before any server is
/// touched; forced, no server is touched at all.
#[test]
fn delete_refused_when_another_project_has_the_same_database() {
    let tmp = TempDir::new("projects-delete-same");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    for name in ["my-app", "my_app"] {
        create(&kit, name, "it").unwrap();
        crate::db::auth::set_project_password(&kit, name, "mysql", "pw").unwrap();
    }
    crate::db::auth::set_project_password(&kit, "my-app", "mongodb", "pw").unwrap();
    assert!(delete_plan(&kit, "my_app", "it")
        .unwrap()
        .databases
        .iter()
        .all(|d| d.database == "my_app"));
    let dbs = FakeDbs::new(&["mysql", "mongodb"]);
    let e = delete(&kit, "my-app", "it", &dbs, false).unwrap_err();
    assert!(format!("{e:#}").contains("my_app"), "{e:#}");
    assert!(dbs.calls().is_empty(), "{:?}", dbs.calls());
    assert!(kit.projects().join("my-app").is_dir());
    assert!(crate::db::auth::project_password(&kit, "my-app", "mysql").is_some());

    // a server not installed is not touched: no reason to refuse
    let only_mongo = FakeDbs::new(&["mongodb"]);
    delete(&kit, "my_app", "it", &only_mongo, false).unwrap();
    assert!(only_mongo.calls().is_empty());
    // now alone with its name
    delete(&kit, "my-app", "it", &dbs, false).unwrap();
    assert_eq!(
        dbs.calls(),
        [
            "start mongodb",
            "drop mongodb my_app",
            "start mysql",
            "drop mysql my_app"
        ]
    );
}

/// Data given to a project renamed outside the app (D-89) drops the
/// databases it was created with, named after the old name; names of
/// before kept by the rule of their time (`config` on MySQL), and one
/// of the server's own never dropped.
#[test]
fn delete_drops_the_names_in_use() {
    let tmp = TempDir::new("projects-delete-names");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    // an orphan of before the names, adopted by "Nuovo"
    let old = kit
        .project_data()
        .join("Vecchio-Nome")
        .join("private")
        .join("db.json");
    fs::create_dir_all(old.parent().unwrap()).unwrap();
    fs::write(&old, r#"{"mysql":"pw","mongodb":"pw2"}"#).unwrap();
    create(&kit, "Nuovo", "it").unwrap();
    state::adopt(&kit, "Vecchio-Nome", "Nuovo").unwrap();
    let plan = delete_plan(&kit, "Nuovo", "it").unwrap();
    assert!(plan.databases.iter().all(|d| d.database == "vecchio_nome"));
    let dbs = FakeDbs::new(&["mysql", "mongodb"]);
    delete(&kit, "Nuovo", "it", &dbs, false).unwrap();
    assert_eq!(
        dbs.calls(),
        [
            "start mongodb",
            "drop mongodb vecchio_nome",
            "start mysql",
            "drop mysql vecchio_nome"
        ]
    );

    // `config` of before: its own database on MySQL; on MongoDB the
    // server's, so nothing is dropped and the user is told
    create(&kit, "config", "it").unwrap();
    crate::db::auth::set_project_password(&kit, "config", "mysql", "pw").unwrap();
    let dbs = FakeDbs::new(&["mysql", "mongodb"]);
    assert_eq!(
        delete_plan(&kit, "config", "it").unwrap().databases[0].database,
        "config"
    );
    crate::db::auth::set_project_password(&kit, "config", "mongodb", "pw").unwrap();
    assert!(delete_plan(&kit, "config", "it").is_err());
    let e = delete(&kit, "config", "it", &dbs, false).unwrap_err();
    assert!(format!("{e:#}").contains("by hand"), "{e:#}");
    assert!(dbs.calls().is_empty());
    assert!(kit.projects().join("config").is_dir());
    delete(&kit, "config", "it", &dbs, true).unwrap();
    assert!(dbs.calls().is_empty());
    assert!(!kit.projects().join("config").exists());
}

/// A project with an empty folder, kit files, two databases and a copy.
fn project_with_dbs(kit: &Kit, name: &str) -> state::ProjectState {
    create(kit, name, "it").unwrap();
    settings::update(kit, name, |s| s.agent = Some("claude".into()));
    crate::db::auth::set_project_password(kit, name, "mysql", "pw-sql").unwrap();
    crate::db::auth::set_project_password(kit, name, "mongodb", "pw-mongo").unwrap();
    let state = state::ProjectState::of(kit, name).unwrap();
    fs::write(state.backups_dir().unwrap().join("x-1.sql"), "12345").unwrap();
    fs::write(state.backups_dir().unwrap().join("x-2.archive.gz"), "123").unwrap();
    state
}

#[test]
fn delete_plan_with_and_without_databases() {
    let tmp = TempDir::new("projects-plan");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    create(&kit, "plain", "it").unwrap();
    let plan = delete_plan(&kit, "plain", "it").unwrap();
    assert_eq!(
        plan,
        DeletePlan {
            folder_exists: true,
            folder_empty: true,
            databases: Vec::new(),
            backups: 0,
            backups_size: 0,
        }
    );
    assert!(
        !kit.project_data().join("plain").exists(),
        "the plan creates nothing"
    );
    assert!(delete_plan(&kit, "..", "it").is_err());
    let missing = delete_plan(&kit, "missing", "it").unwrap();
    assert!(!missing.folder_exists && !missing.folder_empty);

    let state = project_with_dbs(&kit, "Shop Online");
    fs::write(kit.projects().join("Shop Online").join("a.txt"), "x").unwrap();
    let plan = delete_plan(&kit, "Shop Online", "it").unwrap();
    assert!(plan.folder_exists && !plan.folder_empty);
    let engines: Vec<_> = plan.databases.iter().map(|d| d.engine.as_str()).collect();
    assert_eq!(engines, ["mongodb", "mysql"]);
    assert!(plan
        .databases
        .iter()
        .all(|d| d.database == "shop_online" && !d.installed));
    assert_eq!((plan.backups, plan.backups_size), (2, 8));
    let json = serde_json::to_string(&plan).unwrap();
    assert!(
        json.contains("\"folderEmpty\":false") && json.contains("\"backupsSize\":8"),
        "{json}"
    );
    assert!(!json.contains("pw-"), "no password in the plan: {json}");

    // passwords that cannot be read: no plan (only a forced delete)
    fs::write(state.db_file().unwrap(), "{ not json").unwrap();
    assert!(delete_plan(&kit, "Shop Online", "it").is_err());
}

#[test]
fn delete_drops_the_databases_first() {
    let tmp = TempDir::new("projects-delete-dbs");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let state = project_with_dbs(&kit, "shop");
    let dbs = FakeDbs::new(&["mysql", "mongodb"]);
    delete(&kit, "shop", "it", &dbs, false).unwrap();
    assert_eq!(
        dbs.calls(),
        [
            "start mongodb",
            "drop mongodb shop",
            "start mysql",
            "drop mysql shop"
        ]
    );
    assert!(!kit.projects().join("shop").exists());
    assert!(
        !state.dir().exists(),
        "nothing left in data\\projects\\shop"
    );

    // a server not installed is skipped; the rest goes
    let state = project_with_dbs(&kit, "shop");
    let dbs = FakeDbs::new(&["mongodb"]);
    delete(&kit, "shop", "it", &dbs, false).unwrap();
    assert_eq!(dbs.calls(), ["start mongodb", "drop mongodb shop"]);
    assert!(!kit.projects().join("shop").exists() && !state.dir().exists());
}

#[test]
fn an_orphan_without_a_folder_is_deleted_with_its_databases() {
    let tmp = TempDir::new("projects-delete-orphan");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let state = project_with_dbs(&kit, "gone");
    fs::remove_dir(kit.projects().join("gone")).unwrap();
    let dbs = FakeDbs::new(&["mysql", "mongodb"]);
    delete(&kit, "gone", "it", &dbs, false).unwrap();
    assert_eq!(dbs.calls().len(), 4, "{:?}", dbs.calls());
    assert!(!state.dir().exists(), "nothing left in data\\projects");
    // a link or a missing project name is still refused where it was
    assert!(delete(&kit, "..", "it", &dbs, false).is_err());
}

#[test]
fn a_failed_drop_deletes_nothing_and_force_skips_the_servers() {
    let tmp = TempDir::new("projects-delete-fail");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let state = project_with_dbs(&kit, "shop");
    let mut dbs = FakeDbs::new(&["mysql", "mongodb"]);
    dbs.failing.push("mongodb");
    let e = delete(&kit, "shop", "it", &dbs, false).unwrap_err();
    assert!(format!("{e:#}").contains("mongodb did not start"), "{e:#}");
    assert!(!format!("{e:#}").contains("pw-"), "{e:#}");
    assert!(kit.projects().join("shop").is_dir(), "folder kept");
    assert_eq!(
        crate::db::auth::project_password(&kit, "shop", "mongodb").as_deref(),
        Some("pw-mongo"),
        "passwords kept"
    );
    assert_eq!(
        fs::read_dir(state.backups_dir().unwrap()).unwrap().count(),
        2
    );

    // forced: no server touched, the folders go
    let dbs = FakeDbs::new(&["mysql", "mongodb"]);
    delete(&kit, "shop", "it", &dbs, true).unwrap();
    assert!(dbs.calls().is_empty(), "{:?}", dbs.calls());
    assert!(!kit.projects().join("shop").exists() && !state.dir().exists());

    // a non-empty folder stops before any server is touched
    let state = project_with_dbs(&kit, "shop");
    fs::write(kit.projects().join("shop").join("a.txt"), "x").unwrap();
    let dbs = FakeDbs::new(&["mysql", "mongodb"]);
    assert!(delete(&kit, "shop", "it", &dbs, false).is_err());
    assert!(delete(&kit, "shop", "it", &dbs, true).is_err());
    assert!(dbs.calls().is_empty());
    fs::remove_file(kit.projects().join("shop").join("a.txt")).unwrap();

    // unreadable passwords: an error, unless forced
    fs::write(state.db_file().unwrap(), "{ not json").unwrap();
    assert!(delete(&kit, "shop", "it", &dbs, false).is_err());
    assert!(kit.projects().join("shop").is_dir());
    delete(&kit, "shop", "it", &dbs, true).unwrap();
    assert!(dbs.calls().is_empty());
    assert!(!kit.projects().join("shop").exists() && !state.dir().exists());
}

/// A4 handoff: what a project can add follows the catalog's targets, not
/// only ARM64 (no Apache/Nginx off Windows, PHP everywhere since D-112, R
/// everywhere but Windows ARM64).
#[test]
fn addable_follows_the_platform() {
    use crate::kit::Target;
    let tmp = TempDir::new("projects-addable");
    let catalog = Catalog::get();
    let (apache, php, r, node) = (
        catalog.find("apache").unwrap(),
        catalog.find("php").unwrap(),
        catalog.find("r").unwrap(),
        catalog.find("node").unwrap(),
    );
    let on = |t: Target| Kit::at_target(tmp.0.clone(), t);
    let win = on(Target::WIN32_X64);
    assert!(addable(&win, apache) && addable(&win, r) && addable(&win, node));
    let win_arm = on(Target::WIN32_ARM64);
    assert!(addable(&win_arm, apache) && !addable(&win_arm, r));
    for t in [Target::LINUX_ARM64, Target::DARWIN_ARM64, Target::LINUX_X64] {
        let kit = on(t);
        assert!(!addable(&kit, apache), "{t:?}");
        assert!(addable(&kit, php), "{t:?}");
        assert!(addable(&kit, r) && addable(&kit, node), "{t:?}");
    }
}
