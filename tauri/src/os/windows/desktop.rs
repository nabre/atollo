//! The desktop around the app's windows on Windows (`platform::desktop`):
//! IFileOpenDialog for folders and files, `MessageBoxW`.

use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tauri::{Runtime, Window};
use windows::core::HSTRING;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

/// The window's handle, as a number a blocking task can carry.
pub struct Owner<R: Runtime>(isize, PhantomData<fn() -> R>);

pub fn owner<R: Runtime>(window: &Window<R>) -> Result<Owner<R>> {
    Ok(Owner(
        window.hwnd().context("window")?.0 as isize,
        PhantomData,
    ))
}

pub fn pick_folder<R: Runtime>(
    owner: &Owner<R>,
    title: &str,
    start: Option<&Path>,
) -> Result<Option<PathBuf>> {
    pick_dir(owner.0, title, start)
}

/// Windows' "Open" window always starts in the Downloads folder (only as a
/// default: it remembers the last folder of each title).
pub fn pick_files<R: Runtime>(
    owner: &Owner<R>,
    title: &str,
    _downloads: bool,
) -> Result<Vec<PathBuf>> {
    pick(owner.0, title)
}

/// A message box for an error before (or instead of) the window.
pub fn error_box(title: &str, message: &str) {
    let title = HSTRING::from(title);
    let text = HSTRING::from(message);
    // SAFETY: both strings outlive the modal call.
    unsafe { MessageBoxW(None, &text, &title, MB_OK | MB_ICONERROR) };
}

/// The notification area is always there.
pub fn tray_available() -> bool {
    true
}

/// Windows' "Select folder" window over `owner` (a window handle), starting
/// in `start`. `None` when closed without a choice.
fn pick_dir(owner: isize, title: &str, start: Option<&Path>) -> Result<Option<PathBuf>> {
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::Foundation::{ERROR_CANCELLED, HWND};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
    };
    use windows::Win32::UI::Shell::{
        FileOpenDialog, IFileOpenDialog, IShellItem, SHCreateItemFromParsingName,
        FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, SIGDN_FILESYSPATH,
    };
    // SAFETY: as `pick`: COM initialised on this thread
    // for the dialog and released after every COM object of the closure is
    // dropped; the strings COM allocates are freed once read.
    unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
        let result = (|| -> Result<Option<PathBuf>> {
            let dialog: IFileOpenDialog =
                CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
            dialog.SetOptions(
                dialog.GetOptions()? | FOS_PICKFOLDERS | FOS_PATHMUSTEXIST | FOS_FORCEFILESYSTEM,
            )?;
            dialog.SetTitle(&HSTRING::from(title))?;
            if let Some(start) = start.filter(|s| s.is_dir()) {
                let start = HSTRING::from(start.as_os_str());
                let folder: windows::core::Result<IShellItem> =
                    SHCreateItemFromParsingName(PCWSTR(start.as_ptr()), None);
                if let Ok(folder) = folder {
                    let _ = dialog.SetFolder(&folder);
                }
            }
            if let Err(e) = dialog.Show(Some(HWND(owner as _))) {
                if e.code() == ERROR_CANCELLED.to_hresult() {
                    return Ok(None);
                }
                return Err(e.into());
            }
            let path = dialog.GetResult()?.GetDisplayName(SIGDN_FILESYSPATH)?;
            let text = path.to_string();
            CoTaskMemFree(Some(path.0 as _));
            Ok(Some(PathBuf::from(text?)))
        })();
        if init.is_ok() {
            CoUninitialize();
        }
        result
    }
}

/// Windows' "Open" window over `owner` (a window handle), several files,
/// starting in the Downloads folder. Empty when closed without a choice.
fn pick(owner: isize, title: &str) -> Result<Vec<PathBuf>> {
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
