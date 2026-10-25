//! «Salute del kit» (N-3 of report v6/04): what of the PC and of the drive
//! may stop the kit, each with what to do, before it shows up as an
//! obscure error later. In Settings › Il kit and `Atollo check --kit`.
//!
//! - the drive: FAT32 (files of 4 GB at most, no links), exFAT, mounted
//!   `noexec` or `showexec` (`os::volume::notice_for`, D-105);
//! - the free space on it;
//! - characters in the kit's folder that some tools mishandle (`&`, `%`,
//!   `!`, `^`, quotes, letters outside ASCII: cmd, PowerShell, Maven,
//!   Gradle);
//! - files of the kit made private by another account, perhaps on another
//!   PC (PORT-V6-01): the kit cannot read them;
//! - on Linux, the system's libraries the kit's programs need: libaio and
//!   libnuma (MySQL), ICU (.NET), Vulkan (the GPU of llama.cpp).

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::common::i18n::t;
use crate::kit::{Kit, Os};
use crate::os::volume::FsKind;

/// How serious a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Ok,
    Warn,
    Err,
}

/// One thing checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub id: &'static str,
    pub level: Level,
    pub text: String,
    /// What to do, when there is something to do.
    pub fix: Option<String>,
}

fn finding(id: &'static str, level: Level, text: String, fix: Option<String>) -> Finding {
    Finding {
        id,
        level,
        text,
        fix,
    }
}

/// Free space below this: an error (installations and copies fail).
pub const LOW: u64 = 1 << 30;
/// Below this: a warning.
pub const LOWISH: u64 = 5 << 30;

/// The free space finding for `free` bytes.
pub fn space(free: u64, lang: &str) -> Finding {
    let gb = format!("{:.1}", free as f64 / (1u64 << 30) as f64);
    if free < LOW {
        finding(
            "space",
            Level::Err,
            t(lang, "Sul disco del kit restano {0} GB: installazioni, download e copie dei database non riescono.", &[&gb]),
            Some(t(lang, "Libera spazio sul disco, o sposta il kit su un disco più grande.", &[])),
        )
    } else if free < LOWISH {
        finding(
            "space",
            Level::Warn,
            t(lang, "Sul disco del kit restano {0} GB: bastano per poco (un modello locale ne pesa da 5 a 20).", &[&gb]),
            None,
        )
    } else {
        finding(
            "space",
            Level::Ok,
            t(lang, "Spazio libero: {0} GB.", &[&gb]),
            None,
        )
    }
}

/// The characters of `path` that some tools of the kit mishandle: `&`,
/// `%`, `!`, `^` (cmd), quotes (PowerShell's scripts), letters outside
/// ASCII (Maven, Gradle and older tools).
pub fn risky_chars(path: &str) -> Vec<char> {
    let mut out: Vec<char> = path
        .chars()
        .filter(|c| {
            matches!(
                c,
                '&' | '%' | '!' | '^' | '\'' | '"' | '‘' | '’' | '“' | '”' | '`'
            ) || !c.is_ascii()
        })
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

fn path_finding(root: &Path, lang: &str) -> Finding {
    let shown = root.display().to_string();
    let risky = risky_chars(&shown);
    if risky.is_empty() {
        return finding(
            "path",
            Level::Ok,
            t(lang, "Percorso del kit senza caratteri a rischio.", &[]),
            None,
        );
    }
    let list: String = risky.iter().map(|c| format!("«{c}» ")).collect();
    finding(
        "path",
        Level::Warn,
        t(
            lang,
            "Il percorso del kit contiene {0}: alcuni strumenti (cmd, PowerShell, Maven, Gradle) li gestiscono male.",
            &[list.trim_end()],
        ),
        Some(t(lang, "Sposta il kit in una cartella con solo lettere, cifre, trattini e spazi.", &[])),
    )
}

/// The drive's file system (and on macOS and Linux its mount).
fn disk_finding(kit: &Kit, lang: &str) -> Finding {
    if let Some(n) = crate::os::volume::disk_notice(kit, lang) {
        let level = if n.blocking { Level::Err } else { Level::Warn };
        return finding("disk", level, n.text, None);
    }
    let fs = kit.volume().map(|v| v.fs);
    match fs {
        Some(FsKind::Fat32) => finding(
            "disk",
            Level::Warn,
            t(lang, "Il disco del kit è FAT32: file fino a 4 GB (niente modelli locali grandi né copie di database grandi) e niente collegamenti, così le skill degli agenti si copiano.", &[]),
            Some(t(lang, "Per un kit su chiavetta è meglio exFAT, o NTFS se resta su Windows.", &[])),
        ),
        Some(FsKind::ExFat) => finding(
            "disk",
            Level::Ok,
            t(lang, "Il disco del kit è exFAT: va bene; mancano solo collegamenti e permessi, di cui il kit fa a meno.", &[]),
            None,
        ),
        Some(fs) => finding("disk", Level::Ok, t(lang, "Il disco del kit è {0}.", &[fs.as_str()]), None),
        None => finding("disk", Level::Warn, t(lang, "Il disco del kit non si riconosce.", &[]), None),
    }
}

/// Files under `dir` (at most `limit` of them looked at) that this account
/// cannot read.
pub fn unreadable(dir: &Path, limit: usize) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    let mut seen = 0;
    while let Some(d) = stack.pop() {
        let entries = match fs::read_dir(&d) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                out.push(d);
                continue;
            }
            Err(_) => continue,
        };
        for e in entries.flatten() {
            seen += 1;
            if seen > limit {
                return out;
            }
            let path = e.path();
            match e.file_type() {
                Ok(t) if t.is_dir() => stack.push(path),
                Ok(t) if t.is_file() => {
                    if let Err(err) = fs::File::open(&path) {
                        if err.kind() == std::io::ErrorKind::PermissionDenied {
                            out.push(path);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    out
}

fn owners_finding(kit: &Kit, lang: &str) -> Finding {
    // The kit's own files: settings, secrets, passwords, the agents'.
    let mut bad = unreadable(&kit.app_dir(), 5000);
    bad.extend(unreadable(&kit.home(), 20000));
    if bad.is_empty() {
        return finding(
            "owners",
            Level::Ok,
            t(lang, "I file del kit si leggono tutti.", &[]),
            None,
        );
    }
    let first = bad[0].display().to_string();
    let fix = owners_fix(
        Os::current().is_windows(),
        &kit.data().display().to_string(),
    );
    finding(
        "owners",
        Level::Err,
        t(
            lang,
            "{0} file del kit non si leggono, per esempio {1}: li ha resi privati un altro account, forse su un altro PC.",
            &[&bad.len().to_string(), &first],
        ),
        fix.map(|fix| t(lang, "Da amministratore, riprendili con: {0}", &[&fix])),
    )
}

/// The command that takes the kit's files back, to copy and paste: the path
/// is quoted so that no part of it runs. cmd expands `%` even in quotes, so
/// such a path gets no command (the path's check already warns about it).
pub(crate) fn owners_fix(windows: bool, data: &str) -> Option<String> {
    if windows {
        (!data.contains('%'))
            .then(|| format!("takeown /f \"{data}\" /r /d y && icacls \"{data}\" /reset /t"))
    } else {
        Some(format!(
            "sudo chown -R \"$(id -un)\" '{}'",
            data.replace('\'', r"'\''")
        ))
    }
}

/// On Linux, the libraries of the system the kit's programs need.
fn libs_finding(kit: &Kit, lang: &str) -> Option<Finding> {
    if !kit.os().is_linux() {
        return None;
    }
    let arch = kit.arch();
    let mut missing: Vec<&str> =
        crate::db::services::mysql::linux_server_libs(arch, |p: &Path| p.exists()).missing;
    let dirs = crate::env::linux_lib_dirs(kit.target().gnu_arch());
    if !crate::env::has_libicu(&dirs) {
        missing.push("libicu");
    }
    let vulkan = dirs.iter().any(|d| d.join("libvulkan.so.1").exists());
    if !vulkan {
        missing.push("libvulkan1");
    }
    Some(if missing.is_empty() {
        finding(
            "libs",
            Level::Ok,
            t(
                lang,
                "Le librerie di sistema che servono al kit ci sono.",
                &[],
            ),
            None,
        )
    } else {
        finding(
            "libs",
            Level::Warn,
            t(
                lang,
                "Mancano librerie di sistema: {0}. Senza, MySQL (libaio, libnuma), .NET (libicu) o la scheda video dei modelli locali (libvulkan1) non funzionano.",
                &[&missing.join(", ")],
            ),
            Some(t(lang, "Installale con il gestore dei pacchetti del sistema, per esempio: sudo apt install {0}", &[&missing.join(" ")])),
        )
    })
}

/// Every check, in this order: drive, space, path, owners, libraries.
pub fn check(kit: &Kit, lang: &str) -> Vec<Finding> {
    let mut out = vec![disk_finding(kit, lang)];
    if let Some(v) = kit.volume() {
        out.push(space(v.free, lang));
    }
    out.push(path_finding(kit.root(), lang));
    out.push(owners_finding(kit, lang));
    out.extend(libs_finding(kit, lang));
    out
}

#[cfg(test)]
#[path = "../../../test/rust/diagnostics/health.rs"]
mod tests;
