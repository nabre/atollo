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
    s.set("mysql", Some(3308), false).unwrap();
    let list = s.list();
    // MariaDB has settings, yet it is not a server of the page (D-117).
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
    assert!(project_drop_lines(&kit, "shop", "mariadb").is_empty());
    fs::write(
        &file,
        r#"{ "mariadb": "pw", "lines": { "mariadb": "10.6" } }"#,
    )
    .unwrap();
    assert_eq!(project_drop_lines(&kit, "shop", "mariadb"), ["10.6"]);
    auth::add_project_used_line(&kit, "shop", "mariadb", "12.3").unwrap();
    auth::add_project_used_line(&kit, "shop", "mariadb", "10.6").unwrap();
    assert_eq!(
        project_drop_lines(&kit, "shop", "mariadb"),
        ["12.3", "10.6"]
    );
    auth::set_project_line(&kit, "shop", "mariadb", Some("11.4")).unwrap();
    assert_eq!(
        project_drop_lines(&kit, "shop", "mariadb"),
        ["12.3", "10.6", "11.4"]
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

/// A real MySQL, MariaDB and MongoDB of the kit given by DEVKIT_TEST_KIT,
/// on free ports: start, project database and user (a TCP login as that
/// user), copy and restore, drop, stop with nothing left.
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
    for (id, port) in [("mysql", 13316), ("mariadb", 13306), ("mongodb", 37017)] {
        if !installed(&kit, id) {
            eprintln!("{id} not installed: skipped");
            continue;
        }
        let engine = DbEngine::parse(id).unwrap();
        // One project per server; a SQL one says which server its database
        // is on (a password stored), or a kit with both would put a new one
        // on MySQL (D-110).
        let project = format!("Prova {id}");
        let database = format!("prova_{id}");
        if engine.is_sql()
            && auth::try_project_password(&kit, &project, id)
                .unwrap()
                .is_none()
        {
            auth::set_project_password(&kit, &project, id, &auth::new_password().unwrap()).unwrap();
        }
        let before = load_settings(&kit).get(id).cloned().unwrap_or_default();
        s.set(id, Some(port), false).unwrap();
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
                            "db.getSiblingDB('admin').auth('admin', process.env.DEVKIT_DB_ADMIN); {script}"
                        )])
                        .env("DEVKIT_DB_ADMIN", auth::load(&kit).mongodb_admin.unwrap());
                    let out = cmd.output().unwrap();
                    String::from_utf8_lossy(&out.stdout).trim().to_owned()
                }
                DbEngine::MariaDb | DbEngine::MySql => {
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
            DbEngine::MariaDb | DbEngine::MySql => (
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
        // (`--commands=OFF`, `--sandbox`), whatever the restore answers.
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
        s.set(id, before.port, before.autostart).unwrap();
        eprintln!("{id}: dropped and stopped");
    }
}

/// Hard links of `from` in `to` (a copy where the disk has none): the
/// programs of a kit for a scratch one, removable without touching them.
fn link_tree(from: &std::path::Path, to: &std::path::Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let target = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            link_tree(&e.path(), &target);
        } else if fs::hard_link(e.path(), &target).is_err() {
            fs::copy(e.path(), &target).unwrap();
        }
    }
}

/// The servers of the move, with a restore on `fail` that fails once (an
/// interruption half way).
struct FailOnce<'a> {
    inner: ToMysql<'a>,
    fail: &'a str,
    failed: std::cell::Cell<bool>,
}

impl crate::db::migrate_engine::Servers for FailOnce<'_> {
    fn kit(&self) -> &Kit {
        self.inner.kit()
    }
    fn installed_lines(&self, engine: DbEngine) -> Vec<String> {
        self.inner.installed_lines(engine)
    }
    fn start(&self, engine: DbEngine, line: &str) -> Result<bool> {
        self.inner.start(engine, line)
    }
    fn stop(&self, engine: DbEngine, line: &str) {
        self.inner.stop(engine, line)
    }
    fn running(&self, engine: DbEngine, line: &str) -> bool {
        self.inner.running(engine, line)
    }
    fn sql(&self, engine: DbEngine, line: &str, sql: &str) -> Result<()> {
        self.inner.sql(engine, line, sql)
    }
    fn query(&self, engine: DbEngine, line: &str, sql: &str) -> Result<String> {
        self.inner.query(engine, line, sql)
    }
    fn dump(&self, line: &str, database: &str, to: &std::path::Path) -> Result<()> {
        self.inner.dump(line, database, to)
    }
    fn restore(
        &self,
        line: &str,
        database: &str,
        user: &str,
        password: &str,
        from: &std::path::Path,
    ) -> Result<()> {
        if line == self.fail && !self.failed.replace(true) {
            // Half of it restored, then the stop.
            let text = fs::read(from)?;
            let half = from.with_extension("half");
            fs::write(&half, &text[..text.len() / 2])?;
            let _ = self.inner.restore(line, database, user, password, &half);
            let _ = fs::remove_file(half);
            bail!("interrupted (test)");
        }
        self.inner.restore(line, database, user, password, from)
    }
    fn serve(&self, line: &str) -> Result<()> {
        self.inner.serve(line)
    }
}

/// D-117 with the real servers of DEVKIT_TEST_KIT (MariaDB 12.3 and 11.4,
/// MySQL 9.7 and 8.4 installed there), in a scratch kit beside it with
/// their programs linked: two projects, one per MariaDB version, with a
/// trigger, a view, an Aria table, utf8mb3 and uca1400 collations; a first
/// move interrupted on 8.4 leaves MariaDB working; the second moves all,
/// and the projects' users log in to MySQL on MariaDB's ports with their
/// own passwords.
#[test]
fn real_mariadb_to_mysql() {
    use crate::db::migrate_engine;
    let Ok(root) = std::env::var("DEVKIT_TEST_KIT") else {
        eprintln!("DEVKIT_TEST_KIT not set: skipped");
        return;
    };
    let real = Kit::at(PathBuf::from(&root), Arch::detect());
    for (id, line) in [
        ("mariadb", "12.3"),
        ("mariadb", "11.4"),
        ("mysql", "9.7"),
        ("mysql", "8.4"),
    ] {
        if !server_lines(&real, id).iter().any(|l| l == line) {
            eprintln!("{id} {line} not installed in DEVKIT_TEST_KIT: skipped");
            return;
        }
    }
    let scratch = PathBuf::from(&root)
        .parent()
        .unwrap()
        .join(format!("devkit-move-test-{}", std::process::id()));
    let kit = Kit::at(scratch.clone(), Arch::detect());
    kit.prepare().unwrap();
    for id in ["mariadb", "mysql"] {
        link_tree(&real.apps().join(id), &kit.apps().join(id));
        let v = format!("{id}.txt");
        fs::create_dir_all(kit.versions_dir()).unwrap();
        fs::copy(real.versions_dir().join(&v), kit.versions_dir().join(&v)).unwrap();
    }
    let env = KitEnv::build(
        &kit,
        &crate::settings::Settings::default(),
        "en",
        &crate::env::Host::capture(),
    );
    // The temporary folder the app makes at its start (InnoDB needs it).
    for var in ["TEMP", "TMPDIR"] {
        if let Some(t) = env.get(var) {
            fs::create_dir_all(t).unwrap();
        }
    }
    let s = Services::new(kit.clone());
    s.set_line("mariadb", Some("12.3")).unwrap();
    s.set("mariadb", Some(13406), true).unwrap();
    s.set_line_port("mariadb", "11.4", Some(13407)).unwrap();
    assert_eq!(s.start_line("mariadb", None, &env).unwrap(), 13406);
    assert_eq!(s.start_line("mariadb", Some("11.4"), &env).unwrap(), 13407);
    for p in ["shop", "blog"] {
        fs::create_dir_all(kit.projects().join(p)).unwrap();
        fs::create_dir_all(kit.project_data().join(p)).unwrap();
    }
    auth::set_project_line(&kit, "blog", "mariadb", Some("11.4")).unwrap();
    let shop = s.project_db_create("shop", "mariadb").unwrap();
    let blog = s.project_db_create("blog", "mariadb").unwrap();
    assert_eq!((shop.engine.as_str(), shop.port), ("mariadb", 13406));
    assert_eq!(blog.port, 13407);
    let admin = |engine: DbEngine, line: &str, sql: &str| -> String {
        let (port, env) = s.running_at(engine, Some(line)).unwrap();
        let mut cmd = s.sql_tool_at(engine, engine.client_bin(), &env, Some(line), port);
        cmd.args(["-N", "-e", sql]);
        let out = cmd.output().unwrap();
        assert!(
            out.status.success(),
            "{engine} {line}: {sql}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_owned()
    };
    admin(
        DbEngine::MariaDb,
        "12.3",
        "CREATE TABLE shop.orders (id INT AUTO_INCREMENT PRIMARY KEY, note VARCHAR(50)) \
         DEFAULT CHARSET=utf8mb3; \
         INSERT INTO shop.orders (note) VALUES ('caffè'), ('it''s'), (NULL); \
         CREATE TABLE shop.legacy (x INT) ENGINE=Aria; INSERT INTO shop.legacy VALUES (1), (2); \
         CREATE TABLE shop.names (s VARCHAR(10) COLLATE utf8mb4_uca1400_ai_ci); \
         INSERT INTO shop.names VALUES ('a'); \
         CREATE VIEW shop.notes AS SELECT note FROM shop.orders; \
         CREATE TRIGGER shop.orders_bi BEFORE INSERT ON shop.orders FOR EACH ROW SET NEW.note = NEW.note; \
         CREATE DATABASE by_hand; CREATE TABLE by_hand.x (id INT); INSERT INTO by_hand.x VALUES (1);",
    );
    admin(
        DbEngine::MariaDb,
        "11.4",
        "CREATE TABLE blog.wp_posts (id INT PRIMARY KEY, title TEXT); \
         INSERT INTO blog.wp_posts VALUES (1, 'Hello'), (2, 'World');",
    );
    let note_hex = admin(
        DbEngine::MariaDb,
        "12.3",
        "SELECT HEX(note) FROM shop.orders WHERE id = 1",
    );
    // 11.4 stopped: the move starts it, and stops it after.
    s.stop_line("mariadb", Some("11.4"));
    let no_install = |_: &[(String, String)]| -> Result<()> {
        bail!("only the MySQL versions of DEVKIT_TEST_KIT")
    };

    // An interruption half way through 8.4: MariaDB as before.
    let once = FailOnce {
        inner: ToMysql {
            services: &s,
            env: env.clone(),
        },
        fail: "8.4",
        failed: std::cell::Cell::new(false),
    };
    let err = migrate_engine::run(&once, &no_install).unwrap_err();
    eprintln!("first move: {err:#}");
    assert!(format!("{err:#}").contains("interrupted"), "{err:#}");
    let read_only = admin(DbEngine::MariaDb, "12.3", "SELECT @@read_only");
    assert!(read_only == "0" || read_only == "OFF", "{read_only}");
    assert_eq!(
        admin(
            DbEngine::MariaDb,
            "12.3",
            "SELECT COUNT(*) FROM shop.orders"
        ),
        "3"
    );
    assert!(load_settings(&kit).contains_key("mariadb"));
    assert_eq!(migrate_engine::status(&kit).state, "failed");

    // Again: everything moves.
    let servers = ToMysql {
        services: &s,
        env: env.clone(),
    };
    migrate_engine::run(&servers, &no_install).unwrap();
    assert_eq!(migrate_engine::status(&kit).state, "done");
    assert!(s.running_at(DbEngine::MariaDb, Some("12.3")).is_err());
    let (port, _) = s.running_at(DbEngine::MySql, Some("9.7")).unwrap();
    assert_eq!(port, 13406, "MySQL on MariaDB's port");
    assert_eq!(
        s.start_engine(DbEngine::MySql, Some("8.4"), &env).unwrap(),
        13407
    );
    let count = |line: &str, sql: &str| admin(DbEngine::MySql, line, sql);
    assert_eq!(count("9.7", "SELECT COUNT(*) FROM shop.orders"), "3");
    assert_eq!(count("9.7", "SELECT COUNT(*) FROM shop.legacy"), "2");
    assert_eq!(count("9.7", "SELECT COUNT(*) FROM shop.notes"), "3");
    assert_eq!(
        count("9.7", "SELECT HEX(note) FROM shop.orders WHERE id = 1"),
        note_hex,
        "the same bytes"
    );
    assert_eq!(
        count(
            "9.7",
            "SELECT COUNT(*) FROM information_schema.triggers WHERE trigger_schema = 'shop'"
        ),
        "1"
    );
    assert_eq!(count("8.4", "SELECT COUNT(*) FROM blog.wp_posts"), "2");
    assert_eq!(
        count("9.7", "SELECT COUNT(*) FROM by_hand.x"),
        "1",
        "a database of no project"
    );
    // The projects' users, their passwords, MariaDB's ports.
    for (project, before) in [("shop", &shop), ("blog", &blog)] {
        let db = s.project_dbs(project).into_iter().next().unwrap();
        assert_eq!(
            (db.engine.as_str(), db.port, &db.password),
            ("mysql", before.port, &before.password)
        );
        let (who, other) = as_project_user(&s, DbEngine::MySql, &env, db.port, &db);
        assert!(who.contains(project), "{project}: {who}");
        assert!(!other, "{project}: its own database only");
        eprintln!("{project}: MySQL on {} as {who}", db.port);
    }
    assert!(migrate_engine::kept_dir(&kit).join("12.3").is_dir());
    s.stop_all();
    let _ = crate::net::archive::remove_dir(&scratch);
}
