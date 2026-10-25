use super::*;

#[test]
fn volume_root_ends_with_a_separator() {
    let windir = PathBuf::from(std::env::var("SystemRoot").unwrap_or(r"C:\Windows".into()));
    assert!(root_of(&windir).unwrap().to_string_lossy().ends_with('\\'));
}
