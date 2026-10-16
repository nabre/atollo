//! The kit's Windows Terminal (portable mode), where the consoles open by
//! default (DECISIONS.md D-68; the classic console is D-57).
//!
//! - Settings in `data\apps\<platform>\terminal\settings\settings.json`: the
//!   app owns the "Atollo" profiles and the VSCodium color schemes and
//!   rewrites them before each console; the rest of the file is the user's.
//! - Windows Terminal runs all its windows in one process and starts a new
//!   tab with its own environment, not the caller's. So each console is the
//!   kit's PowerShell running `console.ps1`, which gets the kit's
//!   environment through a named pipe of its own. The app checks that the
//!   other end is the kit's PowerShell, then waits for that process: it is
//!   the console, counted until it ends.

use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};
use windows::core::HSTRING;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, LocalFree, ERROR_PIPE_CONNECTED, HANDLE, HLOCAL,
};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
use windows::Win32::Storage::FileSystem::{
    FlushFileBuffers, WriteFile, FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_OUTBOUND,
};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, GetNamedPipeClientProcessId,
    PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_WAIT,
};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};

use super::cfg::pwsh_args;
use crate::env::KitEnv;
use crate::fsx::write_atomic;
use crate::kit::Kit;
use crate::os::jobs::{self, Console};

const PROFILE_PWSH: &str = "{6e0f3c1a-4b7d-4d39-9a41-5f2d0c6b7a01}";
const PROFILE_BASH: &str = "{6e0f3c1a-4b7d-4d39-9a41-5f2d0c6b7a02}";
const SCHEME_DARK: &str = "VSCodium Dark Modern";
const SCHEME_LIGHT: &str = "VSCodium Light Modern";
/// Windows Terminal needs Windows 10 2004 or newer.
const MIN_BUILD: u32 = 19041;
/// Protected DACL, full access for the owner only.
const PIPE_SDDL: &str = "D:P(A;;GA;;;OW)";
/// How long the new tab has to ask for the kit's environment.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(30);

fn exe(kit: &Kit) -> PathBuf {
    kit.apps().join(r"terminal\WindowsTerminal.exe")
}

fn settings_path(kit: &Kit) -> PathBuf {
    kit.apps().join(r"terminal\settings\settings.json")
}

fn pwsh(kit: &Kit) -> PathBuf {
    kit.apps().join(r"pwsh\pwsh.exe")
}

/// Installed, and Windows can run it.
pub fn usable(kit: &Kit) -> bool {
    exe(kit).is_file() && pwsh(kit).is_file() && windows_build().is_none_or(|b| b >= MIN_BUILD)
}

/// The build of Windows (`26200`); `None` when it cannot be read.
pub fn windows_build() -> Option<u32> {
    let key = HSTRING::from(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
    let value = HSTRING::from("CurrentBuildNumber");
    let mut buf = [0u16; 32];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: buffer and size describe the same memory; read-only query.
    let r = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            &key,
            &value,
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    r.is_ok().then_some(())?;
    String::from_utf16_lossy(&buf[..(size as usize / 2).saturating_sub(1)])
        .parse()
        .ok()
}

/// Terminal colors of VSCodium's "Default Dark Modern" / "Default Light
/// Modern" themes, so consoles look like the editor and the app.
fn schemes() -> Vec<Value> {
    vec![
        json!({
            "name": SCHEME_DARK,
            "background": "#181818", "foreground": "#CCCCCC", "cursorColor": "#CCCCCC", "selectionBackground": "#264F78",
            "black": "#000000", "red": "#CD3131", "green": "#0DBC79", "yellow": "#E5E510",
            "blue": "#2472C8", "purple": "#BC3FBC", "cyan": "#11A8CD", "white": "#E5E5E5",
            "brightBlack": "#666666", "brightRed": "#F14C4C", "brightGreen": "#23D18B", "brightYellow": "#F5F543",
            "brightBlue": "#3B8EEA", "brightPurple": "#D670D6", "brightCyan": "#29B8DB", "brightWhite": "#E5E5E5"
        }),
        json!({
            "name": SCHEME_LIGHT,
            "background": "#F8F8F8", "foreground": "#3B3B3B", "cursorColor": "#3B3B3B", "selectionBackground": "#ADD6FF",
            "black": "#000000", "red": "#CD3131", "green": "#107C10", "yellow": "#949800",
            "blue": "#0451A5", "purple": "#BC05BC", "cyan": "#0598BC", "white": "#555555",
            "brightBlack": "#666666", "brightRed": "#CD3131", "brightGreen": "#14CE14", "brightYellow": "#B5BA00",
            "brightBlue": "#0451A5", "brightPurple": "#BC05BC", "brightCyan": "#0598BC", "brightWhite": "#A5A5A5"
        }),
    ]
}

/// One argument of a profile's command line.
fn quote(arg: &str) -> String {
    if arg.is_empty() || arg.contains([' ', '\t', '"', '&', '(', ')', ';']) {
        format!("\"{}\"", arg.replace('"', "\\\""))
    } else {
        arg.to_owned()
    }
}

/// The kit's profiles, for the tabs the user opens with "+" (they get the
/// environment Windows Terminal started with, the kit's).
fn kit_profiles(kit: &Kit) -> Vec<Value> {
    let mut out = Vec::new();
    let pwsh = pwsh(kit);
    let colors = json!({ "dark": SCHEME_DARK, "light": SCHEME_LIGHT });
    if pwsh.is_file() {
        let line = std::iter::once(pwsh.display().to_string())
            .chain(pwsh_args(kit))
            .map(|a| quote(&a))
            .collect::<Vec<_>>()
            .join(" ");
        out.push(json!({
            "guid": PROFILE_PWSH, "name": "Atollo", "commandline": line,
            "icon": pwsh.display().to_string(), "startingDirectory": kit.projects().display().to_string(),
            "colorScheme": colors, "hidden": false
        }));
    }
    let bash = kit.apps().join(r"git\bin\bash.exe");
    if bash.is_file() {
        let rc = kit.kit_files().join("bashrc").display().to_string();
        out.push(json!({
            "guid": PROFILE_BASH, "name": "Atollo - Git Bash",
            "commandline": format!("{} --rcfile {} -i", quote(&bash.display().to_string()), quote(&rc)),
            "icon": kit.apps().join(r"git\git-bash.exe").display().to_string(),
            "startingDirectory": kit.projects().display().to_string(),
            "colorScheme": colors, "hidden": false
        }));
    }
    out
}

/// Merges the kit's part into the user's settings (parsed).
fn merge(existing: &Map<String, Value>, kit: &Kit) -> Map<String, Value> {
    let mut out = existing.clone();
    let ours = kit_profiles(kit);
    let our_guids = [PROFILE_PWSH, PROFILE_BASH];

    let mut profiles = match out.remove("profiles") {
        Some(Value::Object(p)) => p,
        Some(Value::Array(list)) => Map::from_iter([("list".to_owned(), Value::Array(list))]),
        _ => Map::new(),
    };
    let mut list: Vec<Value> = match profiles.remove("list") {
        Some(Value::Array(l)) => l,
        _ => Vec::new(),
    };
    list.retain(|p| {
        !p.get("guid")
            .and_then(Value::as_str)
            .is_some_and(|g| our_guids.contains(&g))
    });
    // PC shells (Windows PowerShell, cmd, Azure...) stay but are hidden.
    for p in &mut list {
        if p.get("source").is_some()
            || matches!(
                p.get("name").and_then(Value::as_str),
                Some("Windows PowerShell" | "Command Prompt" | "Prompt dei comandi")
            )
        {
            if let Some(obj) = p.as_object_mut() {
                obj.insert("hidden".into(), json!(true));
            }
        }
    }
    let mut new_list = ours.clone();
    new_list.extend(list);
    profiles.insert("list".into(), Value::Array(new_list));
    let defaults = profiles
        .entry("defaults")
        .or_insert_with(|| json!({ "font": { "face": "Cascadia Mono", "size": 11 } }));
    if let Some(d) = defaults.as_object_mut() {
        // Otherwise new tabs rebuild the environment from the PC's profile.
        d.insert(
            "compatibility.reloadEnvironmentVariables".into(),
            json!(false),
        );
    }
    out.insert("profiles".into(), Value::Object(profiles));
    out.remove("compatibility.reloadEnvironmentVariables");

    let mut schemes_list: Vec<Value> = match out.remove("schemes") {
        Some(Value::Array(l)) => l,
        _ => Vec::new(),
    };
    schemes_list.retain(|s| {
        !matches!(
            s.get("name").and_then(Value::as_str),
            Some(SCHEME_DARK | SCHEME_LIGHT)
        )
    });
    schemes_list.extend(schemes());
    out.insert("schemes".into(), Value::Array(schemes_list));

    if !ours.is_empty() {
        let default_ok = out
            .get("defaultProfile")
            .and_then(Value::as_str)
            .is_some_and(|d| our_guids.contains(&d));
        if !default_ok {
            out.insert("defaultProfile".into(), json!(PROFILE_PWSH));
        }
    }
    out.entry("theme").or_insert(json!("system"));
    out.entry("disabledProfileSources").or_insert(json!([
        "Windows.Terminal.Wsl",
        "Windows.Terminal.Azure",
        "Windows.Terminal.PowershellCore",
        "Windows.Terminal.VisualStudio"
    ]));
    out.insert(
        "$help".into(),
        json!(
            "Atollo rewrites the Atollo profiles and the VSCodium schemes before each console."
        ),
    );
    out
}

/// Writes the settings; a file that is not plain JSON, or cannot be read
/// (UTF-16, locked...), is left alone. Only a missing file is created.
fn write_settings(kit: &Kit) -> Result<()> {
    let path = settings_path(kit);
    let existing = match fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<Value>(crate::fsx::strip_bom(&text)) {
            Ok(Value::Object(map)) => map,
            _ => bail!("not plain JSON: {}", path.display()),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Map::new(),
        Err(e) => bail!("cannot read {}: {e}", path.display()),
    };
    let merged = merge(&existing, kit);
    if merged != existing {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        write_atomic(
            &path,
            serde_json::to_string_pretty(&Value::Object(merged))?.as_bytes(),
        )
        .with_context(|| path.display().to_string())?;
    }
    Ok(())
}

/// `;` separates Windows Terminal's subcommands, even after `--`.
fn wt_arg(arg: impl AsRef<std::ffi::OsStr>) -> String {
    arg.as_ref().to_string_lossy().replace(';', r"\;")
}

/// Server end of a console's pipe; closed on drop.
struct Pipe(HANDLE);

impl Drop for Pipe {
    fn drop(&mut self) {
        // SAFETY: handle from CreateNamedPipeW, closed once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

fn create_pipe(name: &str) -> Result<Pipe> {
    let path = HSTRING::from(format!(r"\\.\pipe\{name}"));
    // Owner only: the default security lets Everyone read, and the kit's
    // environment holds secrets (the kit's pwsh.exe, checked on connect, can
    // be started by anyone on a drive without permissions).
    let sddl = HSTRING::from(PIPE_SDDL);
    let mut sd = PSECURITY_DESCRIPTOR::default();
    // SAFETY: valid SDDL string; `sd` receives a LocalAlloc'd descriptor freed below.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(&sddl, SDDL_REVISION_1, &mut sd, None)
    }
    .context("security descriptor")?;
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: sd.0,
        bInheritHandle: false.into(),
    };
    // SAFETY: valid name and attributes; one outbound instance for this
    // machine only.
    let pipe = unsafe {
        CreateNamedPipeW(
            &path,
            PIPE_ACCESS_OUTBOUND | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            64 * 1024,
            0,
            0,
            Some(&attributes),
        )
    };
    // SAFETY: frees the descriptor allocated above, once, after its use.
    unsafe { LocalFree(Some(HLOCAL(sd.0))) };
    anyhow::ensure!(!pipe.is_invalid(), "CreateNamedPipeW {name}");
    Ok(Pipe(pipe))
}

/// Waits for the client of `pipe`; after the timeout the app connects
/// itself, so the wait ends and the caller sees its own process id.
fn accept(pipe: &Pipe, name: &str) -> Result<u32> {
    let done = Arc::new(AtomicBool::new(false));
    let watchdog = {
        let (done, path) = (Arc::clone(&done), format!(r"\\.\pipe\{name}"));
        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            while !done.load(Ordering::Relaxed) {
                if start.elapsed() > HANDSHAKE_TIMEOUT {
                    let _ = fs::OpenOptions::new().read(true).open(&path);
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        })
    };
    // SAFETY: `pipe` is a valid server handle; blocking connect.
    let connected = unsafe { ConnectNamedPipe(pipe.0, None) }.is_ok()
        || unsafe { GetLastError() } == ERROR_PIPE_CONNECTED;
    done.store(true, Ordering::Relaxed);
    let _ = watchdog.join();
    anyhow::ensure!(connected, "ConnectNamedPipe");
    let mut pid = 0u32;
    // SAFETY: connected pipe; `pid` outlives the call.
    unsafe { GetNamedPipeClientProcessId(pipe.0, &mut pid) }
        .context("GetNamedPipeClientProcessId")?;
    anyhow::ensure!(
        pid != std::process::id(),
        "the console did not start within {} s",
        HANDSHAKE_TIMEOUT.as_secs()
    );
    Ok(pid)
}

/// Sends the kit's environment to the console that connected.
fn send(pipe: &Pipe, env: &KitEnv) -> Result<()> {
    let vars: Map<String, Value> = env
        .iter()
        .map(|(k, v)| (k.to_owned(), Value::String(v.to_owned())))
        .collect();
    let bytes = serde_json::to_vec(&vars)?;
    let mut sent = 0;
    while sent < bytes.len() {
        let mut n = 0u32;
        // SAFETY: the buffer and counter outlive the synchronous call.
        unsafe { WriteFile(pipe.0, Some(&bytes[sent..]), Some(&mut n), None) }
            .context("WriteFile")?;
        sent += n as usize;
    }
    // Waits until the client read it all; closing the pipe (on drop, not
    // DisconnectNamedPipe, which drops what is unread) then gives it the end.
    // SAFETY: valid connected pipe.
    let _ = unsafe { FlushFileBuffers(pipe.0) };
    Ok(())
}

fn same_file(a: &Path, b: &Path) -> bool {
    a.as_os_str().eq_ignore_ascii_case(b.as_os_str())
}

/// Opens a new Windows Terminal window in `dir` running the kit's
/// PowerShell, or `program` through it; returns that process, the console.
pub fn open(kit: &Kit, env: &KitEnv, dir: &Path, program: Option<&Path>) -> Result<Console> {
    if let Err(e) = write_settings(kit) {
        crate::log::warn(&format!("terminal settings left alone: {e:#}"));
    }
    let name = format!("Atollo-Console-{}", crate::editor::random_token()?);
    let pipe = create_pipe(&name)?;
    let pwsh = pwsh(kit);
    let mut cmd = jobs::kit_command(exe(kit), env);
    cmd.args([
        "--window",
        "new",
        "--profile",
        PROFILE_PWSH,
        "--startingDirectory",
    ])
    .arg(wt_arg(dir))
    .arg("--")
    .arg(wt_arg(&pwsh))
    .args(["-NoLogo", "-NoProfile"]);
    if program.is_none() {
        cmd.arg("-NoExit");
    }
    cmd.args(["-ExecutionPolicy", "Bypass", "-File"])
        .arg(wt_arg(kit.kit_files().join("console.ps1")))
        .arg(&name);
    if let Some(p) = program {
        cmd.arg(wt_arg(p));
    }
    // Windows Terminal is a window; it may hand the request to the one
    // already running and exit at once: not waited for. No standard
    // handles: it would keep the app's open as long as it runs.
    cmd.current_dir(dir)
        .creation_flags(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    jobs::spawn(&mut cmd)?;

    let pid = accept(&pipe, &name)?;
    let image = crate::os::processes::image(pid).unwrap_or_default();
    if !same_file(&image, &pwsh) {
        // SAFETY: valid connected pipe.
        let _ = unsafe { DisconnectNamedPipe(pipe.0) };
        bail!(
            "the console pipe was opened by another program: {}",
            image.display()
        );
    }
    // Opened before the environment is sent: the process is still there.
    let console = Console::open(pid)?;
    send(&pipe, env)?;
    Ok(console)
}

#[cfg(test)]
#[path = "../../../test/rust/editor/wterm.rs"]
mod tests;
