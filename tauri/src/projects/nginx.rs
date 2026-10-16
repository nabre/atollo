//! A project's own Nginx: the kit's `nginx.exe` with a configuration written at
//! every start in `data\projects\<project>\nginx\` (never in the project),
//! plus `php-cgi.exe` on an local FastCGI port when PHP is installed.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use anyhow::{anyhow, bail, Result};

use crate::env::KitEnv;
use crate::fsx::write_atomic;
use crate::kit::Kit;
use crate::os::jobs::{self, Job};
use crate::projects::alias::{self, valid_domain, SiteAlias};
use crate::projects::apache::fwd;

/// Resolves the Nginx installation directory from `env` (`CP_NGINX`) or the kit's default line.
pub fn nginx_dir(kit: &Kit, env: &KitEnv) -> PathBuf {
    env.get("CP_NGINX")
        .map(PathBuf::from)
        .filter(|d| d.join("nginx.exe").is_file())
        .unwrap_or_else(|| {
            let resolved = crate::setup::catalog::kit_path(kit, "nginx");
            if resolved.join("nginx.exe").is_file() {
                resolved
            } else {
                kit.apps().join("nginx")
            }
        })
}

/// Resolves the PHP directory (`php-cgi.exe`) from `env` (`CP_PHP`) or the kit's default line.
pub fn php_dir(kit: &Kit, env: &KitEnv) -> Option<PathBuf> {
    let dir = env
        .get("CP_PHP")
        .map(PathBuf::from)
        .filter(|d| d.join("php-cgi.exe").is_file())
        .unwrap_or_else(|| crate::setup::catalog::kit_path(kit, "php"));
    dir.join("php-cgi.exe").is_file().then_some(dir)
}

/// The folder of the project's Nginx runtime files.
pub fn dir(kit: &Kit, project: &str) -> Result<PathBuf> {
    crate::projects::state::ProjectState::of(kit, project)
        .map(|s| s.nginx_dir())
        .ok_or_else(|| anyhow!("not a project name: {project}"))
}

/// Creates the required Nginx temporary directories inside `run_dir`.
pub fn prepare_dirs(run_dir: &Path) -> Result<()> {
    fs::create_dir_all(run_dir.join("conf.d"))?;
    for sub in ["client_body", "proxy", "fastcgi", "uwsgi", "scgi"] {
        fs::create_dir_all(run_dir.join("temp").join(sub))?;
    }
    Ok(())
}

/// Generates Nginx `location` blocks for checked path aliases.
pub fn alias_locations(aliases: &[SiteAlias], indent: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for alias in aliases {
        let mut target = fwd(&alias.dir);
        if !target.ends_with('/') {
            target.push('/');
        }
        lines.push(format!("{indent}location {}/ {{", alias.url));
        lines.push(format!("{indent}    alias \"{target}\";"));
        lines.push(format!("{indent}    autoindex on;"));
        lines.push(format!("{indent}}}"));
    }
    lines
}

/// Generates an Nginx `server { ... }` VirtualHost block.
#[allow(clippy::too_many_arguments)]
pub fn vhost_block(
    nginx_install: &Path,
    project: Option<&str>,
    doc_root: &Path,
    conf_d_dir: Option<&Path>,
    port: u16,
    fcgi_port: Option<u16>,
    lan: bool,
    domains: &[String],
    aliases: &[SiteAlias],
    default_server: bool,
) -> String {
    let listen = if lan {
        port.to_string()
    } else {
        format!("127.0.0.1:{port}")
    };
    let listen_suffix = if default_server {
        " default_server"
    } else {
        ""
    };
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
    if default_server || names.is_empty() {
        names.push("localhost".to_owned());
    }
    let root_fwd = fwd(doc_root);
    let nginx_fwd = fwd(nginx_install);

    let mut lines = vec![
        "    server {".to_owned(),
        format!("        listen {listen}{listen_suffix};"),
        format!("        server_name {};", names.join(" ")),
        format!("        root \"{root_fwd}\";"),
        "        index index.php index.html index.htm;".to_owned(),
        // Hidden files and folders (`.env`, `.git/`) are refused but
        // `.well-known`: a site served from the project's root would hand
        // them out. A regex location, before the others: it wins over the
        // aliases' prefix locations and over `.php`.
        "        location ~ /\\.(?!well-known(/|$)) {".to_owned(),
        "            deny all;".to_owned(),
        "        }".to_owned(),
    ];
    if fcgi_port.is_some() {
        lines.extend([
            "        location / {".to_owned(),
            "            try_files $uri $uri/ /index.php?$query_string;".to_owned(),
            "        }".to_owned(),
        ]);
    } else {
        lines.extend([
            "        location / {".to_owned(),
            "            try_files $uri $uri/ =404;".to_owned(),
            "        }".to_owned(),
        ]);
    }
    lines.extend(alias_locations(aliases, "        "));
    if let Some(fp) = fcgi_port {
        // `~*`: on NTFS `/INDEX.PHP` is the same file, and must be run,
        // never served as text by `location /`.
        lines.extend([
            "        location ~* \\.php$ {".to_owned(),
            "            try_files $uri =404;".to_owned(),
            format!("            fastcgi_pass 127.0.0.1:{fp};"),
            "            fastcgi_index index.php;".to_owned(),
            format!("            include \"{nginx_fwd}/conf/fastcgi_params\";"),
            "            fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;"
                .to_owned(),
            "        }".to_owned(),
        ]);
    } else {
        // Without PHP-CGI a `.php` would be sent as a download, source and
        // secrets included: refuse it (aliases too, regex wins over prefix).
        lines.extend([
            "        location ~* \\.php$ {".to_owned(),
            "            return 503;".to_owned(),
            "        }".to_owned(),
        ]);
    }
    if let Some(cd) = conf_d_dir {
        lines.push(format!("        include \"{}/conf.d/*.conf\";", fwd(cd)));
    }
    lines.push("    }".to_owned());
    lines.join("\r\n")
}

/// Complete `nginx.conf` for a standalone project Nginx instance.
#[allow(clippy::too_many_arguments)]
pub fn project_conf(
    nginx_install: &Path,
    project: Option<&str>,
    doc_root: &Path,
    run_dir: &Path,
    port: u16,
    fcgi_port: Option<u16>,
    lan: bool,
    domains: &[String],
    aliases: &[SiteAlias],
) -> String {
    let run_fwd = fwd(run_dir);
    let nginx_fwd = fwd(nginx_install);
    let server = vhost_block(
        nginx_install,
        project,
        doc_root,
        Some(run_dir),
        port,
        fcgi_port,
        lan,
        domains,
        aliases,
        true,
    );
    [
        "# --- Atollo: this project's Nginx (written at every start) ---".to_owned(),
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
        server,
        "}".to_owned(),
        String::new(),
    ]
    .join("\r\n")
}

/// Spawns `php-cgi.exe -b 127.0.0.1:<fcgi_port>` inside `job` when PHP is installed,
/// returning the FastCGI port used and the process, whose death the caller
/// watches (a Nginx without its php-cgi answers 502 to every `.php`).
/// `None` when PHP is not installed or the spawn failed (in the log).
pub fn spawn_php_cgi_if_present(kit: &Kit, env: &KitEnv, job: &Job) -> Option<(u16, Child)> {
    let php = php_dir(kit, env)?;
    let cgi = php.join("php-cgi.exe");
    let fcgi_port = match std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .and_then(|l| l.local_addr())
    {
        Ok(addr) => addr.port(),
        Err(e) => {
            crate::log::warn(&format!("php-cgi: no free FastCGI port: {e}"));
            return None;
        }
    };
    let mut cmd = jobs::kit_command(&cgi, env);
    cmd.arg("-b")
        .arg(format!("127.0.0.1:{fcgi_port}"))
        .arg("-c")
        .arg(kit.data().join("php"))
        .env("PHP_FCGI_MAX_REQUESTS", "0")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    match jobs::spawn_in(&mut cmd, job) {
        Ok(child) => Some((fcgi_port, child)),
        Err(e) => {
            crate::log::warn(&format!("php-cgi {}: {e:#}", cgi.display()));
            None
        }
    }
}

/// Prepares `nginx.conf` and returns the `Command` to start `nginx.exe`.
#[allow(clippy::too_many_arguments)]
pub fn command(
    kit: &Kit,
    project: &str,
    project_root: &Path,
    doc_root: &Path,
    port: u16,
    fcgi_port: Option<u16>,
    lan: bool,
    domains: &[String],
    aliases: &BTreeMap<String, String>,
    env: &KitEnv,
) -> Result<Command> {
    let nginx = nginx_dir(kit, env);
    let exe = nginx.join("nginx.exe");
    if !exe.is_file() {
        bail!("Nginx is not installed (Strumenti > Nginx)");
    }
    let run_dir = dir(kit, project)?;
    prepare_dirs(&run_dir)?;
    let conf = run_dir.join("nginx.conf");
    let content = project_conf(
        &nginx,
        Some(project),
        doc_root,
        &run_dir,
        port,
        fcgi_port,
        lan,
        domains,
        &alias::resolve(project_root, &kit.projects(), aliases),
    );
    write_atomic(&conf, content.as_bytes())?;
    let mut cmd = jobs::kit_command(&exe, env);
    cmd.arg("-p")
        .arg(&run_dir)
        .arg("-c")
        .arg(&conf)
        .arg("-g")
        .arg("daemon off;");
    Ok(cmd)
}

#[cfg(test)]
#[path = "../../../test/rust/projects/nginx.rs"]
mod tests;
