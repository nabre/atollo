use super::*;
use crate::db::auth::tests::assert_owner_only;
use crate::kit::{tests::TempDir, Arch, Kit};

#[test]
#[cfg_attr(not(windows), ignore = "expected paths given with backslashes")]
fn copies_in_the_private_folder() {
    let tmp = TempDir::new("db-backups");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let s = Services::new(kit);
    assert!(s.backup_dir("..").is_err());
    assert!(s.backups(r"..\x").is_empty());

    let dir = s.backup_dir("shop").unwrap();
    assert_eq!(dir, tmp.0.join(r"data\projects\shop\private\backups"));
    assert!(s.backups("shop").is_empty());
    // a copy written there (as mariadb-dump's output) is private by
    // inheritance from private\
    for name in ["shop-1.sql", "shop-2.archive.gz", "notes.txt"] {
        fs::write(dir.join(name), "x").unwrap();
    }
    assert_owner_only(&dir.join("shop-1.sql"));
    let list = s.backups("shop");
    let names: Vec<(&str, &str)> = list
        .iter()
        .map(|b| (b.file.as_str(), b.engine.as_str()))
        .collect();
    assert_eq!(
        names,
        [("shop-2.archive.gz", "mongodb"), ("shop-1.sql", "mariadb")]
    );
    assert!(s.backups("other").is_empty(), "one folder per project");

    // no server: export fails before writing anything
    assert!(s.export("shop", "mariadb").is_err());
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 3);

    // names outside the folder are refused before any server is asked
    let outside = tmp.0.join(r"data\projects\shop\private\x.sql");
    fs::write(&outside, "x").unwrap();
    for bad in [
        r"..\x.sql",
        "../x.sql",
        r"C:\x.sql",
        ".hidden.sql",
        "missing.sql",
    ] {
        let e = s.restore("shop", bad).unwrap_err().to_string();
        assert!(
            e.contains("not a copy") || e.contains("not found"),
            "{bad}: {e}"
        );
    }
    // a copy that is there gets past the check (then: no server)
    let e = s.restore("shop", "shop-1.sql").unwrap_err().to_string();
    assert!(!e.contains("not a copy") && !e.contains("not found"), "{e}");
}

/// Dumps and restores run with the server's administrator: only on the
/// database the kit made for the project, never one it shares.
#[test]
#[cfg_attr(not(windows), ignore = "db.json paths given with backslashes")]
fn only_the_projects_own_database() {
    let tmp = TempDir::new("db-backups-own");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let s = Services::new(kit.clone());
    let db_json = |p: &str, json: &str| {
        let file = kit.project_data().join(p).join(r"private\db.json");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, json).unwrap();
    };
    // no database made by the kit: nothing, whatever exists on the server
    let e = s.own_database("shop", "mariadb").unwrap_err().to_string();
    assert!(e.contains("no mariadb database"), "{e}");
    db_json("shop", r#"{"mariadb":"pw","names":{"mariadb":"shop"}}"#);
    assert_eq!(s.own_database("shop", "mariadb").unwrap(), "shop");
    // another project with the same name: refused
    db_json("other", r#"{"mariadb":"pw","names":{"mariadb":"shop"}}"#);
    let e = format!("{:#}", s.own_database("shop", "mariadb").unwrap_err());
    assert!(e.contains("other"), "{e}");
}
