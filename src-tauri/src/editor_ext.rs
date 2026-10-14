//! Recommended extensions of the integrated editor (port of nabre/dev-kit
//! `setup/vscodium/extensions.txt`, `app-extensions.js` and
//! `_vscodium-extensions.ps1`).
//!
//! - list embedded (`catalog/extensions.txt`) plus the user's
//!   `data\app\extensions.txt`, same format: `publisher.name [@program,...]`;
//! - `@program`: installed only when one of those components is in the kit;
//! - each extension is offered once (`data\app\extensions-offered.txt`): if
//!   the user uninstalls it, it does not come back; a failure is retried at
//!   the next start;
//! - CodeLLDB comes from its GitHub release (Open VSX only has a bootstrap
//!   that downloads at run time), checked with GitHub's SHA-256;
//! - installed in the background after the editor starts (DECISIONS.md
//!   D-36), with the server's own command line, into
//!   `data\vscodium-server\extensions`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::catalog::Catalog;
use crate::env::KitEnv;
use crate::jobs;
use crate::kit::{write_atomic, Kit};
use crate::net::{github_digest, Net};
use crate::verify::Expect;

const LIST: &str = include_str!("../catalog/extensions.txt");
pub const CODELLDB: &str = "vadimcn.vscode-lldb";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Lower case `publisher.name`.
    pub id: String,
    /// Component ids; empty = always.
    pub programs: Vec<String>,
}

/// Parses the list format; later lines win for the same id.
pub fn parse(text: &str) -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        let mut words = line.split_whitespace();
        let Some(id) = words.next() else { continue };
        let valid = id.contains('.')
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
        if !valid {
            continue;
        }
        let programs: Vec<String> = words
            .filter_map(|w| w.strip_prefix('@'))
            .flat_map(|w| w.split(','))
            .map(|p| p.trim().to_ascii_lowercase())
            .filter(|p| !p.is_empty())
            .collect();
        let id = id.to_ascii_lowercase();
        out.retain(|e| e.id != id);
        out.push(Entry { id, programs });
    }
    out
}

fn offered_path(kit: &Kit) -> PathBuf {
    kit.app_dir().join("extensions-offered.txt")
}

pub fn extensions_dir(kit: &Kit) -> PathBuf {
    kit.data().join(r"vscodium-server\extensions")
}

fn read_set(path: &Path) -> BTreeSet<String> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_ascii_lowercase())
        .filter(|l| !l.is_empty())
        .collect()
}

fn mark_offered(kit: &Kit, ids: &[String]) -> Result<()> {
    let path = offered_path(kit);
    let mut set = read_set(&path);
    set.extend(ids.iter().map(|i| i.to_ascii_lowercase()));
    let text: String = set.iter().map(|i| format!("{i}\r\n")).collect();
    write_atomic(&path, text.as_bytes())?;
    Ok(())
}

/// Ids of the extensions already in the server's extensions folder.
fn installed(kit: &Kit) -> BTreeSet<String> {
    fs::read_dir(extensions_dir(kit))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_ascii_lowercase();
            // `<publisher>.<name>-<version>[-<platform>]`
            let at = name
                .char_indices()
                .find(|(i, c)| {
                    *c == '-' && name[i + 1..].starts_with(|d: char| d.is_ascii_digit())
                })?
                .0;
            Some(name[..at].to_owned())
        })
        .collect()
}

/// What to install now: needed, not offered yet, not installed.
pub fn pending(kit: &Kit) -> Vec<String> {
    let user = fs::read_to_string(kit.app_dir().join("extensions.txt")).unwrap_or_default();
    let entries = parse(&format!("{LIST}\n{user}"));
    let catalog = Catalog::get();
    let present = |p: &str| catalog.find(p).is_some_and(|c| c.is_installed(kit));
    let offered = read_set(&offered_path(kit));
    let have = installed(kit);
    entries
        .into_iter()
        .filter(|e| e.programs.is_empty() || e.programs.iter().any(|p| present(p)))
        .map(|e| e.id)
        .filter(|id| !offered.contains(id) && !have.contains(id))
        .collect()
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExtState {
    pub running: bool,
    pub total: usize,
    pub done: usize,
    pub failed: Vec<String>,
}

/// Installs the pending extensions in a background thread; `state` follows.
pub fn sync_in_background(
    kit: Kit,
    env: KitEnv,
    net: Option<Arc<Net>>,
    state: Arc<Mutex<ExtState>>,
) {
    let ids = pending(&kit);
    if ids.is_empty() {
        return;
    }
    {
        let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
        if s.running {
            return;
        }
        *s = ExtState {
            running: true,
            total: ids.len(),
            ..ExtState::default()
        };
    }
    std::thread::spawn(move || {
        let result = install(&kit, &env, net.as_deref(), &ids, &state);
        let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
        s.running = false;
        if let Err(e) = result {
            crate::log::warn(&format!("editor extensions: {e:#}"));
        }
    });
}

fn server_cli(kit: &Kit, env: &KitEnv) -> std::process::Command {
    let dir = crate::editor::server_dir(kit);
    let mut cmd = jobs::kit_command(dir.join("node.exe"), env);
    cmd.arg(dir.join(r"out\server-main.js"))
        .arg("--server-data-dir")
        .arg(kit.data().join("vscodium-server"))
        .arg("--accept-server-license-terms");
    cmd
}

fn install(
    kit: &Kit,
    env: &KitEnv,
    net: Option<&Net>,
    ids: &[String],
    state: &Mutex<ExtState>,
) -> Result<()> {
    let mut gallery: Vec<String> = Vec::new();
    let mut sources: Vec<(String, PathBuf)> = Vec::new();
    let work = kit.data().join(r"tmp\setup\extensions");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work)?;
    for id in ids {
        if id == CODELLDB {
            match net.map(|n| codelldb_vsix(kit, n, &work)) {
                Some(Ok(vsix)) => sources.push((id.clone(), vsix)),
                Some(Err(e)) => crate::log::warn(&format!("CodeLLDB: {e:#}")),
                None => {}
            }
        } else {
            gallery.push(id.clone());
        }
    }
    let mut cmd = server_cli(kit, env);
    for id in &gallery {
        cmd.arg("--install-extension").arg(id);
    }
    for (_, vsix) in &sources {
        cmd.arg("--install-extension").arg(vsix);
    }
    crate::log::info(&format!("editor extensions: installing {}", ids.join(", ")));
    let out = cmd
        .stdin(Stdio::null())
        .output()
        .context("server --install-extension")?;
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = fs::remove_dir_all(&work);
    let ok = installed_from_output(&text);
    let succeeded: Vec<String> = ids
        .iter()
        .filter(|id| {
            ok.contains(*id)
                || (sources.iter().any(|(s, _)| s == *id) && installed(kit).contains(*id))
        })
        .cloned()
        .collect();
    mark_offered(kit, &succeeded)?;
    let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
    s.done = succeeded.len();
    s.failed = ids
        .iter()
        .filter(|i| !succeeded.contains(i))
        .cloned()
        .collect();
    if !s.failed.is_empty() {
        crate::log::warn(&format!(
            "editor extensions not installed (retried next start): {}",
            s.failed.join(", ")
        ));
    }
    Ok(())
}

/// Ids from lines like `Extension 'a.b' v1.0 was successfully installed.`
/// (also "is already installed").
pub fn installed_from_output(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter(|l| l.contains("successfully installed") || l.contains("already installed"))
        .filter_map(|l| {
            let start = l.find('\'')? + 1;
            let end = l[start..].find('\'')? + start;
            Some(l[start..end].to_ascii_lowercase())
        })
        .collect()
}

/// CodeLLDB's platform package from GitHub, verified (`Install-CodeLldb`).
fn codelldb_vsix(kit: &Kit, net: &Net, work: &Path) -> Result<PathBuf> {
    let rel = net.github_latest("vadimcn/codelldb")?;
    let name = format!("codelldb-win32-{}.vsix", kit.arch().as_str());
    let asset = rel
        .assets
        .iter()
        .find(|a| a.name == name)
        .ok_or_else(|| anyhow!("no {name} in {}", rel.tag_name))?;
    let file = work.join(&name);
    let cancel = std::sync::atomic::AtomicBool::new(false);
    net.download(&asset.browser_download_url, &file, &cancel, &mut |_, _| {})?;
    crate::verify::check(&file, &Expect::Sha256(github_digest(asset)?))?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn list_format() {
        let e = parse("# c\nMs-Python.python @python\nfoo.bar  # x\nnot-an-id\nvadimcn.vscode-lldb @rust,zig,cc\nfoo.bar @go\n");
        assert_eq!(e.len(), 3);
        assert_eq!(
            e[0],
            Entry {
                id: "ms-python.python".into(),
                programs: vec!["python".into()]
            }
        );
        assert_eq!(e[1].programs, vec!["rust", "zig", "cc"]);
        assert_eq!(
            e[2],
            Entry {
                id: "foo.bar".into(),
                programs: vec!["go".into()]
            },
            "later line wins"
        );
        let embedded = parse(LIST);
        assert!(embedded.len() >= 55, "embedded list loaded");
        for e in &embedded {
            for p in &e.programs {
                assert!(
                    Catalog::get().find(p).is_some(),
                    "{}: unknown program {p}",
                    e.id
                );
            }
        }
    }

    #[test]
    fn pending_follows_programs_offered_and_installed() {
        let tmp = TempDir::new("ext");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let p = pending(&kit);
        assert!(
            p.contains(&"esbenp.prettier-vscode".to_owned()),
            "no tag: always"
        );
        assert!(
            !p.contains(&"ms-python.python".to_owned()),
            "python not installed"
        );
        let py = Catalog::get().find("python").unwrap().key_path(&kit);
        fs::create_dir_all(py.parent().unwrap()).unwrap();
        fs::write(&py, b"").unwrap();
        assert!(pending(&kit).contains(&"ms-python.python".to_owned()));
        mark_offered(&kit, &["ms-python.python".into()]).unwrap();
        fs::create_dir_all(extensions_dir(&kit).join("esbenp.prettier-vscode-12.4.0-universal"))
            .unwrap();
        let p = pending(&kit);
        assert!(!p.contains(&"ms-python.python".to_owned()), "offered once");
        assert!(
            !p.contains(&"esbenp.prettier-vscode".to_owned()),
            "already installed"
        );
    }

    #[test]
    fn output_parsing() {
        let text = "Installing extension 'a.b'...\nExtension 'mikestead.dotenv' v1.0.1 was successfully installed.\nExtension 'X.Y' v2 is already installed.\nFailed Installing Extensions: c.d";
        let got = installed_from_output(text);
        assert!(got.contains("mikestead.dotenv") && got.contains("x.y"));
        assert!(!got.contains("a.b") && !got.contains("c.d"));
    }
}
