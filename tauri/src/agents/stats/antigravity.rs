//! Antigravity: model quotas from Google's server with the login the kit
//! keeps, sessions from its conversation files, which have no token counts
//! (port of nabre/atollo `core/stats/antigravity.js`). The local language
//! server of a running Antigravity is not asked (DECISIONS.md D-61).

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use base64::Engine;
use serde_json::Value;

use super::util::{recent_files, summarize, to_time, Period, Usage};
use super::{Ctx, Fail, Limits, Login, Window};

/// The Google login in the saved credentials (`agents/keyring.rs`): a token in
/// one of the secrets (base64 of JSON, or of a JSON string of JSON).
pub fn read_login(blobs: &[String]) -> Option<Login> {
    blobs.iter().find_map(|b| {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b.trim())
            .ok()?;
        find_token(&String::from_utf8_lossy(&bytes))
    })
}

pub fn find_token(text: &str) -> Option<Login> {
    let mut j: Value = serde_json::from_str(text).ok()?;
    // go-keyring values are strings
    if let Value::String(s) = &j {
        j = serde_json::from_str(s).ok()?;
    }
    fn walk(o: &Value) -> Option<Login> {
        match o {
            Value::Object(m) => {
                let token = m
                    .get("access_token")
                    .or_else(|| m.get("accessToken"))
                    .and_then(Value::as_str);
                if let Some(token) = token {
                    let expires = ["expiry", "expiry_date", "expiresAt", "expires_at"]
                        .iter()
                        .find_map(|k| to_time(m.get(*k)));
                    return Some(Login::Token {
                        token: token.to_owned(),
                        account: None,
                        expires,
                        plan: None,
                    });
                }
                m.values().find_map(walk)
            }
            Value::Array(a) => a.iter().find_map(walk),
            _ => None,
        }
    }
    walk(&j)
}

/// Every named object with `quotaInfo` = one model quota. Internal models
/// (tab completion, `isInternal` or no name) are not quotas the user sees.
/// The answers are protobuf JSON, which leaves out zero values: a quota with
/// a reset time and no `remainingFraction` is used up, not unknown.
pub fn model_quotas(root: &Value) -> Vec<Window> {
    fn walk(o: &Value, seen: &mut BTreeSet<String>, out: &mut Vec<Window>) {
        match o {
            Value::Object(m) => {
                let name = m
                    .get("label")
                    .or_else(|| m.get("displayName"))
                    .and_then(Value::as_str);
                if let (Some(q), Some(name)) = (m.get("quotaInfo").filter(|q| q.is_object()), name)
                {
                    let left = q.get("remainingFraction").and_then(Value::as_f64);
                    let reset = q.get("resetTime").filter(|r| !r.is_null());
                    let internal = m.get("isInternal") == Some(&Value::Bool(true));
                    if !internal && (left.is_some() || reset.is_some()) && seen.insert(name.into())
                    {
                        out.push(Window::new(
                            name.to_owned(),
                            Some((1.0 - left.unwrap_or(0.0)) * 100.0),
                            to_time(reset),
                        ));
                    }
                }
                for (k, v) in m {
                    if k != "quotaInfo" {
                        walk(v, seen, out);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    walk(v, seen, out);
                }
            }
            _ => {}
        }
    }
    let (mut seen, mut out) = (BTreeSet::new(), Vec::new());
    walk(root, &mut seen, &mut out);
    // Most used first; equal ones by name, so the list does not jump
    out.sort_by(|a, b| {
        b.percent
            .unwrap_or(0.0)
            .total_cmp(&a.percent.unwrap_or(0.0))
            .then_with(|| a.label.cmp(&b.label))
    });
    out
}

/// The models offered in Antigravity's model menu (`agentModelSorts`),
/// when the answer lists them; otherwise every model of the answer.
pub fn offered_quotas(j: &Value) -> Vec<Window> {
    let offered: BTreeSet<&str> = j
        .get("agentModelSorts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|s| s.get("groups").and_then(Value::as_array))
        .flatten()
        .filter_map(|g| g.get("modelIds").and_then(Value::as_array))
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    match j.get("models").and_then(Value::as_object) {
        Some(models) if !offered.is_empty() => model_quotas(&Value::Array(
            models
                .iter()
                .filter(|(id, _)| offered.contains(id.as_str()))
                .map(|(_, m)| m.clone())
                .collect(),
        )),
        _ => model_quotas(j),
    }
}

pub fn limits(cx: &Ctx) -> Result<Limits, Fail> {
    if cx.has_env("GEMINI_API_KEY") {
        return Ok(Limits::note(
            cx.t("chiave API Gemini: nessun limite di piano", &[]),
        ));
    }
    let Some(Login::Token { token, expires, .. }) = read_login(&cx.antigravity_blobs()) else {
        return Ok(Limits::error(
            cx.t("accesso Google non ancora fatto: apri Antigravity", &[]),
        ));
    };
    if expires.is_some_and(|e| e < cx.now) {
        return Ok(Limits::error(cx.t(
            "accesso scaduto: le quote tornano dopo il prossimo avvio di Antigravity",
            &[],
        )));
    }
    let auth = format!("Bearer {token}");
    // The server answers only Antigravity's own User-Agent (403 otherwise)
    let j = cx.request(
        "https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels",
        &[
            ("Authorization", &auth),
            ("Content-Type", "application/json"),
            ("User-Agent", "antigravity"),
        ],
        Some(&serde_json::json!({})),
    )?;
    let windows = offered_quotas(&j);
    if windows.is_empty() {
        return Ok(Limits::error(
            cx.t("il server non ha restituito quote", &[]),
        ));
    }
    Ok(Limits {
        windows,
        source: Some(cx.t("server Google", &[])),
        ..Limits::default()
    })
}

/// Antigravity keeps each conversation in
/// `.gemini\antigravity*\conversations\<id>`: a protobuf file (`.pb`) in
/// older versions, an SQLite database (`.db`, with its `-wal` journal) in
/// newer ones. A conversation changed in the period = a session.
pub fn usage(gemini: &Path, p: &Period) -> Usage {
    fn stem(name: &str) -> Option<&str> {
        [".db-wal", ".pb", ".db"]
            .iter()
            .find_map(|ext| name.strip_suffix(ext))
            .filter(|s| !s.is_empty())
    }
    let files = recent_files(gemini, &|n| stem(n).is_some(), p.since(), Some(u64::MAX));
    let mut last: HashMap<std::path::PathBuf, i64> = HashMap::new();
    for f in files {
        let Some(dir) = f.file.parent() else { continue };
        if dir.file_name().is_none_or(|n| n != "conversations") {
            continue;
        }
        let name = f
            .file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let Some(id) = stem(&name) else { continue };
        let at = last.entry(dir.join(id)).or_default();
        *at = (*at).max(f.modified);
    }
    let sessions: Vec<i64> = last.into_values().collect();
    summarize(&[], &sessions, false, p)
}

#[cfg(test)]
#[path = "../../../../test/rust/agents/stats/antigravity.rs"]
mod tests;
