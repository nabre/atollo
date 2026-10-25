//! The proxy of GNOME's settings (`platform::proxy`), from `gsettings`.

use ureq::Proxy;

use crate::os::unix::proxy::{output_of, proxy_url, system_proxy};

/// The environment's proxy, else the system's.
pub fn from_system() -> Option<Proxy> {
    Proxy::try_from_env().or_else(|| system_proxy(read_system))
}

/// GNOME's settings, read with the user's own profile: the app points
/// `HOME` and `XDG_CONFIG_HOME` (where dconf keeps them) into the kit.
fn read_system() -> Option<(String, Vec<String>)> {
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

/// The items of a list printed by `gsettings get` (`['localhost', '::1']`,
/// `@as []` when empty): GNOME's `ignore-hosts`.
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

/// The proxy of GNOME's settings (`org.gnome.system.proxy`) from what
/// `gsettings get` printed: `mode`, then host and port for HTTPS and HTTP.
/// Only `'manual'`: an automatic configuration (PAC) is not read.
pub(crate) fn gnome_proxy(mode: &str, https: (&str, &str), http: (&str, &str)) -> Option<String> {
    let unquote = |s: &str| s.trim().trim_matches('\'').to_owned();
    if unquote(mode) != "manual" {
        return None;
    }
    [https, http]
        .iter()
        .find_map(|(host, port)| proxy_url(&unquote(host), Some(&unquote(port))))
}

#[cfg(test)]
#[path = "../../../../test/rust/os/linux/proxy.rs"]
mod tests;
