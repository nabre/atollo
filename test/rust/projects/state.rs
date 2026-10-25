use super::*;
use crate::kit::{tests::TempDir, Arch};
use crate::test_support::prepared_kit;

#[test]
fn one_folder_per_project() {
    let tmp = TempDir::new("project-state");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    for bad in ["", "..", r"..\x", "a/b", "con", "x."] {
        assert!(ProjectState::of(&kit, bad).is_none(), "{bad:?}");
    }
    let s = ProjectState::of(&kit, "shop").unwrap();
    let shop = tmp.0.join("data").join("projects").join("shop");
    assert_eq!(s.dir(), &shop);
    assert_eq!(s.settings_file(), shop.join("dev-settings.json"));
    assert_eq!(s.ports_file(), shop.join("ports.json"));
    assert_eq!(s.apache_dir(), shop.join("apache"));
    let backups = s.backups_dir().unwrap();
    assert!(backups.is_dir() && backups.ends_with(Path::new("private").join("backups")));
    assert_eq!(s.db_file().unwrap(), shop.join("private").join("db.json"));
    assert_eq!(all(&kit), ["shop"]);

    // renamed with its files; never over another project's files
    fs::write(s.settings_file(), "{}").unwrap();
    let s = s.rename(&kit, "store").unwrap();
    assert!(s.settings_file().is_file());
    assert_eq!(all(&kit), ["store"]);
    ProjectState::of(&kit, "other")
        .unwrap()
        .private_dir()
        .unwrap();
    assert!(s.clone().rename(&kit, "other").is_err());
    assert!(s.clone().rename(&kit, "..").is_err());

    s.remove().unwrap();
    ProjectState::of(&kit, "never").unwrap().remove().unwrap();
    assert_eq!(all(&kit), ["other"]);
}

#[test]
fn orphans_by_files_and_adopted() {
    let (_tmp, kit) = prepared_kit("project-orphans");
    let write = |p: &str, rel: &str, text: &str| {
        let path = crate::common::fsx::join_rel(kit.project_data().join(p), rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    };
    // a project with its files; one gone with only empty folders (made
    // by reading its passwords or copies); one gone with its files
    fs::create_dir_all(kit.projects().join("app")).unwrap();
    write("app", "dev-settings.json", "{}");
    write("app", "private/db.json", r#"{"mysql":"app-pw"}"#);
    ProjectState::of(&kit, "empty")
        .unwrap()
        .backups_dir()
        .unwrap();
    write("old", "dev-settings.json", r#"{"agent":"claude"}"#);
    write(
        "old",
        "ports.json",
        r#"{"ports":{"web":8001},"locked":["web"]}"#,
    );
    write("old", "private/db.json", r#"{"mysql":"pw","future":{}}"#);
    write("old", "private/backups/old-mysql-1.sql", "x");
    assert_eq!(orphans(&kit), ["old"]);
    assert_eq!(
        holdings(&kit, "old").unwrap(),
        Holdings {
            project: "old".into(),
            settings: true,
            ports: 1,
            databases: vec!["mysql".into()],
            backups: 1,
        }
    );
    let json = serde_json::to_string(&holdings(&kit, "old").unwrap()).unwrap();
    assert!(!json.contains("\"pw\""), "no password: {json}");
    assert!(holdings(&kit, "..").is_none());

    // refused: a project with a database, a missing project, not an orphan
    let e = adopt(&kit, "it", "old", "app").unwrap_err();
    assert!(
        e.to_string().starts_with("app ha già dati suoi nel kit"),
        "{e}"
    );
    let e = adopt(&kit, "en", "old", "app").unwrap_err();
    assert!(e.to_string().starts_with("app already has data"), "{e}");
    assert!(adopt(&kit, "it", "old", "missing").is_err());
    assert!(adopt(&kit, "it", "empty", "app").is_err());
    assert!(adopt(&kit, "it", "old", "..").is_err());
    assert!(kit
        .project_data()
        .join("old")
        .join("dev-settings.json")
        .is_file());

    // renamed outside the app: its empty folders are not data
    let runner = crate::projects::run::Runner::new(kit.clone(), std::sync::Arc::new(|_| {}));
    fs::create_dir_all(kit.projects().join("shop")).unwrap();
    ProjectState::of(&kit, "shop")
        .unwrap()
        .private_dir()
        .unwrap();
    let shop = adopt(&kit, "it", "old", "shop").unwrap();
    runner.rename_project("old", "shop");
    assert_eq!(
        fs::read_to_string(shop.settings_file()).unwrap(),
        r#"{"agent":"claude"}"#
    );
    assert!(shop
        .dir()
        .join("private")
        .join("backups")
        .join("old-mysql-1.sql")
        .is_file());
    assert_eq!(
        crate::db::auth::project_password(&kit, "shop", "mysql").as_deref(),
        Some("pw")
    );
    // the name of before travels with the passwords, other keys kept
    let db_file = shop.dir().join("private").join("db.json");
    let db: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(db_file).unwrap()).unwrap();
    assert_eq!(
        db,
        serde_json::json!({ "mysql": "pw", "future": {}, "names": { "mysql": "old" } })
    );
    assert_eq!(
        crate::db::auth::project_db_name(&kit, "shop", "mysql").unwrap(),
        "old"
    );
    assert!(!kit.project_data().join("old").exists());
    assert!(orphans(&kit).is_empty());

    // the runner knows the ports under the new name: a new one is added
    // to the moved file, never written over it
    let ports = runner.ports();
    assert_eq!(ports.ports.get("shop|web"), Some(&8001));
    assert_eq!(ports.locked, ["shop|web"]);
    runner.set_port("shop|api", Some(45678), "it").unwrap();
    let text = fs::read_to_string(shop.ports_file()).unwrap();
    let saved: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(saved["ports"]["web"], 8001, "{text}");
    assert_eq!(saved["ports"]["api"], 45678, "{text}");
    assert!(!kit.project_data().join("old").exists());
}

/// A junction in place of the project's folder of `data\projects` (or
/// the orphan's) is refused before anything is read or moved; so is an
/// orphan whose `db.json` cannot be read.
#[test]
fn adopt_refuses_links_and_unreadable_passwords() {
    use crate::os::safe_path::tests::{junction, remove_junction};
    let (tmp, kit) = prepared_kit("project-adopt-links");
    let write = |p: &str, rel: &str, text: &str| {
        let path = crate::common::fsx::join_rel(kit.project_data().join(p), rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    };
    let old_db = kit
        .project_data()
        .join("old")
        .join("private")
        .join("db.json");
    write("old", "private/db.json", r#"{"mysql":"pw"}"#);
    fs::create_dir_all(kit.projects().join("shop")).unwrap();
    // data\projects\shop is a junction to a folder outside
    let outside = tmp.0.join("outside");
    fs::create_dir_all(outside.join("private")).unwrap();
    junction(&kit.project_data().join("shop"), &outside);
    let e = adopt(&kit, "it", "old", "shop").unwrap_err();
    assert!(e.to_string().contains("link"), "{e}");
    assert!(old_db.is_file());
    assert!(
        outside.join("private").is_dir(),
        "nothing cleaned through it"
    );
    remove_junction(&kit.project_data().join("shop"));

    // the orphan a junction: refused, its target untouched
    let target = tmp.0.join("target");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("dev-settings.json"), "{}").unwrap();
    junction(&kit.project_data().join("linked"), &target);
    assert!(adopt(&kit, "it", "linked", "shop").is_err());
    assert!(target.join("dev-settings.json").is_file());
    remove_junction(&kit.project_data().join("linked"));

    // unreadable passwords: nothing moves
    write("old", "private/db.json", "{ not json");
    assert!(adopt(&kit, "it", "old", "shop").is_err());
    assert!(old_db.is_file());
    assert!(!kit.project_data().join("shop").exists());
    write("old", "private/db.json", r#"{"mysql":"pw"}"#);
    adopt(&kit, "it", "old", "shop").unwrap();
    assert_eq!(
        crate::db::auth::project_db_name(&kit, "shop", "mysql").unwrap(),
        "old"
    );
}

/// A project created, cloned or opened once has its settings (and the
/// versions files made from them): they do not stop a link, the orphan's
/// replace them, keeping the target's last opening and favorite mark where
/// the orphan has none. Ports, servers, a database or copies of its own do.
#[test]
fn adopt_over_settings_only() {
    let (_tmp, kit) = prepared_kit("project-adopt-light");
    let write = |p: &str, rel: &str, text: &str| {
        let path = crate::common::fsx::join_rel(kit.project_data().join(p), rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    };
    let settings = |p: &str| crate::projects::settings::load(&kit, p);
    for p in ["shop", "busy", "served", "ported"] {
        fs::create_dir_all(kit.projects().join(p)).unwrap();
        write(
            p,
            "dev-settings.json",
            r#"{"lastOpened":42,"pinned":true,"agent":"codex","versions":{"php":"8.3"}}"#,
        );
        write(p, "versions.json", r#"{"swaps":[]}"#);
        write(p, "versions.txt", "");
    }
    write("busy", "private/db.json", r#"{"mysql":"busy-pw"}"#);
    write("served", "apache/httpd.conf", "x");
    write("ported", "ports.json", r#"{"ports":{"web":8001}}"#);
    write(
        "old",
        "dev-settings.json",
        r#"{"agent":"claude","versions":{"node":"22"}}"#,
    );
    write("old", "private/db.json", r#"{"mysql":"pw"}"#);
    write("old", "private/backups/old-mysql-1.sql", "x");

    for p in ["busy", "served", "ported"] {
        let e = adopt(&kit, "it", "old", p).unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::AlreadyExists, "{p}");
        assert!(e.to_string().contains("ha già dati suoi"), "{e}");
        assert_eq!(settings(p).last_opened, Some(42), "{p} untouched");
    }
    assert_eq!(orphans(&kit), ["old"]);

    let shop = adopt(&kit, "it", "old", "shop").unwrap();
    let s = settings("shop");
    assert_eq!(s.agent.as_deref(), Some("claude"), "the orphan's wins");
    assert_eq!(s.last_opened, Some(42), "kept: the orphan had none");
    assert!(s.pinned, "kept: the orphan had none");
    assert_eq!(
        s.versions,
        [("node".to_owned(), "22".to_owned())].into_iter().collect(),
        "the orphan's versions replace the target's"
    );
    assert!(shop
        .dir()
        .join("private")
        .join("backups")
        .join("old-mysql-1.sql")
        .is_file());
    assert_eq!(
        crate::db::auth::project_password(&kit, "shop", "mysql").as_deref(),
        Some("pw")
    );
    assert!(!kit.project_data().join("old").exists());
    assert!(orphans(&kit).is_empty());
}
