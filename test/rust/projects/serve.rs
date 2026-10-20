use super::*;
use crate::kit::tests::TempDir;
use std::fs;
use std::io::Read;

#[test]
fn host_header() {
    assert!(is_local_host_header("localhost:7100"));
    assert!(is_local_host_header("127.0.0.1"));
    assert!(is_local_host_header("[::1]:80"));
    assert!(!is_local_host_header("evil.example:7100"));
    assert!(!is_local_host_header("localhost.evil.example"));
    assert!(!is_local_host_header(""));
    // SEC-V5-01: `<project>.localhost` as with `php -S`, Caddy and Apache.
    assert!(is_local_host_header("a.localhost"));
    assert!(is_local_host_header("a.localhost:8000"));
    assert!(is_local_host_header("A.LOCALHOST."));
    assert!(is_local_host_header("localhost.:8000"));
    assert!(!is_local_host_header("localhost.evil.com"));
    assert!(!is_local_host_header("evil.com"));
    assert!(!is_local_host_header("evillocalhost"));

    // With the local network allowed: IP addresses, never names.
    assert!(is_ip_host_header("192.168.1.20:7100"));
    assert!(is_ip_host_header("[fe80::1]:7100"));
    assert!(!is_ip_host_header("pc.example:7100"));
    assert!(!is_ip_host_header("1.2.3.4.evil.example"));
    assert!(!is_ip_host_header(""));
}

#[test]
fn paths() {
    let tmp = TempDir::new("serve");
    let root = tmp.0.join("site");
    let outside = tmp.0.join("outside");
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(root.join("index.html"), "home").unwrap();
    fs::write(root.join("sub/index.html"), "sub").unwrap();
    fs::write(root.join(".env"), "secret").unwrap();
    fs::write(outside.join("secret.txt"), "leaked").unwrap();
    crate::os::safe_path::tests::junction(&root.join("lnk"), &outside);
    assert!(matches!(resolve(&root, "/"), Answer::File(p) if p.ends_with("index.html")));
    assert!(matches!(resolve(&root, "/sub?x=1"), Answer::Redirect(l) if l == "/sub/?x=1"));
    assert!(matches!(resolve(&root, "/.env"), Answer::Status(403, _)));
    assert!(matches!(
        resolve(&root, "/%2e%2e/x"),
        Answer::Status(403, _)
    ));
    assert!(matches!(
        resolve(&root, "/sub/..%5C..%5Cx"),
        Answer::Status(403, _)
    ));
    assert!(matches!(
        resolve(&root, "/lnk/secret.txt"),
        Answer::Status(403, _)
    ));
    assert!(matches!(resolve(&root, "/lnk/"), Answer::Status(403, _)));
    assert!(matches!(
        resolve(&root, "/missing.js"),
        Answer::Status(404, _)
    ));
    assert!(matches!(resolve(&root, "/%zz"), Answer::Status(400, _)));

    // NTFS 8.3 short names (`ENV~1` is `.env`, `GIT~1` is `.git`), in any
    // case and encoding: refused before touching the disk.
    fs::write(root.join("a~b.txt"), "tilde").unwrap();
    for target in [
        "/ENV~1",
        "/env~1",
        "/GIT~1/config",
        "/sub/GIT~1/config",
        "/%45NV%7E1",
        "/sub%5CGIT~1",
    ] {
        assert!(
            matches!(resolve(&root, target), Answer::Status(403, _)),
            "{target}"
        );
    }
    // A `~` not followed by a digit is an ordinary name.
    assert!(matches!(resolve(&root, "/a~b.txt"), Answer::File(p) if p.ends_with("a~b.txt")));
    assert!(matches!(resolve(&root, "/x~.txt"), Answer::Status(404, _)));
}

/// Starts `run` on a free port in a background thread.
fn start(root: &Path) -> u16 {
    let port = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let root = root.to_path_buf();
    std::thread::spawn(move || run(&root, port, "it"));
    port
}

#[test]
fn serves_over_http() {
    let tmp = TempDir::new("serve-http");
    fs::write(tmp.0.join("index.html"), "<h1>ok</h1>").unwrap();
    let port = start(&tmp.0);
    let get = |host: &str| {
        for _ in 0..50 {
            if let Ok(mut s) = TcpStream::connect((Ipv4Addr::LOCALHOST, port)) {
                write!(s, "GET / HTTP/1.1\r\nHost: {host}\r\n\r\n").unwrap();
                let mut out = String::new();
                s.read_to_string(&mut out).unwrap();
                return out;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("server did not start");
    };
    let ok = get(&format!("localhost:{port}"));
    assert!(
        ok.starts_with("HTTP/1.1 200") && ok.ends_with("<h1>ok</h1>"),
        "{ok}"
    );
    assert!(get("rebind.example").starts_with("HTTP/1.1 403"));
}

/// A full house of silent connections (slowloris) is closed by the 5 s read
/// timeout: a real request is answered again soon after, not never.
#[test]
fn slowloris_does_not_block() {
    let tmp = TempDir::new("serve-slowloris");
    fs::write(tmp.0.join("index.html"), "<h1>ok</h1>").unwrap();
    let port = start(&tmp.0);
    let get = || -> Option<String> {
        let mut s = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).ok()?;
        s.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
        write!(s, "GET / HTTP/1.1\r\nHost: localhost:{port}\r\n\r\n").ok()?;
        let mut out = String::new();
        s.read_to_string(&mut out).ok()?;
        Some(out).filter(|o| o.starts_with("HTTP/1.1 200"))
    };
    // The server is up before the flood starts.
    let mut up = false;
    for _ in 0..50 {
        if get().is_some() {
            up = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(up, "server did not start");

    let started = std::time::Instant::now();
    let idle: Vec<TcpStream> = (0..MAX_CONNS + 6)
        .map(|_| TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap())
        .collect();
    let mut answered = None;
    while started.elapsed() < Duration::from_secs(8) {
        if let Some(out) = get() {
            answered = Some(out);
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    drop(idle);
    let out = answered.expect("no answer while silent connections were open");
    assert!(out.ends_with("<h1>ok</h1>"), "{out}");
}

/// A client sending one byte a second never trips the per-read timeout, but
/// the total deadline for the request head closes it all the same.
#[test]
fn dribbling_client_is_cut_after_deadline() {
    let tmp = TempDir::new("serve-dribble");
    fs::write(tmp.0.join("index.html"), "<h1>ok</h1>").unwrap();
    let port = start(&tmp.0);
    let mut s = None;
    for _ in 0..50 {
        if let Ok(c) = TcpStream::connect((Ipv4Addr::LOCALHOST, port)) {
            s = Some(c);
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let mut s = s.expect("server did not start");
    // Once, before the server can cut: macOS refuses setsockopt on a socket
    // the peer has closed (EINVAL).
    s.set_read_timeout(Some(Duration::from_millis(1000)))
        .unwrap();
    let started = std::time::Instant::now();
    let head = format!(
        "GET / HTTP/1.1\r\nHost: localhost:{port}\r\nX-Pad: {}",
        "a".repeat(64)
    );
    let mut cut = false;
    for byte in head.bytes() {
        if s.write_all(&[byte]).and_then(|()| s.flush()).is_err() {
            cut = true;
            break;
        }
        // A closed connection reads as end of stream (or a reset).
        let mut buf = [0u8; 64];
        let closed = match s.read(&mut buf) {
            Ok(0) => true,
            Ok(n) => panic!(
                "unexpected answer: {:?}",
                String::from_utf8_lossy(&buf[..n])
            ),
            Err(e) => !matches!(
                e.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ),
        };
        if closed {
            assert!(
                started.elapsed() >= HEAD_DEADLINE - Duration::from_millis(500),
                "closed too early: {:?}",
                started.elapsed()
            );
            cut = true;
            break;
        }
        assert!(
            started.elapsed() < HEAD_DEADLINE + Duration::from_secs(3),
            "still open after {:?}",
            started.elapsed()
        );
    }
    assert!(cut, "the dribbling client was never cut");
    assert!(started.elapsed() < HEAD_DEADLINE + Duration::from_secs(3));
}
