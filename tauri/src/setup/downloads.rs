//! Files downloaded by hand (DECISIONS.md D-82): when a download fails, the
//! interface shows its address; the file saved by the browser is dropped on
//! the app (or picked) and kept in `downloads\` of the PC's temporary folder
//! (`data\downloads` without one, D-90) until an installation uses it. A
//! file is used only when its checksum is the one the source publishes,
//! whatever its name (a browser may add ` (1)`).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};

use crate::kit::Kit;
use crate::net::verify::{self, Expect};

/// Kept at most this long when no installation uses them.
const KEEP: Duration = Duration::from_secs(30 * 24 * 3600);

/// `downloads\` of the PC's temporary folder (D-90, Q-3), or `data\downloads`
/// without one. The files found in `data\downloads` (kept there by an older
/// version, or put there by hand as the command line says) are moved into it
/// first.
pub fn dir(kit: &Kit) -> PathBuf {
    let in_kit = kit.data().join("downloads");
    let Some(temp) = kit.system_temp() else {
        return in_kit;
    };
    let dir = temp.join("downloads");
    move_files(&in_kit, &dir);
    dir
}

/// Moves the files of `from` (not the `.part` ones) into `to`: renamed on
/// the same disk, else copied through `.part` and deleted. A file that
/// cannot move (still being written) stays, with a warning in the log.
fn move_files(from: &Path, to: &Path) {
    let Ok(entries) = fs::read_dir(from) else {
        return;
    };
    for path in entries.flatten().map(|e| e.path()) {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !path.is_file() || name.ends_with(".part") {
            continue;
        }
        let moved = (|| -> Result<()> {
            fs::create_dir_all(to).with_context(|| to.display().to_string())?;
            let dest = to.join(name);
            if fs::rename(&path, &dest).is_ok() {
                return Ok(());
            }
            let part = to.join(format!("{name}.part"));
            fs::copy(&path, &part)
                .with_context(|| format!("{} → {}", path.display(), part.display()))?;
            fs::rename(&part, &dest).with_context(|| dest.display().to_string())?;
            fs::remove_file(&path).with_context(|| path.display().to_string())
        })();
        if let Err(e) = moved {
            crate::log::warn(&format!("{}: {e:#}", path.display()));
        }
    }
}

/// Copies `files` into `dir(kit)` (through `.part`); returns their
/// names. Folders are refused; the files are left where they are.
pub fn add(kit: &Kit, files: &[PathBuf]) -> Result<Vec<String>> {
    let dir = dir(kit);
    fs::create_dir_all(&dir).with_context(|| dir.display().to_string())?;
    prune(&dir);
    let mut names = Vec::new();
    for file in files {
        if !file.is_file() {
            bail!("not a file: {}", file.display());
        }
        let name = file
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|n| !n.ends_with(".part"))
            .with_context(|| format!("file name: {}", file.display()))?;
        let dest = dir.join(name);
        let part = dir.join(format!("{name}.part"));
        fs::copy(file, &part)
            .with_context(|| format!("{} → {}", file.display(), part.display()))?;
        fs::rename(&part, &dest).with_context(|| dest.display().to_string())?;
        names.push(name.to_owned());
    }
    Ok(names)
}

/// Moves into `dest` the file added by hand with the checksum `expect`:
/// first the one named `file_name`, then any other. `false` when none.
pub fn take(kit: &Kit, file_name: &str, expect: &Expect, dest: &Path) -> Result<bool> {
    let dir = dir(kit);
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(false);
    };
    let mut candidates: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_none_or(|e| e != "part"))
        .collect();
    // The expected name first; the others only if none matches.
    candidates.sort_by_key(|p| {
        p.file_name()
            .is_none_or(|n| !n.eq_ignore_ascii_case(file_name))
    });
    for file in candidates {
        if !verify::matches(&file, expect).unwrap_or(false) {
            continue;
        }
        if fs::rename(&file, dest).is_err() {
            fs::copy(&file, dest).with_context(|| dest.display().to_string())?;
            let _ = fs::remove_file(&file);
        }
        crate::log::info(&format!(
            "{file_name}: the file added by hand {}",
            file.display()
        ));
        return Ok(true);
    }
    Ok(false)
}

/// Windows' "Open" window over `owner` (a window handle), several files,
/// starting in the Downloads folder. Empty when closed without a choice.
/// macOS and Linux: `editor::picker::files`.
#[cfg(windows)]
pub fn pick(owner: isize, title: &str) -> Result<Vec<PathBuf>> {
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::Foundation::{ERROR_CANCELLED, HWND};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
    };
    use windows::Win32::UI::Shell::{
        FOLDERID_Downloads, FileOpenDialog, IFileOpenDialog, IShellItem,
        SHCreateItemFromParsingName, SHGetKnownFolderPath, FOS_ALLOWMULTISELECT, FOS_FILEMUSTEXIST,
        FOS_FORCEFILESYSTEM, KF_FLAG_DEFAULT, SIGDN_FILESYSPATH,
    };
    // SAFETY: COM initialised on this thread for the dialog and released
    // after every COM object of the closure is dropped; the strings COM
    // allocates are freed once read.
    unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
        let result = (|| -> Result<Vec<PathBuf>> {
            let dialog: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
            dialog.SetOptions(
                dialog.GetOptions()?
                    | FOS_ALLOWMULTISELECT
                    | FOS_FILEMUSTEXIST
                    | FOS_FORCEFILESYSTEM,
            )?;
            dialog.SetTitle(&HSTRING::from(title))?;
            if let Ok(path) = SHGetKnownFolderPath(&FOLDERID_Downloads, KF_FLAG_DEFAULT, None) {
                let folder: windows::core::Result<IShellItem> =
                    SHCreateItemFromParsingName(PCWSTR(path.0), None);
                CoTaskMemFree(Some(path.0 as _));
                if let Ok(folder) = folder {
                    let _ = dialog.SetDefaultFolder(&folder);
                }
            }
            if let Err(e) = dialog.Show(Some(HWND(owner as _))) {
                if e.code() == ERROR_CANCELLED.to_hresult() {
                    return Ok(Vec::new());
                }
                return Err(e.into());
            }
            let items = dialog.GetResults()?;
            let mut files = Vec::new();
            for i in 0..items.GetCount()? {
                let path = items.GetItemAt(i)?.GetDisplayName(SIGDN_FILESYSPATH)?;
                let text = path.to_string();
                CoTaskMemFree(Some(path.0 as _));
                files.push(PathBuf::from(text?));
            }
            Ok(files)
        })();
        if init.is_ok() {
            CoUninitialize();
        }
        result
    }
}

/// Removes what no installation used for a month, and `.part` left over.
fn prune(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let now = SystemTime::now();
    for path in entries.flatten().map(|e| e.path()) {
        let old = fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| now.duration_since(t).ok())
            .is_some_and(|age| age > KEEP);
        if path.is_file() && (old || path.extension().is_some_and(|e| e == "part")) {
            let _ = fs::remove_file(&path);
        }
    }
}

#[cfg(test)]
#[path = "../../../test/rust/setup/downloads.rs"]
mod tests;
