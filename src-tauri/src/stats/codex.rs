//! Codex: plan limits from OpenAI's server or from its logs, usage from its
//! sessions (port of nabre/atollo `core/stats/codex.js`).

use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;

use base64::Engine;
use serde_json::Value;

use super::util::{
    json_lines, recent_files, summarize, to_time, FileCache, Period, Record, Tokens, Usage, DAY_MS,
};
use super::{read_json, Ctx, Fail, Limits, Login, Window};

/// A window of Codex, from its server (`limit_window_seconds`, `reset_at`,
/// `reset_after_seconds`) or from its logs (`window_minutes`, `resets_at`,
/// `resets_in_seconds`); `at` = when it was read. Named by its length: up to
/// a day in hours, longer = the week.
pub fn window(w: &Value, at: i64, prefix: Option<&str>, cx: &Ctx) -> Window {
    let f = |k: &str| w.get(k).and_then(Value::as_f64);
    let seconds = f("limit_window_seconds")
        .filter(|s| *s > 0.0)
        .or_else(|| f("window_minutes").map(|m| m * 60.0))
        .unwrap_or(0.0);
    let after = f("reset_after_seconds").or_else(|| f("resets_in_seconds"));
    let mut resets_at = to_time(w.get("reset_at"))
        .or_else(|| to_time(w.get("resets_at")))
        .or_else(|| after.map(|a| at + (a * 1000.0) as i64));
    let mut percent = f("used_percent");
    if resets_at.is_some_and(|r| r <= cx.now) {
        // Window reset since it was read: nothing used yet in the new one
        percent = Some(0.0);
        resets_at = None;
    } else if percent.unwrap_or(0.0) == 0.0
        && seconds > 0.0
        && after.is_some_and(|a| a >= seconds - 60.0)
    {
        // Window not started: its "reset" is now + its length and moves
        resets_at = None;
    }
    let name = if seconds > (DAY_MS / 1000) as f64 {
        cx.t("Settimana", &[])
    } else if seconds > 0.0 {
        cx.t("{0} ore", &[&format!("{}", (seconds / 3600.0).round())])
    } else {
        cx.t("Limite", &[])
    };
    let label = match prefix {
        Some(p) => format!("{name} · {p}"),
        None => name,
    };
    Window::new(label, percent, resets_at)
}

/// Token counts of one `token_count` event, cached input apart.
fn tokens_of(u: &Value) -> Tokens {
    let n = |k: &str| u.get(k).and_then(Value::as_u64).unwrap_or(0);
    let cached = n("cached_input_tokens");
    Tokens::new(
        n("input_tokens").saturating_sub(cached),
        cached,
        n("output_tokens"),
    )
}

struct Snapshot {
    time: i64,
    rate_limits: Value,
}

struct Parsed {
    records: Vec<(Option<String>, Record)>,
    last: Option<i64>,
    snap: Option<Snapshot>,
}

static FILES: LazyLock<FileCache<Parsed>> = LazyLock::new(FileCache::new);

fn parse(text: &str) -> Parsed {
    let mut out = Parsed {
        records: Vec::new(),
        last: None,
        snap: None,
    };
    let mut model: Option<String> = None;
    // The previous total of the session
    let mut prev: Option<Value> = None;
    json_lines(text, |o, _| {
        let empty = Value::Null;
        let p = o.get("payload").unwrap_or(&empty);
        let time = to_time(o.get("timestamp"));
        if let Some(t) = time {
            out.last = Some(out.last.map_or(t, |l| l.max(t)));
        }
        if let Some(m) = p.get("model").and_then(Value::as_str) {
            model = Some(m.to_owned());
        }
        if o.get("type").and_then(Value::as_str) != Some("event_msg")
            || p.get("type").and_then(Value::as_str) != Some("token_count")
        {
            return;
        }
        let info = p.get("info").unwrap_or(&empty);
        let total = info.get("total_token_usage").filter(|v| v.is_object());
        let last = info.get("last_token_usage").filter(|v| v.is_object());
        let n = |v: &Value, k: &str| v.get(k).and_then(Value::as_u64).unwrap_or(0);
        // Codex repeats token_count events (the same total): the growth of the
        // total is what a turn used; last_token_usage when there is no total
        let used = match (total, prev.as_ref()) {
            (Some(total), Some(prev)) => {
                if n(total, "total_tokens") == n(prev, "total_tokens") {
                    None
                } else if ["input_tokens", "cached_input_tokens", "output_tokens"]
                    .iter()
                    .all(|k| n(total, k) >= n(prev, k))
                {
                    let d = |k: &str| n(total, k) - n(prev, k);
                    Some(tokens_of(&serde_json::json!({
                        "input_tokens": d("input_tokens"),
                        "cached_input_tokens": d("cached_input_tokens"),
                        "output_tokens": d("output_tokens"),
                    })))
                } else {
                    last.map(tokens_of)
                }
            }
            _ => last.or(total).map(tokens_of),
        };
        if let Some(total) = total {
            prev = Some(total.clone());
        }
        if let Some(used) = used.filter(|u| u.total > 0) {
            let key = total.map(|t| {
                format!(
                    "{}|{}",
                    o.get("timestamp").map(Value::to_string).unwrap_or_default(),
                    n(t, "total_tokens")
                )
            });
            out.records.push((
                key,
                Record {
                    time,
                    model: model.clone(),
                    tokens: used,
                },
            ));
        }
        if let (Some(rl), Some(time)) = (p.get("rate_limits").filter(|r| r.is_object()), time) {
            out.snap = Some(Snapshot {
                time,
                rate_limits: rl.clone(),
            });
        }
    });
    out
}

/// Usage of the sessions and the limits of the last `token_count` event.
pub fn from_logs(dir: &Path, p: &Period, cx: &Ctx) -> (Usage, Limits) {
    let matches = |n: &str| n.starts_with("rollout-") && n.ends_with(".jsonl");
    let mut files = recent_files(&dir.join("sessions"), &matches, p.since(), None);
    files.extend(recent_files(
        &dir.join("archived_sessions"),
        &matches,
        p.since(),
        None,
    ));
    FILES.keep_only(&files.iter().collect::<Vec<_>>());
    let (mut records, mut sessions, mut seen) = (Vec::new(), Vec::new(), HashSet::new());
    let mut snap: Option<(i64, Value)> = None;
    for f in &files {
        let parsed = FILES.get(f, parse);
        // A forked session copies the events of the first one: counted once
        for (key, r) in &parsed.records {
            if let Some(key) = key {
                if !seen.insert(key.clone()) {
                    continue;
                }
            }
            records.push(r.clone());
        }
        if let Some(last) = parsed.last {
            sessions.push(last);
        }
        if let Some(s) = &parsed.snap {
            if snap.as_ref().is_none_or(|(t, _)| s.time > *t) {
                snap = Some((s.time, s.rate_limits.clone()));
            }
        }
    }
    (summarize(&records, &sessions, true, p), limits_of(snap, cx))
}

fn limits_of(snap: Option<(i64, Value)>, cx: &Ctx) -> Limits {
    let Some((time, rl)) = snap else {
        return Limits::error(cx.t(
            "nessun dato ancora: i limiti arrivano con la prossima risposta di Codex",
            &[],
        ));
    };
    Limits {
        plan: rl
            .get("plan_type")
            .and_then(Value::as_str)
            .map(str::to_owned),
        windows: ["primary", "secondary"]
            .iter()
            .filter_map(|k| rl.get(*k))
            .filter(|w| w.get("used_percent").is_some_and(Value::is_number))
            .map(|w| window(w, time, None, cx))
            .collect(),
        updated: Some(time),
        source: Some(cx.t("ultima risposta di Codex", &[])),
        ..Limits::default()
    }
}

/// The ChatGPT login of `.codex\auth.json` (never refreshed: Codex does that).
pub fn read_login(dir: &Path) -> Option<Login> {
    let j = read_json(&dir.join("auth.json"))?;
    let token = j
        .pointer("/tokens/access_token")
        .and_then(Value::as_str)
        .filter(|t| !t.is_empty());
    let Some(token) = token else {
        return j
            .get("OPENAI_API_KEY")
            .and_then(Value::as_str)
            .filter(|k| !k.is_empty())
            .map(|_| Login::ApiKey);
    };
    let expires = token
        .split('.')
        .nth(1)
        .and_then(|p| {
            base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(p.trim_end_matches('='))
                .ok()
        })
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .and_then(|c| c.get("exp").and_then(Value::as_f64))
        .map(|exp| (exp * 1000.0) as i64);
    Some(Login::Token {
        token: token.to_owned(),
        account: j
            .pointer("/tokens/account_id")
            .and_then(Value::as_str)
            .map(str::to_owned),
        expires,
        plan: None,
    })
}

/// Every object with `used_percent` is one window. `rate_limit` = the plan;
/// the others (code review, additional limits) are named after their key.
pub fn windows_of(j: &Value, at: i64, cx: &Ctx) -> Vec<Window> {
    fn walk(o: &Value, prefix: Option<&str>, at: i64, cx: &Ctx, out: &mut Vec<Window>) {
        match o {
            Value::Object(m) => {
                if m.get("used_percent").is_some_and(Value::is_number) {
                    out.push(window(o, at, prefix, cx));
                    return;
                }
                let own = m
                    .get("limit_name")
                    .or_else(|| m.get("metered_feature"))
                    .and_then(Value::as_str);
                for v in m.values() {
                    walk(v, own.or(prefix), at, cx, out);
                }
            }
            Value::Array(a) => {
                for v in a {
                    walk(v, prefix, at, cx, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    if let Some(r) = j.get("rate_limit") {
        walk(r, None, at, cx, &mut out);
    }
    if let Some(r) = j.get("code_review_rate_limit") {
        walk(r, Some("Code review"), at, cx, &mut out);
    }
    if let Some(r) = j.get("additional_rate_limits") {
        walk(r, None, at, cx, &mut out);
    }
    out
}

/// Limits from Codex's server (`backend-api/wham/usage`, read by Codex
/// itself); `None`: ask the logs.
pub fn online(cx: &Ctx) -> Result<Option<Limits>, Fail> {
    let (token, account) = match read_login(&cx.codex_dir()) {
        None => return Ok(None),
        Some(Login::ApiKey) => {
            return Ok(Some(Limits::note(
                cx.t("chiave API: nessun limite di piano", &[]),
            )))
        }
        Some(Login::Token { expires, .. }) if expires.is_some_and(|e| e < cx.now) => {
            // the session logs, until Codex renews it
            return Ok(None);
        }
        Some(Login::Token { token, account, .. }) => (token, account),
    };
    let auth = format!("Bearer {token}");
    let mut headers = vec![("Authorization", auth.as_str())];
    if let Some(a) = account.as_deref() {
        headers.push(("ChatGPT-Account-Id", a));
    }
    let j = cx.request("https://chatgpt.com/backend-api/wham/usage", &headers, None)?;
    let windows = windows_of(&j, cx.now, cx);
    if windows.is_empty() {
        return Ok(None);
    }
    Ok(Some(Limits {
        plan: j
            .get("plan_type")
            .and_then(Value::as_str)
            .map(str::to_owned),
        windows,
        source: Some(cx.t("server di OpenAI", &[])),
        ..Limits::default()
    }))
}

#[cfg(test)]
mod tests {
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
        let claims =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(br#"{"exp":2000000000}"#);
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
}
