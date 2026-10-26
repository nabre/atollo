use super::*;
use crate::isolation::Blocker;
use crate::kit::Target;
use crate::test_support::TempDir;

/// A kit of a Windows PC, whose VM runs `linux-x64`.
fn win_kit(name: &str) -> (TempDir, Kit) {
    let tmp = TempDir::new(name);
    let kit = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    kit.prepare().unwrap();
    (tmp, kit)
}

fn off() -> VmState {
    VmState::Off {
        blockers: vec![Blocker::HypervisorOff],
    }
}

fn info<'a>(all: &'a [AgentInfo], id: &str) -> &'a AgentInfo {
    all.iter().find(|a| a.id == id).unwrap()
}

/// `id` installed in `kit` at `version` (its key files, its version).
fn installed(kit: &Kit, id: &str, version: &str) {
    let c = crate::setup::Catalog::get().find(id).unwrap();
    c.save_version(kit, version).unwrap();
    for p in c.key_paths(kit) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, "x").unwrap();
    }
    assert!(c.is_installed(kit), "{id}");
}

/// The agents as the app shows them (LEVELS.md §7): available only with
/// the VM on, else with the VM's reason; Claude Code and Codex brought by
/// the VM, Pi and Antigravity once their build for the VM's Linux is in
/// the kit (Q11), never what the PC's folder has.
#[test]
fn agents_status_follows_the_vm() {
    let (_tmp, kit) = win_kit("agents-status");
    let s = status(&kit, &off(), "it");
    assert_eq!(s.len(), 4);
    for a in &s {
        assert!(!a.available, "{}", a.id);
        let why = a.unavailable_reason.as_deref().unwrap();
        assert!(why.contains("Windows Hypervisor Platform"), "{why}");
    }
    for id in ["claude", "codex"] {
        let a = info(&s, id);
        assert!(a.installed && a.provided, "{id}");
        assert_eq!(a.version, None);
        assert_eq!(a.runs_in, RunsIn::SbxSandbox);
    }
    for id in ["pi", "antigravity"] {
        let a = info(&s, id);
        assert!(!a.installed && !a.provided, "{id}");
        assert_eq!(a.runs_in, RunsIn::ProjectVm);
    }
    // Pi for the PC (an earlier Atollo's) is no agent of the VM.
    installed(&kit, "pi", "0.70.1");
    assert!(!info(&status(&kit, &off(), "it"), "pi").installed);
    installed(&kit.for_target(Target::LINUX_X64), "pi", "0.70.2");
    let on = status(&kit, &VmState::On, "it");
    let pi = info(&on, "pi");
    assert!(pi.installed && pi.available);
    assert_eq!(pi.version.as_deref(), Some("0.70.2"));
    assert_eq!(pi.unavailable_reason, None);
    assert!(on.iter().all(|a| a.available));
}

/// The sign-in shown is the VM's (LEVELS.md §13 item 12): the Gemini key
/// through `sbx`'s proxy, Pi's local model; a sign-in of the PC saved in
/// the kit is only told as kit data, and «Esci» removes it as files,
/// starting nothing (E17).
#[test]
fn the_sign_in_shown_is_the_vms() {
    let (_tmp, kit) = win_kit("agents-sign-in");
    assert!(status(&kit, &VmState::On, "en")
        .iter()
        .all(|a| !a.signed_in && !a.kit_login));
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
    let s = status(&kit, &VmState::On, "en");
    for id in ["claude", "codex"] {
        let a = info(&s, id);
        assert!(!a.signed_in && a.kit_login, "{id}");
    }
    assert_eq!(sign_in(&kit, "codex"), Some("api key"));

    let settings = crate::common::settings::Settings {
        gemini_api_key: Some("AIza-key".into()),
        ..Default::default()
    };
    settings.save(&kit).unwrap();
    let agy = info(&status(&kit, &VmState::On, "en"), "antigravity").clone();
    assert!(agy.signed_in && !agy.kit_login);
    assert_eq!(agy.sign_in_kind.as_deref(), Some("api key"));

    sign_out(&kit, "claude").unwrap();
    sign_out(&kit, "codex").unwrap();
    assert!(!kit.home().join(".codex").join("auth.json").exists());
    let s = status(&kit, &VmState::On, "en");
    assert!(!info(&s, "claude").kit_login && !info(&s, "codex").kit_login);
    assert!(sign_out(&kit, "nobody").is_err());
}

/// The Gemini key of the settings never enters the kit's environment
/// (SEC-V6-02): only `sbx`'s proxy hands it over, for a sandbox (§9).
#[test]
fn the_gemini_key_is_never_in_the_kits_environment() {
    let (_tmp, kit) = win_kit("agents-gemini");
    let settings = crate::common::settings::Settings {
        gemini_api_key: Some("AIza-key".into()),
        ..Default::default()
    };
    settings.save(&kit).unwrap();
    assert_eq!(gemini_key(&kit).as_deref(), Some("AIza-key"));
    let kit_env = crate::env::KitEnv::build(
        &kit,
        &crate::common::settings::Settings::load(&kit).settings,
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
