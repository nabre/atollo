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

/// "Rimuovi l'unità" (BUG-V6-15): background work started after it began
/// is skipped; the one already running is waited for; an agent started
/// again resumes it.
#[test]
fn closing_skips_and_waits_for_background_work() {
    use std::sync::mpsc;
    let closing = Closing::default();
    assert_eq!(closing.run(|| 1), Some(1));
    assert!(closing.wait_idle(Duration::ZERO), "nothing running");

    let (started, is_started) = mpsc::channel();
    let (finish, may_finish) = mpsc::channel::<()>();
    let worker = {
        let closing = closing.clone();
        std::thread::spawn(move || {
            closing.run(|| {
                started.send(()).unwrap();
                may_finish.recv().unwrap();
                "synced"
            })
        })
    };
    is_started.recv().unwrap();
    closing.begin();
    assert_eq!(closing.run(|| 2), None, "skipped while closing");
    assert!(
        !closing.wait_idle(Duration::from_millis(50)),
        "the running one is not over"
    );
    finish.send(()).unwrap();
    assert!(closing.wait_idle(Duration::from_secs(10)));
    assert_eq!(worker.join().unwrap(), Some("synced"));

    closing.reopen();
    assert_eq!(closing.run(|| 3), Some(3));
}

/// The watcher's saves of the login (BUG-V6-17): after a failure the next
/// try waits 6 s, 12 s, 24 s… (at most 10 min); after 8 failures of the
/// same pending run, no more tries; a new run starts again from zero.
#[test]
fn login_save_retries_back_off_and_stop() {
    let t0 = Instant::now();
    let run = Some(std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1));
    let mut retry = SaveRetry::default();
    assert!(!retry.due(None, t0), "nothing pending");
    assert!(retry.due(run, t0));
    let mut at = t0;
    let mut waits = Vec::new();
    for n in 1..=SAVE_MAX_FAILURES {
        let last = retry.failed(at);
        assert_eq!(last, n == SAVE_MAX_FAILURES);
        let next = retry.next.unwrap();
        waits.push((next - at).as_secs());
        if !last {
            assert!(!retry.due(run, next - Duration::from_millis(1)), "too soon");
            assert!(retry.due(run, next));
        }
        at = next;
    }
    assert_eq!(waits, [6, 12, 24, 48, 96, 192, 384, 600]);
    assert!(
        !retry.due(run, at + Duration::from_secs(3600)),
        "given up for this run"
    );
    // agy ran again: another pending run, tried at once.
    let again = Some(std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(2));
    assert!(retry.due(again, at));
    retry.failed(at);
    retry.succeeded();
    assert!(retry.due(again, at), "a success clears the wait");
}
