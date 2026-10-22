use super::*;
use crate::test_support::prepared_kit;

/// Every owner read from its file: the pools, the stored ports of the
/// database versions, the projects' actions, the editor's range; a port
/// two of them have goes to the first, but [`refuse_taken`] sees both.
#[test]
fn every_owner_from_its_file() {
    let (_tmp, kit) = prepared_kit("ports-taken");
    assert_eq!(ports_taken(&kit).len(), crate::editor::PORTS.count());
    std::fs::write(
        kit.app_dir().join("servers.json"),
        r#"{ "ports": { "legacy": 8095, "twin": 3307 } }"#,
    )
    .unwrap();
    std::fs::write(
        kit.app_dir().join("services.json"),
        r#"{ "mysql": { "ports": { "8.4": 3307 } }, "mongodb": { "ports": { "8.0": 27018 } } }"#,
    )
    .unwrap();
    let shop = kit.project_data().join("shop");
    std::fs::create_dir_all(&shop).unwrap();
    std::fs::write(shop.join("ports.json"), r#"{ "ports": { "site": 7150 } }"#).unwrap();

    let taken = ports_taken(&kit);
    assert_eq!(taken[&8095], Owner::Pool("legacy".into()));
    assert_eq!(taken[&3307], Owner::Pool("twin".into()));
    assert_eq!(
        taken[&27018],
        Owner::Db {
            engine: DbEngine::MongoDb,
            line: "8.0".into()
        }
    );
    assert_eq!(
        taken[&7150],
        Owner::Project {
            project: "shop".into(),
            action: "site".into()
        }
    );
    assert_eq!(taken[&7000], Owner::Editor);
    assert!(!taken.contains_key(&8096));

    // The pool saving its own port still meets the database version there.
    let err = refuse_taken(&kit, 3307, |o| *o == Owner::Pool("twin".into()), "it").unwrap_err();
    assert_eq!(
        err.to_string(),
        "La porta 3307 è già del database MySQL 8.4: scegline un'altra."
    );
    refuse_taken(&kit, 8095, |o| *o == Owner::Pool("legacy".into()), "it").unwrap();
    refuse_taken(&kit, 8096, |_| false, "it").unwrap();
}

/// The port of the local models is kept for them (`llm.json`).
#[test]
fn the_local_models_keep_their_port() {
    let (_tmp, kit) = prepared_kit("ports-llm");
    std::fs::write(kit.app_dir().join("llm.json"), r#"{ "port": 9931 }"#).unwrap();
    assert_eq!(ports_taken(&kit)[&9931], Owner::Llm);
    let err = refuse_taken(&kit, 9931, |_| false, "it").unwrap_err();
    assert_eq!(
        err.to_string(),
        "La porta 9931 è dei modelli locali: scegline un'altra."
    );
    refuse_taken(&kit, 9931, |o| *o == Owner::Llm, "it").unwrap();
}
