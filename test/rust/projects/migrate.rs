use std::cell::Cell;
use std::collections::BTreeMap;

use serde_json::json;

use super::*;
use crate::db::auth::tests::assert_owner_only;
use crate::kit::{tests::TempDir, Arch};
use crate::test_support::prepared_kit;

thread_local! {
    /// Stops the migration right after the ports, as a crash would.
    pub(super) static INTERRUPT: Cell<bool> = const { Cell::new(false) };
}

const SECRETS: &str = r#"{"mysqlRoot":"r00t","mongodbAdmin":"adm1n","projects":{"shop":{"mysql":"pw-shop-sql","mongodb":"pw-shop-mongo"},"blog":{"mysql":"pw-blog"},".old":{"mysql":"pw-old"}}}"#;
const PORTS: &str = r#"{"ports":{"shop|web":8001,"shop|api":8002,"blog|web":8003,".old|web":8004},"locked":["shop|web",".old|web"]}"#;

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn json_of(path: &Path) -> Value {
    serde_json::from_str(&read(path)).unwrap()
}

/// A kit with every file of layout 1.
fn v1_kit(tmp: &TempDir) -> Kit {
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let old = kit.project_states();
    write(
        &old.join("shop").join("dev-settings.json"),
        r#"{"agent":"claude"}"#,
    );
    write(
        &old.join("shop").join("apache").join("httpd.conf"),
        "Listen 8001\n",
    );
    write(
        &old.join("shop")
            .join("apache")
            .join("conf.d")
            .join("user.conf"),
        "# mine\n",
    );
    write(&old.join("shop").join("versions.json"), r#"{"node":"22"}"#);
    write(&old.join("shop").join("versions.txt"), "a\tb\n");
    write(
        &old.join("blog").join("dev-settings.json"),
        r#"{"agent":"codex"}"#,
    );
    write(&old.join(".old").join("dev-settings.json"), "{}");
    write(&kit.app_dir().join("ports.json"), PORTS);
    write(&auth::secrets_path(&kit), SECRETS);
    let backup = kit.data().join("db").join("backup");
    write(
        &backup.join("shop").join("shop-mysql-1.sql"),
        "CREATE TABLE t;",
    );
    write(&backup.join("shop").join("shop-mongodb-1.archive.gz"), "gz");
    fs::create_dir_all(backup.join("blog")).unwrap();
    kit
}

/// Every file under the kit's `data\` (but the logs) with its bytes.
fn snapshot(kit: &Kit) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(dir: &Path, logs: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in fs::read_dir(dir).into_iter().flatten().flatten() {
            if e.path() == logs {
                continue;
            }
            if e.file_type().unwrap().is_dir() {
                walk(&e.path(), logs, out);
            } else {
                out.insert(e.path(), fs::read(e.path()).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(&kit.data(), &kit.logs(), &mut out);
    out
}

/// What a complete migration of `v1_kit` gives.
fn assert_migrated(kit: &Kit) {
    let new = kit.project_data();
    assert_eq!(read(&kit.app_dir().join("layout.txt")), "2");
    // states, Apache, versions files
    assert_eq!(
        read(&new.join("shop").join("dev-settings.json")),
        r#"{"agent":"claude"}"#
    );
    assert_eq!(
        read(&new.join("shop").join("apache").join("httpd.conf")),
        "Listen 8001\n"
    );
    assert_eq!(
        read(
            &new.join("shop")
                .join("apache")
                .join("conf.d")
                .join("user.conf")
        ),
        "# mine\n"
    );
    assert_eq!(
        read(&new.join("shop").join("versions.json")),
        r#"{"node":"22"}"#
    );
    assert_eq!(read(&new.join("shop").join("versions.txt")), "a\tb\n");
    assert_eq!(
        read(&new.join("blog").join("dev-settings.json")),
        r#"{"agent":"codex"}"#
    );
    // ports, locked included
    assert_eq!(
        json_of(&new.join("shop").join("ports.json")),
        json!({"ports": {"web": 8001, "api": 8002}, "locked": ["web"]})
    );
    assert_eq!(
        json_of(&new.join("blog").join("ports.json")),
        json!({"ports": {"web": 8003}, "locked": []})
    );
    // passwords, private
    let pw = |p, e| auth::project_password(kit, p, e);
    assert_eq!(pw("shop", "mysql").as_deref(), Some("pw-shop-sql"));
    assert_eq!(pw("shop", "mongodb").as_deref(), Some("pw-shop-mongo"));
    assert_eq!(pw("blog", "mysql").as_deref(), Some("pw-blog"));
    assert_owner_only(&new.join("shop").join("private").join("db.json"));
    // with the names their databases were created with (layout 1)
    assert_eq!(
        json_of(&new.join("shop").join("private").join("db.json"))["names"],
        json!({"mysql": "shop", "mongodb": "shop"})
    );
    assert_eq!(
        json_of(&new.join("blog").join("private").join("db.json"))["names"],
        json!({"mysql": "blog"})
    );
    assert_eq!(auth::project_db_name(kit, "blog", "mysql").unwrap(), "blog");
    // copies, owner-only after the move
    let backups = new.join("shop").join("private").join("backups");
    assert_eq!(read(&backups.join("shop-mysql-1.sql")), "CREATE TABLE t;");
    assert_owner_only(&backups.join("shop-mysql-1.sql"));
    assert_owner_only(&backups.join("shop-mongodb-1.archive.gz"));
    // copies of the global files, as they were; the secrets private
    let copies = kit.app_dir().join("pre-v2");
    assert_eq!(read(&copies.join("ports.json")), PORTS);
    assert_eq!(read(&copies.join("secrets.json")), SECRETS);
    assert_owner_only(&copies.join("secrets.json"));
    // the old places: emptied ones gone, an invalid name left
    let old = kit.project_states();
    assert!(!old.join("shop").exists() && !old.join("blog").exists());
    assert!(old.join(".old").join("dev-settings.json").is_file());
    assert!(!kit.data().join("db").join("backup").exists());
    assert_eq!(
        json_of(&kit.app_dir().join("ports.json")),
        json!({"ports": {".old|web": 8004}, "locked": [".old|web"]})
    );
    assert!(!new.join(".old").exists());
}

#[test]
fn v1_kit_to_v2_nothing_lost() {
    let tmp = TempDir::new("migrate-v2");
    let kit = v1_kit(&tmp);
    run(&kit);
    assert_migrated(&kit);
    let secrets = json_of(&auth::secrets_path(&kit));
    assert_eq!(
        secrets,
        json!({"mysqlRoot": "r00t", "mongodbAdmin": "adm1n",
               "projects": {".old": {"mysql": "pw-old"}}}),
        "only what has no project left"
    );
    assert_owner_only(&auth::secrets_path(&kit));

    // a second start does nothing; nor does a full run again
    let before = snapshot(&kit);
    run(&kit);
    assert_eq!(snapshot(&kit), before);
    fs::remove_file(layout_file(&kit)).unwrap();
    run(&kit);
    assert_eq!(snapshot(&kit), before);
}

#[test]
fn a_new_kit_has_nothing_to_move() {
    let (_tmp, kit) = prepared_kit("migrate-new");
    write(&auth::secrets_path(&kit), r#"{"mysqlRoot":"r00t"}"#);
    run(&kit);
    assert_eq!(read(&layout_file(&kit)), "2");
    assert!(!copies_dir(&kit).exists(), "no copy of a file of layout 2");
    assert_eq!(read(&auth::secrets_path(&kit)), r#"{"mysqlRoot":"r00t"}"#);
    assert!(!kit.project_data().exists());
}

#[test]
fn interrupted_after_the_ports_then_retried() {
    let tmp = TempDir::new("migrate-interrupt");
    let kit = v1_kit(&tmp);
    INTERRUPT.set(true);
    run(&kit);
    INTERRUPT.set(false);
    assert!(!layout_file(&kit).exists(), "still layout 1");
    let new = kit.project_data();
    assert!(new.join("shop").join("dev-settings.json").is_file());
    assert!(new.join("shop").join("ports.json").is_file());
    assert_eq!(read(&auth::secrets_path(&kit)), SECRETS, "not touched yet");
    assert!(kit
        .data()
        .join("db")
        .join("backup")
        .join("shop")
        .join("shop-mysql-1.sql")
        .is_file());

    // the next start finishes it, the ports not doubled
    run(&kit);
    assert_migrated(&kit);
}

#[test]
fn present_target_merged_and_password_conflict_kept() {
    let tmp = TempDir::new("migrate-merge");
    let kit = v1_kit(&tmp);
    let shop = ProjectState::of(&kit, "shop").unwrap();
    write(&shop.settings_file(), r#"{"agent":"newer"}"#);
    write(
        &shop.ports_file(),
        r#"{"ports":{"web":9000},"locked":[],"future":1}"#,
    );
    write(
        &shop.db_file().unwrap(),
        r#"{"mysql":"pw-newer","names":{"mysql":"kept"}}"#,
    );
    write(
        &shop.backups_dir().unwrap().join("shop-mysql-1.sql"),
        "newer",
    );
    run(&kit);

    assert_eq!(read(&layout_file(&kit)), "2");
    // what was there wins; the old file stays where it was
    assert_eq!(read(&shop.settings_file()), r#"{"agent":"newer"}"#);
    let old = kit.project_states().join("shop");
    assert_eq!(
        read(&old.join("dev-settings.json")),
        r#"{"agent":"claude"}"#
    );
    assert!(!old.join("apache").exists(), "moved: nothing was there");
    assert_eq!(read(&shop.apache_dir().join("httpd.conf")), "Listen 8001\n");
    assert_eq!(
        json_of(&shop.ports_file()),
        json!({"ports": {"web": 9000, "api": 8002}, "locked": ["web"], "future": 1})
    );
    // passwords: the newer kept, the missing added, the old one kept too
    let pw = |e| auth::project_password(&kit, "shop", e);
    assert_eq!(pw("mysql").as_deref(), Some("pw-newer"));
    assert_eq!(pw("mongodb").as_deref(), Some("pw-shop-mongo"));
    // a name already there is never replaced; the missing one added
    assert_eq!(
        json_of(&shop.db_file().unwrap())["names"],
        json!({"mysql": "kept", "mongodb": "shop"})
    );
    assert_eq!(
        json_of(&auth::secrets_path(&kit))["projects"],
        json!({"shop": {"mysql": "pw-shop-sql"}, ".old": {"mysql": "pw-old"}})
    );
    // copies: the one there wins, the other one moved
    let backups = shop.backups_dir().unwrap();
    assert_eq!(read(&backups.join("shop-mysql-1.sql")), "newer");
    assert_eq!(read(&backups.join("shop-mongodb-1.archive.gz")), "gz");
    let left = kit
        .data()
        .join("db")
        .join("backup")
        .join("shop")
        .join("shop-mysql-1.sql");
    assert_eq!(read(&left), "CREATE TABLE t;");
}
