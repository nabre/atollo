//! The proxy of the system (`platform::proxy`) on Windows: the Internet
//! settings, read by ureq itself (its `win-system-proxy` feature).

use ureq::Proxy;

/// The environment's proxy, else the system's.
pub fn from_system() -> Option<Proxy> {
    Proxy::try_from_env()
}
