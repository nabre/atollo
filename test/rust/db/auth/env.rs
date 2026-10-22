use super::*;

#[test]
fn env_for_laravel_with_mongodb() {
    let db = project_db("mongodb", "shop", "pw1", 27018);
    let out = env_apply_text("DB_CONNECTION=sqlite\n", &db);
    assert!(out.contains("DB_CONNECTION=mongodb"), "{out}");
    assert!(out.contains("DB_DATABASE=shop"), "{out}");
    assert!(
        out.contains("MONGODB_URI=mongodb://shop:pw1@127.0.0.1:27018/shop"),
        "{out}"
    );
}

/// «Copia blocco .env»: every line of the connection, whatever the file
/// already has, in the shape the project's `.env` uses.
#[test]
fn block_to_copy() {
    let db = project_db("mysql", "shop", "pw1", 3307);
    assert_eq!(env_block("", &db), format!("DATABASE_URL={}\n", db.url));
    let laravel = env_block("DB_CONNECTION=sqlite\nDB_DATABASE=shop\n", &db);
    assert_eq!(
        laravel,
        "DB_CONNECTION=mysql\nDB_HOST=127.0.0.1\nDB_PORT=3307\nDB_DATABASE=shop\nDB_USERNAME=shop\nDB_PASSWORD=pw1\n"
    );
    let mongo = project_db("mongodb", "shop", "pw2", 27017);
    assert!(env_block("", &mongo).starts_with("MONGODB_URI=mongodb://shop:pw2@"));
}

#[test]
fn env_for_laravel_and_others() {
    let db = project_db("mysql", "shop", "pw1", 3307);
    let laravel = "APP_NAME=Shop\r\n# DB_HOST=old\r\nDB_CONNECTION=sqlite\r\nDB_HOST=10.0.0.1\r\n";
    let changes = env_changes(laravel, &db);
    assert_eq!(changes[0].key, "DB_CONNECTION");
    assert_eq!(changes[0].old.as_deref(), Some("sqlite"));
    assert!(changes
        .iter()
        .any(|c| c.key == "DB_PASSWORD" && c.old.is_none()));
    let out = env_apply_text(laravel, &db);
    assert!(
        out.contains("# DB_HOST=old\r\n"),
        "comments untouched: {out}"
    );
    assert!(out.contains("DB_HOST=127.0.0.1\r\nDB_PORT=3307") || out.contains("DB_PORT=3307"));
    assert!(out.contains("DB_DATABASE=shop\r\n"));
    assert!(out.starts_with("APP_NAME=Shop\r\n"));
    assert!(env_changes(&out, &db).is_empty(), "applied: nothing left");
    let node = project_db("mongodb", "shop", "pw2", 27017);
    let out = env_apply_text("PORT=3000\n", &node);
    assert_eq!(
        out,
        "PORT=3000\nMONGODB_URI=mongodb://shop:pw2@127.0.0.1:27017/shop?authSource=shop\n"
    );
    let other = env_apply_text("", &db);
    assert_eq!(
        other.trim(),
        "DATABASE_URL=mysql://shop:pw1@127.0.0.1:3307/shop"
    );
}
