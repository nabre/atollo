//! The socket of one app per kit (`os/unix/instance.rs`) on macOS: a file
//! in the user's own temporary folder, and the peer's user from
//! `getpeereid`.

use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use crate::os::unix::instance::SocketName;

/// The user's own temporary folder on macOS (`confstr`): not `TMPDIR`,
/// which the app points into the kit.
pub fn user_tmp() -> Option<PathBuf> {
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

pub fn bind(name: &SocketName) -> std::io::Result<UnixListener> {
    match name {
        SocketName::File(p) => {
            // A socket left by a dead app; the lock says it is not running.
            let _ = std::fs::remove_file(p);
            UnixListener::bind(p)
        }
        SocketName::Abstract(_) => Err(std::io::ErrorKind::Unsupported.into()),
    }
}

pub fn connect(name: &SocketName) -> std::io::Result<UnixStream> {
    match name {
        SocketName::File(p) => UnixStream::connect(p),
        SocketName::Abstract(_) => Err(std::io::ErrorKind::Unsupported.into()),
    }
}

/// The user at the other end of a local socket.
pub fn peer_uid(stream: &UnixStream) -> Option<u32> {
    use std::os::fd::AsRawFd;
    let (mut uid, mut gid) = (0, 0);
    // SAFETY: two out-parameters on the stack.
    let r = unsafe { libc::getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) };
    (r == 0).then_some(uid)
}
