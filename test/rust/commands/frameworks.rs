use super::merge_copy;
use crate::kit::tests::TempDir;
use std::fs;
use std::path::Path;

/// As `merge_into`, by copy: the project's files stay, the source too.
#[test]
fn merge_copy_keeps_what_the_project_has() {
    let tmp = TempDir::new("mergecopy");
    let (made, dir) = (tmp.0.join("made"), tmp.0.join("app"));
    fs::create_dir_all(made.join(r"src\lib")).unwrap();
    fs::write(made.join("package.json"), "new").unwrap();
    fs::write(made.join(r"src\main.js"), "new").unwrap();
    fs::write(made.join(r"src\lib\a.js"), "new").unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("package.json"), "mine").unwrap();
    let mut kept = Vec::new();
    merge_copy(&made, &dir, Path::new(""), &mut kept).unwrap();
    assert_eq!(kept, ["package.json"]);
    assert_eq!(
        fs::read_to_string(dir.join("package.json")).unwrap(),
        "mine"
    );
    assert_eq!(
        fs::read_to_string(dir.join(r"src\lib\a.js")).unwrap(),
        "new"
    );
    assert!(made.join(r"src\main.js").is_file());
}
