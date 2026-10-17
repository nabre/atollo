use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
#[cfg_attr(not(windows), ignore = "credential files written with backslash paths")]
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
        kit.home().join(r".claude\.credentials.json"),
        r#"{"claudeAiOauth":{"accessToken":"t"}}"#,
    )
    .unwrap();
    fs::create_dir_all(kit.home().join(".codex")).unwrap();
    fs::write(
        kit.home().join(r".codex\auth.json"),
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
