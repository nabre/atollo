//! What a program started by the app gets on macOS (`os/unix/jobs.rs`).

use std::process::Command;

/// macOS has no signal for the death of the parent: the guard alone ends
/// the programs of an app that was killed.
pub fn end_with_app(_cmd: &mut Command) {}
