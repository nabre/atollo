//! Files downloaded by hand (DECISIONS.md D-82): when a download fails, the
//! interface shows its address; the file saved by the browser is dropped on
//! the app (or picked) and kept in `downloads\` of the PC's temporary folder
//! (`data\downloads` without one, D-90) until an installation uses it. A
//! file is used only when its checksum is the one the source publishes,
//! whatever its name (a browser may add ` (1)`).
//!
//! Which component a file is for is known from its checksum only: every
//! failed download writes it in `.wanted` ([`want`]), so removing the
//! component also removes the file added for it ([`forget`]).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{bail, Context, Result};

use crate::fsx::write_atomic;
use crate::kit::Kit;
use crate::net::verify::{self, Expect};

/// Kept at most this long when no installation uses them.
const KEEP: Duration = Duration::from_secs(30 * 24 * 3600);

/// The checksums the failed downloads wanted, in `dir(kit)`: one line per
/// file, `<target>\t<component>\t<checksum>`. The folder is the kit's own,
/// shared by the systems it is prepared for, hence the target.
const WANTED: &str = ".wanted";

/// One line of `.wanted`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Wanted {
    target: String,
    id: String,
    expect: Expect,
}

impl Wanted {
    fn of(kit: &Kit, id: &str, expect: &Expect) -> Wanted {
        Wanted {
            target: kit.target().to_string(),
            id: id.to_owned(),
            expect: expect.clone(),
        }
    }

    fn render(&self) -> String {
        let expect = match &self.expect {
            Expect::Sha256(h) => format!("sha256:{h}"),
            Expect::Sha512(h) => format!("sha512:{h}"),
            Expect::NpmIntegrity(h) => format!("npm:{h}"),
            Expect::Sha3_256(h) => format!("sha3-256:{h}"),
        };
        format!("{}\t{}\t{expect}", self.target, self.id)
    }

    fn parse(line: &str) -> Option<Wanted> {
        let mut fields = line.trim().split('\t');
        let (target, id) = (fields.next()?, fields.next()?);
        let expect = match fields.next()?.split_once(':')? {
            ("sha256", h) => Expect::Sha256(h.to_owned()),
            ("sha512", h) => Expect::Sha512(h.to_owned()),
            ("npm", h) => Expect::NpmIntegrity(h.to_owned()),
            ("sha3-256", h) => Expect::Sha3_256(h.to_owned()),
            _ => return None,
        };
        Some(Wanted {
            target: target.to_owned(),
            id: id.to_owned(),
            expect,
        })
    }

    fn is_for(&self, kit: &Kit, id: &str) -> bool {
        self.target == kit.target().to_string() && self.id == id
    }
}

fn read_wanted(dir: &Path) -> Vec<Wanted> {
    fs::read_to_string(dir.join(WANTED))
        .unwrap_or_default()
        .lines()
        .filter_map(Wanted::parse)
        .collect()
}

/// `.wanted` written again with `list`, removed when empty. A failure is
/// logged only: the files are still found by checksum without it.
fn write_wanted(dir: &Path, list: &[Wanted]) {
    let path = dir.join(WANTED);
    let result = if list.is_empty() {
        match fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    } else {
        let text: String = list.iter().map(|w| w.render() + "\n").collect();
        fs::create_dir_all(dir).and_then(|()| write_atomic(&path, text.as_bytes()))
    };
    if let Err(e) = result {
        crate::log::warn(&format!("{}: {e}", path.display()));
    }
}

/// A download of component `id` failed for `kit`: the file the browser
/// will save, with checksum `expect`, is its own.
pub fn want(kit: &Kit, id: &str, expect: &Expect) {
    let dir = dir(kit);
    let mut list = read_wanted(&dir);
    let wanted = Wanted::of(kit, id, expect);
    if !list.contains(&wanted) {
        list.push(wanted);
        write_wanted(&dir, &list);
    }
}

/// Component `id` was removed from `kit`: the files added by hand for it
/// go, except one another component (or the same one on another system)
/// still waits for with the same checksum. A file that cannot be deleted
/// (open elsewhere) stays, with a warning in the log, and goes after a
/// month as the others.
pub fn forget(kit: &Kit, id: &str) {
    let dir = dir(kit);
    let (mine, others): (Vec<Wanted>, Vec<Wanted>) = read_wanted(&dir)
        .into_iter()
        .partition(|w| w.is_for(kit, id));
    if mine.is_empty() {
        return;
    }
    let wanted_by = |file: &Path, list: &[Wanted]| {
        list.iter()
            .any(|w| verify::matches(file, &w.expect).unwrap_or(false))
    };
    for file in added(&dir) {
        if wanted_by(&file, &mine) && !wanted_by(&file, &others) {
            if let Err(e) = fs::remove_file(&file) {
                crate::log::warn(&format!("{}: {e}", file.display()));
            }
        }
    }
    write_wanted(&dir, &others);
}

/// The files added by hand in `dir`: not the `.part` ones being copied,
/// nor `.wanted`.
fn added(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_none_or(|e| e != "part"))
        .filter(|p| p.file_name().is_some_and(|n| n != WANTED))
        .collect()
}

/// `downloads\` of the PC's temporary folder (D-90), or `data\downloads`
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
        // Joined to the one there, which a rename would overwrite.
        if name == WANTED {
            let mut list = read_wanted(to);
            for w in read_wanted(from) {
                if !list.contains(&w) {
                    list.push(w);
                }
            }
            write_wanted(to, &list);
            let _ = fs::remove_file(&path);
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

/// Copies into `dest` the file added by hand with the checksum `expect`:
/// first the one named `file_name`, then any other. Returns the file
/// copied, which stays until the installation that uses it succeeds
/// ([`used`]); `None` when there is none.
pub fn take(kit: &Kit, file_name: &str, expect: &Expect, dest: &Path) -> Result<Option<PathBuf>> {
    let dir = dir(kit);
    let mut candidates = added(&dir);
    // The expected name first; the others only if none matches.
    candidates.sort_by_key(|p| {
        p.file_name()
            .is_none_or(|n| !n.eq_ignore_ascii_case(file_name))
    });
    for file in candidates {
        if !verify::matches(&file, expect).unwrap_or(false) {
            continue;
        }
        // Copied, not moved: an installation that fails (extraction, an
        // antivirus, a file in use) deletes its work folder, and the file
        // must still be here for the next attempt.
        fs::copy(&file, dest).with_context(|| dest.display().to_string())?;
        crate::log::info(&format!(
            "{file_name}: the file added by hand {}",
            file.display()
        ));
        return Ok(Some(file));
    }
    Ok(None)
}

/// The installation that took `file` ([`take`]) succeeded: it goes, and no
/// component waits for its checksum `expect` any more.
pub fn used(kit: &Kit, file: &Path, expect: &Expect) {
    if let Err(e) = fs::remove_file(file) {
        if e.kind() != std::io::ErrorKind::NotFound {
            crate::log::warn(&format!("{}: {e}", file.display()));
        }
    }
    let dir = dir(kit);
    let list = read_wanted(&dir);
    if list.iter().any(|w| w.expect == *expect) {
        let left: Vec<Wanted> = list.into_iter().filter(|w| w.expect != *expect).collect();
        write_wanted(&dir, &left);
    }
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
