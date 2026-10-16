use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn written_and_restored() {
    let tmp = TempDir::new("kitfiles");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    write_all(&kit).unwrap();
    let path = kit.kit_files().join("loopback.js");
    assert!(fs::read_to_string(&path)
        .unwrap()
        .contains("atolloLoopback"));
    assert!(kit.kit_files().join("php-router.php").is_file());
    fs::write(&path, "changed").unwrap();
    write_all(&kit).unwrap();
    assert!(fs::read_to_string(&path)
        .unwrap()
        .contains("atolloLoopback"));
}
