use super::*;
use crate::kit::tests::TempDir;

pub fn project(name: &str, files: &[(&str, &str)]) -> TempDir {
    let tmp = TempDir::new(&format!("detect-{name}"));
    for (path, text) in files {
        let p = tmp.0.join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
    tmp
}

/// A command as the system's shell takes it: `cmd.exe` on Windows (double
/// quotes, `%CP_DATA%`), `/bin/sh` elsewhere (single quotes, `$CP_DATA`);
/// `windows` and `unix` are the expected command of each.
pub fn sh<'a>(windows: &'a str, unix: &'a str) -> &'a str {
    if cfg!(windows) {
        windows
    } else {
        unix
    }
}

#[test]
fn databases_a_project_uses() {
    let laravel = project(
        "db-laravel",
        &[
            (".env", "APP_NAME=x\nDB_CONNECTION=mysql\n"),
            ("composer.json", "{}"),
        ],
    );
    assert_eq!(databases(&laravel.0), BTreeSet::from(["mysql"]));
    let node = project(
        "db-node",
        &[(
            "package.json",
            r#"{"dependencies":{"mongoose":"^8","better-sqlite3":"^11"}}"#,
        )],
    );
    assert_eq!(databases(&node.0), BTreeSet::from(["mongodb", "sqlite"]));
    let py = project(
        "db-py",
        &[
            ("requirements.txt", "flask\npymongo==4\n"),
            ("data/app.sqlite3", ""),
        ],
    );
    assert_eq!(databases(&py.0), BTreeSet::from(["mongodb", "sqlite"]));
    // Words inside other names are not drivers.
    let none = project(
        "db-none",
        &[(
            "package.json",
            r#"{"dependencies":{"mysqlish-ui":"1","notmongodb":"1"}}"#,
        )],
    );
    assert!(databases(&none.0).is_empty());
    assert!(requires(&laravel.0).contains(&"mysql".to_owned()));
}

/// D-117: MariaDB in a project is the kit's MySQL; Laravel's `mariadb`
/// driver is told to become `mysql`, never MariaDB proposed.
#[test]
fn mariadb_of_a_project_is_mysql_with_a_warning() {
    let laravel = project(
        "db-laravel-mariadb",
        &[
            (".env", "APP_NAME=x\nDB_CONNECTION=\"mariadb\"\n"),
            ("composer.json", "{}"),
            ("compose.yaml", "services:\n  db:\n    image: mysql:8.4\n"),
        ],
    );
    let p = detect(&laravel.0, "it");
    assert!(p.requires.contains(&"mysql".to_owned()), "{:?}", p.requires);
    assert!(!p.requires.contains(&"mariadb".to_owned()));
    assert_eq!(p.lines.get("mysql").map(String::as_str), Some("8.4"));
    assert!(
        p.warnings.iter().any(|w| w.contains("DB_CONNECTION=mysql")),
        "{:?}",
        p.warnings
    );
    let mysql = project(
        "db-laravel-mysql",
        &[(".env", "DB_CONNECTION=mysql\n"), ("composer.json", "{}")],
    );
    assert!(!env_wants_mariadb(&mysql.0));
    assert!(!detect(&mysql.0, "it")
        .warnings
        .iter()
        .any(|w| w.contains("DB_CONNECTION")));
    let example = project(
        "db-example-mariadb",
        &[(".env.example", "DB_CONNECTION=mariadb\n")],
    );
    assert!(env_wants_mariadb(&example.0), ".env.example without .env");
}

pub fn ids(p: &Project) -> Vec<&str> {
    p.actions.iter().map(|a| a.id.as_str()).collect()
}

#[test]
fn unknown_and_static() {
    let empty = project("empty", &[]);
    let p = detect(&empty.0, "it");
    assert_eq!(p.platform, "Sconosciuto");
    assert!(p.actions.is_empty() && p.default.is_none());

    let site = project("static", &[("index.html", "<h1>x</h1>"), ("style.css", "")]);
    let p = detect(&site.0, "en");
    assert_eq!(p.kind, Kind::Web);
    assert_eq!(p.actions[0].static_dir.as_deref(), Some("."));
    assert_eq!(p.default.as_deref(), Some("static"));
    assert_eq!(p.languages, ["CSS", "HTML"]);
    assert_eq!(ids(&p), ["static"]);
}

/// `nginx.conf` or `.nginx` outside the PHP projects: a static site gets a
/// Nginx of its own after the file server.
#[test]
fn static_site_with_nginx() {
    for file in ["nginx.conf", ".nginx", "public/nginx.conf"] {
        let site = project(
            &format!("static-nginx-{}", file.replace(['/', '.'], "_")),
            &[("public/index.html", ""), (file, "")],
        );
        let p = detect(&site.0, "it");
        assert_eq!(ids(&p), ["static", "nginx"], "{file}");
        assert_eq!(p.default.as_deref(), Some("static"));
        assert_eq!(p.actions[1].nginx_dir.as_deref(), Some("public"));
        assert!(p.requires.contains(&"nginx".to_owned()), "{file}");
    }
    let php = project("php-dot-nginx", &[("index.php", "<?php"), (".nginx", "")]);
    assert!(ids(&detect(&php.0, "it")).contains(&"nginx"));
}

#[test]
fn composite_project() {
    let tmp = project(
        "composite",
        &[
            (
                "frontend/package.json",
                r#"{"scripts":{"dev":"vite"},"devDependencies":{"vite":"6"}}"#,
            ),
            ("backend/requirements.txt", "flask\n"),
            (
                "backend/app.py",
                "from flask import Flask\napp = Flask(__name__)\n",
            ),
            ("node_modules/x/package.json", "{}"),
        ],
    );
    let p = detect(&tmp.0, "it");
    assert!(
        p.platform.contains("Vite (frontend)") && p.platform.contains("Flask (backend)"),
        "{}",
        p.platform
    );
    let dev = p
        .actions
        .iter()
        .find(|a| a.id == "frontend/npm:dev")
        .unwrap();
    assert_eq!(dev.folder.as_deref(), Some("frontend"));
    assert_eq!(
        dev.command,
        "npm run dev -- --port {port} --strictPort --host 127.0.0.1"
    );
    assert!(ids(&p).contains(&"backend/py:flask"));
    assert!(p.requires.contains(&"python".to_owned()));
}

#[test]
fn quoting() {
    assert_eq!(quote("public"), "public");
    assert_eq!(quote("my site"), sh("\"my site\"", "'my site'"));
    assert_eq!(quote("a&b"), sh("\"a&b\"", "'a&b'"));
    // `/bin/sh` only: a quote in a name stays text
    if cfg!(not(windows)) {
        assert_eq!(quote("it's"), r"'it'\''s'");
    }
}
