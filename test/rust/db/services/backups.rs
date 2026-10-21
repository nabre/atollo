use super::*;
use crate::db::auth::tests::assert_owner_only;
use crate::kit::{tests::TempDir, Arch, Kit};

#[test]
fn copies_in_the_private_folder() {
    let tmp = TempDir::new("db-backups");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let s = Services::new(kit);
    assert!(s.backup_dir("..").is_err());
    assert!(s.backups(r"..\x").is_empty());
    assert!(s.backups("../x").is_empty());

    let dir = s.backup_dir("shop").unwrap();
    let private = tmp
        .0
        .join("data")
        .join("projects")
        .join("shop")
        .join("private");
    assert_eq!(dir, private.join("backups"));
    assert!(s.backups("shop").is_empty());
    // a copy written there (as mysqldump's output) is private by
    // inheritance from private\ on Windows; on Unix a new file keeps the
    // umask's mode, and private/ (0700) is what keeps the others out
    for name in ["shop-1.sql", "shop-2.archive.gz", "notes.txt"] {
        fs::write(dir.join(name), "x").unwrap();
    }
    #[cfg(windows)]
    assert_owner_only(&dir.join("shop-1.sql"));
    #[cfg(unix)]
    assert_owner_only(&private);
    let list = s.backups("shop");
    let names: Vec<(&str, &str)> = list
        .iter()
        .map(|b| (b.file.as_str(), b.engine.as_str()))
        .collect();
    assert_eq!(
        names,
        [("shop-2.archive.gz", "mongodb"), ("shop-1.sql", "mysql")]
    );
    assert!(s.backups("other").is_empty(), "one folder per project");

    // no server: export fails before writing anything
    assert!(s.export("shop", "mysql").is_err());
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 3);

    // names outside the folder are refused before any server is asked
    let outside = private.join("x.sql");
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
fn only_the_projects_own_database() {
    let tmp = TempDir::new("db-backups-own");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let s = Services::new(kit.clone());
    let db_json = |p: &str, json: &str| {
        let file = kit.project_data().join(p).join("private").join("db.json");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, json).unwrap();
    };
    // no database made by the kit: nothing, whatever exists on the server
    let e = s.own_database("shop", "mysql").unwrap_err().to_string();
    assert!(e.contains("no mysql database"), "{e}");
    db_json("shop", r#"{"mysql":"pw","names":{"mysql":"shop"}}"#);
    assert_eq!(s.own_database("shop", "mysql").unwrap(), "shop");
    // another project with the same name: refused
    db_json("other", r#"{"mysql":"pw","names":{"mysql":"shop"}}"#);
    let e = format!("{:#}", s.own_database("shop", "mysql").unwrap_err());
    assert!(e.contains("other"), "{e}");
}

/// A dump is written to `<copy>.part`, listed only once done; a failed one
/// leaves nothing; two in the same second get two names (BUG-V6-06).
#[test]
fn a_copy_is_listed_only_when_whole() {
    let tmp = TempDir::new("db-backups-part");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let s = Services::new(kit);
    let dir = s.backup_dir("shop").unwrap();

    let (file, part, out) = new_copy(&dir, "shop", DbEngine::MongoDb).unwrap();
    assert!(part.is_file() && !file.exists());
    assert!(
        s.backups("shop").is_empty(),
        "a dump being written is no copy"
    );
    // cut short: the half-written archive goes
    drop(out);
    fs::write(&part, "half").unwrap();
    let e = finish_copy(&part, &file, Err(anyhow!("mongodump: no answer"))).unwrap_err();
    assert!(e.to_string().contains("no answer"), "{e}");
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 0, "nothing left");
    assert!(s.backups("shop").is_empty());

    // done: the copy, listed
    let (file, part, out) = new_copy(&dir, "shop", DbEngine::MySql).unwrap();
    drop(out);
    fs::write(&part, "-- dump").unwrap();
    assert_eq!(finish_copy(&part, &file, Ok(())).unwrap(), file);
    assert!(!part.exists());
    let listed = s.backups("shop");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].file, file.file_name().unwrap().to_string_lossy());

    // many at once: as many names, the newest first
    let names: Vec<PathBuf> = (0..5)
        .map(|_| {
            let (file, part, _out) = new_copy(&dir, "shop", DbEngine::MySql).unwrap();
            fs::write(&part, "-- dump").unwrap();
            finish_copy(&part, &file, Ok(())).unwrap()
        })
        .collect();
    let listed = s.backups("shop");
    assert_eq!(listed.len(), 6, "{listed:?}");
    assert_eq!(
        listed[0].file,
        names[4].file_name().unwrap().to_string_lossy(),
        "the newest first"
    );
}

/// Binary columns go into the copy as hexadecimal (BUG-V6-04): the restore
/// streams the file to the client, whatever bytes a BLOB held.
#[test]
fn binary_columns_are_dumped_as_text() {
    assert!(DUMP_ARGS.contains(&"--hex-blob"), "{DUMP_ARGS:?}");
    assert_eq!(DUMP_ARGS[0], "--single-transaction");
}
