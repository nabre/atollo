use super::*;
use crate::kit::{Os, Target};
use crate::router::bins::db_bin_in;
use std::path::Path;

/// The ids of the servers (`SERVERS`, D-73), in the order of the page, so
/// saved files and the interface are unchanged.
#[test]
fn all_covers_the_servers() {
    let ids: Vec<&str> = DbEngine::ALL.iter().map(|e| e.id()).collect();
    assert_eq!(ids, ["mysql", "mongodb"]);
    assert_eq!(crate::db::services::SERVERS, ["mysql", "mongodb"]);
    for e in DbEngine::ALL {
        assert_eq!(DbEngine::from_id(e.id()), Some(e));
        assert_eq!(DbEngine::parse(e.id()).unwrap(), e);
        assert_eq!(e.to_string(), e.id());
        assert_eq!(e.component(), e.id(), "the catalog id is the saved id");
    }
    for other in ["", "sqlite", "redis", "MySQL", "apache", "mongosh"] {
        assert_eq!(DbEngine::from_id(other), None, "{other}");
    }
    assert_eq!(
        DbEngine::parse("redis").unwrap_err().to_string(),
        "unknown server redis"
    );
}

#[test]
fn serde_keeps_the_saved_ids() {
    for e in DbEngine::ALL {
        let json = serde_json::to_string(&e).unwrap();
        assert_eq!(json, format!("\"{}\"", e.id()));
        assert_eq!(serde_json::from_str::<DbEngine>(&json).unwrap(), e);
    }
    assert!(serde_json::from_str::<DbEngine>("\"MySQL\"").is_err());
}

#[test]
fn mongodb_programs_and_port() {
    let g = DbEngine::MongoDb;
    assert_eq!(g.default_port(), 27017);
    assert_eq!(g.server_bin(), "mongod");
    assert_eq!(
        (g.client_component(), g.client_bin()),
        ("mongosh", "mongosh")
    );
    assert_eq!(
        (g.tools_component(), g.dump_bin(), g.restore_bin()),
        ("mongodb-tools", "mongodump", "mongorestore")
    );
    assert_eq!(g.admin_bin(), None);
    assert_eq!(g.admin_user(), "admin");
    assert!(!g.is_sql());
}

/// The programs of every engine on the 6 targets: `.exe` on Windows only,
/// in `<folder>/bin`.
#[test]
fn programs_on_every_target() {
    let dir = Path::new("apps").join("x");
    for t in Target::ALL {
        for e in DbEngine::ALL {
            let stems = [
                Some(e.server_bin()),
                Some(e.client_bin()),
                Some(e.dump_bin()),
                Some(e.restore_bin()),
                e.admin_bin(),
            ];
            for stem in stems.into_iter().flatten() {
                let path = db_bin_in(t.os, &dir, stem);
                let want = match t.os {
                    Os::Win32 => format!("{stem}.exe"),
                    Os::Darwin | Os::Linux => stem.to_owned(),
                };
                assert_eq!(path, dir.join("bin").join(&want), "{} {e}", t.as_str());
            }
        }
    }
}

/// The lists of `auth::is_reserved`.
#[test]
fn reserved_names_per_engine() {
    assert_eq!(
        DbEngine::MongoDb.reserved_names(),
        ["admin", "local", "config"]
    );
    assert_eq!(
        DbEngine::MySql.system_databases(),
        ["information_schema", "mysql", "performance_schema", "sys"]
    );
    assert_eq!(
        DbEngine::MongoDb.system_databases(),
        ["admin", "config", "local"]
    );
}

#[test]
fn urls_and_copies() {
    assert_eq!(
        DbEngine::MongoDb.admin_url(27017),
        "mongodb://admin@127.0.0.1:27017/?authSource=admin"
    );
    assert_eq!(
        DbEngine::MongoDb.project_url("shop", "pw", 27017),
        "mongodb://shop:pw@127.0.0.1:27017/shop?authSource=shop"
    );
    assert_eq!(DbEngine::of_backup("shop-1.sql"), Some(DbEngine::MySql));
    assert_eq!(
        DbEngine::of_backup("shop-1.archive.gz"),
        Some(DbEngine::MongoDb)
    );
    assert_eq!(DbEngine::of_backup("shop-1.gz"), None);
    assert_eq!(DbEngine::of_backup("notes.txt"), None);
    assert_eq!(
        DbEngine::MySql.admin_url(3306),
        "mysql://root@127.0.0.1:3306/"
    );
    assert_eq!(
        DbEngine::MySql.project_url("shop", "pw", 3307),
        "mysql://shop:pw@127.0.0.1:3307/shop"
    );
}

#[test]
fn mysql_programs() {
    let m = DbEngine::MySql;
    assert_eq!((m.id(), m.component()), ("mysql", "mysql"));
    assert_eq!(m.default_port(), 3306);
    assert_eq!(m.server_bin(), "mysqld");
    assert_eq!((m.client_component(), m.client_bin()), ("mysql", "mysql"));
    assert_eq!((m.tools_component(), m.dump_bin()), ("mysql", "mysqldump"));
    assert_eq!(m.restore_bin(), "mysql");
    assert_eq!(m.admin_bin(), Some("mysqladmin"));
    assert_eq!(m.admin_user(), "root");
    assert_eq!(m.backup_ext(), ".sql");
    assert!(m.is_sql());
    assert_eq!(
        m.reserved_names(),
        [
            "mysql",
            "information_schema",
            "performance_schema",
            "sys",
            "root"
        ]
    );
}
