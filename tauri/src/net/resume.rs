//! Downloads of large files (the local models, several GB): kept in
//! `<file>.part` when they fail or are cancelled, and resumed from where they
//! stopped with an HTTP `Range` at the next attempt, also in a later run.
//! The SHA-256 of the whole file is computed while the bytes arrive (the
//! part already on disk is read once first), so the file is never read
//! again at the end; a file that does not match is deleted.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

use super::{copy_body, Cancelled, Net, STALL};

/// Attempts in a row that bring no new byte before the download is given up;
/// an attempt that moved forward starts the count again.
const ATTEMPTS: usize = 5;

/// What an answer to a request from `offset` means for the part on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Resume {
    /// `206` from `offset`: the bytes are appended.
    Append,
    /// `200` (the server ignores `Range`) or a `206` from another byte: the
    /// part is started again.
    Restart,
    /// `416` with the part as long as the whole file: nothing left to get.
    Complete,
}

/// How to use the answer `status` with `Content-Range` `range` to a request
/// sent with the part at `offset` bytes.
pub(crate) fn resume_of(status: u16, range: Option<&str>, offset: u64) -> Resume {
    match status {
        206 => match range.and_then(range_start) {
            Some((start, _)) if start == offset => Resume::Append,
            _ => Resume::Restart,
        },
        416 => match range.and_then(|r| r.trim().strip_prefix("bytes */")) {
            Some(total) if total.trim().parse::<u64>().ok() == Some(offset) && offset > 0 => {
                Resume::Complete
            }
            _ => Resume::Restart,
        },
        _ => Resume::Restart,
    }
}

/// `bytes <start>-<end>/<total>`: the first byte and the total, when known.
fn range_start(range: &str) -> Option<(u64, Option<u64>)> {
    let rest = range.trim().strip_prefix("bytes ")?;
    let (span, total) = rest.split_once('/')?;
    let start = span.split_once('-')?.0.trim().parse().ok()?;
    Some((start, total.trim().parse().ok()))
}

/// `<dest>.part`.
pub fn part_of(dest: &Path) -> PathBuf {
    let mut name = dest.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    dest.with_file_name(name)
}

/// Bytes already downloaded for `dest` (its `.part`), 0 without one.
pub fn downloaded(dest: &Path) -> u64 {
    fs::metadata(part_of(dest)).map_or(0, |m| m.len())
}

/// A writer that hashes what it writes.
struct Hashing<'a> {
    file: &'a mut File,
    hash: &'a mut Sha256,
    /// Bytes written and hashed.
    count: u64,
}

impl Write for Hashing<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.file.write(buf)?;
        self.hash.update(&buf[..n]);
        self.count += n as u64;
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

/// The part on disk read into `hash`; `progress(done)` while it is read.
fn hash_part(
    part: &Path,
    hash: &mut Sha256,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64),
) -> Result<u64> {
    let mut file = File::open(part).with_context(|| part.display().to_string())?;
    let mut buf = vec![0u8; 1024 * 1024];
    let mut done = 0u64;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(Cancelled.into());
        }
        let n = file.read(&mut buf)?;
        if n == 0 {
            return Ok(done);
        }
        hash.update(&buf[..n]);
        done += n as u64;
        progress(done);
    }
}

impl Net {
    /// Downloads `url` to `dest` through `<dest>.part`, resuming a part left
    /// by an earlier attempt, and checks its SHA-256 (`sha256`, hex) and its
    /// size (`size`, when known). `headers` go to `url` only (never to the
    /// address it redirects to: ureq leaves out `Authorization` there).
    /// `progress(done, total)` while bytes arrive. Cancel keeps the part.
    #[allow(clippy::too_many_arguments)]
    pub fn download_resumable(
        &self,
        url: &str,
        dest: &Path,
        sha256: &str,
        size: Option<u64>,
        headers: &[(&str, &str)],
        cancel: &AtomicBool,
        progress: &mut dyn FnMut(u64, Option<u64>),
    ) -> Result<()> {
        let part = part_of(dest);
        if let Some(dir) = dest.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut hash = Sha256::new();
        let mut offset = 0;
        if part.is_file() {
            offset = hash_part(&part, &mut hash, cancel, &mut |done| progress(done, size))?;
        }
        if size.is_some_and(|s| offset > s) {
            // Longer than the file: not a part of it.
            fs::remove_file(&part)?;
            (offset, hash) = (0, Sha256::new());
        }
        let mut idle = 0;
        let mut wait = Duration::from_secs(5);
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(Cancelled.into());
            }
            if size == Some(offset) {
                break;
            }
            let before = offset;
            match self.resume_once(
                url,
                &part,
                &mut offset,
                &mut hash,
                headers,
                cancel,
                &mut |done, total| progress(done, total.or(size)),
            ) {
                Ok(()) => break,
                Err(e) if e.is::<Cancelled>() => return Err(e),
                Err(e) if super::is_permanent(&e) => return Err(e),
                Err(e) => {
                    idle = if offset > before { 0 } else { idle + 1 };
                    if idle >= ATTEMPTS {
                        return Err(e);
                    }
                    crate::log::warn(&format!(
                        "download of {url} stopped at {offset} bytes: {e:#}; trying again"
                    ));
                    std::thread::sleep(wait);
                    wait = (wait * 2).min(Duration::from_secs(60));
                }
            }
        }
        if let Some(size) = size {
            if offset != size {
                bail!("download of {url}: {offset} bytes instead of {size}");
            }
        }
        let got = hex(&hash.finalize());
        if !got.eq_ignore_ascii_case(sha256.trim()) {
            let _ = fs::remove_file(&part);
            bail!(
                "{}: the checksum does not match, the file was deleted",
                dest.file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default()
            );
        }
        fs::rename(&part, dest).with_context(|| dest.display().to_string())
    }

    /// One request from `offset`: the bytes appended to the part (or the
    /// part started again), `offset` and `hash` kept up to date also when
    /// it fails halfway.
    #[allow(clippy::too_many_arguments)]
    fn resume_once(
        &self,
        url: &str,
        part: &Path,
        offset: &mut u64,
        hash: &mut Sha256,
        headers: &[(&str, &str)],
        cancel: &AtomicBool,
        progress: &mut dyn FnMut(u64, Option<u64>),
    ) -> Result<()> {
        let mut request = self.agent.get(url);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        if *offset > 0 {
            request = request.header("Range", format!("bytes={offset}-"));
        }
        let response = request
            .config()
            .http_status_as_error(false)
            .timeout_recv_body(None)
            .build()
            .call()
            .with_context(|| format!("GET {url}"))?;
        let status = response.status().as_u16();
        let header = |name: &str| {
            response
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let range = header("content-range");
        if !(200..300).contains(&status) && status != 416 {
            return Err(
                anyhow::Error::new(ureq::Error::StatusCode(status)).context(format!("GET {url}"))
            );
        }
        let resume = if *offset == 0 && status == 200 {
            Resume::Append
        } else {
            resume_of(status, range.as_deref(), *offset)
        };
        let mut file = match resume {
            Resume::Complete => return Ok(()),
            Resume::Append => {
                // Written, not appended: an append handle cannot be cut
                // (`set_len`) on Windows.
                let mut file = OpenOptions::new()
                    .create(true)
                    .truncate(false)
                    .write(true)
                    .open(part)
                    .with_context(|| part.display().to_string())?;
                file.set_len(*offset)?;
                file.seek(SeekFrom::Start(*offset))?;
                file
            }
            Resume::Restart => {
                if status == 416 {
                    // The part is not what the server has: from the start.
                    let _ = fs::remove_file(part);
                    (*offset, *hash) = (0, Sha256::new());
                    bail!("GET {url}: range refused, the download starts again");
                }
                (*offset, *hash) = (0, Sha256::new());
                File::create(part).with_context(|| part.display().to_string())?
            }
        };
        let total = range
            .as_deref()
            .and_then(range_start)
            .and_then(|(_, total)| total)
            .or_else(|| {
                header("content-length")
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(|n| n + *offset)
            });
        let start = *offset;
        let body = response.into_body().into_reader();
        let mut out = Hashing {
            file: &mut file,
            hash,
            count: 0,
        };
        let copied = copy_body(body, &mut out, cancel, STALL, &mut |done| {
            progress(start + done, total)
        });
        // What reached the disk and the hash counts also when the copy
        // failed: the part is cut to it, so the next request goes on from
        // there (a write cut halfway leaves no byte out of the hash).
        let written = start + out.count;
        out.flush()?;
        file.set_len(written)?;
        *offset = written;
        copied.with_context(|| format!("downloading {url}"))?;
        if let Some(total) = total {
            if written != total {
                bail!("download of {url} stopped at {written} of {total} bytes");
            }
        }
        Ok(())
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
#[path = "../../../test/rust/net/resume.rs"]
mod tests;
