//! Final check of the installed components (port of nabre/atollo
//! `Test-Installation`, `_prepare-finish.ps1:157-217`): each program answers
//! its version command, with the kit environment and a time limit.

use std::io::Read;
use std::process::Stdio;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::catalog::{kit_path, Catalog, Component};
use crate::env::KitEnv;
use crate::jobs;
use crate::kit::Kit;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub id: String,
    pub name: String,
    /// First line of the answer, or why it failed.
    pub answer: String,
    pub ok: bool,
}

const LIMIT: Duration = Duration::from_secs(60);

/// As the only argument of a check: read the program's version resource
/// instead of starting it (a window program, such as Windows Terminal).
pub const FILE_VERSION: &str = "<file-version>";
/// As the only argument of a check: a PHP script (Adminer), compiled by the
/// kit's PHP (`php -l`); the answer is the `@version` of its header.
pub const PHP_SCRIPT: &str = "<php>";

/// Runs the check command of `c`; `None` if it has none or nothing of it is
/// on disk. Half installed (a part missing): failed, naming the part.
pub fn check(kit: &Kit, env: &KitEnv, c: &Component) -> Option<Check> {
    let (program, args) = c.check.split_first()?;
    if !c.is_present(kit) {
        return None;
    }
    let result = match c.missing_part(kit) {
        Some(part) => Err(anyhow::anyhow!("{} not found", part.display())),
        None if args == [FILE_VERSION] => file_version(&kit_path(kit, program)),
        None if args == [PHP_SCRIPT] => php_script(kit, env, &kit_path(kit, program)),
        None => run(env, &kit_path(kit, program), args),
    };
    Some(Check {
        id: c.id.clone(),
        name: c.name.clone(),
        ok: result.is_ok(),
        answer: result.unwrap_or_else(|e| format!("{e:#}")),
    })
}

/// Every installed component with a check, in catalog order.
pub fn check_all(kit: &Kit, env: &KitEnv) -> Vec<Check> {
    Catalog::get()
        .all()
        .iter()
        .filter_map(|c| check(kit, env, c))
        .collect()
}

/// The file version of `exe` (`a.b.c.d`), from its version resource.
fn file_version(exe: &std::path::Path) -> Result<String> {
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
    };
    if !exe.is_file() {
        bail!("{} not found", exe.display());
    }
    let name = HSTRING::from(exe.as_os_str());
    // SAFETY: the buffer has the size Windows asked for; the fixed info is
    // read only after VerQueryValueW points inside it with the right length.
    unsafe {
        let size = GetFileVersionInfoSizeW(PCWSTR(name.as_ptr()), None);
        if size == 0 {
            bail!("no version resource");
        }
        let mut data = vec![0u8; size as usize];
        GetFileVersionInfoW(PCWSTR(name.as_ptr()), None, size, data.as_mut_ptr().cast())
            .context("version resource")?;
        let mut info: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut len = 0u32;
        let root = HSTRING::from("\\");
        if !VerQueryValueW(
            data.as_ptr().cast(),
            PCWSTR(root.as_ptr()),
            &mut info,
            &mut len,
        )
        .as_bool()
            || info.is_null()
            || (len as usize) < std::mem::size_of::<VS_FIXEDFILEINFO>()
        {
            bail!("no version in the resource");
        }
        let f = &*(info as *const VS_FIXEDFILEINFO);
        Ok(format!(
            "{}.{}.{}.{}",
            f.dwFileVersionMS >> 16,
            f.dwFileVersionMS & 0xffff,
            f.dwFileVersionLS >> 16,
            f.dwFileVersionLS & 0xffff
        ))
    }
}

fn php_script(kit: &Kit, env: &KitEnv, script: &std::path::Path) -> Result<String> {
    if !script.is_file() {
        bail!("{} not found", script.display());
    }
    run(
        env,
        &kit_path(kit, "php/php.exe"),
        &["-l".to_owned(), script.display().to_string()],
    )?;
    let head = std::fs::read(script)?;
    let head = String::from_utf8_lossy(&head[..head.len().min(2048)]);
    Ok(header_version(&head).unwrap_or("compiled").to_owned())
}

/// `x.y.z` of the `@version x.y.z` line of a PHP header.
fn header_version(head: &str) -> Option<&str> {
    head.lines()
        .find_map(|l| l.split("@version").nth(1))
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

fn run(env: &KitEnv, exe: &std::path::Path, args: &[String]) -> Result<String> {
    if !exe.is_file() {
        bail!("{} not found", exe.display());
    }
    let is_batch = exe
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("bat") || e.eq_ignore_ascii_case("cmd"));
    let mut cmd = if is_batch {
        let words: Vec<&str> = args.iter().map(String::as_str).collect();
        jobs::batch_command(exe, &words, env)
    } else {
        let mut c = jobs::kit_command(exe, env);
        c.args(args);
        c
    };
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = jobs::spawn(&mut cmd).context("start")?;
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > LIMIT {
            let _ = child.kill();
            bail!("no answer within {} s", LIMIT.as_secs());
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let mut out = String::new();
    if let Some(mut s) = child.stdout.take() {
        let _ = s.read_to_string(&mut out);
    }
    if let Some(mut s) = child.stderr.take() {
        let _ = s.read_to_string(&mut out);
    }
    let first = out
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or_default()
        .to_owned();
    if !status.success() {
        bail!("exit {status}: {first}");
    }
    if first.is_empty() {
        bail!("empty answer");
    }
    Ok(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn every_check_names_an_installed_file() {
        for c in Catalog::get().all() {
            if let Some((program, _)) = c.check.split_first() {
                let top = program.split('/').next().unwrap_or_default();
                let owned = c.folder.as_deref() == Some(top)
                    || top == "tools"
                    || *program == c.key
                    || c.keys.contains(program);
                assert!(owned, "{}: check {program} outside its folder", c.id);
            }
        }
        for c in Catalog::get().all() {
            assert!(!c.check.is_empty(), "{}: no final check", c.id);
        }
    }

    #[test]
    fn php_header_version() {
        let head = "<?php\n/** Adminer\n* @license x\n* @version 6.1.2\n*/namespace\n";
        assert_eq!(header_version(head), Some("6.1.2"));
        assert_eq!(header_version("<?php echo 1;"), None);
    }

    /// A window program is checked by its version resource, not started.
    #[test]
    fn file_version_without_starting() {
        let system = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
        let notepad = std::path::Path::new(&system).join(r"System32\notepad.exe");
        let v = file_version(&notepad).unwrap();
        assert_eq!(v.split('.').count(), 4, "{v}");
        assert!(v.split('.').all(|n| n.parse::<u16>().is_ok()), "{v}");
        assert!(file_version(std::path::Path::new(r"C:\no\such.exe")).is_err());
        let wt = Catalog::get().find("terminal").unwrap();
        assert_eq!(wt.check[1..], [FILE_VERSION]);
    }

    /// `@` in the check program is the kit root, as in the key.
    #[test]
    fn check_program_at_the_kit_root() {
        let tmp = TempDir::new("checkup-root");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let pnpm = Catalog::get().find("pnpm").unwrap();
        for p in pnpm.key_paths(&kit) {
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, b"@echo 10.0.0\r\n").unwrap();
        }
        let result = check(&kit, &KitEnv::default(), pnpm).unwrap();
        assert!(result.ok, "{}", result.answer);
        assert_eq!(result.answer, "10.0.0");
    }

    /// Half installed: in the check, failed, with the missing part.
    #[test]
    fn half_installed_fails_the_check() {
        let tmp = TempDir::new("checkup-half");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let cmake = Catalog::get().find("cmake").unwrap();
        assert!(check(&kit, &KitEnv::default(), cmake).is_none(), "absent");
        let key = &cmake.key_paths(&kit)[0];
        std::fs::create_dir_all(key.parent().unwrap()).unwrap();
        std::fs::write(key, b"").unwrap();
        let result = check(&kit, &KitEnv::default(), cmake).unwrap();
        assert!(!result.ok);
        assert!(result.answer.contains("ninja.exe"), "{}", result.answer);
    }
}
