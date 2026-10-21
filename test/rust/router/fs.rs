use super::*;
use crate::kit::tests::TempDir;

/// A private file of another account, as an NTFS disk brings it from
/// another PC (PORT-V6-01): a protected DACL with no entry this account
/// matches, so it can neither read nor rewrite it.
#[cfg(windows)]
fn foreign(path: &Path) {
    set_dacl(path, "D:P").unwrap();
    assert!(fs::read_to_string(path).is_err(), "not readable any more");
    assert!(fs::write(path, "x").is_err(), "not writable any more");
}

/// PORT-V6-01: a private file this account may not rewrite is replaced
/// by one of its own, owner-only again.
#[cfg(windows)]
#[test]
fn foreign_private_file_is_replaced() {
    let tmp = TempDir::new("fs-foreign");
    let dir = tmp.0.join("O’Brien {x}");
    let file = dir.join("my secret.txt");
    write_private(&file, "old").unwrap();
    foreign(&file);
    write_in_private(&file, "new").unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "new");

    let token = dir.join("token");
    replace_private(&token, "one").unwrap();
    foreign(&token);
    replace_private(&token, "two").unwrap();
    assert_eq!(fs::read_to_string(&token).unwrap(), "two");
}

/// A session secret is a new file each time, with the text and nothing
/// else, owner-only (mode 600 off Windows).
#[test]
fn replace_private_writes_a_new_file() {
    let tmp = TempDir::new("fs-replace");
    let file = tmp.0.join("O’Brien’s token");
    replace_private(&file, "first, longer").unwrap();
    replace_private(&file, "second").unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "second");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&file).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}

/// "Access denied" names the file and says what to do; another error
/// only names the file.
#[test]
fn access_errors_name_the_file() {
    let path = Path::new("D:/O’Brien/Atollo/data/app/secrets/gemini-api-key");
    let denied = access_error(path, io::Error::from(io::ErrorKind::PermissionDenied));
    let text = format!("{denied:#}");
    assert!(text.contains("gemini-api-key"), "{text}");
    assert!(text.contains("another"), "{text}");
    let other = format!("{:#}", access_error(path, io::Error::other("disk gone")));
    assert!(
        other.contains("gemini-api-key") && other.contains("disk gone"),
        "{other}"
    );
}
