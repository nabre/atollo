use super::*;

#[test]
fn session_members_by_session_id() {
    let me = std::process::id();
    // (pid, session id): 100 leads, 101 and 102 are its jobs, 200 another
    // session, 300 a job that left with `setsid`.
    let procs = [(100, 100), (101, 100), (102, 100), (200, 200), (300, 300)];
    assert_eq!(in_session(&procs, 100, true), [100, 101, 102]);
    // The leader reaped: its jobs only.
    let after = [(101, 100), (102, 100), (200, 200)];
    assert_eq!(in_session(&after, 100, false), [101, 102]);
    // Its pid now another program's (so its session is gone): none.
    let reused = [(100, 7), (200, 200)];
    assert!(in_session(&reused, 100, false).is_empty());
    let reused_leader = [(100, 100), (101, 100)];
    assert!(in_session(&reused_leader, 100, false).is_empty());
    // Never the app, never pid 0.
    assert!(in_session(&[(me, 100)], 100, true).is_empty());
    assert!(in_session(&[(0, 0), (5, 0)], 0, true).is_empty());
}

#[test]
fn sessions_get_a_utf8_locale_when_the_kit_env_has_none() {
    use crate::kit::Os;
    // Started from Finder or launchd: nothing set.
    assert_eq!(
        utf8_locale(Os::Darwin, None, None, None),
        Some(("LANG", "en_US.UTF-8"))
    );
    assert_eq!(
        utf8_locale(Os::Linux, None, Some(""), Some("")),
        Some(("LANG", "C.UTF-8"))
    );
    // The user's UTF-8 locale stays, in any spelling.
    assert_eq!(
        utf8_locale(Os::Darwin, None, None, Some("it_IT.UTF-8")),
        None
    );
    assert_eq!(utf8_locale(Os::Linux, None, None, Some("de_DE.utf8")), None);
    assert_eq!(
        utf8_locale(Os::Darwin, None, Some("UTF-8"), Some("C")),
        None
    );
    // Another charset: only the characters' category changes.
    assert_eq!(
        utf8_locale(Os::Linux, None, None, Some("fr_FR.ISO-8859-1")),
        Some(("LC_CTYPE", "C.UTF-8"))
    );
    assert_eq!(
        utf8_locale(Os::Darwin, None, Some("C"), Some("it_IT.UTF-8")),
        Some(("LC_CTYPE", "en_US.UTF-8"))
    );
    // An LC_ALL of the user's is never overridden.
    assert_eq!(utf8_locale(Os::Linux, Some("C"), None, None), None);
}
