//! What this Mac lacks for Docker Sandboxes (`platform::virt`): Apple
//! silicon (`hw.optional.arm64`, which an x64 app under Rosetta reads too)
//! and macOS 14 or later. The Virtualization framework is always there.

use std::process::{Command, Stdio};

use crate::platform::virt::Gap;

pub fn gaps() -> Vec<Gap> {
    gaps_of(apple_silicon(), version().as_deref())
}

/// The gaps of a Mac with or without Apple silicon on macOS `version`.
pub(crate) fn gaps_of(apple_silicon: bool, version: Option<&str>) -> Vec<Gap> {
    if !apple_silicon {
        return vec![Gap::Processor];
    }
    match version {
        Some(v) if major(v) < 14 => vec![Gap::OldMacos(v.to_owned())],
        _ => Vec::new(),
    }
}

/// The first number of `14.6.1`; 0 when there is none.
fn major(version: &str) -> u32 {
    version
        .split('.')
        .next()
        .and_then(|m| m.trim().parse().ok())
        .unwrap_or(0)
}

/// The output of a program of the system, trimmed.
fn output(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn apple_silicon() -> bool {
    output("/usr/sbin/sysctl", &["-n", "hw.optional.arm64"]).as_deref() == Some("1")
}

fn version() -> Option<String> {
    output("/usr/bin/sw_vers", &["-productVersion"]).filter(|v| !v.is_empty())
}

#[cfg(test)]
#[path = "../../../../test/rust/os/macos/virt.rs"]
mod tests;
