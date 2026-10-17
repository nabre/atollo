//! Archives and folder swaps for the installer (port of nabre/dev-kit
//! `Expand-Package`, `Replace-Dir` and `Replace-File`).
//!
//! Extraction refuses entries that would land outside the target (absolute
//! paths, `..`, drive prefixes) and entries whose way from the target crosses
//! a symbolic link; a file is written only as a new file (an existing one is
//! removed first, never truncated through a link). Links are never created on
//! Windows. On Unix symbolic links are created last, after every file and
//! folder (as GNU tar does), and only if the target is relative, has no `\`
//! or `:`, and resolves inside the target folder through real folders only
//! (so a chain of links cannot climb out); a hard link is created only to a
//! regular file already extracted there. Other links are skipped. On a disk
//! without links (exFAT, FAT) the target is copied in place of the link. File
//! times are not restored (exFAT rejects them; `tar -m` in the kit). Swaps are atomic from the user's point of view: the
//! new folder is complete before the old one moves, and a failure puts the
//! old one back.

use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Zip,
    TarGz,
    TarXz,
    /// A 7z archive, also inside a self-extracting `.7z.exe`.
    SevenZip,
    /// Not an archive: the download is the file itself (an exe), copied
    /// into the target folder under its name.
    File,
}

/// Which part of the extracted tree becomes the component folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inner {
    /// The archive root.
    Root,
    /// Its only top-level folder (error if there is not exactly one).
    Single,
    /// The only top-level folder if there is exactly one, else the root.
    Auto,
    /// A named subfolder.
    Named(String),
    /// The only folder inside a named subfolder (`package/vendor/<target>`).
    SingleUnder(String),
    /// Its only top-level folder, followed by a relative path inside it
    /// (e.g. `Contents/Home` for macOS Temurin JDK `.tar.gz`).
    SingleThen(&'static str),
}

/// `base` + an archive entry name, or `None` if the name would escape `base`.
/// Splits on both `'/'` and `'\\'` so paths work identically on Windows,
/// macOS, and Linux without creating literal backslashes on Unix.
pub fn safe_join(base: &Path, name: &str) -> Option<PathBuf> {
    if name.starts_with(['/', '\\']) || name.contains(':') {
        return None;
    }
    let mut out = base.to_path_buf();
    let mut depth = 0usize;
    for seg in name.split(['/', '\\']) {
        match seg {
            "" | "." => {}
            ".." => return None,
            part => {
                out.push(part);
                depth += 1;
            }
        }
    }
    (depth > 0).then_some(out)
}

#[cfg(unix)]
fn set_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(perms.mode() | 0o755);
        let _ = fs::set_permissions(path, perms);
    }
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) {}

pub fn extract(archive: &Path, kind: Kind, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    match kind {
        Kind::Zip => extract_zip(archive, dest),
        Kind::TarGz => extract_tar_gz(archive, dest),
        Kind::TarXz => extract_tar_xz(archive, dest),
        Kind::SevenZip => extract_7z(archive, dest),
        Kind::File => {
            let name = archive.file_name().context("file name")?;
            let out = dest.join(name);
            fs::copy(archive, &out)?;
            set_executable(&out);
            Ok(())
        }
    }
    .with_context(|| format!("extracting {}", archive.display()))
}

/// Refuses `path` if it, or a folder between `dest` and it, is a symbolic
/// link (or another link Windows reports as one): nothing is ever written
/// through a link, wherever it came from.
fn no_link_on_the_way(dest: &Path, path: &Path) -> Result<()> {
    let rel = path
        .strip_prefix(dest)
        .with_context(|| format!("{} is outside {}", path.display(), dest.display()))?;
    let mut cur = dest.to_path_buf();
    for part in rel.components() {
        cur.push(part);
        match fs::symlink_metadata(&cur) {
            Ok(meta) if meta.file_type().is_symlink() => {
                bail!("unsafe path in archive: {} is a link", cur.display())
            }
            Ok(_) => {}
            Err(_) => break, // Not there yet: nothing below it exists either.
        }
    }
    Ok(())
}

/// Creates the folder `dir` inside `dest`, never through a link.
fn prepare_dir(dest: &Path, dir: &Path) -> Result<()> {
    no_link_on_the_way(dest, dir)?;
    fs::create_dir_all(dir)?;
    Ok(())
}

/// A new file at `path` inside `dest`: what was there (a file of an
/// earlier entry, or a link) is removed, never truncated or followed.
fn create_file(dest: &Path, path: &Path) -> Result<File> {
    if let Some(dir) = path.parent() {
        prepare_dir(dest, dir)?;
    }
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => {
            bail!("unsafe path in archive: {} is a folder", path.display())
        }
        Ok(_) => fs::remove_file(path)?,
        Err(_) => {}
    }
    Ok(fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?)
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<()> {
    let mut zip = zip::ZipArchive::new(File::open(archive)?)?;
    #[cfg(unix)]
    let mut links: Vec<(PathBuf, String)> = Vec::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(path) = safe_join(dest, entry.name()) else {
            bail!("unsafe path in archive: {}", entry.name());
        };
        if entry.is_dir() {
            prepare_dir(dest, &path)?;
        } else if entry.is_symlink() {
            #[cfg(unix)]
            {
                let mut target = String::new();
                if entry.read_to_string(&mut target).is_ok() {
                    links.push((path, target.trim().to_owned()));
                }
            }
            continue;
        } else {
            let mode = entry.unix_mode();
            io::copy(&mut entry, &mut create_file(dest, &path)?)?;
            if mode.is_some_and(|m| m & 0o111 != 0)
                || path
                    .parent()
                    .and_then(|p| p.file_name())
                    .is_some_and(|n| n == "bin" || n == "sbin")
            {
                set_executable(&path);
            }
        }
    }
    #[cfg(unix)]
    finish_links_unix(dest, links)?;
    Ok(())
}

/// The file system refuses symbolic links: exFAT and FAT answer `EPERM`
/// (Linux) or `ENOTSUP`/`EOPNOTSUPP` (macOS). Any other error is a real one.
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) fn links_unsupported(e: &io::Error) -> bool {
    #[cfg(unix)]
    if let Some(code) = e.raw_os_error() {
        return code == libc::EPERM || code == libc::ENOTSUP || code == libc::EOPNOTSUPP;
    }
    e.kind() == io::ErrorKind::Unsupported
}

/// Where the link `link_path -> link_target` lands, relative to `dest`, if
/// it resolves inside `dest` the way the kernel resolves it: every name
/// crossed on the way (all but the last) must be a real folder, not a link,
/// so a link made earlier (`t -> .`, then `s -> t/..`) cannot be used to
/// climb out. `None` for an absolute target or one outside `dest`.
#[cfg_attr(not(unix), allow(dead_code))]
fn resolve_inside(dest: &Path, link_path: &Path, link_target: &str) -> Option<PathBuf> {
    if link_target.is_empty() || link_target.starts_with('/') || link_target.contains(['\\', ':']) {
        return None;
    }
    let rel_parent = link_path.parent()?.strip_prefix(dest).ok()?;
    let mut steps: Vec<&std::ffi::OsStr> = rel_parent.iter().collect();
    steps.extend(
        link_target
            .split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .map(std::ffi::OsStr::new),
    );
    let mut resolved = PathBuf::new();
    for (i, step) in steps.iter().enumerate() {
        if *step == ".." {
            if !resolved.pop() {
                return None; // Above `dest`.
            }
            continue;
        }
        resolved.push(step);
        if i + 1 < steps.len() {
            match fs::symlink_metadata(dest.join(&resolved)) {
                Ok(meta) if meta.is_dir() => {}
                _ => return None, // A link, a file or nothing: not followed.
            }
        }
    }
    Some(resolved)
}

/// Creates the symbolic links `(path, target)` collected while extracting or
/// copying into `dest`, after every file and folder exists. A link whose
/// target is outside `dest` (or crosses another link) is skipped. Where the
/// file system has no links ([`links_unsupported`]: exFAT, FAT) the target
/// is copied in place of the link, in a second pass, so a target that is
/// itself such a copy is complete first. `make` creates one link
/// (`symlink(target, path)`; the tests inject a refusal). Returns the links
/// skipped, for a warning.
#[cfg_attr(not(unix), allow(dead_code))]
fn finish_links(
    dest: &Path,
    links: Vec<(PathBuf, String)>,
    mut make: impl FnMut(&str, &Path) -> io::Result<()>,
) -> Result<Vec<String>> {
    let mut skipped = Vec::new();
    let mut copies: Vec<(PathBuf, PathBuf)> = Vec::new();
    for (path, target) in links {
        let Some(parent) = path.parent() else {
            continue;
        };
        if parent.strip_prefix(dest).is_err() {
            skipped.push(format!("{} -> {target}", path.display()));
            continue;
        }
        prepare_dir(dest, parent)?;
        let Some(resolved) = resolve_inside(dest, &path, &target) else {
            skipped.push(format!("{} -> {target}", path.display()));
            continue;
        };
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_dir() => continue, // A real folder stays.
            Ok(_) => fs::remove_file(&path)?,
            Err(_) => {}
        }
        match make(&target, &path) {
            Ok(()) => {}
            Err(e) if links_unsupported(&e) => copies.push((path, dest.join(resolved))),
            Err(e) => return Err(e).with_context(|| path.display().to_string()),
        }
    }
    // Copies in place of the links: first those whose source exists and
    // holds no copy still to make (a link to a link, a folder with links).
    let there = |p: &Path| fs::symlink_metadata(p).is_ok();
    loop {
        let ready = copies.iter().position(|(path, source)| {
            there(source)
                && !copies
                    .iter()
                    .any(|(p, _)| p != path && p.starts_with(source))
        });
        // Links that point at each other: one goes first, as it is.
        let Some(i) = ready.or_else(|| copies.iter().position(|(_, s)| there(s))) else {
            break;
        };
        let (path, source) = copies.remove(i);
        let meta = fs::symlink_metadata(&source)?;
        if meta.file_type().is_symlink() || path.starts_with(&source) {
            // A link not followed, or a folder that would hold itself.
            skipped.push(format!("{} -> {}", path.display(), source.display()));
        } else if meta.is_dir() {
            copy_tree(&source, &path).with_context(|| path.display().to_string())?;
        } else {
            fs::copy(&source, &path).with_context(|| path.display().to_string())?;
        }
    }
    skipped.extend(
        copies
            .into_iter()
            .map(|(path, source)| format!("{} -> {}", path.display(), source.display())),
    );
    Ok(skipped)
}

/// [`finish_links`] with real links, the skipped ones in the log.
#[cfg(unix)]
fn finish_links_unix(dest: &Path, links: Vec<(PathBuf, String)>) -> Result<()> {
    let skipped = finish_links(dest, links, |target, path| {
        std::os::unix::fs::symlink(target, path)
    })?;
    for link in skipped {
        crate::log::warn(&format!(
            "link not made (outside the folder, or through another link): {link}"
        ));
    }
    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<()> {
    let gz = flate2::read::GzDecoder::new(io::BufReader::new(File::open(archive)?));
    extract_tar_stream(gz, dest)
}

/// Memory the xz decoder may take, in KiB (1 GiB; `xz -9` needs 65 MiB).
const XZ_MEM_LIMIT_KIB: u32 = 1 << 20;

fn extract_tar_xz(archive: &Path, dest: &Path) -> Result<()> {
    // Decoded in process and fed to the same tar walk as `.tar.gz`, so every
    // entry goes through `safe_join` and the link rules of `extract_tar_stream`.
    let xz = lzma_rust2::XzReader::new_mem_limit(
        io::BufReader::new(File::open(archive)?),
        true,
        XZ_MEM_LIMIT_KIB,
    );
    extract_tar_stream(xz, dest)
}

fn extract_tar_stream<R: Read>(reader: R, dest: &Path) -> Result<()> {
    let mut tar = tar::Archive::new(reader);
    tar.set_preserve_mtime(false);
    tar.set_preserve_permissions(false);
    #[cfg(unix)]
    let mut links: Vec<(PathBuf, String)> = Vec::new();
    for entry in tar.entries()? {
        let mut entry = entry?;
        let name = entry.path()?.to_string_lossy().into_owned();
        let Some(path) = safe_join(dest, &name) else {
            if name.trim_matches(['.', '/']).is_empty() {
                continue; // "./"
            }
            bail!("unsafe path in archive: {name}");
        };
        let entry_type = entry.header().entry_type();
        let mode = entry.header().mode().ok();
        match entry_type {
            tar::EntryType::Directory => prepare_dir(dest, &path)?,
            tar::EntryType::Regular | tar::EntryType::Continuous => {
                io::copy(&mut entry, &mut create_file(dest, &path)?)?;
                if mode.is_some_and(|m| m & 0o111 != 0)
                    || path
                        .parent()
                        .and_then(|p| p.file_name())
                        .is_some_and(|n| n == "bin" || n == "sbin")
                {
                    set_executable(&path);
                }
            }
            #[cfg(unix)]
            tar::EntryType::Symlink => {
                if let Ok(Some(target)) = entry.link_name() {
                    links.push((path, target.to_string_lossy().into_owned()));
                }
            }
            #[cfg(unix)]
            tar::EntryType::Link => {
                if let Ok(Some(target)) = entry.link_name() {
                    if let Some(target_path) = safe_join(dest, &target.to_string_lossy()) {
                        let regular = no_link_on_the_way(dest, &target_path).is_ok()
                            && fs::symlink_metadata(&target_path)
                                .is_ok_and(|m| m.file_type().is_file());
                        if regular {
                            if let Some(dir) = path.parent() {
                                prepare_dir(dest, dir)?;
                            }
                            if fs::symlink_metadata(&path).is_ok_and(|m| !m.is_dir()) {
                                fs::remove_file(&path)?;
                            }
                            let _ = fs::hard_link(&target_path, &path);
                        }
                    }
                }
            }
            // Devices, pax headers, or links on Windows: never followed or created.
            _ => {}
        }
    }
    #[cfg(unix)]
    finish_links_unix(dest, links)?;
    Ok(())
}

/// 7z signature: `7z\xBC\xAF\x27\x1C`.
const SEVEN_Z: [u8; 6] = [b'7', b'z', 0xBC, 0xAF, 0x27, 0x1C];

/// A file seen from `offset` on, so a self-extractor's stub is skipped.
struct Offset {
    file: File,
    offset: u64,
}

impl Read for Offset {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Seek for Offset {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let abs = match pos {
            SeekFrom::Start(n) => self.file.seek(SeekFrom::Start(self.offset + n))?,
            other => self.file.seek(other)?,
        };
        abs.checked_sub(self.offset)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek before archive start"))
    }
}

/// Offsets where a 7z signature starts (the stub of a self-extractor may
/// contain the bytes too: each candidate is tried in turn).
fn seven_z_offsets(path: &Path) -> Result<Vec<u64>> {
    let mut file = File::open(path)?;
    let mut found = Vec::new();
    let mut buf = vec![0u8; 1024 * 1024];
    let mut base = 0u64;
    let mut carry: Vec<u8> = Vec::new();
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let mut window = std::mem::take(&mut carry);
        let start = base - window.len() as u64;
        window.extend_from_slice(&buf[..n]);
        for (i, w) in window.windows(SEVEN_Z.len()).enumerate() {
            if w == SEVEN_Z {
                found.push(start + i as u64);
            }
        }
        carry = window[window.len().saturating_sub(SEVEN_Z.len() - 1)..].to_vec();
        base += n as u64;
    }
    found.dedup();
    Ok(found)
}

fn extract_7z(archive: &Path, dest: &Path) -> Result<()> {
    let mut last = None;
    for offset in seven_z_offsets(archive)? {
        let source = Offset {
            file: File::open(archive)?,
            offset,
        };
        let mut reader =
            match sevenz_rust2::ArchiveReader::new(source, sevenz_rust2::Password::empty()) {
                Ok(r) => r,
                Err(e) => {
                    last = Some(e.to_string());
                    continue;
                }
            };
        let mut failure: Option<anyhow::Error> = None;
        let result = reader.for_each_entries(|entry, data| {
            let Some(path) = safe_join(dest, entry.name()) else {
                failure = Some(anyhow::anyhow!("unsafe path in archive: {}", entry.name()));
                return Ok(false);
            };
            if entry.is_directory() {
                if let Err(e) = prepare_dir(dest, &path) {
                    failure = Some(e);
                    return Ok(false);
                }
            } else {
                match create_file(dest, &path) {
                    Ok(mut file) => {
                        io::copy(data, &mut file)?;
                    }
                    Err(e) => {
                        failure = Some(e);
                        return Ok(false);
                    }
                }
            }
            Ok(true)
        });
        if let Some(e) = failure {
            return Err(e);
        }
        return result.map_err(|e| anyhow::anyhow!("7z: {e}"));
    }
    bail!("no 7z archive found ({})", last.unwrap_or_default())
}

/// The folder that `inner` designates inside an extracted tree.
pub fn pick_inner(extracted: &Path, inner: &Inner) -> Result<PathBuf> {
    let dirs = || -> Result<(Vec<PathBuf>, usize)> {
        let mut dirs = Vec::new();
        let mut total = 0;
        for entry in fs::read_dir(extracted)? {
            let entry = entry?;
            total += 1;
            if entry.file_type()?.is_dir() {
                dirs.push(entry.path());
            }
        }
        Ok((dirs, total))
    };
    match inner {
        Inner::Root => Ok(extracted.to_path_buf()),
        Inner::Named(name) => {
            let path = crate::fsx::join_rel(extracted, name);
            if !path.is_dir() {
                bail!("the archive has no folder {name}");
            }
            Ok(path)
        }
        Inner::SingleUnder(name) => {
            let sub = crate::fsx::join_rel(extracted, name);
            pick_inner(&sub, &Inner::Single).with_context(|| format!("in {name}"))
        }
        Inner::SingleThen(sub) => {
            let top = pick_inner(extracted, &Inner::Single)?;
            let path = crate::fsx::join_rel(&top, sub);
            if !path.is_dir() {
                bail!("the archive has no folder {sub} inside {}", top.display());
            }
            Ok(path)
        }
        Inner::Single => match dirs()? {
            (dirs, 1) if dirs.len() == 1 => Ok(dirs.into_iter().next().unwrap_or_default()),
            (_, n) => bail!("expected one folder at the top of the archive, found {n} items"),
        },
        Inner::Auto => match dirs()? {
            (dirs, 1) if dirs.len() == 1 => Ok(dirs.into_iter().next().unwrap_or_default()),
            _ => Ok(extracted.to_path_buf()),
        },
    }
}

fn stamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_default()
}

#[derive(Debug, thiserror::Error)]
#[error("{0} is in use: close the programs of the kit and try again")]
pub struct InUse(pub PathBuf);

/// Puts `new` (a complete folder on the same disk) in place of `dest`. The
/// old folder is renamed first. Windows also refuses that when a program of
/// the PC only keeps one of its folders open (an indexer, an antivirus, File
/// Explorer): then, if no program runs from it, the old tree moves out entry
/// by entry, and a folder that cannot move is emptied and stays, empty, for
/// the new tree to fill. A program running from it, or a file locked, stops
/// it: everything goes back and `InUse` is returned.
pub fn replace_dir(new: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let old = sibling(dest, "old");
    let had_old = dest.exists();
    if had_old && fs::rename(dest, &old).is_err() {
        // A running program could be moved too (Windows allows renaming a
        // running exe), and would then load the new files with the old ones.
        if !crate::os::processes::running_from(&crate::os::processes::all(), dest).is_empty() {
            return Err(InUse(dest.to_path_buf()).into());
        }
        if let Err(e) = move_tree(dest, &old) {
            crate::log::info(&format!("{}: {e}", dest.display()));
            let _ = move_tree(&old, dest);
            let _ = fs::remove_dir_all(&old);
            return Err(InUse(dest.to_path_buf()).into());
        }
    }
    // `dest` is gone, or only empty folders held open are left in it.
    let moved = retry_denied(|| {
        if dest.exists() {
            move_tree(new, dest).map(|()| {
                let _ = fs::remove_dir_all(new);
            })
        } else {
            fs::rename(new, dest)
        }
    });
    if let Err(e) = moved {
        if had_old {
            let _ = fs::remove_dir_all(dest);
            if fs::rename(&old, dest).is_err() {
                let _ = move_tree(&old, dest);
            }
        }
        return Err(e).with_context(|| format!("moving into {}", dest.display()));
    }
    if had_old {
        let _ = fs::remove_dir_all(&old);
    }
    Ok(())
}

/// Puts `tree` (a complete folder, in the PC's temporary folder or on the
/// kit's disk) in place of `dest`. On the same disk it is `replace_dir`. From
/// another disk, where a rename fails, the tree is first checked against the
/// destination's disk (`check_fits`, `check_space`), then copied next to
/// `dest` into `<dest>.devkit-new`, which `replace_dir` swaps in: an
/// interrupted copy leaves only that folder, which `clean_leftovers` removes,
/// and the installed version is never half replaced.
pub fn stage_into(tree: &Path, dest: &Path) -> Result<()> {
    let parent = dest.parent().context("no parent folder")?;
    fs::create_dir_all(parent)?;
    let same = crate::os::volume::same_volume(tree, parent);
    stage_with(tree, dest, same)
}

/// `stage_into` with the disk decision given (tests cannot make a second disk).
fn stage_with(tree: &Path, dest: &Path, same_volume: bool) -> Result<()> {
    if same_volume {
        return replace_dir(tree, dest);
    }
    check_fits(tree, dest)?;
    check_space(tree, dest)?;
    let staged = staged_path(dest);
    let copied = retry_denied(|| {
        // Each attempt starts from empty: a leftover of an interrupted copy,
        // or what the antivirus let through before it said no.
        if staged.exists() {
            fs::remove_dir_all(&staged)?;
        }
        copy_tree(tree, &staged)
    });
    if let Err(e) = copied {
        let _ = fs::remove_dir_all(&staged);
        return Err(e).with_context(|| format!("copying into {}", staged.display()));
    }
    let swapped = replace_dir(&staged, dest);
    if swapped.is_err() {
        let _ = fs::remove_dir_all(&staged);
    }
    swapped
}

/// `<dest>.devkit-new`, where a tree from another disk is copied first.
fn staged_path(dest: &Path) -> PathBuf {
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    dest.with_file_name(format!("{name}{STAGED}"))
}

const STAGED: &str = ".devkit-new";

/// Copies the folder `from` into `to` (created); hard links become separate
/// files. On Unix a symbolic link is made again with the same relative
/// target when that stays inside the copy (the rules of the extraction,
/// [`finish_links`]): `bin/python3 -> python3.12` of a tree installed from
/// a temporary folder on another disk. A link that leaves the tree is
/// skipped with a warning; on a disk without links the target is copied.
/// On Windows links are skipped, as in the extraction.
pub fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    let mut links = Vec::new();
    copy_entries(from, to, &mut links)?;
    #[cfg(unix)]
    finish_links_unix(to, links).map_err(io::Error::other)?;
    Ok(())
}

/// The files and folders of `copy_tree`; the links to make in `links`
/// (none on Windows).
#[cfg_attr(not(unix), allow(clippy::only_used_in_recursion))]
fn copy_entries(from: &Path, to: &Path, links: &mut Vec<(PathBuf, String)>) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = to.join(entry.file_name());
        if kind.is_symlink() {
            #[cfg(unix)]
            match fs::read_link(entry.path())?.to_str() {
                Some(t) => links.push((target, t.to_owned())),
                None => crate::log::warn(&format!(
                    "link not copied (target not UTF-8): {}",
                    entry.path().display()
                )),
            }
            continue;
        }
        if kind.is_dir() {
            copy_entries(&entry.path(), &target, links)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// The files of `tree` (a folder or one file) with their sizes; links are
/// skipped, as `copy_tree` does.
fn sizes(tree: &Path) -> io::Result<Vec<(PathBuf, u64)>> {
    let meta = fs::symlink_metadata(tree)?;
    if !meta.is_dir() {
        return Ok(vec![(tree.to_path_buf(), meta.len())]);
    }
    let mut out = Vec::new();
    let mut stack = vec![tree.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                stack.push(entry.path());
            } else {
                out.push((entry.path(), entry.metadata()?.len()));
            }
        }
    }
    Ok(out)
}

/// The nearest folder of `path` that exists: the disk of a destination not
/// created yet.
fn existing_ancestor(path: &Path) -> &Path {
    path.ancestors().find(|p| p.is_dir()).unwrap_or(path)
}

/// `1.5 GiB`, `20 MiB`, `300 KiB`, `12 bytes`.
fn human(bytes: u64) -> String {
    const UNITS: [(&str, u64); 3] = [("GiB", 1 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)];
    for (unit, size) in UNITS {
        if bytes >= size {
            return format!("{:.1} {unit}", bytes as f64 / size as f64);
        }
    }
    format!("{bytes} bytes")
}

/// Every file of `tree` fits on the disk of `dest`: FAT32 holds no file of
/// 4 GiB or more. Checked before the installed version is touched. When the
/// disk cannot be read, nothing is refused (a copy that fails still does).
pub fn check_fits(tree: &Path, dest: &Path) -> Result<()> {
    match crate::os::volume::info(existing_ancestor(dest)) {
        Ok(v) => fits(tree, v.fs.max_file_size(), v.fs.as_str()),
        Err(e) => {
            crate::log::warn(&format!("{}: {e}", dest.display()));
            Ok(())
        }
    }
}

/// `check_fits` with the disk's limit given.
fn fits(tree: &Path, limit: Option<u64>, fs_name: &str) -> Result<()> {
    let Some(limit) = limit else {
        return Ok(());
    };
    if let Some((file, size)) = sizes(tree)?.into_iter().find(|(_, s)| *s > limit) {
        let name = file.strip_prefix(tree).unwrap_or(&file);
        let name = if name.as_os_str().is_empty() {
            &file
        } else {
            name
        };
        bail!(
            "{} is {} ({size} bytes): too large for the kit's disk ({fs_name}, at most {} per file)",
            name.display(),
            human(size),
            human(limit)
        );
    }
    Ok(())
}

/// The disk of `dest` has room for a copy of `tree`. When the disk cannot be
/// read, nothing is refused (a copy that fails still does).
pub fn check_space(tree: &Path, dest: &Path) -> Result<()> {
    match crate::os::volume::free_space(existing_ancestor(dest)) {
        Ok(free) => has_space(tree, free, dest),
        Err(e) => {
            crate::log::warn(&format!("{}: {e}", dest.display()));
            Ok(())
        }
    }
}

/// `check_space` with the free bytes given.
fn has_space(tree: &Path, free: u64, dest: &Path) -> Result<()> {
    let needed: u64 = sizes(tree)?.iter().map(|(_, s)| s).sum();
    if needed > free {
        bail!(
            "not enough space for {}: {} needed, {} free",
            dest.display(),
            human(needed),
            human(free)
        );
    }
    Ok(())
}

/// Runs `op` again while Windows answers "access denied", up to 30 s: the
/// antivirus scans the files just extracted and, while it holds them, their
/// folder cannot move (Java and Kotlin failed so on their first install).
fn retry_denied(mut op: impl FnMut() -> io::Result<()>) -> io::Result<()> {
    let start = Instant::now();
    loop {
        match op() {
            Err(e)
                if e.kind() == io::ErrorKind::PermissionDenied
                    && start.elapsed() < Duration::from_secs(30) =>
            {
                std::thread::sleep(Duration::from_millis(250));
            }
            other => return other,
        }
    }
}

/// Moves the content of `from` into `to` (created if needed), entry by
/// entry: a folder that cannot be renamed is entered and emptied, and stays
/// where it is if Windows does not let it go. A file that cannot move is an
/// error (in use).
fn move_tree(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let source = entry.path();
        let target = to.join(entry.file_name());
        let is_dir = entry.file_type()?.is_dir();
        if is_dir && target.is_dir() {
            move_tree(&source, &target)?;
            let _ = fs::remove_dir(&source);
            continue;
        }
        if let Err(e) = fs::rename(&source, &target) {
            if !is_dir {
                return Err(e);
            }
            move_tree(&source, &target)?;
            let _ = fs::remove_dir(&source);
        }
    }
    Ok(())
}

/// Removes a component folder: renamed first, so a folder in use is left
/// untouched (`InUse`).
/// Puts `new` in place of the file `dest`. The old file is renamed to
/// `<name>.old` first: Windows allows renaming a running exe, so a program in
/// use can still be updated (`Replace-File` of the kit). Leftover `.old`
/// files are deleted when possible.
pub fn replace_file(new: &Path, dest: &Path) -> Result<()> {
    let old = dest.with_file_name(format!(
        "{}.old",
        dest.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    ));
    let _ = fs::remove_file(&old);
    // Copied next to `dest` first, not renamed: `new` may be on another disk
    // (PC temp folder), and an interrupted copy leaves only `<name>.devkit-new`.
    let staged = staged_path(dest);
    retry_denied(|| fs::copy(new, &staged).map(|_| ()))
        .with_context(|| format!("writing {}", staged.display()))?;
    let had_old = dest.exists();
    if had_old && fs::rename(dest, &old).is_err() {
        let _ = fs::remove_file(&staged);
        return Err(InUse(dest.to_path_buf()).into());
    }
    if let Err(e) = fs::rename(&staged, dest) {
        let _ = fs::remove_file(&staged);
        if had_old {
            let _ = fs::rename(&old, dest);
        }
        return Err(e).with_context(|| format!("writing {}", dest.display()));
    }
    let _ = fs::remove_file(&old);
    Ok(())
}

pub fn remove_dir(dest: &Path) -> Result<()> {
    if !dest.exists() {
        return Ok(());
    }
    let old = sibling(dest, "old");
    fs::rename(dest, &old).map_err(|_| InUse(dest.to_path_buf()))?;
    let _ = fs::remove_dir_all(&old);
    Ok(())
}

fn sibling(dest: &Path, tag: &str) -> PathBuf {
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    dest.with_file_name(format!("{name}.{tag}-{}", stamp()))
}

/// Deletes leftovers of interrupted swaps (`*.old-*`, `*.new-*`) and copies
/// (`*.devkit-new`, `stage_into`) in `dir`.
pub fn clean_leftovers(dir: &Path) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.contains(".old-") || name.contains(".new-") || name.ends_with(STAGED) {
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                let _ = fs::remove_dir_all(entry.path());
            } else {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../test/rust/net/archive.rs"]
mod tests;
