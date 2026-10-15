//! What a combination of a base and its additions does, in order, and the
//! placeholders (`{name}`, `{package}`) its lines are filled with.

use std::collections::BTreeMap;

use serde::Serialize;

use super::{Action, EnvVar, Kind, Piece, Problem, Variant};

/// What a combination does, in order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    /// Components of the kit to install first.
    pub needs: Vec<String>,
    /// In `projects\`.
    pub create: Vec<String>,
    /// Inside the project.
    pub steps: Vec<String>,
    pub actions: Vec<Action>,
    /// The kit's database server where the project gets its database.
    pub database: Option<String>,
    /// Variables of `.env` (`{name}`, `{package}` not filled in yet).
    pub env: Vec<EnvVar>,
    /// Italian sources of the notes.
    pub notes: Vec<String>,
}

fn add_actions(out: &mut Vec<Action>, more: &[Action]) {
    for a in more {
        match out.iter_mut().find(|o| o.name == a.name) {
            Some(o) => o.command.clone_from(&a.command),
            None => out.push(a.clone()),
        }
    }
}

fn add_needs(out: &mut Vec<String>, more: &[String]) {
    for n in more {
        if !out.contains(n) {
            out.push(n.clone());
        }
    }
}

/// The base with the additions: the creation (the base's, or the one of the
/// addition that replaces it), the base's steps, then those of each
/// addition in the order of the list.
pub fn plan(pieces: &[Piece], base: &str, addons: &[String]) -> Result<Plan, Problem> {
    let b = pieces
        .iter()
        .find(|p| p.id == base && p.kind == Kind::Base)
        .ok_or_else(|| ("Base sconosciuta: {0}", vec![base.to_owned()]))?;
    let mut chosen: Vec<(&Piece, &Variant)> = Vec::new();
    for p in pieces.iter().filter(|p| p.kind == Kind::Addon) {
        if !addons.contains(&p.id) {
            continue;
        }
        let v =
            p.on.get(base)
                .ok_or_else(|| ("{0} non si usa con questa base.", vec![p.name.clone()]))?;
        chosen.push((p, v));
    }
    if let Some(missing) = addons
        .iter()
        .find(|a| chosen.iter().all(|(p, _)| &p.id != *a))
    {
        return Err(("Aggiunta sconosciuta: {0}", vec![missing.clone()]));
    }
    let mut slots: BTreeMap<&str, &str> = BTreeMap::new();
    for (p, _) in &chosen {
        if let Some(slot) = p.slot.as_deref() {
            if let Some(other) = slots.insert(slot, &p.name) {
                return Err((
                    "{0} e {1} non vanno insieme.",
                    vec![other.to_owned(), p.name.clone()],
                ));
            }
        }
    }
    let replacing: Vec<&(&Piece, &Variant)> = chosen
        .iter()
        .filter(|(_, v)| !v.create.is_empty())
        .collect();
    if replacing.len() > 1 {
        return Err((
            "{0} e {1} non vanno insieme.",
            vec![replacing[0].0.name.clone(), replacing[1].0.name.clone()],
        ));
    }
    let mut out = Plan {
        needs: b.needs.clone(),
        create: replacing
            .first()
            .map_or_else(|| b.create.clone(), |(_, v)| v.create.clone()),
        steps: b.steps.clone(),
        actions: b.actions.clone(),
        database: None,
        env: b.env.clone(),
        notes: b.note.iter().cloned().collect(),
    };
    for (p, v) in &chosen {
        if let Some(db) = &v.database {
            if let Some(other) = &out.database {
                if other != db {
                    return Err((
                        "{0} e {1} non vanno insieme.",
                        vec![other.clone(), p.name.clone()],
                    ));
                }
            }
            out.database = Some(db.clone());
            add_needs(&mut out.needs, std::slice::from_ref(db));
        }
        add_needs(&mut out.needs, &v.needs);
        for e in &v.env {
            match out.env.iter_mut().find(|o| o.key == e.key) {
                Some(o) => o.value.clone_from(&e.value),
                None => out.env.push(e.clone()),
            }
        }
        out.steps.extend(v.steps.iter().cloned());
        add_actions(&mut out.actions, &v.actions);
        out.notes.extend(v.note.iter().cloned());
    }
    Ok(out)
}

/// `{name}` and `{package}` of a project: the package id is
/// `com.example.` with the name's letters and digits in lower case.
pub fn package_of(name: &str) -> String {
    let mut id: String = name
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if id.is_empty() || id.starts_with(|c: char| c.is_ascii_digit()) {
        id.insert(0, 'a');
    }
    format!("com.example.{id}")
}

/// A line with `{package}` and `{name}` filled in; the name goes in
/// PowerShell quotes, so spaces and `$` stay as they are.
pub fn fill(line: &str, name: &str) -> String {
    let quoted = format!("'{}'", name.replace('\'', "''"));
    line.replace("{package}", &package_of(name))
        .replace("{name}", &quoted)
}

/// A value of `.env` with `{name}` and `{package}` filled in, in double
/// quotes when it has spaces, quotes, `#` or `$`.
pub fn env_value(value: &str, name: &str) -> String {
    let v = value
        .replace("{package}", &package_of(name))
        .replace("{name}", name);
    if v.contains([' ', '"', '#', '\'', '$']) {
        format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projects::frameworks::{check_env, official};

    fn ids(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_owned()).collect()
    }
    #[test]
    fn combinations() {
        let p = official();
        let laravel_react = plan(p, "laravel", &ids(&["react", "tailwind", "nativephp"])).unwrap();
        assert!(laravel_react.create[0].contains("laravel/react-starter-kit"));
        assert_eq!(laravel_react.steps[0], "npm install");
        assert!(laravel_react
            .steps
            .iter()
            .any(|s| s.contains("nativephp/desktop")));
        assert!(laravel_react
            .actions
            .iter()
            .any(|a| a.name == "App desktop"));
        assert!(laravel_react.notes.len() >= 2);

        let electron_vue = plan(p, "electron", &ids(&["vue"])).unwrap();
        assert!(electron_vue.create[0].contains("@quick-start/electron"));
        let start = electron_vue
            .actions
            .iter()
            .find(|a| a.name == "Avvia")
            .unwrap();
        assert_eq!(
            start.command, "npm run dev",
            "the addition replaces the action"
        );
        assert_eq!(electron_vue.actions.len(), 2);

        let tauri = plan(p, "tauri", &[]).unwrap();
        assert_eq!(tauri.needs, ["rust", "node"]);

        let mysql = plan(p, "laravel", &ids(&["mysql", "bootstrap"])).unwrap();
        assert_eq!(mysql.database.as_deref(), Some("mariadb"));
        assert!(mysql.needs.contains(&"mariadb".to_owned()));
        assert!(
            plan(p, "laravel", &ids(&["mysql", "mongodb"])).is_err(),
            "one database"
        );
        for base in ["laravel", "symfony", "vite", "nextjs", "electron", "tauri"] {
            for css in ["tailwind", "bootstrap", "bulma"] {
                plan(p, base, &ids(&[css])).unwrap_or_else(|_| panic!("{css} with {base}"));
            }
        }
        assert!(plan(p, "symfony", &ids(&["vite-bundle"]))
            .unwrap()
            .needs
            .contains(&"node".to_owned()));

        let web_app = plan(p, "vite", &ids(&["react", "tailwind", "capacitor"])).unwrap();
        assert!(web_app.needs.contains(&"android".to_owned()));
        assert!(web_app.steps.iter().any(|s| s.contains("cap add android")));

        let tauri_android = plan(p, "tauri", &ids(&["react", "capacitor"])).unwrap();
        assert!(tauri_android
            .steps
            .iter()
            .any(|s| s.contains("tauri android init")));
        assert!(tauri_android.needs.contains(&"android".to_owned()));

        let mobile = plan(p, "laravel", &ids(&["nativephp-mobile"])).unwrap();
        assert_eq!(mobile.needs, ["php", "node", "java", "android"]);

        assert_eq!(
            plan(p, "laravel", &ids(&["react", "vue"])).unwrap_err().0,
            "{0} e {1} non vanno insieme."
        );
        assert_eq!(
            plan(p, "symfony", &ids(&["react"])).unwrap_err().0,
            "{0} non si usa con questa base."
        );
        assert!(plan(p, "react", &[]).is_err(), "an addition is not a base");
        assert!(plan(p, "vite", &ids(&["nothing"])).is_err());
    }

    #[test]
    fn env_values() {
        assert_eq!(env_value("{name}", "shop"), "shop");
        assert_eq!(env_value("{name}", "my shop"), "\"my shop\"");
        assert_eq!(env_value("{package}", "my shop"), "com.example.myshop");
        let p = official();
        let plan = plan(p, "laravel", &ids(&["nativephp"])).unwrap();
        let keys: Vec<&str> = plan.env.iter().map(|e| e.key.as_str()).collect();
        assert_eq!(keys, ["APP_NAME", "NATIVEPHP_APP_ID"]);
        let bad = vec![EnvVar {
            key: "1A".into(),
            value: String::new(),
        }];
        assert!(check_env(&bad).is_err());
    }

    #[test]
    fn filled_lines() {
        assert_eq!(
            fill(
                "npm create vite@9 {name} -- --identifier {package}",
                "it's mine"
            ),
            "npm create vite@9 'it''s mine' -- --identifier com.example.itsmine"
        );
    }

    #[test]
    fn package_ids() {
        assert_eq!(package_of("My App 2"), "com.example.myapp2");
        assert_eq!(package_of("2048"), "com.example.a2048");
        assert_eq!(package_of("è"), "com.example.a");
    }
}
