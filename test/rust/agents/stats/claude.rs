use super::*;
use crate::kit::tests::TempDir;
use serde_json::json;
use std::fs;

#[test]
fn plan_windows_and_credits() {
    let cx = Ctx::for_tests("it");
    let j = json!({
        "five_hour": {"utilization": 12.34, "resets_at": "2026-10-14T13:00:00+00:00"},
        "seven_day": {"utilization": 40.0},
        "limits": [
            {"kind": "session", "percent": 12, "resets_at": "2026-10-14T13:00:00Z"},
            {"kind": "weekly_all", "percent": 40},
            {"kind": "weekly_scoped", "percent": 75, "scope": {"model": {"display_name": "Opus"}}},
            {"kind": "weekly_scoped"}
        ]
    });
    let w = windows(&j, &cx);
    let labels: Vec<&str> = w.iter().map(|w| w.label.as_str()).collect();
    assert_eq!(labels, ["5 ore", "Settimana", "Settimana · Opus"]);
    assert_eq!(w[0].percent, Some(12.3));
    assert!(w[0].resets_at.is_some());
    assert_eq!(w[2].level, "warning");

    let old = json!({"five_hour": {"utilization": 5}, "seven_day_opus": {"utilization": 95}});
    let w = windows(&old, &cx);
    assert_eq!(w.len(), 2);
    assert_eq!(w[1].label, "Settimana · Opus");
    assert_eq!(w[1].level, "critical");

    let spend = json!({"spend": {"enabled": true,
        "used": {"amount_minor": 1250, "exponent": 2, "currency": "EUR"},
        "limit": {"amount_minor": 5000, "exponent": 2}}});
    let c = credits(&spend).unwrap();
    assert_eq!(
        (c.used, c.limit, c.currency.as_str(), c.percent),
        (12.5, Some(50.0), "EUR", Some(25.0))
    );
    let extra = json!({"extra_usage": {"is_enabled": true, "used_credits": 300, "monthly_limit": 1000, "decimal_places": 2}});
    let c = credits(&extra).unwrap();
    assert_eq!((c.used, c.limit, c.percent), (3.0, Some(10.0), Some(30.0)));
    assert!(credits(&json!({"extra_usage": {"is_enabled": false}})).is_none());
}

#[test]
fn sessions_and_tokens() {
    let tmp = TempDir::new("stats-claude");
    let dir = tmp.0.join(".claude");
    let proj = dir.join("projects").join("p1");
    fs::create_dir_all(proj.join("subagents")).unwrap();
    let now = super::super::util::now_ms();
    let t = now - 3_600_000;
    let line = |id: &str, out: u64| {
        format!(
            r#"{{"type":"assistant","timestamp":{t},"requestId":"r{id}","message":{{"id":"m{id}","model":"claude-opus","usage":{{"input_tokens":10,"cache_creation_input_tokens":5,"cache_read_input_tokens":100,"output_tokens":{out}}}}}}}"#
        )
    };
    // one reply written twice (two content blocks), one more reply
    fs::write(
        proj.join("s1.jsonl"),
        format!(
            "{}\r\n{}\n{}\nnot json\n{{\"type\":\"user\"}}\n",
            line("1", 3),
            line("1", 7),
            line("2", 1)
        ),
    )
    .unwrap();
    // the same reply copied in a resumed session
    fs::write(proj.join("s2.jsonl"), line("2", 1)).unwrap();
    fs::write(proj.join("subagents").join("agent-x.jsonl"), line("3", 2)).unwrap();
    let p = Period::now();
    let u = usage(&dir, &p);
    assert_eq!(u.week, Tokens::new(45, 300, 10));
    assert_eq!(u.sessions_week, 2, "subagents are not sessions");
    assert_eq!(u.models.get("claude-opus"), Some(&355));
    // parsed again only when a file changes
    fs::write(
        proj.join("s2.jsonl"),
        format!("{}\n{}", line("2", 1), line("4", 1)),
    )
    .unwrap();
    assert_eq!(usage(&dir, &p).week.total, 355 + 116);
}

#[test]
fn logins() {
    let tmp = TempDir::new("stats-claude-login");
    assert!(read_login(&tmp.0).is_none());
    fs::write(
        tmp.0.join(".credentials.json"),
        r#"{"claudeAiOauth":{"accessToken":"tok","expiresAt":1000,"subscriptionType":"max"}}"#,
    )
    .unwrap();
    let Some(Login::Token {
        token,
        expires,
        plan,
        ..
    }) = read_login(&tmp.0)
    else {
        panic!("no login");
    };
    assert_eq!(
        (token.as_str(), expires, plan.as_deref()),
        ("tok", Some(1_000_000), Some("max"))
    );
}
