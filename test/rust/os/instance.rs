use super::*;
use crate::kit::Arch;
use crate::kit::Os;
use std::sync::mpsc;

#[cfg(windows)]
fn unique_kit(tag: &str) -> Kit {
    Kit::at(
        PathBuf::from(format!(r"Z:\atollo-test-{}-{tag}", std::process::id())),
        Arch::X64,
    )
}

#[cfg(windows)]
#[test]
fn second_instance_of_same_kit_is_secondary() {
    let kit = unique_kit("same");
    let first = acquire(&kit).unwrap();
    assert!(matches!(first, Instance::Primary(_)));
    assert!(matches!(acquire(&kit).unwrap(), Instance::Secondary));
    let other = unique_kit("other");
    assert!(matches!(acquire(&other).unwrap(), Instance::Primary(_)));
    drop(first);
    assert!(matches!(acquire(&kit).unwrap(), Instance::Primary(_)));
}

#[cfg(windows)]
#[test]
fn message_reaches_primary() {
    let kit = unique_kit("pipe");
    let (tx, rx) = mpsc::channel();
    listen(&kit, move |m| {
        let _ = tx.send(m);
    });
    notify_primary(&kit, "focus").unwrap();
    assert_eq!(rx.recv_timeout(Duration::from_secs(5)).unwrap(), "focus");
}

/// D-105: never a socket inside the kit; on macOS a name that
/// fits the 104 bytes of `sun_path`, whatever the kit's folder.
#[test]
fn socket_names() {
    let id = "0123456789abcdef";
    assert_eq!(
        socket_name(Os::Linux, id, 1000, None),
        Some(SocketName::Abstract(format!("Atollo-1000-{id}")))
    );
    let tmp = Path::new("/var/folders/zz/zyxvpxvq6csfxvn_n0000000000000/T/");
    let mac = socket_name(Os::Darwin, id, 501, Some(tmp)).unwrap();
    let SocketName::File(path) = &mac else {
        panic!("{mac:?}");
    };
    assert!(
        path.starts_with(tmp) && path.as_os_str().len() <= 103,
        "{path:?}"
    );
    // Too long a temporary folder, or none: no socket (the TCP fallback).
    let long = PathBuf::from(format!("/{}", "x".repeat(90)));
    assert_eq!(socket_name(Os::Darwin, id, 501, Some(&long)), None);
    assert_eq!(socket_name(Os::Darwin, id, 501, None), None);
    // A kit in a 150-character folder: the id is a fixed 16 characters.
    let deep = Kit::at(
        PathBuf::from(format!("/media/u/{}", "k".repeat(150))),
        Arch::X64,
    );
    assert_eq!(deep.id().len(), 16);
}

/// D-105: the TCP fallback takes the message with the right token only.
#[test]
fn tcp_fallback_checks_the_token() {
    let tmp = crate::kit::tests::TempDir::new("instance-tcp");
    let file = tmp.0.join("instance.json");
    let (tx, rx) = mpsc::channel();
    listen_tcp(&file, "secret-token".into(), move |m| {
        let _ = tx.send(m);
    })
    .unwrap();
    let endpoint: TcpEndpoint =
        serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    assert_eq!(endpoint.token, "secret-token");
    // Created owner-only, no temporary file left beside it.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&file).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    assert_eq!(std::fs::read_dir(&tmp.0).unwrap().count(), 1);

    // A wrong token, and none at all: nothing arrives.
    let forged = tmp.0.join("forged.json");
    for token in ["secret-tokem", "secret", ""] {
        let wrong = TcpEndpoint {
            port: endpoint.port,
            token: token.into(),
        };
        std::fs::write(&forged, serde_json::to_string(&wrong).unwrap()).unwrap();
        notify_tcp(&forged, "focus").unwrap();
    }
    notify_tcp(&file, "focus").unwrap();
    assert_eq!(rx.recv_timeout(Duration::from_secs(5)).unwrap(), "focus");
    assert!(rx.recv_timeout(Duration::from_millis(300)).is_err());
    assert!(notify_tcp(&tmp.0.join("missing.json"), "focus").is_err());
    assert!(same_token("abc", "abc") && !same_token("abc", "abd") && !same_token("ab", "abc"));
}

/// Two starts of the same kit on Linux and macOS: the second one is
/// secondary and its message reaches the first through the socket.
#[cfg(unix)]
#[test]
fn second_start_reaches_the_first_on_unix() {
    let tmp = crate::kit::tests::TempDir::new("instance-unix");
    let kit = Kit::at(tmp.0.join("Kit"), Arch::X64);
    // A TCP fallback of an earlier start: gone once the lock is taken.
    let stale = kit.app_dir().join("instance.json");
    std::fs::create_dir_all(kit.app_dir()).unwrap();
    std::fs::write(&stale, r#"{"port":1,"token":"old"}"#).unwrap();
    let first = acquire(&kit).unwrap();
    assert!(!stale.exists());
    assert!(matches!(first, Instance::Primary(_)));
    assert!(matches!(acquire(&kit).unwrap(), Instance::Secondary));
    let (tx, rx) = mpsc::channel();
    listen(&kit, move |m| {
        let _ = tx.send(m);
    });
    notify_primary(&kit, "focus").unwrap();
    assert_eq!(rx.recv_timeout(Duration::from_secs(5)).unwrap(), "focus");
    // Nothing of the socket in the kit.
    assert!(!kit.app_dir().join("instance.sock").exists());
}

/// Both ends of the socket check the other's user (abstract names have no
/// permissions): a connection of this user passes.
#[cfg(unix)]
#[test]
fn socket_peer_is_this_user() {
    let (a, b) = std::os::unix::net::UnixStream::pair().unwrap();
    // SAFETY: getuid cannot fail.
    let me = unsafe { libc::getuid() };
    assert_eq!(peer_uid(&a), Some(me));
    assert!(same_user(b).is_ok());
}
