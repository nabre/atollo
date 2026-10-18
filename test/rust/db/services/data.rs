use super::*;
use crate::db::services::{
    bin_at, line_port, line_ports, lines_in_use, load_settings, log_path, project_client_env,
    project_db_port, project_line, server_lines, Setting, State,
};
use crate::kit::{tests::TempDir, Arch};

/// `id` installed at `version`, the default folder of its line, with the
/// key paths of the kit's target (`bin/mariadbd.exe` on Windows,
/// `bin/mariadbd` elsewhere).
fn fake_server(kit: &Kit, id: &str, version: &str) {
    let c = Catalog::get().find(id).unwrap();
    let dir = crate::versions::install_dir(kit, c, version);
    let folder = c.folder.as_deref().unwrap();
    let target = kit.target();
    for k in std::iter::once(c.key_for(target)).chain(c.keys_for(target)) {
        if let Some(rel) = k.strip_prefix(folder) {
            let file = crate::fsx::join_rel(&dir, rel);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, b"").unwrap();
        }
    }
    c.save_version(kit, version).unwrap();
}

/// The MariaDB programs of a line folder of an older version, as the
/// kit's target names them.
fn fake_mariadb_line(kit: &Kit, dir: &std::path::Path) {
    fs::create_dir_all(dir.join("bin")).unwrap();
    for stem in ["mariadbd", "mariadb"] {
        let exe = crate::router::bins::exe_name(kit.os(), stem);
        fs::write(dir.join("bin").join(exe), b"").unwrap();
    }
}

#[test]
fn each_version_has_its_data() {
    let tmp = TempDir::new("services-lines");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    // Data of an older kit, then the server's own folder per version.
    fake_server(&kit, "mariadb", "11.8.3");
    let root = kit.data().join("db").join("mariadb");
    fs::create_dir_all(root.join("mysql")).unwrap();
    fs::write(root.join("ibdata1"), b"x").unwrap();
    migrate_data(&kit);
    assert!(root.join("11.8").join("mysql").is_dir() && !root.join("mysql").exists());
    assert_eq!(data_dir(&kit, "mariadb"), root.join("11.8"));

    // A newer version starts with a copy; the old data stays.
    let c = Catalog::get().find("mariadb").unwrap();
    let newer = crate::versions::saved_dir(&kit, c, "12.3");
    fake_mariadb_line(&kit, &newer);
    assert_eq!(server_line(&kit, "mariadb").as_deref(), Some("12.3"));
    assert_eq!(server_lines(&kit, "mariadb"), ["12.3", "11.8"]);
    assert_eq!(
        seed(&kit, "mariadb", "12.3", &[]).unwrap().as_deref(),
        Some("11.8")
    );
    assert!(
        root.join("12.3").join("ibdata1").is_file() && root.join("11.8").join("ibdata1").is_file()
    );
    assert_eq!(
        seed(&kit, "mariadb", "12.3", &[]).unwrap(),
        None,
        "only the first time"
    );
    // Back to the older version: its own data, a setting of the server.
    let s = Services::new(kit.clone());
    s.set_line("mariadb", Some("11.8")).unwrap();
    assert_eq!(data_dir(&kit, "mariadb"), root.join("11.8"));
    assert!(
        s.set_line("mariadb", Some("10.6")).is_err(),
        "not installed"
    );
    assert_eq!(
        bin(&kit, "mariadb", "mariadbd.exe")
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap(),
        "11.8"
    );

    // MongoDB: one major at a time.
    let mongo = kit.data().join("db").join("mongodb");
    fs::create_dir_all(mongo.join("7.0")).unwrap();
    fs::write(mongo.join("7.0").join("WiredTiger"), b"").unwrap();
    assert_eq!(
        seed(&kit, "mongodb", "9.0", &[]).unwrap(),
        None,
        "7.0 cannot jump to 9.0"
    );
    assert_eq!(
        seed(&kit, "mongodb", "8.0", &[]).unwrap().as_deref(),
        Some("7.0")
    );
}

/// The data of a running version is never copied (PLAN.md §13.2).
#[test]
fn no_copy_of_a_running_version() {
    let tmp = TempDir::new("services-seed-running");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let root = kit.data().join("db").join("mariadb");
    fs::create_dir_all(root.join("10.6").join("mysql")).unwrap();
    let err = seed(&kit, "mariadb", "11.8", &["10.6".to_owned()]).unwrap_err();
    assert!(format!("{err:#}").contains("stop mariadb 10.6"), "{err:#}");
    assert!(!root.join("11.8").exists(), "nothing copied");
    assert_eq!(
        seed(&kit, "mariadb", "11.8", &["11.8".to_owned()])
            .unwrap()
            .as_deref(),
        Some("10.6")
    );
}

/// Two versions of MariaDB side by side (PLAN.md §13.2): the one the
/// server runs on its port, the other on the next free one, stored at
/// its first start; each with its data, program and log; the project's
/// database on the version chosen for it, at that version's port.
#[test]
fn versions_side_by_side() {
    let tmp = TempDir::new("services-side");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    fake_server(&kit, "mariadb", "11.4.5");
    let c = Catalog::get().find("mariadb").unwrap();
    let older = crate::versions::saved_dir(&kit, c, "10.6");
    fake_mariadb_line(&kit, &older);
    assert_eq!(server_lines(&kit, "mariadb"), ["11.4", "10.6"]);
    assert_eq!(server_line(&kit, "mariadb").as_deref(), Some("11.4"));
    assert_eq!(line_port(&kit, "mariadb", None), 3306);
    assert_eq!(line_port(&kit, "mariadb", Some("11.4")), 3306);
    assert_eq!(line_port(&kit, "mariadb", Some("10.6")), 3307);
    assert_eq!(
        data_dir_at(&kit, "mariadb", Some("10.6")),
        kit.data().join("db").join("mariadb").join("10.6")
    );
    assert!(bin_at(&kit, "mariadb", Some("10.6"), "mariadbd.exe").starts_with(&older));
    assert_eq!(
        log_path(&kit, "mariadb", Some("10.6")),
        kit.logs().join("mariadb-10.6.log")
    );
    assert_eq!(
        log_path(&kit, "mariadb", Some("11.4")),
        kit.logs().join("mariadb.log")
    );

    // The page: the other version with its port and data.
    let s = Services::new(kit.clone());
    let info = s.list().into_iter().find(|i| i.id == "mariadb").unwrap();
    assert_eq!((info.port, info.line.as_deref()), (3306, Some("11.4")));
    assert_eq!(info.others.len(), 1);
    let other = &info.others[0];
    assert_eq!((other.line.as_str(), other.port), ("10.6", 3307));
    assert_eq!(other.state, State::Stopped);
    assert_eq!(other.url, "mysql://root@127.0.0.1:3307/");

    // Ports of the other versions: stored, never the server's nor another's.
    s.set_line_port("mariadb", "10.6", Some(3316)).unwrap();
    assert_eq!(line_port(&kit, "mariadb", Some("10.6")), 3316);
    assert!(s.set_line_port("mariadb", "10.6", Some(3306)).is_err());
    assert!(s.set_line_port("mariadb", "10.6", Some(80)).is_err());
    assert!(s.set_line_port("mariadb", "11.4", Some(3320)).is_err());
    assert!(s.set_line_port("mariadb", "9.0", Some(3320)).is_err());
    s.set("mariadb", Some(3310), true).unwrap();
    assert_eq!(
        load_settings(&kit)["mariadb"].ports["10.6"],
        3316,
        "the server's settings keep the other ports"
    );
    s.set_line_port("mariadb", "10.6", None).unwrap();
    assert_eq!(line_port(&kit, "mariadb", Some("10.6")), 3311);
    s.set("mariadb", None, false).unwrap();
    assert!(s
        .start_line("mariadb", Some("9.0"), &KitEnv::default())
        .is_err());

    // The project's database follows the version chosen for it.
    crate::db::auth::set_project_password(&kit, "shop", "mariadb", "pw").unwrap();
    assert_eq!(project_db_port(&kit, "shop"), Some(3306));
    assert_eq!(
        project_db_port(&kit, "blog"),
        None,
        "no database: no DB_PORT"
    );
    assert!(s
        .project_db_line("shop", "mariadb", Some("9.0"), &KitEnv::default())
        .is_err());
    crate::db::auth::set_project_line(&kit, "shop", "mariadb", Some("10.6")).unwrap();
    assert_eq!(
        project_line(&kit, "shop", "mariadb").as_deref(),
        Some("10.6")
    );
    assert_eq!(project_db_port(&kit, "shop"), Some(3307));
    let dbs = s.project_dbs("shop");
    assert_eq!((dbs[0].port, dbs[0].url.contains(":3307/")), (3307, true));
    let err = s.project_db_create("shop", "mariadb").unwrap_err();
    assert!(
        format!("{err:#}").contains("mariadb 10.6 is not running"),
        "{err:#}"
    );

    // A saved state of before (no `ports`) reads as it was.
    fs::write(
        kit.app_dir().join("services.json"),
        r#"{ "mariadb": { "port": 3308, "autostart": true } }"#,
    )
    .unwrap();
    let old = &load_settings(&kit)["mariadb"];
    assert!(old.autostart && old.ports.is_empty());
    assert_eq!(line_port(&kit, "mariadb", Some("10.6")), 3309);
}

/// The automatic start starts, beside the server, the other installed
/// versions the projects' databases are on: not the server's own, not one
/// chosen without a database, not one uninstalled.
#[test]
fn autostart_lines_follow_the_projects() {
    use crate::db::auth;
    let tmp = TempDir::new("services-autostart");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    fake_server(&kit, "mariadb", "11.4.5");
    let c = Catalog::get().find("mariadb").unwrap();
    let older = crate::versions::saved_dir(&kit, c, "10.6");
    fake_mariadb_line(&kit, &older);
    assert!(lines_in_use(&kit, "mariadb").is_empty());
    // chosen, no database yet: not started
    auth::set_project_line(&kit, "blog", "mariadb", Some("10.6")).unwrap();
    auth::set_project_password(&kit, "blog", "mariadb", "pw").unwrap();
    auth::set_project_line(&kit, "blog", "mariadb", None).unwrap();
    auth::set_project_password(&kit, "shop", "mariadb", "pw").unwrap();
    auth::set_project_line(&kit, "shop", "mariadb", Some("11.4")).unwrap();
    assert!(
        lines_in_use(&kit, "mariadb").is_empty(),
        "the server's own version is started anyway"
    );
    auth::set_project_line(&kit, "shop", "mariadb", Some("10.6")).unwrap();
    auth::set_project_password(&kit, "wiki", "mariadb", "pw").unwrap();
    auth::set_project_line(&kit, "wiki", "mariadb", Some("9.0")).unwrap();
    assert_eq!(lines_in_use(&kit, "mariadb"), ["10.6"], "9.0 not installed");
    assert!(lines_in_use(&kit, "mongodb").is_empty());

    // A version uninstalled is told as such, for the interface to offer it.
    let s = Services::new(kit.clone());
    let missing = |e: anyhow::Error| {
        e.downcast_ref::<crate::db::services::LineMissing>()
            .map(|m| (m.id.clone(), m.line.clone()))
    };
    let err = s
        .start_for("wiki", "mariadb", &KitEnv::default())
        .unwrap_err();
    assert_eq!(missing(err), Some(("mariadb".into(), "9.0".into())));
    let err = s
        .project_db_line("wiki", "mariadb", Some("9.1"), &KitEnv::default())
        .unwrap_err();
    assert_eq!(missing(err), Some(("mariadb".into(), "9.1".into())));

    // `MYSQL_HOME` of a project on another version: that version's login,
    // once its start wrote it; none on the server's version.
    assert!(
        project_client_env(&kit, "shop").is_empty(),
        "not started yet"
    );
    auth::write_line_client_ini(&kit, "10.6", 3307, "pw").unwrap();
    let env = project_client_env(&kit, "shop");
    assert_eq!(env.len(), 1);
    let (name, home) = &env[0];
    assert_eq!(*name, "MYSQL_HOME");
    assert_eq!(*home, kit.app_dir().join("db").join("lines").join("10.6"));
    let ini = fs::read_to_string(home.join("my.ini")).unwrap();
    assert!(
        ini.contains("port=3307") && ini.contains("user=root"),
        "{ini}"
    );
    assert!(
        project_client_env(&kit, "blog").is_empty(),
        "the server's version"
    );
    assert!(auth::write_line_client_ini(&kit, "..", 3307, "pw").is_err());
}

/// T3.8.8: every catalog need for a SQL server (phpMyAdmin's `needs`, a
/// framework's `database`, an addition's) resolves through `sql_need` to
/// MySQL on a new kit, to MariaDB on a kit that runs only MariaDB; other
/// needs stay as they are. The callers outside `db` (setup runner, plan,
/// detection) are wired by the coordinator (requests of p7).
#[test]
fn a_new_kit_needs_mysql_for_every_sql_need() {
    use crate::db::services::sql_need_with;
    use crate::projects::frameworks::official;
    use crate::setup::catalog::Catalog;
    let mut needs: Vec<String> = Catalog::get()
        .find("phpmyadmin")
        .expect("phpmyadmin in the catalog")
        .needs
        .clone();
    for p in official() {
        needs.extend(p.database.clone());
        needs.extend(p.on.values().filter_map(|v| v.database.clone()));
    }
    assert!(needs.iter().any(|n| n == "mariadb"), "{needs:?}");
    let new_kit = |_: &str| false;
    let mariadb_kit = |id: &str| id == "mariadb";
    let both = |id: &str| id == "mariadb" || id == "mysql";
    for need in &needs {
        let sql = need == "mariadb" || need == "mysql";
        let on_new = sql_need_with(need, new_kit);
        assert_eq!(on_new, if sql { "mysql" } else { need.as_str() }, "{need}");
        assert_ne!(on_new, "mariadb", "a new kit never gets MariaDB: {need}");
        let on_old = sql_need_with(need, mariadb_kit);
        assert_eq!(
            on_old,
            if sql { "mariadb" } else { need.as_str() },
            "{need}"
        );
        assert_eq!(
            sql_need_with(need, both),
            if sql { "mysql" } else { need.as_str() }
        );
    }
    assert_eq!(sql_need_with("mongodb", new_kit), "mongodb");
}

/// MySQL (D-110): the SQL server of a new kit, MariaDB kept where it runs;
/// a project's SQL database stays where it is; the login of another
/// version of MySQL reaches the project's programs.
#[test]
fn mysql_is_the_kits_sql_server() {
    use crate::db::auth;
    use crate::db::services::{project_engine, project_sql_engine, sql_engine, sql_engine_with};
    use crate::db::DbEngine;
    assert_eq!(sql_engine_with(|_| false), DbEngine::MySql, "a new kit");
    assert_eq!(sql_engine_with(|id| id == "mariadb"), DbEngine::MariaDb);
    assert_eq!(sql_engine_with(|id| id == "mysql"), DbEngine::MySql);
    assert_eq!(sql_engine_with(|_| true), DbEngine::MySql, "both: MySQL");

    let tmp = TempDir::new("services-mysql");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let s = Services::new(kit.clone());
    // A new kit: MySQL and MongoDB on the page, no MariaDB.
    let ids: Vec<String> = s.list().into_iter().map(|i| i.id).collect();
    assert_eq!(ids, ["mysql", "mongodb"]);
    assert_eq!(sql_engine(&kit), DbEngine::MySql);
    assert_eq!(
        project_engine(&kit, "shop", DbEngine::MariaDb),
        DbEngine::MySql,
        "`mariadb` of the detection reaches MySQL"
    );
    assert_eq!(
        project_engine(&kit, "shop", DbEngine::MongoDb),
        DbEngine::MongoDb
    );
    let err = s
        .start_for("shop", "mariadb", &KitEnv::default())
        .unwrap_err();
    assert!(
        format!("{err:#}").contains("mysql is not installed"),
        "{err:#}"
    );

    // A kit with MariaDB and no MySQL keeps MariaDB, on the page too.
    fake_server(&kit, "mariadb", "11.4.5");
    assert_eq!(sql_engine(&kit), DbEngine::MariaDb);
    let ids: Vec<String> = s.list().into_iter().map(|i| i.id).collect();
    assert_eq!(ids, ["mariadb", "mysql", "mongodb"]);
    auth::set_project_password(&kit, "old", "mariadb", "pw").unwrap();

    // MySQL installed: new databases there, the old one stays on MariaDB.
    fake_server(&kit, "mysql", "9.7.2");
    let c = Catalog::get().find("mysql").unwrap();
    let older = crate::versions::saved_dir(&kit, c, "8.4");
    fs::create_dir_all(older.join("bin")).unwrap();
    for stem in ["mysqld", "mysql"] {
        let exe = crate::router::bins::exe_name(kit.os(), stem);
        fs::write(older.join("bin").join(exe), b"").unwrap();
    }
    assert_eq!(sql_engine(&kit), DbEngine::MySql);
    assert_eq!(project_sql_engine(&kit, "new"), DbEngine::MySql);
    assert_eq!(project_sql_engine(&kit, "old"), DbEngine::MariaDb);
    assert_eq!(
        project_engine(&kit, "old", DbEngine::MySql),
        DbEngine::MariaDb
    );
    assert_eq!(server_lines(&kit, "mysql"), ["9.7", "8.4"]);
    assert_eq!(line_port(&kit, "mysql", None), 3306);
    assert_eq!(line_port(&kit, "mysql", Some("8.4")), 3307);
    assert!(bin_at(&kit, "mysql", Some("8.4"), "mysqld").starts_with(&older));

    // A project on MySQL 8.4: its port, then the login of 8.4 once written.
    auth::set_project_password(&kit, "shop", "mysql", "pw").unwrap();
    assert_eq!(project_db_port(&kit, "shop"), Some(3306));
    auth::set_project_line(&kit, "shop", "mysql", Some("8.4")).unwrap();
    assert_eq!(project_db_port(&kit, "shop"), Some(3307));
    assert!(
        project_client_env(&kit, "shop").is_empty(),
        "not started yet"
    );
    auth::write_mysql_line_cnf(&kit, "8.4", 3307, "pw").unwrap();
    let env = project_client_env(&kit, "shop");
    let names: Vec<&str> = env.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, ["ATOLLO_MYSQL_CNF", "CP_MYSQL"]);
    let cnf = fs::read_to_string(&env[0].1).unwrap();
    assert!(
        cnf.contains("port=3307\n") && cnf.contains("protocol=TCP\n"),
        "{cnf}"
    );
    assert_eq!(env[1].1, older);
    assert!(auth::write_mysql_line_cnf(&kit, "../x", 3307, "pw").is_err());
}

#[test]
fn line_ports_skip_the_taken_ones() {
    let lines: Vec<String> = ["12.3", "11.4", "10.11", "10.6"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    let mut setting = Setting::default();
    setting.ports.insert("10.6".into(), 3307);
    setting.ports.insert("10.11".into(), 3306);
    let ports = line_ports(&setting, "mariadb", Some("12.3"), &lines);
    assert_eq!(ports["12.3"], 3306);
    assert_eq!(ports["10.6"], 3307, "stored");
    assert_eq!(ports["11.4"], 3308, "first free after the server's");
    assert_eq!(
        ports["10.11"], 3309,
        "stored on the server's port: given again"
    );
    let values: std::collections::BTreeSet<u16> = ports.values().copied().collect();
    assert_eq!(values.len(), 4, "one port each");
}
