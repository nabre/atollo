//! The configuration a pool writes in `data\servers\<pool_id>\` at
//! every start or reload: Apache `httpd.conf` and VirtualHosts, Nginx
//! `nginx.conf` and `server` blocks, Caddy's `Caddyfile` with its sites.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::PoisonError;

use anyhow::{Context, Result};

use crate::detect::{self};
use crate::fsx::write_atomic;
use crate::kit::Kit;
use crate::projects::alias;
use crate::projects::apache::{self, fwd};
use crate::projects::caddy;
use crate::projects::nginx::{self, FastCgi};
use crate::setup::catalog::Catalog;

use super::pools::vhost_file;
use super::{serves, AttachedSite, WebServerEngine, WebServerPool, WebServers};

/// Resolves the directory of a versioned component (`apache`, `nginx`, `caddy`, `php`) for `line`.
pub(super) fn component_dir(kit: &Kit, id: &str, line: Option<&str>) -> PathBuf {
    if let Some(c) = Catalog::get().find(id) {
        if let Some(dir) = crate::versions::dir_for(kit, c, line) {
            return dir;
        }
    }
    kit.apps().join(id)
}

/// Generates Apache VirtualHost file content for one attached project in a pool.
pub(super) fn apache_vhost_conf(
    port: u16,
    project_root: &Path,
    projects_root: &Path,
    doc_root: &Path,
    site: &AttachedSite,
) -> String {
    let doc_fwd = fwd(doc_root);
    let mut aliases_list = vec![site.server_name.clone()];
    for d in &site.domains {
        let d = d.trim().to_ascii_lowercase();
        if alias::valid_domain(&d) && !aliases_list.contains(&d) {
            aliases_list.push(d);
        }
    }
    let mut lines = vec![
        format!("<VirtualHost *:{port}>"),
        format!("    ServerName {}", site.server_name),
    ];
    if aliases_list.len() > 1 {
        lines.push(format!("    ServerAlias {}", aliases_list[1..].join(" ")));
    }
    lines.extend([
        format!("    DocumentRoot \"{doc_fwd}\""),
        format!("    <Directory \"{doc_fwd}\">"),
        "        Options Indexes FollowSymLinks".to_owned(),
        "        AllowOverride All".to_owned(),
        "        Require all granted".to_owned(),
        "    </Directory>".to_owned(),
    ]);
    lines.extend(apache::hidden_deny("    "));
    lines.extend(apache::alias_directives(
        &format!("${{CP_PROJECTS_FWD}}/{}", site.project),
        &alias::resolve(project_root, projects_root, &site.aliases),
        "    ",
    ));
    lines.push("</VirtualHost>".to_owned());
    lines.push(String::new());
    lines.join("\r\n")
}

/// The local names, besides `localhost`, that reach a pool's own page out
/// of LAN mode (never `*.localhost`: Apache's first match would take the
/// projects' sites).
pub(super) const POOL_LOCAL_HOSTS: [&str; 2] = ["127.0.0.1", "[::1]"];

/// Generates the master `httpd.conf` for an Apache pool.
pub(super) fn apache_pool_conf(
    base: &str,
    pool_dir: &Path,
    port: u16,
    lan: bool,
) -> Result<String> {
    let kit_part = apache::strip_kit_tail(base)?;
    let dir_fwd = fwd(pool_dir);
    let listen = if lan {
        port.to_string()
    } else {
        format!("127.0.0.1:{port}")
    };
    let default_htdocs = "${SRVROOT}/htdocs";
    let mut site = vec![
        "# --- Atollo: shared Apache VirtualHost pool (written at every start) ---".to_owned(),
        format!("Listen {listen}"),
        format!("ServerName localhost:{port}"),
        format!("PidFile \"{dir_fwd}/httpd.pid\""),
        format!("ErrorLog \"{dir_fwd}/error.log\""),
        "<IfModule log_config_module>".to_owned(),
        format!("    CustomLog \"{dir_fwd}/access.log\" common"),
        "</IfModule>".to_owned(),
        format!("DocumentRoot \"{default_htdocs}\""),
        "DirectoryIndex index.php index.html".to_owned(),
    ];
    site.extend(apache::php_module_blocks("CP_PHP_FWD"));
    // The first VirtualHost answers any Host no other one names, never a
    // project. Out of LAN mode it refuses it with 403, as a project's own
    // Apache does (D-98: a page of another site re-pointed to 127.0.0.1,
    // DNS rebinding), and Apache's own page stays for the pool's local names.
    // In LAN mode (an IP of the PC as `Host`) it is Apache's own page.
    if !lan {
        site.extend([
            format!("<VirtualHost *:{port}>"),
            "    ServerName refused.invalid".to_owned(),
            "    <Location \"/\">".to_owned(),
            "        Require all denied".to_owned(),
            "    </Location>".to_owned(),
            "</VirtualHost>".to_owned(),
        ]);
    }
    site.push(format!("<VirtualHost *:{port}>"));
    site.push("    ServerName localhost".to_owned());
    if !lan {
        site.push(format!("    ServerAlias {}", POOL_LOCAL_HOSTS.join(" ")));
    }
    site.extend([
        format!("    DocumentRoot \"{default_htdocs}\""),
        "</VirtualHost>".to_owned(),
    ]);
    site.push(format!("IncludeOptional \"{dir_fwd}/vhosts/*.conf\""));
    site.push(format!("IncludeOptional \"{dir_fwd}/conf.d/*.conf\""));
    Ok(format!(
        "{}\r\n\r\n{}\r\n",
        kit_part.trim_end(),
        site.join("\r\n")
    ))
}

/// Generates the master `nginx.conf` for an Nginx pool.
pub(super) fn nginx_pool_conf(
    nginx_install: &Path,
    pool_dir: &Path,
    port: u16,
    lan: bool,
) -> String {
    let run_fwd = fwd(pool_dir);
    let nginx_fwd = fwd(nginx_install);
    let listen = if lan {
        port.to_string()
    } else {
        format!("127.0.0.1:{port}")
    };
    let mut lines = vec![
        "# --- Atollo: shared Nginx VirtualHost pool (written at every start) ---".to_owned(),
        "worker_processes 1;".to_owned(),
        format!("pid \"{run_fwd}/nginx.pid\";"),
        format!("error_log \"{run_fwd}/error.log\";"),
        "events {".to_owned(),
        "    worker_connections 1024;".to_owned(),
        "}".to_owned(),
        "http {".to_owned(),
        format!("    include \"{nginx_fwd}/conf/mime.types\";"),
        "    default_type application/octet-stream;".to_owned(),
        format!("    access_log \"{run_fwd}/access.log\";"),
        format!("    client_body_temp_path \"{run_fwd}/temp/client_body\";"),
        format!("    proxy_temp_path \"{run_fwd}/temp/proxy\";"),
        format!("    fastcgi_temp_path \"{run_fwd}/temp/fastcgi\";"),
        format!("    uwsgi_temp_path \"{run_fwd}/temp/uwsgi\";"),
        format!("    scgi_temp_path \"{run_fwd}/temp/scgi\";"),
        "    sendfile on;".to_owned(),
        "    keepalive_timeout 65;".to_owned(),
        nginx::SERVER_NAMES_HASH.to_owned(),
        format!("    include \"{run_fwd}/vhosts/*.conf\";"),
        format!("    include \"{run_fwd}/conf.d/*.conf\";"),
    ];
    if lan {
        // An IP of the PC as `Host`: Nginx's own page, never a project.
        lines.extend([
            "    server {".to_owned(),
            format!("        listen {listen} default_server;"),
            "        server_name localhost;".to_owned(),
            format!("        root \"{nginx_fwd}/html\";"),
            "        index index.html index.htm;".to_owned(),
            "    }".to_owned(),
        ]);
    } else {
        // Nginx's own page for the pool's local names; any other `Host`, or
        // none, gets 421 from the default server, as a project's own Nginx
        // (D-98: DNS rebinding).
        lines.extend([
            "    server {".to_owned(),
            format!("        listen {listen};"),
            format!(
                "        server_name localhost {};",
                POOL_LOCAL_HOSTS.join(" ")
            ),
            format!("        root \"{nginx_fwd}/html\";"),
            "        index index.html index.htm;".to_owned(),
            "    }".to_owned(),
            "    server {".to_owned(),
            format!("        listen {listen} default_server;"),
            "        server_name _;".to_owned(),
            "        return 421;".to_owned(),
            "    }".to_owned(),
        ]);
    }
    lines.extend(["}".to_owned(), String::new()]);
    lines.join("\r\n")
}

/// The document root of an attached site: its folder inside the project,
/// checked without following links (D-77). `None` (the site is not served,
/// with a warning in the log, as for a skipped alias) when the folder is
/// missing, outside the project or reached through a junction: never the
/// project root in its place, which would serve `.env`, `.git` and the like.
pub(super) fn site_doc_root(project_root: &Path, site: &AttachedSite) -> Option<PathBuf> {
    match detect::inside_dir(project_root, &site.folder) {
        Ok(dir) => Some(dir),
        Err(_) => {
            crate::log::warn(&format!(
                "{}: site folder {} not served (missing, not a folder of the project, or through a link)",
                project_root.display(),
                site.folder
            ));
            None
        }
    }
}

/// The site block of `site` in a Caddy pool on `port`: its own name and
/// its domains, the document root `doc_root`, its checked aliases, PHP
/// when the pool has it. `None` (in the log) when its folder cannot be
/// written in a Caddyfile.
pub(super) fn caddy_site_conf(
    port: u16,
    lan: bool,
    doc_root: &Path,
    aliases: &[alias::SiteAlias],
    fcgi: Option<&FastCgi>,
    site: &AttachedSite,
) -> Option<Vec<String>> {
    // The site's own ServerName first, then its domains (BUG-V3-02).
    let mut names = vec![site.server_name.clone()];
    for d in &site.domains {
        let d = d.trim().to_ascii_lowercase();
        if alias::valid_domain(&d) && !names.contains(&d) {
            names.push(d);
        }
    }
    let Some(routes) = caddy::site_routes(doc_root, aliases, fcgi, "\t") else {
        crate::log::warn(&format!(
            "{}: not served by Caddy, its folder cannot be written in a Caddyfile",
            doc_root.display()
        ));
        return None;
    };
    Some(caddy::site_block(port, lan, &names, &routes))
}

impl WebServers {
    /// Writes `httpd.conf` / `nginx.conf` and the `vhosts/*.conf` of the
    /// sites of `pool` it [`serves`], but those of the sites whose project
    /// was stopped ([`Self::detach`]).
    pub(super) fn write_pool_configs(
        &self,
        pool: &WebServerPool,
        fcgi: Option<&FastCgi>,
        lan: bool,
    ) -> Result<PathBuf> {
        self.write_pool_configs_with(pool, fcgi, lan, None)
    }

    /// [`Self::write_pool_configs`] with the admin socket of a running Caddy
    /// (`admin`; none for the other engines, or a file written for the
    /// next start).
    pub(super) fn write_pool_configs_with(
        &self,
        pool: &WebServerPool,
        fcgi: Option<&FastCgi>,
        lan: bool,
        admin: Option<&Path>,
    ) -> Result<PathBuf> {
        let detached = self
            .detached
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let started = self.started_sites();
        let served = pool.sites.iter().filter(|s| {
            serves(s, &started) && !detached.contains(&(s.project.clone(), s.action.clone()))
        });
        let pool_dir = self.kit.data().join("servers").join(&pool.id);
        let vhosts_dir = pool_dir.join("vhosts");
        let conf_d_dir = pool_dir.join("conf.d");
        fs::create_dir_all(&vhosts_dir)?;
        fs::create_dir_all(&conf_d_dir)?;

        // Clear old generated vhost files before writing current ones.
        if let Ok(entries) = fs::read_dir(&vhosts_dir) {
            for entry in entries.flatten() {
                if entry
                    .path()
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("conf"))
                {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }

        match pool.engine {
            WebServerEngine::Apache => {
                let apache_dir = component_dir(&self.kit, "apache", pool.server_line.as_deref());
                let base = fs::read_to_string(apache_dir.join("conf").join("httpd.conf"))
                    .context("Apache httpd.conf")?;
                for site in served {
                    let proj_root = self.kit.projects().join(&site.project);
                    let Some(doc_root) = site_doc_root(&proj_root, site) else {
                        continue;
                    };
                    let vhost = apache_vhost_conf(
                        pool.port,
                        &proj_root,
                        &self.kit.projects(),
                        &doc_root,
                        site,
                    );
                    write_atomic(&vhosts_dir.join(vhost_file(site)), vhost.as_bytes())?;
                }
                let main_conf = apache_pool_conf(&base, &pool_dir, pool.port, lan)?;
                let conf_path = pool_dir.join("httpd.conf");
                write_atomic(&conf_path, main_conf.as_bytes())?;
                Ok(conf_path)
            }
            WebServerEngine::Nginx => {
                nginx::prepare_dirs(&pool_dir)?;
                let nginx_dir = component_dir(&self.kit, "nginx", pool.server_line.as_deref());
                for site in served {
                    let proj_root = self.kit.projects().join(&site.project);
                    let Some(doc_root) = site_doc_root(&proj_root, site) else {
                        continue;
                    };
                    // The site's own ServerName first (never `<project>.localhost`
                    // for every action of the project: BUG-V3-02).
                    let names: Vec<String> = std::iter::once(site.server_name.clone())
                        .chain(site.domains.iter().cloned())
                        .collect();
                    let vhost = nginx::vhost_block(
                        &nginx_dir,
                        None,
                        &doc_root,
                        None,
                        pool.port,
                        fcgi,
                        lan,
                        &names,
                        &alias::resolve_nginx(&proj_root, &self.kit.projects(), &site.aliases),
                        false,
                    );
                    write_atomic(
                        &vhosts_dir.join(vhost_file(site)),
                        format!("{vhost}\r\n").as_bytes(),
                    )?;
                }
                let main_conf = nginx_pool_conf(&nginx_dir, &pool_dir, pool.port, lan);
                let conf_path = pool_dir.join("nginx.conf");
                write_atomic(&conf_path, main_conf.as_bytes())?;
                Ok(conf_path)
            }
            WebServerEngine::Caddy => {
                // One Caddyfile with every site in it: `caddy reload` takes
                // it whole, the sites that stay go on without a pause.
                let mut sites = Vec::new();
                for site in served {
                    let proj_root = self.kit.projects().join(&site.project);
                    let Some(doc_root) = site_doc_root(&proj_root, site) else {
                        continue;
                    };
                    let aliases =
                        caddy::resolve_aliases(&proj_root, &self.kit.projects(), &site.aliases);
                    sites.extend(caddy_site_conf(
                        pool.port, lan, &doc_root, &aliases, fcgi, site,
                    ));
                }
                let user_snippets = fs::read_dir(&conf_d_dir).is_ok_and(|entries| {
                    entries.flatten().any(|e| {
                        e.path()
                            .extension()
                            .is_some_and(|x| x.eq_ignore_ascii_case("caddy"))
                    })
                });
                let main_conf =
                    caddy::pool_conf(&pool_dir, pool.port, admin, lan, &sites, user_snippets)?;
                let conf_path = pool_dir.join("Caddyfile");
                write_atomic(&conf_path, main_conf.as_bytes())?;
                Ok(conf_path)
            }
        }
    }
}
