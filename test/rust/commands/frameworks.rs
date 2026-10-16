use super::merge_copy;
use crate::kit::tests::TempDir;
use std::fs;
use std::path::Path;

/// As `merge_into`, by copy: the project's files stay, the source too.
#[test]
fn merge_copy_keeps_what_the_project_has() {
    let tmp = TempDir::new("mergecopy");
    let (made, dir) = (tmp.0.join("made"), tmp.0.join("app"));
    fs::create_dir_all(made.join(r"src\lib")).unwrap();
    fs::write(made.join("package.json"), "new").unwrap();
    fs::write(made.join(r"src\main.js"), "new").unwrap();
    fs::write(made.join(r"src\lib\a.js"), "new").unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("package.json"), "mine").unwrap();
    let mut kept = Vec::new();
    merge_copy(&made, &dir, Path::new(""), &mut kept).unwrap();
    assert_eq!(kept, ["package.json"]);
    assert_eq!(
        fs::read_to_string(dir.join("package.json")).unwrap(),
        "mine"
    );
    assert_eq!(
        fs::read_to_string(dir.join(r"src\lib\a.js")).unwrap(),
        "new"
    );
    assert!(made.join(r"src\main.js").is_file());
}

/// `.env` holds passwords: listed in `.gitignore` when Git would take it.
#[test]
fn env_goes_in_gitignore() {
    let tmp = TempDir::new("ignoreenv");
    let dir = tmp.0.join("site");
    fs::create_dir_all(&dir).unwrap();
    super::ignore_env(&dir).unwrap();
    assert!(!dir.join(".gitignore").exists(), "no .env, nothing to do");
    fs::write(dir.join(".env"), "A=1\n").unwrap();
    super::ignore_env(&dir).unwrap();
    assert_eq!(
        fs::read_to_string(dir.join(".gitignore")).unwrap(),
        "/.env\n"
    );
    fs::write(dir.join(".gitignore"), "/vendor\r\n/var").unwrap();
    super::ignore_env(&dir).unwrap();
    super::ignore_env(&dir).unwrap();
    assert_eq!(
        fs::read_to_string(dir.join(".gitignore")).unwrap(),
        "/vendor\r\n/var\r\n/.env\r\n"
    );
}

#[test]
fn database_for_the_steps() {
    let db = crate::db::auth::project_db("mariadb", "site", "pw", 3307);
    let vars = super::db_vars(&db);
    let get = |k: &str| vars.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    assert_eq!(get("ATOLLO_DB_PORT"), Some("3307"));
    assert_eq!(get("ATOLLO_DB_NAME"), Some("site"));
    assert_eq!(
        get("ATOLLO_DB_URL"),
        Some("mysql://site:pw@127.0.0.1:3307/site")
    );
}
