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
