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
        let fixed = text.replace("${CP_APPS_FWD}/php/", "${CP_PHP_FWD}/");
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

/// Dynamic `<IfFile>` blocks for PHP 8, PHP 7, and PHP 5 Thread-Safe Apache modules.
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
            "    <FilesMatch \"\\.php$\">".to_owned(),
            "        SetHandler application/x-httpd-php".to_owned(),
            "    </FilesMatch>".to_owned(),
            "</IfFile>".to_owned(),
        ]);
    }
    out
}

/// Directives (`Alias` + `<Directory>`) for checked path aliases; `own` is
/// the expression of the project's folder (`${CP_PROJECT_FWD}`).
pub fn alias_directives(own: &str, aliases: &[SiteAlias], indent: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for alias in aliases {
        let target = alias.apache_path(own);
        lines.push(format!("{indent}Alias \"{}\" \"{target}\"", alias.url));
        lines.push(format!("{indent}<Directory \"{target}\">"));
        lines.push(format!("{indent}    Options Indexes FollowSymLinks"));
        lines.push(format!("{indent}    AllowOverride All"));
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
    if !server_aliases.is_empty() {
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
    site.push("DirectoryIndex index.php index.html".to_owned());
    site.extend(php_module_blocks("CP_PHP_FWD"));
    site.push(format!("IncludeOptional \"{dir}/conf.d/*.conf\""));
    let joined = site.join("\r\n");
    Ok(format!("{}\r\n\r\n{joined}\r\n", kit_part.trim_end()))
}

/// The folder of the project's Apache files.
pub fn dir(kit: &Kit, project: &str) -> Result<PathBuf> {
    crate::projects::state::ProjectState::of(kit, project)
        .map(|s| s.apache_dir())
        .ok_or_else(|| anyhow!("not a project name: {project}"))
}

/// Resolves the Apache installation directory from `env` (`CP_APACHE`) or the kit's default line.
pub fn apache_dir(kit: &Kit, env: &KitEnv) -> PathBuf {
    env.get("CP_APACHE")
        .map(PathBuf::from)
        .filter(|d| d.join(r"bin\httpd.exe").is_file())
        .unwrap_or_else(|| {
            let resolved = crate::setup::catalog::kit_path(kit, "apache");
            if resolved.join(r"bin\httpd.exe").is_file() {
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
    let exe = apache.join(r"bin\httpd.exe");
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
mod tests {
    use super::*;

    #[test]
    fn conf_of_a_project() {
        let base = "Define SRVROOT \"${CP_APACHE_FWD}\"\r\nServerRoot \"${SRVROOT}\"\r\n\r\n# --- Atollo ---\r\nInclude \"${CP_ROOT_FWD}/data/apache/kit.conf\"\r\n";
        let root = Path::new(r"D:\kit\projects\shop\public");
        let dir = Path::new(r"D:\kit\data\projects\shop\apache");
        let c = project_conf(base, root, dir, 7105, false).unwrap();
        assert!(c.starts_with("Define SRVROOT"));
        assert!(!c.contains("kit.conf"));
        assert!(c.contains("Listen 127.0.0.1:7105\r\n"));
        assert!(c.contains("DocumentRoot \"D:/kit/projects/shop/public\""));
        assert!(c.contains("PidFile \"D:/kit/data/projects/shop/apache/httpd.pid\""));
        assert!(c.contains("php8apache2_4.dll"));
        assert!(c.contains("php7apache2_4.dll"));
        assert!(c.contains("php5apache2_4.dll"));
        assert!(c.contains("IncludeOptional \"D:/kit/data/projects/shop/apache/conf.d/*.conf\""));
        let lan = project_conf(base, root, dir, 7105, true).unwrap();
        assert!(lan.contains("Listen 7105\r\n"));
        assert!(project_conf("Listen 80", root, dir, 7105, false).is_err());

        let tmp = crate::kit::tests::TempDir::new("apache-alias");
        let projects = tmp.0.join("projects");
        fs::create_dir_all(projects.join(r"shop\storage\media")).unwrap();
        fs::create_dir_all(projects.join(r"common\assets")).unwrap();
        let mut aliases = BTreeMap::new();
        aliases.insert("/media".into(), "storage/media".into());
        aliases.insert("/shared".into(), "@common/assets".into());
        aliases.insert("/up".into(), "../common/assets".into());
        let aliases = alias::resolve(&projects.join("shop"), &projects, &aliases);
        let with_site = project_conf_with_site(
            base,
            Some("shop"),
            root,
            dir,
            7105,
            false,
            &["api.shop.localhost".into(), "bad\"domain.localhost".into()],
            &aliases,
        )
        .unwrap();
        assert!(with_site.contains("ServerAlias shop.localhost api.shop.localhost\r\n"));
        assert!(with_site.contains("Alias \"/media\" \"${CP_PROJECT_FWD}/storage/media\""));
        assert!(with_site.contains("Alias \"/shared\" \"${CP_PROJECTS_FWD}/common/assets\""));
        assert!(!with_site.contains("/up") && !with_site.contains(".."));
    }
}
