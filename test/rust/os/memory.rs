use super::*;

#[test]
fn meminfo_total() {
    let text = "MemTotal:       16303740 kB\nMemFree:          301000 kB\n";
    assert_eq!(mem_total(text), Some(16_303_740 * 1024));
    assert_eq!(mem_total("MemFree: 1 kB\n"), None);
}

/// The PC the tests run on has some memory.
#[test]
fn this_pc_has_ram() {
    assert!(total_ram().is_some_and(|b| b > 256 << 20));
}
