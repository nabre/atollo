use super::*;
use crate::agents::sync::tests::kit;
use std::time::Duration;

#[test]
fn folders_both_ways_conflicts_and_deletions() {
    let (_t, _kit, p) = kit("tree");
    let (a, b) = (p.agents.join("skills"), p.claude.join("skills"));
    fs::create_dir_all(a.join("one")).unwrap();
    fs::write(a.join("one").join("SKILL.md"), "A1").unwrap();
    fs::create_dir_all(b.join("two")).unwrap();
    fs::write(b.join("two").join("SKILL.md"), "B1").unwrap();
    let mut r = Report::default();
    sync_tree(&p, &a, &b, "t", true, "skills", &mut r).unwrap();
    assert_eq!(
        fs::read_to_string(b.join("one").join("SKILL.md")).unwrap(),
        "A1"
    );
    assert_eq!(
        fs::read_to_string(a.join("two").join("SKILL.md")).unwrap(),
        "B1"
    );

    // Both change the same file: the newer wins, the other is kept. Same
    // size, so only the times tell the change: set them, a write right after
    // the sync can fall in the same tick of the file system's clock.
    let now = std::time::SystemTime::now();
    let write_at = |path: &std::path::Path, text: &str, at| {
        fs::write(path, text).unwrap();
        fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(at)
            .unwrap();
    };
    write_at(
        &a.join("one").join("SKILL.md"),
        "A2",
        now + Duration::from_secs(2),
    );
    write_at(
        &b.join("one").join("SKILL.md"),
        "B2",
        now + Duration::from_secs(4),
    );
    sync_tree(&p, &a, &b, "t", true, "skills", &mut r).unwrap();
    assert_eq!(
        fs::read_to_string(a.join("one").join("SKILL.md")).unwrap(),
        "B2"
    );
    assert!(r.warnings.iter().any(|w| w.contains("both sides")));
    assert_eq!(fs::read_dir(p.state.join("conflicts")).unwrap().count(), 1);

    // Deleted on the replica: deleted in the master, empty folder too.
    fs::remove_dir_all(b.join("two")).unwrap();
    sync_tree(&p, &a, &b, "t", true, "skills", &mut r).unwrap();
    assert!(!a.join("two").exists());
}

/// BUG-V6-16: an instruction file deleted on one side is deleted on the
/// other only after a copy in `.sync\deleted`.
#[test]
fn deleted_instructions_are_kept() {
    let (_t, _kit, p) = kit("tree-deleted");
    let (a, b) = (p.agents.join("AGENTS.md"), p.codex.join("AGENTS.md"));
    fs::create_dir_all(&p.agents).unwrap();
    fs::write(&a, "my rules").unwrap();
    let mut r = Report::default();
    sync_tree(&p, &a, &b, "i", false, "AGENTS.md", &mut r).unwrap();
    assert_eq!(fs::read_to_string(&b).unwrap(), "my rules");
    fs::remove_file(&b).unwrap();
    sync_tree(&p, &a, &b, "i", false, "AGENTS.md", &mut r).unwrap();
    assert!(!a.exists(), "deleted");
    let kept: Vec<_> = fs::read_dir(p.state.join("deleted"))
        .unwrap()
        .flatten()
        .collect();
    assert_eq!(kept.len(), 1);
    assert!(kept[0]
        .file_name()
        .to_string_lossy()
        .ends_with("-AGENTS.md"));
    assert_eq!(fs::read_to_string(kept[0].path()).unwrap(), "my rules");
}

/// An editor saving through a temporary file: the file is away for a
/// moment and back with new text. Nothing is deleted, the new text wins.
#[test]
fn a_file_away_for_a_moment_is_not_deleted() {
    let (_t, _kit, p) = kit("tree-away");
    let (a, b) = (p.agents.join("AGENTS.md"), p.claude.join("CLAUDE.md"));
    fs::create_dir_all(&p.agents).unwrap();
    fs::write(&a, "v1").unwrap();
    let mut r = Report::default();
    sync_tree(&p, &a, &b, "i", false, "AGENTS.md", &mut r).unwrap();
    fs::remove_file(&b).unwrap();
    let back = b.clone();
    let editor = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(2));
        fs::write(&back, "v2, saved").unwrap();
    });
    sync_tree(&p, &a, &b, "i", false, "AGENTS.md", &mut r).unwrap();
    editor.join().unwrap();
    assert!(a.exists(), "not deleted");
    assert!(!p.state.join("deleted").exists());
    sync_tree(&p, &a, &b, "i", false, "AGENTS.md", &mut r).unwrap();
    assert_eq!(fs::read_to_string(&a).unwrap(), "v2, saved");
}

#[cfg(windows)]
#[test]
fn junction_inside_is_never_written_through() {
    let (t, _kit, p) = kit("tree-confined");
    let (a, b) = (p.agents.join("x"), p.claude.join("x"));
    let outside = t.0.join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(a.join("sub")).unwrap();
    fs::write(a.join("sub").join("f.txt"), "hi").unwrap();
    fs::create_dir_all(&b).unwrap();
    junction::create(&outside, b.join("sub")).unwrap();
    let mut r = Report::default();
    sync_tree(&p, &a, &b, "c", true, "x", &mut r).unwrap();
    assert!(!outside.join("f.txt").exists(), "nothing written outside");
    assert!(r.warnings.iter().any(|w| w.contains("linked elsewhere")));
}
