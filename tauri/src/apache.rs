//! A project's own Apache (user request, 14/10/2026): the kit's httpd with a
//! configuration of the project, written again at every start in
//! `data\app\projects\<project>\apache\` (never in the project, which holds
//! no path of the kit): the action's port, the folder as DocumentRoot with
//! `.htaccess` on, the kit's PHP as Apache module, its own logs and pid file.
//! The kit's site (`data\apache\kit.conf`, port 8080) is left out, so the
//! kit's Apache and those of the projects run side by side. Directives of
//! the user: `.conf` files in `conf.d\` next to it, which the kit never
//! touches.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

use crate::env::KitEnv;
use crate::fsx::write_atomic;
use crate::jobs;
use crate::kit::Kit;

/// `C:\a\b` as Apache wants it: `C:/a/b`.
fn fwd(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

/// PHP in `kit.conf` of an older kit, `${CP_APPS_FWD}/php/`: its folder now
/// holds one folder per version (versions.rs), the default is
/// `${CP_PHP_FWD}`. The only change the kit makes to that file.
pub fn fix_kit_conf(kit: &Kit) -> Result<()> {
    let conf = kit.data().join(r"apache\kit.conf");
    let Ok(text) = fs::read_to_string(&conf) else {
        return Ok(());
    };
    let fixed = text.replace("${CP_APPS_FWD}/php/", "${CP_PHP_FWD}/");
    if fixed != text {
        write_atomic(&conf, fixed.as_bytes()).with_context(|| conf.display().to_string())?;
    }
    Ok(())
}

/// The project's configuration: the kit's httpd.conf (modules, MIME types)
/// without its Dev-Kit tail, then the project's site.
pub fn project_conf(base: &str, root: &Path, dir: &Path, port: u16, lan: bool) -> Result<String> {
    if !base.contains("Define SRVROOT") {
        bail!("httpd.conf in an unexpected format (no Define SRVROOT)");
    }
    let kit_part = base
        .lines()
        .filter(|l| !l.contains("# --- Dev-Kit ---") && !l.contains("/data/apache/kit.conf"))
        .collect::<Vec<_>>()
        .join("\r\n");
    let (root, dir) = (fwd(root), fwd(dir));
    let listen = if lan {
        port.to_string()
    } else {
        format!("127.0.0.1:{port}")
    };
    let site = [
        "# --- Dev-Kit: this project's Apache (written at every start) ---".to_owned(),
        format!("Listen {listen}"),
        format!("ServerName localhost:{port}"),
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
        "DirectoryIndex index.php index.html".to_owned(),
        "<IfFile \"${CP_PHP_FWD}/php8apache2_4.dll\">".to_owned(),
        "    LoadFile \"${CP_PHP_FWD}/php8ts.dll\"".to_owned(),
        "    LoadModule php_module \"${CP_PHP_FWD}/php8apache2_4.dll\"".to_owned(),
        "    PHPIniDir \"${CP_ROOT_FWD}/data/php\"".to_owned(),
        "    <FilesMatch \"\\.php$\">".to_owned(),
        "        SetHandler application/x-httpd-php".to_owned(),
        "    </FilesMatch>".to_owned(),
        "</IfFile>".to_owned(),
        format!("IncludeOptional \"{dir}/conf.d/*.conf\""),
    ]
    .join("\r\n");
    Ok(format!("{}\r\n\r\n{site}\r\n", kit_part.trim_end()))
}

/// The folder of the project's Apache files.
pub fn dir(kit: &Kit, project: &str) -> Result<PathBuf> {
    crate::project_settings::dir(kit, project)
        .map(|d| d.join("apache"))
        .ok_or_else(|| anyhow!("not a project name: {project}"))
}

/// `httpd.exe -f <project conf>`, the configuration written for `root`.
pub fn command(
    kit: &Kit,
    project: &str,
    root: &Path,
    port: u16,
    lan: bool,
    env: &KitEnv,
) -> Result<Command> {
    let apache = kit.apps().join("apache");
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
        project_conf(&base, root, &dir, port, lan)?.as_bytes(),
    )?;
    let mut cmd = jobs::kit_command(&exe, env);
    cmd.arg("-f").arg(&conf);
    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conf_of_a_project() {
        let base = "Define SRVROOT \"${CP_APPS_FWD}/apache\"\r\nServerRoot \"${SRVROOT}\"\r\n\r\n# --- Dev-Kit ---\r\nInclude \"${CP_ROOT_FWD}/data/apache/kit.conf\"\r\n";
        let root = Path::new(r"D:\kit\projects\shop\public");
        let dir = Path::new(r"D:\kit\data\app\projects\shop\apache");
        let c = project_conf(base, root, dir, 7105, false).unwrap();
        assert!(c.starts_with("Define SRVROOT"));
        assert!(!c.contains("kit.conf"));
        assert!(c.contains("Listen 127.0.0.1:7105\r\n"));
        assert!(c.contains("DocumentRoot \"D:/kit/projects/shop/public\""));
        assert!(c.contains("PidFile \"D:/kit/data/app/projects/shop/apache/httpd.pid\""));
        assert!(
            c.contains("IncludeOptional \"D:/kit/data/app/projects/shop/apache/conf.d/*.conf\"")
        );
        let lan = project_conf(base, root, dir, 7105, true).unwrap();
        assert!(lan.contains("Listen 7105\r\n"));
        assert!(project_conf("Listen 80", root, dir, 7105, false).is_err());
    }
}
