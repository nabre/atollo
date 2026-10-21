//! Claude Code: plan limits from Anthropic's server, usage from its sessions.

use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

use serde_json::Value;

use super::util::{
    clamp_percent, json_lines, recent_files, summarize, to_time, FileCache, Period, Record, Tokens,
    Usage,
};
use super::{read_json, Credits, Ctx, Fail, Limits, Login, Window};

/// The login of `.claude\.credentials.json` (only read, never refreshed:
/// a refresh would replace the token Claude Code keeps).
pub fn read_login(dir: &Path) -> Option<Login> {
    let j = read_json(&dir.join(".credentials.json"))?;
    let o = j.get("claudeAiOauth")?;
    let token = o.get("accessToken")?.as_str().filter(|t| !t.is_empty())?;
    Some(Login::Token {
        token: token.to_owned(),
        account: None,
        expires: to_time(o.get("expiresAt")),
        plan: o
            .get("subscriptionType")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

fn number(v: Option<&Value>) -> Option<f64> {
    v?.as_f64()
}

/// The plan limits: the server's `limits` list (session, weekly_all,
/// weekly_scoped with the model or surface it applies to), else the
/// documented windows of older answers.
pub fn windows(j: &Value, cx: &Ctx) -> Vec<Window> {
    let five = || cx.t("{0} ore", &["5"]);
    let week = || cx.t("Settimana", &[]);
    let list: Vec<&Value> = j
        .get("limits")
        .and_then(Value::as_array)
        .map(|l| {
            l.iter()
                .filter(|l| l.get("percent").is_some_and(Value::is_number))
                .collect()
        })
        .unwrap_or_default();
    if !list.is_empty() {
        let name = |v: Option<&Value>| -> Option<String> {
            match v? {
                Value::Object(o) => o
                    .get("display_name")
                    .or_else(|| o.get("id"))
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                Value::String(s) => Some(s.clone()),
                _ => None,
            }
        };
        return list
            .into_iter()
            .map(|l| {
                let str_of = |k: &str| l.get(k).and_then(Value::as_str).unwrap_or("");
                let (kind, group) = (str_of("kind"), str_of("group"));
                let scope = l.get("scope");
                let scope = [
                    name(scope.and_then(|s| s.get("model"))),
                    name(scope.and_then(|s| s.get("surface"))),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" · ");
                let base = if kind == "session" || group == "session" {
                    five()
                } else if kind.starts_with("weekly") || group == "weekly" {
                    week()
                } else if !kind.is_empty() || !group.is_empty() {
                    (if kind.is_empty() { group } else { kind }).replace('_', " ")
                } else {
                    cx.t("Limite", &[])
                };
                // The same windows with one decimal, when the answer has them
                let precise = match kind {
                    "session" => number(j.pointer("/five_hour/utilization")),
                    "weekly_all" => number(j.pointer("/seven_day/utilization")),
                    _ => None,
                };
                let label = if scope.is_empty() {
                    base
                } else {
                    format!("{base} · {scope}")
                };
                Window::new(
                    label,
                    precise.or(number(l.get("percent"))),
                    to_time(l.get("resets_at")),
                )
            })
            .collect();
    }
    let older: [(&str, String); 4] = [
        ("five_hour", five()),
        ("seven_day", week()),
        ("seven_day_opus", format!("{} · Opus", week())),
        ("seven_day_sonnet", format!("{} · Sonnet", week())),
    ];
    older
        .into_iter()
        .filter_map(|(key, label)| {
            let w = j.get(key)?;
            let p = number(w.get("utilization"))?;
            Some(Window::new(label, Some(p), to_time(w.get("resets_at"))))
        })
        .collect()
}

/// Paid credits used beyond the plan limits (monthly cap), if enabled.
pub fn credits(j: &Value) -> Option<Credits> {
    let money = |m: Option<&Value>| -> Option<f64> {
        let m = m?;
        let amount = m.get("amount_minor")?.as_f64()?;
        let exp = m.get("exponent").and_then(Value::as_i64).unwrap_or(0);
        Some(amount / 10f64.powi(exp as i32))
    };
    let make = |used: f64, cap: Option<f64>, currency: Option<&str>, percent: Option<f64>| {
        let percent = match cap {
            Some(c) if c > 0.0 => clamp_percent(used / c * 100.0),
            _ => percent.and_then(clamp_percent),
        };
        Credits::new(used, cap, currency.unwrap_or("USD"), percent)
    };
    fn currency(m: Option<&Value>) -> Option<&str> {
        m?.get("currency")?.as_str()
    }
    if let Some(s) = j
        .get("spend")
        .filter(|s| s.get("enabled") == Some(&Value::Bool(true)))
    {
        if let Some(used) = money(s.get("used")) {
            let cap = money(s.get("limit")).or_else(|| {
                let c = s.get("cap")?;
                money(c.get("money")).or_else(|| money(c.get("credits")))
            });
            return Some(make(
                used,
                cap,
                currency(s.get("used")).or(currency(s.get("limit"))),
                number(s.get("percent")),
            ));
        }
    }
    let e = j.get("extra_usage")?;
    if e.get("is_enabled") != Some(&Value::Bool(true)) {
        return None;
    }
    let used = e.get("used_credits")?.as_f64()?;
    let f = 10f64.powi(e.get("decimal_places").and_then(Value::as_i64).unwrap_or(0) as i32);
    Some(make(
        used / f,
        number(e.get("monthly_limit")).map(|m| m / f),
        e.get("currency").and_then(Value::as_str),
        number(e.get("utilization")),
    ))
}

pub fn limits(cx: &Ctx) -> Result<Limits, Fail> {
    let Some(login) = read_login(&cx.claude_dir()) else {
        return Ok(Limits::error(
            cx.t("accesso non ancora fatto: apri Claude Code", &[]),
        ));
    };
    let Login::Token {
        token,
        expires,
        plan,
        ..
    } = login
    else {
        return Ok(Limits::note(
            cx.t("chiave API: nessun limite di piano", &[]),
        ));
    };
    if expires.is_some_and(|e| e < cx.now) {
        return Ok(Limits::error(
            cx.t("accesso scaduto: apri Claude Code per rinnovarlo", &[]),
        ));
    }
    let auth = format!("Bearer {token}");
    let j = cx.request(
        "https://api.anthropic.com/api/oauth/usage",
        &[
            ("Authorization", &auth),
            ("anthropic-beta", "oauth-2025-04-20"),
        ],
        None,
    )?;
    Ok(Limits {
        plan,
        windows: windows(&j, cx),
        credits: credits(&j),
        source: Some(cx.t("server di Anthropic", &[])),
        ..Limits::default()
    })
}

/// A file of Claude's subagents (Task tool): its tokens count, not as a session.
fn is_subagent(file: &Path) -> bool {
    file.components().any(|c| c.as_os_str() == "subagents")
        || file
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("agent-"))
}

struct Parsed {
    records: Vec<(String, Record)>,
    last: Option<i64>,
}

static FILES: LazyLock<FileCache<Parsed>> = LazyLock::new(FileCache::new);

fn parse(text: &str, file: &Path) -> Parsed {
    let mut out = Parsed {
        records: Vec::new(),
        last: None,
    };
    json_lines(text, |o, line| {
        let Some(m) = o.get("message") else { return };
        let Some(u) = m.get("usage") else { return };
        if o.get("type").and_then(Value::as_str) != Some("assistant") {
            return;
        }
        let time = to_time(o.get("timestamp"));
        if let Some(t) = time {
            out.last = Some(out.last.map_or(t, |l| l.max(t)));
        }
        let n = |k: &str| u.get(k).and_then(Value::as_u64).unwrap_or(0);
        // The same reply is written once per content block, with the same id
        let (mid, rid) = (
            m.get("id").and_then(Value::as_str).unwrap_or(""),
            o.get("requestId").and_then(Value::as_str).unwrap_or(""),
        );
        let id = if mid.is_empty() && rid.is_empty() {
            format!("{}:{line}", file.display())
        } else {
            format!("{mid}|{rid}")
        };
        out.records.push((
            id,
            Record {
                time,
                model: m.get("model").and_then(Value::as_str).map(str::to_owned),
                tokens: Tokens::new(
                    n("input_tokens") + n("cache_creation_input_tokens"),
                    n("cache_read_input_tokens"),
                    n("output_tokens"),
                ),
            },
        ));
    });
    out
}

pub fn usage(dir: &Path, p: &Period) -> Usage {
    let files = recent_files(
        &dir.join("projects"),
        &|n| n.ends_with(".jsonl"),
        p.since(),
        None,
    );
    FILES.keep_only(&files.iter().collect::<Vec<_>>());
    let mut by_id: HashMap<String, Record> = HashMap::new();
    let mut sessions = Vec::new();
    for f in &files {
        let parsed = FILES.get(f, |text| parse(text, &f.file));
        // A reply counted once, also when copied into a resumed session: the
        // largest count of its copies (older versions wrote partial counts first)
        for (id, x) in &parsed.records {
            match by_id.get_mut(id) {
                None => {
                    by_id.insert(id.clone(), x.clone());
                }
                Some(prev) => {
                    prev.tokens = Tokens::new(
                        prev.tokens.input.max(x.tokens.input),
                        prev.tokens.cached.max(x.tokens.cached),
                        prev.tokens.output.max(x.tokens.output),
                    );
                    prev.time = match (prev.time, x.time) {
                        (Some(a), Some(b)) => Some(a.min(b)),
                        (a, b) => a.or(b),
                    };
                    if prev.model.is_none() {
                        prev.model.clone_from(&x.model);
                    }
                }
            }
        }
        if let Some(last) = parsed.last.filter(|_| !is_subagent(&f.file)) {
            sessions.push(last);
        }
    }
    let records: Vec<Record> = by_id.into_values().collect();
    summarize(&records, &sessions, true, p)
}

#[cfg(test)]
#[path = "../../../../test/rust/agents/stats/claude.rs"]
mod tests;
