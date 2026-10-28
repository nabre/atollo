use super::*;
use crate::test_support::prepared_kit;

fn project(kit: &Kit, name: &str) {
    fs::create_dir_all(kit.projects().join(name)).unwrap();
}

#[test]
fn names_are_those_of_a_folder() {
    for good in ["default", "lavoro", "Cliente ACME", "a-b_c"] {
        assert!(valid_name(good), "{good}");
    }
    for bad in ["", ".", "..", "a/b", r"a\b", "con", "x.", " x ", "a:b"] {
        assert!(!valid_name(bad), "{bad:?}");
    }
    let (_tmp, kit) = prepared_kit("accounts-names");
    assert!(create(&kit, Agent::Claude, "../x").is_err());
    assert!(
        create(&kit, Agent::Antigravity, "x").is_err(),
        "no accounts"
    );
}

/// A project uses the default account until it picks one; the choice is
/// kept in its settings, the default one written as nothing.
#[test]
fn a_projects_choice_is_kept() {
    let (_tmp, kit) = prepared_kit("accounts-choice");
    project(&kit, "shop");
    assert_eq!(chosen(&kit, "shop", Agent::Codex), DEFAULT);
    create(&kit, Agent::Codex, "lavoro").unwrap();
    set(&kit, "shop", Agent::Codex, "lavoro");
    assert_eq!(chosen(&kit, "shop", Agent::Codex), "lavoro");
    assert_eq!(chosen(&kit, "shop", Agent::Claude), DEFAULT);
    assert_eq!(
        crate::projects::settings::load(&kit, "shop").agent_accounts["codex"],
        "lavoro"
    );
    set(&kit, "shop", Agent::Codex, DEFAULT);
    assert!(crate::projects::settings::load(&kit, "shop")
        .agent_accounts
        .is_empty());
}

/// The list: every agent, the default account always there, the projects
/// that use each one and whether it signed in.
#[test]
fn the_list_tells_who_uses_what() {
    let (_tmp, kit) = prepared_kit("accounts-list");
    project(&kit, "shop");
    project(&kit, "blog");
    create(&kit, Agent::Claude, "lavoro").unwrap();
    set(&kit, "shop", Agent::Claude, "lavoro");
    fs::write(
        dir(&kit, Agent::Claude, "lavoro").join(".credentials.json"),
        "{}",
    )
    .unwrap();
    let all = list(&kit);
    assert_eq!(all.len(), Agent::ALL.len());
    let claude = all.iter().find(|a| a.agent == Agent::Claude).unwrap();
    assert!(claude.supported);
    let names: Vec<&str> = claude.accounts.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["default", "lavoro"]);
    assert_eq!(claude.accounts[0].used_by, ["blog"]);
    assert_eq!(claude.accounts[1].used_by, ["shop"]);
    assert!(claude.accounts[1].signed_in && !claude.accounts[0].signed_in);
    let agy = all.iter().find(|a| a.agent == Agent::Antigravity).unwrap();
    assert!(!agy.supported);
}

/// Deleting: never the default account, never one a project uses; a
/// rename takes the projects along.
#[test]
fn delete_is_refused_while_used() {
    let (_tmp, kit) = prepared_kit("accounts-delete");
    project(&kit, "shop");
    create(&kit, Agent::Pi, "casa").unwrap();
    set(&kit, "shop", Agent::Pi, "casa");
    assert_eq!(
        delete_refused(&kit, Agent::Pi, "casa"),
        Some(vec!["shop".to_owned()])
    );
    assert!(delete(&kit, Agent::Pi, "casa").is_err());
    assert_eq!(delete_refused(&kit, Agent::Pi, DEFAULT), Some(Vec::new()));
    rename(&kit, Agent::Pi, "casa", "personale").unwrap();
    assert_eq!(chosen(&kit, "shop", Agent::Pi), "personale");
    assert!(!dir(&kit, Agent::Pi, "casa").exists());
    assert!(rename(&kit, Agent::Pi, DEFAULT, "x").is_err());
    set(&kit, "shop", Agent::Pi, DEFAULT);
    delete(&kit, Agent::Pi, "personale").unwrap();
    assert!(!dir(&kit, Agent::Pi, "personale").exists());
}

/// A session gets its project's account, made when missing; «Esci» takes
/// only that account's credentials.
#[test]
fn a_session_gets_its_projects_account() {
    let (_tmp, kit) = prepared_kit("accounts-session");
    project(&kit, "shop");
    let (d, var) = for_session(&kit, Some("shop"), Agent::Claude)
        .unwrap()
        .unwrap();
    assert_eq!(var, "CLAUDE_CONFIG_DIR");
    assert_eq!(d, dir(&kit, Agent::Claude, DEFAULT));
    assert!(d.is_dir());
    set(&kit, "shop", Agent::Codex, "lavoro");
    let (d, var) = for_session(&kit, Some("shop"), Agent::Codex)
        .unwrap()
        .unwrap();
    assert_eq!(
        (d.clone(), var),
        (dir(&kit, Agent::Codex, "lavoro"), "CODEX_HOME")
    );
    assert!(for_session(&kit, Some("shop"), Agent::Antigravity)
        .unwrap()
        .is_none());
    // A folder's sandbox takes the default account, whatever a project chose
    // (D-187).
    let (folder, _) = for_session(&kit, None, Agent::Codex).unwrap().unwrap();
    assert_eq!(folder, dir(&kit, Agent::Codex, DEFAULT));
    fs::write(d.join("auth.json"), "{}").unwrap();
    fs::write(d.join("config.toml"), "").unwrap();
    sign_out(&kit, Agent::Codex, "lavoro").unwrap();
    assert!(!d.join("auth.json").exists() && d.join("config.toml").exists());
}
