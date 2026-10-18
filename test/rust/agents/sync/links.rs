use super::*;
use crate::agents::sync::tests::kit;
use crate::agents::sync::{run, tools};

#[test]
fn skills_become_junctions_and_deletions_are_kept() {
    let (_t, kit, p) = kit("links");
    let mine = p.codex.join("skills").join("review");
    fs::create_dir_all(&mine).unwrap();
    fs::write(mine.join("SKILL.md"), "review").unwrap();
    fs::create_dir_all(p.codex.join("skills").join(".system")).unwrap();
    let r = run(&kit, None);
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    assert_eq!(
        fs::read_to_string(p.agents.join("skills").join("review").join("SKILL.md")).unwrap(),
        "review"
    );
    for t in tools(&p) {
        let link = t.skills.join("review");
        assert!(is_reparse(&link), "{} linked", t.name);
        assert_eq!(fs::read_to_string(link.join("SKILL.md")).unwrap(), "review");
    }
    assert!(
        p.codex.join("skills").join(".system").is_dir(),
        "dot folders untouched"
    );

    // Claude deletes it: kept in .sync\deleted, links gone everywhere.
    junction::delete(p.claude.join("skills").join("review")).unwrap();
    fs::remove_dir(p.claude.join("skills").join("review")).unwrap();
    run(&kit, None);
    assert!(!p.agents.join("skills").join("review").exists());
    assert_eq!(fs::read_dir(p.state.join("deleted")).unwrap().count(), 1);
    assert!(fs::symlink_metadata(p.codex.join("skills").join("review")).is_err());
}

#[test]
fn junction_to_old_place_is_remade() {
    let (t, kit, p) = kit("links-moved");
    fs::create_dir_all(p.agents.join("skills").join("s1")).unwrap();
    fs::write(p.agents.join("skills").join("s1").join("SKILL.md"), "x").unwrap();
    let old = t.0.join("old-kit-skill");
    fs::create_dir_all(&old).unwrap();
    fs::create_dir_all(p.claude.join("skills")).unwrap();
    junction::create(&old, p.claude.join("skills").join("s1")).unwrap();
    run(&kit, None);
    let target = junction::get_target(p.claude.join("skills").join("s1")).unwrap();
    assert!(
        same(&target, &p.agents.join("skills").join("s1")),
        "{}",
        target.display()
    );
}
