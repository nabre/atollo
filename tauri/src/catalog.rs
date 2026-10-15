//! The components of the kit and the user's choice.
//!
//! The catalog (`catalog/components.json`, embedded) is the only list of
//! components (docs/app/CATALOG.md). The choice is `data\app\components.txt`,
//! in the format of nabre/dev-kit `_components.ps1:31-80`: three comment lines,
//! then `id=1` or `id=0` per component, UTF-8 without BOM, CRLF. Versions are
//! `data\app\versions\<id>.txt`, plain text.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::fsx::write_atomic;
use crate::kit::Kit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Group {
    Required,
    Agents,
    Languages,
    /// Tools tied to an operating system (Windows Terminal): shown in
    /// their own group of "Linguaggi e strumenti", chosen like the others.
    Os,
    /// Database servers and tools (D-73), with their own group too.
    Databases,
    /// Part of Dev-Kit itself (the editor): not in "Linguaggi e strumenti",
    /// always installed, at the version fixed in the catalog for this
    /// release (`version`, `sha256`).
    App,
}

impl Group {
    /// Always installed: never turned off or skipped.
    pub fn always(self) -> bool {
        matches!(self, Group::Required | Group::App)
    }
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
    /// The other parts (Maven, Ninja, Composer...), same rules as `key`:
    /// installed only when the key and all of these exist.
    #[serde(default)]
    pub keys: Vec<String>,
    /// The folder of `data\apps\<platform>` the component owns, if any.
    #[serde(default)]
    pub folder: Option<String>,
    #[serde(default)]
    pub x64_only: bool,
    /// No build runs on ARM64, not even emulated (R): not available there.
    #[serde(default)]
    pub no_arm64: bool,
    /// Other files or folders it owns (relative to `data\apps\<platform>`,
    /// `@` = the kit root), removed with it unless another installed
    /// component lists them too.
    #[serde(default)]
    pub remove: Vec<String>,
    /// Version command for the final check: program (relative to
    /// `data\apps\<platform>`, `@` = the kit root) and arguments.
    #[serde(default)]
    pub check: Vec<String>,
    /// Group `app`: the version this release of Dev-Kit installs, and the
    /// SHA-256 of its download.
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    /// Other minor lines can be kept next to the default folder, and chosen
    /// for the kit or a project (`versions.rs`, D-80).
    #[serde(default)]
    pub versioned: bool,
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

    /// Always installed: the system components (required, and the parts of
    /// the app) and, transitively, what they need (GitHub CLI needs Git, the
    /// editor needs PowerShell 7). Never turned off or skipped.
    pub fn locked(&self) -> BTreeSet<&str> {
        let mut out = BTreeSet::new();
        let mut todo: Vec<&str> = self
            .components
            .iter()
            .filter(|c| c.group.always())
            .map(|c| c.id.as_str())
            .collect();
        while let Some(id) = todo.pop() {
            if let Some(c) = self.find(id) {
                if out.insert(c.id.as_str()) {
                    todo.extend(c.needs.iter().map(String::as_str));
                }
            }
        }
        out
    }

    pub fn is_locked(&self, id: &str) -> bool {
        self.locked().contains(id)
    }

    /// The components in the order of the Tools page: by section, then by
    /// name (case aside). An installation runs in this order too, as the
    /// user sees the list (D-81). The only need at installation time (pnpm
    /// installed with the kit's Node.js) holds: languages come before tools.
    pub fn in_page_order(&self) -> Vec<&Component> {
        let mut out: Vec<&Component> = self.components.iter().collect();
        out.sort_by_key(|c| (c.page_section(), c.name.to_lowercase()));
        out
    }
}

/// A path of the catalog: relative to `data\apps\<platform>`, or to the kit
/// root with `@`. In a versioned component's folder it is that of its
/// default version (`node/node.exe` → `node\22.11\node.exe`, versions.rs).
pub fn kit_path(kit: &Kit, rel: &str) -> PathBuf {
    match rel.strip_prefix('@') {
        Some(r) => kit.root().join(r.replace('/', "\\")),
        None => crate::versions::resolve(kit, rel)
            .unwrap_or_else(|| kit.apps().join(rel.replace('/', "\\"))),
    }
}

/// A file, or a non-empty folder.
fn exists(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(m) if m.is_file() => true,
        Ok(m) if m.is_dir() => fs::read_dir(path).is_ok_and(|mut d| d.next().is_some()),
        _ => false,
    }
}

impl Component {
    /// Its section of the Tools page: required (and the editor), agents,
    /// languages (several versions), tools (one, the latest), databases,
    /// tied to an operating system.
    pub fn page_section(&self) -> u8 {
        match self.group {
            Group::Required | Group::App => 0,
            Group::Agents => 1,
            Group::Languages if self.versioned => 2,
            Group::Languages => 3,
            Group::Databases => 4,
            Group::Os => 5,
        }
    }

    /// Where the key lives: `data\apps\<platform>\…`, or the kit root with `@`.
    #[cfg(test)]
    pub fn key_path(&self, kit: &Kit) -> PathBuf {
        kit_path(kit, &self.key)
    }

    /// The key, then the other parts (`keys`).
    pub fn key_paths(&self, kit: &Kit) -> Vec<PathBuf> {
        std::iter::once(&self.key)
            .chain(&self.keys)
            .map(|k| kit_path(kit, k))
            .collect()
    }

    /// Installed: the key and every other part are a file, or a non-empty
    /// folder.
    pub fn is_installed(&self, kit: &Kit) -> bool {
        self.key_paths(kit).iter().all(|p| exists(p))
    }

    /// Something of it is on disk (a part, or its folder), even if it is not
    /// fully installed: what can be removed.
    pub fn is_present(&self, kit: &Kit) -> bool {
        self.key_paths(kit)
            .iter()
            .chain(&self.folder_path(kit))
            .any(|p| exists(p))
    }

    /// The first part that is missing (the key, then `keys`).
    pub fn missing_part(&self, kit: &Kit) -> Option<PathBuf> {
        self.key_paths(kit).into_iter().find(|p| !exists(p))
    }

    /// Everything removed when the component is turned off, except what
    /// another component still on disk owns too (`tools\vcruntime` of PHP
    /// and Apache goes with the last of the two).
    pub fn owned_paths(&self, kit: &Kit) -> Vec<PathBuf> {
        let shared: Vec<PathBuf> = Catalog::get()
            .all()
            .iter()
            .filter(|o| o.id != self.id && o.is_present(kit))
            .flat_map(|o| o.all_owned(kit))
            .collect();
        self.all_owned(kit)
            .into_iter()
            .filter(|p| !shared.contains(p))
            .collect()
    }

    fn all_owned(&self, kit: &Kit) -> Vec<PathBuf> {
        self.folder_path(kit)
            .into_iter()
            .chain(self.remove.iter().map(|r| kit_path(kit, r)))
            // Saved lines of an older kit, not moved yet (versions::migrate).
            .chain(
                self.versioned
                    .then(|| kit.apps().join("versions").join(&self.id)),
            )
            .collect()
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

    /// A part of the app (fixed `version`) installed at another version: the
    /// release of Dev-Kit changed, so it is installed again.
    pub fn outdated(&self, kit: &Kit) -> bool {
        self.version
            .as_deref()
            .is_some_and(|v| self.installed_version(kit).as_deref() != Some(v))
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

    /// The defaults of the catalog, with what they need (Claude Code is a
    /// default, Git is not: Git comes with it).
    pub fn defaults(catalog: &Catalog) -> Choice {
        let mut choice = Choice {
            chosen: catalog
                .all()
                .iter()
                .map(|c| (c.id.clone(), false))
                .collect(),
        };
        for c in catalog.all().iter().filter(|c| c.def) {
            choice.set(catalog, &c.id, true);
        }
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
            if catalog.is_locked(&c.id) {
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

    /// Turns a component on or off, following dependencies both ways, as the
    /// "Languages and tools" page of nabre/dev-kit does.
    pub fn set(&mut self, catalog: &Catalog, id: &str, on: bool) {
        let Some(c) = catalog.find(id) else { return };
        if catalog.is_locked(&c.id) {
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
            "# Dev-Kit: components chosen on the \"Linguaggi e strumenti\" page.\r\n\
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
        assert!(catalog.all().len() >= 29, "catalog did not load");
        let mut ids = HashSet::new();
        for c in catalog.all() {
            assert!(ids.insert(c.id.as_str()), "duplicate id {}", c.id);
            assert!(
                !c.key.is_empty() && !c.desc.is_empty() && c.home.starts_with("https://"),
                "{}",
                c.id
            );
            assert!(
                c.keys.iter().all(|k| !k.is_empty() && *k != c.key),
                "{}: keys",
                c.id
            );
            assert!(!(c.x64_only && c.no_arm64), "{}: x64Only or noArm64", c.id);
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
        // Always installed: the kit's shell and GitHub CLI only; the rest
        // comes when something needs it (D-59).
        let required: Vec<&str> = catalog
            .all()
            .iter()
            .filter(|c| c.group == Group::Required)
            .map(|c| c.id.as_str())
            .collect();
        assert_eq!(required, ["gh", "pwsh"]);
        for id in ["git", "node", "uv"] {
            let c = catalog.find(id).unwrap();
            assert!(c.group == Group::Languages && !c.def, "{id}");
        }
        assert_eq!(catalog.find("claude").unwrap().needs, ["git"]);
        // Only the parts of the app have a fixed version, and all of them do.
        for c in catalog.all() {
            let app = c.group == Group::App;
            assert_eq!(c.version.is_some(), app, "{}: version", c.id);
            assert_eq!(c.sha256.is_some(), app, "{}: sha256", c.id);
        }
        assert_eq!(
            catalog.find("vscodium-server").map(|c| c.group),
            Some(Group::App)
        );
        // Saved lines are copies of the component's own folder: languages,
        // and the database servers (each version with its data, D-81).
        for c in catalog.all().iter().filter(|c| c.versioned) {
            assert!(
                c.folder.is_some()
                    && (c.group == Group::Languages
                        || crate::services::SERVERS.contains(&c.id.as_str())),
                "{}: versioned",
                c.id
            );
        }
        assert!(catalog.find("node").unwrap().versioned);
    }

    #[test]
    fn choice_file_compatible_with_dev_kit() {
        let catalog = Catalog::get();
        // As written by nabre/dev-kit Save-ComponentChoice (plus a BOM and LF).
        let text =
            "\u{feff}# comment\napache=1\nphp=0\nrust=1\ngh=0\nunknown=1\nbad line\nkotlin = 1\nvscodium=1\n";
        let choice = Choice::parse(text, catalog);
        assert!(!choice.is_chosen("php"));
        assert!(!choice.is_chosen("apache"), "apache needs php");
        assert!(choice.is_chosen("rust"));
        assert!(choice.is_chosen("gh"), "required stays on");
        assert!(
            choice.is_chosen("claude") && choice.is_chosen("git"),
            "a default comes with what it needs"
        );
        assert!(
            !choice.is_chosen("kotlin"),
            "kotlin needs java, which is off by default"
        );
        assert!(choice.is_chosen("python"), "not mentioned: default");

        let rendered = choice.render(catalog);
        assert!(rendered.ends_with("\r\n") && !rendered.starts_with('\u{feff}'));
        assert!(
            !rendered.contains("unknown=") && !rendered.contains("\nvscodium="),
            "ids no longer in the catalog (VSCodium desktop) are dropped"
        );
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
        choice.set(catalog, "gh", false);
        assert!(choice.is_chosen("gh"), "required cannot be turned off");
        // Git is not locked (GitHub CLI works without it, D-81): turned off,
        // what needs it (Claude Code) goes with it.
        choice.set(catalog, "git", false);
        assert!(!choice.is_chosen("git") && !choice.is_chosen("claude"));
        assert!(!catalog.is_locked("git") && catalog.is_locked("pwsh"));
        assert!(!catalog.is_locked("node"));
    }

    #[test]
    fn page_order_is_by_section_then_name() {
        let catalog = Catalog::get();
        let order = catalog.in_page_order();
        assert_eq!(order.len(), catalog.all().len(), "every component once");
        let pos = |id: &str| order.iter().position(|c| c.id == id).unwrap();
        // Sections in the order of the page; inside one, by name.
        for w in order.windows(2) {
            let (a, b) = (w[0], w[1]);
            assert!(a.page_section() <= b.page_section(), "{} / {}", a.id, b.id);
            if a.page_section() == b.page_section() {
                assert!(
                    a.name.to_lowercase() <= b.name.to_lowercase(),
                    "{} / {}",
                    a.name,
                    b.name
                );
            }
        }
        // pnpm is installed with the kit's Node.js: languages come first.
        assert!(pos("node") < pos("pnpm"));
        assert!(pos("python") < pos("uv"), "languages before tools");
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

        let folder: Component = serde_json::from_value(serde_json::json!({
            "id": "f", "name": "F", "group": "languages", "def": false, "mb": 1,
            "desc": "f", "home": "https://f", "key": "f"
        }))
        .unwrap();
        fs::create_dir_all(folder.key_path(&kit)).unwrap();
        assert!(!folder.is_installed(&kit), "empty folder is not installed");
        fs::write(folder.key_path(&kit).join("x"), b"").unwrap();
        assert!(folder.is_installed(&kit));

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

    fn touch(path: &std::path::Path) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"").unwrap();
    }

    #[test]
    fn every_part_must_be_installed() {
        let tmp = TempDir::new("catalog-parts");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let catalog = Catalog::get();
        for (id, parts) in [
            ("java", &["maven/bin/mvn.cmd", "gradle/bin/gradle.bat"][..]),
            ("cmake", &["tools/ninja.exe"]),
            ("php", &["php/composer.phar"]),
            ("pnpm", &["@data/npm-global/yarn.cmd"]),
            ("vscodium-server", &["vscodium-server/node.exe"]),
        ] {
            let c = catalog.find(id).unwrap();
            assert_eq!(c.keys, parts, "{id}");
            touch(&c.key_path(&kit));
            assert!(!c.is_installed(&kit), "{id}: main file only");
            for p in &c.key_paths(&kit)[1..] {
                touch(p);
            }
            assert!(c.is_installed(&kit), "{id}: every part");
        }
        let pnpm = catalog.find("pnpm").unwrap();
        assert!(pnpm.key_paths(&kit)[1].starts_with(kit.data().join("npm-global")));
    }

    #[test]
    fn editor_installed_means_the_files_started() {
        let tmp = TempDir::new("catalog-editor");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let dir = crate::editor::server_dir(&kit);
        touch(&dir.join(r"bin\codium-server.cmd"));
        assert!(!crate::editor::is_installed(&kit));
        touch(&dir.join(r"out\server-main.js"));
        assert!(!crate::editor::is_installed(&kit), "node.exe missing");
        touch(&dir.join("node.exe"));
        assert!(crate::editor::is_installed(&kit));
        assert!(Catalog::get()
            .find("vscodium-server")
            .unwrap()
            .is_installed(&kit));
    }

    #[test]
    fn shared_paths_go_with_the_last_owner() {
        let tmp = TempDir::new("catalog-shared");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let catalog = Catalog::get();
        let (php, apache) = (
            catalog.find("php").unwrap(),
            catalog.find("apache").unwrap(),
        );
        let vc = kit.apps().join(r"tools\vcruntime");
        for c in [php, apache] {
            for p in c.key_paths(&kit) {
                touch(&p);
            }
        }
        assert!(!php.owned_paths(&kit).contains(&vc), "Apache still uses it");
        assert!(!apache.owned_paths(&kit).contains(&vc), "PHP still uses it");
        assert!(php.owned_paths(&kit).contains(&kit.apps().join("php")));
        // Half of PHP still there: it still counts as an owner.
        fs::remove_file(php.key_path(&kit)).unwrap();
        assert!(
            !apache.owned_paths(&kit).contains(&vc),
            "composer.phar left"
        );
        // PHP removed (what its removal deletes, vcruntime aside).
        for p in php.owned_paths(&kit) {
            if p.is_dir() {
                fs::remove_dir_all(&p).unwrap();
            } else if p.exists() {
                fs::remove_file(&p).unwrap();
            }
        }
        assert!(apache.owned_paths(&kit).contains(&vc), "last owner");
        assert!(apache
            .owned_paths(&kit)
            .contains(&kit.apps().join("apache")));
    }
}
