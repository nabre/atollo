use super::*;

/// go-keyring on macOS stores `go-keyring-base64:<base64>` and reads also
/// the hex form and a value without a prefix.
#[test]
fn keychain_values_as_go_keyring() {
    let raw = br#"{"token":"ya29.a0","refresh":"1//x"}"#;
    let stored = mac_encode(raw);
    assert!(stored.starts_with("go-keyring-base64:"));
    assert_eq!(mac_decode(&stored).unwrap(), raw);
    assert_eq!(
        mac_decode(&format!("{stored}\n")).unwrap(),
        raw,
        "security -w ends with a newline"
    );
    assert_eq!(mac_decode("go-keyring-encoded:6869").unwrap(), b"hi");
    assert!(mac_decode("go-keyring-encoded:686").is_err());
    assert_eq!(mac_decode("plain").unwrap(), b"plain");
    assert!(mac_decode("go-keyring-base64:@@").is_err());
}

/// The secret goes through stdin, never among the arguments; names that
/// `security -i` would split or interpret are refused.
#[test]
fn keychain_add_command() {
    let line = mac_add_command("gemini", "antigravity", b"s3cret").unwrap();
    assert_eq!(
        line,
        format!(
            "add-generic-password -U -s gemini -a antigravity -w {}\n",
            mac_encode(b"s3cret")
        )
    );
    assert!(mac_add_command("gem ini", "antigravity", b"x").is_err());
    assert!(mac_add_command("gemini", "a;rm", b"x").is_err());
    assert!(mac_add_command("gemini", "a\"b", b"x").is_err());
    assert!(mac_add_command("", "antigravity", b"x").is_err());
    // Any secret is fine: base64 has only plain characters.
    assert!(mac_add_command("gemini", "antigravity", "è \n'\"$".as_bytes()).is_ok());
}

#[test]
fn unix_items_and_fingerprints() {
    let a = unix_item("gemini", "antigravity", b"one");
    assert_eq!(a.target, "gemini:antigravity");
    assert_eq!(a.user.as_deref(), Some("antigravity"));
    assert_eq!(
        a.blob.as_deref(),
        Some(
            base64::engine::general_purpose::STANDARD
                .encode(b"one")
                .as_str()
        )
    );
    let b = unix_item("gemini", "antigravity", b"two");
    assert_ne!(a.written, b.written, "a changed secret looks rewritten");
    assert_eq!(
        a.written,
        unix_item("gemini", "antigravity", b"one").written
    );
    assert!(
        !format!("{a:?}").contains(a.blob.as_deref().unwrap()),
        "no secret in Debug"
    );
}

#[test]
fn sealing_round_trip() {
    let key = [7u8; 32];
    let nonce = [3u8; NONCE];
    let secret = b"secret of this computer, longer than one block of 32 bytes";
    let blob = base64::engine::general_purpose::STANDARD.encode(secret);
    let sealed = seal(&key, &nonce, &blob).unwrap();
    assert!(sealed.starts_with(SEALED));
    assert!(!sealed.contains(&blob));
    assert_eq!(open_sealed(&key, &sealed).unwrap(), blob);
    // Another key, an altered byte, a cut value: refused.
    assert!(open_sealed(&[8u8; 32], &sealed).is_err());
    let mut raw = base64::engine::general_purpose::STANDARD
        .decode(&sealed[SEALED.len()..])
        .unwrap();
    raw[NONCE] ^= 1;
    let altered = format!(
        "{SEALED}{}",
        base64::engine::general_purpose::STANDARD.encode(&raw)
    );
    assert!(open_sealed(&key, &altered).is_err());
    assert!(open_sealed(&key, "kc1:AAAA").is_err());
    assert!(open_sealed(&key, &blob).is_err(), "not sealed");
    // Another nonce, another value for the same secret.
    assert_ne!(seal(&key, &[4u8; NONCE], &blob).unwrap(), sealed);
    // Empty secret.
    let empty = seal(&key, &nonce, "").unwrap();
    assert_eq!(open_sealed(&key, &empty).unwrap(), "");
}

/// The tag of a run's files: the machine and the account, never the host
/// name (which changes with the network and repeats between Macs).
#[test]
fn run_files_tagged_by_machine_and_account() {
    let mac = "4C4C4544-0042-3510-8051-B4C04F4D4C31";
    let a = machine_tag(mac, 501);
    assert_eq!(a, machine_tag(&mac.to_ascii_lowercase(), 501), "stable");
    assert_eq!(a, machine_tag(&format!("{mac}\n"), 501));
    assert_ne!(a, machine_tag(mac, 502), "another account");
    assert_ne!(a, machine_tag("4C4C4544-0042-3510-8051-B4C04F4D4C32", 501));
    assert_eq!(a.len(), 17);
    assert!(a.starts_with('m') && a[1..].bytes().all(|b| b.is_ascii_hexdigit()));
    assert!(
        !a.to_ascii_uppercase().contains("4C4C4544"),
        "the id is not written"
    );
    assert_eq!(host_tag(&a), a, "usable in a file name");
}

/// The collection agy's sign-in goes into: `login` by its object path,
/// whatever `default` is and wherever it sits; never another one.
#[test]
fn secret_service_login_collection_only() {
    const BASE: &str = "/org/freedesktop/secrets/collection/";
    assert_eq!(LOGIN_COLLECTION, format!("{BASE}login"));
    // GNOME: `login` (also the `default` alias), `session`.
    assert_eq!(
        login_collection([
            "/org/freedesktop/secrets/collection/session",
            "/org/freedesktop/secrets/collection/login",
        ])
        .unwrap(),
        1
    );
    // KWallet with a `login` collection next to the default `kdewallet`.
    assert_eq!(
        login_collection([
            "/org/freedesktop/secrets/collection/kdewallet",
            "/org/freedesktop/secrets/collection/login",
        ])
        .unwrap(),
        1
    );
    // KeePassXC (a database `Passwords` as `default`), or nothing at all:
    // fail closed, never `default` nor a collection named like login.
    for paths in [
        &[
            "/org/freedesktop/secrets/collection/Passwords",
            "/org/freedesktop/secrets/aliases/default",
        ][..],
        &[
            "/org/freedesktop/secrets/collection/login2",
            "/org/freedesktop/secrets/collection/Login",
            "/org/freedesktop/secrets/collection/login/x",
        ][..],
        &[][..],
    ] {
        let e = login_collection(paths.iter().copied()).unwrap_err();
        assert_eq!(e.to_string(), NO_LOGIN_COLLECTION);
    }
}

/// `security find-generic-password`: only 44 is "no such item"; 36
/// (interaction not allowed), 128 (unlock cancelled) or a kill are errors,
/// never an empty Keychain.
#[test]
fn keychain_read_failures_are_errors() {
    assert!(mac_found(Some(0)).unwrap());
    assert!(!mac_found(Some(44)).unwrap());
    for code in [Some(36), Some(128), Some(1), Some(51), None] {
        assert!(mac_found(code).is_err(), "{code:?}");
    }
    assert!(format!("{:#}", mac_found(Some(36)).unwrap_err()).contains("36"));
}

#[test]
fn missing_secret_service_recognized() {
    let real = "Secret Service: Platform secure storage failure: DBus error: The name org.freedesktop.secrets was not provided by any .service files";
    assert!(no_secret_service(real));
    assert!(no_secret_service(
        "org.freedesktop.DBus.Error.ServiceUnknown: x"
    ));
    assert!(no_secret_service(
        "Unable to autolaunch a dbus-daemon without a $DISPLAY"
    ));
    assert!(!no_secret_service(
        "Secret Service: Couldn't access platform secure storage: collection is locked"
    ));
    assert!(!no_secret_service("prompt dismissed"));
}
