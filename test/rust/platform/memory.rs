use super::*;

/// The PC the tests run on has some memory.
#[test]
fn this_pc_has_ram() {
    assert!(total_ram().is_some_and(|b| b > 256 << 20));
}
