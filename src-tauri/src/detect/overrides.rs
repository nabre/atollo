//! `panel.json` in the project folder: corrects or replaces what was
//! detected. Keys mirror the Project and Action fields:
//!
//! ```json
//! { "name": "Gestionale", "type": "web", "replace": false, "default": "dev",
//!   "actions": [
//!     { "id": "dev", "label": "Avvia tutto", "type": "web",
//!       "command": "npm run dev -- --port {port}", "folder": "frontend",
//!       "url": "http://localhost:{port}/admin", "env": { "API_URL": "http://localhost:8000" } },
//!     { "id": "site", "label": "Sito", "static": "public" } ] }
//! ```
//!
//! `static` (instead of `command`): that folder served by the exe's file
//! server, always a web action. The old `pannello.json` (read only when
//! `panel.json` is missing) uses the Italian keys `nome`, `tipo`,
//! `piattaforma`, `sostituisci`, `predefinita`, `azioni`, `etichetta`,
//! `comando`, `cartella`, `statico` and `{porta}`; both sets are accepted in
//! either file. The file holds no path of the kit, which may move.

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde_json::{Map, Value};

use super::{quote, tr, Action, Kind, Project, OVERRIDE_FILES};
use crate::kit::write_atomic;

/// Id of the action written by "Servi come sito web".
pub const SITE_ACTION: &str = "site";

fn pick<'a>(o: &'a Map<String, Value>, en: &str, it: &str) -> Option<&'a Value> {
    o.get(en)
        .filter(|v| !v.is_null() && v.as_str() != Some(""))
        .or_else(|| o.get(it))
        .filter(|v| !v.is_null())
}

fn text(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// `{porta}` of old files becomes `{port}`.
fn ports(s: &str) -> String {
    s.replace("{porta}", "{port}")
}

/// The override file of `dir`, if any (panel.json first).
pub fn file_of(dir: &Path) -> Option<PathBuf> {
    OVERRIDE_FILES
        .iter()
        .map(|f| dir.join(f))
        .find(|p| crate::safe_path::is_file(dir, p))
}

fn read_object(dir: &Path, path: &Path) -> Option<Map<String, Value>> {
    let bytes = crate::safe_path::read(dir, path)?;
    let text = String::from_utf8_lossy(&bytes);
    match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
        Ok(Value::Object(o)) => Some(o),
        _ => None,
    }
}

pub(super) fn apply(project: &mut Project, dir: &Path, lang: &str) {
    let Some(path) = file_of(dir) else { return };
    let file = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let Some(o) = read_object(dir, &path) else {
        project
            .warnings
            .insert(0, tr(lang, "{0} non è un JSON valido: ignorato.", &[&file]));
        return;
    };
    project.custom = true;
    let kind = text(pick(&o, "type", "tipo")).and_then(|t| Kind::parse(&t));
    if let Some(name) = text(pick(&o, "name", "nome")) {
        project.name = name;
    }
    if let Some(k) = kind {
        project.kind = k;
    }
    if let Some(p) = text(pick(&o, "platform", "piattaforma")) {
        project.platform = p;
    }
    if let Some(d) = text(pick(&o, "default", "predefinita")) {
        project.default = Some(d);
    }
    project.reason = format!(
        "{} · {}",
        project.reason,
        tr(lang, "personalizzato da {0}", &[&file])
    );
    let Some(Value::Array(list)) = pick(&o, "actions", "azioni") else {
        return;
    };
    let added: Vec<Action> = list
        .iter()
        .filter_map(Value::as_object)
        .filter_map(|a| {
            let id = text(a.get("id"))?;
            let command = text(pick(a, "command", "comando")).map(|c| ports(&c));
            let served = pick(a, "static", "statico")
                .and_then(Value::as_str)
                .map(str::to_owned);
            if command.is_none() && served.is_none() {
                return None;
            }
            let a_kind = text(pick(a, "type", "tipo")).and_then(|t| Kind::parse(&t));
            let label = text(pick(a, "label", "etichetta")).unwrap_or_else(|| id.clone());
            let mut action = match &command {
                Some(c) => Action::new(&id, &label, c, a_kind.unwrap_or(Kind::Command)),
                None => Action::serve(&id, &label, served.as_deref().unwrap_or(".")),
            };
            action.folder = text(pick(a, "folder", "cartella"));
            action.url = text(a.get("url")).map(|u| ports(&u));
            if let Some(Value::Object(env)) = a.get("env") {
                for (k, v) in env {
                    if let Some(v) = text(Some(v)) {
                        action.env.insert(k.clone(), ports(&v));
                    }
                }
            }
            action.custom = true;
            Some(action)
        })
        .collect();
    let replace = pick(&o, "replace", "sostituisci").and_then(Value::as_bool) == Some(true);
    let remaining: Vec<Action> = if replace {
        Vec::new()
    } else {
        project
            .actions
            .iter()
            .filter(|a| !added.iter().any(|n| n.id == a.id))
            .cloned()
            .collect()
    };
    project.actions = added.into_iter().chain(remaining).collect();
    // No type in the file: the most important of the actions (a site added
    // to a folder not recognised makes it a web project).
    if kind.is_none() {
        if let Some(k) = project.actions.iter().map(|a| a.kind).max() {
            project.kind = k;
        }
    }
}

/// A folder inside the project: relative, no `..`, existing, not through a
/// link that leads outside. Returns it with `/` separators (`.` = root).
pub fn inside(project_dir: &Path, folder: &str) -> Result<(PathBuf, String)> {
    let rel = Path::new(folder.trim());
    if rel.is_absolute()
        || rel
            .components()
            .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
    {
        bail!("folder outside the project: {folder}");
    }
    // Checked as text and without following links (D-77): canonicalize
    // would follow a junction, possibly to a network share.
    let full =
        crate::safe_path::lexical(&project_dir.join(rel)).with_context(|| folder.to_owned())?;
    if !crate::safe_path::is_dir(project_dir, &full) {
        bail!("folder outside the project: {folder}");
    }
    let shown = rel
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/");
    Ok((full, if shown.is_empty() { ".".into() } else { shown }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SiteKind {
    Static,
    Php,
    None,
}

/// "Servi come sito web": writes (or removes, `None`) the action `site` in
/// the project's panel.json; the rest of the file stays as it is.
pub fn set_site(project_dir: &Path, folder: &str, kind: SiteKind, lang: &str) -> Result<()> {
    let path = file_of(project_dir).unwrap_or_else(|| project_dir.join(OVERRIDE_FILES[0]));
    let mut o = if crate::safe_path::is_file(project_dir, &path) {
        match read_object(project_dir, &path) {
            Some(o) => o,
            None => bail!(
                "{}",
                tr(
                    lang,
                    "{0} non è un JSON valido: correggilo prima.",
                    &[&path.file_name().unwrap_or_default().to_string_lossy()]
                )
            ),
        }
    } else {
        Map::new()
    };
    let list_key = if !o.get("actions").is_some_and(Value::is_array)
        && o.get("azioni").is_some_and(Value::is_array)
    {
        "azioni"
    } else {
        "actions"
    };
    let default_key = if o.get("default").is_none() && o.get("predefinita").is_some() {
        "predefinita"
    } else {
        "default"
    };
    let mut actions: Vec<Value> = o
        .get(list_key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|a| a.get("id").and_then(Value::as_str) != Some(SITE_ACTION))
        .collect();
    if kind == SiteKind::None {
        if o.get(default_key).and_then(Value::as_str) == Some(SITE_ACTION) {
            o.remove(default_key);
        }
    } else {
        let (_, rel) = inside(project_dir, folder)?;
        let shown = if rel == "." {
            tr(lang, "cartella del progetto", &[])
        } else {
            rel.clone()
        };
        let label = tr(lang, "sito web ({0})", &[&shown]);
        let action = if kind == SiteKind::Static {
            serde_json::json!({ "id": SITE_ACTION, "label": label, "static": rel })
        } else {
            serde_json::json!({ "id": SITE_ACTION, "label": label, "type": "web",
                "command": format!("php -S 127.0.0.1:{{port}} -t {}", quote(&rel)) })
        };
        actions.insert(0, action);
        o.insert(default_key.into(), Value::String(SITE_ACTION.into()));
    }
    if actions.is_empty() {
        o.remove(list_key);
    } else {
        o.insert(list_key.into(), Value::Array(actions));
    }
    if o.is_empty() {
        // A file written only for the site: removed.
        match fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
            _ => {}
        }
    } else {
        let json = serde_json::to_string_pretty(&Value::Object(o))? + "\n";
        write_atomic(&path, json.as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detect::detect;
    use crate::detect::tests::{ids, project};

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
            "php -S 127.0.0.1:{port} -t www"
        );
        set_site(&tmp.0, "", SiteKind::None, "it").unwrap();
        let left: Value =
            serde_json::from_str(&fs::read_to_string(tmp.0.join("panel.json")).unwrap()).unwrap();
        assert_eq!(left, serde_json::json!({"name":"X"}));

        let only = project("site-only", &[("index.html", "")]);
        set_site(&only.0, ".", SiteKind::Static, "it").unwrap();
        set_site(&only.0, ".", SiteKind::None, "it").unwrap();
        assert!(!only.0.join("panel.json").exists());
    }
}
