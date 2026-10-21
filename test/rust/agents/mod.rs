use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn sign_in_detection() {
    let tmp = TempDir::new("agents");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let env = KitEnv::default();
    assert!(status(&kit).iter().all(|a| !a.signed_in && !a.installed));
    fs::create_dir_all(kit.home().join(".claude")).unwrap();
    fs::write(
        kit.home().join(".claude").join(".credentials.json"),
        r#"{"claudeAiOauth":{"accessToken":"t"}}"#,
    )
    .unwrap();
    fs::create_dir_all(kit.home().join(".codex")).unwrap();
    fs::write(
        kit.home().join(".codex").join("auth.json"),
        r#"{"OPENAI_API_KEY":"k"}"#,
    )
    .unwrap();
    let s = status(&kit);
    assert_eq!(s[0].sign_in_kind.as_deref(), Some("account"));
    assert_eq!(s[1].sign_in_kind.as_deref(), Some("api key"));
    assert!(!s[2].signed_in);
    sign_out(&kit, &env, "claude").unwrap();
    assert!(!status(&kit)[0].signed_in);
}

/// Claude Code with the token of `claude setup-token` (macOS): signed in
/// with a token; "Esci" deletes it.
#[test]
fn claude_token_sign_in() {
    let tmp = TempDir::new("agents-token");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let env = KitEnv::default();
    assert_eq!(sign_in(&kit, "claude"), None);
    let token = format!("sk-ant-oat01-{}", "a".repeat(90));
    claude_token::save(&kit, &token).unwrap();
    assert_eq!(sign_in(&kit, "claude"), Some("token"));
    let info = status(&kit);
    let claude = info.iter().find(|a| a.id == "claude").unwrap();
    assert_eq!(
        claude.token_link,
        cfg!(target_os = "macos"),
        "offered on macOS only"
    );
    assert!(info.iter().all(|a| a.id == "claude" || !a.token_link));
    sign_out(&kit, &env, "claude").unwrap();
    assert_eq!(sign_in(&kit, "claude"), None);
    assert!(!claude_token::path(&kit).exists());
}

/// The Gemini key of the settings: Antigravity signed in with it, and the
/// key in Antigravity's environment only, never in the kit's nor in the
/// other agents' (SEC-V6-02).
#[test]
fn gemini_key_for_antigravity_only() {
    let tmp = TempDir::new("agents-gemini");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let env = KitEnv::default();
    assert_eq!(sign_in(&kit, "antigravity"), None);
    assert_eq!(launch_env(&kit, &env, "antigravity").get(GEMINI_VAR), None);
    let settings = crate::settings::Settings {
        gemini_api_key: Some("AIza-key".into()),
        ..Default::default()
    };
    settings.save(&kit).unwrap();
    assert_eq!(sign_in(&kit, "antigravity"), Some("api key"));
    assert_eq!(
        launch_env(&kit, &env, "antigravity").get(GEMINI_VAR),
        Some("AIza-key")
    );
    for other in ["claude", "codex"] {
        assert_eq!(launch_env(&kit, &env, other).get(GEMINI_VAR), None);
    }
    let kit_env = KitEnv::build(
        &kit,
        &crate::settings::Settings::load(&kit).settings,
        "en",
        &crate::env::Host::from_pairs(std::iter::empty()),
    );
    assert_eq!(kit_env.get(GEMINI_VAR), None);
}
