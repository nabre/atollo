use std::time::SystemTime;

use super::*;
use crate::kit::tests::TempDir;

pub fn kit(name: &str) -> (TempDir, Kit, Paths) {
    let tmp = TempDir::new(name);
    let kit = Kit::at(tmp.0.join("k (1) & co"), crate::test_support::kit_arch());
    kit.prepare().unwrap();
    let p = Paths::of(&kit);
    (tmp, kit, p)
}

#[test]
fn instructions_reach_every_agent_and_back() {
    let (_t, kit, p) = kit("sync-instr");
    fs::write(p.agents.join("AGENTS.md").tap_dir(), "shared v1").unwrap();
    let r = run(&kit, None);
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    for f in [
        p.claude.join("CLAUDE.md"),
        p.codex.join("AGENTS.md"),
        p.gemini.join("AGENTS.md"),
    ] {
        assert_eq!(
            fs::read_to_string(&f).unwrap(),
            "shared v1",
            "{}",
            f.display()
        );
    }
    // Codex changes it: everyone gets it.
    std::thread::sleep(Duration::from_millis(20));
    fs::write(p.codex.join("AGENTS.md"), "from codex").unwrap();
    run(&kit, None);
    assert_eq!(
        fs::read_to_string(p.agents.join("AGENTS.md")).unwrap(),
        "from codex"
    );
    assert_eq!(
        fs::read_to_string(p.claude.join("CLAUDE.md")).unwrap(),
        "from codex"
    );
    assert_eq!(
        fs::read_to_string(p.gemini.join("AGENTS.md")).unwrap(),
        "from codex"
    );
    // Claude deletes it: deleted everywhere.
    fs::remove_file(p.claude.join("CLAUDE.md")).unwrap();
    run(&kit, None);
    assert!(!p.agents.join("AGENTS.md").exists());
    assert!(!p.gemini.join("AGENTS.md").exists());
}

trait TapDir {
    fn tap_dir(self) -> Self;
}
impl TapDir for PathBuf {
    fn tap_dir(self) -> Self {
        fs::create_dir_all(self.parent().unwrap()).unwrap();
        self
    }
}

#[test]
fn rule_and_trust() {
    let (_t, kit, p) = kit("sync-trust");
    fs::create_dir_all(p.projects.join("app")).unwrap();
    fs::create_dir_all(p.projects.join(".hidden")).unwrap();
    fs::create_dir_all(&p.claude).unwrap();
    fs::write(
        p.claude_json(),
        r#"{"projects":{"x":{"history":[1]}},"numStartups":3}"#,
    )
    .unwrap();
    run(&kit, None);
    assert!(fs::read_to_string(p.claude.join("rules").join("atollo.md"))
        .unwrap()
        .contains("Atollo"));
    assert!(p
        .gemini
        .join("config")
        .join("rules")
        .join("atollo.md")
        .is_file());
    // SEC-V6-02: a sync trusts nothing.
    let read = || -> Value {
        serde_json::from_str(&fs::read_to_string(p.claude_json()).unwrap()).unwrap()
    };
    let app = p
        .projects
        .join("app")
        .display()
        .to_string()
        .replace('\\', "/");
    let cfg = read();
    assert!(cfg["projects"].get(&app).is_none(), "trusted by nothing");
    assert_eq!(cfg["numStartups"], 3, "the rest kept");
    assert_eq!(cfg["projects"]["x"]["history"][0], 1);
    let codex = fs::read_to_string(p.codex.join("config.toml")).unwrap();
    assert!(!codex.contains("[projects."), "{codex}");
}

#[test]
fn lock_is_released_and_stale_lock_taken() {
    let (_t, _kit, p) = kit("sync-lock");
    {
        let _l = Lock::take(&p.state).unwrap();
        assert!(p.state.join("lock").exists());
    }
    assert!(!p.state.join("lock").exists());
    fs::create_dir_all(p.state.join("lock")).unwrap();
    fs::write(p.state.join("lock").join("pid"), "4000000000").unwrap();
    assert!(Lock::take(&p.state).is_ok(), "holder gone: stale");
}

/// BUG-V6-18: the silence of the holder is measured on its heartbeat, not
/// on the lock folder's time; a held lock keeps beating.
#[test]
fn lock_staleness_follows_the_heartbeat() {
    let (_t, _kit, p) = kit("sync-lock-beat");
    let lock = p.state.join("lock");
    let old = SystemTime::now() - Duration::from_secs(600);
    let set_old = |f: &Path| {
        fs::File::options()
            .write(true)
            .open(f)
            .unwrap()
            .set_modified(old)
            .unwrap();
    };
    // A holder alive (this process) but silent for 10 minutes, in a folder
    // just made: stale.
    fs::create_dir_all(&lock).unwrap();
    fs::write(lock.join("pid"), std::process::id().to_string()).unwrap();
    fs::write(lock.join("beat"), "").unwrap();
    set_old(&lock.join("beat"));
    let held = Lock::take(&p.state).unwrap();

    // Held: the beat is renewed by itself, without any step.
    set_old(&lock.join("beat"));
    std::thread::sleep(BEAT * 4);
    assert!(Lock::silence(&lock) < Duration::from_secs(60));
    drop(held);
    assert!(!lock.exists());
}

#[test]
fn canonical_sorts_keys() {
    let a: Value = serde_json::from_str(r#"{"b":1,"a":{"y":2,"x":[1,"s"]}}"#).unwrap();
    let b: Value = serde_json::from_str(r#"{"a":{"x":[1,"s"],"y":2},"b":1}"#).unwrap();
    assert_eq!(canonical(&a), canonical(&b));
}
