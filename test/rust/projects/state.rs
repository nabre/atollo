use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn one_folder_per_project() {
    let tmp = TempDir::new("project-state");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    for bad in ["", "..", r"..\x", "a/b", "con", "x."] {
        assert!(ProjectState::of(&kit, bad).is_none(), "{bad:?}");
    }
    let s = ProjectState::of(&kit, "shop").unwrap();
    assert_eq!(s.dir(), &tmp.0.join(r"data\projects\shop"));
    assert_eq!(
        s.settings_file(),
        tmp.0.join(r"data\projects\shop\dev-settings.json")
    );
    assert_eq!(s.ports_file(), tmp.0.join(r"data\projects\shop\ports.json"));
    assert_eq!(s.apache_dir(), tmp.0.join(r"data\projects\shop\apache"));
    let backups = s.backups_dir().unwrap();
    assert!(backups.is_dir() && backups.ends_with(r"private\backups"));
    assert_eq!(
        s.db_file().unwrap(),
        tmp.0.join(r"data\projects\shop\private\db.json")
    );
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
    let tmp = TempDir::new("project-orphans");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let write = |p: &str, rel: &str, text: &str| {
        let path = kit.project_data().join(p).join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    };
    // a project with its files; one gone with only empty folders (made
    // by reading its passwords or copies); one gone with its files
    fs::create_dir_all(kit.projects().join("app")).unwrap();
    write("app", "dev-settings.json", "{}");
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
    write("old", r"private\db.json", r#"{"mariadb":"pw","future":{}}"#);
    write("old", r"private\backups\old-mariadb-1.sql", "x");
    assert_eq!(orphans(&kit), ["old"]);
    assert_eq!(
        holdings(&kit, "old").unwrap(),
        Holdings {
            project: "old".into(),
            settings: true,
            ports: 1,
            databases: vec!["mariadb".into()],
            backups: 1,
        }
    );
    let json = serde_json::to_string(&holdings(&kit, "old").unwrap()).unwrap();
    assert!(!json.contains("\"pw\""), "no password: {json}");
    assert!(holdings(&kit, "..").is_none());

    // refused: a project with files, a missing project, not an orphan
    assert!(adopt(&kit, "old", "app").is_err());
    assert!(adopt(&kit, "old", "missing").is_err());
    assert!(adopt(&kit, "empty", "app").is_err());
    assert!(adopt(&kit, "old", "..").is_err());
    assert!(kit.project_data().join(r"old\dev-settings.json").is_file());

    // renamed outside the app: its empty folders are not data
    let runner = crate::projects::run::Runner::new(kit.clone(), std::sync::Arc::new(|_| {}));
    fs::create_dir_all(kit.projects().join("shop")).unwrap();
    ProjectState::of(&kit, "shop")
        .unwrap()
        .private_dir()
        .unwrap();
    let shop = adopt(&kit, "old", "shop").unwrap();
    runner.rename_project("old", "shop");
    assert_eq!(
        fs::read_to_string(shop.settings_file()).unwrap(),
        r#"{"agent":"claude"}"#
    );
    assert!(shop
        .dir()
        .join(r"private\backups\old-mariadb-1.sql")
        .is_file());
    assert_eq!(
        crate::db::auth::project_password(&kit, "shop", "mariadb").as_deref(),
        Some("pw")
    );
    // the name of before travels with the passwords, other keys kept
    let db: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(shop.dir().join(r"private\db.json")).unwrap())
            .unwrap();
    assert_eq!(
        db,
        serde_json::json!({ "mariadb": "pw", "future": {}, "names": { "mariadb": "old" } })
    );
    assert_eq!(
        crate::db::auth::project_db_name(&kit, "shop", "mariadb").unwrap(),
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
    use crate::os::safe_path::tests::junction;
    let tmp = TempDir::new("project-adopt-links");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let write = |p: &str, rel: &str, text: &str| {
        let path = kit.project_data().join(p).join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    };
    write("old", r"private\db.json", r#"{"mariadb":"pw"}"#);
    fs::create_dir_all(kit.projects().join("shop")).unwrap();
    // data\projects\shop is a junction to a folder outside
    let outside = tmp.0.join("outside");
    fs::create_dir_all(outside.join("private")).unwrap();
    junction(&kit.project_data().join("shop"), &outside);
    let e = adopt(&kit, "old", "shop").unwrap_err();
    assert!(e.to_string().contains("link"), "{e}");
    assert!(kit.project_data().join(r"old\private\db.json").is_file());
    assert!(
        outside.join("private").is_dir(),
        "nothing cleaned through it"
    );
    fs::remove_dir(kit.project_data().join("shop")).unwrap();

    // the orphan a junction: refused, its target untouched
    let target = tmp.0.join("target");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("dev-settings.json"), "{}").unwrap();
    junction(&kit.project_data().join("linked"), &target);
    assert!(adopt(&kit, "linked", "shop").is_err());
    assert!(target.join("dev-settings.json").is_file());
    fs::remove_dir(kit.project_data().join("linked")).unwrap();

    // unreadable passwords: nothing moves
    write("old", r"private\db.json", "{ not json");
    assert!(adopt(&kit, "old", "shop").is_err());
    assert!(kit.project_data().join(r"old\private\db.json").is_file());
    assert!(!kit.project_data().join("shop").exists());
    write("old", r"private\db.json", r#"{"mariadb":"pw"}"#);
    adopt(&kit, "old", "shop").unwrap();
    assert_eq!(
        crate::db::auth::project_db_name(&kit, "shop", "mariadb").unwrap(),
        "old"
    );
}
