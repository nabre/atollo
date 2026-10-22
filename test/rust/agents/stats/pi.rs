use super::*;

use crate::kit::tests::TempDir;

/// Lines as Pi 1.3 writes them: a reply with its usage (from a session on
/// the kit's local model), a user message, a summary with its own usage.
fn session(now: i64) -> String {
    [
        r#"{"type":"session","version":3,"id":"s","timestamp":"2026-10-21T22:12:35.570Z","cwd":"/p"}"#.to_owned(),
        format!(r#"{{"type":"message","id":"a","timestamp":"2026-10-21T22:12:36.000Z","message":{{"role":"user","content":"Hello","timestamp":{now}}}}}"#),
        format!(r#"{{"type":"message","id":"b","timestamp":"2026-10-21T22:12:40.000Z","message":{{"role":"assistant","api":"openai-completions","provider":"atollo","model":"qwen2.5-0.5b","usage":{{"input":20,"output":10,"cacheRead":1637,"cacheWrite":5,"totalTokens":1672}},"stopReason":"stop","timestamp":{now}}}}}"#),
        r#"{"type":"compaction","id":"c","timestamp":"2026-10-21T22:12:41.000Z","summary":"x","usage":{"input":100,"output":50,"cacheRead":0,"cacheWrite":0},"provider":"atollo","model":"qwen2.5-0.5b"}"#.to_owned(),
    ]
    .join("\n")
}

#[test]
fn replies_and_summaries_count() {
    let now = super::super::util::now_ms();
    let parsed = parse(&session(now));
    assert_eq!(parsed.records.len(), 2);
    let r = &parsed.records[0];
    assert_eq!(r.model.as_deref(), Some("atollo/qwen2.5-0.5b"));
    assert_eq!(r.tokens, Tokens::new(25, 1637, 10));
    assert_eq!(r.time, Some(now));
    assert_eq!(parsed.records[1].tokens, Tokens::new(100, 0, 50));
    assert!(parsed.last.is_some());
}

/// The sessions of every folder under `sessions`, in the week.
#[test]
fn usage_of_the_sessions() {
    let tmp = TempDir::new("stats-pi");
    let dir = tmp.0.join("sessions").join("--p--");
    std::fs::create_dir_all(&dir).unwrap();
    let now = super::super::util::now_ms();
    std::fs::write(dir.join("2026_s.jsonl"), session(now)).unwrap();
    let u = usage(&tmp.0, &Period::now());
    assert!(u.tokens);
    assert_eq!(u.today.output, 60);
    assert_eq!(u.models.len(), 1);
}
