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
//! server, always a web action; `apache` (instead of `command`): that folder
//! served by an Apache of the project's own, always a web action; `nginx`
//! and `caddy` the same with Nginx or Caddy. The old `pannello.json` (read only when
//! `panel.json` is missing) uses the Italian keys `nome`, `tipo`,
//! `piattaforma`, `sostituisci`, `predefinita`, `azioni`, `etichetta`,
//! `comando`, `cartella`, `statico` and `{porta}`; both sets are accepted in
//! either file. The file holds no path of the kit, which may move.

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde_json::{Map, Value};

use super::{php_server, tr, Action, Kind, Project, OVERRIDE_FILES};
use crate::fsx::write_atomic;

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
        .find(|p| crate::os::safe_path::is_file(dir, p))
}

fn read_object(dir: &Path, path: &Path) -> Option<Map<String, Value>> {
    let text = crate::os::safe_path::read_text(dir, path)?;
    match serde_json::from_str(&text) {
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
            let apache = a.get("apache").and_then(Value::as_str).map(str::to_owned);
            let nginx = a.get("nginx").and_then(Value::as_str).map(str::to_owned);
            let caddy = a.get("caddy").and_then(Value::as_str).map(str::to_owned);
            if command.is_none()
                && served.is_none()
                && apache.is_none()
                && nginx.is_none()
                && caddy.is_none()
            {
                return None;
            }
            let a_kind = text(pick(a, "type", "tipo")).and_then(|t| Kind::parse(&t));
            let label = text(pick(a, "label", "etichetta")).unwrap_or_else(|| id.clone());
            let mut action = match &command {
                Some(c) => Action::new(&id, &label, c, a_kind.unwrap_or(Kind::Command)),
                None => match (&apache, &nginx, &caddy) {
                    (Some(folder), _, _) => Action::apache(&id, &label, folder),
                    (None, Some(folder), _) => Action::nginx(&id, &label, folder),
                    (None, None, Some(folder)) => Action::caddy(&id, &label, folder),
                    (None, None, None) => {
                        Action::serve(&id, &label, served.as_deref().unwrap_or("."))
                    }
                },
            };
            action.folder = text(pick(a, "folder", "cartella"));
            action.url = text(a.get("url")).map(|u| ports(&u));
            action.server = text(a.get("server"));
            if let Some(Value::Array(domains)) = a.get("domains") {
                action.domains = domains
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|d| valid_domain(d))
                    .map(str::to_owned)
                    .collect();
            }
            if let Some(Value::Object(aliases)) = a.get("aliases") {
                for (k, v) in aliases {
                    if let Some(v) = v.as_str() {
                        if valid_alias_url(k) && valid_alias_target(v) {
                            action.aliases.insert(k.clone(), v.trim().to_owned());
                        }
                    }
                }
            }
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

/// Validates an RFC 6761 `.localhost` domain alias (`mio-sito.localhost`).
pub fn valid_domain(d: &str) -> bool {
    crate::projects::alias::valid_domain(d)
}

/// Validates a URL path prefix for an alias (`/media`, `/uploads`).
pub fn valid_alias_url(u: &str) -> bool {
    crate::projects::alias::valid_url(u)
}

/// Validates a portable relative path alias target (`storage/uploads` or
/// `@other/public`): plain names only, no absolute path, drive or `..`.
pub fn valid_alias_target(target: &str) -> bool {
    crate::projects::alias::Target::parse(target).is_some()
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
        crate::os::safe_path::lexical(&project_dir.join(rel)).with_context(|| folder.to_owned())?;
    if !crate::os::safe_path::is_dir(project_dir, &full) {
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
    /// An Apache of the project's own (`projects/apache.rs` or pooled server).
    Apache,
    /// An Nginx of the project's own or pooled server.
    Nginx,
    /// A Caddy of the project's own or pooled server (WP-3.3).
    Caddy,
    None,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteOptions {
    #[serde(default)]
    pub server: Option<String>,
    #[serde(default)]
    pub domains: Vec<String>,
    #[serde(default)]
    pub aliases: std::collections::BTreeMap<String, String>,
}

/// The project's panel.json changed by `edit` (its actions and its default
/// action); every other key, and the Italian keys of old files, stay as
/// they are. An object left empty removes the file.
fn edit_panel(
    project_dir: &Path,
    lang: &str,
    edit: impl FnOnce(&mut Vec<Value>, &mut Option<String>) -> Result<()>,
) -> Result<()> {
    let path = file_of(project_dir).unwrap_or_else(|| project_dir.join(OVERRIDE_FILES[0]));
    if crate::os::safe_path::is_link(&path) {
        bail!("{}: link, left alone", path.display());
    }
    let mut o = if crate::os::safe_path::is_file(project_dir, &path) {
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
        .unwrap_or_default();
    let mut default = o
        .get(default_key)
        .and_then(Value::as_str)
        .map(str::to_owned);
    edit(&mut actions, &mut default)?;
    if actions.is_empty() {
        o.remove(list_key);
    } else {
        o.insert(list_key.into(), Value::Array(actions));
    }
    match default {
        Some(d) => {
            o.insert(default_key.into(), Value::String(d));
        }
        None => {
            o.remove(default_key);
        }
    }
    if o.is_empty() {
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

/// "Servi come sito web": writes (or removes, `None`) the action `site` in
/// the project's panel.json; the rest of the file stays as it is.
#[cfg(test)]
pub fn set_site(project_dir: &Path, folder: &str, kind: SiteKind, lang: &str) -> Result<()> {
    set_site_with_options(project_dir, folder, kind, &SiteOptions::default(), lang)
}

/// "Servi come sito web" with optional server profile, `.localhost` domains and path aliases.
pub fn set_site_with_options(
    project_dir: &Path,
    folder: &str,
    kind: SiteKind,
    options: &SiteOptions,
    lang: &str,
) -> Result<()> {
    let site = if kind == SiteKind::None {
        None
    } else {
        let (_, rel) = inside(project_dir, folder)?;
        let shown = if rel == "." {
            tr(lang, "cartella del progetto", &[])
        } else {
            rel.clone()
        };
        let label = tr(lang, "sito web ({0})", &[&shown]);
        let mut obj = match kind {
            SiteKind::Static => {
                serde_json::json!({ "id": SITE_ACTION, "label": label, "static": rel })
            }
            SiteKind::Apache => serde_json::json!({ "id": SITE_ACTION,
                "label": tr(lang, "sito web Apache ({0})", &[&shown]), "apache": rel }),
            SiteKind::Nginx => serde_json::json!({ "id": SITE_ACTION,
                "label": tr(lang, "sito web Nginx ({0})", &[&shown]), "nginx": rel }),
            SiteKind::Caddy => serde_json::json!({ "id": SITE_ACTION,
                "label": tr(lang, "sito web Caddy ({0})", &[&shown]), "caddy": rel }),
            SiteKind::Php | SiteKind::None => {
                serde_json::json!({ "id": SITE_ACTION, "label": label, "type": "web",
                    "command": php_server(project_dir, &rel) })
            }
        };
        if matches!(kind, SiteKind::Apache | SiteKind::Nginx | SiteKind::Caddy) {
            if let Some(s) = options
                .server
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty() && *s != "auto")
            {
                obj["server"] = Value::String(s.to_owned());
            }
            let mut domains = Vec::new();
            for d in &options.domains {
                let d = d.trim().to_ascii_lowercase();
                if d.is_empty() {
                    continue;
                }
                if !valid_domain(&d) {
                    bail!(
                        "{}",
                        tr(
                            lang,
                            "Dominio non valido: usa un nome che termina con .localhost (es. mio-sito.localhost).",
                            &[]
                        )
                    );
                }
                if !domains.contains(&d) {
                    domains.push(d);
                }
            }
            if !domains.is_empty() {
                obj["domains"] = serde_json::to_value(domains)?;
            }
            let mut aliases = std::collections::BTreeMap::new();
            for (url_path, target) in &options.aliases {
                let u = url_path.trim();
                let t = target.trim().replace('\\', "/");
                if u.is_empty() && t.is_empty() {
                    continue;
                }
                if !valid_alias_url(u) || !valid_alias_target(&t) {
                    bail!(
                        "{}",
                        tr(
                            lang,
                            "Alias non valido: usa un percorso /url e una cartella relativa (niente percorsi assoluti o ..).",
                            &[]
                        )
                    );
                }
                aliases.insert(u.to_owned(), t);
            }
            if !aliases.is_empty() {
                obj["aliases"] = serde_json::to_value(aliases)?;
            }
        }
        Some(obj)
    };
    edit_panel(project_dir, lang, |actions, default| {
        actions.retain(|a| a.get("id").and_then(Value::as_str) != Some(SITE_ACTION));
        match site {
            Some(action) => {
                actions.insert(0, action);
                *default = Some(SITE_ACTION.into());
            }
            None => {
                if default.as_deref() == Some(SITE_ACTION) {
                    *default = None;
                }
            }
        }
        Ok(())
    })
}

/// An action as the project page edits it (D-78).
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionEdit {
    pub id: String,
    pub label: String,
    pub command: String,
    /// `web`, `command`, `desktop`.
    pub kind: String,
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub env: std::collections::BTreeMap<String, String>,
}

fn valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id.len() <= 40
        // `:` and `.` too: detected ids look like `npm:dev`, and the page
        // corrects a detected action by saving one with its id.
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "-_:.".contains(c))
}

fn valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Adds or replaces (same id) an action in panel.json: a detected action
/// with the same id is corrected by it.
pub fn save_action(project_dir: &Path, a: &ActionEdit, lang: &str) -> Result<()> {
    let bad = |what: &str| anyhow::anyhow!("{}", tr(lang, "Azione non valida: {0}.", &[what]));
    if !valid_id(&a.id) {
        return Err(bad("id"));
    }
    if a.label.trim().is_empty() || a.label.contains('\n') {
        return Err(bad(&tr(lang, "nome", &[])));
    }
    let command = a.command.trim();
    if command.is_empty() || command.contains(['\n', '\r']) {
        return Err(bad(&tr(lang, "comando", &[])));
    }
    if Kind::parse(&a.kind).is_none() {
        return Err(bad(&tr(lang, "tipo", &[])));
    }
    let folder = match a
        .folder
        .as_deref()
        .map(str::trim)
        .filter(|f| !f.is_empty() && *f != ".")
    {
        Some(f) => Some(inside(project_dir, f)?.1),
        None => None,
    };
    for (k, v) in &a.env {
        if !valid_env_name(k) || v.contains(['\n', '\r']) {
            return Err(bad(&tr(lang, "variabile {0}", &[k])));
        }
    }
    let mut action = serde_json::json!({
        "id": a.id, "label": a.label.trim(), "type": a.kind, "command": command,
    });
    if let Some(f) = folder {
        action["folder"] = Value::String(f);
    }
    if let Some(u) = a.url.as_deref().map(str::trim).filter(|u| !u.is_empty()) {
        action["url"] = Value::String(u.to_owned());
    }
    if !a.env.is_empty() {
        action["env"] = serde_json::to_value(&a.env)?;
    }
    edit_panel(project_dir, lang, |actions, _| {
        match actions
            .iter()
            .position(|x| x.get("id").and_then(Value::as_str) == Some(&a.id))
        {
            Some(i) => actions[i] = action,
            None => actions.push(action),
        }
        Ok(())
    })
}

/// Removes an action from panel.json (a detected one with that id comes
/// back as detected).
pub fn remove_action(project_dir: &Path, id: &str, lang: &str) -> Result<()> {
    edit_panel(project_dir, lang, |actions, default| {
        actions.retain(|a| a.get("id").and_then(Value::as_str) != Some(id));
        if default.as_deref() == Some(id) {
            *default = None;
        }
        Ok(())
    })
}

/// The action started by "Avvia" in the project list and its summary.
pub fn set_default_action(project_dir: &Path, id: &str, lang: &str) -> Result<()> {
    if !valid_id(id) && id != SITE_ACTION {
        bail!("{}", tr(lang, "Azione non valida: {0}.", &["id"]));
    }
    edit_panel(project_dir, lang, |_, default| {
        *default = Some(id.to_owned());
        Ok(())
    })
}

#[cfg(test)]
#[path = "../../../test/rust/detect/overrides.rs"]
mod tests;
