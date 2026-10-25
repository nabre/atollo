//! The code written for one operating system, and the only part of the
//! engine that looks at which system it is built for (`#[cfg(windows)]`,
//! `#[cfg(unix)]`, `#[cfg(target_os = …)]`, `cfg!(…)`, the `windows` and
//! `libc` crates):
//!
//! - `windows/`: Win32, WebView2, ConPTY, the Credential Manager;
//! - `unix/`: what macOS and Linux share (POSIX): process groups and the
//!   guard, `flock`, modes and owners, Unix sockets;
//! - `macos/` and `linux/`: what only one of the two has (libproc and
//!   `/proc`, `statfs` and `mountinfo`, the Keychain and the Secret
//!   Service...).
//!
//! Each system offers the same modules under the same names, the ones
//! `crate::platform` needs: `macos/` and `linux/` re-export those of
//! `unix/` they share. [`current`] is the system the app is built for. The
//! rest of the engine never reaches in here: it calls `crate::platform`,
//! the abstract layer, whose functions every system implements with the
//! same signature (the build of each system checks it, and
//! `test/rust/os/rule.rs` keeps every check of the system inside `os/`).

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(unix)]
pub mod unix;
#[cfg(windows)]
pub mod windows;

/// The system the app is built for.
#[cfg(target_os = "linux")]
pub use self::linux as current;
/// The system the app is built for.
#[cfg(target_os = "macos")]
pub use self::macos as current;
/// The system the app is built for.
#[cfg(windows)]
pub use self::windows as current;

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
compile_error!("Atollo is built for Windows, macOS and Linux only");

#[cfg(test)]
#[path = "../../../test/rust/os/rule.rs"]
mod tests;
