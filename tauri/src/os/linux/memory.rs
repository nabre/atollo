//! The PC's memory (`platform::memory`): `MemTotal` of `/proc/meminfo`.

/// The physical memory, in bytes (`MemTotal` of `/proc/meminfo`).
pub fn total_ram() -> Option<u64> {
    mem_total(&std::fs::read_to_string("/proc/meminfo").ok()?)
}

/// `MemTotal: <n> kB` of a `/proc/meminfo` text, in bytes.
pub(crate) fn mem_total(meminfo: &str) -> Option<u64> {
    meminfo.lines().find_map(|l| {
        let kb = l.strip_prefix("MemTotal:")?.trim().strip_suffix("kB")?;
        kb.trim().parse::<u64>().ok().map(|kb| kb * 1024)
    })
}

#[cfg(test)]
#[path = "../../../../test/rust/os/linux/memory.rs"]
mod tests;
