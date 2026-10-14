//! Processes of the PC whose program lives in a given folder (agents of the
//! kit, programs to close before removing the drive, other kits).

use std::path::{Path, PathBuf};

use windows::core::PWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
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

/// The processes in `all` whose program is in `folder`.
pub fn running_from(all: &[Proc], folder: &Path) -> Vec<u32> {
    let prefix = format!("{}\\", folder.display()).to_lowercase();
    all.iter()
        .filter(|p| {
            p.path
                .display()
                .to_string()
                .to_lowercase()
                .starts_with(&prefix)
        })
        .map(|p| p.pid)
        .collect()
}

/// Every process with its program path (those the user can query).
pub fn all() -> Vec<Proc> {
    let mut out = Vec::new();
    // SAFETY: snapshot handle closed at the end; each process handle is
    // query-only and closed right after use.
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
            if let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, e.th32ProcessID) {
                let mut buf = [0u16; 1024];
                let mut size = buf.len() as u32;
                if QueryFullProcessImageNameW(
                    h,
                    PROCESS_NAME_WIN32,
                    PWSTR(buf.as_mut_ptr()),
                    &mut size,
                )
                .is_ok()
                {
                    out.push(Proc {
                        pid: e.th32ProcessID,
                        parent: e.th32ParentProcessID,
                        name,
                        path: PathBuf::from(String::from_utf16_lossy(&buf[..size as usize])),
                    });
                }
                let _ = CloseHandle(h);
            }
            ok = Process32NextW(snap, &mut e).is_ok();
        }
        let _ = CloseHandle(snap);
    }
    out
}

/// Processes whose program is under `dir` (case-insensitive).
pub fn running_under(dir: &Path) -> Vec<Proc> {
    let prefix = format!(
        "{}\\",
        dir.display()
            .to_string()
            .trim_end_matches('\\')
            .to_lowercase()
    );
    all()
        .into_iter()
        .filter(|p| {
            p.path
                .display()
                .to_string()
                .to_lowercase()
                .starts_with(&prefix)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_itself() {
        let me = std::env::current_exe().unwrap();
        let dir = me.parent().unwrap();
        assert!(running_under(dir)
            .iter()
            .any(|p| p.pid == std::process::id()));
        assert!(running_under(Path::new(r"Z:\no\such\folder")).is_empty());
    }
}
