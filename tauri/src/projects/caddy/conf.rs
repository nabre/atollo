//! The text of the Caddyfiles: a project's own Caddy and the shared pools'
//! (`servers/config.rs`), with the sites' routes, names and aliases.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::projects::alias::{self, look_inside, valid_domain, Inside, SiteAlias};
use crate::projects::apache::{fwd, ALIAS_NO_RUN, PHP_NEVER, PHP_RUN};
use crate::projects::fastcgi::{FastCgi, FCGI_KEY_PARAM};
use crate::servers::host_label;

use super::{quoted_path, socket_path_usable};

/// The address of the admin socket `path` for Caddy: `unix/<path>`, with
/// `/` (`unix//tmp/…` on Unix, `unix/C:/…` on Windows).
pub fn admin_address(path: &Path) -> String {
    format!("unix/{}", fwd(path))
}

/// The global options of an instance whose folder is `run_dir`: its admin
/// endpoint on the socket `admin` (owner-only), else none (`admin off`: a
/// project's own Caddy, which restarts with its action, or a file written
/// only for the next start); no automatic HTTPS, nothing saved outside the
/// configuration, its storage in its folder.
pub fn global_options(admin: Option<&Path>, run_dir: &Path) -> Result<Vec<String>> {
    let storage = quoted_path(&run_dir.join("storage"))
        .ok_or_else(|| anyhow!("{}: not usable in a Caddyfile", run_dir.display()))?;
    let admin = admin.filter(|p| socket_path_usable(p));
    Ok(vec![
        "{".to_owned(),
        match admin {
            Some(path) => format!("\tadmin \"{}|0600\"", admin_address(path)),
            None => "\tadmin off".to_owned(),
        },
        "\tauto_https off".to_owned(),
        "\tpersist_config off".to_owned(),
        format!("\tstorage file_system {storage}"),
        "}".to_owned(),
    ])
}

/// The PHP of a site (or of an alias) at `indent`: the FastCGI server, the
/// key of this start and the files PHP runs ([`PHP_RUN`]). The key is not
/// in the text: Caddy reads it from its own environment at every request
/// (`{env.ATOLLO_FCGI_KEY}`, set by [`with_key`]), so neither the file on
/// disk nor the configuration Caddy holds contains it.
pub(super) fn php_lines(fcgi: &FastCgi, indent: &str) -> Vec<String> {
    vec![
        format!("{indent}php_fastcgi {} {{", fcgi.addr.caddy_upstream()),
        format!("{indent}\tenv {FCGI_KEY_PARAM} {{env.{FCGI_KEY_PARAM}}}"),
        format!("{indent}\tsplit .php .phtml .phar"),
        format!("{indent}}}"),
    ]
}

/// The routes of a site, in the order they run (`route`): hidden files and
/// folders (`.env`, `.git/`) but `.well-known`, and the files of
/// [`PHP_NEVER`], refused with 404 everywhere, aliases included; without
/// PHP the files of [`PHP_RUN`] refused with 503 (never sent as text); the
/// aliases (their folders listed, static files only: the files of
/// [`ALIAS_NO_RUN`] refused with 404, SEC-V4-01); then
/// the document root with PHP's front controller and the files. `None`
/// when the document root cannot be written in a Caddyfile; an alias that
/// cannot is left out (in the log).
pub fn site_routes(
    doc_root: &Path,
    aliases: &[SiteAlias],
    fcgi: Option<&FastCgi>,
    indent: &str,
) -> Option<Vec<String>> {
    let root = quoted_path(doc_root)?;
    let i = indent;
    let mut lines = vec![
        format!("{i}route {{"),
        format!("{i}\t@hidden {{"),
        format!("{i}\t\tpath_regexp /\\."),
        format!("{i}\t\tnot path /.well-known /.well-known/*"),
        format!("{i}\t}}"),
        format!("{i}\trespond @hidden 404"),
        format!("{i}\t@never path_regexp (?i){PHP_NEVER}"),
        format!("{i}\trespond @never 404"),
    ];
    if fcgi.is_none() {
        lines.extend([
            format!("{i}\t@php path_regexp (?i){PHP_RUN}"),
            format!("{i}\trespond @php 503"),
        ]);
    }
    let inner = format!("{i}\t\t");
    for a in aliases {
        let Some(target) = quoted_path(&a.dir) else {
            crate::log::warn(&format!(
                "alias {} not served by Caddy: {} cannot be written in a Caddyfile",
                a.url,
                a.dir.display()
            ));
            continue;
        };
        lines.push(format!("{i}\tredir {} {}/ 308", a.url, a.url));
        lines.push(format!("{i}\thandle_path {}/* {{", a.url));
        lines.push(format!("{inner}root * {target}"));
        // Static files only (SEC-V4-01): no PHP in an alias' folder.
        lines.push(format!("{inner}@script path_regexp (?i){ALIAS_NO_RUN}"));
        lines.push(format!("{inner}respond @script 404"));
        lines.push(format!("{inner}file_server browse"));
        lines.push(format!("{i}\t}}"));
    }
    lines.push(format!("{i}\troot * {root}"));
    if let Some(f) = fcgi {
        lines.extend(php_lines(f, &format!("{i}\t")));
    }
    lines.push(format!("{i}\tfile_server"));
    lines.push(format!("{i}}}"));
    Some(lines)
}

/// A site block: the addresses (`http://<name>:<port>`, or every `Host` with
/// `names` empty), on 127.0.0.1 only out of LAN mode, then `body`.
pub fn site_block(port: u16, lan: bool, names: &[String], body: &[String]) -> Vec<String> {
    let addresses = if names.is_empty() {
        format!("http://:{port}")
    } else {
        names
            .iter()
            .map(|n| format!("http://{n}:{port}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut lines = vec![format!("{addresses} {{")];
    if !lan {
        lines.push("\tbind 127.0.0.1".to_owned());
    }
    lines.extend(body.iter().cloned());
    lines.push("}".to_owned());
    lines
}

/// Against DNS rebinding (a page of another site that renames itself to
/// 127.0.0.1 and reads the project): a `Host` no site names, or none, gets
/// 421 (D-98). Caddy tries this block last.
pub(super) fn refuse_other_hosts(port: u16) -> Vec<String> {
    site_block(port, false, &[], &["\trespond 421".to_owned()])
}

/// The local `Host` names a project's own Caddy answers besides
/// `localhost` and the project's domains out of LAN mode (SEC-V3-03).
pub(super) const LOCAL_HOSTS: [&str; 3] = ["127.0.0.1", "[::1]", "*.localhost"];

/// The names a site answers: `<label>.localhost` (the project as a DNS
/// label, [`host_label`], like the pools: BUG-V6-08) and its valid domains
/// (lowercase, once each); a project's own Caddy (`dedicated`) also
/// `localhost` and, out of LAN mode, [`LOCAL_HOSTS`].
pub fn site_names(
    project: Option<&str>,
    domains: &[String],
    dedicated: bool,
    lan: bool,
) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    if let Some(p) = project.filter(|p| !p.is_empty()) {
        names.push(format!("{}.localhost", host_label(p)));
    }
    for d in domains {
        let d = d.trim().to_ascii_lowercase();
        if valid_domain(&d) && !names.contains(&d) {
            names.push(d);
        }
    }
    if dedicated || names.is_empty() {
        names.push("localhost".to_owned());
    }
    if dedicated && !lan {
        names.extend(LOCAL_HOSTS.iter().map(|h| (*h).to_owned()));
    }
    names
}

/// The complete `Caddyfile` of a project's own Caddy: no admin endpoint
/// (it restarts with its action, never reloads); in LAN mode the site
/// answers any `Host` (an IP of the PC included), else only its names and
/// the local ones, every other `Host` refused with 421; the `*.caddy` of
/// `conf_d` (the project's `caddy\conf.d`, when it has some) in the site,
/// as in a pool (BUG-V5-06).
#[allow(clippy::too_many_arguments)]
pub fn project_conf(
    project: Option<&str>,
    doc_root: &Path,
    run_dir: &Path,
    port: u16,
    fcgi: Option<&FastCgi>,
    lan: bool,
    domains: &[String],
    aliases: &[SiteAlias],
    conf_d: Option<&Path>,
) -> Result<String> {
    let mut routes = site_routes(doc_root, aliases, fcgi, "\t").ok_or_else(|| {
        anyhow!(
            "{}: this folder cannot be served by Caddy (its path holds \" \\ {{ or }})",
            doc_root.display()
        )
    })?;
    if let Some(dir) = conf_d {
        match quoted_path(&dir.join("*.caddy")) {
            Some(glob) => routes.insert(0, format!("\timport {glob}")),
            None => crate::log::warn(&format!(
                "{}: not imported, it cannot be written in a Caddyfile",
                dir.display()
            )),
        }
    }
    let mut lines =
        vec!["# --- Atollo: this project's Caddy (written at every start) ---".to_owned()];
    lines.extend(global_options(None, run_dir)?);
    let names = if lan {
        Vec::new()
    } else {
        site_names(project, domains, true, lan)
    };
    lines.extend(site_block(port, lan, &names, &routes));
    if !lan {
        lines.extend(refuse_other_hosts(port));
    }
    lines.push(String::new());
    Ok(lines.join("\n"))
}

/// The local names, besides `localhost`, that reach a pool's own page out
/// of LAN mode (never `*.localhost`, which are the projects' sites).
pub(super) const POOL_LOCAL_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];

/// The `Caddyfile` of a shared pool: `sites` (blocks of [`site_block`]),
/// the pool's own page for its local names (any `Host` in LAN mode, an IP
/// of the PC included), any other `Host` refused with 421; the user's own
/// `conf.d\*.caddy` imported when there is any.
pub fn pool_conf(
    pool_dir: &Path,
    port: u16,
    admin: Option<&Path>,
    lan: bool,
    sites: &[Vec<String>],
    user_snippets: bool,
) -> Result<String> {
    let mut lines =
        vec!["# --- Atollo: shared Caddy pool (written at every start or reload) ---".to_owned()];
    lines.extend(global_options(admin, pool_dir)?);
    if user_snippets {
        if let Some(glob) = quoted_path(&pool_dir.join("conf.d").join("*.caddy")) {
            lines.push(format!("import {glob}"));
        }
    }
    for site in sites {
        lines.extend(site.iter().cloned());
    }
    let page = ["\trespond \"Atollo: Caddy pool\" 200".to_owned()];
    if lan {
        lines.extend(site_block(port, lan, &[], &page));
    } else {
        let names: Vec<String> = POOL_LOCAL_HOSTS.iter().map(|h| (*h).to_owned()).collect();
        lines.extend(site_block(port, lan, &names, &page));
        lines.extend(refuse_other_hosts(port));
    }
    lines.push(String::new());
    Ok(lines.join("\n"))
}

/// The aliases [`alias::resolve`] keeps that Caddy can serve: those whose
/// folder has no link or junction inside ([`look_inside`]), since Caddy's
/// file server follows them on every system; the others are left out with
/// a warning in the log.
pub fn resolve_aliases(
    project_root: &Path,
    projects_root: &Path,
    aliases: &BTreeMap<String, String>,
) -> Vec<SiteAlias> {
    alias::resolve(project_root, projects_root, aliases)
        .into_iter()
        .filter(|a| match look_inside(&a.dir) {
            Inside::Clean => true,
            Inside::Link(p) => {
                crate::log::warn(&format!(
                    "{}: alias {} not served by Caddy: {} is a link, a junction or unreadable (Caddy would follow it)",
                    project_root.display(),
                    a.url,
                    p.display()
                ));
                false
            }
            Inside::TooLarge => {
                crate::log::warn(&format!(
                    "{}: alias {} not served by Caddy: its folder is too large to check for links (over {} levels or {} entries)",
                    project_root.display(),
                    a.url,
                    alias::INSIDE_DEPTH,
                    alias::INSIDE_ENTRIES
                ));
                false
            }
        })
        .collect()
}

/// The URLs of the aliases [`resolve_aliases`] leaves out, for the
/// interface to say so.
pub fn skipped_aliases(
    project_root: &Path,
    projects_root: &Path,
    aliases: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut out = alias::skipped(project_root, projects_root, aliases);
    for (url, target) in aliases {
        let url = url.trim();
        if out.iter().any(|u| u == url) {
            continue;
        }
        let one = BTreeMap::from([(url.to_owned(), target.clone())]);
        let clean = alias::resolve(project_root, projects_root, &one)
            .iter()
            .all(|a| look_inside(&a.dir) == Inside::Clean);
        if !clean {
            out.push(url.to_owned());
        }
    }
    out
}
