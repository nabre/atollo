//! What macOS and Linux share (POSIX), re-exported by `os/macos/` and
//! `os/linux/` under the names of `crate::platform`. These modules may call
//! `crate::os::current` for what differs between the two.

pub mod console;
pub mod desktop;
pub mod files;
pub mod guard;
pub mod host;
pub mod instance;
pub mod jobs;
pub mod links;
pub mod local_time;
pub mod lock;
pub mod processes;
pub mod proxy;
pub mod pty;
pub mod random;
pub mod sessions;
pub mod signature;
pub mod sockets;
pub mod temp;
pub mod users;
pub mod volume;
