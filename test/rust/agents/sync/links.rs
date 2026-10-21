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
    // The kit's own link, from its old place.
    let old = t.0.join("old kit").join(r"data\agents\skills\s1");
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

/// BUG-V6-30: the user's own links to folders outside the kit stay, also
/// when one has the name of a shared skill (then a warning says so).
#[test]
fn links_of_the_user_are_left_alone() {
    let (t, kit, p) = kit("links-user");
    fs::create_dir_all(p.agents.join("skills").join("s1")).unwrap();
    fs::write(p.agents.join("skills").join("s1").join("SKILL.md"), "x").unwrap();
    let mine = t.0.join("my skills");
    for n in ["own", "s1"] {
        fs::create_dir_all(mine.join(n)).unwrap();
        fs::write(mine.join(n).join("SKILL.md"), n).unwrap();
    }
    let dir = p.claude.join("skills");
    fs::create_dir_all(&dir).unwrap();
    junction::create(mine.join("own"), dir.join("own")).unwrap();
    junction::create(mine.join("s1"), dir.join("s1")).unwrap();
    let r = run(&kit, None);
    for n in ["own", "s1"] {
        assert!(
            same(&junction::get_target(dir.join(n)).unwrap(), &mine.join(n)),
            "{n} kept"
        );
    }
    assert!(
        r.warnings.iter().any(|w| w.contains("skill s1 not linked")),
        "{:?}",
        r.warnings
    );
    assert!(!p.agents.join("skills").join("own").exists(), "not adopted");
    // Not taken for a kit link: removing it later deletes no shared skill.
    junction::delete(dir.join("s1")).unwrap();
    fs::remove_dir(dir.join("s1")).unwrap();
    run(&kit, None);
    assert!(p
        .agents
        .join("skills")
        .join("s1")
        .join("SKILL.md")
        .is_file());
}

#[test]
fn kit_links_told_from_the_user_ones() {
    let shared = Path::new(r"D:\kit\data\agents\skills");
    assert!(kit_link(
        shared,
        Path::new(r"\\?\D:\kit\data\agents\skills\a")
    ));
    assert!(kit_link(
        shared,
        Path::new(r"E:\old kit\data\agents\skills\a")
    ));
    assert!(kit_link(shared, Path::new(r"E:\Data\Agents\Skills\a")));
    assert!(!kit_link(
        shared,
        Path::new(r"D:\kit\data\agents\skillsx\a")
    ));
    assert!(!kit_link(shared, Path::new(r"E:\my skills\a")));
    assert!(!kit_link(shared, Path::new(r"D:\kit\data\agents\skills")));
}
