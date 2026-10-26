//! Files and folders on Windows (`platform::files`, `platform::safe_path`):
//! links and junctions, protected DACLs, the renames another program holds
//! up.

use std::fs;
use std::io;
use std::path::Path;

use anyhow::{bail, Context, Result};
use windows::core::HSTRING;
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{SetFileSecurityW, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR};

/// Windows has no executable bit: a program is one by its extension.
pub const HAS_EXECUTABLE_BIT: bool = false;

/// A symbolic link or a junction (reparse point), seen without following it.
pub fn is_reparse(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    meta.file_type().is_symlink() || meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

/// The bytes the file `path` takes on the disk: what a sparse or
/// compressed file has written (`GetCompressedFileSizeW`), its size when
/// that cannot be read.
pub fn allocated(path: &Path, meta: &fs::Metadata) -> u64 {
    let mut high = 0u32;
    // SAFETY: `high` lives for the call; the name is a valid wide string.
    let low = unsafe {
        windows::Win32::Storage::FileSystem::GetCompressedFileSizeW(
            &HSTRING::from(path.as_os_str()),
            Some(&mut high),
        )
    };
    if low == u32::MAX && io::Error::last_os_error().raw_os_error().unwrap_or(0) != 0 {
        return meta.len();
    }
    (u64::from(high) << 32) | u64::from(low)
}

/// Protected DACL with full access for the owner only.
pub fn restrict_to_owner(path: &Path) -> Result<()> {
    set_dacl(path, "D:P(A;;FA;;;OW)")
}

/// Protected DACL with full access for the owner only, inherited.
pub fn restrict_dir_to_owner(path: &Path) -> Result<()> {
    set_dacl(path, "D:P(A;OICI;FA;;;OW)")
}

/// No executable bit on Windows.
pub fn set_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

/// A new file, owner-only before a byte is written.
pub fn create_private(path: &Path) -> io::Result<fs::File> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    if let Err(e) = restrict_to_owner(path) {
        crate::common::log::warn(&format!("{}: permissions: {e:#}", path.display()));
    }
    Ok(file)
}

/// What to say of a file another account made private.
pub fn denied_text(shown: &str, e: &io::Error) -> String {
    format!(
        "{shown}: access denied ({e}). It was made private by another Windows account, \
         perhaps on another PC (an NTFS disk keeps its permissions): delete it, or as \
         administrator run takeown /f \"{shown}\" and icacls \"{shown}\" /reset"
    )
}

/// Access denied (5) or sharing violation (32): another program holds the
/// target of the rename.
pub fn rename_busy(e: &io::Error) -> bool {
    matches!(e.raw_os_error(), Some(5 | 32))
}

pub(crate) fn set_dacl(path: &Path, sddl: &str) -> Result<()> {
    let sddl = HSTRING::from(sddl);
    let mut sd = PSECURITY_DESCRIPTOR::default();
    // SAFETY: valid SDDL string; `sd` receives a LocalAlloc'd descriptor freed below.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(&sddl, SDDL_REVISION_1, &mut sd, None)
    }
    .context("security descriptor")?;
    let file = HSTRING::from(path.as_os_str());
    // SAFETY: `sd` is a valid descriptor for the duration of the call.
    let result = unsafe { SetFileSecurityW(&file, DACL_SECURITY_INFORMATION, sd) };
    // SAFETY: frees the descriptor allocated above, once.
    unsafe { LocalFree(Some(HLOCAL(sd.0))) };
    result.ok().context("SetFileSecurityW")
}

/// The temporary folders are per user.
pub fn owner_only(_meta: &fs::Metadata) -> bool {
    true
}

pub fn safe_parent(_meta: &fs::Metadata) -> bool {
    true
}

/// A plain folder: the parent, the user's own, gives its permissions.
pub fn create_dir_owner_only(path: &Path) -> io::Result<()> {
    fs::DirBuilder::new().create(path)
}

pub fn add_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

pub fn is_executable(_path: &Path) -> io::Result<bool> {
    Ok(true)
}

/// A drive (`C:x`) or an alternate data stream.
pub fn special_in_name(name: &str) -> bool {
    name.contains(':')
}

/// Never the same entry: a root changed only in case is not a move here.
pub fn same_entry(_a: &Path, _b: &Path) -> bool {
    false
}

/// No execute bits.
pub fn any_exec_bit(_meta: &fs::Metadata) -> bool {
    false
}

/// The file version in the resources of the program `exe`.
pub fn program_version(exe: &Path) -> Result<String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{w, PCWSTR};
    use windows::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
    };
    if !exe.is_file() {
        bail!("{} not found", exe.display());
    }
    let path: Vec<u16> = exe.as_os_str().encode_wide().chain([0]).collect();
    // SAFETY: a NUL-terminated path.
    let size = unsafe { GetFileVersionInfoSizeW(PCWSTR(path.as_ptr()), None) };
    anyhow::ensure!(size > 0, "{}: no version", exe.display());
    let mut data = vec![0u8; size as usize];
    // SAFETY: a buffer of `size` bytes.
    unsafe { GetFileVersionInfoW(PCWSTR(path.as_ptr()), None, size, data.as_mut_ptr().cast()) }
        .context("GetFileVersionInfoW")?;
    let mut info: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut len = 0u32;
    // SAFETY: `data` holds the version resource read above; `info` points
    // into it while it lives.
    let found = unsafe { VerQueryValueW(data.as_ptr().cast(), w!("\\"), &mut info, &mut len) };
    anyhow::ensure!(
        found.as_bool() && !info.is_null() && len as usize >= size_of::<VS_FIXEDFILEINFO>(),
        "{}: no version",
        exe.display()
    );
    // SAFETY: checked above: a VS_FIXEDFILEINFO inside `data`.
    let fixed = unsafe { &*info.cast::<VS_FIXEDFILEINFO>() };
    let (ms, ls) = (fixed.dwFileVersionMS, fixed.dwFileVersionLS);
    Ok(format!(
        "{}.{}.{}.{}",
        ms >> 16,
        ms & 0xffff,
        ls >> 16,
        ls & 0xffff
    ))
}

/// The files of the Windows Installer package `msi` in `dest`: an
/// administrative installation (`msiexec /a`), which only extracts them,
/// without an administrator for a per-user package; nothing is registered
/// on the PC. The tree is `TARGETDIR`'s (`LocalApp\<product>\…`).
pub fn extract_msi(msi: &Path, dest: &Path) -> Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    // `msiexec` reads its own command line: the paths in double quotes,
    // without a `\` before the closing one, and with `\` only (a `/` in
    // the package's path fails to open it: 1619).
    let quoted = |p: &Path| {
        let path = p.display().to_string().replace('/', "\\");
        format!("\"{}\"", path.trim_end_matches('\\'))
    };
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into());
    let status = std::process::Command::new(Path::new(&root).join("System32").join("msiexec.exe"))
        .raw_arg(format!("/a {} /qn TARGETDIR={}", quoted(msi), quoted(dest)))
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(std::process::Stdio::null())
        .status()
        .context("msiexec")?;
    // 3010: done, a restart is asked (never for an extraction).
    match status.code() {
        Some(0 | 3010) => Ok(()),
        other => bail!("msiexec /a {}: exit {other:?}", msi.display()),
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/os/windows/files.rs"]
mod tests;
