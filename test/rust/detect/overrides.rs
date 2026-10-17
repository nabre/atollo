use super::*;
use crate::detect::detect;
use crate::detect::tests::{ids, project, sh};

fn edit(id: &str, command: &str) -> ActionEdit {
    ActionEdit {
        id: id.into(),
        label: "API".into(),
        command: command.into(),
        kind: "web".into(),
        folder: None,
        url: None,
        env: Default::default(),
    }
}

#[test]
fn actions_edited_from_the_page() {
    let tmp = project(
        "edit-actions",
        &[
            ("package.json", r#"{"scripts":{"dev":"vite"}}"#),
            ("api/main.py", "print(1)"),
        ],
    );
    let mut a = edit("api", "python main.py --port {port}");
    a.folder = Some("api".into());
    a.env.insert("APP_ENV".into(), "local".into());
    save_action(&tmp.0, &a, "it").unwrap();
    let p = detect(&tmp.0, "it");
    let api = p.actions.iter().find(|x| x.id == "api").unwrap();
    assert_eq!(api.folder.as_deref(), Some("api"));
    assert_eq!(api.env["APP_ENV"], "local");
    assert!(api.custom);
    // Same id: replaced, not added.
    a.command = "uvicorn main:app --port {port}".into();
    save_action(&tmp.0, &a, "it").unwrap();
    let p = detect(&tmp.0, "it");
    assert_eq!(p.actions.iter().filter(|x| x.id == "api").count(), 1);
    // A detected action corrected, then back to the detected one.
    save_action(&tmp.0, &edit("npm:dev", "npm run dev -- --host"), "it").unwrap();
    let p = detect(&tmp.0, "it");
    let dev: Vec<_> = p.actions.iter().filter(|x| x.id == "npm:dev").collect();
    assert_eq!(dev.len(), 1);
    assert_eq!(dev[0].command, "npm run dev -- --host");
    assert!(dev[0].custom);
    remove_action(&tmp.0, "npm:dev", "it").unwrap();
    let p = detect(&tmp.0, "it");
    assert!(
        !p.actions.iter().find(|x| x.id == "npm:dev").unwrap().custom,
        "detected again"
    );
    set_default_action(&tmp.0, "api", "it").unwrap();
    assert_eq!(detect(&tmp.0, "it").default.as_deref(), Some("api"));
    remove_action(&tmp.0, "api", "it").unwrap();
    let p = detect(&tmp.0, "it");
    assert!(!p.actions.iter().any(|x| x.id == "api"));
    assert_ne!(p.default.as_deref(), Some("api"));
    assert!(!tmp.0.join("panel.json").exists(), "empty file removed");
    // Refused: bad id, two lines, outside folder, bad variable.
    assert!(save_action(&tmp.0, &edit("A B", "x"), "it").is_err());
    assert!(save_action(&tmp.0, &edit("x", "a\nb"), "it").is_err());
    let mut out = edit("x", "y");
    out.folder = Some("../..".into());
    assert!(save_action(&tmp.0, &out, "it").is_err());
    let mut var = edit("x", "y");
    var.env.insert("1A".into(), "z".into());
    assert!(save_action(&tmp.0, &var, "it").is_err());
}

#[test]
fn old_file_keeps_its_keys() {
    let tmp = project(
        "edit-old",
        &[(
            "pannello.json",
            r#"{"nome":"Vecchio","azioni":[{"id":"a","etichetta":"A","comando":"x"}]}"#,
        )],
    );
    save_action(&tmp.0, &edit("b", "y"), "it").unwrap();
    let text = fs::read_to_string(tmp.0.join("pannello.json")).unwrap();
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["nome"], "Vecchio");
    assert_eq!(v["azioni"].as_array().unwrap().len(), 2);
    assert!(v.get("actions").is_none());
}

#[test]
fn overrides_and_old_file() {
    let tmp = project(
        "ovr",
        &[
            (
                "package.json",
                r#"{"scripts":{"dev":"vite"},"devDependencies":{"vite":"6"}}"#,
            ),
            (
                "panel.json",
                r#"{"name":"Gestionale","default":"all","actions":[
                    {"id":"all","label":"Tutto","type":"web","command":"npm run dev -- --port {port}","env":{"X":"{port}"}},
                    {"id":"site","static":"public"},
                    {"label":"no id","command":"x"}]}"#,
            ),
        ],
    );
    let p = detect(&tmp.0, "it");
    assert!(p.custom);
    assert_eq!(p.name, "Gestionale");
    assert_eq!(&ids(&p)[..3], ["all", "site", "npm:dev"]);
    assert_eq!(p.actions[0].env["X"], "{port}");
    assert_eq!(p.actions[1].static_dir.as_deref(), Some("public"));
    assert_eq!(p.default.as_deref(), Some("all"));

    let old = project(
        "ovr-old",
        &[(
            "pannello.json",
            r#"{"sostituisci":true,"tipo":"comando","azioni":[{"id":"a","comando":"echo {porta}"}]}"#,
        )],
    );
    let p = detect(&old.0, "it");
    assert_eq!(ids(&p), ["a"]);
    assert_eq!(p.actions[0].command, "echo {port}");
    assert_eq!(p.kind, Kind::Command);

    let bad = project("ovr-bad", &[("panel.json", "{nope")]);
    assert!(detect(&bad.0, "it").warnings[0].contains("panel.json"));
}

#[test]
fn site_written_and_removed() {
    let tmp = project(
        "site",
        &[("www/index.html", ""), ("panel.json", r#"{"name":"X"}"#)],
    );
    assert!(set_site(&tmp.0, "..", SiteKind::Static, "it").is_err());
    assert!(set_site(&tmp.0, "missing", SiteKind::Static, "it").is_err());
    set_site(&tmp.0, "www", SiteKind::Static, "it").unwrap();
    let p = detect(&tmp.0, "it");
    assert_eq!(p.default.as_deref(), Some("site"));
    assert_eq!(p.actions[0].static_dir.as_deref(), Some("www"));
    assert_eq!(p.name, "X");
    set_site(&tmp.0, "www", SiteKind::Php, "it").unwrap();
    assert_eq!(
        detect(&tmp.0, "it").actions[0].command,
        sh(
            r#"php -S 127.0.0.1:{port} -t www "%CP_DATA%\kit\php-router.php""#,
            r#"php -S 127.0.0.1:{port} -t www "$CP_DATA/kit/php-router.php""#
        )
    );
    set_site(&tmp.0, "www", SiteKind::Apache, "it").unwrap();
    let p = detect(&tmp.0, "it");
    assert_eq!(p.actions[0].apache_dir.as_deref(), Some("www"));
    assert_eq!(p.actions[0].kind, Kind::Web);
    assert!(p.actions[0].command.is_empty());
    assert!(p.requires.contains(&"apache".to_owned()) && p.requires.contains(&"php".to_owned()));
    set_site(&tmp.0, "", SiteKind::None, "it").unwrap();
    let left: Value =
        serde_json::from_str(&fs::read_to_string(tmp.0.join("panel.json")).unwrap()).unwrap();
    assert_eq!(left, serde_json::json!({"name":"X"}));

    let only = project("site-only", &[("index.html", "")]);
    set_site(&only.0, ".", SiteKind::Static, "it").unwrap();
    set_site(&only.0, ".", SiteKind::None, "it").unwrap();
    assert!(!only.0.join("panel.json").exists());
}
