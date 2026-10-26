//! The components of the kit and the user's choice.
//!
//! The catalog (`catalog/components.json`, embedded) is the only list of
//! components (docs/app/CATALOG.md). The choice is `data/app/components.txt`,
//! in this format: three comment lines,
//! then `id=1` or `id=0` per component, UTF-8 without BOM, CRLF. Versions are
//! `data/app/versions/<target>/<id>.txt`, plain text.
//!
//! Availability per target (D-108): a component is installed on a target
//! only with a build made for it, natively or with Rosetta 2 on a Mac with
//! Apple silicon; otherwise it is "not available" with the reason, never a
//! binary of another architecture that does not start.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::common::fsx::{join_rel, read_text, write_atomic};
use crate::kit::{Arch, Kit, Target};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Group {
    Required,
    Agents,
    /// Local language models (llama.cpp, D-151): the server the agents and
    /// the chat use, with the models of `data/models`.
    Ai,
    Languages,
    /// Database servers and tools (D-73), with their own group too.
    Databases,
    /// Part of Atollo itself (the editor): not in "Linguaggi e strumenti",
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
    /// Components it needs on every target. Private: what it needs on a
    /// target is [`Component::needs_on`], with `needs_by_target` too.
    #[serde(default)]
    needs: Vec<String>,
    /// Components it needs on one target only, after `needs`: C and C++
    /// are `zig cc` on macOS and Linux (`{"linux-x64": ["zig"]}`).
    #[serde(default)]
    needs_by_target: BTreeMap<String, Vec<String>>,
    /// Italian; translated by the catalogs.
    pub desc: String,
    pub home: String,
    /// The release notes of a version, `{version}` replaced (N-9); without,
    /// the site (`home`).
    #[serde(default)]
    pub notes: Option<String>,
    pub key: String,
    /// The other parts (Maven, Ninja, Composer...), same rules as `key`:
    /// installed only when the key and all of these exist.
    #[serde(default)]
    pub keys: Vec<String>,
    /// The folder of `data/apps/<target>` the component owns, if any.
    #[serde(default)]
    pub folder: Option<String>,
    /// No Apple silicon build: the x64 one runs on `darwin-arm64` with
    /// Rosetta 2, and is not available on a Mac without it
    /// (`router::target::rosetta_missing`).
    #[serde(default)]
    pub rosetta: bool,
    /// Explicit list of targets where this component is available (e.g.
    /// `["win32-x64"]` for ConPTY). Empty = all targets.
    #[serde(default)]
    pub targets: Vec<String>,
    /// Targets where no build exists, with the reason the interface shows
    /// (Italian, translated by the catalogs): `{"linux-arm64": "…"}`.
    #[serde(default)]
    pub unavailable: BTreeMap<String, String>,
    /// Other files or folders it owns (relative to `data/apps/<target>`,
    /// `@` = the kit root), removed with it unless another installed
    /// component lists them too. Names of Windows; on macOS and Linux
    /// `remove_unix` takes their place when the catalog gives it.
    #[serde(default)]
    pub remove: Vec<String>,
    /// The same as `remove` on macOS and Linux, where the files are laid
    /// out differently (`bin/pnpm`, `lib/node_modules/pnpm`); same rules.
    #[serde(default)]
    pub remove_unix: Vec<String>,
    /// Version command for the final check: program (relative to
    /// `data/apps/<target>`, `@` = the kit root) and arguments.
    #[serde(default)]
    pub check: Vec<String>,
    /// The oldest version Atollo works with (a component that follows its
    /// source's latest, like Docker Sandboxes): one older, or of unknown
    /// version, is [`Component::outdated`] and installed again.
    #[serde(default)]
    pub min_version: Option<String>,
    /// Group `app`: the version this release of Atollo installs, and the
    /// SHA-256 of its download.
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    /// SHA-256 fixed per target, with `version`, for any component whose
    /// supplier publishes no hash (the editor today; PHP and MySQL builds
    /// later): the installer checks the download against it and refuses
    /// another one.
    #[serde(default)]
    pub sha256_by_target: BTreeMap<String, String>,
    /// Other minor lines can be kept next to the default folder, and chosen
    /// for the kit or a project (`versions/mod.rs`, D-80).
    #[serde(default)]
    pub versioned: bool,
    /// One folder per minor line too, but a single version, always the
    /// latest: a new patch replaces its line, a new line takes the place of
    /// the old one (`versions/mod.rs`).
    #[serde(default)]
    pub minor_folder: bool,
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
            let file: File = serde_json::from_str(include_str!("../../catalog/components.json"))
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

    /// Always installed on `target`: the system components (required, and
    /// the parts of the app) and, transitively, what they need there
    /// (the editor needs PowerShell 7). Git is not among them: GitHub CLI
    /// works without it (D-81). Never turned off or skipped.
    pub fn locked_on(&self, target: Target) -> BTreeSet<&str> {
        let always: Vec<&str> = self
            .components
            .iter()
            .filter(|c| c.group.always())
            .map(|c| c.id.as_str())
            .collect();
        self.with_needs_on(&always, target)
    }

    /// Locked on every target: the system components and what they need
    /// on every target (`needs` alone). What the saved choice keeps on,
    /// the same whatever system reads it; a run on a target installs
    /// [`Catalog::locked_on`] that target.
    pub fn locked(&self) -> BTreeSet<&str> {
        let always: Vec<&str> = self
            .components
            .iter()
            .filter(|c| c.group.always())
            .map(|c| c.id.as_str())
            .collect();
        self.closure(&always, Component::needs_on_every_target)
    }

    /// [`Catalog::locked`] contains `id`.
    pub fn is_locked(&self, id: &str) -> bool {
        self.locked().contains(id)
    }

    /// `ids` (those in the catalog) and, transitively, what they need on
    /// `target`.
    pub fn with_needs_on<'a>(&'a self, ids: &[&str], target: Target) -> BTreeSet<&'a str> {
        self.closure(ids, |c| c.needs_on(target))
    }

    /// What `ids` need on `target`, transitively, and not on every target:
    /// Zig for C and C++ on macOS and Linux. Never in the saved choice
    /// (see [`Component::needs_on_every_target`]): a run on `target` adds
    /// it to the chosen components that need it, and keeps it while they
    /// are chosen. An id of `ids` is in it only when another one needs it.
    pub fn needs_only_on<'a>(&'a self, ids: &[&str], target: Target) -> BTreeSet<&'a str> {
        let found: Vec<&Component> = ids.iter().filter_map(|id| self.find(id)).collect();
        let here: Vec<&str> = found.iter().flat_map(|c| c.needs_on(target)).collect();
        let everywhere: Vec<&str> = found
            .iter()
            .flat_map(|c| c.needs_on_every_target())
            .collect();
        let here = self.with_needs_on(&here, target);
        let everywhere = self.closure(&everywhere, Component::needs_on_every_target);
        here.difference(&everywhere).copied().collect()
    }

    /// `ids` (those in the catalog) and, transitively, what `needs` gives.
    fn closure<'a, I>(
        &'a self,
        ids: &[&str],
        needs: impl Fn(&'a Component) -> I,
    ) -> BTreeSet<&'a str>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut out = BTreeSet::new();
        let mut todo: Vec<&'a str> = ids
            .iter()
            .filter_map(|id| self.find(id))
            .map(|c| c.id.as_str())
            .collect();
        while let Some(id) = todo.pop() {
            if let Some(c) = self.find(id) {
                if out.insert(c.id.as_str()) {
                    todo.extend(needs(c));
                }
            }
        }
        out
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

/// Translates a catalog relative path (`"node/node.exe"`, `"python/python.exe"`,
/// `"tinytex/bin/windows/tlmgr.bat"`, `"@data/npm-global/pnpm.cmd"`) to the
/// layout of `target`. On Windows (`win32-x64`) paths are kept
/// as written; on macOS and Linux they follow the Unix binary layout.
pub fn translate_rel(target: Target, rel: &str) -> String {
    if target.is_windows() {
        return rel.to_owned();
    }
    let norm = rel.replace('\\', "/");
    match norm.as_str() {
        // Git's catalog key (`git/bin/bash.exe` on Windows) is its `git` on
        // Unix: Git of the kit has no shell there. The kit's bash profiles
        // and terminal use the system's `/bin/bash` (editor/cfg.rs,
        // editor/pty.rs), never this path as a shell (D-109).
        "git/bin/bash.exe" | "git/cmd/git.exe" => return "git/bin/git".to_owned(),
        "python/python.exe" => return "python/bin/python3".to_owned(),
        "node/node.exe" => return "node/bin/node".to_owned(),
        "node/npm.cmd" => return "node/bin/npm".to_owned(),
        "node/npx.cmd" => return "node/bin/npx".to_owned(),
        "php/php.exe" => return "php/bin/php".to_owned(),
        "@data/npm-global/pnpm.cmd" => return "@data/npm-global/bin/pnpm".to_owned(),
        "@data/npm-global/pnpx.cmd" => return "@data/npm-global/bin/pnpx".to_owned(),
        "@data/npm-global/pn.cmd" => return "@data/npm-global/bin/pn".to_owned(),
        "@data/npm-global/pnx.cmd" => return "@data/npm-global/bin/pnx".to_owned(),
        "@data/npm-global/yarn.cmd" => return "@data/npm-global/bin/yarn".to_owned(),
        "@data/npm-global/yarnpkg.cmd" => return "@data/npm-global/bin/yarnpkg".to_owned(),
        _ => {}
    }
    let mapped = if let Some(rest) = norm.strip_prefix("tinytex/bin/windows/") {
        format!("tinytex/bin/{}/{rest}", target.texlive_bin())
    } else {
        norm
    };
    for ext in [".exe", ".cmd", ".bat"] {
        if let Some(stem) = mapped.strip_suffix(ext) {
            return stem.to_owned();
        }
    }
    mapped
}

/// A path of the catalog: relative to `data/apps/<target>`, or to the kit
/// root with `@`. In the folder of a component with one folder per line it
/// is that of its default version (`node/node.exe` → `node/22.11/node.exe`,
/// versions/mod.rs).
pub fn kit_path(kit: &Kit, rel: &str) -> PathBuf {
    let translated = translate_rel(kit.target(), rel);
    match translated.strip_prefix('@') {
        Some(r) => join_rel(kit.root(), r),
        None => crate::versions::resolve(kit, &translated)
            .unwrap_or_else(|| join_rel(kit.apps(), &translated)),
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
    /// local models, languages (several versions), tools (one, the
    /// latest), databases.
    pub fn page_section(&self) -> u8 {
        match self.group {
            Group::Required | Group::App => 0,
            Group::Agents => 1,
            Group::Ai => 2,
            Group::Languages if self.versioned => 3,
            Group::Languages => 4,
            Group::Databases => 5,
        }
    }

    /// What it needs on `target`: `needs`, then `needs_by_target` of that
    /// target (C and C++ need Zig on macOS and Linux only). The only way to
    /// read its dependencies, so none is taken for the wrong system.
    pub fn needs_on(&self, target: Target) -> impl Iterator<Item = &str> + '_ {
        self.needs
            .iter()
            .chain(
                self.needs_by_target
                    .get(target.as_str())
                    .into_iter()
                    .flatten()
                    .filter(|n| !self.needs.contains(n)),
            )
            .map(String::as_str)
    }

    /// What it needs on every target (`needs` alone). The saved choice
    /// follows these only: `components.txt` is one for every system the
    /// kit runs on, so a dependency of one target must not turn a
    /// component on or off for the others; a run adds those of its own
    /// target ([`Catalog::needs_only_on`]).
    pub fn needs_on_every_target(&self) -> impl Iterator<Item = &str> + '_ {
        self.needs.iter().map(String::as_str)
    }

    /// [`Component::needs_on`] as a list, for the interface.
    pub fn needs_list_on(&self, target: Target) -> Vec<String> {
        self.needs_on(target).map(str::to_owned).collect()
    }

    /// True when the catalog has a build for `target` (native, or x64 with
    /// Rosetta 2): a supported target (D-176), listed in `targets` (or no
    /// list), not in `unavailable`. Whether Rosetta 2 is there is the
    /// host's business (`router::target::installable_on`).
    pub fn is_available_on(&self, target: Target) -> bool {
        if !target.is_supported() {
            return false;
        }
        if !self.targets.is_empty() && !self.targets.iter().any(|t| t == target.as_str()) {
            return false;
        }
        !self.unavailable.contains_key(target.as_str())
    }

    /// Why it is not available on `target`, as the catalog says it (Italian
    /// source, for `i18n::t`); `None` without a reason of its own (outside
    /// `targets`) or when it is available.
    pub fn unavailable_reason(&self, target: Target) -> Option<&str> {
        self.unavailable.get(target.as_str()).map(String::as_str)
    }

    /// True when `target` uses the x64 build under OS binary translation:
    /// `rosetta` on `darwin-arm64`.
    pub fn is_emulated_on(&self, target: Target) -> bool {
        target == Target::DARWIN_ARM64 && self.rosetta
    }

    /// What it owns besides its folder on `target`: `remove`, or on macOS
    /// and Linux `remove_unix` when the catalog gives it.
    pub fn remove_for(&self, target: Target) -> &[String] {
        if target.is_windows() || self.remove_unix.is_empty() {
            &self.remove
        } else {
            &self.remove_unix
        }
    }

    /// Fixed SHA-256 for `target` (checks `sha256_by_target` first, then `sha256`
    /// on `win32-x64`).
    pub fn sha256_for(&self, target: Target) -> Option<&str> {
        if let Some(s) = self.sha256_by_target.get(target.as_str()) {
            return Some(s.as_str());
        }
        if target.is_windows() {
            return self.sha256.as_deref();
        }
        None
    }

    /// `self.key` translated for `target`.
    pub fn key_for(&self, target: Target) -> String {
        translate_rel(target, &self.key)
    }

    /// `self.keys` translated for `target`.
    pub fn keys_for(&self, target: Target) -> Vec<String> {
        self.keys.iter().map(|k| translate_rel(target, k)).collect()
    }

    /// Where the key lives: `data/apps/<target>/…`, or the kit root with `@`.
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
    /// another component still on disk owns too (`tools/vcruntime` of PHP
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
            .chain(
                self.remove_for(kit.target())
                    .iter()
                    .map(|r| kit_path(kit, r)),
            )
            // Saved lines of an older kit, not moved yet (versions::migrate).
            .chain(
                self.versioned
                    .then(|| kit.apps().join("versions").join(&self.id)),
            )
            .collect()
    }

    /// Its folder holds one folder per minor line (`git .56`): versioned,
    /// or `minor_folder`.
    pub fn per_line(&self) -> bool {
        self.versioned || self.minor_folder
    }

    pub fn folder_path(&self, kit: &Kit) -> Option<PathBuf> {
        self.folder.as_ref().map(|f| kit.apps().join(f))
    }

    fn version_file(kit: &Kit, id: &str) -> PathBuf {
        kit.versions_dir().join(format!("{id}.txt"))
    }

    fn legacy_version_file(kit: &Kit, id: &str) -> PathBuf {
        kit.versions_root().join(format!("{id}.txt"))
    }

    pub fn installed_version(&self, kit: &Kit) -> Option<String> {
        let read = |p: PathBuf| {
            read_text(&p)
                .ok()
                .map(|v| v.trim().to_owned())
                .filter(|v| !v.is_empty())
        };
        read(Self::version_file(kit, &self.id)).or_else(|| {
            (kit.os().is_windows() && kit.arch() == Arch::X64)
                .then(|| read(Self::legacy_version_file(kit, &self.id)))
                .flatten()
        })
    }

    /// A part of the app (fixed `version`) installed at another version: the
    /// release of Atollo changed, so it is installed again.
    pub fn outdated(&self, kit: &Kit) -> bool {
        let installed = self.installed_version(kit);
        self.version
            .as_deref()
            .is_some_and(|v| installed.as_deref() != Some(v))
            || self.min_version.as_deref().is_some_and(|min| {
                installed
                    .as_deref()
                    .is_none_or(|i| crate::setup::updates::newer(min, i))
            })
    }

    pub fn save_version(&self, kit: &Kit, version: &str) -> Result<()> {
        let path = Self::version_file(kit, &self.id);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        write_atomic(&path, version.as_bytes()).with_context(|| path.display().to_string())
    }

    /// `<id>.reinstall` next to its version: the kit moved and the
    /// component cannot follow (a conda environment on macOS/Linux, D-92).
    fn reinstall_file(kit: &Kit, id: &str) -> PathBuf {
        kit.versions_dir().join(format!("{id}.reinstall"))
    }

    /// To be installed again although it is on disk: set when the kit moved
    /// (`kit/` `check_moved`), cleared by a successful installation.
    pub fn needs_reinstall(&self, kit: &Kit) -> bool {
        Self::reinstall_file(kit, &self.id).is_file()
    }

    /// Marks it to be installed again (`needs_reinstall`); the text is the
    /// root it was installed for, for the logs.
    pub fn mark_reinstall(&self, kit: &Kit, old_root: &str) -> std::io::Result<()> {
        let path = Self::reinstall_file(kit, &self.id);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        write_atomic(&path, old_root.as_bytes())
    }

    pub fn clear_reinstall(&self, kit: &Kit) {
        let _ = fs::remove_file(Self::reinstall_file(kit, &self.id));
    }

    /// `<id>@<line>.reinstall`: the same for a saved line (another version
    /// kept next to the default folder, `versions::saved`), also a conda
    /// environment that cannot follow the kit (D-92).
    fn line_reinstall_file(kit: &Kit, id: &str, line: &str) -> PathBuf {
        kit.versions_dir().join(format!("{id}@{line}.reinstall"))
    }

    /// Marks saved `line` to be installed again (`lines_to_reinstall`).
    pub fn mark_line_reinstall(
        &self,
        kit: &Kit,
        line: &str,
        old_root: &str,
    ) -> std::io::Result<()> {
        let path = Self::line_reinstall_file(kit, &self.id, line);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        write_atomic(&path, old_root.as_bytes())
    }

    pub fn clear_line_reinstall(&self, kit: &Kit, line: &str) {
        let _ = fs::remove_file(Self::line_reinstall_file(kit, &self.id, line));
    }

    /// The saved lines marked to be installed again that are still on disk,
    /// newest first. A mark whose line went (removed from the Tools page or
    /// by hand) is dropped: nothing is left to repair; one whose line became
    /// the default (the default removed first) marks the component instead.
    pub fn lines_to_reinstall(&self, kit: &Kit) -> Vec<String> {
        let prefix = format!("{}@", self.id);
        let mut marked: BTreeSet<String> = fs::read_dir(kit.versions_dir())
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.strip_prefix(&prefix)?
                    .strip_suffix(".reinstall")
                    .map(str::to_owned)
            })
            .collect();
        let saved: Vec<String> = crate::versions::saved(kit, self)
            .into_iter()
            .map(|l| l.line)
            .filter(|l| marked.remove(l))
            .collect();
        let default = crate::versions::default_line(kit, self);
        for gone in marked {
            if default.as_deref() == Some(gone.as_str()) && !self.needs_reinstall(kit) {
                let path = Self::line_reinstall_file(kit, &self.id, &gone);
                let old_root = read_text(&path).unwrap_or_default();
                if self.mark_reinstall(kit, &old_root).is_err() {
                    continue;
                }
            }
            self.clear_line_reinstall(kit, &gone);
        }
        saved
    }

    pub fn forget_version(&self, kit: &Kit) {
        self.clear_reinstall(kit);
        let prefix = format!("{}@", self.id);
        for e in fs::read_dir(kit.versions_dir())
            .into_iter()
            .flatten()
            .flatten()
        {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with(&prefix) && name.ends_with(".reinstall") {
                let _ = fs::remove_file(e.path());
            }
        }
        let _ = fs::remove_file(Self::version_file(kit, &self.id));
        if kit.os().is_windows() {
            let _ = fs::remove_file(Self::legacy_version_file(kit, &self.id));
        }
    }
}

/// One read-change-write of `components.txt` at a time: held from the read
/// of the [`Choice`] to its write, by every writer (BUG-V6-33).
pub fn choice_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Which components the user wants. `components.txt` is one for the whole
/// kit, read on every system it runs on: it follows the dependencies that
/// hold on every target only (`needs`). Those of one target (C and C++
/// need Zig on macOS and Linux) are added by the run on that target
/// ([`Catalog::needs_only_on`]) and never saved, so opening or preparing
/// the kit on Linux leaves the choice of Windows as it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    chosen: BTreeMap<String, bool>,
}

impl Choice {
    fn path(kit: &Kit) -> PathBuf {
        kit.app_dir().join("components.txt")
    }

    /// The defaults of the catalog, with what they need (Claude Code is a
    /// default, Git is not: Git comes with it). The same on every target.
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
        let text = read_text(&Self::path(kit)).ok()?;
        Some(Self::parse(&text, catalog))
    }

    /// [`Choice::read`], or the defaults on the first start.
    pub fn read_or_defaults(kit: &Kit, catalog: &Catalog) -> Choice {
        Self::read(kit, catalog).unwrap_or_else(|| Self::defaults(catalog))
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

    /// Required components are always on; a component whose dependency on
    /// every target is off is off too (`Repair-ComponentChoice`). One of a
    /// single target does not count: see [`Choice`].
    pub fn repair(&mut self, catalog: &Catalog) {
        for id in catalog.locked() {
            self.chosen.insert(id.to_owned(), true);
        }
        loop {
            let mut changed = false;
            for c in catalog.all() {
                if self.is_chosen(&c.id) && c.needs_on_every_target().any(|n| !self.is_chosen(n)) {
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

    /// The ids chosen, in catalog order.
    pub fn chosen_ids<'a>(&self, catalog: &'a Catalog) -> Vec<&'a str> {
        catalog
            .all()
            .iter()
            .filter(|c| self.is_chosen(&c.id))
            .map(|c| c.id.as_str())
            .collect()
    }

    /// Turns a component on or off, following the dependencies on every
    /// target both ways. Those of one target only are not saved (see
    /// [`Choice`]).
    pub fn set(&mut self, catalog: &Catalog, id: &str, on: bool) {
        let locked = catalog.locked();
        self.set_with(catalog, &locked, id, on);
    }

    fn set_with(&mut self, catalog: &Catalog, locked: &BTreeSet<&str>, id: &str, on: bool) {
        let Some(c) = catalog.find(id) else { return };
        if locked.contains(c.id.as_str()) {
            return;
        }
        self.chosen.insert(id.to_owned(), on);
        if on {
            for need in c.needs_on_every_target() {
                self.set_with(catalog, locked, need, true);
            }
        } else {
            for other in catalog
                .all()
                .iter()
                .filter(|o| o.needs_on_every_target().any(|n| n == id))
            {
                self.set_with(catalog, locked, &other.id, false);
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
#[path = "../../../test/rust/setup/catalog.rs"]
mod tests;
