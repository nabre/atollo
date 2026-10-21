use super::*;
use crate::agents::sync::tests::kit;
use crate::agents::sync::{run, trust_project};
use crate::kit::{tests::TempDir, Arch, Kit, Os, Target};

fn codex_text(p: &Paths) -> String {
    fs::read_to_string(p.codex.join("config.toml")).unwrap_or_default()
}

/// Launching in a project trusts that project for Codex once, with its own
/// table; a table already there for it is left as it is.
#[test]
fn codex_table_written_once_and_the_user_s_kept() {
    let (_t, kit, p) = kit("trust-codex");
    for n in ["app", "other", "mine"] {
        fs::create_dir_all(p.projects.join(n)).unwrap();
    }
    fs::create_dir_all(&p.codex).unwrap();
    let mine = codex_key(&p, &p.projects.join("mine"));
    fs::write(
        p.codex.join("config.toml"),
        format!("model = \"o9\"\n\n[projects.'{mine}']\ntrust_level = \"untrusted\"\n"),
    )
    .unwrap();
    trust_project(&kit, &p.projects.join("app"));
    trust_project(&kit, &p.projects.join("app"));
    trust_project(&kit, &p.projects.join("mine"));
    let text = codex_text(&p);
    let app = codex_key(&p, &p.projects.join("app"));
    assert_eq!(
        text.matches(&format!("[projects.'{app}']\ntrust_level = \"trusted\""))
            .count(),
        1,
        "{text}"
    );
    assert!(
        text.contains("trust_level = \"untrusted\""),
        "the user's choice"
    );
    assert!(!text.contains("other"), "only the project launched");
    run(&kit, None);
    let after = codex_text(&p);
    assert!(after.contains(&format!("[projects.'{app}']")), "{after}");
    assert_eq!(after.matches("[projects.").count(), 2, "{after}");
}

/// Only a folder right inside `projects\` is trusted.
#[test]
fn only_projects_of_the_kit() {
    let (t, kit, p) = kit("trust-outside");
    let outside = t.0.join("elsewhere");
    fs::create_dir_all(&outside).unwrap();
    fs::create_dir_all(p.projects.join("a").join("sub")).unwrap();
    for dir in [
        outside,
        p.projects.clone(),
        p.projects.join("a").join("sub"),
    ] {
        trust_project(&kit, &dir);
    }
    assert!(!p.claude_json().exists());
    assert!(!codex_text(&p).contains("[projects."));
}

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

/// PORT-V6-02: Codex's key is lower case only on Windows.
#[test]
fn codex_keys_keep_their_case_off_windows() {
    let tmp = TempDir::new("trust-case");
    for os in [Os::Linux, Os::Win32] {
        let kit = Kit::at_target(
            tmp.0.join(format!("Kit-{os:?}")),
            Target::new(os, Arch::X64),
        );
        let p = Paths::of(&kit);
        let dir = p.projects.join("App");
        fs::create_dir_all(&dir).unwrap();
        codex(&p, &dir).unwrap();
        let app = dir.display().to_string();
        let key = if os == Os::Win32 {
            app.to_lowercase()
        } else {
            app
        };
        let text = codex_text(&p);
        assert!(
            text.contains(&format!("[projects.'{key}']")),
            "{os:?}: {text}"
        );
        codex(&p, &dir).unwrap();
        assert_eq!(codex_text(&p), text, "{os:?}: once");
    }
}
