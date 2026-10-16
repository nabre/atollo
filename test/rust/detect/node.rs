use super::*;
use crate::detect::detect as detect_all;
use crate::detect::tests::{ids, project};

#[test]
fn port_arguments_rules() {
    assert_eq!(
        port_arguments("vite"),
        Some("--port {port} --strictPort --host 127.0.0.1")
    );
    assert_eq!(
        port_arguments("tsc && npx vite --open"),
        Some("--port {port} --strictPort --host 127.0.0.1")
    );
    assert_eq!(
        port_arguments("cross-env NODE_ENV=dev next dev"),
        Some("-p {port} -H 127.0.0.1")
    );
    assert_eq!(port_arguments("concurrently \"a\" \"b\""), None);
    assert_eq!(port_arguments("node server.js"), Some(""));
}

#[test]
fn frameworks_and_scripts() {
    let tmp = project(
        "node-next",
        &[(
            "package.json",
            r#"{"scripts":{"build":"next build","dev":"next dev","predev":"echo","bad name":"x","start":"next start"},
                    "dependencies":{"next":"15","react":"19"}}"#,
        )],
    );
    let p = detect_all(&tmp.0, "it");
    assert_eq!(p.platform, "Next.js");
    assert_eq!(p.kind, Kind::Web);
    assert_eq!(
        ids(&p),
        ["npm:dev", "npm:start", "npm:build", "npm:install"]
    );
    assert_eq!(
        p.actions[0].command,
        "npm run dev -- -p {port} -H 127.0.0.1"
    );
    assert_eq!(p.actions[2].kind, Kind::Command);
    assert_eq!(p.default.as_deref(), Some("npm:dev"));
    assert!(p.warnings.iter().any(|w| w.contains("bad name")));
    assert!(p.warnings.iter().any(|w| w.contains("npm install")));
}

#[test]
fn express_tauri_electron() {
    let srv = project(
        "node-express",
        &[(
            "package.json",
            r#"{"scripts":{"start":"node index.js"},"dependencies":{"express":"5"}}"#,
        )],
    );
    let p = detect_all(&srv.0, "it");
    assert_eq!(p.platform, "Node.js (express)");
    assert_eq!(p.actions[0].command, "npm run start");
    assert_eq!(p.actions[0].kind, Kind::Web);

    let tauri_app = project(
        "node-tauri",
        &[
            (
                "package.json",
                r#"{"scripts":{"dev":"vite","tauri":"tauri"},"devDependencies":{"@tauri-apps/cli":"2","vite":"6"}}"#,
            ),
            (
                "src-tauri/tauri.conf.json",
                r#"{"build":{"devUrl":"http://localhost:1420"}}"#,
            ),
            ("src-tauri/Cargo.toml", "[dependencies]\ntauri = \"2\"\n"),
        ],
    );
    let p = detect_all(&tauri_app.0, "it");
    assert_eq!(p.kind, Kind::Desktop);
    assert_eq!(&ids(&p)[..3], ["tauri:dev", "tauri:build", "npm:dev"]);
    assert_eq!(p.actions[2].kind, Kind::Web);
    assert!(p.warnings.iter().any(|w| w.contains("1420")));

    let electron = project(
        "node-electron",
        &[("package.json", r#"{"devDependencies":{"electron":"30"}}"#)],
    );
    let p = detect_all(&electron.0, "it");
    assert_eq!(ids(&p), ["electron", "npm:install"]);
    assert_eq!(p.kind, Kind::Desktop);
}

#[test]
fn other_package_managers_warned() {
    let uses_npm = |name: &str, lock: &str| {
        let tmp = project(name, &[("package.json", "{}"), (lock, "")]);
        !detect_all(&tmp.0, "it")
            .warnings
            .iter()
            .any(|w| w.contains("pnpm, yarn o bun"))
    };
    for lock in ["pnpm-lock.yaml", "yarn.lock", "bun.lock", "bun.lockb"] {
        assert!(!uses_npm(&format!("node-{lock}"), lock), "{lock}");
    }
    assert!(uses_npm("node-npm", "package-lock.json"));
}
