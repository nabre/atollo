//! The knowledge of the matrix, `catalog/relations.json` (embedded): the
//! links between components, the runtimes the PC provides, the files where
//! projects declare versions.

use serde::Deserialize;

pub(super) const KNOWN: &str = include_str!("../../../catalog/relations.json");

#[derive(Debug, Deserialize)]
pub(super) struct Known {
    pub(super) links: Vec<LinkSpec>,
    pub(super) runtimes: Vec<RuntimeSpec>,
    pub(super) projects: Vec<ProjectSpec>,
}

#[derive(Debug, Deserialize)]
pub(super) struct LinkSpec {
    pub(super) from: String,
    pub(super) to: String,
    pub(super) kind: String,
    #[serde(default)]
    pub(super) range: Option<String>,
    #[serde(default)]
    pub(super) optional: bool,
    /// Only the version of `to` the kit uses counts: `from` picks it
    /// whatever a project uses (Gradle on the newest JDK, Composer per PHP).
    #[serde(default)]
    pub(super) kit: bool,
    #[serde(default)]
    pub(super) note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct RuntimeSpec {
    pub(super) id: String,
    pub(super) name: String,
    /// Component ids; `@app` = the app itself, `@x64Only` = every
    /// component without an ARM64 build.
    #[serde(rename = "for")]
    pub(super) for_: Vec<String>,
    /// Only on ARM64 kits.
    #[serde(default)]
    pub(super) arm64: bool,
    #[serde(default)]
    pub(super) note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ProjectSpec {
    pub(super) component: String,
    pub(super) file: String,
    /// A dotted path in a JSON file (`engines.node`).
    #[serde(default)]
    pub(super) json: Option<String>,
    /// Group 1 of a regex over the file.
    #[serde(default)]
    pub(super) regex: Option<String>,
    /// The first line of the file.
    #[serde(default)]
    pub(super) whole: bool,
    /// A version that names a line (`3.12.4` = any 3.12).
    #[serde(default)]
    pub(super) line: bool,
    /// A bare version is a minimum (`go 1.21`).
    #[serde(default)]
    pub(super) minimum: bool,
}

/// The notes of the knowledge (Italian), translated by the catalogs.
#[cfg(test)]
pub fn notes() -> Vec<&'static str> {
    let k = known();
    k.links
        .iter()
        .filter_map(|l| l.note.as_deref())
        .chain(k.runtimes.iter().filter_map(|r| r.note.as_deref()))
        .collect()
}

pub(super) fn known() -> &'static Known {
    static KNOWN_DATA: std::sync::OnceLock<Known> = std::sync::OnceLock::new();
    KNOWN_DATA.get_or_init(|| {
        // Validated by the tests: a broken file ships an empty matrix.
        serde_json::from_str(KNOWN).unwrap_or(Known {
            links: Vec::new(),
            runtimes: Vec::new(),
            projects: Vec::new(),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::catalog::Catalog;
    use crate::versions::ranges::{Bare, Range};
    use crate::versions::relations::split;

    #[test]
    fn knowledge_is_valid() {
        let k: Known = serde_json::from_str(KNOWN).unwrap();
        let catalog = Catalog::get();
        let parts = ["composer", "gradle", "maven"];
        for l in &k.links {
            let (c, part) = split(&l.from);
            assert!(catalog.find(c).is_some(), "{}", l.from);
            assert!(part.is_none_or(|p| parts.contains(&p)), "{}", l.from);
            assert!(catalog.find(&l.to).is_some(), "{}", l.to);
            if let Some(r) = &l.range {
                assert!(Range::parse(r, Bare::Exact).is_some(), "{r}");
            }
            assert!(["needs", "uses", "runs-on", "loads"].contains(&l.kind.as_str()));
        }
        for r in &k.runtimes {
            for f in &r.for_ {
                assert!(f.starts_with('@') || catalog.find(f).is_some(), "{f}");
            }
        }
        for p in &k.projects {
            assert!(catalog.find(&p.component).is_some(), "{}", p.component);
            assert!(
                p.json.is_some() || p.regex.is_some() || p.whole,
                "{}",
                p.file
            );
            if let Some(re) = &p.regex {
                regex::Regex::new(re).unwrap();
            }
        }
    }
}
