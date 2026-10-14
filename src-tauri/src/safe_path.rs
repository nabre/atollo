//! Reading inside a project without ever following a link (DECISIONS.md
//! D-77). A project is code from anywhere (a clone, a zip): a symbolic link
//! or a junction in it pointing to `\\server\share` would make Windows send
//! the user's NTLM credentials to that server as soon as the app reads
//! through it, and the app reads projects on its own (the list, the
//! detection, the editor extensions every 15 s). So every path is resolved
//! as text first, must stay under its root, and each step from the root
//! down is looked at with `symlink_metadata`, which never follows a link.

use std::fs;
use std::path::{Component, Path, PathBuf};

/// `path` with `.` and `..` resolved by text only (nothing on disk is
/// touched); None when `..` would climb above its root.
pub fn lexical(path: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::ParentDir => {
                if !out.pop() {
                    return None;
                }
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    Some(out)
}

/// A symbolic link or a junction (reparse point), seen without following it.
pub fn is_reparse(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    meta.file_type().is_symlink() || meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

/// `path` itself is a link or a junction.
pub fn is_link(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| is_reparse(&m))
}

/// No link or junction at `root` or on the way from it down to `path`
/// (both already lexical). A step that does not exist ends the walk:
/// nothing behind it can be read.
pub fn no_links(root: &Path, path: &Path) -> bool {
    if is_link(root) {
        return false;
    }
    let Ok(rest) = path.strip_prefix(root) else {
        return false;
    };
    let mut cur = root.to_path_buf();
    for c in rest.components() {
        cur.push(c);
        match fs::symlink_metadata(&cur) {
            Ok(m) if is_reparse(&m) => return false,
            Ok(_) => {}
            Err(_) => return true,
        }
    }
    true
}

/// `path` under `root` (text) with no link on the way, ready to be read.
fn checked(root: &Path, path: &Path) -> Option<PathBuf> {
    let path = lexical(path)?;
    let root = lexical(root)?;
    if path.to_string_lossy().starts_with(r"\\")
        || !path.starts_with(&root)
        || !no_links(&root, &path)
    {
        return None;
    }
    Some(path)
}

pub fn read(root: &Path, path: &Path) -> Option<Vec<u8>> {
    fs::read(checked(root, path)?).ok()
}

pub fn read_text(root: &Path, path: &Path) -> Option<String> {
    read(root, path).map(|b| {
        String::from_utf8_lossy(&b)
            .trim_start_matches('\u{feff}')
            .to_owned()
    })
}

fn meta(root: &Path, path: &Path) -> Option<fs::Metadata> {
    fs::symlink_metadata(checked(root, path)?).ok()
}

pub fn exists(root: &Path, path: &Path) -> bool {
    meta(root, path).is_some()
}

pub fn is_file(root: &Path, path: &Path) -> bool {
    meta(root, path).is_some_and(|m| m.is_file())
}

pub fn is_dir(root: &Path, path: &Path) -> bool {
    meta(root, path).is_some_and(|m| m.is_dir())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::kit::tests::TempDir;

    /// A real junction (`mklink /J` needs no admin rights).
    pub(crate) fn junction(link: &Path, target: &Path) {
        let ok = std::process::Command::new("cmd")
            .args(["/d", "/c", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap()
            .success();
        assert!(ok, "mklink /J");
    }

    #[test]
    fn links_are_never_followed() {
        let tmp = TempDir::new("safe-path");
        let root = tmp.0.join("p");
        let outside = tmp.0.join("outside");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(&outside).unwrap();
        fs::write(root.join("src").join("a.txt"), "a").unwrap();
        fs::write(outside.join("secret.txt"), "s").unwrap();
        junction(&root.join("lnk"), &outside);
        assert_eq!(
            read_text(&root, &root.join("src").join("a.txt")).as_deref(),
            Some("a")
        );
        assert!(is_dir(&root, &root.join("src")));
        assert!(read(&root, &root.join("lnk").join("secret.txt")).is_none());
        assert!(!exists(&root, &root.join("lnk")));
        assert!(!is_dir(&root, &root.join("lnk")));
        assert!(read(&root, &root.join("..").join("outside").join("secret.txt")).is_none());
        assert!(read(&root, Path::new(r"\\server\share\x")).is_none());
        // A project folder that is itself a junction: nothing in it.
        let linked = tmp.0.join("linked");
        junction(&linked, &root);
        assert!(read(&linked, &linked.join("src").join("a.txt")).is_none());
        assert!(is_link(&linked) && !is_link(&root));
    }
}
