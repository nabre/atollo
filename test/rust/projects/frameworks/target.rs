use super::*;
use crate::kit::tests::TempDir;
use crate::test_support::prepared_kit;

#[test]
fn existing_folders() {
    let (_tmp, kit) = prepared_kit("frameworks-targets");
    let p = kit.projects();
    fs::create_dir_all(p.join("empty")).unwrap();
    fs::create_dir_all(p.join("files")).unwrap();
    fs::write(p.join("files").join("notes.txt"), "x").unwrap();
    fs::create_dir_all(p.join("repo").join(".git")).unwrap();
    let states: Vec<(String, Existing)> = targets(&kit)
        .into_iter()
        .map(|t| (t.name, t.state))
        .collect();
    assert_eq!(
        states,
        [
            ("empty".to_owned(), Existing::Empty),
            ("files".to_owned(), Existing::Files),
            ("repo".to_owned(), Existing::Git)
        ]
    );
    assert_eq!(existing(&kit, "new").unwrap(), Existing::New);
    assert_eq!(existing(&kit, "files").unwrap(), Existing::Files);
    assert!(existing(&kit, "repo").is_err());
    assert!(existing(&kit, "..").is_err());
}

#[test]
fn merging_keeps_what_is_there() {
    let tmp = TempDir::new("frameworks-merge");
    let (from, to) = (tmp.0.join("from"), tmp.0.join("to"));
    fs::create_dir_all(from.join("src")).unwrap();
    fs::create_dir_all(to.join("src")).unwrap();
    fs::write(from.join("package.json"), "new").unwrap();
    fs::write(from.join("README.md"), "new").unwrap();
    fs::write(from.join("src").join("main.ts"), "new").unwrap();
    fs::write(from.join("src").join("app.css"), "new").unwrap();
    fs::write(to.join("README.md"), "mine").unwrap();
    fs::write(to.join("src").join("app.css"), "mine").unwrap();
    let mut kept = Vec::new();
    merge_into(&from, &to, &mut kept).unwrap();
    kept.sort();
    // with the system's separator: `src\app.css` on Windows
    let nested = std::path::Path::new("src").join("app.css");
    assert_eq!(kept, ["README.md", nested.to_str().unwrap()]);
    assert_eq!(fs::read_to_string(to.join("README.md")).unwrap(), "mine");
    assert_eq!(fs::read_to_string(to.join("package.json")).unwrap(), "new");
    assert_eq!(
        fs::read_to_string(to.join("src").join("main.ts")).unwrap(),
        "new"
    );
    assert_eq!(
        fs::read_to_string(to.join("src").join("app.css")).unwrap(),
        "mine"
    );
}
