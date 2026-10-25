//! The socket of one app per kit (`os/unix/instance.rs`) on Linux: an
//! abstract name, nothing on disk, and the peer's user from `SO_PEERCRED`.

use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use crate::os::unix::instance::SocketName;

/// Linux needs no temporary folder: the socket is abstract.
pub fn user_tmp() -> Option<PathBuf> {
    None
}

pub fn bind(name: &SocketName) -> std::io::Result<UnixListener> {
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::SocketAddr;
    match name {
        SocketName::Abstract(n) => UnixListener::bind_addr(&SocketAddr::from_abstract_name(n)?),
        SocketName::File(p) => UnixListener::bind(p),
    }
}

pub fn connect(name: &SocketName) -> std::io::Result<UnixStream> {
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::SocketAddr;
    match name {
        SocketName::Abstract(n) => UnixStream::connect_addr(&SocketAddr::from_abstract_name(n)?),
        SocketName::File(p) => UnixStream::connect(p),
    }
}

/// The user at the other end of a local socket.
pub fn peer_uid(stream: &UnixStream) -> Option<u32> {
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
