use super::*;
use crate::kit::tests::TempDir;
use crate::platform::files;

/// A private file of another account, as an NTFS disk brings it from
/// another PC (PORT-V6-01): a protected DACL with no entry this account
/// matches, so it can neither read nor rewrite it.
fn foreign(path: &Path) {
    set_dacl(path, "D:P").unwrap();
    assert!(fs::read_to_string(path).is_err(), "not readable any more");
    assert!(fs::write(path, "x").is_err(), "not writable any more");
}

/// PORT-V6-01: a private file this account may not rewrite is replaced
/// by one of its own, owner-only again.
#[test]
fn foreign_private_file_is_replaced() {
    let tmp = TempDir::new("fs-foreign");
    let dir = tmp.0.join("O’Brien {x}");
    let file = dir.join("my secret.txt");
    files::write_private(&file, "old").unwrap();
    foreign(&file);
    files::write_in_private(&file, "new").unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "new");

    let token = dir.join("token");
    files::replace_private(&token, "one").unwrap();
    foreign(&token);
    files::replace_private(&token, "two").unwrap();
    assert_eq!(fs::read_to_string(&token).unwrap(), "two");
}
