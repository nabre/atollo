//! One running app per kit (DECISIONS.md D-08).
//!
//! A named mutex `Local\Atollo-App-<kit id>` marks the running app; a second
//! start of the same kit sends a message through the pipe of the same name
//! (for now only "focus") and exits. Two different kits (two USB sticks) have
//! different ids and run side by side. The pipe rejects remote clients and
//! keeps the default security of the user's session.

use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use windows::core::HSTRING;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, ERROR_PIPE_CONNECTED, HANDLE,
};
use windows::Win32::Storage::FileSystem::{ReadFile, PIPE_ACCESS_INBOUND};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_BYTE,
    PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows::Win32::System::Threading::CreateMutexW;

use crate::kit::Kit;

pub enum Instance {
    /// This process is the app of the kit; keep the guard alive.
    Primary(MutexGuard),
    /// Another process already is.
    Secondary,
}

pub struct MutexGuard(HANDLE);

// SAFETY: a mutex handle is a kernel handle usable from any thread.
unsafe impl Send for MutexGuard {}
unsafe impl Sync for MutexGuard {}

impl Drop for MutexGuard {
    fn drop(&mut self) {
        // SAFETY: handle from CreateMutexW, closed once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

fn name(kit: &Kit) -> String {
    format!("Atollo-App-{}", kit.id())
}

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

fn pipe_path(kit: &Kit) -> String {
    format!(r"\\.\pipe\{}", name(kit))
}

/// Sends `message` to the running app of the kit (retries while it starts).
pub fn notify_primary(kit: &Kit, message: &str) -> Result<()> {
    let path = pipe_path(kit);
    let mut last = None;
    for _ in 0..20 {
        match OpenOptions::new().write(true).open(&path) {
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

/// Serves the pipe on a background thread; `on_message` gets each message.
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

#[cfg(test)]
#[path = "../../../test/rust/os/instance.rs"]
mod tests;
