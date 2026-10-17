//! One running app per kit (DECISIONS.md D-08).
//!
//! A named mutex `Local\Atollo-App-<kit id>` marks the running app; a second
//! start of the same kit sends a message through the pipe of the same name
//! (for now only "focus") and exits. Two different kits (two USB sticks) have
//! different ids and run side by side. The pipe rejects remote clients and
//! keeps the default security of the user's session.
//!
//! macOS and Linux: an exclusive `flock` on `data/app/instance.lock` marks
//! the running app and a Unix socket `data/app/instance.sock` carries the
//! message. Both live inside the kit, not in the system's temporary folder:
//! the kit is portable, and its id names nothing outside it there.

#[cfg(windows)]
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

use anyhow::{bail, Context, Result};
#[cfg(windows)]
use windows::core::HSTRING;
#[cfg(windows)]
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, ERROR_PIPE_CONNECTED, HANDLE,
};
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::{ReadFile, PIPE_ACCESS_INBOUND};
#[cfg(windows)]
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE,
    PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
#[cfg(windows)]
use windows::Win32::System::Threading::CreateMutexW;

use crate::kit::Kit;

pub enum Instance {
    /// This process is the app of the kit; keep the guard alive.
    Primary(MutexGuard),
    /// Another process already is.
    Secondary,
}

#[cfg(windows)]
pub struct MutexGuard(HANDLE);

/// The locked `instance.lock`: the lock ends when the file closes.
#[cfg(unix)]
pub struct MutexGuard(#[allow(dead_code)] std::fs::File);

// SAFETY: a mutex handle is a kernel handle usable from any thread.
#[cfg(windows)]
unsafe impl Send for MutexGuard {}
#[cfg(windows)]
unsafe impl Sync for MutexGuard {}

#[cfg(windows)]
impl Drop for MutexGuard {
    fn drop(&mut self) {
        // SAFETY: handle from CreateMutexW, closed once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

#[cfg(windows)]
fn name(kit: &Kit) -> String {
    format!("Atollo-App-{}", kit.id())
}

#[cfg(windows)]
pub fn acquire(kit: &Kit) -> Result<Instance> {
    let mutex = HSTRING::from(format!(r"Local\{}", name(kit)));
    // SAFETY: valid name; default security; not initially owned.
    let handle = unsafe { CreateMutexW(None, false, &mutex) }.context("CreateMutexW")?;
    // SAFETY: reads the thread's last error, set by CreateMutexW.
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        // SAFETY: handle from CreateMutexW.
        let _ = unsafe { CloseHandle(handle) };
        return Ok(Instance::Secondary);
    }
    Ok(Instance::Primary(MutexGuard(handle)))
}

#[cfg(unix)]
pub fn acquire(kit: &Kit) -> Result<Instance> {
    use std::os::fd::AsRawFd;
    let dir = kit.app_dir();
    std::fs::create_dir_all(&dir).with_context(|| dir.display().to_string())?;
    let path = dir.join("instance.lock");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .with_context(|| path.display().to_string())?;
    // SAFETY: a valid descriptor owned by `file`.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        let e = std::io::Error::last_os_error();
        if e.kind() == std::io::ErrorKind::WouldBlock {
            return Ok(Instance::Secondary);
        }
        return Err(e).with_context(|| format!("flock {}", path.display()));
    }
    Ok(Instance::Primary(MutexGuard(file)))
}

#[cfg(windows)]
fn pipe_path(kit: &Kit) -> String {
    format!(r"\\.\pipe\{}", name(kit))
}

#[cfg(unix)]
fn socket_path(kit: &Kit) -> std::path::PathBuf {
    kit.app_dir().join("instance.sock")
}

/// Sends `message` to the running app of the kit (retries while it starts).
pub fn notify_primary(kit: &Kit, message: &str) -> Result<()> {
    #[cfg(windows)]
    let (path, open) = (pipe_path(kit), |p: &String| {
        OpenOptions::new().write(true).open(p)
    });
    #[cfg(unix)]
    let (path, open) = (socket_path(kit).display().to_string(), |p: &String| {
        std::os::unix::net::UnixStream::connect(p)
    });
    let mut last = None;
    for _ in 0..20 {
        match open(&path) {
            Ok(mut pipe) => {
                pipe.write_all(message.as_bytes())?;
                return Ok(());
            }
            Err(e) => last = Some(e),
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    match last {
        Some(e) => Err(e).with_context(|| format!("opening {path}")),
        None => bail!("opening {path}"),
    }
}

/// Serves the socket on a background thread; `on_message` gets each message.
/// Called by the primary only: a socket left by a dead app is replaced.
#[cfg(unix)]
pub fn listen(kit: &Kit, on_message: impl Fn(String) + Send + 'static) {
    use std::io::Read;
    use std::os::unix::net::UnixListener;
    let path = socket_path(kit);
    let _ = std::fs::remove_file(&path);
    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            crate::log::warn(&format!("{}: {e}", path.display()));
            return;
        }
    };
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut buf = Vec::new();
            if (&mut stream).take(4096).read_to_end(&mut buf).is_ok() {
                on_message(String::from_utf8_lossy(&buf).into_owned());
            }
        }
    });
}

/// Serves the pipe on a background thread; `on_message` gets each message.
#[cfg(windows)]
pub fn listen(kit: &Kit, on_message: impl Fn(String) + Send + 'static) {
    let path = HSTRING::from(pipe_path(kit));
    std::thread::spawn(move || loop {
        // SAFETY: valid name; inbound byte pipe for this machine only.
        let pipe = unsafe {
            CreateNamedPipeW(
                &path,
                PIPE_ACCESS_INBOUND,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                PIPE_UNLIMITED_INSTANCES,
                0,
                4096,
                0,
                None,
            )
        };
        if pipe.is_invalid() {
            return;
        }
        // SAFETY: `pipe` is a valid server handle; blocking connect.
        let connected = unsafe { ConnectNamedPipe(pipe, None) }.is_ok()
            || unsafe { GetLastError() } == ERROR_PIPE_CONNECTED;
        if connected {
            let mut buf = [0u8; 4096];
            let mut read = 0u32;
            // SAFETY: buffer and counter outlive the synchronous call.
            if unsafe { ReadFile(pipe, Some(&mut buf), Some(&mut read), None) }.is_ok() {
                on_message(String::from_utf8_lossy(&buf[..read as usize]).into_owned());
            }
            // SAFETY: valid connected pipe.
            let _ = unsafe { DisconnectNamedPipe(pipe) };
        }
        // SAFETY: handle from CreateNamedPipeW, closed once.
        let _ = unsafe { CloseHandle(pipe) };
    });
}

#[cfg(all(test, windows))]
#[path = "../../../test/rust/os/instance.rs"]
mod tests;
