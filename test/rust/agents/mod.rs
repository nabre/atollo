use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn sign_in_detection() {
    let tmp = TempDir::new("agents");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let env = KitEnv::default();
    assert!(status(&kit, &env)
        .iter()
        .all(|a| !a.signed_in && !a.installed));
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
    let s = status(&kit, &env);
    assert_eq!(s[0].sign_in_kind.as_deref(), Some("account"));
    assert_eq!(s[1].sign_in_kind.as_deref(), Some("api key"));
    assert!(!s[2].signed_in);
    sign_out(&kit, &env, "claude").unwrap();
    assert!(!status(&kit, &env)[0].signed_in);
}

/// Claude Code with the token of `claude setup-token` (macOS): signed in
/// with a token; "Esci" deletes it.
#[test]
fn claude_token_sign_in() {
    let tmp = TempDir::new("agents-token");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let env = KitEnv::default();
    assert_eq!(sign_in(&kit, &env, "claude"), None);
    let token = format!("sk-ant-oat01-{}", "a".repeat(90));
    claude_token::save(&kit, &token).unwrap();
    assert_eq!(sign_in(&kit, &env, "claude"), Some("token"));
    let info = status(&kit, &env);
    let claude = info.iter().find(|a| a.id == "claude").unwrap();
    assert_eq!(
        claude.token_link,
        cfg!(target_os = "macos"),
        "offered on macOS only"
    );
    assert!(info.iter().all(|a| a.id == "claude" || !a.token_link));
    sign_out(&kit, &env, "claude").unwrap();
    assert_eq!(sign_in(&kit, &env, "claude"), None);
    assert!(!claude_token::path(&kit).exists());
}
