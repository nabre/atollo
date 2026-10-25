//! The proxy of the system (`platform::proxy`) on macOS and Linux: what
//! the two share; each reads its settings in its own `proxy.rs`.

use ureq::Proxy;

use crate::platform::proxy::with_exceptions;

/// `http://host:port` of a proxy (a proxy for HTTPS is reached over HTTP
/// too); no port when the system gives none.
pub(crate) fn proxy_url(host: &str, port: Option<&str>) -> Option<String> {
    let host = host.trim();
    if host.is_empty() {
        return None;
    }
    let host = host
        .strip_prefix("http://")
        .or_else(|| host.strip_prefix("https://"))
        .unwrap_or(host)
        .trim_end_matches('/');
    match port.map(str::trim).filter(|p| !p.is_empty() && *p != "0") {
        Some(p) => Some(format!("http://{host}:{p}")),
        None => Some(format!("http://{host}")),
    }
}

/// The system's proxy (`read`, D-105), with its exceptions: used only when
/// neither the settings nor `HTTPS_PROXY`/`HTTP_PROXY` give one. Read once per run (every `Net` would start `scutil` or
/// `gsettings` again): a proxy changed in the system counts from the next
/// start of the app.
pub fn system_proxy(read: fn() -> Option<(String, Vec<String>)>) -> Option<Proxy> {
    type Found = Option<(String, Vec<String>)>;
    static URL: std::sync::OnceLock<Found> = std::sync::OnceLock::new();
    let (url, hosts) = URL.get_or_init(read).as_ref()?;
    match Proxy::new(url) {
        Ok(p) => Some(with_exceptions(p, hosts)),
        Err(e) => {
            crate::common::log::warn(&format!("system proxy {url}: {e}"));
            None
        }
    }
}

/// The text a program printed, `None` when it is missing or fails.
pub(crate) fn output_of(cmd: &mut std::process::Command) -> Option<String> {
    let out = cmd
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}
