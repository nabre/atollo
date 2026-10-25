use super::*;

/// D-105: the system's exceptions reach the proxy (a review finding).
#[test]
fn system_proxy_exceptions() {
    let proxy = Proxy::new("http://web.corp.example:8080").unwrap();
    let hosts = ["localhost".to_owned(), "*.corp.example".to_owned()];
    let proxy = with_exceptions(proxy, &hosts);
    assert_eq!((proxy.host(), proxy.port()), ("web.corp.example", 8080));
    let uri = |u: &str| u.parse::<ureq::http::Uri>().unwrap();
    assert!(proxy.is_no_proxy(&uri("http://localhost/x")));
    assert!(proxy.is_no_proxy(&uri("https://git.corp.example/x")));
    assert!(!proxy.is_no_proxy(&uri("https://github.com/x")));
    let plain = with_exceptions(Proxy::new("http://p.example:3128").unwrap(), &[]);
    assert!(!plain.is_no_proxy(&uri("http://localhost/x")));
}
