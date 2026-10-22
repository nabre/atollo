//! «Esporta diagnostica» (N-1 of report v6/04): one zip the user attaches
//! to a report of a problem, made on the PC and sent nowhere by the app.
//!
//! `atollo-diagnostica-<yyyymmdd-hhmmss>.zip` in `data\app\diagnostics`:
//! - `summary.txt`: Atollo's version, the kit's system and the PC's, the
//!   kit's drive (file system, kind, free space, `noexec`);
//! - `env.txt`: the kit's environment, as `Atollo env` prints it;
//! - `check.txt`: every installed program's answer, as `Atollo check`;
//! - `components.txt`: the catalog with the choice and the versions;
//! - `ports.txt`: the ports Atollo keeps and who has each;
//! - `logs\*.log`: the last part of each log of `data\app\logs`.
//!
//! Before anything is written, every secret the kit keeps (the files of
//! `data\app\secrets`, Claude Code's token, the databases' passwords, the
//! agents' logins, GitHub's token) is replaced by `***` wherever it
//! appears, on top of the masking of secret-looking names
//! (`env::mask_secrets`); the real profile folder (the user's name)
//! becomes `~`, and the variables naming the user or the PC `***`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::env::KitEnv;
use crate::kit::Kit;

/// The part of each log that goes into the zip: its end.
const LOG_TAIL: usize = 256 * 1024;

/// A string of a file of secrets this short is not a secret (a version, a
/// name).
const MIN_SECRET: usize = 12;

/// Where the zips go.
pub fn dir(kit: &Kit) -> PathBuf {
    kit.app_dir().join("diagnostics")
}

/// The secrets the kit keeps, longest first (so a secret inside another is
/// replaced after it).
pub fn secrets(kit: &Kit, env: &KitEnv) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut add = |s: &str| {
        let s = s.trim();
        if s.len() >= 8 && !s.contains(char::is_whitespace) {
            out.push(s.to_owned());
        }
    };
    // Files that hold one secret each.
    let one = [
        crate::agents::claude_token::path(kit),
        crate::settings::gemini_file(kit),
    ];
    let secret_dir = kit.app_dir().join("secrets");
    let files = fs::read_dir(&secret_dir)
        .map(|d| d.flatten().map(|e| e.path()).collect::<Vec<_>>())
        .unwrap_or_default();
    for f in one.iter().chain(&files) {
        if let Ok(text) = fs::read_to_string(f) {
            add(&text);
        }
    }
    // Files of logins and passwords: every long string in them.
    let home = kit.home();
    let mut json = vec![
        crate::db::auth::dir(kit).join("secrets.json"),
        home.join(".claude").join(".credentials.json"),
        home.join(".codex").join("auth.json"),
        crate::agents::sync::pi::dir(&home).join("auth.json"),
        kit.data().join("antigravity").join("keyring.json"),
    ];
    if let Ok(projects) = fs::read_dir(kit.project_data()) {
        json.extend(
            projects
                .flatten()
                .map(|p| p.path().join("private").join("db.json")),
        );
    }
    for f in json {
        if let Some(v) = crate::fsx::read_json::<Value>(&f) {
            long_strings(&v, &mut |s| {
                if s.len() >= MIN_SECRET {
                    add(s);
                }
            });
        }
    }
    if let Some(token) = crate::github::token(kit, env) {
        add(&token);
    }
    out.sort_by_key(|s| std::cmp::Reverse(s.len()));
    out.dedup();
    out
}

fn long_strings(v: &Value, each: &mut dyn FnMut(&str)) {
    match v {
        Value::String(s) => each(s),
        Value::Array(a) => a.iter().for_each(|x| long_strings(x, each)),
        Value::Object(o) => o.values().for_each(|x| long_strings(x, each)),
        _ => {}
    }
}

/// `text` with the secret-looking names masked, every secret of `secrets`
/// replaced by `***`, the real profile folder by `~`.
pub fn clean(text: &str, secrets: &[String], profile: Option<&str>) -> String {
    let mut out = crate::env::mask_secrets(text);
    for s in secrets {
        if out.contains(s.as_str()) {
            out = out.replace(s.as_str(), "***");
        }
    }
    if let Some(p) = profile.filter(|p| p.len() > 3) {
        out = out.replace(p, "~");
        // As some programs write it: `/` in a Windows path, or without the
        // drive (`HOMEPATH=\Users\<name>`).
        let slashed = p.replace('\\', "/");
        out = out.replace(&slashed, "~");
        if let Some((_, rest)) = p.split_once(':').filter(|(d, _)| d.len() == 1) {
            if rest.len() > 3 {
                out = out.replace(rest, "~");
                out = out.replace(&rest.replace('\\', "/"), "~");
            }
        }
    }
    out
}

/// Variables that name the user or the PC: their value becomes `***` in
/// the environment's text.
const IDENTITY: [&str; 8] = [
    "USERNAME",
    "USER",
    "LOGNAME",
    "USERDOMAIN",
    "USERDOMAIN_ROAMINGPROFILE",
    "LOGONSERVER",
    "COMPUTERNAME",
    "HOSTNAME",
];

/// `KEY=VALUE` lines with the values of [`IDENTITY`] masked.
pub fn without_identity(env_text: &str) -> String {
    env_text
        .lines()
        .map(|l| match l.split_once('=') {
            Some((k, _)) if IDENTITY.iter().any(|i| k.eq_ignore_ascii_case(i)) => {
                format!("{k}=***\n")
            }
            _ => format!("{l}\n"),
        })
        .collect()
}

/// The last `max` bytes of `path`, from the start of a line.
fn tail(path: &Path, max: usize) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let start = bytes.len().saturating_sub(max);
    let text = String::from_utf8_lossy(&bytes[start..]).into_owned();
    Some(if start > 0 {
        text.split_once('\n')
            .map_or(text.clone(), |(_, rest)| rest.to_owned())
    } else {
        text
    })
}

fn summary(kit: &Kit) -> String {
    let mut s = String::new();
    let line = |s: &mut String, k: &str, v: String| s.push_str(&format!("{k:<14} {v}\n"));
    line(&mut s, "Atollo", env!("CARGO_PKG_VERSION").to_owned());
    line(&mut s, "Target", kit.target().as_str().to_owned());
    line(
        &mut s,
        "Host",
        crate::kit::Target::detect().as_str().to_owned(),
    );
    if let Some(build) = crate::os::windows_build() {
        line(&mut s, "Windows build", build.to_string());
    }
    if let Some(ram) = crate::os::memory::total_ram() {
        line(&mut s, "RAM", format!("{} MB", ram / (1024 * 1024)));
    }
    match kit.volume() {
        Some(v) => {
            line(&mut s, "Disk", v.fs.as_str().to_owned());
            line(&mut s, "Disk kind", format!("{:?}", v.drive).to_lowercase());
            line(
                &mut s,
                "Disk free",
                format!("{} MB", v.free / (1024 * 1024)),
            );
            line(&mut s, "Disk noexec", v.noexec.to_string());
        }
        None => line(&mut s, "Disk", "unknown".to_owned()),
    }
    line(&mut s, "Kit", kit.root().display().to_string());
    s
}

fn ports(kit: &Kit) -> String {
    crate::ports::ports_taken(kit)
        .iter()
        .filter(|(_, o)| !matches!(o, crate::ports::Owner::Editor))
        .map(|(p, o)| format!("{p:<6} {o:?}\n"))
        .collect()
}

/// `yyyymmdd-hhmmss` (UTC).
fn stamp() -> String {
    crate::agents::sync::stamp()
}

/// Makes the zip; its path.
pub fn export(kit: &Kit, env: &KitEnv, lang: &str) -> Result<PathBuf> {
    let mut secrets = secrets(kit, env);
    // The user's and the PC's names, wherever they appear (a log, a path).
    // The kit's environment has them, else the app's own.
    for name in IDENTITY {
        let value = env
            .get(name)
            .map(str::to_owned)
            .or_else(|| std::env::var(name).ok())
            .unwrap_or_default();
        let value = value.trim_start_matches('\\').to_owned();
        if value.len() >= 4 {
            secrets.push(value);
        }
    }
    secrets.sort_by_key(|s| std::cmp::Reverse(s.len()));
    let profile = env.get("CP_USER_PROFILE").map(str::to_owned);
    let clean = |t: &str| clean(t, &secrets, profile.as_deref());
    let mut entries: Vec<(String, String)> = vec![("summary.txt".into(), summary(kit))];
    let mut text = Vec::new();
    crate::cli::print_env(env, &mut text)?;
    entries.push((
        "env.txt".into(),
        without_identity(&String::from_utf8_lossy(&text)),
    ));
    let mut text = Vec::new();
    crate::cli::print_checks(&crate::setup::checkup::check_all(kit, env), &mut text)?;
    entries.push((
        "check.txt".into(),
        String::from_utf8_lossy(&text).into_owned(),
    ));
    let mut text = Vec::new();
    crate::cli::print_components(kit, lang, &mut text)?;
    entries.push((
        "components.txt".into(),
        String::from_utf8_lossy(&text).into_owned(),
    ));
    entries.push(("ports.txt".into(), ports(kit)));
    let mut logs: Vec<PathBuf> = fs::read_dir(kit.logs())
        .map(|d| {
            d.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "log"))
                .collect()
        })
        .unwrap_or_default();
    logs.sort();
    for log in logs {
        if let (Some(name), Some(text)) = (log.file_name(), tail(&log, LOG_TAIL)) {
            entries.push((format!("logs/{}", name.to_string_lossy()), text));
        }
    }
    let out = dir(kit);
    fs::create_dir_all(&out)?;
    let path = out.join(format!("atollo-diagnostica-{}.zip", stamp()));
    let file = fs::File::create(&path).with_context(|| path.display().to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (name, text) in entries {
        zip.start_file(name, options)?;
        zip.write_all(clean(&text).as_bytes())?;
    }
    zip.finish()?;
    crate::log::info(&format!("diagnostics: {}", path.display()));
    Ok(path)
}

/// «Segnala un problema» (N-2): the address of a new issue of Atollo with
/// the bug form (`.github/ISSUE_TEMPLATE/bug.yml`) and its version and
/// system fields filled in; nothing else of the PC goes into it, and
/// nothing is sent until the user presses «Create» on GitHub. Relative to
/// github.com.
pub fn issue_path(kit: &Kit) -> String {
    // The kit's system, and the PC's when it differs (x64 emulation).
    let (kit_target, host) = (kit.target(), crate::kit::Target::detect());
    let mut system = kit_target.as_str().to_owned();
    if host != kit_target {
        system.push_str(&format!(" on {}", host.as_str()));
    }
    if let Some(build) = crate::os::windows_build() {
        system.push_str(&format!(", Windows build {build}"));
    }
    if let Some(v) = kit.volume() {
        system.push_str(&format!(
            ", {} {}",
            v.fs.as_str(),
            format!("{:?}", v.drive).to_lowercase()
        ));
    }
    format!(
        "nabre/atollo/issues/new?template=bug.yml&version={}&system={}",
        encode(env!("CARGO_PKG_VERSION")),
        encode(&system)
    )
}

/// `text` for a query string.
fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
#[path = "../../test/rust/diagnostics.rs"]
mod tests;
