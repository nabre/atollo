//! The proxy of the downloads when the settings give none: the
//! environment's (`HTTPS_PROXY`, `HTTP_PROXY`), else the system's (D-105):
//! Windows' Internet settings through ureq, System Settings › Network on
//! macOS (`scutil --proxy`), GNOME's settings on Linux (`gsettings`), each
//! read its way (`os/*/proxy.rs`); and the hosts reached without it.

use ureq::Proxy;

/// The proxy of the environment or of the system, with the system's
/// exceptions.
pub fn from_system() -> Option<Proxy> {
    crate::os::current::proxy::from_system()
}

/// The hosts of «Senza proxy per» (`localhost,127.0.0.1,.example.com`),
/// separated by commas, semicolons or spaces as in `NO_PROXY`.
pub(crate) fn no_proxy_hosts(list: Option<&str>) -> Vec<String> {
    list.unwrap_or_default()
        .split([',', ';', ' '])
        .map(str::trim)
        .filter(|h| !h.is_empty())
        .map(str::to_owned)
        .collect()
}

/// `proxy` with the exceptions (the system's `*.local`, `localhost`, or
/// those of the Settings): the hosts reached directly. A rebuilt proxy that
/// fails keeps `proxy`.
pub(crate) fn with_exceptions(proxy: Proxy, hosts: &[String]) -> Proxy {
    if hosts.is_empty() {
        return proxy;
    }
    let mut builder = Proxy::builder(proxy.protocol())
        .host(proxy.host())
        .port(proxy.port());
    if let Some(user) = proxy.username() {
        builder = builder.username(user);
    }
    if let Some(password) = proxy.password() {
        builder = builder.password(password);
    }
    for host in hosts {
        builder = builder.no_proxy(host);
    }
    builder.build().unwrap_or(proxy)
}

#[cfg(test)]
#[path = "../../../test/rust/platform/proxy.rs"]
mod tests;
