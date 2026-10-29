//! The names of Atollo's own releases (`app-v1.2.3` tags, assets named
//! `Atollo-<version>-<target>.<ext>`): read by the update (`setup::self_update`)
//! and by the sessions' check of the running app (`platform::sessions`).

/// The prefix of a release's tag on GitHub.
pub const TAG_PREFIX: &str = "app-v";

/// `1.2.3` from `app-v1.2.3` (or `1.2.3`), as numbers for comparing.
pub fn parse_version(text: &str) -> Option<(u64, u64, u64)> {
    let v = text.strip_prefix(TAG_PREFIX).unwrap_or(text);
    let mut parts = v.split('.');
    let n = |p: Option<&str>| p.and_then(|s| s.parse().ok());
    let out = (n(parts.next())?, n(parts.next())?, n(parts.next())?);
    parts.next().is_none().then_some(out)
}

/// What follows the version in a release name (`win32-x64.exe` of
/// `Atollo-0.3.0-win32-x64.exe`); `None` for any other name.
pub fn release_rest(name: &str) -> Option<&str> {
    let (version, rest) = name.strip_prefix("Atollo-")?.split_once('-')?;
    let numbers =
        version.starts_with(|c: char| c.is_ascii_digit()) && parse_version(version).is_some();
    (numbers && !rest.is_empty()).then_some(rest)
}

/// A release name, as the app keeps it on disk (`Atollo-0.3.0-win32-x64.exe`).
pub fn is_release_name(name: &str) -> bool {
    release_rest(name).is_some()
}
