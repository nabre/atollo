//! configuration of the project, written again at every start in
//! `data\projects\<project>\apache\` (never in the project, which holds
//! no path of the kit): the action's port, the folder as DocumentRoot with
//! `.htaccess` on, the kit's PHP as Apache module, its own logs and pid file.
//! The kit's site (`data\apache\kit.conf`, port 8080) is left out, so the
//! kit's Apache and those of the projects run side by side. Directives of
//! the user: `.conf` files in `conf.d\` next to it, which the kit never
//! touches.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

use crate::env::KitEnv;
use crate::fsx::write_atomic;
use crate::kit::Kit;
use crate::os::jobs;
use crate::projects::alias::{self, valid_domain, SiteAlias};

/// `C:\a\b` as Apache wants it: `C:/a/b`.
pub fn fwd(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

/// PHP in `kit.conf` of an older kit, `${CP_APPS_FWD}/php/`: its folder now
/// holds one folder per version (versions/mod.rs), the default is
/// `${CP_PHP_FWD}`. Also updates migrated `httpd.conf` files from
/// `${CP_APPS_FWD}/apache` to `${CP_APACHE_FWD}`.
pub fn fix_kit_conf(kit: &Kit) -> Result<()> {
    let conf = kit.data().join(r"apache\kit.conf");
    if let Ok(text) = fs::read_to_string(&conf) {
        let fixed = migrate_kit_conf(&text);
        if fixed != text {
            write_atomic(&conf, fixed.as_bytes()).with_context(|| conf.display().to_string())?;
        }
    }
    let store = kit.apps().join("apache");
    let mut dirs = vec![store.clone()];
    if let Ok(entries) = fs::read_dir(&store) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                dirs.push(entry.path());
            }
        }
    }
    for dir in dirs {
        let httpd = dir.join(r"conf\httpd.conf");
        if let Ok(text) = fs::read_to_string(&httpd) {
            let fixed = text.replace(
                "Define SRVROOT \"${CP_APPS_FWD}/apache\"",
                "Define SRVROOT \"${CP_APACHE_FWD}\"",
            );
            if fixed != text {
                write_atomic(&httpd, fixed.as_bytes())
                    .with_context(|| httpd.display().to_string())?;
            }
        }
    }
    Ok(())
}

/// The files PHP runs, in any case (`INDEX.PHP` is PHP on Windows too):
/// `.php`, `.phtml` templates and `.phar` archives, as Debian's PHP module
/// does. Nginx uses the same pattern for its FastCGI location.
pub const PHP_RUN: &str = r"\.(?:php|phtml|phar)$";

/// Files never served, with or without PHP: `.inc` includes and the old
/// `.php3`…`.php8`, which PHP does not run here and which as text would
/// hand out source and secrets.
pub const PHP_NEVER: &str = r"\.(?:inc|php[3-8])$";

/// The text of an older `kit.conf` (created once, never rewritten) brought
/// up to date: PHP under `${CP_PHP_FWD}`, the PHP files ([`PHP_RUN`])
/// matched in any case and refused (403) without a PHP module, `.inc` and
/// the old `.php3`…`.php8` always refused, as [`php_module_blocks`] writes
/// them, and hidden files refused ([`hidden_deny`]). Idempotent.
pub fn migrate_kit_conf(text: &str) -> String {
    let mut fixed = text
        .replace("${CP_APPS_FWD}/php/", "${CP_PHP_FWD}/")
        .replace("\"\\.php$\"", "\"(?i)\\.php$\"")
        .replace("\"(?i)\\.php$\"", &format!("\"(?i){PHP_RUN}\""));
    let mut missing = Vec::new();
    if !fixed.contains("<IfModule !php_module>") {
        missing.extend(php_missing_block());
    }
    let never = php_never_block();
    if !fixed.contains(&never[0]) {
        missing.extend(never);
    }
    let hidden = hidden_deny("");
    if !fixed.contains(&hidden[0]) {
        missing.extend(hidden);
    }
    if !missing.is_empty() {
        let missing = missing.join("\r\n");
        // After the deny block, else after the last module block (`IfModule`
        // is read in order), or at the end when there is none.
        let deny_end = fixed.find("<IfModule !php_module>").and_then(|from| {
            fixed[from..]
                .find("\n</IfModule>")
                .map(|at| from + at + "\n</IfModule>".len())
        });
        let module_end = fixed.rfind("</IfFile>").map(|at| at + "</IfFile>".len());
        match deny_end.or(module_end) {
            Some(at) => {
                fixed.insert_str(at, &format!("\r\n{missing}"));
            }
            None => {
                if !fixed.is_empty() && !fixed.ends_with('\n') {
                    fixed.push_str("\r\n");
                }
                fixed.push_str(&format!("{missing}\r\n"));
            }
        }
    }
    fixed
}

/// `.inc` and the old `.php3`…`.php8` refused (403), PHP or not.
fn php_never_block() -> Vec<String> {
    vec![
        format!("<FilesMatch \"(?i){PHP_NEVER}\">"),
        "    Require all denied".to_owned(),
        "</FilesMatch>".to_owned(),
    ]
}

/// Without a PHP module the PHP files ([`PHP_RUN`]) are refused (403).
fn php_missing_block() -> Vec<String> {
    vec![
        "<IfModule !php_module>".to_owned(),
        "    <IfModule !php7_module>".to_owned(),
        "        <IfModule !php5_module>".to_owned(),
        format!("            <FilesMatch \"(?i){PHP_RUN}\">"),
        "                Require all denied".to_owned(),
        "            </FilesMatch>".to_owned(),
        "        </IfModule>".to_owned(),
        "    </IfModule>".to_owned(),
        "</IfModule>".to_owned(),
    ]
}

/// Dynamic `<IfFile>` blocks for PHP 8, PHP 7, and PHP 5 Thread-Safe Apache
/// modules, running the files of [`PHP_RUN`]. Without any of them those
/// would be sent as text, source and secrets included: they are refused
/// instead (403), as are always the files of [`PHP_NEVER`].
pub fn php_module_blocks(php_var: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (dll, ts, module) in [
        ("php8apache2_4.dll", "php8ts.dll", "php_module"),
        ("php7apache2_4.dll", "php7ts.dll", "php7_module"),
        ("php5apache2_4.dll", "php5ts.dll", "php5_module"),
    ] {
        out.extend([
            format!("<IfFile \"${{{php_var}}}/{dll}\">"),
            format!("    LoadFile \"${{{php_var}}}/{ts}\""),
            format!("    LoadModule {module} \"${{{php_var}}}/{dll}\""),
            "    PHPIniDir \"${CP_ROOT_FWD}/data/php\"".to_owned(),
            format!("    <FilesMatch \"(?i){PHP_RUN}\">"),
            "        SetHandler application/x-httpd-php".to_owned(),
            "    </FilesMatch>".to_owned(),
            "</IfFile>".to_owned(),
        ]);
    }
    out.extend(php_missing_block());
    out.extend(php_never_block());
    out
}

/// Refuses (403) hidden files and folders (`.env`, `.git/`, `.htpasswd`)
/// but `.well-known`: a site served from the project's root (WordPress
/// `.`) would otherwise hand them out. By URL (`LocationMatch`, aliases
/// included, never matching a dotted folder above the site on disk) and
/// by file name (`FilesMatch`).
pub fn hidden_deny(indent: &str) -> Vec<String> {
    vec![
        format!("{indent}<LocationMatch \"/\\.(?!well-known(/|$))\">"),
        format!("{indent}    Require all denied"),
        format!("{indent}</LocationMatch>"),
        format!("{indent}<FilesMatch \"^\\.(?!well-known$)\">"),
        format!("{indent}    Require all denied"),
        format!("{indent}</FilesMatch>"),
    ]
}

/// Directives (`Alias` + `<Directory>`) for checked path aliases; `own` is
/// the expression of the project's folder (`${CP_PROJECT_FWD}`). No link is
/// followed in their folders (WEB-8), and a `.htaccess` there cannot turn
/// `FollowSymLinks` back on: everything else of `AllowOverride All` stays.
pub fn alias_directives(own: &str, aliases: &[SiteAlias], indent: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for alias in aliases {
        let target = alias.apache_path(own);
        lines.push(format!("{indent}Alias \"{}\" \"{target}\"", alias.url));
        lines.push(format!("{indent}<Directory \"{target}\">"));
        lines.push(format!("{indent}    Options Indexes"));
        lines.push(format!(
            "{indent}    AllowOverride AuthConfig FileInfo Indexes Limit Options=Indexes,MultiViews"
        ));
        lines.push(format!("{indent}    Require all granted"));
        lines.push(format!("{indent}</Directory>"));
    }
    lines
}

/// Strips the Atollo tail from `conf\httpd.conf`.
pub fn strip_kit_tail(base: &str) -> Result<String> {
    if !base.contains("Define SRVROOT") {
        bail!("httpd.conf in an unexpected format (no Define SRVROOT)");
    }
    Ok(base
        .lines()
        .filter(|l| !l.contains("# --- Atollo ---") && !l.contains("/data/apache/kit.conf"))
        .collect::<Vec<_>>()
        .join("\r\n"))
}

/// The project's configuration: the kit's httpd.conf (modules, MIME types)
/// without its Atollo tail, then the project's site.
#[cfg(test)]
pub fn project_conf(base: &str, root: &Path, dir: &Path, port: u16, lan: bool) -> Result<String> {
    project_conf_with_site(base, None, root, dir, port, lan, &[], &[])
}

/// Like [`project_conf`], with optional project id (`.localhost` alias),
/// extra `.localhost` domains, and portable path aliases.
#[allow(clippy::too_many_arguments)]
pub fn project_conf_with_site(
    base: &str,
    project: Option<&str>,
    root: &Path,
    dir: &Path,
    port: u16,
    lan: bool,
    domains: &[String],
    aliases: &[SiteAlias],
) -> Result<String> {
    let kit_part = strip_kit_tail(base)?;
    let (root, dir) = (fwd(root), fwd(dir));
    let listen = if lan {
        port.to_string()
    } else {
        format!("127.0.0.1:{port}")
    };
    let mut server_aliases: Vec<String> = Vec::new();
    if let Some(p) = project.filter(|p| !p.is_empty()) {
        let def = format!("{p}.localhost");
        if !server_aliases.contains(&def) {
            server_aliases.push(def);
        }
    }
    for d in domains {
        let d = d.trim().to_ascii_lowercase();
        if valid_domain(&d) && !server_aliases.contains(&d) {
            server_aliases.push(d);
        }
    }

    let mut site = vec![
        "# --- Atollo: this project's Apache (written at every start) ---".to_owned(),
        format!("Listen {listen}"),
        format!("ServerName localhost:{port}"),
    ];
    if lan && !server_aliases.is_empty() {
        site.push(format!("ServerAlias {}", server_aliases.join(" ")));
    }
    site.extend([
        format!("PidFile \"{dir}/httpd.pid\""),
        format!("ErrorLog \"{dir}/error.log\""),
        "<IfModule log_config_module>".to_owned(),
        format!("    CustomLog \"{dir}/access.log\" common"),
        "</IfModule>".to_owned(),
        format!("DocumentRoot \"{root}\""),
        format!("<Directory \"{root}\">"),
        "    Options Indexes FollowSymLinks".to_owned(),
        "    AllowOverride All".to_owned(),
        "    Require all granted".to_owned(),
        "</Directory>".to_owned(),
    ]);
    site.extend(alias_directives("${CP_PROJECT_FWD}", aliases, ""));
    site.extend(hidden_deny(""));
    site.push("DirectoryIndex index.php index.html".to_owned());
    site.extend(php_module_blocks("CP_PHP_FWD"));
    if !lan {
        site.extend(local_hosts_only(port, &server_aliases));
    }
    site.push(format!("IncludeOptional \"{dir}/conf.d/*.conf\""));
    let joined = site.join("\r\n");
    Ok(format!("{}\r\n\r\n{joined}\r\n", kit_part.trim_end()))
}

/// Against DNS rebinding (a page of another site that renames itself to
/// 127.0.0.1 and reads the project): only a local `Host` is served,
/// `localhost`, `127.0.0.1`, `[::1]`, any `*.localhost` and the project's
/// own names. Any other `Host` (or none) gets the first VirtualHost, which
/// refuses everything (403); the site's directives stay in the main server,
/// which both inherit. Not in LAN mode, where an IP of the PC is the `Host`.
fn local_hosts_only(port: u16, server_aliases: &[String]) -> Vec<String> {
    let mut names = vec![
        "127.0.0.1".to_owned(),
        "[::1]".to_owned(),
        "*.localhost".to_owned(),
    ];
    names.extend(server_aliases.iter().cloned());
    vec![
        format!("<VirtualHost *:{port}>"),
        "    ServerName refused.invalid".to_owned(),
        "    <Location \"/\">".to_owned(),
        "        Require all denied".to_owned(),
        "    </Location>".to_owned(),
        "</VirtualHost>".to_owned(),
        format!("<VirtualHost *:{port}>"),
        "    ServerName localhost".to_owned(),
        format!("    ServerAlias {}", names.join(" ")),
        "</VirtualHost>".to_owned(),
    ]
}

/// The folder of the project's Apache files.
pub fn dir(kit: &Kit, project: &str) -> Result<PathBuf> {
    crate::projects::state::ProjectState::of(kit, project)
        .map(|s| s.apache_dir())
        .ok_or_else(|| anyhow!("not a project name: {project}"))
}

/// The server program of the Apache in `dir`: `bin\httpd.exe` on Windows,
/// `bin/httpd` on macOS and Linux (BUG-V3-08).
pub fn httpd_exe(kit: &Kit, dir: &Path) -> PathBuf {
    let name = if kit.os().is_windows() {
        "httpd.exe"
    } else {
        "httpd"
    };
    dir.join("bin").join(name)
}

/// Resolves the Apache installation directory from `env` (`CP_APACHE`) or the kit's default line.
pub fn apache_dir(kit: &Kit, env: &KitEnv) -> PathBuf {
    env.get("CP_APACHE")
        .map(PathBuf::from)
        .filter(|d| httpd_exe(kit, d).is_file())
        .unwrap_or_else(|| {
            let resolved = crate::setup::catalog::kit_path(kit, "apache");
            if httpd_exe(kit, &resolved).is_file() {
                resolved
            } else {
                kit.apps().join("apache")
            }
        })
}

/// `httpd.exe -f <project conf>`, the configuration written for `root`.
#[allow(clippy::too_many_arguments)]
pub fn command(
    kit: &Kit,
    project: &str,
    project_root: &Path,
    doc_root: &Path,
    port: u16,
    lan: bool,
    domains: &[String],
    aliases: &BTreeMap<String, String>,
    env: &KitEnv,
) -> Result<Command> {
    let apache = apache_dir(kit, env);
    let exe = httpd_exe(kit, &apache);
    if !exe.is_file() {
        bail!("Apache is not installed (Strumenti > Apache)");
    }
    let base = fs::read_to_string(apache.join(r"conf\httpd.conf")).context("httpd.conf")?;
    let dir = dir(kit, project)?;
    fs::create_dir_all(dir.join("conf.d"))?;
    let conf = dir.join("httpd.conf");
    write_atomic(
        &conf,
        project_conf_with_site(
            &base,
            Some(project),
            doc_root,
            &dir,
            port,
            lan,
            domains,
            &alias::resolve(project_root, &kit.projects(), aliases),
        )?
        .as_bytes(),
    )?;
    let mut cmd = jobs::kit_command(&exe, env);
    cmd.env("CP_APACHE", apache.display().to_string())
        .env("CP_APACHE_FWD", fwd(&apache))
        .env("CP_PROJECT_FWD", fwd(project_root))
        .env("CP_PROJECTS_FWD", fwd(&kit.projects()))
        .arg("-f")
        .arg(&conf);
    Ok(cmd)
}

#[cfg(test)]
#[path = "../../../test/rust/projects/apache.rs"]
mod tests;
