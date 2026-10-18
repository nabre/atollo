use super::merge_copy;
use crate::kit::tests::TempDir;
use std::fs;
use std::path::Path;

/// As `merge_into`, by copy: the project's files stay, the source too.
#[test]
fn merge_copy_keeps_what_the_project_has() {
    let tmp = TempDir::new("mergecopy");
    let (made, dir) = (tmp.0.join("made"), tmp.0.join("app"));
    fs::create_dir_all(made.join("src").join("lib")).unwrap();
    fs::write(made.join("package.json"), "new").unwrap();
    fs::write(made.join("src").join("main.js"), "new").unwrap();
    fs::write(made.join("src").join("lib").join("a.js"), "new").unwrap();
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
        fs::read_to_string(dir.join("src").join("lib").join("a.js")).unwrap(),
        "new"
    );
    assert!(made.join("src").join("main.js").is_file());
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

/// Issue #3: phpMyAdmin and the databases of WordPress and Laravel install
/// MySQL, and phpMyAdmin is available on the six targets.
#[test]
fn a_new_kit_installs_mysql_for_phpmyadmin_wordpress_and_laravel() {
    use crate::kit::Target;
    use crate::projects::frameworks::{official, plan};
    use crate::setup::catalog::Catalog;
    let catalog = Catalog::get();
    let wordpress = plan(official(), "wordpress", &[]).unwrap();
    let laravel = plan(official(), "laravel", &["mysql".to_owned()]).unwrap();
    let wanted: [(&str, Vec<&str>); 3] = [
        ("phpmyadmin", vec!["phpmyadmin"]),
        ("wordpress", super::plan_needs(&wordpress)),
        ("laravel", super::plan_needs(&laravel)),
    ];
    for target in Target::ALL {
        for (what, ids) in &wanted {
            let needed = crate::commands::with_needs(catalog, ids, target);
            let needed: Vec<&str> = needed.iter().map(|c| c.id.as_str()).collect();
            assert!(
                needed.contains(&"mysql"),
                "{what} on {target:?}: {needed:?}"
            );
        }
        for id in crate::commands::with_needs(catalog, &["phpmyadmin"], target) {
            assert!(id.is_available_on(target), "{} on {target:?}", id.id);
        }
    }
}

#[test]
fn database_for_the_steps() {
    let db = crate::db::auth::project_db("mysql", "site", "pw", 3307);
    let vars = super::db_vars(&db);
    let get = |k: &str| vars.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    assert_eq!(get("ATOLLO_DB_PORT"), Some("3307"));
    assert_eq!(get("ATOLLO_DB_NAME"), Some("site"));
    assert_eq!(
        get("ATOLLO_DB_URL"),
        Some("mysql://site:pw@127.0.0.1:3307/site")
    );
}
