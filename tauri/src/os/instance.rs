//! One running app per kit (DECISIONS.md D-08).
//!
//! A named mutex `Local\Atollo-App-<kit id>` marks the running app; a second
//! start of the same kit sends a message through the pipe of the same name
//! (for now only "focus") and exits. Two different kits (two USB sticks) have
//! different ids and run side by side. The pipe rejects remote clients and
//! keeps the default security of the user's session.
//!
//! macOS and Linux: an exclusive `flock` on `data/app/instance.lock` marks
//! the running app and a socket carries the message (WP-1.6), never inside
//! the kit, whose disk (FAT, exFAT) may hold no socket and whose folder may
//! be longer than a socket's name allows:
//! - Linux: an abstract socket `\0Atollo-<uid>-<id>`, nothing on disk;
//! - macOS: `atollo-<id>.sock` in the user's own temporary folder
//!   (`/var/folders/…/T/`, short enough for the 104 bytes of a name);
//! - when that socket cannot be made: a TCP port on 127.0.0.1 with a token,
//!   both in `data/app/instance.json` (owner only where the disk allows).

#[cfg(windows)]
use std::fs::OpenOptions;
use std::io::Write;
#[cfg(any(unix, test))]
use std::path::{Path, PathBuf};
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
    // The lock is ours: a TCP fallback left by an earlier start describes
    // no running app, and a second start that read it while this one is
    // still starting would send its message to a port that is not the
    // app's and report success.
    let _ = std::fs::remove_file(endpoint_file(kit));
    Ok(Instance::Primary(MutexGuard(file)))
}

#[cfg(windows)]
fn pipe_path(kit: &Kit) -> String {
    format!(r"\\.\pipe\{}", name(kit))
}

/// The name of the socket of a kit (`id`, [`Kit::id`]) on macOS and Linux.
#[cfg(any(unix, test))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SocketName {
    /// Linux's abstract namespace (no file): the user's id in the name keeps
    /// two users of the same PC apart, and both ends check the other's user
    /// (abstract names have no permissions).
    Abstract(String),
    /// A socket file (macOS).
    File(PathBuf),
}

/// The longest socket file name macOS takes (`sun_path` is 104 bytes,
/// with the final NUL); Linux takes 107, abstract names included.
#[cfg(any(unix, test))]
const MAX_SOCKET_NAME: usize = 103;

/// The socket of the kit `id` on `os`: abstract on Linux; on macOS in
/// `user_tmp`, the user's temporary folder, if the name fits.
#[cfg(any(unix, test))]
pub(crate) fn socket_name(
    os: crate::kit::Os,
    id: &str,
    uid: u32,
    user_tmp: Option<&Path>,
) -> Option<SocketName> {
    if os.is_linux() {
        return Some(SocketName::Abstract(format!("Atollo-{uid}-{id}")));
    }
    let path = user_tmp?.join(format!("atollo-{id}.sock"));
    (path.as_os_str().len() <= MAX_SOCKET_NAME).then_some(SocketName::File(path))
}

/// The user's own temporary folder on macOS (`confstr`): not `TMPDIR`,
/// which the app points into the kit.
#[cfg(target_os = "macos")]
fn user_tmp() -> Option<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let mut buf = [0u8; 1024];
    // SAFETY: the buffer outlives the call, which writes at most its length.
    let len = unsafe {
        libc::confstr(
            libc::_CS_DARWIN_USER_TEMP_DIR,
            buf.as_mut_ptr().cast(),
            buf.len(),
        )
    };
    let text = buf.get(..len.checked_sub(1)?)?;
    (!text.is_empty()).then(|| PathBuf::from(std::ffi::OsStr::from_bytes(text)))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn user_tmp() -> Option<PathBuf> {
    None
}

#[cfg(unix)]
fn socket_of(kit: &Kit) -> Option<SocketName> {
    // SAFETY: a plain query of this process.
    let uid = unsafe { libc::getuid() };
    socket_name(
        crate::kit::Os::current(),
        &kit.id(),
        uid,
        user_tmp().as_deref(),
    )
}

#[cfg(target_os = "linux")]
fn bind(name: &SocketName) -> std::io::Result<std::os::unix::net::UnixListener> {
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::{SocketAddr, UnixListener};
    match name {
        SocketName::Abstract(n) => UnixListener::bind_addr(&SocketAddr::from_abstract_name(n)?),
        SocketName::File(p) => UnixListener::bind(p),
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
fn bind(name: &SocketName) -> std::io::Result<std::os::unix::net::UnixListener> {
    match name {
        SocketName::File(p) => {
            // A socket left by a dead app; the lock says it is not running.
            let _ = std::fs::remove_file(p);
            std::os::unix::net::UnixListener::bind(p)
        }
        SocketName::Abstract(_) => Err(std::io::ErrorKind::Unsupported.into()),
    }
}

#[cfg(target_os = "linux")]
fn connect(name: &SocketName) -> std::io::Result<std::os::unix::net::UnixStream> {
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::{SocketAddr, UnixStream};
    match name {
        SocketName::Abstract(n) => UnixStream::connect_addr(&SocketAddr::from_abstract_name(n)?),
        SocketName::File(p) => UnixStream::connect(p),
    }
    .and_then(same_user)
}

#[cfg(all(unix, not(target_os = "linux")))]
fn connect(name: &SocketName) -> std::io::Result<std::os::unix::net::UnixStream> {
    match name {
        SocketName::File(p) => std::os::unix::net::UnixStream::connect(p),
        SocketName::Abstract(_) => Err(std::io::ErrorKind::Unsupported.into()),
    }
    .and_then(same_user)
}

/// The user at the other end of a local socket.
#[cfg(target_os = "linux")]
fn peer_uid(stream: &std::os::unix::net::UnixStream) -> Option<u32> {
    use std::os::fd::AsRawFd;
    // SAFETY: a zeroed ucred is valid; getsockopt fills at most `len` bytes.
    unsafe {
        let mut cred: libc::ucred = std::mem::zeroed();
        let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        let r = libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            std::ptr::from_mut(&mut cred).cast(),
            &mut len,
        );
        (r == 0).then_some(cred.uid)
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
fn peer_uid(stream: &std::os::unix::net::UnixStream) -> Option<u32> {
    use std::os::fd::AsRawFd;
    let (mut uid, mut gid) = (0, 0);
    // SAFETY: two out-parameters on the stack.
    let r = unsafe { libc::getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) };
    (r == 0).then_some(uid)
}

/// `stream` when the other end is this user. Linux's abstract names have
/// no permissions: another user of the PC could connect to the app, or
/// take the name first and get the second start's message.
#[cfg(unix)]
fn same_user(
    stream: std::os::unix::net::UnixStream,
) -> std::io::Result<std::os::unix::net::UnixStream> {
    // SAFETY: getuid cannot fail.
    if peer_uid(&stream) == Some(unsafe { libc::getuid() }) {
        Ok(stream)
    } else {
        Err(std::io::ErrorKind::PermissionDenied.into())
    }
}

/// The address and token of the TCP fallback, in `data/app/instance.json`.
#[cfg(any(unix, test))]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TcpEndpoint {
    pub port: u16,
    pub token: String,
}

#[cfg(unix)]
fn endpoint_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("instance.json")
}

/// Serves the TCP fallback on 127.0.0.1 and writes its port and a new
/// token to `file`: a message counts only after the right token and a
/// line break.
#[cfg(any(unix, test))]
pub(crate) fn listen_tcp(
    file: &Path,
    token: String,
    on_message: impl Fn(String) + Send + 'static,
) -> Result<()> {
    use std::io::Read;
    let listener =
        std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).context("TCP 127.0.0.1")?;
    let endpoint = TcpEndpoint {
        port: listener.local_addr()?.port(),
        token,
    };
    write_owner_only(file, &serde_json::to_string(&endpoint)?)
        .with_context(|| file.display().to_string())?;
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let mut buf = Vec::new();
            if (&mut stream).take(8192).read_to_end(&mut buf).is_err() {
                continue;
            }
            let text = String::from_utf8_lossy(&buf);
            match text.split_once('\n') {
                Some((token, message)) if same_token(token, &endpoint.token) => {
                    on_message(message.to_owned())
                }
                _ => {}
            }
        }
    });
    Ok(())
}

/// Writes `text` to `path` through a file created owner-only (0600) and
/// then renamed: the token is never readable by another user, not even
/// for a moment (FAT and exFAT keep no modes: there nothing can be done).
#[cfg(unix)]
fn write_owner_only(path: &Path, text: &str) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".tmp-{}", std::process::id()));
    let tmp = PathBuf::from(name);
    let _ = std::fs::remove_file(&tmp);
    let written = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)
        .and_then(|mut f| f.write_all(text.as_bytes()))
        .and_then(|()| std::fs::rename(&tmp, path));
    if written.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    written
}

/// Windows runs the TCP fallback only in tests.
#[cfg(all(test, not(unix)))]
fn write_owner_only(path: &Path, text: &str) -> Result<()> {
    crate::router::fs::write_in_private(path, text)
}

/// The two tokens are equal, compared in a time that does not tell how
/// much of them matched.
#[cfg(any(unix, test))]
fn same_token(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0, |d, (x, y)| d | (x ^ y)) == 0
}

/// Sends `message` through the TCP fallback described in `file`.
#[cfg(any(unix, test))]
pub(crate) fn notify_tcp(file: &Path, message: &str) -> std::io::Result<()> {
    let text = std::fs::read_to_string(file)?;
    let endpoint: TcpEndpoint = serde_json::from_str(&text).map_err(std::io::Error::other)?;
    let address = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, endpoint.port));
    let mut stream = std::net::TcpStream::connect_timeout(&address, Duration::from_secs(2))?;
    stream.write_all(format!("{}\n{message}", endpoint.token).as_bytes())
}

/// Sends `message` to the running app of the kit (retries while it starts).
#[cfg(windows)]
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

/// Sends `message` to the running app of the kit (retries while it
/// starts): through its socket, else through the TCP fallback.
#[cfg(unix)]
pub fn notify_primary(kit: &Kit, message: &str) -> Result<()> {
    let socket = socket_of(kit);
    let file = endpoint_file(kit);
    let mut last = None;
    for _ in 0..20 {
        let sent = match socket.as_ref().map(connect) {
            Some(Ok(mut stream)) => stream.write_all(message.as_bytes()),
            Some(Err(e)) if file.is_file() => notify_tcp(&file, message).map_err(|_| e),
            Some(Err(e)) => Err(e),
            None => notify_tcp(&file, message),
        };
        match sent {
            Ok(()) => return Ok(()),
            Err(e) => last = Some(e),
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    match last {
        Some(e) => Err(e).with_context(|| format!("instance socket {socket:?}")),
        None => bail!("instance socket {socket:?}"),
    }
}

/// Serves the socket on a background thread; `on_message` gets each message.
/// Called by the primary only. Without a socket, the TCP fallback.
#[cfg(unix)]
pub fn listen(kit: &Kit, on_message: impl Fn(String) + Send + 'static) {
    use std::io::Read;
    let file = endpoint_file(kit);
    let listener = match socket_of(kit).as_ref().map(bind) {
        Some(Ok(l)) => l,
        failed => {
            if let Some(Err(e)) = failed {
                crate::log::warn(&format!("instance socket: {e}; TCP on 127.0.0.1"));
            }
            let token = match crate::editor::random_token() {
                Ok(t) => t,
                Err(e) => {
                    crate::log::warn(&format!("instance token: {e:#}"));
                    return;
                }
            };
            if let Err(e) = listen_tcp(&file, token, on_message) {
                crate::log::warn(&format!("instance TCP: {e:#}"));
            }
            return;
        }
    };
    // A fallback left by an earlier start would send a second start to a
    // port that is no longer the app's.
    let _ = std::fs::remove_file(&file);
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let Ok(mut stream) = same_user(stream) else {
                crate::log::warn("instance socket: a connection of another user, ignored");
                continue;
            };
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

#[cfg(test)]
#[path = "../../../test/rust/os/instance.rs"]
mod tests;
