use super::*;

/// One test: the creation is one for the whole app.
#[test]
fn a_creation_from_start_to_end() {
    let env = vec![EnvVar {
        key: "APP_NAME".into(),
        value: "{name}".into(),
    }];
    let id = start("laravel", &["react".into()], "app", &env, 2);
    let c = snapshot();
    assert!(c.running);
    assert_eq!((c.id, c.name.as_str(), c.total, c.lines), (id, "app", 2, 0));
    assert_eq!(c.env, env);

    assert_eq!(record(id, 0, None), 0, "a step without a line");
    assert_eq!(record(id, 1, Some("> composer create-project".into())), 1);
    for i in 0..MAX_LOG {
        record(id, 1, Some(format!("line {i}")));
    }
    let c = snapshot();
    assert_eq!(c.step, 1);
    assert_eq!(c.lines, MAX_LOG + 1);
    assert_eq!(c.log.len(), MAX_LOG, "the oldest lines go");
    assert_eq!(c.log[0], "line 0");

    // An older creation's late line or end changes nothing.
    assert_eq!(record(id - 1, 2, Some("old".into())), MAX_LOG + 1);
    assert!(finish(id - 1, Ok("old".into())).running);

    let end = finish(id, Err("Comando non riuscito".into()));
    assert!(!end.running);
    assert_eq!(end.error, Some("Comando non riuscito".into()));
    assert_eq!(end.created, None);
    assert_eq!(snapshot(), end);

    let next = start("vite", &[], "web", &[], 1);
    assert!(next > id);
    let c = snapshot();
    assert_eq!((c.lines, c.error.clone(), c.log.len()), (0, None, 0));
    assert_eq!(
        finish(next, Ok("web".into())).created.as_deref(),
        Some("web")
    );
}
