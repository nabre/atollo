use super::*;
use crate::agents::sync::tests::kit;

#[test]
fn project_skills_both_ways() {
    let (_t, _kit, p) = kit("project");
    let proj = p.projects.join("app");
    fs::create_dir_all(proj.join(r".agents\skills\lint")).unwrap();
    fs::write(proj.join(r".agents\skills\lint\SKILL.md"), "lint").unwrap();
    let mut r = Report::default();
    sync(&p, &proj.join("src"), &mut r).unwrap_or(());
    sync(&p, &proj, &mut r).unwrap();
    assert_eq!(
        fs::read_to_string(proj.join(r".claude\skills\lint\SKILL.md")).unwrap(),
        "lint"
    );
    // Nothing created where neither folder exists.
    let other = p.projects.join("other");
    fs::create_dir_all(&other).unwrap();
    sync(&p, &other, &mut r).unwrap();
    assert!(!other.join(".claude").exists());
}
