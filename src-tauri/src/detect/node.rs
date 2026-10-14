//! Node.js: package.json scripts, Electron, Tauri, web frameworks.

use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use serde_json::Value;

use super::{exists, read_json, tr, Action, Found, Kind};

const SERVER_SCRIPTS: [&str; 5] = ["dev", "start", "serve", "develop", "preview"];
const NODE_SERVERS: [&str; 9] = [
    "express",
    "fastify",
    "koa",
    "@hapi/hapi",
    "@nestjs/core",
    "hono",
    "@adonisjs/core",
    "restify",
    "socket.io",
];
const WEB_FRAMEWORKS: [(&str, &str); 14] = [
    ("next", "Next.js"),
    ("nuxt", "Nuxt"),
    ("@sveltejs/kit", "SvelteKit"),
    ("astro", "Astro"),
    ("@remix-run/dev", "Remix"),
    ("@react-router/dev", "React Router"),
    ("@angular/core", "Angular"),
    ("react-scripts", "Create React App"),
    ("@vue/cli-service", "Vue CLI"),
    ("vite", "Vite"),
    ("parcel", "Parcel"),
    ("webpack-dev-server", "webpack"),
    ("@11ty/eleventy", "Eleventy"),
    ("gatsby", "Gatsby"),
];

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|e| panic!("{pattern}: {e}"))
}

static COMPOSITE: LazyLock<Regex> =
    LazyLock::new(|| re(r"concurrently|npm-run-all|run-p\b|run-s\b|\bturbo\b|\bnx\b|\s&\s"));
static NPX: LazyLock<Regex> = LazyLock::new(|| re(r"^npx\s+"));
static CROSS_ENV: LazyLock<Regex> = LazyLock::new(|| re(r"^cross-env\s+(?:\S+=\S*\s+)*"));
static PORT_RULES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    [
        (
            r"^(vite|remix\s+vite:dev|react-router\s+dev)\b",
            "--port {port} --strictPort --host 127.0.0.1",
        ),
        (r"^next\s+(dev|start)\b", "-p {port} -H 127.0.0.1"),
        (
            r"^(nuxt|nuxi)\s+(dev|preview|start)\b",
            "--port {port} --host 127.0.0.1",
        ),
        (r"^ng\s+serve\b", "--port {port} --host 127.0.0.1"),
        (
            r"^astro\s+(dev|preview|start)\b",
            "--port {port} --host 127.0.0.1",
        ),
        (
            r"^vue-cli-service\s+serve\b",
            "--port {port} --host 127.0.0.1",
        ),
        (
            r"^(webpack\s+serve|webpack-dev-server)\b",
            "--port {port} --host 127.0.0.1",
        ),
        (r"^parcel\b", "--port {port} --host 127.0.0.1"),
        (r"^gatsby\s+develop\b", "-p {port} -H 127.0.0.1"),
        (
            r"^@?11ty/eleventy.*--serve|^eleventy.*--serve",
            "--port={port}",
        ),
        (r"^http-server\b", "-p {port} -a 127.0.0.1"),
        (
            r"^live-server\b",
            "--port={port} --host=127.0.0.1 --no-browser",
        ),
        (r"^serve\b", "-l tcp://127.0.0.1:{port}"),
    ]
    .into_iter()
    .map(|(p, a)| (re(p), a))
    .collect()
});
static SCRIPT_NAME: LazyLock<Regex> = LazyLock::new(|| re(r"^[\w:.@/+-]+$"));
static SERVER_PREFIX: LazyLock<Regex> = LazyLock::new(|| re(r"^(dev|start|serve):"));
static WEB_TOOL: LazyLock<Regex> = LazyLock::new(|| re(r"^(vite|next|nuxt|astro|ng\s+serve)"));
static NODE_RUNNER: LazyLock<Regex> = LazyLock::new(|| re(r"\b(node|nodemon|tsx|ts-node|bun)\b"));
static TAURI_WORD: LazyLock<Regex> = LazyLock::new(|| re(r"\btauri\b"));

/// Arguments appended to `npm run <script> --` so the tool uses the assigned
/// port (npm appends them to the last command of the script). `Some("")`:
/// none needed (`PORT` is enough); `None`: a composite script, left alone.
pub fn port_arguments(script: &str) -> Option<&'static str> {
    if COMPOSITE.is_match(script) {
        return None;
    }
    let last = script.rsplit("&&").next().unwrap_or("").trim();
    let last = NPX.replace(last, "");
    let last = CROSS_ENV.replace(&last, "");
    Some(
        PORT_RULES
            .iter()
            .find(|(re, _)| re.is_match(&last))
            .map_or("", |(_, arg)| arg),
    )
}

fn script_order(name: &str) -> usize {
    SERVER_SCRIPTS
        .iter()
        .position(|s| *s == name)
        .unwrap_or(if SERVER_PREFIX.is_match(name) { 10 } else { 20 })
}

/// Tauri's actions: through the npm script `tauri` or `cargo tauri`.
pub fn tauri(dir: &Path, has_tauri_script: bool, lang: &str) -> (Vec<Action>, Vec<String>) {
    let mut actions = Vec::new();
    let mut warnings = Vec::new();
    if has_tauri_script {
        actions.push(Action::new(
            "tauri:dev",
            "tauri dev",
            "npm run tauri dev",
            Kind::Desktop,
        ));
        actions.push(Action::new(
            "tauri:build",
            "tauri build",
            "npm run tauri build",
            Kind::Command,
        ));
    } else {
        actions.push(Action::new(
            "tauri:dev",
            "cargo tauri dev",
            "cargo tauri dev",
            Kind::Desktop,
        ));
        actions.push(Action::new(
            "tauri:build",
            "cargo tauri build",
            "cargo tauri build",
            Kind::Command,
        ));
        warnings.push(tr(
            lang,
            "Senza lo script npm \"tauri\" serve tauri-cli: cargo install tauri-cli",
            &[],
        ));
    }
    let conf = read_json(dir, r"src-tauri\tauri.conf.json").unwrap_or(Value::Null);
    let dev_url = conf
        .pointer("/build/devUrl")
        .or_else(|| conf.pointer("/build/devPath"))
        .and_then(Value::as_str)
        .filter(|u| u.starts_with("http"));
    warnings.push(match dev_url {
        Some(url) => tr(
            lang,
            "Tauri apre la sua finestra (WebView2) e usa la porta fissa di tauri.conf.json ({0}): qui vedi solo i log.",
            &[url],
        ),
        None => tr(lang, "Tauri apre la sua finestra (WebView2): qui vedi solo i log.", &[]),
    });
    (actions, warnings)
}

pub(super) fn detect(dir: &Path, lang: &str) -> Option<Found> {
    let pkg = read_json(dir, "package.json")?;
    let mut deps = serde_json::Map::new();
    for key in ["dependencies", "devDependencies"] {
        if let Some(Value::Object(m)) = pkg.get(key) {
            deps.extend(m.clone());
        }
    }
    let has = |n: &str| deps.contains_key(n);
    let scripts: Vec<(String, String)> = match pkg.get("scripts") {
        Some(Value::Object(m)) => m
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_owned()))
            .collect(),
        _ => Vec::new(),
    };
    let script = |n: &str| scripts.iter().any(|(k, _)| k == n);
    let mut warnings = Vec::new();
    let mut actions = Vec::new();

    let (mut platform, mut kind, mut reason) = (
        "Node.js".to_owned(),
        Kind::Command,
        "package.json".to_owned(),
    );
    let is_tauri = exists(dir, "src-tauri") || has("@tauri-apps/cli");
    if is_tauri {
        platform = "Tauri".into();
        kind = Kind::Desktop;
        reason = "package.json + src-tauri".into();
    } else if has("electron") {
        platform = "Electron".into();
        kind = Kind::Desktop;
        reason = tr(lang, "{0} con {1}", &["package.json", "electron"]);
    } else if let Some((dep, name)) = WEB_FRAMEWORKS.iter().find(|(n, _)| has(n)) {
        platform = (*name).into();
        kind = Kind::Web;
        reason = tr(lang, "{0} con {1}", &["package.json", dep]);
    } else if let Some(srv) = NODE_SERVERS.iter().find(|n| has(n)) {
        platform = format!("Node.js ({srv})");
        kind = Kind::Web;
        reason = tr(lang, "{0} con {1}", &["package.json", srv]);
    }

    if is_tauri {
        let (a, w) = tauri(dir, script("tauri"), lang);
        actions.extend(a);
        warnings.extend(w);
    }

    // pre/post hooks of an existing script are not actions of their own.
    let mut names: Vec<&(String, String)> = scripts
        .iter()
        .filter(|(n, _)| {
            let base = n.strip_prefix("pre").or_else(|| n.strip_prefix("post"));
            base.is_none_or(|b| !script(b))
        })
        .collect();
    names.sort_by_key(|(n, _)| script_order(n));
    for (name, text) in names {
        if is_tauri && name == "tauri" {
            continue;
        }
        // The name goes into a cmd.exe command: plain names only.
        if !SCRIPT_NAME.is_match(name) {
            warnings.push(tr(
                lang,
                "Script npm \"{0}\" ignorato: il nome contiene caratteri speciali per la shell.",
                &[name],
            ));
            continue;
        }
        let server = SERVER_SCRIPTS.contains(&name.as_str()) || SERVER_PREFIX.is_match(name);
        let mut action_kind = Kind::Command;
        if server && kind == Kind::Desktop {
            action_kind = if WEB_TOOL.is_match(text.trim()) {
                Kind::Web
            } else {
                Kind::Desktop
            };
        } else if server && (kind == Kind::Web || NODE_RUNNER.is_match(text)) {
            action_kind = Kind::Web;
        }
        // Tauri: the frontend alone (vite) is a web server; "tauri dev" opens the window.
        if is_tauri && action_kind == Kind::Desktop && !TAURI_WORD.is_match(text) {
            action_kind = Kind::Web;
        }
        let mut command = format!("npm run {name}");
        if action_kind == Kind::Web {
            if let Some(arg) = port_arguments(text).filter(|a| !a.is_empty()) {
                command = format!("{command} -- {arg}");
            }
        }
        actions.push(Action::new(
            &format!("npm:{name}"),
            &format!("npm run {name}"),
            &command,
            action_kind,
        ));
    }

    if kind == Kind::Desktop && !is_tauri && !script("start") && !script("dev") {
        actions.insert(
            0,
            Action::new("electron", "electron .", "npx electron .", Kind::Desktop),
        );
    }
    actions.push(Action::new(
        "npm:install",
        "npm install",
        "npm install",
        Kind::Command,
    ));

    if !deps.is_empty() && !exists(dir, "node_modules") {
        warnings.push(tr(
            lang,
            "Dipendenze non installate: esegui prima \"npm install\".",
            &[],
        ));
    }
    if ["pnpm-lock.yaml", "yarn.lock", "bun.lockb"]
        .iter()
        .any(|f| exists(dir, f))
    {
        warnings.push(tr(
            lang,
            "Il progetto usa pnpm, yarn o bun: il pannello usa npm. Se serve, cambia i comandi in panel.json.",
            &[],
        ));
    }
    if platform == "Electron" {
        warnings.push(tr(
            lang,
            "Electron apre la sua finestra: qui vedi solo i log.",
            &[],
        ));
    }
    Some(Found {
        platform,
        kind,
        reason,
        actions,
        warnings,
    })
}

#[cfg(test)]
mod tests {
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
}
