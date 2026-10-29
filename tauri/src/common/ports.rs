//! Port probes: something answers on a port, a port is free on both
//! loopbacks. Who keeps which port is [`crate::ports`] (D-189).

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};
use std::time::Duration;

/// Something answers on the port (IPv4 or IPv6 loopback).
pub fn responds(port: u16) -> bool {
    let t = Duration::from_millis(400);
    TcpStream::connect_timeout(&SocketAddr::from((Ipv4Addr::LOCALHOST, port)), t).is_ok()
        || TcpStream::connect_timeout(&SocketAddr::from((Ipv6Addr::LOCALHOST, port)), t).is_ok()
}

fn can_listen(addr: SocketAddr) -> bool {
    match TcpListener::bind(addr) {
        Ok(_) => true,
        // No IPv6 on this PC: not a reason to skip the port.
        Err(e) => !matches!(
            e.kind(),
            std::io::ErrorKind::AddrInUse | std::io::ErrorKind::PermissionDenied
        ),
    }
}

/// Nothing answers on the port and it can be bound on both loopbacks: the
/// probe of the actions, the database servers, the local models and the
/// pools. `editor::free_ports` checks IPv4 only, for its own range.
pub fn is_port_free(port: u16) -> bool {
    !responds(port)
        && can_listen(SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
        && can_listen(SocketAddr::from((Ipv6Addr::LOCALHOST, port)))
}
