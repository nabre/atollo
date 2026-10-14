//! Archives and folder swaps for the installer (port of nabre/atollo
//! `Expand-Package`, `Replace-Dir` and `Replace-File`).
//!
//! Extraction refuses entries that would land outside the target (absolute
//! paths, `..`, drive prefixes) and skips links. File times are not restored
//! (exFAT rejects them; `tar -m` in the kit). Swaps are atomic from the
//! user's point of view: the new folder is complete before the old one moves,
//! and a failure puts the old one back.

use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Zip,
    TarGz,
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
    #[allow(dead_code, reason = "used by the language installers of phase 4")]
    Single,
    /// The only top-level folder if there is exactly one, else the root.
    Auto,
    /// A named subfolder.
    Named(String),
    /// The only folder inside a named subfolder (`package/vendor/<target>`).
    SingleUnder(String),
}

/// `base` + an archive entry name, or `None` if the name would escape `base`.
pub fn safe_join(base: &Path, name: &str) -> Option<PathBuf> {
    let mut out = base.to_path_buf();
    let mut depth = 0usize;
    for part in Path::new(&name.replace('/', "\\")).components() {
        match part {
            Component::Normal(p) => {
                out.push(p);
                depth += 1;
            }
            Component::CurDir => {}
            _ => return None,
        }
    }
    (depth > 0).then_some(out)
}

pub fn extract(archive: &Path, kind: Kind, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    match kind {
        Kind::Zip => extract_zip(archive, dest),
        Kind::TarGz => extract_tar_gz(archive, dest),
        Kind::SevenZip => extract_7z(archive, dest),
        Kind::File => {
            let name = archive.file_name().context("file name")?;
            fs::copy(archive, dest.join(name))
                .map(|_| ())
                .map_err(Into::into)
        }
    }
    .with_context(|| format!("extracting {}", archive.display()))
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<()> {
    let mut zip = zip::ZipArchive::new(File::open(archive)?)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(path) = safe_join(dest, entry.name()) else {
            bail!("unsafe path in archive: {}", entry.name());
        };
        if entry.is_dir() {
            fs::create_dir_all(&path)?;
        } else if entry.is_symlink() {
            continue;
        } else {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir)?;
            }
            io::copy(&mut entry, &mut File::create(&path)?)?;
        }
    }
    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<()> {
    let gz = flate2::read::GzDecoder::new(io::BufReader::new(File::open(archive)?));
    let mut tar = tar::Archive::new(gz);
    tar.set_preserve_mtime(false);
    tar.set_preserve_permissions(false);
    for entry in tar.entries()? {
        let mut entry = entry?;
        let name = entry.path()?.to_string_lossy().into_owned();
        let Some(path) = safe_join(dest, &name) else {
            if name.trim_matches(['.', '/']).is_empty() {
                continue; // "./"
            }
            bail!("unsafe path in archive: {name}");
        };
        match entry.header().entry_type() {
            tar::EntryType::Directory => fs::create_dir_all(&path)?,
            tar::EntryType::Regular | tar::EntryType::Continuous => {
                if let Some(dir) = path.parent() {
                    fs::create_dir_all(dir)?;
                }
                io::copy(&mut entry, &mut File::create(&path)?)?;
            }
            // Links, devices, pax headers: never followed or created.
            _ => {}
        }
    }
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
                fs::create_dir_all(&path)?;
            } else {
                if let Some(dir) = path.parent() {
                    fs::create_dir_all(dir)?;
                }
                io::copy(data, &mut File::create(&path)?)?;
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
            let path = extracted.join(name);
            if !path.is_dir() {
                bail!("the archive has no folder {name}");
            }
            Ok(path)
        }
        Inner::SingleUnder(name) => {
            let sub = extracted.join(name.replace('/', "\\"));
            pick_inner(&sub, &Inner::Single).with_context(|| format!("in {name}"))
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
        if !crate::processes::running_from(&crate::processes::all(), dest).is_empty() {
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
    let moved = if dest.exists() {
        move_tree(new, dest).map(|()| {
            let _ = fs::remove_dir_all(new);
        })
    } else {
        fs::rename(new, dest)
    };
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
    let had_old = dest.exists();
    if had_old {
        fs::rename(dest, &old).map_err(|_| InUse(dest.to_path_buf()))?;
    }
    // Copy, not rename: `new` may be on another disk (PC temp folder).
    if let Err(e) = fs::copy(new, dest) {
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

/// Deletes leftovers of interrupted swaps (`*.old-*`, `*.new-*`) in `dir`.
pub fn clean_leftovers(dir: &Path) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.contains(".old-") || name.contains(".new-") {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::tests::TempDir;
    use std::io::Write;

    #[test]
    fn unsafe_names_are_refused() {
        let base = Path::new(r"D:\x");
        assert_eq!(
            safe_join(base, "a/b.txt"),
            Some(PathBuf::from(r"D:\x\a\b.txt"))
        );
        assert_eq!(safe_join(base, "./a"), Some(PathBuf::from(r"D:\x\a")));
        for bad in [
            "../evil",
            "a/../../evil",
            "/abs",
            r"C:\abs",
            "C:rel",
            r"\\server\share\x",
            "",
        ] {
            assert_eq!(safe_join(base, bad), None, "{bad}");
        }
    }

    fn make_zip(path: &Path, files: &[(&str, &str)]) {
        let mut zip = zip::ZipWriter::new(File::create(path).unwrap());
        for (name, content) in files {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }

    #[test]
    fn zip_extract_and_inner() {
        let tmp = TempDir::new("zip");
        let archive = tmp.0.join("a.zip");
        make_zip(
            &archive,
            &[("top-1.0/bin/tool.exe", "x"), ("top-1.0/readme.txt", "r")],
        );
        let out = tmp.0.join("out");
        extract(&archive, Kind::Zip, &out).unwrap();
        let inner = pick_inner(&out, &Inner::Single).unwrap();
        assert!(inner.join("bin").join("tool.exe").is_file());
        assert_eq!(pick_inner(&out, &Inner::Auto).unwrap(), inner);
        assert_eq!(pick_inner(&out, &Inner::Root).unwrap(), out);

        let flat = tmp.0.join("flat.zip");
        make_zip(&flat, &[("pwsh.exe", "x"), ("Modules/a.psd1", "m")]);
        let out2 = tmp.0.join("out2");
        extract(&flat, Kind::Zip, &out2).unwrap();
        assert!(pick_inner(&out2, &Inner::Single).is_err());
        assert_eq!(pick_inner(&out2, &Inner::Auto).unwrap(), out2);
    }

    #[test]
    fn zip_slip_is_refused() {
        let tmp = TempDir::new("zipslip");
        let archive = tmp.0.join("evil.zip");
        make_zip(&archive, &[("../evil.txt", "x")]);
        assert!(extract(&archive, Kind::Zip, &tmp.0.join("out")).is_err());
        assert!(!tmp.0.join("evil.txt").exists());
    }

    #[test]
    fn tar_gz_extract() {
        let tmp = TempDir::new("targz");
        let archive = tmp.0.join("a.tar.gz");
        {
            let gz = flate2::write::GzEncoder::new(
                File::create(&archive).unwrap(),
                flate2::Compression::fast(),
            );
            let mut tar = tar::Builder::new(gz);
            let mut header = tar::Header::new_gnu();
            header.set_size(5);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, "server/bin/run.cmd", &b"hello"[..])
                .unwrap();
            tar.into_inner().unwrap().finish().unwrap();
        }
        let out = tmp.0.join("out");
        extract(&archive, Kind::TarGz, &out).unwrap();
        assert_eq!(
            fs::read_to_string(out.join(r"server\bin\run.cmd")).unwrap(),
            "hello"
        );
    }

    #[test]
    fn seven_z_signature_search_spans_buffers() {
        let tmp = TempDir::new("7zsig");
        let file = tmp.0.join("stub.exe");
        let mut bytes = vec![0u8; 1024 * 1024 - 3];
        bytes.extend_from_slice(&SEVEN_Z);
        bytes.extend_from_slice(b"rest");
        fs::write(&file, &bytes).unwrap();
        assert_eq!(seven_z_offsets(&file).unwrap(), vec![1024 * 1024 - 3]);
    }

    #[test]
    fn replace_file_works_while_in_use() {
        let tmp = TempDir::new("replfile");
        let dest = tmp.0.join("tool.exe");
        fs::write(&dest, "1").unwrap();
        let held = File::open(&dest).unwrap();
        let new = tmp.0.join("new.exe");
        fs::write(&new, "2").unwrap();
        replace_file(&new, &dest).unwrap();
        assert_eq!(fs::read_to_string(&dest).unwrap(), "2");
        drop(held);
        fs::write(&new, "3").unwrap();
        replace_file(&new, &dest).unwrap();
        assert!(
            !tmp.0.join("tool.exe.old").exists(),
            "old removed once free"
        );
    }

    /// A program of the PC keeps a folder of the tree open (as Cloud Code's
    /// indexer did with `extensions\terminal-suggest` of the editor): the
    /// swap still works. A file really in use stops it, and nothing changes.
    #[test]
    fn replace_dir_with_a_folder_held_open() {
        use std::os::windows::fs::OpenOptionsExt;
        const SHARE_READ_WRITE: u32 = 1 | 2;
        const BACKUP_SEMANTICS: u32 = 0x0200_0000;
        let tmp = TempDir::new("replheld");
        let dest = tmp.0.join("editor");
        let make = |root: &Path, text: &str| {
            fs::create_dir_all(root.join(r"extensions\held")).unwrap();
            fs::write(root.join(r"extensions\held\a.txt"), text).unwrap();
            fs::write(root.join("node.exe"), text).unwrap();
        };
        make(&dest, "old");
        fs::write(dest.join("gone.txt"), "old").unwrap();
        let held = fs::OpenOptions::new()
            .read(true)
            .share_mode(SHARE_READ_WRITE)
            .custom_flags(BACKUP_SEMANTICS)
            .open(dest.join(r"extensions\held"))
            .unwrap();
        assert!(
            fs::rename(&dest, tmp.0.join("x")).is_err(),
            "Windows refuses the whole folder"
        );

        let new = tmp.0.join("new");
        make(&new, "new");
        fs::create_dir_all(new.join("out")).unwrap();
        replace_dir(&new, &dest).unwrap();
        assert_eq!(fs::read_to_string(dest.join("node.exe")).unwrap(), "new");
        assert_eq!(
            fs::read_to_string(dest.join(r"extensions\held\a.txt")).unwrap(),
            "new"
        );
        assert!(dest.join("out").is_dir() && !dest.join("gone.txt").exists());
        assert!(!new.exists());
        drop(held);
        let names: Vec<String> = fs::read_dir(&tmp.0)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            names.iter().all(|n| !n.starts_with("editor.old")),
            "{names:?}"
        );

        // A file really in use: InUse, and the folder is as before.
        let busy = fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(dest.join("node.exe"))
            .unwrap();
        let newer = tmp.0.join("newer");
        make(&newer, "newer");
        let err = replace_dir(&newer, &dest).unwrap_err();
        assert!(err.is::<InUse>(), "{err:#}");
        drop(busy);
        assert_eq!(fs::read_to_string(dest.join("node.exe")).unwrap(), "new");
        assert_eq!(
            fs::read_to_string(dest.join(r"extensions\held\a.txt")).unwrap(),
            "new"
        );
        assert!(dest.join("out").is_dir());
    }

    #[test]
    fn plain_file_kind() {
        let tmp = TempDir::new("kindfile");
        let f = tmp.0.join("agy.exe");
        fs::write(&f, "x").unwrap();
        extract(&f, Kind::File, &tmp.0.join("out")).unwrap();
        assert!(tmp.0.join("out").join("agy.exe").is_file());
    }

    #[test]
    fn replace_and_remove() {
        let tmp = TempDir::new("replace");
        let dest = tmp.0.join("apps").join("tool");
        let new1 = tmp.0.join("new1");
        fs::create_dir_all(&new1).unwrap();
        fs::write(new1.join("v.txt"), "1").unwrap();
        replace_dir(&new1, &dest).unwrap();
        assert_eq!(fs::read_to_string(dest.join("v.txt")).unwrap(), "1");

        let new2 = tmp.0.join("new2");
        fs::create_dir_all(&new2).unwrap();
        fs::write(new2.join("v.txt"), "2").unwrap();
        replace_dir(&new2, &dest).unwrap();
        assert_eq!(fs::read_to_string(dest.join("v.txt")).unwrap(), "2");
        let names: Vec<_> = fs::read_dir(dest.parent().unwrap())
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect();
        assert_eq!(
            names,
            vec![std::ffi::OsString::from("tool")],
            "old folder removed"
        );

        // A file locked (open without FILE_SHARE_DELETE) blocks the swap and
        // leaves everything as it was.
        let held = {
            use std::os::windows::fs::OpenOptionsExt;
            fs::OpenOptions::new()
                .read(true)
                .share_mode(1)
                .open(dest.join("v.txt"))
                .unwrap()
        };
        let new3 = tmp.0.join("new3");
        fs::create_dir_all(&new3).unwrap();
        let err = replace_dir(&new3, &dest).unwrap_err();
        assert!(err.is::<InUse>(), "{err:#}");
        assert!(new3.exists());
        assert!(remove_dir(&dest).unwrap_err().is::<InUse>());
        drop(held);
        remove_dir(&dest).unwrap();
        assert!(!dest.exists());
    }
}
