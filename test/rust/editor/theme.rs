use super::*;
use crate::kit::tests::TempDir;
use serde_json::Value;

#[test]
fn embedded_extension_is_valid() {
    let package: Value = serde_json::from_str(FILES[0].1).unwrap();
    assert_eq!(
        format!(
            "{}.{}",
            package["publisher"].as_str().unwrap(),
            package["name"].as_str().unwrap()
        ),
        ID
    );
    assert!(version().starts_with("1.0."));
    let themes = package["contributes"]["themes"].as_array().unwrap();
    for (theme, label, kind) in [(&themes[0], DARK, "dark"), (&themes[1], LIGHT, "light")] {
        assert_eq!(theme["id"], label);
        let rel = theme["path"].as_str().unwrap().trim_start_matches("./");
        let (_, text) = FILES.iter().find(|(n, _)| *n == rel).unwrap();
        let colors: Value = serde_json::from_str(text).unwrap();
        assert_eq!(colors["type"], kind);
        assert!(colors["tokenColors"].as_array().unwrap().len() > 50);
        assert!(colors["colors"]["editor.background"].is_string());
    }
}

#[test]
fn vsix_holds_the_extension() {
    let tmp = TempDir::new("theme-vsix");
    let kit = Kit::at(tmp.0.clone(), crate::test_support::kit_arch());
    let path = tmp.0.join("t.vsix");
    write_vsix(&path).unwrap();
    let mut zip = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
    assert!(zip.by_name("extension/package.json").is_ok());
    assert!(zip.by_name("extension/themes/dark.json").is_ok());
    assert!(!is_current(&kit));
    let dir = crate::editor::ext::extensions_dir(&kit).join(format!("{ID}-{}", version()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("package.json"), FILES[0].1).unwrap();
    assert!(is_current(&kit));
}
