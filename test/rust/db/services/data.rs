use super::*;
use crate::db::services::{
    bin_at, line_port, line_ports, lines_in_use, load_settings, log_path, project_client_env,
    project_db_port, project_line, server_lines, Setting,
};
use crate::kit::{tests::TempDir, Arch};

/// `id` installed at `version`, the default folder of its line, with the
/// key paths of the kit's target (`bin/mysqld.exe` on Windows,
/// `bin/mysqld` elsewhere).
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

/// The MySQL programs of a line folder of an older version, as the
/// kit's target names them.
fn fake_mysql_line(kit: &Kit, dir: &std::path::Path) {
    fs::create_dir_all(dir.join("bin")).unwrap();
    for stem in ["mysqld", "mysql"] {
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
    fake_server(&kit, "mysql", "8.4.5");
    let root = kit.data().join("db").join("mysql");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("mysql.ibd"), b"x").unwrap();
    fs::write(root.join("ibdata1"), b"x").unwrap();
    migrate_data(&kit);
    assert!(root.join("8.4").join("mysql.ibd").is_file() && !root.join("mysql.ibd").exists());
    assert_eq!(data_dir(&kit, "mysql"), root.join("8.4"));

    // A newer version starts with a copy; the old data stays.
    let c = Catalog::get().find("mysql").unwrap();
    let newer = crate::versions::saved_dir(&kit, c, "9.7");
    fake_mysql_line(&kit, &newer);
    assert_eq!(server_line(&kit, "mysql").as_deref(), Some("9.7"));
    assert_eq!(server_lines(&kit, "mysql"), ["9.7", "8.4"]);
    assert_eq!(
        seed(&kit, "mysql", "9.7", &[]).unwrap().as_deref(),
        Some("8.4")
    );
    assert!(
        root.join("9.7").join("ibdata1").is_file() && root.join("8.4").join("ibdata1").is_file()
    );
    assert_eq!(
        seed(&kit, "mysql", "9.7", &[]).unwrap(),
        None,
        "only the first time"
    );
    // Back to the older version: its own data, a setting of the server.
    let s = Services::new(kit.clone());
    s.set_line("mysql", Some("8.4")).unwrap();
    assert_eq!(data_dir(&kit, "mysql"), root.join("8.4"));
    assert!(s.set_line("mysql", Some("8.0")).is_err(), "not installed");
    assert_eq!(
        bin(&kit, "mysql", "mysqld.exe")
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap(),
        "8.4"
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

/// The data of a running version is never copied (D-102).
#[test]
fn no_copy_of_a_running_version() {
    let tmp = TempDir::new("services-seed-running");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let root = kit.data().join("db").join("mysql");
    fs::create_dir_all(root.join("8.0")).unwrap();
    fs::write(root.join("8.0").join("mysql.ibd"), b"").unwrap();
    let err = seed(&kit, "mysql", "8.4", &["8.0".to_owned()]).unwrap_err();
    assert!(format!("{err:#}").contains("stop mysql 8.0"), "{err:#}");
    assert!(!root.join("8.4").exists(), "nothing copied");
    assert_eq!(
        seed(&kit, "mysql", "8.4", &["8.4".to_owned()])
            .unwrap()
            .as_deref(),
        Some("8.0")
    );
}

/// Two versions of MySQL side by side (D-102): the one the
/// server runs on its port, the other on the next free one, stored at
/// its first start; each with its data, program and log; the project's
/// database on the version chosen for it, at that version's port.
#[test]
fn versions_side_by_side() {
    let tmp = TempDir::new("services-side");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    fake_server(&kit, "mysql", "9.3.1");
    let c = Catalog::get().find("mysql").unwrap();
    let older = crate::versions::saved_dir(&kit, c, "8.0");
    fake_mysql_line(&kit, &older);
    assert_eq!(server_lines(&kit, "mysql"), ["9.3", "8.0"]);
    assert_eq!(server_line(&kit, "mysql").as_deref(), Some("9.3"));
    assert_eq!(line_port(&kit, "mysql", None), 3306);
    assert_eq!(line_port(&kit, "mysql", Some("9.3")), 3306);
    assert_eq!(line_port(&kit, "mysql", Some("8.0")), 3307);
    assert_eq!(
        data_dir_at(&kit, "mysql", Some("8.0")),
        kit.data().join("db").join("mysql").join("8.0")
    );
    assert!(bin_at(&kit, "mysql", Some("8.0"), "mysqld.exe").starts_with(&older));
    assert_eq!(
        log_path(&kit, "mysql", Some("8.0")),
        kit.logs().join("mysql-8.0.log")
    );
    assert_eq!(
        log_path(&kit, "mysql", Some("9.3")),
        kit.logs().join("mysql.log")
    );

    let s = Services::new(kit.clone());

    // Ports of the other versions: stored, never the server's nor another's.
    s.set_line_port("mysql", "8.0", Some(3316), "it").unwrap();
    assert_eq!(line_port(&kit, "mysql", Some("8.0")), 3316);
    assert!(s.set_line_port("mysql", "8.0", Some(3306), "it").is_err());
    assert!(s.set_line_port("mysql", "8.0", Some(80), "it").is_err());
    assert!(s.set_line_port("mysql", "9.3", Some(3320), "it").is_err());
    assert!(s.set_line_port("mysql", "9.0", Some(3320), "it").is_err());
    s.set("mysql", Some(3310), true, "it").unwrap();
    assert_eq!(
        load_settings(&kit)["mysql"].ports["8.0"],
        3316,
        "the server's settings keep the other ports"
    );
    s.set_line_port("mysql", "8.0", None, "it").unwrap();
    assert_eq!(line_port(&kit, "mysql", Some("8.0")), 3311);
    s.set("mysql", None, false, "it").unwrap();
    assert!(s
        .start_line("mysql", Some("9.0"), &KitEnv::default())
        .is_err());

    // The project's database follows the version chosen for it.
    crate::db::auth::set_project_password(&kit, "shop", "mysql", "pw").unwrap();
    assert_eq!(project_db_port(&kit, "shop"), Some(3306));
    assert_eq!(
        project_db_port(&kit, "blog"),
        None,
        "no database: no DB_PORT"
    );
    assert!(s
        .project_db_line("shop", "mysql", Some("9.0"), &KitEnv::default())
        .is_err());
    crate::db::auth::set_project_line(&kit, "shop", "mysql", Some("8.0")).unwrap();
    assert_eq!(project_line(&kit, "shop", "mysql").as_deref(), Some("8.0"));
    assert_eq!(project_db_port(&kit, "shop"), Some(3307));
    let dbs = s.project_dbs("shop");
    assert_eq!((dbs[0].port, dbs[0].url.contains(":3307/")), (3307, true));
    let err = s.project_db_create("shop", "mysql").unwrap_err();
    assert!(
        format!("{err:#}").contains("mysql 8.0 is not running"),
        "{err:#}"
    );

    // A saved state of before (no `ports`) reads as it was.
    fs::write(
        kit.app_dir().join("services.json"),
        r#"{ "mysql": { "port": 3308, "autostart": true } }"#,
    )
    .unwrap();
    let old = &load_settings(&kit)["mysql"];
    assert!(old.autostart && old.ports.is_empty());
    assert_eq!(line_port(&kit, "mysql", Some("8.0")), 3309);
}

/// BUG-V5-05: a server or a version of it is never set on the port of a
/// pool, of a project's action or the editor's, and the owner is named;
/// a port it already has stays its own.
#[test]
fn ports_of_pools_projects_and_editor_refused() {
    let tmp = TempDir::new("services-ports-taken");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    fake_server(&kit, "mysql", "9.3.1");
    let c = Catalog::get().find("mysql").unwrap();
    fake_mysql_line(&kit, &crate::versions::saved_dir(&kit, c, "8.0"));
    fs::write(
        kit.app_dir().join("servers.json"),
        r#"{ "ports": { "legacy": 3320 } }"#,
    )
    .unwrap();
    let shop = kit.project_data().join("shop");
    fs::create_dir_all(&shop).unwrap();
    fs::write(shop.join("ports.json"), r#"{ "ports": { "site": 7150 } }"#).unwrap();
    let s = Services::new(kit.clone());

    let refused = |r: Result<()>| format!("{:#}", r.unwrap_err());
    let pool = "Port 3320 already belongs to server legacy: choose another one.";
    assert_eq!(refused(s.set("mysql", Some(3320), false, "en")), pool);
    assert_eq!(
        refused(s.set_line_port("mysql", "8.0", Some(3320), "en")),
        pool
    );
    assert_eq!(
        refused(s.set_line_port("mysql", "8.0", Some(7150), "en")),
        "Port 7150 already belongs to project shop → site: choose another one."
    );
    assert_eq!(
        refused(s.set("mongodb", Some(7010), false, "en")),
        "Ports 7000 to 7099 belong to the editor"
    );
    assert!(load_settings(&kit).is_empty(), "nothing stored");

    // Its own ports, and those of the versions of the same server (which
    // its own checks move or refuse), are no other owner's.
    s.set("mysql", None, true, "en").unwrap();
    s.set_line_port("mysql", "8.0", Some(3307), "en").unwrap();
    s.set("mysql", Some(3310), true, "en").unwrap();
    assert_eq!(line_port(&kit, "mysql", None), 3310);
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
    fake_server(&kit, "mysql", "9.3.1");
    let c = Catalog::get().find("mysql").unwrap();
    let older = crate::versions::saved_dir(&kit, c, "8.0");
    fake_mysql_line(&kit, &older);
    assert!(lines_in_use(&kit, "mysql").is_empty());
    // chosen, no database yet: not started
    auth::set_project_line(&kit, "blog", "mysql", Some("8.0")).unwrap();
    auth::set_project_password(&kit, "blog", "mysql", "pw").unwrap();
    auth::set_project_line(&kit, "blog", "mysql", None).unwrap();
    auth::set_project_password(&kit, "shop", "mysql", "pw").unwrap();
    auth::set_project_line(&kit, "shop", "mysql", Some("9.3")).unwrap();
    assert!(
        lines_in_use(&kit, "mysql").is_empty(),
        "the server's own version is started anyway"
    );
    auth::set_project_line(&kit, "shop", "mysql", Some("8.0")).unwrap();
    auth::set_project_password(&kit, "wiki", "mysql", "pw").unwrap();
    auth::set_project_line(&kit, "wiki", "mysql", Some("9.0")).unwrap();
    assert_eq!(lines_in_use(&kit, "mysql"), ["8.0"], "9.0 not installed");
    assert!(lines_in_use(&kit, "mongodb").is_empty());

    // A version uninstalled is told as such, for the interface to offer it.
    let s = Services::new(kit.clone());
    let missing = |e: anyhow::Error| {
        e.downcast_ref::<crate::db::services::LineMissing>()
            .map(|m| (m.id.clone(), m.line.clone()))
    };
    let err = s
        .start_for("wiki", "mysql", &KitEnv::default())
        .unwrap_err();
    assert_eq!(missing(err), Some(("mysql".into(), "9.0".into())));
    let err = s
        .project_db_line("wiki", "mysql", Some("9.1"), &KitEnv::default())
        .unwrap_err();
    assert_eq!(missing(err), Some(("mysql".into(), "9.1".into())));

    // The login of a project on another version, once its start wrote it;
    // none on the server's version.
    assert!(
        project_client_env(&kit, "shop").is_empty(),
        "not started yet"
    );
    auth::write_mysql_line_cnf(&kit, "8.0", 3307, "pw").unwrap();
    assert_eq!(project_client_env(&kit, "shop").len(), 2);
    assert!(
        project_client_env(&kit, "blog").is_empty(),
        "the server's version"
    );
}

/// MySQL (D-110): the kit's SQL server; the login of another version of
/// MySQL reaches the project's programs.
#[test]
fn mysql_is_the_kits_sql_server() {
    use crate::db::auth;
    let tmp = TempDir::new("services-mysql");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let s = Services::new(kit.clone());
    // A new kit: MySQL and MongoDB on the page.
    let ids: Vec<String> = s.list().into_iter().map(|i| i.id).collect();
    assert_eq!(ids, ["mysql", "mongodb"]);
    let err = s
        .start_for("shop", "mysql", &KitEnv::default())
        .unwrap_err();
    assert!(
        format!("{err:#}").contains("mysql is not installed"),
        "{err:#}"
    );

    // MySQL 9.7 and 8.4 installed: each on its port.
    fake_server(&kit, "mysql", "9.7.2");
    let c = Catalog::get().find("mysql").unwrap();
    let older = crate::versions::saved_dir(&kit, c, "8.4");
    fs::create_dir_all(older.join("bin")).unwrap();
    for stem in ["mysqld", "mysql"] {
        let exe = crate::router::bins::exe_name(kit.os(), stem);
        fs::write(older.join("bin").join(exe), b"").unwrap();
    }
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
    let lines: Vec<String> = ["9.7", "9.3", "8.4", "8.0"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    let mut setting = Setting::default();
    setting.ports.insert("8.0".into(), 3307);
    setting.ports.insert("8.4".into(), 3306);
    setting.ports.insert("9.3".into(), 3307);
    let ports = line_ports(&setting, "mysql", Some("9.7"), &lines);
    assert_eq!(ports["8.4"], 3306, "stored, also on the server's port");
    assert_eq!(ports["9.3"], 3307, "stored, first in order");
    assert_eq!(
        ports["9.7"], 3308,
        "the server's port taken: the first free"
    );
    assert_eq!(ports["8.0"], 3309, "stored on another's port: given again");
    let values: std::collections::BTreeSet<u16> = ports.values().copied().collect();
    assert_eq!(values.len(), 4, "one port each");
}

/// BUG-V5-02: another version chosen for the server moves no port, of a
/// version started (stored at its start) or not.
#[test]
fn ports_stay_when_the_server_changes_version() {
    let lines: Vec<String> = ["9.7", "8.4"].iter().map(|s| (*s).to_owned()).collect();
    let mut setting = Setting::default();
    setting.ports.insert("9.7".into(), 3306);
    setting.ports.insert("8.4".into(), 3307);
    let before = line_ports(&setting, "mysql", Some("9.7"), &lines);
    let after = line_ports(&setting, "mysql", Some("8.4"), &lines);
    assert_eq!(before, after);
    assert_eq!((after["9.7"], after["8.4"]), (3306, 3307));

    let tmp = TempDir::new("services-main-ports");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    fake_server(&kit, "mysql", "9.3.1");
    let c = Catalog::get().find("mysql").unwrap();
    fake_mysql_line(&kit, &crate::versions::saved_dir(&kit, c, "8.0"));
    let s = Services::new(kit.clone());
    let ports = |kit: &Kit| {
        (
            line_port(kit, "mysql", Some("9.3")),
            line_port(kit, "mysql", Some("8.0")),
        )
    };
    assert_eq!(ports(&kit), (3306, 3307));
    // Never started: the change stores them.
    s.set_line("mysql", Some("8.0")).unwrap();
    assert_eq!(server_line(&kit, "mysql").as_deref(), Some("8.0"));
    assert_eq!(ports(&kit), (3306, 3307));
    assert_eq!(line_port(&kit, "mysql", None), 3307, "the server's version");
    assert_eq!(s.list()[0].port, 3307);
    s.set_line("mysql", None).unwrap();
    assert_eq!(ports(&kit), (3306, 3307));

    // A new port of the server is the one of the version it runs, unless
    // another version has it.
    assert!(s.set("mysql", Some(3307), false, "it").is_err());
    s.set("mysql", Some(3310), false, "it").unwrap();
    assert_eq!(ports(&kit), (3310, 3307));
    s.set("mysql", Some(3310), true, "it").unwrap();
    assert_eq!(ports(&kit), (3310, 3307), "the same port: nothing moves");
}
