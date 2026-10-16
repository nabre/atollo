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
