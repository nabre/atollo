//! Processes of the PC whose program lives in a given folder (agents of the
//! kit, programs to close before removing the drive, other kits). Windows
//! reads them with ToolHelp, Linux from `/proc`, macOS with `libproc`.

use std::path::{Path, PathBuf, MAIN_SEPARATOR};

#[cfg(windows)]
use windows::core::PWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
#[cfg(windows)]
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Proc {
    pub pid: u32,
    pub parent: u32,
    pub name: String,
    pub path: PathBuf,
}

/// `folder` as a lower-case prefix ending with the separator.
fn prefix_of(folder: &Path) -> String {
    format!(
        "{}{MAIN_SEPARATOR}",
        folder
            .display()
            .to_string()
            .trim_end_matches(MAIN_SEPARATOR)
            .to_lowercase()
    )
}

fn is_under(p: &Proc, prefix: &str) -> bool {
    p.path
        .display()
        .to_string()
        .to_lowercase()
        .starts_with(prefix)
}

/// The processes in `all` whose program is in `folder`.
pub fn running_from(all: &[Proc], folder: &Path) -> Vec<u32> {
    let prefix = prefix_of(folder);
    all.iter()
        .filter(|p| is_under(p, &prefix))
        .map(|p| p.pid)
        .collect()
}

/// Every process with its program path (those the user can query).
#[cfg(windows)]
pub fn all() -> Vec<Proc> {
    let mut out = Vec::new();
    // SAFETY: snapshot handle closed at the end.
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return out;
        };
        let mut e = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut ok = Process32FirstW(snap, &mut e).is_ok();
        while ok {
            let len = e
                .szExeFile
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(e.szExeFile.len());
            let name = String::from_utf16_lossy(&e.szExeFile[..len]);
            if let Some(path) = image(e.th32ProcessID) {
                out.push(Proc {
                    pid: e.th32ProcessID,
                    parent: e.th32ParentProcessID,
                    name,
                    path,
                });
            }
            ok = Process32NextW(snap, &mut e).is_ok();
        }
        let _ = CloseHandle(snap);
    }
    out
}

/// The parent of a process from `/proc/<pid>/stat`: the fourth field, after
/// the name in parentheses (which may itself hold spaces and parentheses).
#[cfg(any(target_os = "linux", test))]
fn stat_parent(stat: &str) -> Option<u32> {
    let rest = &stat[stat.rfind(')')? + 1..];
    rest.split_whitespace().nth(1)?.parse().ok()
}

/// Every process with its program path (those the user can read).
#[cfg(target_os = "linux")]
pub fn all() -> Vec<Proc> {
    let Ok(dir) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    dir.flatten()
        .filter_map(|e| {
            let pid: u32 = e.file_name().to_str()?.parse().ok()?;
            let path = image(pid)?;
            let stat = std::fs::read_to_string(e.path().join("stat")).ok()?;
            Some(Proc {
                pid,
                parent: stat_parent(&stat).unwrap_or(0),
                name: path.file_name()?.to_string_lossy().into_owned(),
                path,
            })
        })
        .collect()
}

/// The program of process `pid`, if the user can read it.
#[cfg(target_os = "linux")]
pub fn image(pid: u32) -> Option<PathBuf> {
    std::fs::read_link(format!("/proc/{pid}/exe")).ok()
}

/// Every process with its program path (those the user can query).
#[cfg(target_os = "macos")]
pub fn all() -> Vec<Proc> {
    // SAFETY: a null buffer asks only for the count.
    let n = unsafe { libc::proc_listallpids(std::ptr::null_mut(), 0) };
    if n <= 0 {
        return Vec::new();
    }
    // Room for processes started in between.
    let mut pids = vec![0 as libc::pid_t; n as usize + 64];
    let bytes = (pids.len() * size_of::<libc::pid_t>()) as libc::c_int;
    // SAFETY: the buffer and its size in bytes describe the same memory.
    let n = unsafe { libc::proc_listallpids(pids.as_mut_ptr().cast(), bytes) };
    pids.truncate(n.max(0) as usize);
    pids.into_iter()
        .filter(|&pid| pid > 0)
        .filter_map(|pid| {
            let pid = pid as u32;
            let path = image(pid)?;
            let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
            let size = size_of::<libc::proc_bsdinfo>() as libc::c_int;
            // SAFETY: `info` is the structure PROC_PIDTBSDINFO fills, of `size` bytes.
            let read = unsafe {
                libc::proc_pidinfo(
                    pid as libc::c_int,
                    libc::PROC_PIDTBSDINFO,
                    0,
                    std::ptr::from_mut(&mut info).cast(),
                    size,
                )
            };
            Some(Proc {
                pid,
                parent: if read == size { info.pbi_ppid } else { 0 },
                name: path.file_name()?.to_string_lossy().into_owned(),
                path,
            })
        })
        .collect()
}

/// The program of process `pid`, if the user can query it.
#[cfg(target_os = "macos")]
pub fn image(pid: u32) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let mut buf = vec![0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
    // SAFETY: the buffer and its size describe the same memory.
    let n = unsafe {
        libc::proc_pidpath(
            pid as libc::c_int,
            buf.as_mut_ptr().cast(),
            buf.len() as u32,
        )
    };
    (n > 0).then(|| PathBuf::from(std::ffi::OsStr::from_bytes(&buf[..n as usize])))
}

/// Other Unix systems: no process list.
#[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
pub fn all() -> Vec<Proc> {
    Vec::new()
}

#[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
pub fn image(_pid: u32) -> Option<PathBuf> {
    None
}

/// The program of process `pid`, if the user can query it.
#[cfg(windows)]
pub fn image(pid: u32) -> Option<PathBuf> {
    // SAFETY: query-only handle, closed right after use; the buffer and its
    // size outlive the call.
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let ok =
            QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut size)
                .is_ok();
        let _ = CloseHandle(h);
        ok.then(|| PathBuf::from(String::from_utf16_lossy(&buf[..size as usize])))
    }
}

/// Processes whose program is under `dir` (case-insensitive).
pub fn running_under(dir: &Path) -> Vec<Proc> {
    let prefix = prefix_of(dir);
    all().into_iter().filter(|p| is_under(p, &prefix)).collect()
}

#[cfg(test)]
#[path = "../../../test/rust/os/processes.rs"]
mod tests;
