use super::*;
use crate::test_support::prepared_kit;

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
            assert!(p.category.is_some(), "{}: category", p.id);
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
    let (_tmp, kit) = prepared_kit("frameworks");
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
    let (_tmp, kit) = prepared_kit("frameworks-bad");
    fs::write(user_file(&kit), "{ \"pieces\": ").unwrap();
    let loaded = load(&kit);
    assert!(loaded.problem.is_some());
    assert_eq!(loaded.pieces, official());
}

/// A creation's line asks no question and has no time limit on Composer's
/// scripts: Laravel's starter kits run `npm install` and a build inside
/// `artisan install:features`, over Composer's 300 s on a slow disk.
#[test]
fn a_creation_line_has_no_composer_timeout() {
    let kit = crate::kit::Kit::at_target("kit".into(), crate::kit::Target::WIN32_X64);
    let cmd = line_command(
        &kit,
        &crate::env::KitEnv::default(),
        &[],
        std::path::Path::new("."),
        "composer create-project laravel/react-starter-kit app",
    );
    let env: Vec<(String, String)> = cmd
        .get_envs()
        .filter_map(|(k, v)| Some((k.to_str()?.to_owned(), v?.to_str()?.to_owned())))
        .collect();
    for (k, v) in [
        ("COMPOSER_PROCESS_TIMEOUT", "0"),
        ("COMPOSER_NO_INTERACTION", "1"),
        ("CI", "1"),
    ] {
        assert!(env.contains(&(k.to_owned(), v.to_owned())), "{k}: {env:?}");
    }
}
