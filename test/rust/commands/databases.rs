use super::*;

/// The interface knows `mysql` and `mongodb` only (D-117): a SQL server
/// found by the detection is `mysql`, once; others are kept, unknown ids
/// left out.
#[test]
fn detected_servers_as_the_projects() {
    let found = |ids: &[&str]| ids.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    assert_eq!(
        detected_as(found(&["mariadb", "sqlite", "redis"])),
        ["mysql", "sqlite"]
    );
    assert_eq!(
        detected_as(found(&["mariadb", "mysql", "mongodb"])),
        ["mysql", "mongodb"]
    );
    assert!(detected_as(found(&["apache"])).is_empty());
}

/// A project's database still on a MariaDB whose data has not moved yet
/// reaches the interface as `mysql` (D-117).
#[test]
fn the_interface_says_mysql_for_every_sql_server() {
    assert_eq!(ui_id("mariadb"), "mysql");
    assert_eq!(ui_id("mysql"), "mysql");
    assert_eq!(ui_id("mongodb"), "mongodb");
    let db = for_ui(crate::db::auth::project_db("mariadb", "shop", "pw", 3307));
    assert_eq!(db.engine, "mysql");
    assert_eq!(db.port, 3307, "the port of the server it is on");
}
