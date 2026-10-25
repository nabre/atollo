//! Random bytes (`platform::random`) of `/dev/urandom`.

use std::io::Read;

use anyhow::{Context, Result};

pub fn fill(bytes: &mut [u8]) -> Result<()> {
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(bytes))
        .context("/dev/urandom")
}
