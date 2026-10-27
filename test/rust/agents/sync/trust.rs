use super::*;
use crate::agents::sync::run;
use crate::agents::sync::tests::kit;

/// The trust earlier versions gave Claude for the whole `projects\` folder
/// is taken back, once; the projects' own entries stay.
#[test]
fn claude_pretrust_of_projects_taken_back_once() {
    let (_t, kit, p) = kit("trust-forget");
    let fwd = |d: &Path| d.display().to_string().replace('\\', "/");
    let root = fwd(&p.projects);
    let app = fwd(&p.projects.join("app"));
    fs::create_dir_all(&p.claude).unwrap();
    fs::write(
        p.claude_json(),
        serde_json::json!({ "projects": {
            &root: { "hasTrustDialogAccepted": true, "history": [1] },
            &app: { "hasTrustDialogAccepted": true },
        } })
        .to_string(),
    )
    .unwrap();
    run(&kit, None);
    let cfg: Value = read_json(&p.claude_json()).unwrap().unwrap();
    assert!(cfg["projects"][&root]
        .get("hasTrustDialogAccepted")
        .is_none());
    assert_eq!(cfg["projects"][&root]["history"][0], 1);
    assert_eq!(cfg["projects"][&app]["hasTrustDialogAccepted"], true);

    // Trusted again by the user afterwards: left alone.
    let mut cfg = cfg;
    cfg["projects"][&root]["hasTrustDialogAccepted"] = Value::Bool(true);
    write_json(&p.claude_json(), &cfg).unwrap();
    run(&kit, None);
    let cfg: Value = read_json(&p.claude_json()).unwrap().unwrap();
    assert_eq!(cfg["projects"][&root]["hasTrustDialogAccepted"], true);
}
