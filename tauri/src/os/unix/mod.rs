//! What macOS and Linux share (POSIX), re-exported by `os/macos/` and
//! `os/linux/` under the names of `crate::platform`. These modules may call
//! `crate::os::current` for what differs between the two.

pub mod files;
pub mod guard;
pub mod instance;
pub mod jobs;
pub mod lock;
pub mod sessions;
pub mod system;
pub mod temp;
pub mod users;
pub mod volume;
