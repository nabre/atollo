use super::*;
use crate::kit::tests::TempDir;
use std::io::Read;

#[test]
fn host_header() {
    assert!(is_local_host_header("localhost:7100"));
    assert!(is_local_host_header("127.0.0.1"));
    assert!(is_local_host_header("[::1]:80"));
    assert!(!is_local_host_header("evil.example:7100"));
    assert!(!is_local_host_header("localhost.evil.example"));
    assert!(!is_local_host_header(""));

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
    fs::create_dir_all(tmp.0.join("sub")).unwrap();
    fs::write(tmp.0.join("index.html"), "home").unwrap();
    fs::write(tmp.0.join("sub/index.html"), "sub").unwrap();
    fs::write(tmp.0.join(".env"), "secret").unwrap();
    let root = fs::canonicalize(&tmp.0).unwrap();
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
        resolve(&root, "/missing.js"),
        Answer::Status(404, _)
    ));
    assert!(matches!(resolve(&root, "/%zz"), Answer::Status(400, _)));
}

#[test]
fn serves_over_http() {
    let tmp = TempDir::new("serve-http");
    fs::write(tmp.0.join("index.html"), "<h1>ok</h1>").unwrap();
    let port = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let root = tmp.0.clone();
    std::thread::spawn(move || run(&root, port, "it"));
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
