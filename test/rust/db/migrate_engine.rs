//! The move of MariaDB's data to MySQL (D-117) against fake servers: the
//! databases are maps, a dump is a text the fake restore reads back.

use super::*;
use crate::db::services::{load_settings, Setting};
use crate::kit::{tests::TempDir, Arch};
use crate::setup::catalog::Catalog;
use std::cell::RefCell;

type Tables = BTreeMap<String, Vec<(String, u64)>>;

#[derive(Default)]
struct Fake {
    kit: Option<Kit>,
    mariadb: RefCell<BTreeMap<String, Tables>>,
    mysql: RefCell<BTreeMap<String, Tables>>,
    mariadb_lines: Vec<String>,
    mysql_lines: RefCell<Vec<String>>,
    running: RefCell<BTreeSet<(DbEngine, String)>>,
    calls: RefCell<Vec<String>>,
    /// A step that fails: `restore 8.4 blog as blog`, `start mysql 8.4`…
    fail: RefCell<Option<String>>,
    /// Rows the fake restore loses (a check that must fail).
    lose_rows: RefCell<bool>,
    /// MariaDB's global sql_mode.
    sql_mode: RefCell<String>,
}

impl Fake {
    fn call(&self, what: String) -> Result<()> {
        let failing = self.fail.borrow().as_deref() == Some(what.as_str());
        self.calls.borrow_mut().push(what.clone());
        if failing {
            bail!("{what}: failed (fake)");
        }
        Ok(())
    }

    fn store(&self, engine: DbEngine) -> &RefCell<BTreeMap<String, Tables>> {
        match engine {
            DbEngine::MySql => &self.mysql,
            _ => &self.mariadb,
        }
    }

    fn called(&self, what: &str) -> bool {
        self.calls.borrow().iter().any(|c| c == what)
    }
}

impl Servers for Fake {
    fn kit(&self) -> &Kit {
        self.kit.as_ref().unwrap()
    }

    fn installed_lines(&self, engine: DbEngine) -> Vec<String> {
        match engine {
            DbEngine::MySql => self.mysql_lines.borrow().clone(),
            _ => self.mariadb_lines.clone(),
        }
    }

    fn start(&self, engine: DbEngine, line: &str) -> Result<bool> {
        self.call(format!("start {engine} {line}"))?;
        Ok(!self.running.borrow_mut().insert((engine, line.to_owned())))
    }

    fn stop(&self, engine: DbEngine, line: &str) {
        let _ = self.call(format!("stop {engine} {line}"));
        self.running.borrow_mut().remove(&(engine, line.to_owned()));
    }

    fn running(&self, engine: DbEngine, line: &str) -> bool {
        self.running.borrow().contains(&(engine, line.to_owned()))
    }

    fn sql(&self, engine: DbEngine, line: &str, sql: &str) -> Result<()> {
        if let Some(rest) = sql.strip_prefix("DROP DATABASE IF EXISTS `") {
            let name = rest.split('`').next().unwrap_or_default();
            self.call(format!("drop {engine} {line} {name}"))?;
            if let Some(dbs) = self.store(engine).borrow_mut().get_mut(line) {
                dbs.remove(name);
            }
            return Ok(());
        }
        let created = sql
            .strip_prefix("CREATE DATABASE IF NOT EXISTS `")
            .map(|rest| ("user", rest))
            .or_else(|| {
                sql.strip_prefix("CREATE DATABASE `")
                    .map(|rest| ("database", rest))
            });
        if let Some((what, rest)) = created {
            let name = rest.split('`').next().unwrap_or_default();
            self.call(format!("{what} {engine} {line} {name}"))?;
            self.store(engine)
                .borrow_mut()
                .entry(line.to_owned())
                .or_default()
                .entry(name.to_owned())
                .or_default();
            return Ok(());
        }
        if sql.starts_with("CREATE USER 'dkmove_") {
            return self.call(format!("account {engine} {line}"));
        }
        if sql.starts_with("DROP USER IF EXISTS 'dkmove_") {
            return self.call(format!("drop account {engine} {line}"));
        }
        self.call(format!("sql {engine} {line} {}", sql.trim()))
    }

    fn query(&self, engine: DbEngine, line: &str, sql: &str) -> Result<String> {
        let store = self.store(engine).borrow();
        let dbs = store.get(line).cloned().unwrap_or_default();
        if sql.starts_with("SHOW DATABASES") {
            let mut names: Vec<String> = ["information_schema", "mysql"]
                .iter()
                .map(|s| (*s).to_owned())
                .collect();
            names.extend(dbs.keys().cloned());
            return Ok(names.join("\n") + "\n");
        }
        if sql.starts_with("SELECT @@GLOBAL.sql_mode") {
            return Ok(format!("{}\tSTRICT_TRANS_TABLES\n", self.sql_mode.borrow()));
        }
        if sql.starts_with("SELECT default_character_set_name") {
            return Ok("utf8mb4\tutf8mb4_uca1400_ai_ci\n".to_owned());
        }
        if sql.starts_with("SELECT 'table'") {
            let mut out = String::new();
            for (d, tables) in &dbs {
                for (t, _) in tables {
                    out.push_str(&format!(
                        "table\t{}\t{}\ttable\n",
                        d.to_lowercase(),
                        t.to_lowercase()
                    ));
                }
                for (t, _) in tables {
                    out.push_str(&format!("{d}\t{t}\n"));
                }
            }
            return Ok(out);
        }
        // Row counts, one per `SELECT COUNT(*) FROM `d`.`t`;` line.
        let mut out = String::new();
        for l in sql.lines() {
            let parts: Vec<&str> = l.split('`').collect();
            let (d, t) = (parts[1], parts[3]);
            let n = dbs
                .get(d)
                .and_then(|ts| ts.iter().find(|(x, _)| x == t))
                .map_or(0, |(_, n)| *n);
            out.push_str(&format!("{n}\n"));
        }
        Ok(out)
    }

    fn dump(&self, line: &str, database: &str, to: &Path) -> Result<()> {
        self.call(format!("dump {line} {database}"))?;
        let store = self.mariadb.borrow();
        let mut text = String::from("/*M!999999\\- enable the sandbox mode */\n");
        for (t, n) in &store[line][database] {
            if t == "seq" {
                text.push_str("CREATE SEQUENCE `seq` start with 1;\n");
            }
            text.push_str(&format!("-- table {database} {t} {n}\n"));
        }
        fs::write(to, text)?;
        Ok(())
    }

    fn restore(
        &self,
        line: &str,
        database: &str,
        user: &str,
        password: &str,
        from: &Path,
    ) -> Result<()> {
        assert_ne!(user, "root", "never the administrator");
        assert!(!password.is_empty());
        let who = if user.starts_with("dkmove_") {
            "dkmove"
        } else {
            user
        };
        self.call(format!("restore {line} {database} as {who}"))?;
        let text = fs::read_to_string(from)?;
        assert!(!text.contains("/*M!"), "the filtered dump");
        let mut store = self.mysql.borrow_mut();
        let dbs = store.entry(line.to_owned()).or_default();
        for l in text.lines() {
            if let Some(rest) = l.strip_prefix("-- table ") {
                let p: Vec<&str> = rest.split(' ').collect();
                assert_eq!(p[0], database, "only its own database");
                let n: u64 = p[2].parse().unwrap();
                let n = if *self.lose_rows.borrow() { n / 2 } else { n };
                dbs.entry(p[0].to_owned())
                    .or_default()
                    .push((p[1].to_owned(), n));
            }
        }
        Ok(())
    }

    fn serve(&self, line: &str) -> Result<()> {
        self.call(format!("serve {line}"))
    }
}

/// A kit with MariaDB 12.3 (the server, with `shop`) and 11.4 (port 3307,
/// with `blog`), their programs on disk, two projects, MySQL 9.7 and 8.4
/// installable.
fn kit_with_mariadb(name: &str) -> (TempDir, Fake) {
    let tmp = TempDir::new(name);
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let c = Catalog::get().find_internal("mariadb").unwrap();
    for (version, line) in [("12.3.3", "12.3"), ("11.4.9", "11.4")] {
        let dir = if line == "12.3" {
            crate::versions::install_dir(&kit, c, version)
        } else {
            crate::versions::saved_dir(&kit, c, line)
        };
        fs::create_dir_all(dir.join("bin")).unwrap();
        for stem in ["mariadbd", "mariadb"] {
            let exe = crate::router::bins::exe_name(kit.os(), stem);
            fs::write(dir.join("bin").join(exe), b"").unwrap();
        }
        fs::create_dir_all(
            kit.data()
                .join("db")
                .join("mariadb")
                .join(line)
                .join("mysql"),
        )
        .unwrap();
    }
    c.save_version(&kit, "12.3.3").unwrap();
    let mut settings = BTreeMap::new();
    let mut maria = Setting {
        autostart: true,
        ..Setting::default()
    };
    maria.ports.insert("11.4".into(), 3307);
    settings.insert("mariadb".to_owned(), maria);
    crate::fsx::write_json(&kit.app_dir().join("services.json"), &settings).unwrap();
    let mut secrets = auth::load(&kit);
    secrets.mariadb_root = Some("rootpw".into());
    auth::save(&kit, &secrets).unwrap();
    for (p, db) in [("shop", "shop"), ("blog", "blog")] {
        fs::create_dir_all(kit.projects().join(p)).unwrap();
        fs::create_dir_all(kit.project_data().join(p)).unwrap();
        auth::ensure_project_db(&kit, p, "mariadb").unwrap();
        assert_eq!(auth::project_db_name(&kit, p, "mariadb").unwrap(), db);
    }
    auth::set_project_line(&kit, "blog", "mariadb", Some("11.4")).unwrap();
    auth::add_project_used_line(&kit, "blog", "mariadb", "11.4").unwrap();
    let fake = Fake {
        kit: Some(kit),
        mariadb_lines: vec!["12.3".into(), "11.4".into()],
        ..Fake::default()
    };
    let shop: Tables = [(
        "shop".to_owned(),
        vec![("orders".to_owned(), 12), ("Users".to_owned(), 3)],
    )]
    .into();
    let blog: Tables = [("blog".to_owned(), vec![("wp_posts".to_owned(), 40)])].into();
    fake.mariadb.borrow_mut().insert("12.3".into(), shop);
    fake.mariadb.borrow_mut().insert("11.4".into(), blog);
    (tmp, fake)
}

fn installs(fake: &Fake) -> impl Fn(&[(String, String)]) -> Result<()> + '_ {
    |wanted: &[(String, String)]| {
        for (id, line) in wanted {
            assert_eq!(id, "mysql");
            fake.mysql_lines.borrow_mut().push(line.clone());
        }
        Ok(())
    }
}

#[test]
fn versions_go_newest_first_the_servers_to_mysqls() {
    let lines = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let offered = lines(&["9.7", "8.4"]);
    assert_eq!(
        plan_lines(&lines(&["12.3", "11.4"]), Some("12.3"), "9.7", &offered),
        [("12.3".into(), "9.7".into()), ("11.4".into(), "8.4".into())]
    );
    // The server's version first, whatever its age; the oldest share the
    // last MySQL version.
    assert_eq!(
        plan_lines(
            &lines(&["12.3", "11.4", "10.6"]),
            Some("11.4"),
            "8.4",
            &offered
        ),
        [
            ("11.4".into(), "8.4".into()),
            ("12.3".into(), "9.7".into()),
            ("10.6".into(), "9.7".into())
        ]
    );
    assert_eq!(mysql_offered(), ["9.7", "8.4"], "the pinned lines");
}

#[test]
fn every_version_moves_and_the_configuration_last() {
    let (_tmp, fake) = kit_with_mariadb("move-all");
    let kit = fake.kit().clone();
    let before_services = fs::read_to_string(kit.app_dir().join("services.json")).unwrap();
    let before_blog = fs::read_to_string(db_json(&kit, "blog")).unwrap();
    assert_eq!(status(&kit).state, "pending");
    assert_eq!(crate::db::services::sql_engine(&kit), DbEngine::MariaDb);

    assert_eq!(run(&fake, &installs(&fake)).unwrap(), Outcome::Done);

    // The data, version by version, with the users of the projects.
    let mysql = fake.mysql.borrow();
    assert_eq!(
        mysql["9.7"]["shop"],
        [("orders".to_owned(), 12), ("Users".to_owned(), 3)]
    );
    assert_eq!(mysql["8.4"]["blog"], [("wp_posts".to_owned(), 40)]);
    assert!(fake.called("user mysql 9.7 shop") && fake.called("user mysql 8.4 blog"));
    assert!(fake.called("sql mariadb 12.3 SET GLOBAL read_only = ON;"));
    assert!(fake.called("serve 9.7"), "MariaDB was started with the app");
    assert!(!fake.called("serve 8.4"));
    let order: Vec<String> = fake
        .calls
        .borrow()
        .iter()
        .filter(|c| c.starts_with("dump"))
        .cloned()
        .collect();
    assert_eq!(
        order,
        ["dump 12.3 shop", "dump 11.4 blog"],
        "the server's version first"
    );
    // Each database restored by its project's user, never root.
    assert!(fake.called("restore 9.7 shop as shop") && fake.called("restore 8.4 blog as blog"));
    assert!(fake.called("database mysql 9.7 shop"), "created first");

    // db.json, services.json, secrets.json rewritten, copies kept.
    let shop: serde_json::Value = crate::fsx::read_json(&db_json(&kit, "shop")).unwrap();
    assert!(shop.get("mariadb").is_none() && shop["mysql"].is_string());
    assert_eq!(shop["names"]["mysql"], "shop");
    assert!(shop.get("lines").is_none(), "on the server's version");
    let blog: serde_json::Value = crate::fsx::read_json(&db_json(&kit, "blog")).unwrap();
    assert_eq!(blog["lines"]["mysql"], "8.4");
    assert_eq!(blog["used"]["mysql"], serde_json::json!(["8.4"]));
    assert_eq!(
        fs::read_to_string(copy_of(&db_json(&kit, "blog"))).unwrap(),
        before_blog
    );
    let services = load_settings(&kit);
    assert!(!services.contains_key("mariadb"));
    let m = &services["mysql"];
    assert_eq!(
        (m.port, m.line.as_deref(), m.autostart),
        (None, Some("9.7"), true)
    );
    assert_eq!(m.ports.get("8.4"), Some(&3307), "MariaDB 11.4's port");
    assert_eq!(
        fs::read_to_string(copy_of(&kit.app_dir().join("services.json"))).unwrap(),
        before_services
    );
    let secrets = auth::load(&kit);
    assert_eq!(
        (secrets.mysql_root.as_deref(), secrets.mariadb_root),
        (Some("rootpw"), None)
    );

    // MariaDB's data kept, with the dumps; nothing left to move.
    let kept = kept_dir(&kit);
    assert!(
        kept.join("12.3").join("mysql").is_dir()
            && kept.join("dumps").join("11.4.0.mysql.sql").is_file()
    );
    assert!(!kit.data().join("db").join("mariadb").exists());
    let st = status(&kit);
    assert_eq!(st.state, "done");
    assert!(st.kept.is_some() && st.error.is_none());
    assert_eq!(crate::db::services::sql_engine(&kit), DbEngine::MySql);
    // A second run has nothing to do.
    assert_eq!(run(&fake, &installs(&fake)).unwrap(), Outcome::Nothing);

    // The user deletes the data kept: copies go too.
    drop_kept(&kit).unwrap();
    assert!(!kept.exists() && !copy_of(&db_json(&kit, "blog")).exists());
    assert_eq!(status(&kit).state, "none");
}

/// A step that fails leaves MariaDB as it was: running, writable, the
/// configuration untouched; the next run drops what the failed one restored
/// and moves everything.
#[test]
fn a_failed_step_leaves_mariadb_active_and_a_new_run_resumes() {
    let (_tmp, fake) = kit_with_mariadb("move-fail");
    let kit = fake.kit().clone();
    let services = fs::read_to_string(kit.app_dir().join("services.json")).unwrap();
    let blog = fs::read_to_string(db_json(&kit, "blog")).unwrap();
    *fake.fail.borrow_mut() = Some("restore 8.4 blog as blog".into());
    let err = run(&fake, &installs(&fake)).unwrap_err();
    assert!(format!("{err:#}").contains("MariaDB 11.4"), "{err:#}");
    assert!(fake.called("sql mariadb 12.3 SET GLOBAL read_only = OFF;"));
    assert!(fake.called("sql mariadb 11.4 SET GLOBAL read_only = OFF;"));
    assert!(
        fake.called("stop mariadb 12.3"),
        "started by the move: stopped"
    );
    assert_eq!(
        fs::read_to_string(kit.app_dir().join("services.json")).unwrap(),
        services
    );
    assert_eq!(fs::read_to_string(db_json(&kit, "blog")).unwrap(), blog);
    assert!(!copy_of(&db_json(&kit, "blog")).exists());
    assert!(kit
        .data()
        .join("db")
        .join("mariadb")
        .join("12.3")
        .join("mysql")
        .is_dir());
    assert_eq!(crate::db::services::sql_engine(&kit), DbEngine::MariaDb);
    let st = status(&kit);
    assert_eq!(st.state, "failed");
    assert!(
        st.dump
            .as_deref()
            .is_some_and(|d| d.ends_with("11.4.0.mysql.sql")),
        "{st:?}"
    );
    assert_eq!(load(&kit).lines["11.4"].restored, ["blog"]);

    // Again: what the failed run restored goes first, then all moves.
    *fake.fail.borrow_mut() = None;
    fake.calls.borrow_mut().clear();
    assert_eq!(run(&fake, &installs(&fake)).unwrap(), Outcome::Done);
    assert!(fake.called("drop mysql 9.7 shop") && fake.called("drop mysql 8.4 blog"));
    assert!(
        fake.called("dump 12.3 shop"),
        "dumped again: MariaDB may have changed"
    );
    assert_eq!(
        fake.mysql.borrow()["8.4"]["blog"],
        [("wp_posts".to_owned(), 40)]
    );
}

/// The check compares the rows: a restore that lost some stops the move.
#[test]
fn rows_that_differ_stop_the_move() {
    let (_tmp, fake) = kit_with_mariadb("move-rows");
    *fake.lose_rows.borrow_mut() = true;
    let err = run(&fake, &installs(&fake)).unwrap_err();
    assert!(format!("{err:#}").contains("rows"), "{err:#}");
    assert_eq!(status(fake.kit()).state, "failed");
}

/// A database MySQL has already is never overwritten.
#[test]
fn a_database_mysql_has_already_is_a_conflict() {
    let (_tmp, fake) = kit_with_mariadb("move-conflict");
    let theirs: Tables = [("shop".to_owned(), vec![("mine".to_owned(), 1)])].into();
    fake.mysql.borrow_mut().insert("9.7".into(), theirs);
    let err = run(&fake, &installs(&fake)).unwrap_err();
    assert!(
        format!("{err:#}").contains("has a database shop already"),
        "{err:#}"
    );
    assert_eq!(fake.mysql.borrow()["9.7"]["shop"], [("mine".to_owned(), 1)]);
    assert!(!fake.called("drop mysql 9.7 shop"));
}

#[test]
fn sequences_stop_the_move_with_their_names() {
    let (_tmp, fake) = kit_with_mariadb("move-seq");
    fake.mariadb
        .borrow_mut()
        .get_mut("12.3")
        .unwrap()
        .get_mut("shop")
        .unwrap()
        .push(("seq".into(), 1));
    let err = run(&fake, &installs(&fake)).unwrap_err();
    assert!(format!("{err:#}").contains("shop.seq"), "{err:#}");
    assert!(fake
        .calls
        .borrow()
        .iter()
        .all(|c| !c.starts_with("restore")));
    assert!(
        load(fake.kit())
            .dump
            .is_some_and(|d| d.ends_with("12.3.0.sql")),
        "the dump kept"
    );
}

/// Without MariaDB's programs on this system (a kit made on Windows on a
/// Mac) the data waits; nothing is downloaded.
#[test]
fn without_mariadbs_programs_the_data_waits() {
    let (_tmp, mut fake) = kit_with_mariadb("move-blocked");
    fake.mariadb_lines = vec!["12.3".into()];
    let none = |_: &[(String, String)]| -> Result<()> { panic!("nothing installed") };
    let err = run(&fake, &none).unwrap_err();
    assert!(
        format!("{err:#}").contains("MariaDB 11.4: its programs"),
        "{err:#}"
    );
    assert!(fake.calls.borrow().is_empty());
}

/// An interrupted switch ends at the next start from where it stopped:
/// MariaDB does not start again meanwhile, the copies stay the originals.
#[test]
fn an_interrupted_switch_ends_at_the_next_run() {
    let (_tmp, fake) = kit_with_mariadb("move-switch");
    let kit = fake.kit().clone();
    let original = fs::read_to_string(db_json(&kit, "shop")).unwrap();
    // The state a switch leaves when the app dies after the copies and
    // services.json, before the projects.
    let mut state = State {
        phase: Phase::Switching,
        main: Some("9.7".into()),
        copied: true,
        projects: vec!["blog".into(), "shop".into()],
        ..State::default()
    };
    for (l, m, port) in [("12.3", "9.7", 3306), ("11.4", "8.4", 3307)] {
        state.lines.insert(
            l.into(),
            LineState {
                mysql: m.into(),
                port,
                ..LineState::default()
            },
        );
    }
    for f in rewritten(&kit, &state.projects) {
        fs::copy(&f, copy_of(&f)).unwrap();
    }
    rewrite_services(&kit, &state, false).unwrap();
    save(&kit, &state).unwrap();
    assert!(switching(&kit));
    let s = crate::db::services::Services::new(kit.clone());
    let err = s
        .start("mariadb", &crate::env::KitEnv::default())
        .unwrap_err();
    assert!(format!("{err:#}").contains("moving to MySQL"), "{err:#}");

    assert_eq!(run(&fake, &installs(&fake)).unwrap(), Outcome::Done);
    assert!(
        fake.calls.borrow().iter().all(|c| !c.starts_with("dump")),
        "no copy again"
    );
    let shop: serde_json::Value = crate::fsx::read_json(&db_json(&kit, "shop")).unwrap();
    assert!(shop["mysql"].is_string());
    assert_eq!(
        fs::read_to_string(copy_of(&db_json(&kit, "shop"))).unwrap(),
        original
    );
    assert_eq!(load_settings(&kit)["mysql"].ports.get("8.4"), Some(&3307));
    assert!(kept_dir(&kit).is_dir() && !switching(&kit));
}

/// D-117: MariaDB's programs go once no data waits for them; data that
/// waits keeps them.
#[test]
fn mariadbs_programs_go_once_nothing_waits() {
    let (_tmp, fake) = kit_with_mariadb("move-retire");
    let kit = fake.kit().clone();
    let apps = kit.apps().join("mariadb");
    crate::versions::disk::retire_mariadb(&kit);
    assert!(apps.is_dir(), "data waits: kept");
    run(&fake, &installs(&fake)).unwrap();
    crate::versions::disk::retire_mariadb(&kit);
    assert!(!apps.exists(), "moved: removed");
    assert!(
        kept_dir(&kit).join("11.4").join("mysql").is_dir(),
        "the data stays"
    );
}

/// Security review of D-117: a database no project owns is restored by an
/// account made for it alone, dropped after; a MariaDB that runs with
/// NO_BACKSLASH_ESCAPES is not dumped at all.
#[test]
fn every_database_is_restored_by_an_account_of_its_own() {
    let (_tmp, fake) = kit_with_mariadb("move-accounts");
    let mine: Vec<(String, u64)> = vec![("notes".to_owned(), 2)];
    fake.mariadb
        .borrow_mut()
        .get_mut("12.3")
        .unwrap()
        .insert("by_hand".to_owned(), mine.clone());
    run(&fake, &installs(&fake)).unwrap();
    assert!(fake.called("account mysql 9.7"));
    assert!(fake.called("restore 9.7 by_hand as dkmove"));
    assert!(fake.called("drop account mysql 9.7"), "the account goes");
    assert_eq!(fake.mysql.borrow()["9.7"]["by_hand"], mine);
    assert!(
        fake.calls.borrow().iter().all(|c| !c.ends_with("as root")),
        "never the administrator"
    );
    assert!(restore_account_sql("dkmove_1", "pw", "a_b%c").contains("ON `a\\_b\\%c`.*"));
    assert_eq!(
        charset_of("utf8mb3\tutf8mb3_general_ci\n"),
        " CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci"
    );
    assert_eq!(charset_of("x;DROP\ty"), "", "only plain names");
}

#[test]
fn a_mariadb_with_no_backslash_escapes_is_not_dumped() {
    let (_tmp, fake) = kit_with_mariadb("move-nbe");
    *fake.sql_mode.borrow_mut() = "STRICT_TRANS_TABLES,NO_BACKSLASH_ESCAPES".into();
    let err = run(&fake, &installs(&fake)).unwrap_err();
    assert!(
        format!("{err:#}").contains("NO_BACKSLASH_ESCAPES"),
        "{err:#}"
    );
    assert!(fake.calls.borrow().iter().all(|c| !c.starts_with("dump")));
    assert!(fake.called("sql mariadb 12.3 SET GLOBAL read_only = OFF;"));
}

#[test]
fn names_and_strings_of_the_checks() {
    assert_eq!(sql_str("it's \\ ok"), "'it''s \\\\ ok'");
    assert_eq!(sql_name("a`b"), "`a``b`");
    assert_eq!(unescape("a\\tb\\\\c\\n"), "a\tb\\c\n");
    assert_eq!(names_of("shop\t1\n\nblog\n"), ["shop", "blog"]);
}

/// A dump holds every row of its database: its folder and its files are the
/// owner's only (security review of D-117).
#[cfg(unix)]
#[test]
fn dumps_are_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = crate::kit::tests::TempDir::new("dumps-private");
    let dir = tmp.0.join("mariadb").join("dumps");
    private_dumps(&dir).unwrap();
    let file = dir.join("12.3.0.sql");
    write_through(&file, |to| Ok(fs::write(to, "INSERT 1;")?)).unwrap();
    let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&dir), 0o700);
    assert_eq!(mode(&file), 0o600);
    assert!(!file.with_extension("partial").exists());
}
