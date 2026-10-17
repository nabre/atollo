use super::*;

#[test]
fn names_and_sql() {
    for engine in ["mariadb", "mongodb"] {
        assert_eq!(db_name("Negozio-Online", engine), "negozio_online");
        assert_eq!(db_name("app & co (1)", engine), "app___co__1_");
        assert_eq!(db_name("---", engine), "progetto");
        assert_eq!(db_name("mysql2", engine), "mysql2");
        assert_eq!(db_name(&"x".repeat(80), engine).len(), 64);
    }
    // never the server's own databases or users; those of the other
    // server are ordinary names
    for (project, maria, mongo) in [
        ("mysql", "mysql_db", "mysql"),
        ("MySQL", "mysql_db", "mysql"),
        ("sys", "sys_db", "sys"),
        (
            "information_schema",
            "information_schema_db",
            "information_schema",
        ),
        (
            "performance_schema",
            "performance_schema_db",
            "performance_schema",
        ),
        ("root", "root_db", "root"),
        ("mariadb.sys", "mariadb_sys_db", "mariadb_sys"),
        ("admin", "admin", "admin_db"),
        ("local", "local", "local_db"),
        ("Config", "config", "config_db"),
    ] {
        assert_eq!(db_name(project, "mariadb"), maria, "{project}");
        assert_eq!(db_name(project, "mongodb"), mongo, "{project}");
        assert!(project_drop_sql(&db_name(project, "mariadb")).contains(&format!("`{maria}`")));
    }
    assert!(is_reserved("other", "admin") && is_reserved("other", "mysql"));
    assert_eq!(legacy_db_name("mysql"), "mysql");
    let pw = "0123456789abcdef0123456789abcdef";
    let sql = project_sql("shop", pw);
    assert!(sql.contains("CREATE DATABASE IF NOT EXISTS `shop`"));
    assert!(sql.contains("GRANT ALL PRIVILEGES ON `shop`.* TO 'shop'@'127.0.0.1'"));
    // `_` is a wildcard in a database-level GRANT: only my_app, never my2app
    let sql = project_sql("my_app", pw);
    assert!(sql.contains("CREATE DATABASE IF NOT EXISTS `my_app`"));
    assert!(sql.contains(r"GRANT ALL PRIVILEGES ON `my\_app`.* TO 'my_app'@'::1'"));
    assert!(sql.contains("DELETE FROM mysql.db WHERE Db = 'my_app' AND User = 'my_app'"));
    assert!(!project_sql("shop", pw).contains("mysql.db"));
    assert!(root_password_sql(pw).contains("'root'@'::1' IDENTIFIED BY"));
}

#[test]
fn env_for_laravel_with_mongodb() {
    let db = project_db("mongodb", "shop", "pw1", 27018);
    let out = env_apply_text("DB_CONNECTION=sqlite\n", &db);
    assert!(out.contains("DB_CONNECTION=mongodb"), "{out}");
    assert!(out.contains("DB_DATABASE=shop"), "{out}");
    assert!(
        out.contains("MONGODB_URI=mongodb://shop:pw1@127.0.0.1:27018/shop"),
        "{out}"
    );
}

#[test]
fn env_for_laravel_and_others() {
    let db = project_db("mariadb", "shop", "pw1", 3307);
    let laravel = "APP_NAME=Shop\r\n# DB_HOST=old\r\nDB_CONNECTION=sqlite\r\nDB_HOST=10.0.0.1\r\n";
    let changes = env_changes(laravel, &db);
    assert_eq!(changes[0].key, "DB_CONNECTION");
    assert_eq!(changes[0].old.as_deref(), Some("sqlite"));
    assert!(changes
        .iter()
        .any(|c| c.key == "DB_PASSWORD" && c.old.is_none()));
    let out = env_apply_text(laravel, &db);
    assert!(
        out.contains("# DB_HOST=old\r\n"),
        "comments untouched: {out}"
    );
    assert!(out.contains("DB_HOST=127.0.0.1\r\nDB_PORT=3307") || out.contains("DB_PORT=3307"));
    assert!(out.contains("DB_DATABASE=shop\r\n"));
    assert!(out.starts_with("APP_NAME=Shop\r\n"));
    assert!(env_changes(&out, &db).is_empty(), "applied: nothing left");
    let node = project_db("mongodb", "shop", "pw2", 27017);
    let out = env_apply_text("PORT=3000\n", &node);
    assert_eq!(
        out,
        "PORT=3000\nMONGODB_URI=mongodb://shop:pw2@127.0.0.1:27017/shop?authSource=shop\n"
    );
    let other = env_apply_text("", &db);
    assert_eq!(
        other.trim(),
        "DATABASE_URL=mysql://shop:pw1@127.0.0.1:3307/shop"
    );
}

#[cfg(windows)]
#[test]
fn secrets_round_trip_and_gitignore() {
    let tmp = crate::kit::tests::TempDir::new("db-auth");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64);
    kit.prepare().unwrap();
    let mut s = load(&kit);
    assert_eq!(s, Secrets::default());
    s.mariadb_root = Some(new_password().unwrap());
    save(&kit, &s).unwrap();
    assert_eq!(load(&kit), s);
    assert_eq!(s.mariadb_root.as_ref().unwrap().len(), 32);
    write_client_ini(&kit, 3306, "abc").unwrap();
    assert!(fs::read_to_string(client_ini(&kit))
        .unwrap()
        .contains("password=abc\r\nhost=127.0.0.1\r\nport=3306"));
    let p = tmp.0.join("p");
    fs::create_dir_all(&p).unwrap();
    assert!(!env_ignored(&p));
    fs::write(p.join(".gitignore"), "/vendor\n.env\n").unwrap();
    assert!(env_ignored(&p));
    fs::write(p.join(".env.example"), "DB_CONNECTION=mysql\n").unwrap();
    env_apply(&p, &project_db("mariadb", "p", "z", 3306)).unwrap();
    assert!(fs::read_to_string(p.join(".env"))
        .unwrap()
        .contains("DB_PASSWORD=z"));
}

#[cfg(windows)]
/// The DACL of `path` as SDDL text (`D:P(A;;FA;;;OW)`).
pub(crate) fn dacl(path: &Path) -> String {
    use windows::core::{HSTRING, PWSTR};
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::Win32::Security::Authorization::{
        ConvertSecurityDescriptorToStringSecurityDescriptorW, SDDL_REVISION_1,
    };
    use windows::Win32::Security::{
        GetFileSecurityW, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
    };
    let file = HSTRING::from(path.as_os_str());
    let mut needed = 0u32;
    // SAFETY: a size query, no buffer; fails with the size needed.
    let _ = unsafe { GetFileSecurityW(&file, DACL_SECURITY_INFORMATION.0, None, 0, &mut needed) };
    assert!(needed > 0, "{}: no security descriptor", path.display());
    let mut buf = vec![0u8; needed as usize];
    let sd = PSECURITY_DESCRIPTOR(buf.as_mut_ptr().cast());
    let mut text = PWSTR::null();
    // SAFETY: `buf` is `needed` bytes and outlives `sd`; `text` is
    // LocalAlloc'd by the conversion and freed once below.
    unsafe {
        GetFileSecurityW(
            &file,
            DACL_SECURITY_INFORMATION.0,
            Some(sd),
            needed,
            &mut needed,
        )
        .ok()
        .unwrap();
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut text,
            None,
        )
        .unwrap();
        let s = text.to_string().unwrap();
        LocalFree(Some(HLOCAL(text.0.cast())));
        s
    }
}

/// Unix: no permission for the group or the others.
#[cfg(unix)]
pub(crate) fn assert_owner_only(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mode = fs::metadata(path).unwrap().permissions().mode();
    assert_eq!(mode & 0o077, 0, "{}: {mode:o}", path.display());
}

/// One entry, for the owner, nobody else (the test folder is on NTFS).
#[cfg(windows)]
pub(crate) fn assert_owner_only(path: &Path) {
    let sddl = dacl(path);
    assert_eq!(sddl.matches("(A;").count(), 1, "{}: {sddl}", path.display());
    assert!(sddl.ends_with(";FA;;;OW)"), "{}: {sddl}", path.display());
}

#[cfg(windows)]
#[test]
fn project_passwords_in_private_folder() {
    let tmp = crate::kit::tests::TempDir::new("db-auth-project");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64);
    kit.prepare().unwrap();
    assert_eq!(project_password(&kit, "shop", "mariadb"), None);
    assert!(set_project_password(&kit, "..", "mariadb", "x").is_err());
    assert!(project_passwords(&kit, r"..\x").is_empty());

    // private\ is owner-only before db.json exists: a file created in it
    // by anyone (here a plain write) inherits that DACL.
    let state = ProjectState::of(&kit, "shop").unwrap();
    let private = state.private_dir().unwrap();
    assert!(!private.join("db.json").exists());
    let sddl = dacl(&private);
    assert!(sddl.starts_with("D:P") && sddl.contains("OICI"), "{sddl}");
    assert_owner_only(&private);
    fs::write(private.join("probe"), "").unwrap();
    assert_owner_only(&private.join("probe"));
    fs::remove_file(private.join("probe")).unwrap();

    let pw = new_password().unwrap();
    set_project_password(&kit, "shop", "mariadb", &pw).unwrap();
    set_project_password(&kit, "shop", "mongodb", "m1").unwrap();
    let file = tmp.0.join(r"data\projects\shop\private\db.json");
    assert_owner_only(&file);
    assert_eq!(project_password(&kit, "shop", "mariadb"), Some(pw.clone()));
    assert_eq!(project_password(&kit, "other", "mariadb"), None);

    // keys of a newer version stay; a file that is not JSON is never
    // overwritten
    let mut json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
    json["future"] = serde_json::json!({ "a": 1 });
    fs::write(&file, json.to_string()).unwrap();
    set_project_password(&kit, "shop", "mongodb", "m2").unwrap();
    let text = fs::read_to_string(&file).unwrap();
    assert!(
        text.contains("\"future\"") && text.contains("\"m2\""),
        "{text}"
    );
    let projects = project_passwords(&kit, "shop");
    assert_eq!(projects.len(), 2, "only the passwords: {projects:?}");

    // secrets.json keeps the administrators and an old `projects`
    // untouched (for the migration), never the new passwords
    let old = r#"{"mariadbRoot":"r1","projects":{"old":{"mariadb":"o1"}}}"#;
    fs::create_dir_all(dir(&kit)).unwrap();
    fs::write(secrets_path(&kit), old).unwrap();
    let mut s = load(&kit);
    assert_eq!(s.mariadb_root.as_deref(), Some("r1"));
    s.mongodb_admin = Some("a1".into());
    save(&kit, &s).unwrap();
    let saved: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(secrets_path(&kit)).unwrap()).unwrap();
    assert_eq!(
        saved["projects"],
        serde_json::json!({ "old": { "mariadb": "o1" } })
    );
    assert_eq!(saved["mongodbAdmin"], "a1");
    let text = saved.to_string();
    assert!(!text.contains(&pw) && !text.contains("m2"), "{text}");

    // dropped: the passwords come back once, the file goes
    let dropped = drop_project(&kit, "shop").unwrap();
    assert_eq!(dropped.get("mariadb"), Some(&pw));
    assert_eq!(dropped.get("mongodb").map(String::as_str), Some("m2"));
    assert_eq!(dropped.len(), 2);
    assert!(!file.exists());
    assert!(drop_project(&kit, "shop").unwrap().is_empty());
    assert_eq!(project_password(&kit, "shop", "mariadb"), None);

    fs::write(&file, "{ not json").unwrap();
    assert!(set_project_password(&kit, "shop", "mariadb", "x").is_err());
    assert!(drop_project(&kit, "shop").is_err());
    assert_eq!(fs::read_to_string(&file).unwrap(), "{ not json");
}

/// A `db.json` that cannot be read is an error, never "no password":
/// otherwise a new one would reach the server and be stored nowhere.
#[test]
fn unreadable_passwords_stop_the_creation() {
    let tmp = crate::kit::tests::TempDir::new("db-auth-unreadable");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64);
    kit.prepare().unwrap();
    let private = tmp.0.join(r"data\projects\shop\private");

    // reading creates nothing
    assert_eq!(try_project_password(&kit, "shop", "mariadb").unwrap(), None);
    assert!(try_project_passwords(&kit, "shop").unwrap().is_empty());
    assert!(!private.exists(), "reading made private\\");
    assert!(try_project_password(&kit, "..", "mariadb").is_err());

    // a new password and its name are stored before they are returned
    // (to the server)
    let (name, pw) = ensure_project_db(&kit, "shop", "mariadb").unwrap();
    assert_eq!(name, "shop");
    assert_eq!(pw.len(), 32);
    assert_eq!(project_password(&kit, "shop", "mariadb"), Some(pw.clone()));
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(private.join("db.json")).unwrap()).unwrap();
    assert_eq!(json["names"]["mariadb"], "shop");
    assert_eq!(
        ensure_project_db(&kit, "shop", "mariadb").unwrap(),
        (name, pw)
    );

    // not JSON: an error, the file untouched
    let file = private.join("db.json");
    fs::write(&file, "{ not json").unwrap();
    assert!(try_project_password(&kit, "shop", "mariadb").is_err());
    assert!(try_project_passwords(&kit, "shop").is_err());
    assert_eq!(project_password(&kit, "shop", "mariadb"), None, "lenient");
    assert!(ensure_project_db(&kit, "shop", "mariadb").is_err());
    assert!(ensure_project_db(&kit, "shop", "mongodb").is_err());
    assert!(project_db_name(&kit, "shop", "mariadb").is_err());
    assert_eq!(fs::read_to_string(&file).unwrap(), "{ not json");

    // another project's unreadable file stops a creation too: what it
    // names is unknown
    assert!(ensure_project_db(&kit, "blog", "mariadb").is_err());
    assert!(!tmp.0.join(r"data\projects\blog").exists());

    // a folder where the file should be: cannot be read either
    fs::remove_file(&file).unwrap();
    fs::create_dir(&file).unwrap();
    assert!(try_project_password(&kit, "shop", "mariadb").is_err());
    assert!(ensure_project_db(&kit, "shop", "mariadb").is_err());
    fs::remove_dir(&file).unwrap();

    // dropping reads without creating: nothing comes back after
    assert!(drop_project(&kit, "shop").unwrap().is_empty());
    fs::remove_dir_all(tmp.0.join(r"data\projects\shop")).unwrap();
    assert!(drop_project(&kit, "shop").unwrap().is_empty());
    assert!(!private.exists());
}

fn write_db_json(kit: &Kit, project: &str, json: &str) {
    let file = kit.project_data().join(project).join(r"private\db.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, json).unwrap();
}

fn db_json(kit: &Kit, project: &str) -> serde_json::Value {
    let file = kit.project_data().join(project).join(r"private\db.json");
    serde_json::from_str(&fs::read_to_string(file).unwrap()).unwrap()
}

/// The stored name wins; data from before names were stored keeps the
/// rule of its time; a new database gets the server's reserved names
/// with `_db`, only for that server.
#[test]
fn names_stored_legacy_and_new() {
    let tmp = crate::kit::tests::TempDir::new("db-auth-names");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64);
    kit.prepare().unwrap();

    // nothing stored: the new rule, per server, nothing created
    assert_eq!(
        project_db_name(&kit, "admin", "mongodb").unwrap(),
        "admin_db"
    );
    assert_eq!(project_db_name(&kit, "admin", "mariadb").unwrap(), "admin");
    assert_eq!(
        project_db_name(&kit, "mysql", "mariadb").unwrap(),
        "mysql_db"
    );
    assert!(!kit.project_data().join("admin").exists());
    let (name, _) = ensure_project_db(&kit, "admin", "mongodb").unwrap();
    assert_eq!(name, "admin_db");
    let (name, _) = ensure_project_db(&kit, "admin", "mariadb").unwrap();
    assert_eq!(name, "admin");
    assert_eq!(
        db_json(&kit, "admin")["names"],
        serde_json::json!({ "mongodb": "admin_db", "mariadb": "admin" })
    );
    // the passwords' readers skip the names
    assert_eq!(project_passwords(&kit, "admin").len(), 2);

    // a project of before the names (a password, no name): the rule of
    // its time, `config` on MariaDB kept; on MongoDB never touched
    write_db_json(&kit, "config", r#"{"mariadb":"pw1","mongodb":"pw2"}"#);
    assert_eq!(
        project_db_name(&kit, "config", "mariadb").unwrap(),
        "config"
    );
    let err = project_db_name(&kit, "config", "mongodb").unwrap_err();
    assert!(format!("{err:#}").contains("by hand"), "{err:#}");
    assert!(ensure_project_db(&kit, "config", "mongodb").is_err());
    let (name, pw) = ensure_project_db(&kit, "config", "mariadb").unwrap();
    assert_eq!((name.as_str(), pw.as_str()), ("config", "pw1"));
    assert_eq!(db_json(&kit, "config")["names"]["mariadb"], "config");
    assert!(db_json(&kit, "config")["names"].get("mongodb").is_none());

    // a stored name wins over the project's name; one of the server's
    // own, or not a name, is refused even when stored
    write_db_json(
        &kit,
        "renamed",
        r#"{"mariadb":"pw","names":{"mariadb":"old_name"},"future":1}"#,
    );
    assert_eq!(
        project_db_name(&kit, "renamed", "mariadb").unwrap(),
        "old_name"
    );
    write_db_json(
        &kit,
        "bad",
        r#"{"mariadb":"pw","names":{"mariadb":"mysql"}}"#,
    );
    assert!(project_db_name(&kit, "bad", "mariadb").is_err());
    write_db_json(&kit, "bad", r#"{"mariadb":"pw","names":{"mariadb":"a`b"}}"#);
    assert!(project_db_name(&kit, "bad", "mariadb").is_err());
    write_db_json(&kit, "bad", r#"{"mariadb":"pw","names":"x"}"#);
    assert!(project_db_name(&kit, "bad", "mariadb").is_err());
    fs::remove_dir_all(kit.project_data().join("bad")).unwrap();

    // the names of before are pinned (adopt, migration); never replaced
    write_db_json(
        &kit,
        "old",
        r#"{"mariadb":"pw","mongodb":"pw2","names":{"mongodb":"kept"}}"#,
    );
    keep_legacy_names(&kit, "old").unwrap();
    assert_eq!(
        db_json(&kit, "old")["names"],
        serde_json::json!({ "mariadb": "old", "mongodb": "kept" })
    );
    let mut s = ProjectSecrets::new();
    s.insert("mongodb".into(), "pw".into());
    assert!(
        !add_legacy_names(&mut s, "admin").unwrap(),
        "reserved: left out"
    );
    keep_legacy_names(&kit, "nothing").unwrap();
    assert!(!kit.project_data().join("nothing").exists());
}

/// Passwords not yet moved by the migration, or left in secrets.json by
/// it, name databases too: nothing is done while data of layout 1 waits,
/// and what stays in secrets.json is seen by the collision check.
#[test]
fn data_of_before_is_never_unseen() {
    let tmp = crate::kit::tests::TempDir::new("db-auth-pending");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64);
    kit.prepare().unwrap();
    // layout 1 still there: refused, nothing stored
    fs::create_dir_all(kit.project_states().join("my-app")).unwrap();
    let err = ensure_project_db(&kit, "my_app", "mariadb").unwrap_err();
    assert!(format!("{err:#}").contains("not all moved"), "{err:#}");
    assert!(!kit.project_data().join("my_app").exists());
    fs::remove_dir_all(kit.project_states()).unwrap();
    fs::write(kit.app_dir().join("layout.txt"), "2").unwrap();
    // left in secrets.json (a conflict, a name that is no project's)
    let secrets = Secrets {
        other: [(
            "projects".to_owned(),
            serde_json::json!({ "my-app": { "mariadb": "pw" } }),
        )]
        .into(),
        ..Secrets::default()
    };
    save(&kit, &secrets).unwrap();
    let err = ensure_project_db(&kit, "my_app", "mariadb").unwrap_err();
    assert!(format!("{err:#}").contains("my-app"), "{err:#}");
    assert!(ensure_project_db(&kit, "my_app", "mongodb").is_ok());
}

/// Two projects that would get the same name: the second one's creation
/// is refused before anything is stored or sent to a server.
#[test]
fn same_name_refused_on_creation() {
    let tmp = crate::kit::tests::TempDir::new("db-auth-collision");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64);
    kit.prepare().unwrap();
    ensure_project_db(&kit, "my-app", "mariadb").unwrap();
    let err = ensure_project_db(&kit, "my_app", "mariadb").unwrap_err();
    assert!(format!("{err:#}").contains("my-app"), "{err:#}");
    assert!(
        !kit.project_data().join("my_app").exists(),
        "nothing stored"
    );
    assert_eq!(
        name_used_by(&kit, "my_app", "mariadb", "my_app")
            .unwrap()
            .as_deref(),
        Some("my-app")
    );
    // another server, another name space
    ensure_project_db(&kit, "my_app", "mongodb").unwrap();

    // a project of before the names counts with the rule of its time
    write_db_json(&kit, "città", r#"{"mariadb":"pw"}"#);
    assert!(ensure_project_db(&kit, "cittè", "mariadb").is_err());
    assert!(check_name_free(&kit, "città", "mariadb", "citt_").is_ok());
    // a name stored by an adopted project counts too
    write_db_json(
        &kit,
        "new-name",
        r#"{"mariadb":"pw","names":{"mariadb":"shop"}}"#,
    );
    assert!(ensure_project_db(&kit, "shop", "mariadb").is_err());
    assert!(ensure_project_db(&kit, "SHOP", "mongodb").is_ok());
}

#[test]
fn drop_scripts() {
    let sql = project_drop_sql("shop");
    assert!(sql.starts_with("DROP DATABASE IF EXISTS `shop`;"));
    assert!(
        sql.contains("DROP USER IF EXISTS 'shop'@'localhost', 'shop'@'127.0.0.1', 'shop'@'::1';")
    );
    assert!(MONGO_DROP_JS.contains("dropUser(n)") && MONGO_DROP_JS.contains("dropDatabase()"));
    assert!(!MONGO_DROP_JS.contains("DEVKIT_DB_PASSWORD"));
}

/// The version of a server for the project's database (PLAN.md §13.2):
/// in `lines` of `db.json`, beside the passwords and the names, which its
/// readers do not take for a password.
#[test]
fn project_lines_in_db_json() {
    let tmp = crate::kit::tests::TempDir::new("db-auth-lines");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64);
    kit.prepare().unwrap();
    assert_eq!(project_line(&kit, "shop", "mariadb").unwrap(), None);
    set_project_line(&kit, "shop", "mariadb", None).unwrap();
    assert!(
        !kit.project_data().join(r"shop\private\db.json").exists(),
        "nothing created to remove a version"
    );
    write_db_json(
        &kit,
        "shop",
        r#"{ "mariadb": "pw", "names": { "mariadb": "shop_old" } }"#,
    );
    set_project_line(&kit, "shop", "mariadb", Some("10.6")).unwrap();
    assert_eq!(
        project_line(&kit, "shop", "mariadb").unwrap().as_deref(),
        Some("10.6")
    );
    assert_eq!(project_line(&kit, "shop", "mongodb").unwrap(), None);
    let json = db_json(&kit, "shop");
    assert_eq!(json["mariadb"], "pw", "password kept");
    assert_eq!(json["names"]["mariadb"], "shop_old", "name kept");
    assert_eq!(
        project_passwords(&kit, "shop").keys().collect::<Vec<_>>(),
        ["mariadb"],
        "lines is not a password"
    );
    assert!(set_project_line(&kit, "shop", "mariadb", Some("x; DROP")).is_err());
    set_project_line(&kit, "shop", "mariadb", None).unwrap();
    assert!(
        db_json(&kit, "shop").get("lines").is_none(),
        "empty lines removed"
    );
    assert_eq!(project_line(&kit, "shop", "mariadb").unwrap(), None);
    // A value that is not a version is ignored, never used in a path.
    write_db_json(&kit, "shop", r#"{ "lines": { "mariadb": "..\\x" } }"#);
    assert_eq!(project_line(&kit, "shop", "mariadb").unwrap(), None);
}
