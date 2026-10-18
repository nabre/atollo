//! Caddy, the kit's web server on every system (Q-3, D-111, WP-3.3): a
//! project's own Caddy, with a `Caddyfile` written at every start in
//! `data\projects\<project>\caddy\` (never in the project), and the text of
//! the shared pools' `Caddyfile` (`servers/config.rs`).
//!
//! Caddy reads no `.htaccess`: the site gets the rules Nginx's site has
//! (`projects/nginx.rs`), front controller (`index.php`, as WordPress,
//! Drupal and Laravel want it), hidden files and `.inc`/`.php3`…`.php8`
//! refused with 404, aliases checked on disk. PHP is the kit's FastCGI
//! server (`projects/fastcgi.rs`: php-cgi on Windows, php-fpm elsewhere),
//! sent the start's key with every request (`env DEVKIT_FCGI_KEY`).
//!
//! Every instance has its own admin endpoint on a free port of 127.0.0.1
//! (two pools never meet on Caddy's `:2019`), used by `caddy reload` to
//! change the sites without stopping the others; its data and config
//! folders are in the instance's folder, nothing goes to the user's
//! profile.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

use crate::env::KitEnv;
use crate::fsx::write_atomic;
use crate::kit::{Kit, Os};
use crate::os::jobs;
use crate::projects::alias::{self, look_inside, valid_domain, Inside, SiteAlias};
use crate::projects::apache::{fwd, PHP_NEVER, PHP_RUN};
use crate::projects::fastcgi::{FastCgi, FCGI_KEY_PARAM};

/// The server program of the Caddy in `dir` on `os`: `caddy.exe` on
/// Windows, `caddy` elsewhere, at the root of its folder (WP-3.1).
pub fn caddy_exe_for(os: Os, dir: &Path) -> PathBuf {
    dir.join(if os.is_windows() {
        "caddy.exe"
    } else {
        "caddy"
    })
}

/// [`caddy_exe_for`] on the kit's system.
pub fn caddy_exe(kit: &Kit, dir: &Path) -> PathBuf {
    caddy_exe_for(kit.os(), dir)
}

/// The Caddy folder from `env` (`CP_CADDY`, a version chosen for the
/// project) or the kit's default line.
pub fn caddy_dir(kit: &Kit, env: &KitEnv) -> PathBuf {
    env.get("CP_CADDY")
        .map(PathBuf::from)
        .filter(|d| caddy_exe(kit, d).is_file())
        .unwrap_or_else(|| {
            let resolved = crate::setup::catalog::kit_path(kit, "caddy");
            if caddy_exe(kit, &resolved).is_file() {
                resolved
            } else {
                kit.apps().join("caddy")
            }
        })
}

/// The folder of the project's Caddy files.
pub fn dir(kit: &Kit, project: &str) -> Result<PathBuf> {
    crate::projects::state::ProjectState::of(kit, project)
        .map(|s| s.caddy_dir())
        .ok_or_else(|| anyhow!("not a project name: {project}"))
}

/// A path as a Caddyfile token: with `/`, in double quotes. `None` when it
/// holds a character Caddy would read as something else (`"`, `\`, `{` and
/// `}` of a placeholder, a line break): such a folder is not served.
pub fn quoted_path(path: &Path) -> Option<String> {
    let text = fwd(path);
    if text.is_empty() || text.contains(['"', '\\', '{', '}', '\n', '\r', '`']) {
        return None;
    }
    Some(format!("\"{text}\""))
}

/// A free port of 127.0.0.1 for an instance's admin endpoint, chosen as
/// the kit's other ports (the system's, let go before Caddy takes it).
pub fn free_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .context("no free port for Caddy's admin endpoint")?;
    Ok(listener.local_addr()?.port())
}

/// The global options of an instance whose folder is `run_dir`: its own
/// admin endpoint (none with `admin` `None`, a file written only for the
/// next start), no automatic HTTPS, nothing saved outside the
/// configuration, its storage in its folder.
pub fn global_options(admin: Option<u16>, run_dir: &Path) -> Result<Vec<String>> {
    let storage = quoted_path(&run_dir.join("storage"))
        .ok_or_else(|| anyhow!("{}: not usable in a Caddyfile", run_dir.display()))?;
    Ok(vec![
        "{".to_owned(),
        match admin {
            Some(port) => format!("\tadmin 127.0.0.1:{port}"),
            None => "\tadmin off".to_owned(),
        },
        "\tauto_https off".to_owned(),
        "\tpersist_config off".to_owned(),
        format!("\tstorage file_system {storage}"),
        "}".to_owned(),
    ])
}

/// The PHP of a site (or of an alias) at `indent`: the FastCGI server with
/// the key of this start and the files PHP runs ([`PHP_RUN`]).
fn php_lines(fcgi: &FastCgi, indent: &str) -> Vec<String> {
    vec![
        format!("{indent}php_fastcgi 127.0.0.1:{} {{", fcgi.port),
        format!("{indent}\tenv {FCGI_KEY_PARAM} {}", fcgi.key),
        format!("{indent}\tsplit .php .phtml .phar"),
        format!("{indent}}}"),
    ]
}

/// The routes of a site, in the order they run (`route`): hidden files and
/// folders (`.env`, `.git/`) but `.well-known`, and the files of
/// [`PHP_NEVER`], refused with 404 everywhere, aliases included; without
/// PHP the files of [`PHP_RUN`] refused with 503 (never sent as text); the
/// aliases (their folders listed, PHP run in them as Apache does); then
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
        if let Some(f) = fcgi {
            lines.extend(php_lines(f, &inner));
        }
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
/// 421, as from a project's own Nginx (D-98). Caddy tries this block last.
fn refuse_other_hosts(port: u16) -> Vec<String> {
    site_block(port, false, &[], &["\trespond 421".to_owned()])
}

/// The local `Host` names a project's own Caddy answers besides
/// `localhost` and the project's domains out of LAN mode (SEC-V3-03).
const LOCAL_HOSTS: [&str; 3] = ["127.0.0.1", "[::1]", "*.localhost"];

/// The names a site answers: `<project>.localhost` and its valid domains
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
        names.push(format!("{p}.localhost"));
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

/// The complete `Caddyfile` of a project's own Caddy: in LAN mode the site
/// answers any `Host` (an IP of the PC included), else only its names and
/// the local ones, every other `Host` refused with 421.
#[allow(clippy::too_many_arguments)]
pub fn project_conf(
    project: Option<&str>,
    doc_root: &Path,
    run_dir: &Path,
    port: u16,
    admin: Option<u16>,
    fcgi: Option<&FastCgi>,
    lan: bool,
    domains: &[String],
    aliases: &[SiteAlias],
) -> Result<String> {
    let routes = site_routes(doc_root, aliases, fcgi, "\t").ok_or_else(|| {
        anyhow!(
            "{}: this folder cannot be served by Caddy (its path holds \" \\ {{ or }})",
            doc_root.display()
        )
    })?;
    let mut lines =
        vec!["# --- Dev-Kit: this project's Caddy (written at every start) ---".to_owned()];
    lines.extend(global_options(admin, run_dir)?);
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
const POOL_LOCAL_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];

/// The `Caddyfile` of a shared pool: `sites` (blocks of [`site_block`]),
/// the pool's own page for its local names (any `Host` in LAN mode, an IP
/// of the PC included), any other `Host` refused with 421; the user's own
/// `conf.d\*.caddy` imported when there is any.
pub fn pool_conf(
    pool_dir: &Path,
    port: u16,
    admin: Option<u16>,
    lan: bool,
    sites: &[Vec<String>],
    user_snippets: bool,
) -> Result<String> {
    let mut lines =
        vec!["# --- Dev-Kit: shared Caddy pool (written at every start or reload) ---".to_owned()];
    lines.extend(global_options(admin, pool_dir)?);
    if user_snippets {
        if let Some(glob) = quoted_path(&pool_dir.join("conf.d").join("*.caddy")) {
            lines.push(format!("import {glob}"));
        }
    }
    for site in sites {
        lines.extend(site.iter().cloned());
    }
    let page = ["\trespond \"Dev-Kit: Caddy pool\" 200".to_owned()];
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

/// `caddy run` of the `Caddyfile` `conf`.
pub fn run_args(conf: &Path) -> Vec<OsString> {
    vec![
        "run".into(),
        "--config".into(),
        conf.as_os_str().to_owned(),
        "--adapter".into(),
        "caddyfile".into(),
    ]
}

/// `caddy reload` of `conf` on the instance whose admin endpoint is
/// 127.0.0.1:`admin`: the new sites are served, the others go on
/// without a pause.
pub fn reload_args(conf: &Path, admin: u16) -> Vec<OsString> {
    let mut args = run_args(conf);
    args[0] = "reload".into();
    args.push("--address".into());
    args.push(format!("127.0.0.1:{admin}").into());
    args
}

/// `caddy` with the kit's environment, its data and config folders in the
/// instance's folder `run_dir` (Caddy reads `XDG_DATA_HOME` and
/// `XDG_CONFIG_HOME` before the system's folders, on Windows too).
pub fn caddy_command(exe: &Path, run_dir: &Path, env: &KitEnv) -> Command {
    let mut cmd = jobs::kit_command(exe, env);
    cmd.env("XDG_DATA_HOME", run_dir.join("data"))
        .env("XDG_CONFIG_HOME", run_dir.join("config"));
    cmd
}

/// Prepares the project's `Caddyfile` and returns the `Command` that starts
/// its Caddy.
#[allow(clippy::too_many_arguments)]
pub fn command(
    kit: &Kit,
    project: &str,
    project_root: &Path,
    doc_root: &Path,
    port: u16,
    fcgi: Option<&FastCgi>,
    lan: bool,
    domains: &[String],
    aliases: &BTreeMap<String, String>,
    env: &KitEnv,
) -> Result<Command> {
    let caddy = caddy_dir(kit, env);
    let exe = caddy_exe(kit, &caddy);
    if !exe.is_file() {
        bail!("Caddy is not installed (Strumenti > Caddy)");
    }
    let run_dir = dir(kit, project)?;
    fs::create_dir_all(&run_dir)?;
    let conf = run_dir.join("Caddyfile");
    let content = project_conf(
        Some(project),
        doc_root,
        &run_dir,
        port,
        Some(free_port()?),
        fcgi,
        lan,
        domains,
        &resolve_aliases(project_root, &kit.projects(), aliases),
    )?;
    write_atomic(&conf, content.as_bytes())?;
    let mut cmd = caddy_command(&exe, &run_dir, env);
    cmd.args(run_args(&conf));
    Ok(cmd)
}

#[cfg(test)]
#[path = "../../../test/rust/projects/caddy.rs"]
mod tests;
