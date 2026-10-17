use super::*;
use crate::agents::sync::tests::kit;
use std::time::Duration;

#[test]
fn folders_both_ways_conflicts_and_deletions() {
    let (_t, _kit, p) = kit("tree");
    let (a, b) = (p.agents.join("skills"), p.claude.join("skills"));
    fs::create_dir_all(a.join("one")).unwrap();
    fs::write(a.join(r"one\SKILL.md"), "A1").unwrap();
    fs::create_dir_all(b.join("two")).unwrap();
    fs::write(b.join(r"two\SKILL.md"), "B1").unwrap();
    let mut r = Report::default();
    sync_tree(&p, &a, &b, "t", true, "skills", &mut r).unwrap();
    assert_eq!(fs::read_to_string(b.join(r"one\SKILL.md")).unwrap(), "A1");
    assert_eq!(fs::read_to_string(a.join(r"two\SKILL.md")).unwrap(), "B1");

    // Both change the same file: the newer wins, the other is kept.
    fs::write(a.join(r"one\SKILL.md"), "A2").unwrap();
    std::thread::sleep(Duration::from_millis(30));
    fs::write(b.join(r"one\SKILL.md"), "B2").unwrap();
    sync_tree(&p, &a, &b, "t", true, "skills", &mut r).unwrap();
    assert_eq!(fs::read_to_string(a.join(r"one\SKILL.md")).unwrap(), "B2");
    assert!(r.warnings.iter().any(|w| w.contains("both sides")));
    assert_eq!(fs::read_dir(p.state.join("conflicts")).unwrap().count(), 1);

    // Deleted on the replica: deleted in the master, empty folder too.
    fs::remove_dir_all(b.join("two")).unwrap();
    sync_tree(&p, &a, &b, "t", true, "skills", &mut r).unwrap();
    assert!(!a.join("two").exists());
}

#[cfg(windows)]
#[test]
fn junction_inside_is_never_written_through() {
    let (t, _kit, p) = kit("tree-confined");
    let (a, b) = (p.agents.join("x"), p.claude.join("x"));
    let outside = t.0.join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(a.join("sub")).unwrap();
    fs::write(a.join(r"sub\f.txt"), "hi").unwrap();
    fs::create_dir_all(&b).unwrap();
    junction::create(&outside, b.join("sub")).unwrap();
    let mut r = Report::default();
    sync_tree(&p, &a, &b, "c", true, "x", &mut r).unwrap();
    assert!(!outside.join("f.txt").exists(), "nothing written outside");
    assert!(r.warnings.iter().any(|w| w.contains("linked elsewhere")));
}
