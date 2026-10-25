//! Caddy's command lines (run, reload, validate) and the process of a
//! project's own Caddy, with the start's FastCGI key in its environment.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Result};

use crate::common::fsx::write_atomic;
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::platform::jobs;
use crate::projects::fastcgi::{FastCgi, FCGI_KEY_PARAM};

use super::conf::{admin_address, project_conf, resolve_aliases};
use super::{caddy_dir, caddy_exe, dir, has_snippets};

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

/// `caddy reload` of `conf` on the instance whose admin socket is `admin`:
/// the new sites are served, the others go on without a pause.
pub fn reload_args(conf: &Path, admin: &Path) -> Vec<OsString> {
    let mut args = run_args(conf);
    args[0] = "reload".into();
    args.push("--address".into());
    args.push(admin_address(admin).into());
    args
}

/// `caddy validate` of the `Caddyfile` `conf`: adapted and provisioned,
/// nothing started. After a failed reload it tells a configuration Caddy
/// refuses from an instance that does not answer (BUG-V6-11).
pub fn validate_args(conf: &Path) -> Vec<OsString> {
    let mut args = run_args(conf);
    args[0] = "validate".into();
    args
}

/// The reason of a failed `caddy reload` from its `stderr`, for the log:
/// the last non-empty lines (Caddy ends with the error, an adapt error
/// names the Caddyfile's line), at most 2000 characters.
pub fn reload_failure(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.trim().is_empty())
        .collect();
    let tail = lines[lines.len().saturating_sub(5)..].join(" | ");
    if tail.is_empty() {
        return "no output".to_owned();
    }
    let mut chars = tail.chars();
    let short: String = chars.by_ref().take(2000).collect();
    if chars.next().is_some() {
        format!("{short}...")
    } else {
        short
    }
}

/// The key of `fcgi` in the environment of the Caddy that runs (never in
/// its configuration, [`php_lines`]): the only place it reads it from.
pub fn with_key(cmd: &mut Command, fcgi: Option<&FastCgi>) {
    if let Some(f) = fcgi {
        cmd.env(FCGI_KEY_PARAM, &f.key);
    }
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
/// its Caddy. `lang` is the user's, for the message of a missing Caddy,
/// which names a menu of the interface.
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
    lang: &str,
) -> Result<Command> {
    let caddy = caddy_dir(kit, env);
    let exe = caddy_exe(kit, &caddy);
    if !exe.is_file() {
        bail!(
            "{}",
            crate::common::i18n::t(lang, "{0} non è installato (Strumenti > {0})", &["Caddy"])
        );
    }
    let run_dir = dir(kit, project)?;
    // Where the user's own `.caddy` go, as `apache\conf.d` for Apache.
    let conf_d = run_dir.join("conf.d");
    fs::create_dir_all(&conf_d)?;
    let conf = run_dir.join("Caddyfile");
    let content = project_conf(
        Some(project),
        doc_root,
        &run_dir,
        port,
        fcgi,
        lan,
        domains,
        &resolve_aliases(project_root, &kit.projects(), aliases),
        Some(&conf_d)
            .filter(|d| has_snippets(d))
            .map(PathBuf::as_path),
    )?;
    write_atomic(&conf, content.as_bytes())?;
    let mut cmd = caddy_command(&exe, &run_dir, env);
    cmd.args(run_args(&conf));
    with_key(&mut cmd, fcgi);
    Ok(cmd)
}
