//! The components of the kit and the user's choice.
//!
//! The catalog (`catalog/components.json`, embedded) is the only list of
//! components (docs/app/CATALOG.md). The choice is `data\app\components.txt`,
//! in the format of nabre/atollo `_components.ps1:31-80`: three comment lines,
//! then `id=1` or `id=0` per component, UTF-8 without BOM, CRLF. Versions are
//! `data\app\versions\<id>.txt`, plain text.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::kit::{write_atomic, Kit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Group {
    Required,
    Agents,
    Languages,
    Optional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub id: String,
    pub name: String,
    pub group: Group,
    /// Chosen when `components.txt` does not mention it.
    pub def: bool,
    /// Size on disk, about.
    pub mb: u32,
    #[serde(default)]
    pub needs: Vec<String>,
    /// Italian; translated by the catalogs.
    pub desc: String,
    pub home: String,
    pub key: String,
    /// The folder of `data\apps\<platform>` the component owns, if any.
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default)]
    pub x64_only: bool,
}

#[derive(Debug, Deserialize)]
struct File {
    components: Vec<Component>,
}

pub struct Catalog {
    components: Vec<Component>,
}

impl Catalog {
    pub fn get() -> &'static Catalog {
        static CATALOG: OnceLock<Catalog> = OnceLock::new();
        CATALOG.get_or_init(|| {
            // Validated by the tests; a broken file would ship an empty catalog.
            let file: File = serde_json::from_str(include_str!("../catalog/components.json"))
                .unwrap_or(File {
                    components: Vec::new(),
                });
            Catalog {
                components: file.components,
            }
        })
    }

    pub fn all(&self) -> &[Component] {
        &self.components
    }

    pub fn find(&self, id: &str) -> Option<&Component> {
        self.components.iter().find(|c| c.id == id)
    }
}

impl Component {
    /// Where the key lives: `data\apps\<platform>\…`, or the kit root with `@`.
    pub fn key_path(&self, kit: &Kit) -> PathBuf {
        match self.key.strip_prefix('@') {
            Some(rel) => kit.root().join(rel.replace('/', "\\")),
            None => kit.apps().join(self.key.replace('/', "\\")),
        }
    }

    /// Installed: the key is a file, or a non-empty folder.
    pub fn is_installed(&self, kit: &Kit) -> bool {
        let path = self.key_path(kit);
        match fs::metadata(&path) {
            Ok(m) if m.is_file() => true,
            Ok(m) if m.is_dir() => fs::read_dir(&path).is_ok_and(|mut d| d.next().is_some()),
            _ => false,
        }
    }

    pub fn folder_path(&self, kit: &Kit) -> Option<PathBuf> {
        self.folder.as_ref().map(|f| kit.apps().join(f))
    }

    fn version_file(kit: &Kit, id: &str) -> PathBuf {
        kit.app_dir().join("versions").join(format!("{id}.txt"))
    }

    pub fn installed_version(&self, kit: &Kit) -> Option<String> {
        fs::read_to_string(Self::version_file(kit, &self.id))
            .ok()
            .map(|v| v.trim().to_owned())
            .filter(|v| !v.is_empty())
    }

    pub fn save_version(&self, kit: &Kit, version: &str) -> Result<()> {
        let path = Self::version_file(kit, &self.id);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        write_atomic(&path, version.as_bytes()).with_context(|| path.display().to_string())
    }

    pub fn forget_version(&self, kit: &Kit) {
        let _ = fs::remove_file(Self::version_file(kit, &self.id));
    }
}

/// Which components the user wants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    chosen: BTreeMap<String, bool>,
}

impl Choice {
    fn path(kit: &Kit) -> PathBuf {
        kit.app_dir().join("components.txt")
    }

    /// The defaults of the catalog.
    pub fn defaults(catalog: &Catalog) -> Choice {
        let chosen = catalog
            .all()
            .iter()
            .map(|c| (c.id.clone(), c.def))
            .collect();
        let mut choice = Choice { chosen };
        choice.repair(catalog);
        choice
    }

    /// Reads `components.txt`; ids it does not mention take the default.
    /// Returns `None` when the file does not exist (first start).
    pub fn read(kit: &Kit, catalog: &Catalog) -> Option<Choice> {
        let text = fs::read_to_string(Self::path(kit)).ok()?;
        Some(Self::parse(&text, catalog))
    }

    pub fn parse(text: &str, catalog: &Catalog) -> Choice {
        let mut choice = Self::defaults(catalog);
        for line in text.lines() {
            let line = line.trim().trim_start_matches('\u{feff}');
            let Some((id, value)) = line.split_once('=') else {
                continue;
            };
            let (id, value) = (id.trim(), value.trim());
            if catalog.find(id).is_some() && (value == "0" || value == "1") {
                choice.chosen.insert(id.to_owned(), value == "1");
            }
        }
        choice.repair(catalog);
        choice
    }

    /// Required components are always on; a component whose dependency is
    /// off is off too (`Repair-ComponentChoice`).
    pub fn repair(&mut self, catalog: &Catalog) {
        for c in catalog.all() {
            if c.group == Group::Required {
                self.chosen.insert(c.id.clone(), true);
            }
        }
        loop {
            let mut changed = false;
            for c in catalog.all() {
                if self.is_chosen(&c.id) && c.needs.iter().any(|n| !self.is_chosen(n)) {
                    self.chosen.insert(c.id.clone(), false);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    pub fn is_chosen(&self, id: &str) -> bool {
        self.chosen.get(id).copied().unwrap_or(false)
    }

    #[allow(dead_code, reason = "first caller: Components page, phase 4")]
    /// Turns a component on or off, following dependencies both ways, as the
    /// "Languages and tools" page of nabre/atollo does.
    pub fn set(&mut self, catalog: &Catalog, id: &str, on: bool) {
        let Some(c) = catalog.find(id) else { return };
        if c.group == Group::Required {
            return;
        }
        self.chosen.insert(id.to_owned(), on);
        if on {
            for need in &c.needs {
                self.set(catalog, need, true);
            }
        } else {
            for other in catalog
                .all()
                .iter()
                .filter(|o| o.needs.iter().any(|n| n == id))
            {
                self.set(catalog, &other.id, false);
            }
        }
    }

    pub fn render(&self, catalog: &Catalog) -> String {
        let mut out = String::from(
            "# Atollo: components chosen on the \"Linguaggi e strumenti\" page.\r\n\
             # 1 = install, 0 = do not install (removed at the next update).\r\n\
             # Required components are always installed.\r\n",
        );
        for c in catalog.all() {
            out.push_str(&format!("{}={}\r\n", c.id, u8::from(self.is_chosen(&c.id))));
        }
        out
    }

    pub fn write(&self, kit: &Kit, catalog: &Catalog) -> Result<()> {
        let path = Self::path(kit);
        write_atomic(&path, self.render(catalog).as_bytes())
            .with_context(|| path.display().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};
    use std::collections::HashSet;

    #[test]
    fn catalog_is_valid() {
        let catalog = Catalog::get();
        assert!(catalog.all().len() >= 30, "catalog did not load");
        let mut ids = HashSet::new();
        for c in catalog.all() {
            assert!(ids.insert(c.id.as_str()), "duplicate id {}", c.id);
            assert!(
                !c.key.is_empty() && !c.desc.is_empty() && c.home.starts_with("https://"),
                "{}",
                c.id
            );
            if let Some(folder) = &c.folder {
                assert!(
                    c.key == *folder || c.key.starts_with(&format!("{folder}/")),
                    "{}: key outside folder",
                    c.id
                );
            }
        }
        for c in catalog.all() {
            for n in &c.needs {
                assert!(ids.contains(n.as_str()), "{} needs unknown {n}", c.id);
            }
        }
        for id in [
            "git",
            "node",
            "uv",
            "gh",
            "pwsh",
            "terminal",
            "vscodium-server",
        ] {
            assert_eq!(
                catalog.find(id).map(|c| c.group),
                Some(Group::Required),
                "{id}"
            );
        }
    }

    #[test]
    fn choice_file_compatible_with_atollo() {
        let catalog = Catalog::get();
        // As written by nabre/atollo Save-ComponentChoice (plus a BOM and LF).
        let text =
            "\u{feff}# comment\napache=1\nphp=0\nrust=1\ngit=0\nunknown=1\nbad line\nkotlin = 1\n";
        let choice = Choice::parse(text, catalog);
        assert!(!choice.is_chosen("php"));
        assert!(!choice.is_chosen("apache"), "apache needs php");
        assert!(choice.is_chosen("rust"));
        assert!(choice.is_chosen("git"), "required stays on");
        assert!(
            !choice.is_chosen("kotlin"),
            "kotlin needs java, which is off by default"
        );
        assert!(choice.is_chosen("python"), "not mentioned: default");

        let rendered = choice.render(catalog);
        assert!(rendered.ends_with("\r\n") && !rendered.starts_with('\u{feff}'));
        assert_eq!(Choice::parse(&rendered, catalog), choice);
    }

    #[test]
    fn dependencies_follow_both_ways() {
        let catalog = Catalog::get();
        let mut choice = Choice::defaults(catalog);
        choice.set(catalog, "kotlin", true);
        assert!(choice.is_chosen("java"));
        choice.set(catalog, "java", false);
        assert!(!choice.is_chosen("kotlin"));
        choice.set(catalog, "git", false);
        assert!(choice.is_chosen("git"), "required cannot be turned off");
    }

    #[test]
    fn installed_and_versions() {
        let tmp = TempDir::new("catalog");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let catalog = Catalog::get();
        let pwsh = catalog.find("pwsh").unwrap();
        assert!(!pwsh.is_installed(&kit));
        let exe = pwsh.key_path(&kit);
        fs::create_dir_all(exe.parent().unwrap()).unwrap();
        fs::write(&exe, b"").unwrap();
        assert!(pwsh.is_installed(&kit));

        let server = catalog.find("vscodium-server").unwrap();
        fs::create_dir_all(server.key_path(&kit)).unwrap();
        assert!(!server.is_installed(&kit), "empty folder is not installed");
        fs::write(server.key_path(&kit).join("x"), b"").unwrap();
        assert!(server.is_installed(&kit));

        let pnpm = catalog.find("pnpm").unwrap();
        assert!(pnpm
            .key_path(&kit)
            .starts_with(kit.data().join("npm-global")));

        pwsh.save_version(&kit, "7.6.7").unwrap();
        assert_eq!(pwsh.installed_version(&kit).as_deref(), Some("7.6.7"));
        pwsh.forget_version(&kit);
        assert_eq!(pwsh.installed_version(&kit), None);

        assert!(Choice::read(&kit, catalog).is_none());
        Choice::defaults(catalog).write(&kit, catalog).unwrap();
        assert_eq!(Choice::read(&kit, catalog), Some(Choice::defaults(catalog)));
    }
}
