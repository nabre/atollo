use super::*;

#[test]
fn lines() {
    for (v, l) in [
        ("v22.11.0", Some("22.11")),
        ("3.13.10", Some("3.13")),
        ("8.4.13 (Composer 2.8.12)", Some("8.4")),
        ("21.0.5+11 (Maven 3.9.11, Gradle 9.1.0)", Some("21.0")),
        ("1.25", Some("1.25")),
        ("0.15.1", Some("0.15")),
        ("go1.25.3", Some("1.25")),
        ("22", None),
        ("", None),
    ] {
        assert_eq!(line_of(v).as_deref(), l, "{v}");
    }
    assert!(valid_line("22.11") && !valid_line("22.11.0") && !valid_line("x"));
    assert!(!valid_line(r"..\x") && !valid_line("1.2;3"));
}

pub(super) fn install_fake(kit: &Kit, c: &Component, dir: &Path) {
    let rel = c.key.strip_prefix(c.folder.as_deref().unwrap()).unwrap();
    let key = dir.join(rel.trim_start_matches('/').replace('/', "\\"));
    fs::create_dir_all(key.parent().unwrap()).unwrap();
    fs::write(key, b"").unwrap();
    let _ = kit;
}
