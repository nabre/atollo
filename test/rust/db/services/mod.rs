use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn settings_round_trip_and_defaults() {
    let tmp = TempDir::new("services");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let s = Services::new(kit.clone());
    assert_eq!(line_port(&kit, "mariadb", None), 3306);
    assert_eq!(line_port(&kit, "mongodb", None), 27017);
    s.set("mariadb", Some(3307), true).unwrap();
    s.set("mongodb", Some(27017), false).unwrap();
    let all = load_settings(&kit);
    assert_eq!(
        all["mariadb"],
        Setting {
            port: Some(3307),
            autostart: true,
            ..Setting::default()
        }
    );
    assert_eq!(all["mongodb"].port, None, "the default port is not stored");
    assert!(s.set("mariadb", Some(80), false).is_err());
    assert!(s.set("redis", None, false).is_err());
    let list = s.list();
    assert_eq!(list[0].port, 3307);
    assert_eq!(list[0].url, "mysql://root@127.0.0.1:3307/");
    assert_eq!(
        list[1].url,
        "mongodb://admin@127.0.0.1:27017/?authSource=admin"
    );
    assert!(list.iter().all(|i| i.password.is_none()), "no password yet");
    assert!(s.project_dbs("shop").is_empty());
    assert!(
        s.project_db_create("shop", "mariadb").is_err(),
        "server not running"
    );
    assert!(list
        .iter()
        .all(|i| i.state == State::Stopped && !i.installed));
    assert!(
        s.start("mariadb", &KitEnv::default()).is_err(),
        "not installed"
    );
    assert!(
        s.project_db_drop("shop", "mariadb", &KitEnv::default())
            .is_err(),
        "not installed: nothing dropped"
    );
    assert_eq!(s.running(), 0);
}

/// A real MariaDB and MongoDB of the kit given by DEVKIT_TEST_KIT, on
/// free ports: start, answer, stop with nothing left.
#[test]
fn real_databases_start_and_stop() {
    let Ok(root) = std::env::var("DEVKIT_TEST_KIT") else {
        eprintln!("DEVKIT_TEST_KIT not set: skipped");
        return;
    };
    let kit = Kit::at(PathBuf::from(root), Arch::detect());
    kit.prepare().unwrap();
    let env = KitEnv::build(
        &kit,
        &crate::settings::Settings::default(),
        "en",
        &crate::env::Host::capture(),
    );
    let s = Services::new(kit.clone());
    for (id, port) in [("mariadb", 13306), ("mongodb", 37017)] {
        if !installed(&kit, id) {
            eprintln!("{id} not installed: skipped");
            continue;
        }
        let before = load_settings(&kit).get(id).cloned().unwrap_or_default();
        s.set(id, Some(port), false).unwrap();
        assert_eq!(s.start(id, &env).unwrap(), port, "{id}");
        assert!(responds(port), "{id} answers");
        let info = s.list().into_iter().find(|i| i.id == id).unwrap();
        assert_eq!(info.state, State::Running);
        assert!(info.password.is_some(), "{id}: administrator password set");
        let db = s.project_db_create("Prova Kit", id).unwrap();
        assert_eq!(db.database, "prova_kit");
        let again = s.project_db_create("Prova Kit", id).unwrap();
        assert_eq!(again.password, db.password, "{id}: same password");
        assert_eq!(
            s.project_dbs("Prova Kit")
                .iter()
                .filter(|d| d.engine == id)
                .count(),
            1
        );
        // Export, lose the data, restore: the data is back (D-78).
        let admin = |script: &str| -> String {
            match id {
                "mongodb" => {
                    let mut cmd = jobs::kit_command(bin(&kit, "mongosh", "mongosh.exe"), &env);
                    cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
                        .arg(port.to_string())
                        .args(["--eval", &format!(
                            "db.getSiblingDB('admin').auth('admin', process.env.DEVKIT_DB_ADMIN); {script}"
                        )])
                        .env("DEVKIT_DB_ADMIN", auth::load(&kit).mongodb_admin.unwrap());
                    let out = cmd.output().unwrap();
                    String::from_utf8_lossy(&out.stdout).trim().to_owned()
                }
                _ => {
                    let mut cmd = s.mariadb_tool("mariadb.exe", &env);
                    cmd.args(["-N", "-e", script]);
                    let out = cmd.output().unwrap();
                    String::from_utf8_lossy(&out.stdout).trim().to_owned()
                }
            }
        };
        let (fill, count, lose) = match id {
            "mongodb" => (
                "db.getSiblingDB('prova_kit').t.insertOne({x: 7})",
                "print(db.getSiblingDB('prova_kit').t.countDocuments())",
                "db.getSiblingDB('prova_kit').t.drop()",
            ),
            _ => (
                "CREATE TABLE IF NOT EXISTS prova_kit.t (x INT); INSERT INTO prova_kit.t VALUES (7);",
                "SELECT COUNT(*) FROM prova_kit.t",
                "DROP TABLE prova_kit.t",
            ),
        };
        admin(fill);
        let n = admin(count);
        assert!(
            n.parse::<u32>().is_ok_and(|n| n > 0),
            "{id}: data written, count {n:?}"
        );
        // The explorer sees the database, the table and its rows (D-79).
        let dbs = crate::db::browse::tree(&s, id).unwrap();
        let db = dbs
            .iter()
            .find(|d| d.name == "prova_kit")
            .expect("database listed");
        assert!(
            db.tables.iter().any(|t| t.name == "t"),
            "{id}: table listed"
        );
        assert!(
            dbs.iter().any(|d| d.system),
            "{id}: system databases listed"
        );
        let page = crate::db::browse::rows(&s, id, "prova_kit", "t", 0).unwrap();
        assert!(page.total > 0 && !page.rows.is_empty(), "{id}: rows read");
        assert!(page.columns.iter().any(|c| c.name == "x"), "{id}: column x");
        if id == "mariadb" {
            assert!(crate::db::browse::rows(&s, id, "prova_kit", "no`; DROP", 0).is_err());
        }
        // Adminer logged in with the token only, ended with MariaDB (D-85).
        let adminer = (id == "mariadb" && installed(&kit, "adminer")).then(|| {
            let url = s.adminer_url().unwrap();
            crate::db::adminer::tests::assert_logs_in(&url);
            assert_eq!(s.adminer_url().unwrap(), url, "the same Adminer");
            url
        });
        let file = s.export("Prova Kit", id).unwrap();
        assert!(file.is_file(), "{id}: copy written");
        admin(lose);
        assert_ne!(admin(count), n, "{id}: data lost before restore");
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        assert!(s
            .backups("Prova Kit")
            .iter()
            .any(|b| b.file == name && b.engine == id));
        s.restore("Prova Kit", &name).unwrap();
        assert_eq!(admin(count), n, "{id}: data back after restore");
        assert!(s.restore("Prova Kit", r"..\x.sql").is_err());
        let _ = fs::remove_file(&file);
        // Deleting the project drops its database and user: a stopped
        // server is started for it and stopped again (PLAN-DATA.md WP-5).
        s.stop(id);
        s.project_db_drop("Prova Kit", id, &env).unwrap();
        assert!(!responds(port), "{id}: stopped again after the drop");
        s.start(id, &env).unwrap();
        assert!(
            !crate::db::browse::tree(&s, id)
                .unwrap()
                .iter()
                .any(|d| d.name == "prova_kit"),
            "{id}: database dropped"
        );
        s.project_db_drop("Prova Kit", id, &env).unwrap();
        assert!(responds(port), "{id}: a running server stays running");
        s.stop(id);
        assert!(!responds(port), "{id} stopped");
        if let Some(url) = adminer {
            assert!(ureq::get(&url).call().is_err(), "Adminer stopped");
        }
        assert_eq!(s.running(), 0);
        s.set(id, before.port, before.autostart).unwrap();
    }
}
