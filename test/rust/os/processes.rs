use super::*;

#[test]
fn finds_itself() {
    let me = std::env::current_exe().unwrap();
    let dir = me.parent().unwrap();
    assert!(running_under(dir)
        .iter()
        .any(|p| p.pid == std::process::id()));
    assert!(running_under(Path::new(r"Z:\no\such\folder")).is_empty());
}
