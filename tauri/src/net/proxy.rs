//! The proxy: the system's on macOS and Linux (D-105), and the hosts reached without it.

use ureq::Proxy;

/// `http://host:port` of a proxy (a proxy for HTTPS is reached over HTTP
/// too); no port when the system gives none.
#[cfg(any(unix, test))]
fn proxy_url(host: &str, port: Option<&str>) -> Option<String> {
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

/// The proxy in the output of macOS `scutil --proxy` (the settings of
/// System Settings › Network): HTTPS's if on, else HTTP's.
#[cfg(any(target_os = "macos", test))]
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
#[cfg(any(target_os = "macos", test))]
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

/// The items of a list printed by `gsettings get` (`['localhost', '::1']`,
/// `@as []` when empty): GNOME's `ignore-hosts`.
#[cfg(any(target_os = "linux", test))]
pub(crate) fn gsettings_list(text: &str) -> Vec<String> {
    let text = text.trim();
    let text = text.strip_prefix("@as").unwrap_or(text).trim();
    let inner = text
        .strip_prefix('[')
        .and_then(|t| t.strip_suffix(']'))
        .unwrap_or("");
    inner
        .split(',')
        .map(|s| s.trim().trim_matches('\'').trim())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
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

/// The proxy of GNOME's settings (`org.gnome.system.proxy`) from what
/// `gsettings get` printed: `mode`, then host and port for HTTPS and HTTP.
/// Only `'manual'`: an automatic configuration (PAC) is not read.
#[cfg(any(target_os = "linux", test))]
pub(crate) fn gnome_proxy(mode: &str, https: (&str, &str), http: (&str, &str)) -> Option<String> {
    let unquote = |s: &str| s.trim().trim_matches('\'').to_owned();
    if unquote(mode) != "manual" {
        return None;
    }
    [https, http]
        .iter()
        .find_map(|(host, port)| proxy_url(&unquote(host), Some(&unquote(port))))
}

/// The system's proxy on macOS and Linux (D-105), with its exceptions:
/// used only when neither the settings nor `HTTPS_PROXY`/`HTTP_PROXY` give
/// one. Read once per run (every `Net` would start `scutil` or
/// `gsettings` again): a proxy changed in the system counts from the next
/// start of the app.
#[cfg(unix)]
pub(super) fn system_proxy() -> Option<Proxy> {
    type Found = Option<(String, Vec<String>)>;
    static URL: std::sync::OnceLock<Found> = std::sync::OnceLock::new();
    let (url, hosts) = URL.get_or_init(read_system_proxy).as_ref()?;
    match Proxy::new(url) {
        Ok(p) => Some(with_exceptions(p, hosts)),
        Err(e) => {
            crate::common::log::warn(&format!("system proxy {url}: {e}"));
            None
        }
    }
}

/// The text a program printed, `None` when it is missing or fails.
#[cfg(unix)]
fn output_of(cmd: &mut std::process::Command) -> Option<String> {
    let out = cmd
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(target_os = "macos")]
fn read_system_proxy() -> Option<(String, Vec<String>)> {
    let output = output_of(std::process::Command::new("/usr/sbin/scutil").arg("--proxy"))?;
    Some((scutil_proxy(&output)?, scutil_exceptions(&output)))
}

/// GNOME's settings, read with the user's own profile: the app points
/// `HOME` and `XDG_CONFIG_HOME` (where dconf keeps them) into the kit.
#[cfg(target_os = "linux")]
fn read_system_proxy() -> Option<(String, Vec<String>)> {
    let profile = std::env::var_os("CP_USER_PROFILE").or_else(|| std::env::var_os("HOME"))?;
    let get = |schema: &str, key: &str| {
        let mut cmd = std::process::Command::new("gsettings");
        cmd.args(["get", schema, key])
            .env("HOME", &profile)
            .env_remove("XDG_CONFIG_HOME");
        output_of(&mut cmd).unwrap_or_default()
    };
    let mode = get("org.gnome.system.proxy", "mode");
    if mode.trim().trim_matches('\'') != "manual" {
        return None;
    }
    let pair = |schema| (get(schema, "host"), get(schema, "port"));
    let (https, http) = (
        pair("org.gnome.system.proxy.https"),
        pair("org.gnome.system.proxy.http"),
    );
    let url = gnome_proxy(&mode, (&https.0, &https.1), (&http.0, &http.1))?;
    Some((
        url,
        gsettings_list(&get("org.gnome.system.proxy", "ignore-hosts")),
    ))
}

#[cfg(test)]
#[path = "../../../test/rust/net/proxy.rs"]
mod tests;
