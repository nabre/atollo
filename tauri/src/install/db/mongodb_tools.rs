//! The MongoDB Database Tools (mongodump, mongorestore) of `release.json`:
//! on Linux the build of this PC's distribution, read from `os-release`.

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::install::lang::version_key;
use crate::kit::{Arch, Os, Target};
use crate::net::archive::Kind;

/// The newest MongoDB Database Tools for Windows x64 from `release.json`
/// (version, https link, SHA-256).
#[cfg(test)]
fn mongodb_tools_zip(release: &Value) -> Result<(String, String, String)> {
    let (v, u, s, _) = mongodb_tools_for(Target::WIN32_X64, release)?;
    Ok((v, u, s))
}

/// The newest MongoDB Database Tools for `target` from `release.json`; on
/// Linux the build of the distribution of this PC, unknown here
/// (`mongodb_tools_linux_name` with no `os-release`).
#[cfg(test)]
pub(crate) fn mongodb_tools_for(
    target: Target,
    release: &Value,
) -> Result<(String, String, String, Kind)> {
    mongodb_tools_for_distro(target, release, None)
}

/// A Linux distribution as MongoDB names its builds: family and version
/// (`ubuntu` 24.04 → `["ubuntu", [24, 4]]`), from the text of
/// `/etc/os-release`. Derivatives follow their base: Linux Mint and Pop!_OS
/// by `UBUNTU_CODENAME`, Rocky, Alma, CentOS and Oracle Linux as RHEL,
/// Fedora as the newest RHEL, openSUSE Leap as SUSE. `None` for the others.
pub fn linux_distro(os_release: &str) -> Option<(&'static str, Vec<u64>)> {
    let field = |name: &str| {
        os_release.lines().find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == name).then(|| v.trim().trim_matches(['"', '\'']).to_owned())
        })
    };
    let id = field("ID").unwrap_or_default().to_ascii_lowercase();
    let like = field("ID_LIKE").unwrap_or_default().to_ascii_lowercase();
    let like: Vec<&str> = like.split_whitespace().collect();
    let version = field("VERSION_ID")
        .map(|v| version_key(&v))
        .filter(|v| !v.is_empty());
    let ubuntu_codename = |name: &str| -> Option<Vec<u64>> {
        Some(match name {
            "focal" => vec![20, 4],
            "jammy" => vec![22, 4],
            "noble" => vec![24, 4],
            "resolute" => vec![26, 4],
            _ => return None,
        })
    };
    match id.as_str() {
        "ubuntu" => return Some(("ubuntu", version?)),
        "debian" => return Some(("debian", version?)),
        "rhel" | "centos" | "rocky" | "almalinux" | "ol" => return Some(("rhel", version?)),
        "fedora" => return Some(("rhel", vec![u64::MAX])),
        "amzn" => return Some(("amazon", version?)),
        "sles" | "sled" | "opensuse-leap" => return Some(("suse", version?)),
        _ => {}
    }
    if like.contains(&"ubuntu") {
        let codename = field("UBUNTU_CODENAME").unwrap_or_default();
        return ubuntu_codename(&codename).map(|v| ("ubuntu", v));
    }
    if like
        .iter()
        .any(|l| ["rhel", "centos", "fedora"].contains(l))
    {
        return Some(("rhel", version?));
    }
    None
}

/// The family and version of a MongoDB build name (`ubuntu2404` →
/// `("ubuntu", [24, 4])`, `debian92` → `[9, 2]`, `rhel88` → `[8, 8]`,
/// `rhel10` → `[10]`, `amazon` → `[1]`).
fn mongodb_build_version(name: &str) -> Option<(&'static str, Vec<u64>)> {
    for family in ["ubuntu", "debian", "rhel", "amazon", "suse"] {
        let Some(digits) = name.strip_prefix(family) else {
            continue;
        };
        if !digits.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let n: u64 = digits.parse().unwrap_or(1);
        let v = match family {
            "ubuntu" if digits.len() == 4 => vec![n / 100, n % 100],
            // `debian92`, `rhel88`; `debian12` and `rhel10` are majors.
            "debian" | "rhel" if digits.len() == 2 && n > 13 => vec![n / 10, n % 10],
            _ => vec![n],
        };
        return Some((family, v));
    }
    None
}

/// The build of the MongoDB Database Tools (a name of `release.json`:
/// `ubuntu2404`, `debian12`, `rhel93`...) for the distribution described by
/// `os_release`, among `names` (those of the right CPU): the newest of its
/// family not newer than it. When the distribution is unknown, or not
/// readable (a kit prepared from another system), the oldest Ubuntu build
/// from 20.04 on: built on the oldest glibc, it runs on the newer
/// distributions too (the tools are Go programs that need only glibc and
/// Kerberos, verified with `ldd` on 100.19.1).
pub fn mongodb_tools_linux_name<'a>(
    os_release: Option<&str>,
    names: &[&'a str],
) -> Option<&'a str> {
    let builds: Vec<(&str, &'static str, Vec<u64>)> = names
        .iter()
        .filter_map(|n| mongodb_build_version(n).map(|(f, v)| (*n, f, v)))
        .collect();
    if let Some((family, version)) = os_release.and_then(linux_distro) {
        // RHEL-like systems and Debian keep one glibc per major
        // (`VERSION_ID="7"` of CentOS 7 takes `rhel70`, `8.2` takes the
        // newest `rhel8x`, Debian 9's `VERSION_ID="9"` takes `debian92`).
        let fits = |v: &Vec<u64>| {
            if family == "rhel" || family == "debian" {
                v.first() <= version.first()
            } else {
                *v <= version
            }
        };
        let best = builds
            .iter()
            .filter(|(_, f, v)| *f == family && fits(v))
            .max_by(|a, b| a.2.cmp(&b.2));
        if let Some((name, _, _)) = best {
            return Some(name);
        }
    }
    builds
        .iter()
        .filter(|(_, f, v)| *f == "ubuntu" && *v >= vec![20, 4])
        .min_by(|a, b| a.2.cmp(&b.2))
        .map(|(name, _, _)| *name)
}

/// The text of `/etc/os-release` (or `/usr/lib/os-release`) of this PC when
/// it runs the kit's Linux target; `None` elsewhere.
pub(super) fn host_os_release(target: Target) -> Option<String> {
    if target.os != Os::Linux {
        return None;
    }
    crate::platform::host::os_release()
}

/// The newest MongoDB Database Tools for `target` from `release.json`, on
/// Linux for the distribution `os_release` (`mongodb_tools_linux_name`).
pub(crate) fn mongodb_tools_for_distro(
    target: Target,
    release: &Value,
    os_release: Option<&str>,
) -> Result<(String, String, String, Kind)> {
    let v = release["versions"]
        .as_array()
        .and_then(|a| a.first())
        .ok_or_else(|| anyhow!("MongoDB tools: no versions"))?;
    let number = v["version"].as_str().unwrap_or_default().to_owned();
    let mut downloads = v["downloads"].as_array().into_iter().flatten();
    let (archive, kind) = match target.os {
        Os::Win32 => {
            let d = downloads
                .find(|d| d["name"] == "windows" && d["arch"] == "x86_64")
                .ok_or_else(|| anyhow!("MongoDB tools {number}: no Windows zip"))?;
            (&d["archive"], Kind::Zip)
        }
        Os::Darwin => {
            let arch = match target.arch {
                Arch::Arm64 => "arm64",
                Arch::X64 => "x86_64",
            };
            let d = downloads
                .find(|d| d["name"] == "macos" && d["arch"] == arch)
                .ok_or_else(|| anyhow!("MongoDB tools {number}: no macOS {arch} zip"))?;
            (&d["archive"], Kind::Zip)
        }
        Os::Linux => {
            let arch = match target.arch {
                Arch::Arm64 => "arm64",
                Arch::X64 => "x86_64",
            };
            // MongoDB says `arm64` for Ubuntu, `aarch64` for the others.
            let ours: Vec<&Value> = downloads
                .filter(|d| d["arch"] == arch || d["arch"] == target.gnu_arch())
                .collect();
            let names: Vec<&str> = ours.iter().filter_map(|d| d["name"].as_str()).collect();
            let name = mongodb_tools_linux_name(os_release, &names)
                .ok_or_else(|| anyhow!("MongoDB tools {number}: no Linux {arch} archive"))?;
            let d = ours
                .iter()
                .find(|d| d["name"] == name)
                .ok_or_else(|| anyhow!("MongoDB tools {number}: no Linux {arch} archive"))?;
            (&d["archive"], Kind::TarGz)
        }
    };
    let url = archive["url"]
        .as_str()
        .filter(|u| u.starts_with("https://"))
        .ok_or_else(|| anyhow!("MongoDB tools {number}: no https archive"))?;
    let sha = archive["sha256"]
        .as_str()
        .filter(|s| s.len() == 64)
        .ok_or_else(|| anyhow!("MongoDB tools {number}: no SHA-256"))?;
    Ok((number, url.to_owned(), sha.to_owned(), kind))
}

#[cfg(test)]
#[path = "../../../../test/rust/install/db/mongodb_tools.rs"]
mod tests;
