//! One running app per kit (`platform::instance`) on macOS and Linux: an
//! exclusive `flock` on `data/app/instance.lock` marks the running app and
//! a socket carries the message (D-105), never inside the kit, whose disk
//! (FAT, exFAT) may hold no socket and whose folder may be longer than a
//! socket's name allows:
//! - Linux: an abstract socket `\0Atollo-<uid>-<id>`, nothing on disk;
//! - macOS: `atollo-<id>.sock` in the user's own temporary folder
//!   (`/var/folders/…/T/`, short enough for the 104 bytes of a name);
//! - when that socket cannot be made: a TCP port on 127.0.0.1 with a token,
//!   both in `data/app/instance.json` (owner only where the disk allows).
//!
//! What differs between the two (the socket's address, the peer's user)
//! is in each system's `local_socket.rs`.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};

use crate::kit::Kit;
use crate::os::current::local_socket::{bind, connect, peer_uid, user_tmp};
use crate::platform::instance::Instance;

/// The locked `instance.lock`: the lock ends when the file closes.
pub struct MutexGuard(#[allow(dead_code)] std::fs::File);

pub fn acquire(kit: &Kit) -> Result<Instance> {
    let dir = kit.app_dir();
    std::fs::create_dir_all(&dir).with_context(|| dir.display().to_string())?;
    let path = dir.join("instance.lock");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .with_context(|| path.display().to_string())?;
    if let Err(e) = super::lock::try_lock(&file) {
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

/// The name of the socket of a kit (`id`, [`Kit::id`]) on macOS and Linux.
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
const MAX_SOCKET_NAME: usize = 103;

/// The socket of the kit `id` on `os`: abstract on Linux; on macOS in
/// `user_tmp`, the user's temporary folder, if the name fits.
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

fn socket_of(kit: &Kit) -> Option<SocketName> {
    socket_name(
        crate::platform::OS,
        &kit.id(),
        super::users::real(),
        user_tmp().as_deref(),
    )
}

/// `stream` when the other end is this user. Linux's abstract names have
/// no permissions: another user of the PC could connect to the app, or
/// take the name first and get the second start's message.
pub(crate) fn same_user(
    stream: std::os::unix::net::UnixStream,
) -> std::io::Result<std::os::unix::net::UnixStream> {
    if peer_uid(&stream) == Some(super::users::real()) {
        Ok(stream)
    } else {
        Err(std::io::ErrorKind::PermissionDenied.into())
    }
}

/// The address and token of the TCP fallback, in `data/app/instance.json`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TcpEndpoint {
    pub port: u16,
    pub token: String,
}

fn endpoint_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("instance.json")
}

/// Serves the TCP fallback on 127.0.0.1 and writes its port and a new
/// token to `file`: a message counts only after the right token and a
/// line break.
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

/// The two tokens are equal, compared in a time that does not tell how
/// much of them matched.
fn same_token(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0, |d, (x, y)| d | (x ^ y)) == 0
}

/// Sends `message` through the TCP fallback described in `file`.
pub(crate) fn notify_tcp(file: &Path, message: &str) -> std::io::Result<()> {
    let text = std::fs::read_to_string(file)?;
    let endpoint: TcpEndpoint = serde_json::from_str(&text).map_err(std::io::Error::other)?;
    let address = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, endpoint.port));
    let mut stream = std::net::TcpStream::connect_timeout(&address, Duration::from_secs(2))?;
    stream.write_all(format!("{}\n{message}", endpoint.token).as_bytes())
}

/// Sends `message` to the running app of the kit (retries while it
/// starts): through its socket, else through the TCP fallback.
pub fn notify_primary(kit: &Kit, message: &str) -> Result<()> {
    let socket = socket_of(kit);
    let file = endpoint_file(kit);
    let mut last = None;
    for _ in 0..20 {
        let sent = match socket.as_ref().map(|s| connect(s).and_then(same_user)) {
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
pub fn listen(kit: &Kit, on_message: impl Fn(String) + Send + 'static) {
    use std::io::Read;
    let file = endpoint_file(kit);
    let listener = match socket_of(kit).as_ref().map(bind) {
        Some(Ok(l)) => l,
        failed => {
            if let Some(Err(e)) = failed {
                crate::common::log::warn(&format!("instance socket: {e}; TCP on 127.0.0.1"));
            }
            let token = match crate::editor::random_token() {
                Ok(t) => t,
                Err(e) => {
                    crate::common::log::warn(&format!("instance token: {e:#}"));
                    return;
                }
            };
            if let Err(e) = listen_tcp(&file, token, on_message) {
                crate::common::log::warn(&format!("instance TCP: {e:#}"));
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
                crate::common::log::warn("instance socket: a connection of another user, ignored");
                continue;
            };
            // One thread serves every connection: a peer that never closes
            // (a stopped second start, any local tool) must not hold back
            // the focus requests after it, as on the TCP path (BUG-V7-21).
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let mut buf = Vec::new();
            if (&mut stream).take(4096).read_to_end(&mut buf).is_ok() {
                on_message(String::from_utf8_lossy(&buf).into_owned());
            }
        }
    });
}

#[cfg(test)]
#[path = "../../../../test/rust/os/unix/instance.rs"]
mod tests;
