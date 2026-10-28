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
    assert_eq!(
        databases(&laravel.0, &walk::walk(&laravel.0)),
        BTreeSet::from(["mysql"])
    );
    let node = project(
        "db-node",
        &[(
            "package.json",
            r#"{"dependencies":{"mongoose":"^8","better-sqlite3":"^11"}}"#,
        )],
    );
    assert_eq!(
        databases(&node.0, &walk::walk(&node.0)),
        BTreeSet::from(["mongodb", "sqlite"])
    );
    // The npm driver of MariaDB: the kit's MySQL.
    let maria = project(
        "db-node-maria",
        &[("package.json", r#"{"dependencies":{"mariadb":"^3"}}"#)],
    );
    assert_eq!(
        databases(&maria.0, &walk::walk(&maria.0)),
        BTreeSet::from(["mysql"])
    );
    let py = project(
        "db-py",
        &[
            ("requirements.txt", "flask\npymongo==4\n"),
            ("data/app.sqlite3", ""),
        ],
    );
    assert_eq!(
        databases(&py.0, &walk::walk(&py.0)),
        BTreeSet::from(["mongodb", "sqlite"])
    );
    // Words inside other names are not drivers.
    let none = project(
        "db-none",
        &[(
            "package.json",
            r#"{"dependencies":{"mysqlish-ui":"1","notmongodb":"1"}}"#,
        )],
    );
    assert!(databases(&none.0, &walk::walk(&none.0)).is_empty());
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

/// Nginx is no longer in the kit (D-120): a `nginx.conf` or `.nginx` asks
/// for nothing, the static site has the file server, the PHP one Caddy.
#[test]
fn nginx_files_ask_for_nothing() {
    for file in ["nginx.conf", ".nginx", "public/nginx.conf"] {
        let site = project(
            &format!("static-nginx-{}", file.replace(['/', '.'], "_")),
            &[("public/index.html", ""), (file, "")],
        );
        let p = detect(&site.0, "it");
        assert_eq!(ids(&p), ["static"], "{file}");
        assert!(!p.requires.contains(&"nginx".to_owned()), "{file}");
    }
    let php = project("php-dot-nginx", &[("index.php", "<?php"), (".nginx", "")]);
    let p = detect(&php.0, "it");
    assert!(!ids(&p).contains(&"nginx"));
    assert!(ids(&p).contains(&"caddy"));
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

/// D-186: the tools of every file count, at any depth, not only those of
/// the root and its first folders.
#[test]
fn tools_are_found_at_any_depth() {
    let p = project(
        "deep",
        &[
            ("a/b/c/d/go.mod", "module x\n"),
            ("a/b/c/d/e/Cargo.toml", "[package]\nname='x'\n"),
            ("one/two/three/four/five/app.rb", "puts 1\n"),
            ("svc/api/package.json", r#"{"packageManager":"pnpm@9.1.0"}"#),
            (
                "tools/.tool-versions",
                "python 3.12.4\ngolang 1.22\nunknown 1\n",
            ),
            (
                "x/y/rust-toolchain.toml",
                "[toolchain]\nchannel = \"1.80\"\n",
            ),
        ],
    );
    let found = requires(&p.0);
    for id in ["go", "rust", "ruby", "node", "pnpm", "python"] {
        assert!(found.contains(&id.to_owned()), "{id}: {found:?}");
    }
    assert!(languages(&p.0).contains(&"Ruby".to_owned()));
}

/// What holds no sources is not read: dependencies, builds, the VM's and
/// the editor's folders (every folder whose name starts with `.`).
#[test]
fn ignored_folders_are_not_read() {
    let p = project(
        "ignored",
        &[
            ("node_modules/pkg/Cargo.toml", ""),
            ("vendor/bundle/ruby/x.rb", ""),
            ("target/linux/debug/x.go", ""),
            (".venv-linux/lib/x.py", ""),
            (".git/hooks/x.php", ""),
            ("src/.hidden/x.java", ""),
        ],
    );
    assert!(requires(&p.0).is_empty(), "{:?}", requires(&p.0));
}

/// A link to a folder is never entered (D-77), at any depth.
#[test]
fn a_linked_folder_is_not_walked() {
    use crate::platform::safe_path::tests::{junction, remove_junction};
    let outside = project("link-target", &[("Cargo.toml", "")]);
    let p = project("linked", &[("web/index.html", "")]);
    let link = p.0.join("web").join("lib");
    junction(&link, &outside.0);
    let walk = walk::walk(&p.0);
    assert_eq!(walk.files, ["web/index.html"]);
    assert!(!requires(&p.0).contains(&"rust".to_owned()));
    remove_junction(&link);
}

/// The budget stops a walk where it is, and the project says so; the
/// files near the root are read first.
#[test]
fn a_walk_stops_at_its_budget() {
    let mut files: Vec<(String, &str)> = (0..30).map(|i| (format!("deep/f{i}.txt"), "")).collect();
    files.push(("go.mod".into(), "module x\n"));
    let files: Vec<(&str, &str)> = files.iter().map(|(p, t)| (p.as_str(), *t)).collect();
    let p = project("budget", &files);
    let cut = walk::walk_with(&p.0, 10, walk::MAX_TIME);
    assert!(cut.cut);
    assert!(cut.files.contains(&"go.mod".to_owned()), "{:?}", cut.files);
    let whole = walk::walk_with(&p.0, 1000, walk::MAX_TIME);
    assert!(!whole.cut);
    assert_eq!(whole.files.len(), 31);
    assert_eq!(walk::depth("a/b/c"), 2);
    assert_eq!(walk::name_of("a/b/c.txt"), "c.txt");
    assert!(!detect(&p.0, "it").partial);
}
