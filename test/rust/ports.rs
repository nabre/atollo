use super::*;
use crate::kit::{tests::TempDir, Arch};

/// Every owner read from its file: the pools, the stored ports of the
/// database versions, the projects' actions, the editor's range; a port
/// two of them have goes to the first, but [`refuse_taken`] sees both.
#[test]
fn every_owner_from_its_file() {
    let tmp = TempDir::new("ports-taken");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
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
