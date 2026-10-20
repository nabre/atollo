use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn settings_round_trip_and_defaults() {
    let tmp = TempDir::new("services");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let s = Services::new(kit.clone());
    assert_eq!(line_port(&kit, "mysql", None), 3306);
    assert_eq!(line_port(&kit, "mongodb", None), 27017);
    s.set("mysql", Some(3307), true, "it").unwrap();
    s.set("mongodb", Some(27017), false, "it").unwrap();
    let all = load_settings(&kit);
    assert_eq!(
        all["mysql"],
        Setting {
            port: Some(3307),
            autostart: true,
            ..Setting::default()
        }
    );
    assert_eq!(all["mongodb"].port, None, "the default port is not stored");
    assert!(s.set("mysql", Some(80), false, "it").is_err());
    assert!(s.set("redis", None, false, "it").is_err());
    s.set("mysql", Some(3308), false, "it").unwrap();
    let list = s.list();
    let ids: Vec<&str> = list.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(ids, ["mysql", "mongodb"]);
    assert_eq!(list[0].port, 3308);
    assert_eq!(list[0].url, "mysql://root@127.0.0.1:3308/");
    assert_eq!(list[0].user, "root");
    assert_eq!(
        list[1].url,
        "mongodb://admin@127.0.0.1:27017/?authSource=admin"
    );
    assert!(list.iter().all(|i| i.password.is_none()), "no password yet");
    assert!(s.project_dbs("shop").is_empty());
    assert!(
        s.project_db_create("shop", "mysql").is_err(),
        "server not running"
    );
    assert!(list
        .iter()
        .all(|i| i.state == State::Stopped && !i.installed));
    assert!(
        s.start("mysql", &KitEnv::default()).is_err(),
        "not installed"
    );
    assert!(
        s.project_db_drop("shop", "mysql", &KitEnv::default())
            .is_err(),
        "not installed: nothing dropped"
    );
    assert_eq!(s.running(), 0);
}

/// BUG-V5-02: ports stored by starts at the same time are all kept.
#[test]
fn keep_port_loses_no_entry() {
    let tmp = TempDir::new("services-keep-port");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let s = Services::new(kit.clone());
    s.set("mysql", None, true, "it").unwrap();
    std::thread::scope(|scope| {
        for i in 0..16u16 {
            let s = &s;
            scope.spawn(move || s.keep_port("mysql", &format!("8.{i}"), 3307 + i).unwrap());
        }
    });
    let all = load_settings(&kit);
    assert_eq!(all["mysql"].ports.len(), 16, "{:?}", all["mysql"].ports);
    assert_eq!(all["mysql"].ports["8.15"], 3322);
    assert!(all["mysql"].autostart, "the rest of the file kept");
}

/// BUG-V5-02: a start, a stop or a change of version of a server waits for
/// the one in progress (here a turn held by the test), so the check of
/// what runs and the start happen in the same turn.
#[test]
fn operations_on_a_server_take_turns() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let tmp = TempDir::new("services-turns");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let s = Services::new(kit);
    let done = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let turn = s.op(DbEngine::MySql);
        let (s, done) = (&s, &done);
        let start = scope.spawn(move || {
            let r = s.start("mysql", &KitEnv::default());
            done.fetch_add(1, Ordering::SeqCst);
            r
        });
        scope.spawn(move || {
            s.stop("mysql");
            done.fetch_add(1, Ordering::SeqCst);
        });
        scope.spawn(move || {
            let _ = s.set_line("mysql", None);
            done.fetch_add(1, Ordering::SeqCst);
        });
        // Another engine has its own turn.
        s.stop("mongodb");
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(done.load(Ordering::SeqCst), 0, "all wait for the turn");
        drop(turn);
        let err = start.join().unwrap().unwrap_err();
        assert!(format!("{err:#}").contains("not installed"), "{err:#}");
    });
    assert_eq!(done.load(Ordering::SeqCst), 3);
}

/// BUG-V5-04: each run of a tool has a login file of its own.
#[test]
fn login_files_differ_per_run() {
    let tmp = TempDir::new("services-login-files");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    let a = tools::login_file(&kit, "mysql.exe", "cnf");
    let b = tools::login_file(&kit, "mysql.exe", "cnf");
    assert_ne!(a, b);
    assert_eq!(a.parent(), Some(auth::dir(&kit).as_path()));
    let name = a.file_name().unwrap().to_string_lossy().into_owned();
    assert!(
        name.starts_with(&format!("mysql.exe-{}-", std::process::id())) && name.ends_with(".cnf"),
        "{name}"
    );
}

/// Deleting a project drops its database from every version it was created
/// on (`used`), then the one it is on now, each once; a `db.json` of before
/// has only the one it is on.
#[test]
fn drop_lines_cover_every_version_used() {
    let tmp = TempDir::new("services-drop-lines");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let file = kit
        .project_data()
        .join("shop")
        .join("private")
        .join("db.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    assert!(project_drop_lines(&kit, "shop", "mysql").is_empty());
    fs::write(&file, r#"{ "mysql": "pw", "lines": { "mysql": "8.0" } }"#).unwrap();
    assert_eq!(project_drop_lines(&kit, "shop", "mysql"), ["8.0"]);
    auth::add_project_used_line(&kit, "shop", "mysql", "9.7").unwrap();
    auth::add_project_used_line(&kit, "shop", "mysql", "8.0").unwrap();
    assert_eq!(project_drop_lines(&kit, "shop", "mysql"), ["9.7", "8.0"]);
    auth::set_project_line(&kit, "shop", "mysql", Some("8.4")).unwrap();
    assert_eq!(
        project_drop_lines(&kit, "shop", "mysql"),
        ["9.7", "8.0", "8.4"]
    );
    assert!(project_drop_lines(&kit, "shop", "mongodb").is_empty());
}

/// The project user's own login over TCP: a query on its database works,
/// one on `mysql.user` is refused.
fn as_project_user(
    s: &Services,
    engine: DbEngine,
    env: &KitEnv,
    port: u16,
    db: &auth::ProjectDb,
) -> (String, bool) {
    let (mut cmd, login) = s
        .sql_as_project(
            engine,
            engine.client_bin(),
            env,
            None,
            port,
            &db.user,
            &db.password,
        )
        .unwrap();
    cmd.args(["-N", "-e", "SELECT CURRENT_USER(), DATABASE()"])
        .arg(&db.database);
    let own = cmd.output().unwrap();
    let (mut cmd, login2) = s
        .sql_as_project(
            engine,
            engine.client_bin(),
            env,
            None,
            port,
            &db.user,
            &db.password,
        )
        .unwrap();
    cmd.args(["-N", "-e", "SELECT COUNT(*) FROM mysql.user"]);
    let other = cmd.output().unwrap();
    let _ = fs::remove_file(login);
    let _ = fs::remove_file(login2);
    assert!(
        own.status.success(),
        "{engine}: {}",
        String::from_utf8_lossy(&own.stderr)
    );
    (
        String::from_utf8_lossy(&own.stdout).trim().to_owned(),
        other.status.success(),
    )
}

/// A real MySQL and MongoDB of the kit given by ATOLLO_TEST_KIT,
/// on free ports: start, project database and user (a TCP login as that
/// user), copy and restore, drop, stop with nothing left.
#[test]
fn real_databases_start_and_stop() {
    let Ok(root) = std::env::var("ATOLLO_TEST_KIT") else {
        eprintln!("ATOLLO_TEST_KIT not set: skipped");
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
    for (id, port) in [("mysql", 13316), ("mongodb", 37017)] {
        if !installed(&kit, id) {
            eprintln!("{id} not installed: skipped");
            continue;
        }
        let engine = DbEngine::parse(id).unwrap();
        // One project per server.
        let project = format!("Prova {id}");
        let database = format!("prova_{id}");
        let before = load_settings(&kit).get(id).cloned().unwrap_or_default();
        s.set(id, Some(port), false, "it").unwrap();
        assert_eq!(s.start(id, &env).unwrap(), port, "{id}");
        assert!(responds(port), "{id} answers");
        eprintln!("{id}: started on {port}");
        let info = s.list().into_iter().find(|i| i.id == id).unwrap();
        assert_eq!(info.state, State::Running);
        assert!(info.password.is_some(), "{id}: administrator password set");
        let db = s.project_db_create(&project, id).unwrap();
        assert_eq!(db.database, database);
        assert_eq!(db.engine, id);
        let again = s.project_db_create(&project, id).unwrap();
        assert_eq!(again.password, db.password, "{id}: same password");
        assert_eq!(
            s.project_dbs(&project)
                .iter()
                .filter(|d| d.engine == id)
                .count(),
            1
        );
        eprintln!("{id}: database and user {database} created");
        if engine.is_sql() {
            let (who, other) = as_project_user(&s, engine, &env, port, &db);
            assert!(who.contains(&database), "{id}: {who}");
            assert!(!other, "{id}: the project's user reads only its database");
            eprintln!("{id}: TCP login as {who}");
        }
        // Export, lose the data, restore: the data is back (D-78).
        let admin = |script: &str| -> String {
            match engine {
                DbEngine::MongoDb => {
                    let mut cmd = jobs::kit_command(bin(&kit, "mongosh", "mongosh.exe"), &env);
                    cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
                        .arg(port.to_string())
                        .args(["--eval", &format!(
                            "db.getSiblingDB('admin').auth('admin', process.env.ATOLLO_DB_ADMIN); {script}"
                        )])
                        .env("ATOLLO_DB_ADMIN", auth::load(&kit).mongodb_admin.unwrap());
                    let out = cmd.output().unwrap();
                    String::from_utf8_lossy(&out.stdout).trim().to_owned()
                }
                DbEngine::MySql => {
                    let mut cmd = s.sql_tool_at(engine, engine.client_bin(), &env, None, port);
                    cmd.args(["-N", "-e", script]);
                    let out = cmd.output().unwrap();
                    String::from_utf8_lossy(&out.stdout).trim().to_owned()
                }
            }
        };
        let (fill, count, lose) = match engine {
            DbEngine::MongoDb => (
                format!("db.getSiblingDB('{database}').t.insertOne({{x: 7}})"),
                format!("print(db.getSiblingDB('{database}').t.countDocuments())"),
                format!("db.getSiblingDB('{database}').t.drop()"),
            ),
            DbEngine::MySql => (
                format!(
                    "CREATE TABLE IF NOT EXISTS {database}.t (x INT); INSERT INTO {database}.t VALUES (7);"
                ),
                format!("SELECT COUNT(*) FROM {database}.t"),
                format!("DROP TABLE {database}.t"),
            ),
        };
        admin(&fill);
        // A trigger of the project's user: its dump has `DELIMITER ;;`,
        // which the restore must still read with the client's commands off.
        if engine.is_sql() {
            let (mut cmd, login) = s
                .sql_as_project(
                    engine,
                    engine.client_bin(),
                    &env,
                    None,
                    port,
                    &db.user,
                    &db.password,
                )
                .unwrap();
            cmd.arg("-e")
                .arg("CREATE TRIGGER t_bi BEFORE INSERT ON t FOR EACH ROW SET NEW.x = NEW.x")
                .arg(&db.database);
            let out = cmd.output().unwrap();
            let _ = fs::remove_file(login);
            assert!(
                out.status.success(),
                "{id}: trigger: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        let n = admin(&count);
        assert!(
            n.parse::<u32>().is_ok_and(|n| n > 0),
            "{id}: data written, count {n:?}"
        );
        // The explorer sees the database, the table and its rows (D-79).
        let dbs = crate::db::browse::tree(&s, id, None).unwrap();
        let listed = dbs
            .iter()
            .find(|d| d.name == database)
            .expect("database listed");
        assert!(
            listed.tables.iter().any(|t| t.name == "t"),
            "{id}: table listed"
        );
        assert!(
            dbs.iter().any(|d| d.system),
            "{id}: system databases listed"
        );
        let page = crate::db::browse::rows(&s, id, None, &database, "t", 0).unwrap();
        assert!(page.total > 0 && !page.rows.is_empty(), "{id}: rows read");
        assert!(page.columns.iter().any(|c| c.name == "x"), "{id}: column x");
        if engine.is_sql() {
            assert!(crate::db::browse::rows(&s, id, None, &database, "no`; DROP", 0).is_err());
        }
        // phpMyAdmin logged in with the token only, ended with the server (D-107).
        let pma = (engine.is_sql() && installed(&kit, "phpmyadmin")).then(|| {
            let url = s.phpmyadmin_url(engine, None, "en").unwrap();
            crate::db::phpmyadmin::tests::assert_logs_in(&url);
            assert_eq!(
                s.phpmyadmin_url(engine, None, "en").unwrap(),
                url,
                "the same phpMyAdmin"
            );
            eprintln!("{id}: phpMyAdmin logged in");
            url
        });
        let file = s.export(&project, id).unwrap();
        assert!(file.is_file(), "{id}: copy written");
        eprintln!(
            "{id}: copy {} ({} bytes)",
            file.display(),
            fs::metadata(&file).map(|m| m.len()).unwrap_or(0)
        );
        admin(&lose);
        assert_ne!(admin(&count), n, "{id}: data lost before restore");
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        assert!(s
            .backups(&project)
            .iter()
            .any(|b| b.file == name && b.engine == id));
        s.restore(&project, &name).unwrap();
        assert_eq!(admin(&count), n, "{id}: data back after restore");
        if engine.is_sql() {
            let triggers = admin(&format!(
                "SELECT COUNT(*) FROM information_schema.TRIGGERS WHERE TRIGGER_SCHEMA = '{database}'"
            ));
            assert_eq!(triggers, "1", "{id}: the trigger is back");
        }
        eprintln!("{id}: restored as the project's user, {n} row(s) back");
        assert!(s.restore(&project, r"..\x.sql").is_err());
        let _ = fs::remove_file(&file);
        // A copy from elsewhere with the client's own commands: none runs
        // (`--commands=OFF`), whatever the restore answers.
        if engine.is_sql() {
            let dir = s.backup_dir(&project).unwrap();
            let pwned = dir.join("pwned.txt");
            let tee = dir.join("tee.txt");
            let evil = format!(
                "SELECT 1;\n\\! echo pwned > {p}\nsystem echo pwned > {p}\n\
                 tee {t}\nSELECT 2;\n",
                p = pwned.display(),
                t = tee.display()
            );
            fs::write(dir.join("evil-1.sql"), evil).unwrap();
            let answer = s.restore(&project, "evil-1.sql");
            eprintln!("{id}: restore of a copy with client commands: {answer:?}");
            assert!(!pwned.exists(), "{id}: a client command ran");
            assert!(!tee.exists(), "{id}: tee ran");
            let _ = fs::remove_file(dir.join("evil-1.sql"));
        }
        // Deleting the project drops its database and user: a stopped
        // server is started for it and stopped again (D-89).
        s.stop(id);
        assert!(!responds(port), "{id}: stopped");
        s.project_db_drop(&project, id, &env).unwrap();
        assert!(!responds(port), "{id}: stopped again after the drop");
        s.start(id, &env).unwrap();
        assert!(
            !crate::db::browse::tree(&s, id, None)
                .unwrap()
                .iter()
                .any(|d| d.name == database),
            "{id}: database dropped"
        );
        s.project_db_drop(&project, id, &env).unwrap();
        assert!(responds(port), "{id}: a running server stays running");
        s.stop(id);
        assert!(!responds(port), "{id} stopped");
        if let Some(url) = pma {
            assert!(ureq::get(&url).call().is_err(), "phpMyAdmin stopped");
        }
        assert_eq!(s.running(), 0);
        s.set(id, before.port, before.autostart, "it").unwrap();
        eprintln!("{id}: dropped and stopped");
    }
}
