use super::*;
use crate::kit::tests::TempDir;
use serde_json::json;
use std::fs;

#[test]
fn windows_from_server_and_logs() {
    let cx = Ctx::for_tests("it");
    let now = cx.now;
    let j = json!({
        "plan_type": "plus",
        "rate_limit": {
            "primary_window": {"used_percent": 30, "limit_window_seconds": 18000, "reset_after_seconds": 600},
            "secondary_window": {"used_percent": 0, "limit_window_seconds": 604800, "reset_after_seconds": 604800}
        },
        "code_review_rate_limit": {"primary_window": {"used_percent": 95, "limit_window_seconds": 604800}},
        "additional_rate_limits": [{"limit_name": "GPT-5-Codex-Mini", "rate_limit": {"primary_window": {"used_percent": 10, "limit_window_seconds": 18000}}}]
    });
    let w = windows_of(&j, now, &cx);
    let labels: Vec<&str> = w.iter().map(|w| w.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "5 ore",
            "Settimana",
            "Settimana · Code review",
            "5 ore · GPT-5-Codex-Mini"
        ]
    );
    assert_eq!(w[0].resets_at, Some(now + 600_000));
    assert_eq!(w[1].resets_at, None, "a window not started has no reset");
    assert_eq!(w[2].level, "critical");

    // a window of the logs that reset since it was read
    let old = window(
        &json!({"used_percent": 80, "window_minutes": 300, "resets_at": 1}),
        0,
        None,
        &cx,
    );
    assert_eq!((old.percent, old.resets_at), (Some(0.0), None));
}

#[test]
#[cfg_attr(not(windows), ignore = "session folders created with backslash paths")]
fn sessions_count_growth_once() {
    let tmp = TempDir::new("stats-codex");
    let dir = tmp.0.join(".codex");
    let day = dir.join(r"sessions\2026\10\14");
    fs::create_dir_all(&day).unwrap();
    fs::create_dir_all(dir.join("archived_sessions")).unwrap();
    let cx = Ctx::for_tests("en");
    let t = cx.now - 60_000;
    let ev = |ts: i64, total: u64, input: u64, cached: u64, out: u64| {
        format!(
            r#"{{"timestamp":{ts},"type":"event_msg","payload":{{"type":"token_count","info":{{"total_token_usage":{{"input_tokens":{input},"cached_input_tokens":{cached},"output_tokens":{out},"total_tokens":{total}}}}},"rate_limits":{{"plan_type":"pro","primary":{{"used_percent":42,"window_minutes":300}}}}}}}}"#
        )
    };
    let text = [
        format!(r#"{{"timestamp":{t},"type":"turn_context","payload":{{"model":"gpt-5"}}}}"#),
        ev(t, 110, 100, 40, 10),
        ev(t + 1, 110, 100, 40, 10), // repeated: not counted again
        ev(t + 2, 180, 160, 60, 20),
    ]
    .join("\n");
    fs::write(day.join("rollout-a.jsonl"), &text).unwrap();
    // a fork copies the same events
    fs::write(dir.join(r"archived_sessions\rollout-b.jsonl"), &text).unwrap();
    fs::write(day.join("other.jsonl"), &text).unwrap();
    let (u, l) = from_logs(&dir, &Period::now(), &cx);
    // first event: 60 new + 40 cached + 10 out; then growth: 40 + 20 + 10
    assert_eq!(u.week, Tokens::new(100, 60, 20));
    assert_eq!(u.sessions_week, 2);
    assert_eq!(u.models.get("gpt-5"), Some(&180));
    assert_eq!(l.plan.as_deref(), Some("pro"));
    assert_eq!(l.windows.len(), 1);
    assert_eq!(l.windows[0].label, "5 hours");
    assert_eq!(l.windows[0].percent, Some(42.0));
}

#[test]
fn logins() {
    let tmp = TempDir::new("stats-codex-login");
    assert!(read_login(&tmp.0).is_none());
    fs::write(tmp.0.join("auth.json"), r#"{"OPENAI_API_KEY":"k"}"#).unwrap();
    assert!(matches!(read_login(&tmp.0), Some(Login::ApiKey)));
    let claims = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(br#"{"exp":2000000000}"#);
    fs::write(
        tmp.0.join("auth.json"),
        format!(r#"{{"tokens":{{"access_token":"h.{claims}.s","account_id":"acc"}}}}"#),
    )
    .unwrap();
    let Some(Login::Token {
        expires, account, ..
    }) = read_login(&tmp.0)
    else {
        panic!("no login");
    };
    assert_eq!(
        (expires, account.as_deref()),
        (Some(2_000_000_000_000), Some("acc"))
    );
}
