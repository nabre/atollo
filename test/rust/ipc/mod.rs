use super::*;
use crate::test_support::prepared_kit;
use std::fs;

#[test]
fn project_names_cannot_escape() {
    let (_tmp, kit) = prepared_kit("cmd-proj");
    fs::create_dir_all(kit.projects().join("app")).unwrap();
    assert_eq!(project_dir(&kit, None, "it").unwrap(), kit.projects());
    assert_eq!(
        project_dir(&kit, Some("app"), "it").unwrap(),
        kit.projects().join("app")
    );
    for bad in ["..", r"..\data", "a/b", "C:\\x", "missing", "con"] {
        assert!(project_dir(&kit, Some(bad), "it").is_err(), "{bad}");
    }
}

/// «Apri nel browser» opens the local server and the sites of a pool
/// (`<project>.localhost`), nothing else.
#[test]
fn local_urls_include_the_pool_sites() {
    for good in [
        "http://localhost:7100/",
        "http://127.0.0.1:7100/a?b#c",
        "http://shop.localhost:8080/",
        "http://api.shop-2.localhost:8080",
    ] {
        assert!(app::is_local_url(good), "{good}");
    }
    for bad in [
        "https://localhost:7100/",
        "http://example.com:80/",
        "http://localhost.example.com:80/",
        "http://evil.com/.localhost:80/",
        "http://.localhost:80/",
        "http://a..localhost:80/",
        "http://a_b.localhost:80/",
        "http://shop.localhost:99999/",
        "http://localhost:7100/ \"x\"",
    ] {
        assert!(!app::is_local_url(bad), "{bad}");
    }
}

/// Commands queued with [`in_order`] run one at a time in the order they
/// asked, whatever the order their tasks are polled in afterwards (an
/// `async` command runs on any thread of the pool).
#[test]
fn a_queue_keeps_the_order_asked() {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Waker};
    let queue = Queue::const_new(());
    let done = Arc::new(Mutex::new(Vec::new()));
    let mut cx = Context::from_waker(Waker::noop());
    // A command applying while the others arrive.
    let busy = queue.try_lock().unwrap();
    let mut tasks: Vec<Pin<Box<dyn Future<Output = ()> + '_>>> = (0..8)
        .map(|i| {
            let done = Arc::clone(&done);
            Box::pin(in_order(&queue, async move {
                done.lock().unwrap().push(i);
            })) as Pin<Box<dyn Future<Output = ()> + '_>>
        })
        .collect();
    // Each one asks, in this order.
    for t in &mut tasks {
        assert!(t.as_mut().poll(&mut cx).is_pending());
    }
    drop(busy);
    // Polled from the last: they still run from the first.
    let mut left: Vec<_> = tasks.into_iter().rev().collect();
    while !left.is_empty() {
        left.retain_mut(|t| t.as_mut().poll(&mut cx).is_pending());
    }
    assert_eq!(*done.lock().unwrap(), (0..8).collect::<Vec<_>>());
}

/// `gemini_key_set`'s save: the settings as they are on disk with the key
/// alone, trimmed; "" removes it. The other values, and the keys of a
/// newer version, stay.
#[test]
fn the_gemini_key_is_saved_alone() {
    let (_tmp, kit) = prepared_kit("ipc-gemini-key");
    Settings::update(&kit, |s, _| {
        s.auto_update = false;
        s.lang = Some("en".into());
        s.other.insert("future".into(), serde_json::json!(1));
        anyhow::Ok(())
    })
    .unwrap();
    let on_disk = || Settings::load(&kit).settings;
    let turned_on = app::write_settings(&kit, "it", on_disk(), Some(" k-1 ".into())).unwrap();
    assert!(!turned_on);
    let saved = on_disk();
    assert_eq!(saved.gemini_api_key.as_deref(), Some("k-1"));
    assert!(!saved.auto_update);
    assert_eq!(saved.lang.as_deref(), Some("en"));
    assert_eq!(saved.other.get("future"), Some(&serde_json::json!(1)));
    app::write_settings(&kit, "it", on_disk(), Some(String::new())).unwrap();
    let saved = on_disk();
    assert_eq!(saved.gemini_api_key, None);
    assert_eq!(saved.lang.as_deref(), Some("en"));
}

#[test]
fn unavailable_msg_names_the_system() {
    use crate::kit::Target;
    let mac = unavailable_msg("it", "Apache", Target::DARWIN_ARM64);
    assert_eq!(mac, "Apache non è disponibile per darwin-arm64.");
    let linux = unavailable_msg("en", "PHP", Target::LINUX_X64);
    assert_eq!(linux, "PHP is not available for linux-x64.");
    for lang in ["it", "en", "fr", "de"] {
        let text = unavailable_msg(lang, "Caddy", Target::LINUX_ARM64);
        assert!(
            text.contains("linux-arm64") && !text.contains("ARM64"),
            "{text}"
        );
    }
}
