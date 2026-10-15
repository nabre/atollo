//! Path aliases and `.localhost` domains of a served site (`/media` ->
//! `storage/uploads`, `/shared` -> `@common/assets`). They come from the
//! project's panel.json, a file of its repository: here they are parsed
//! (plain names only, `@` for another project) and resolved on disk without
//! following links (DECISIONS.md D-77) before a web server configuration may
//! contain them. The Apache and Nginx generators take only [`SiteAlias`], so
//! an alias that was not checked cannot reach a configuration.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::kit::validate_project_name;
use crate::os::safe_path;

/// RFC 6761 `.localhost` domain (`mio-sito.localhost`).
pub fn valid_domain(d: &str) -> bool {
    let d = d.trim();
    let Some(prefix) = d.strip_suffix(".localhost") else {
        return false;
    };
    !prefix.is_empty()
        && d.len() <= 120
        && prefix.split('.').all(|part| {
            !part.is_empty()
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        })
}

/// URL path prefix of an alias (`/media`, `/uploads`): `/`, then ASCII
/// letters, digits and `-_./`; no `..`, no trailing `/`.
pub fn valid_url(u: &str) -> bool {
    let u = u.trim();
    u.starts_with('/')
        && u.len() >= 2
        && u.len() <= 80
        && !u.ends_with('/')
        && !u.contains("..")
        && u.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.'))
}

/// Where an alias points, parsed from `storage/uploads`, `.` or `@common/assets`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A folder of the project itself (no parts: its root).
    Project(Vec<String>),
    /// A folder of another project of `projects\`.
    Other { project: String, parts: Vec<String> },
}

/// A name that is safe as it is in any configuration: ASCII letters, digits
/// and `-_.`, not starting with `.` (so never `..`).
fn plain(part: &str) -> bool {
    !part.is_empty()
        && !part.starts_with('.')
        && part
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

impl Target {
    /// `None` for an absolute path, a drive, `..`, or any character outside
    /// the plain names (quotes, line breaks, `$`, `{`, `;`, spaces).
    pub fn parse(target: &str) -> Option<Target> {
        let t = target.trim().replace('\\', "/");
        if t.is_empty()
            || t.len() > 160
            || t.starts_with('/')
            || t.contains(':')
            || t.contains("..")
        {
            return None;
        }
        let (other, body) = match t.strip_prefix('@') {
            Some(rest) => {
                let (project, body) = rest.split_once('/').unwrap_or((rest, "."));
                (Some(project), body)
            }
            None => (None, t.as_str()),
        };
        let mut parts = Vec::new();
        for part in body.split('/') {
            if part == "." {
                continue;
            }
            if !plain(part) {
                return None;
            }
            parts.push(part.to_owned());
        }
        match other {
            None => Some(Target::Project(parts)),
            Some(project) if plain(project) && validate_project_name(project).is_ok() => {
                Some(Target::Other {
                    project: project.to_owned(),
                    parts,
                })
            }
            Some(_) => None,
        }
    }
}

/// An alias checked on disk, ready for a configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteAlias {
    /// `/media`
    pub url: String,
    /// `None`: a folder of the project itself.
    other: Option<String>,
    parts: Vec<String>,
    /// The folder on disk.
    pub dir: PathBuf,
}

impl SiteAlias {
    /// The folder for Apache: under `own` (the expression of the project's
    /// folder, `${CP_PROJECT_FWD}`) or under `${CP_PROJECTS_FWD}` for another
    /// project; made only of checked names, so portable and safe in quotes.
    pub fn apache_path(&self, own: &str) -> String {
        let mut s = match &self.other {
            None => own.to_owned(),
            Some(project) => format!("${{CP_PROJECTS_FWD}}/{project}"),
        };
        for part in &self.parts {
            s.push('/');
            s.push_str(part);
        }
        s
    }
}

/// The aliases of a site that can be served: URL and target parsed, the
/// folder existing under `projects_root` with no link or junction on the way
/// (checked as text, never through `canonicalize`, which follows links). The
/// others are left out, each with a warning in the log.
pub fn resolve(
    project_root: &Path,
    projects_root: &Path,
    aliases: &BTreeMap<String, String>,
) -> Vec<SiteAlias> {
    aliases
        .iter()
        .filter_map(|(url, target)| {
            let found = resolve_one(project_root, projects_root, url, target);
            if found.is_none() {
                crate::log::warn(&format!(
                    "{}: alias {url} -> {target} not served (not a folder of the projects, or through a link)",
                    project_root.display()
                ));
            }
            found
        })
        .collect()
}

/// The URLs of the aliases [`resolve`] leaves out, for the interface to say
/// so (without the log warning: `resolve` already wrote it).
pub fn skipped(
    project_root: &Path,
    projects_root: &Path,
    aliases: &BTreeMap<String, String>,
) -> Vec<String> {
    aliases
        .iter()
        .filter(|(url, target)| resolve_one(project_root, projects_root, url, target).is_none())
        .map(|(url, _)| url.trim().to_owned())
        .collect()
}

fn resolve_one(
    project_root: &Path,
    projects_root: &Path,
    url: &str,
    target: &str,
) -> Option<SiteAlias> {
    let url = url.trim();
    if !valid_url(url) {
        return None;
    }
    let (other, parts) = match Target::parse(target)? {
        Target::Project(parts) => (None, parts),
        Target::Other { project, parts } => (Some(project), parts),
    };
    let base = match &other {
        None => project_root.to_path_buf(),
        Some(project) => projects_root.join(project),
    };
    let dir = safe_path::lexical(&parts.iter().fold(base, |d, p| d.join(p)))?;
    let root = safe_path::lexical(projects_root)?;
    if !safe_path::is_dir(&root, &dir) {
        return None;
    }
    Some(SiteAlias {
        url: url.to_owned(),
        other,
        parts,
        dir,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::tests::TempDir;
    use crate::os::safe_path::tests::junction;
    use std::fs;

    #[test]
    fn targets_parsed() {
        assert_eq!(
            Target::parse("storage/uploads"),
            Some(Target::Project(vec!["storage".into(), "uploads".into()]))
        );
        assert_eq!(Target::parse("."), Some(Target::Project(vec![])));
        assert_eq!(Target::parse(r"public\img"), Target::parse("public/img"));
        assert_eq!(
            Target::parse("@common/assets"),
            Some(Target::Other {
                project: "common".into(),
                parts: vec!["assets".into()]
            })
        );
        for bad in [
            "",
            "../x",
            "../../Windows",
            "a/../b",
            "C:/x",
            r"\\srv\x",
            "/x",
            "storage/",
            ".git",
            "@../x",
            "@common/../x",
            "@con/x",
            "@",
            "@/x",
            "a\"b",
            "a\nb",
            "a\rb",
            "${X}",
            "a;b",
            "a{b",
            "a b",
            "a$b",
        ] {
            assert_eq!(Target::parse(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn urls_and_domains() {
        assert!(valid_url("/media") && valid_url("/a/b.c"));
        for bad in [
            "", "/", "media", "/a/", "/a/../b", "/a\"b", "/a b", "/a\nb", "/a{b",
        ] {
            assert!(!valid_url(bad), "{bad:?}");
        }
        assert!(valid_domain("mio-sito.localhost") && valid_domain("a.b.localhost"));
        for bad in [
            "localhost",
            ".localhost",
            "a.com",
            "A.localhost",
            "a\".localhost",
            "a b.localhost",
        ] {
            assert!(!valid_domain(bad), "{bad:?}");
        }
    }

    #[test]
    fn resolved_only_inside_the_projects_without_links() {
        let tmp = TempDir::new("site-alias");
        let projects = tmp.0.join("projects");
        let shop = projects.join("shop");
        fs::create_dir_all(shop.join(r"storage\uploads")).unwrap();
        fs::create_dir_all(projects.join(r"common\assets")).unwrap();
        let outside = tmp.0.join("outside");
        fs::create_dir_all(&outside).unwrap();
        junction(&shop.join("leak"), &outside);

        let mut aliases = BTreeMap::new();
        aliases.insert("/media".to_owned(), "storage/uploads".to_owned());
        aliases.insert("/shared".to_owned(), "@common/assets".to_owned());
        aliases.insert("/root".to_owned(), ".".to_owned());
        aliases.insert("/leak".to_owned(), "leak".to_owned());
        aliases.insert("/missing".to_owned(), "nope".to_owned());
        aliases.insert("/ghost".to_owned(), "@ghost/x".to_owned());
        aliases.insert("/up".to_owned(), "../common/assets".to_owned());
        aliases.insert("bad".to_owned(), "storage".to_owned());

        let found = resolve(&shop, &projects, &aliases);
        let urls: Vec<&str> = found.iter().map(|a| a.url.as_str()).collect();
        assert_eq!(urls, ["/media", "/root", "/shared"]);
        assert_eq!(
            skipped(&shop, &projects, &aliases),
            ["/ghost", "/leak", "/missing", "/up", "bad"]
        );
        let by_url = |u: &str| found.iter().find(|a| a.url == u).unwrap();
        assert_eq!(by_url("/media").dir, shop.join(r"storage\uploads"));
        assert_eq!(
            by_url("/media").apache_path("${CP_PROJECT_FWD}"),
            "${CP_PROJECT_FWD}/storage/uploads"
        );
        assert_eq!(
            by_url("/shared").apache_path("${CP_PROJECT_FWD}"),
            "${CP_PROJECTS_FWD}/common/assets"
        );
        assert_eq!(
            by_url("/root").apache_path("${CP_PROJECT_FWD}"),
            "${CP_PROJECT_FWD}"
        );

        // a project that is itself a junction is not followed either
        junction(&projects.join("linked"), &outside);
        let mut via = BTreeMap::new();
        via.insert("/x".to_owned(), "@linked/.".to_owned());
        assert!(resolve(&shop, &projects, &via).is_empty());
    }
}
