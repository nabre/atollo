use super::*;

/// The detection says `mariadb` for any MySQL-compatible project: the
/// project's SQL server is shown instead, once; others are kept, unknown
/// ids left out.
#[test]
fn detected_servers_as_the_projects() {
    let found = |ids: &[&str]| ids.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    assert_eq!(
        detected_as(found(&["mariadb", "sqlite", "redis"]), DbEngine::MySql),
        ["mysql", "sqlite"]
    );
    assert_eq!(
        detected_as(found(&["mariadb", "mysql", "mongodb"]), DbEngine::MariaDb),
        ["mariadb", "mongodb"]
    );
    assert!(detected_as(found(&["apache"]), DbEngine::MySql).is_empty());
}
