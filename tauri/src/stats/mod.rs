//! Usage statistics of the kit's agents (port of nabre/dev-kit
//! `core/stats/*.js`), counted the same way for all three:
//!
//! - limits: every plan limit as "% used" (0-100) with the time it resets:
//!   the 5-hour and weekly windows of Claude and Codex, the per-model quotas
//!   of Antigravity. Same thresholds for all (`WARN`, `CRITICAL`); "peak" =
//!   the highest one. Paid extra credits (Claude) are apart: not a plan
//!   limit, not in the peak.
//! - usage: tokens of today and of the last 7 local days, today included
//!   (new input, cached input, output), per day, sessions, last activity,
//!   models.
//!
//! Sources (all inside the kit):
//! - Claude: Anthropic's server (`/api/oauth/usage`, the data of `/usage`)
//!   with the login of `data\home\.claude\.credentials.json`; sessions in
//!   `.claude\projects\**\*.jsonl`.
//! - Codex: OpenAI's server (`backend-api/wham/usage`, the data of
//!   `/status`) with the login of `.codex\auth.json`, else the last
//!   `token_count` event of its sessions; sessions in `.codex\sessions` and
//!   `archived_sessions`.
//! - Antigravity: Google's server (`v1internal:fetchAvailableModels`) with
//!   the login the kit keeps in `data\antigravity\keyring.json`; sessions =
//!   its conversation files in `.gemini` (no token counts).
//!
//! The servers' interfaces are undocumented and may change after an update
//! of an agent (DECISIONS.md D-61). Logins are only read, never refreshed: a
//! refresh would replace the token the agent itself keeps. Tokens go only to
//! their provider's server and never to the log.

mod antigravity;
mod claude;
mod codex;
pub mod util;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use crate::catalog::Catalog;
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::net::Net;
use util::{clamp_percent, level, now_ms, Period, Usage, CRITICAL, WARN};

/// Servers asked at most every `ONLINE_EVERY`, at most every
/// `FORCED_EVERY` with "Refresh"; after a 429 (too many requests) nobody asks
/// until the server allows it (`Retry-After`, otherwise a wait that doubles
/// at every 429 in a row, from `BACKOFF_MIN` to `BACKOFF_MAX`).
const ONLINE_EVERY: i64 = 2 * 60 * 1000;
const FORCED_EVERY: i64 = 20 * 1000;
const BACKOFF_MIN: i64 = 2 * 60 * 1000;
const BACKOFF_MAX: i64 = 30 * 60 * 1000;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// One limit: always "% used" of a window or quota.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    pub label: String,
    pub percent: Option<f64>,
    pub resets_at: Option<i64>,
    pub level: &'static str,
}

impl Window {
    pub fn new(label: String, percent: Option<f64>, resets_at: Option<i64>) -> Window {
        let percent = percent.and_then(clamp_percent);
        Window {
            label,
            percent,
            resets_at,
            level: level(percent),
        }
    }
}

/// Paid credits used beyond the plan limits (Claude).
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Credits {
    pub used: f64,
    pub limit: Option<f64>,
    pub currency: String,
    pub percent: Option<f64>,
    pub level: &'static str,
}

impl Credits {
    pub fn new(used: f64, limit: Option<f64>, currency: &str, percent: Option<f64>) -> Credits {
        Credits {
            used,
            limit,
            currency: currency.to_owned(),
            percent,
            level: level(percent),
        }
    }
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    pub plan: Option<String>,
    pub windows: Vec<Window>,
    pub credits: Option<Credits>,
    /// Where they come from ("Anthropic's server").
    pub source: Option<String>,
    /// No plan limits, and why (an API key).
    pub note: Option<String>,
    pub error: Option<String>,
    /// When they were read.
    pub updated: Option<i64>,
    /// After a 429: the last limits read, asked again at `retry_at`.
    pub limited: bool,
    pub retry_at: Option<i64>,
}

impl Limits {
    pub fn error(text: String) -> Limits {
        Limits {
            error: Some(text),
            ..Limits::default()
        }
    }
    pub fn note(text: String) -> Limits {
        Limits {
            note: Some(text),
            ..Limits::default()
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStats {
    pub id: &'static str,
    pub name: String,
    /// "none", "expired", "apikey", "ok".
    pub account: &'static str,
    pub limits: Limits,
    pub usage: Usage,
    pub peak: Option<f64>,
    pub peak_label: Option<String>,
    pub level: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub updated: i64,
    pub warning: f64,
    pub critical: f64,
    pub agents: Vec<AgentStats>,
}

/// A login read from the kit.
pub enum Login {
    ApiKey,
    Token {
        token: String,
        account: Option<String>,
        expires: Option<i64>,
        plan: Option<String>,
    },
}

/// A server that did not give the limits.
#[derive(Debug, Clone)]
pub struct Fail {
    pub message: String,
    pub status: Option<u16>,
    pub retry_after: i64,
}

/// What a reading needs: the kit, its environment, the language, the
/// network (`None` in the tests: no server is asked).
pub struct Ctx {
    home: PathBuf,
    data: PathBuf,
    env: HashMap<String, String>,
    lang: &'static str,
    net: Option<Net>,
    pub now: i64,
}

impl Ctx {
    pub fn new(kit: &Kit, env: &KitEnv, lang: &'static str, net: Option<Net>) -> Ctx {
        let keys = ["ANTHROPIC_API_KEY", "GEMINI_API_KEY"];
        Ctx {
            home: kit.home(),
            data: kit.data(),
            env: keys
                .iter()
                .filter_map(|k| Some(((*k).to_owned(), env.get(k)?.to_owned())))
                .collect(),
            lang,
            net,
            now: now_ms(),
        }
    }

    #[cfg(test)]
    pub fn for_tests(lang: &'static str) -> Ctx {
        Ctx {
            home: PathBuf::new(),
            data: PathBuf::new(),
            env: HashMap::new(),
            lang,
            net: None,
            now: now_ms(),
        }
    }

    pub fn t(&self, it_source: &str, args: &[&str]) -> String {
        crate::i18n::t(self.lang, it_source, args)
    }

    pub fn has_env(&self, name: &str) -> bool {
        self.env.get(name).is_some_and(|v| !v.is_empty())
    }

    pub fn claude_dir(&self) -> PathBuf {
        self.home.join(".claude")
    }

    pub fn codex_dir(&self) -> PathBuf {
        self.home.join(".codex")
    }

    pub fn antigravity_blobs(&self) -> Vec<String> {
        crate::keyring::saved_secrets(&self.data.join("antigravity"))
    }

    /// A JSON request to an agent's server, errors in the user's language.
    pub fn request(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: Option<&Value>,
    ) -> Result<Value, Fail> {
        let Some(net) = &self.net else {
            return Err(Fail {
                message: self.t("Limiti non disponibili", &[]),
                status: None,
                retry_after: 0,
            });
        };
        net.api_json(url, headers, body, REQUEST_TIMEOUT)
            .map_err(|e| {
                let message = match e.status {
                    None => {
                        crate::log::warn(&format!("stats {}: {}", host_of(url), e.detail));
                        self.t("il server non risponde", &[])
                    }
                    Some(401) => self.t(
                        "accesso scaduto o non valido: apri l'agente per rinnovarlo",
                        &[],
                    ),
                    Some(403) => self.t("il server ha rifiutato la richiesta (403)", &[]),
                    Some(429) => self.t("troppe richieste al server (429)", &[]),
                    Some(s) => self.t("il server ha risposto {0}", &[&s.to_string()]),
                };
                Fail {
                    message,
                    status: e.status,
                    retry_after: util::retry_after(e.retry_after.as_deref(), now_ms()),
                }
            })
    }
}

fn host_of(url: &str) -> &str {
    url.split('/').nth(2).unwrap_or(url)
}

pub fn read_json(path: &Path) -> Option<Value> {
    crate::fsx::read_json(path)
}

/// The last reading of one server.
#[derive(Clone, Default)]
struct Online {
    at: i64,
    value: Option<Limits>,
    /// The last limits read without errors.
    good: Option<Limits>,
    /// 429s in a row.
    failures: u32,
    blocked_until: i64,
}

static ONLINE: LazyLock<Mutex<HashMap<&'static str, Online>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn online_lock() -> std::sync::MutexGuard<'static, HashMap<&'static str, Online>> {
    ONLINE.lock().unwrap_or_else(|e| e.into_inner())
}

/// The limits of `id` from its server, asked again only when the last
/// reading is old enough and the server allows it.
fn cached_online(
    id: &'static str,
    force: bool,
    now: i64,
    fetch: impl FnOnce() -> Result<Limits, Fail>,
) -> Limits {
    let last = {
        let mut all = online_lock();
        let c = all.entry(id).or_default();
        if let Some(value) = &c.value {
            // After a 429 not even "Refresh" asks before the server allows it
            if c.blocked_until > now || now - c.at < if force { FORCED_EVERY } else { ONLINE_EVERY }
            {
                return value.clone();
            }
        }
        // Claimed before asking: a reading at the same time shows the last one
        let last = c.clone();
        c.at = now;
        last
    };
    let result = fetch();
    let at = now;
    let mut entry = Online {
        at,
        good: last.good.clone(),
        ..Online::default()
    };
    match result {
        Err(e) if e.status == Some(429) => {
            entry.failures = last.failures + 1;
            let wait = (BACKOFF_MIN << (entry.failures - 1).min(8)).min(BACKOFF_MAX);
            entry.blocked_until = at + e.retry_after.max(wait);
            // The last limits read stay, with the time of the next request
            let mut value = entry
                .good
                .clone()
                .unwrap_or_else(|| Limits::error(e.message));
            value.limited = true;
            value.retry_at = Some(entry.blocked_until);
            entry.value = Some(value);
        }
        Err(e) => {
            entry.value = Some(Limits {
                updated: Some(at),
                ..Limits::error(e.message)
            });
        }
        Ok(mut value) => {
            value.updated = value.updated.or(Some(at));
            if value.error.is_none() {
                entry.good = Some(value.clone());
            }
            entry.value = Some(value);
        }
    }
    let value = entry.value.clone().unwrap_or_default();
    online_lock().insert(id, entry);
    value
}

fn account(login: Option<&Login>, now: i64) -> &'static str {
    match login {
        None => "none",
        Some(Login::ApiKey) => "apikey",
        Some(Login::Token { expires, .. }) if expires.is_some_and(|e| e < now) => "expired",
        Some(Login::Token { .. }) => "ok",
    }
}

/// Peak and level of an agent: its most used plan limit (credits apart).
fn finish(mut a: AgentStats, cx: &Ctx) -> AgentStats {
    if a.account == "none" && a.limits.windows.is_empty() {
        a.limits = Limits::error(cx.t(
            "accesso non fatto: apri {0} per vedere limiti e consumi",
            &[&a.name],
        ));
    }
    let top = a
        .limits
        .windows
        .iter()
        .filter(|w| w.percent.is_some())
        .max_by(|x, y| {
            x.percent
                .unwrap_or(0.0)
                .total_cmp(&y.percent.unwrap_or(0.0))
        });
    // Every limit at 0%: no limit is "the most used"
    a.peak = top.and_then(|w| w.percent);
    a.peak_label = top
        .filter(|w| w.percent > Some(0.0))
        .map(|w| w.label.clone());
    a.level = level(a.peak);
    a
}

/// The statistics of the agents installed in the kit. `force`: "Refresh"
/// (the servers are asked sooner, never before a 429 wait ends).
pub fn collect(kit: &Kit, env: &KitEnv, lang: &'static str, force: bool) -> Report {
    let net = Net::new(&crate::settings::Settings::load(kit).settings, None).ok();
    let cx = Ctx::new(kit, env, lang, net);
    let catalog = Catalog::get();
    let installed = |id: &str| catalog.find(id).is_some_and(|c| c.is_installed(kit));
    let period = Period::now();
    let name = |id: &str| catalog.find(id).map_or(id.to_owned(), |c| c.name.clone());
    let blank = |id: &'static str| AgentStats {
        id,
        name: name(id),
        account: "none",
        limits: Limits::default(),
        usage: Usage::default(),
        peak: None,
        peak_label: None,
        level: "none",
    };
    let (cx, period) = (&cx, &period);
    let agents = std::thread::scope(|s| {
        let claude = installed("claude").then(|| {
            s.spawn(move || {
                let limits = cached_online("claude", force, cx.now, || claude::limits(cx));
                let login = claude::read_login(&cx.claude_dir(), cx.has_env("ANTHROPIC_API_KEY"));
                AgentStats {
                    account: account(login.as_ref(), cx.now),
                    limits,
                    usage: claude::usage(&cx.claude_dir(), period),
                    ..blank("claude")
                }
            })
        });
        let codex = installed("codex").then(|| {
            s.spawn(move || {
                let (usage, from_logs) = codex::from_logs(&cx.codex_dir(), period, cx);
                // No login for the server: an empty answer, and the logs answer
                let online = cached_online("codex", force, cx.now, || {
                    Ok(codex::online(cx)?.unwrap_or_default())
                });
                // Its server when the login allows it, the logs otherwise
                let answered = !online.windows.is_empty() || online.note.is_some();
                let limits = if online.error.is_none() && answered {
                    online
                } else {
                    from_logs
                };
                AgentStats {
                    account: account(codex::read_login(&cx.codex_dir()).as_ref(), cx.now),
                    limits,
                    usage,
                    ..blank("codex")
                }
            })
        });
        let agy = installed("antigravity").then(|| {
            s.spawn(move || {
                let limits =
                    cached_online("antigravity", force, cx.now, || antigravity::limits(cx));
                let login = if cx.has_env("GEMINI_API_KEY") {
                    Some(Login::ApiKey)
                } else {
                    antigravity::read_login(&cx.antigravity_blobs())
                };
                AgentStats {
                    account: account(login.as_ref(), cx.now),
                    limits,
                    usage: antigravity::usage(&cx.home.join(".gemini"), period),
                    ..blank("antigravity")
                }
            })
        });
        [claude, codex, agy]
            .into_iter()
            .flatten()
            .filter_map(|h| h.join().ok())
            .collect::<Vec<_>>()
    });
    Report {
        updated: now_ms(),
        warning: WARN,
        critical: CRITICAL,
        agents: agents.into_iter().map(|a| finish(a, cx)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fail(status: u16, retry_after: i64) -> Fail {
        Fail {
            message: format!("HTTP {status}"),
            status: Some(status),
            retry_after,
        }
    }

    #[test]
    fn servers_are_asked_sparingly() {
        let id = "test-sparingly";
        let ok = || {
            Ok(Limits {
                windows: vec![Window::new("w".into(), Some(10.0), None)],
                ..Limits::default()
            })
        };
        let mut calls = 0;
        let now = now_ms();
        let first = cached_online(id, false, now, || {
            calls += 1;
            ok()
        });
        assert_eq!(first.windows.len(), 1);
        // soon after: the last reading, even with "Refresh"
        cached_online(id, true, now + 1000, || {
            calls += 1;
            ok()
        });
        assert_eq!(calls, 1);
        // "Refresh" after FORCED_EVERY asks; a 429 keeps the last limits
        let limited = cached_online(id, true, now + FORCED_EVERY + 1, || {
            calls += 1;
            Err(fail(429, 0))
        });
        assert_eq!(calls, 2);
        assert!(limited.limited && limited.windows.len() == 1);
        let retry = limited.retry_at.unwrap();
        assert!(retry >= now + BACKOFF_MIN, "waits at least BACKOFF_MIN");
        // during the wait nobody asks, not even "Refresh"
        cached_online(id, true, now + FORCED_EVERY * 3, || {
            calls += 1;
            ok()
        });
        assert_eq!(calls, 2);
    }

    #[test]
    fn errors_are_not_kept_as_limits() {
        let id = "test-errors";
        let now = now_ms();
        let e = cached_online(id, false, now, || Err(fail(500, 0)));
        assert!(e.error.is_some() && !e.limited);
        // a 429 without an earlier good reading: the error, with the retry time
        let l = cached_online(id, false, now + ONLINE_EVERY + 1, || {
            Err(fail(429, 3_600_000))
        });
        assert!(l.limited && l.error.is_some());
        assert!(l.retry_at.unwrap() >= now + 3_600_000);
    }

    #[test]
    fn peak_is_the_most_used_limit() {
        let cx = Ctx::for_tests("it");
        let mut a = AgentStats {
            id: "claude",
            name: "Claude Code".into(),
            account: "ok",
            limits: Limits {
                windows: vec![
                    Window::new("5 ore".into(), Some(20.0), None),
                    Window::new("Settimana".into(), Some(75.0), None),
                ],
                credits: Some(Credits::new(9.0, Some(10.0), "USD", Some(90.0))),
                ..Limits::default()
            },
            usage: Usage::default(),
            peak: None,
            peak_label: None,
            level: "none",
        };
        let f = finish(a.clone(), &cx);
        assert_eq!(
            (f.peak, f.peak_label.as_deref(), f.level),
            (Some(75.0), Some("Settimana"), "warning")
        );
        a.limits.windows = vec![Window::new("5 ore".into(), Some(0.0), None)];
        let f = finish(a.clone(), &cx);
        assert_eq!((f.peak, f.peak_label, f.level), (Some(0.0), None, "ok"));
        a.account = "none";
        a.limits = Limits::default();
        let f = finish(a, &cx);
        assert_eq!(
            f.limits.error.as_deref(),
            Some("accesso non fatto: apri Claude Code per vedere limiti e consumi")
        );
        assert_eq!(f.level, "none");
    }
}
