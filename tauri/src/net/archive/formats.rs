//! The archive formats: zip, `.tar.gz`, `.tar.xz` and 7z (also a self-extracting `.7z.exe`).

use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use super::links::{finish_links_here, hard_link_or_copy};
use super::{create_file, no_link_on_the_way, prepare_dir, safe_join, set_executable};

pub(super) fn extract_zip(archive: &Path, dest: &Path) -> Result<()> {
    let mut zip = zip::ZipArchive::new(File::open(archive)?)?;
    let mut links: Vec<(PathBuf, String)> = Vec::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(path) = safe_join(dest, entry.name()) else {
            bail!("unsafe path in archive: {}", entry.name());
        };
        if entry.is_dir() {
            prepare_dir(dest, &path)?;
        } else if entry.is_symlink() {
            let mut target = String::new();
            if entry.read_to_string(&mut target).is_ok() {
                links.push((path, target.trim().to_owned()));
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
    finish_links_here(dest, links)?;
    Ok(())
}

pub(super) fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<()> {
    let gz = flate2::read::GzDecoder::new(io::BufReader::new(File::open(archive)?));
    extract_tar_stream(gz, dest)
}

/// Memory the xz decoder may take, in KiB (1 GiB; `xz -9` needs 65 MiB).
const XZ_MEM_LIMIT_KIB: u32 = 1 << 20;

pub(super) fn extract_tar_xz(archive: &Path, dest: &Path) -> Result<()> {
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
            tar::EntryType::Symlink => {
                if let Ok(Some(target)) = entry.link_name() {
                    links.push((path, target.to_string_lossy().into_owned()));
                }
            }
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
                            hard_link_or_copy(&target_path, &path, |t, p| fs::hard_link(t, p))?;
                        } else {
                            crate::log::warn(&format!(
                                "hard link not made (not a file inside the folder): {} -> {}",
                                path.display(),
                                target.display()
                            ));
                        }
                    }
                }
            }
            // Devices, pax headers: never followed or created.
            _ => {}
        }
    }
    finish_links_here(dest, links)?;
    Ok(())
}

/// 7z signature: `7z\xBC\xAF\x27\x1C`.
pub(super) const SEVEN_Z: [u8; 6] = [b'7', b'z', 0xBC, 0xAF, 0x27, 0x1C];

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
pub(super) fn seven_z_offsets(path: &Path) -> Result<Vec<u64>> {
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

pub(super) fn extract_7z(archive: &Path, dest: &Path) -> Result<()> {
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
