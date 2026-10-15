//! The lines each source offers, kept in `datappersion-lines.json`,
//! and whether a chosen line has to be downloaded.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::{check, find, is_present, valid_line};
use crate::kit::Kit;

/// The lines a source offered for a component, and when (Unix seconds).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offered {
    pub lines: Vec<String>,
    pub checked: u64,
}

pub(crate) fn offered_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("version-lines.json")
}

/// Every component's offered lines (only valid ones, newest first).
pub fn offered(kit: &Kit) -> BTreeMap<String, Offered> {
    let mut all: BTreeMap<String, Offered> =
        crate::fsx::read_json(&offered_file(kit)).unwrap_or_default();
    all.retain(|id, _| find(id).is_some());
    for o in all.values_mut() {
        o.lines.retain(|l| valid_line(l));
        o.lines
            .sort_by_key(|l| std::cmp::Reverse(crate::install::version_key(l)));
        o.lines.dedup();
    }
    all
}

/// Keeps what the source of `id` offers now.
pub fn save_offered(kit: &Kit, id: &str, lines: &[String]) -> Result<()> {
    // Lists of several components can be asked for at the same time.
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut all = offered(kit);
    let checked = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    all.insert(
        id.to_owned(),
        Offered {
            lines: lines.iter().filter(|l| valid_line(l)).cloned().collect(),
            checked,
        },
    );
    crate::fsx::write_json(&offered_file(kit), &all)
}

/// `line` of `id` has to be downloaded before it can be used: the component
/// is installed, the line is not on disk and its installer can fetch it.
pub fn to_download(kit: &Kit, id: &str, line: &str) -> Result<bool> {
    check(id, Some(line))?;
    let Some(c) = find(id) else { return Ok(false) };
    if !c.is_installed(kit) || is_present(kit, c, line) {
        return Ok(false);
    }
    if !crate::install::installer(id).is_some_and(|i| i.has_lines()) {
        bail!("{id} {line}: not saved and it cannot be downloaded");
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};
    use crate::setup::catalog::Catalog;
    use crate::versions::tests::install_fake;
    use crate::versions::{install_dir, saved_dir, tools};

    #[test]
    fn offered_lines_and_downloads() {
        let tmp = TempDir::new("versions-offered");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let node = Catalog::get().find("node").unwrap();

        // Nothing installed: nothing to download.
        assert!(!to_download(&kit, "node", "20.18").unwrap());

        let main = install_dir(&kit, node, "v22.11.0");
        install_fake(&kit, node, &main);
        node.save_version(&kit, "v22.11.0").unwrap();
        let old = saved_dir(&kit, node, "20.18");
        install_fake(&kit, node, &old);
        assert!(!to_download(&kit, "node", "22.11").unwrap(), "the default");
        assert!(!to_download(&kit, "node", "20.18").unwrap(), "saved");
        assert!(to_download(&kit, "node", "18.20").unwrap());
        assert!(to_download(&kit, "node", "x").is_err());

        // Kept per component, valid lines only, newest first.
        save_offered(&kit, "node", &["20.18".into(), "24.1".into(), "bad".into()]).unwrap();
        save_offered(&kit, "claude", &["1.0".into()]).unwrap();
        let all = offered(&kit);
        assert_eq!(all.keys().collect::<Vec<_>>(), ["node"]);
        assert_eq!(all["node"].lines, ["24.1", "20.18"]);
        assert!(all["node"].checked > 0);

        crate::projects::settings::update(&kit, "app", |s| {
            s.versions.insert("node".into(), "20.18".into());
        });
        let tool = tools(&kit).into_iter().find(|t| t.id == "node").unwrap();
        assert_eq!(tool.available, ["24.1", "20.18"]);
        assert!(tool.checked.is_some());
        assert_eq!(tool.projects.get("20.18").map(Vec::len), Some(1));
    }
}
