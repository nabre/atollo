//! The proxy of System Settings › Network (`platform::proxy`), from
//! `scutil --proxy`.

use ureq::Proxy;

use crate::os::unix::proxy::{output_of, proxy_url, system_proxy};

/// The environment's proxy, else the system's.
pub fn from_system() -> Option<Proxy> {
    Proxy::try_from_env().or_else(|| system_proxy(read_system))
}

fn read_system() -> Option<(String, Vec<String>)> {
    let output = output_of(std::process::Command::new("/usr/sbin/scutil").arg("--proxy"))?;
    Some((scutil_proxy(&output)?, scutil_exceptions(&output)))
}

/// The proxy in the output of macOS `scutil --proxy` (the settings of
/// System Settings › Network): HTTPS's if on, else HTTP's.
pub(crate) fn scutil_proxy(output: &str) -> Option<String> {
    let mut keys = std::collections::BTreeMap::new();
    let mut depth = 0usize;
    for line in output.lines() {
        let line = line.trim();
        // Only the keys of the top dictionary (not ExceptionsList's items).
        if line == "}" {
            depth = depth.saturating_sub(1);
            continue;
        }
        if let Some((key, value)) = line.split_once(" : ") {
            if depth == 1 {
                keys.insert(key.trim(), value.trim());
            }
        }
        if line.ends_with('{') {
            depth += 1;
        }
    }
    ["HTTPS", "HTTP"].iter().find_map(|kind| {
        let on = keys.get(format!("{kind}Enable").as_str()) == Some(&"1");
        let host = keys.get(format!("{kind}Proxy").as_str())?;
        on.then(|| proxy_url(host, keys.get(format!("{kind}Port").as_str()).copied()))
            .flatten()
    })
}

/// The hosts reached without the proxy in macOS `scutil --proxy`: the items
/// of the top dictionary's `ExceptionsList`.
pub(crate) fn scutil_exceptions(output: &str) -> Vec<String> {
    let mut hosts = Vec::new();
    let mut depth = 0usize;
    let mut inside = false;
    for line in output.lines() {
        let line = line.trim();
        if line == "}" {
            depth = depth.saturating_sub(1);
            inside &= depth >= 2;
            continue;
        }
        if inside && depth == 2 {
            if let Some((_, host)) = line.split_once(" : ") {
                hosts.push(host.trim().to_owned());
            }
        }
        if line.ends_with('{') {
            depth += 1;
            inside |= depth == 2 && line.starts_with("ExceptionsList ");
        }
    }
    hosts
}

#[cfg(test)]
#[path = "../../../../test/rust/os/macos/proxy.rs"]
mod tests;
