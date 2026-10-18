use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn official_catalog_is_valid() {
    let pieces = official();
    assert!(pieces.len() >= 15, "catalog did not load");
    validate(pieces).unwrap();
    for p in pieces {
        let mut c = p.clone();
        clean(&mut c);
        assert_eq!(&c, p, "{}: not clean", p.id);
        assert!(
            !p.desc.is_empty() && p.home.starts_with("https://"),
            "{}",
            p.id
        );
        assert_eq!(p.kind == Kind::Addon, p.slot.is_some(), "{}: slot", p.id);
        if p.kind == Kind::Base {
            assert!(p.on.is_empty(), "{}", p.id);
            assert!(p.create.iter().all(|l| l.contains("{name}")), "{}", p.id);
        }
        for v in p.on.values() {
            assert!(v.create.iter().all(|l| l.contains("{name}")), "{}", p.id);
        }
    }
}

/// A framework file of before may name `mariadb` as its database: MySQL,
/// the kit's only SQL server, for the piece and for an addition.
#[test]
fn a_database_of_before_is_mysql() {
    let piece: Piece = serde_json::from_str(
        r#"{ "id": "mine", "kind": "addon", "name": "Mine", "database": "mariadb",
             "on": { "vite": { "database": "mariadb" }, "laravel": { "database": "mongodb" } } }"#,
    )
    .unwrap();
    assert_eq!(piece.database.as_deref(), Some("mysql"));
    assert_eq!(piece.on["vite"].database.as_deref(), Some("mysql"));
    assert_eq!(piece.on["laravel"].database.as_deref(), Some("mongodb"));
    assert_eq!(
        piece.on["vite"].needs,
        Vec::<String>::new(),
        "defaults kept"
    );
    let none: Piece =
        serde_json::from_str(r#"{ "id": "x", "kind": "base", "name": "X" }"#).unwrap();
    assert_eq!(none.database, None);
}

#[test]
fn saves_only_the_changes() {
    let tmp = TempDir::new("frameworks");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    assert_eq!(load(&kit).pieces, official());

    let mut all = official().to_vec();
    all[0].steps.push("  npm run build ".into());
    all[0].steps.push(String::new());
    let mut mine = all[2].clone();
    mine.id = "mine".into();
    all.push(mine);
    save(&kit, all).unwrap();
    let saved: File = serde_json::from_str(&fs::read_to_string(user_file(&kit)).unwrap()).unwrap();
    assert_eq!(
        saved
            .pieces
            .iter()
            .map(|p| p.id.as_str())
            .collect::<Vec<_>>(),
        ["laravel", "mine"]
    );
    let loaded = load(&kit).pieces;
    assert_eq!(loaded[0].steps, ["npm install", "npm run build"]);
    assert_eq!(loaded.len(), official().len() + 1);

    // Back to the official instructions: the file goes away.
    save(&kit, official().to_vec()).unwrap();
    assert!(!user_file(&kit).exists());
    assert!(save(
        &kit,
        vec![Piece {
            name: String::new(),
            ..official()[0].clone()
        }]
    )
    .is_err());
}

#[test]
fn broken_file_is_reported_not_replaced() {
    let tmp = TempDir::new("frameworks-bad");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    fs::write(user_file(&kit), "{ \"pieces\": ").unwrap();
    let loaded = load(&kit);
    assert!(loaded.problem.is_some());
    assert_eq!(loaded.pieces, official());
}
