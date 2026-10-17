use super::*;
use crate::kit::tests::TempDir;
use serde_json::json;
use std::fs;

#[test]
fn logins_in_the_saved_secrets() {
    let b64 = |s: &str| base64::engine::general_purpose::STANDARD.encode(s);
    let inner = r#"{"token":{"access_token":"ya29","expiry":"2026-10-14T10:00:00Z"}}"#;
    let wrapped = serde_json::to_string(inner).unwrap();
    for secret in [inner.to_owned(), wrapped] {
        let Some(Login::Token { token, expires, .. }) =
            read_login(&["not base64!".into(), b64("nothing"), b64(&secret)])
        else {
            panic!("no login in {secret}");
        };
        assert_eq!(token, "ya29");
        assert_eq!(
            expires,
            super::super::util::parse_iso("2026-10-14T10:00:00Z")
        );
    }
    assert!(read_login(&[b64(r#"{"refresh_token":"x"}"#)]).is_none());
}

#[test]
fn quotas_of_the_offered_models() {
    let j = json!({
        "agentModelSorts": [{"groups": [{"modelIds": ["pro", "flash"]}]}],
        "models": {
            "pro": {"displayName": "Gemini Pro", "quotaInfo": {"remainingFraction": 0.25, "resetTime": "2026-10-14T18:00:00Z"}},
            "flash": {"displayName": "Gemini Flash", "quotaInfo": {"resetTime": "2026-10-14T18:00:00Z"}},
            "tab": {"displayName": "Tab", "isInternal": true, "quotaInfo": {"remainingFraction": 0.5}},
            "hidden": {"displayName": "Not offered", "quotaInfo": {"remainingFraction": 1}}
        }
    });
    let w = offered_quotas(&j);
    let got: Vec<(&str, Option<f64>)> = w.iter().map(|w| (w.label.as_str(), w.percent)).collect();
    assert_eq!(
        got,
        [("Gemini Flash", Some(100.0)), ("Gemini Pro", Some(75.0))]
    );
    // no menu in the answer: every visible model
    let mut all = j.clone();
    all.as_object_mut().unwrap().remove("agentModelSorts");
    assert_eq!(offered_quotas(&all).len(), 3);
}

#[test]
#[cfg_attr(
    not(windows),
    ignore = "conversation folder created with a backslash path"
)]
fn conversations_are_sessions() {
    let tmp = TempDir::new("stats-agy");
    let conv = tmp.0.join(r"antigravity\conversations");
    fs::create_dir_all(&conv).unwrap();
    for f in ["a.db", "a.db-wal", "b.pb", "c.txt"] {
        fs::write(conv.join(f), b"x").unwrap();
    }
    fs::write(tmp.0.join("d.pb"), b"x").unwrap();
    let u = usage(&tmp.0, &Period::now());
    assert!(!u.tokens);
    assert_eq!(u.sessions_week, 2);
    assert!(u.last.is_some());
}
