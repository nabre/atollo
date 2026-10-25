//! `flock`, the lock of a file on macOS and Linux: exclusive, held while the
//! file stays open and dropped by the system when the process dies; the
//! kit's locks (`platform::lock`) are one on their file, the name is not
//! used.

use std::collections::BTreeSet;
use std::fs::File;
use std::io;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

/// Takes the exclusive lock of `file` without waiting: `WouldBlock` when
/// another open file holds it.
pub fn try_lock(file: &File) -> io::Result<()> {
    // SAFETY: a valid descriptor owned by `file`.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// The locked file, and its place in the set of this process's locks.
pub struct Held {
    path: PathBuf,
    _file: File,
}

impl Drop for Held {
    fn drop(&mut self) {
        unclaim(&self.path);
    }
}

/// The files this process holds the lock of: a second taker in this
/// process is refused before it opens the file.
static TAKEN: Mutex<BTreeSet<PathBuf>> = Mutex::new(BTreeSet::new());

/// The lock of `file` for this process, or `None` while a thread of this
/// process holds it.
fn claim(file: &Path) -> Option<PathBuf> {
    let path = file.to_path_buf();
    TAKEN
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(path.clone())
        .then_some(path)
}

fn unclaim(path: &Path) {
    TAKEN
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .remove(path);
}

fn open(file: &Path) -> Result<File> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).with_context(|| dir.display().to_string())?;
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(file)
        .with_context(|| file.display().to_string())
}

pub fn try_take(_name: &str, file: &Path) -> Result<Option<Held>> {
    let Some(path) = claim(file) else {
        return Ok(None);
    };
    let locked = open(file).and_then(|f| match try_lock(&f) {
        Ok(()) => Ok(Some(f)),
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(e).with_context(|| format!("flock {}", file.display())),
    });
    match locked {
        Ok(Some(f)) => Ok(Some(Held { path, _file: f })),
        other => {
            unclaim(&path);
            other.map(|_| None)
        }
    }
}

/// Tries every 50 ms for at most `timeout`.
pub fn wait(name: &str, file: &Path, timeout: Duration) -> Result<Option<Held>> {
    let start = Instant::now();
    loop {
        if let Some(held) = try_take(name, file)? {
            return Ok(Some(held));
        }
        if start.elapsed() > timeout {
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
