//! A project's editor settings in its `.vscode\settings.json` (DECISIONS.md
//! D-78, user choice: in the project, shared with Git; VSCodium reads it by
//! itself). The page changes four common ones: format on save, indentation
//! (spaces or tabs, and how many) and the folders hidden in the editor;
//! every other key stays. A file with comments (JSONC) is left alone:
//! writing it again would drop them. Never through a link (D-77).

use std::path::Path;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::fsx::write_atomic;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSettings {
    /// None: not set in the project (the editor's own default).
    pub format_on_save: Option<bool>,
    pub tab_size: Option<u8>,
    pub insert_spaces: Option<bool>,
    /// `files.exclude` patterns set to true, e.g. `**/node_modules`.
    #[serde(default)]
    pub hidden: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorSettingsView {
    pub settings: EditorSettings,
    /// The file exists.
    pub exists: bool,
    /// It has comments or is not plain JSON: shown, not written.
    pub read_only: bool,
}

fn path(project_dir: &Path) -> std::path::PathBuf {
    project_dir.join(".vscode").join("settings.json")
}

fn read(project_dir: &Path) -> Option<String> {
    crate::os::safe_path::read_text(project_dir, &path(project_dir))
}

pub fn get(project_dir: &Path) -> EditorSettingsView {
    let Some(text) = read(project_dir) else {
        return EditorSettingsView {
            settings: EditorSettings::default(),
            exists: false,
            read_only: false,
        };
    };
    let Ok(Value::Object(o)) = serde_json::from_str::<Value>(&text) else {
        return EditorSettingsView {
            settings: EditorSettings::default(),
            exists: true,
            read_only: true,
        };
    };
    let hidden = o
        .get("files.exclude")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .filter(|(_, v)| v.as_bool() == Some(true))
                .map(|(k, _)| k.clone())
                .collect()
        })
        .unwrap_or_default();
    EditorSettingsView {
        settings: EditorSettings {
            format_on_save: o.get("editor.formatOnSave").and_then(Value::as_bool),
            tab_size: o
                .get("editor.tabSize")
                .and_then(Value::as_u64)
                .and_then(|n| u8::try_from(n).ok()),
            insert_spaces: o.get("editor.insertSpaces").and_then(Value::as_bool),
            hidden,
        },
        exists: true,
        read_only: false,
    }
}

fn set_or_remove(o: &mut Map<String, Value>, key: &str, value: Option<Value>) {
    match value {
        Some(v) => {
            o.insert(key.to_owned(), v);
        }
        None => {
            o.remove(key);
        }
    }
}

pub fn set(project_dir: &Path, s: &EditorSettings) -> Result<()> {
    let file = path(project_dir);
    let folder = project_dir.join(".vscode");
    if crate::os::safe_path::is_link(&folder) || crate::os::safe_path::is_link(&file) {
        bail!(".vscode is a link: left alone");
    }
    if s.tab_size.is_some_and(|n| n == 0 || n > 16) {
        bail!("indentation: 1 to 16");
    }
    if s.hidden
        .iter()
        .any(|h| h.trim().is_empty() || h.contains(['\n', '\r']))
    {
        bail!("hidden folders: one pattern per line");
    }
    let mut o = match read(project_dir) {
        Some(text) => match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(o)) => o,
            _ => bail!(".vscode\\settings.json has comments or is not plain JSON: left alone"),
        },
        None => Map::new(),
    };
    set_or_remove(
        &mut o,
        "editor.formatOnSave",
        s.format_on_save.map(Value::Bool),
    );
    set_or_remove(&mut o, "editor.tabSize", s.tab_size.map(Value::from));
    set_or_remove(
        &mut o,
        "editor.insertSpaces",
        s.insert_spaces.map(Value::Bool),
    );
    // Patterns set to false by hand stay; ours are the true ones.
    let mut exclude: Map<String, Value> = o
        .get("files.exclude")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, v)| v.as_bool() != Some(true))
        .collect();
    for h in &s.hidden {
        exclude.insert(h.trim().to_owned(), Value::Bool(true));
    }
    set_or_remove(
        &mut o,
        "files.exclude",
        (!exclude.is_empty()).then_some(Value::Object(exclude)),
    );
    if o.is_empty() && !file.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(&folder)?;
    let json = serde_json::to_string_pretty(&Value::Object(o))? + "\n";
    write_atomic(&file, json.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::tests::TempDir;
    use std::fs;

    #[test]
    fn settings_round_trip_keeping_other_keys() {
        let tmp = TempDir::new("vscode-settings");
        let d = tmp.0.join("p");
        fs::create_dir_all(d.join(".vscode")).unwrap();
        assert!(!get(&d).exists);
        fs::write(
            d.join(".vscode").join("settings.json"),
            r#"{"python.analysis.typeCheckingMode":"strict","files.exclude":{"**/.git":false,"**/dist":true}}"#,
        )
        .unwrap();
        let v = get(&d);
        assert_eq!(v.settings.hidden, ["**/dist"]);
        let s = EditorSettings {
            format_on_save: Some(true),
            tab_size: Some(4),
            insert_spaces: Some(true),
            hidden: vec!["**/vendor".into()],
        };
        set(&d, &s).unwrap();
        let text = fs::read_to_string(d.join(".vscode").join("settings.json")).unwrap();
        let o: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(o["python.analysis.typeCheckingMode"], "strict");
        assert_eq!(o["files.exclude"]["**/.git"], false, "false patterns stay");
        assert_eq!(o["files.exclude"]["**/vendor"], true);
        assert!(
            o["files.exclude"].get("**/dist").is_none(),
            "unchecked: removed"
        );
        assert_eq!(get(&d).settings, s);
        assert!(set(
            &d,
            &EditorSettings {
                tab_size: Some(0),
                ..s.clone()
            }
        )
        .is_err());
        // Comments: shown as read only, never written.
        fs::write(d.join(".vscode").join("settings.json"), "// mine\n{}").unwrap();
        assert!(get(&d).read_only);
        assert!(set(&d, &s).is_err());
        assert_eq!(
            fs::read_to_string(d.join(".vscode").join("settings.json")).unwrap(),
            "// mine\n{}"
        );
    }
}
