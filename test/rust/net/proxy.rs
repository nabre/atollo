use super::*;

/// D-105: the proxy of System Settings on macOS, from `scutil --proxy`.
#[test]
fn macos_proxy_from_scutil() {
    let both = "<dictionary> {
  ExceptionsList : <array> {
    0 : *.local
    1 : 169.254/16
  }
  FTPPassive : 1
  HTTPEnable : 1
  HTTPPort : 8080
  HTTPProxy : web.corp.example
  HTTPSEnable : 1
  HTTPSPort : 8443
  HTTPSProxy : secure.corp.example
}
";
    assert_eq!(
        scutil_proxy(both).as_deref(),
        Some("http://secure.corp.example:8443")
    );
    let http_only = both.replace("HTTPSEnable : 1", "HTTPSEnable : 0");
    assert_eq!(
        scutil_proxy(&http_only).as_deref(),
        Some("http://web.corp.example:8080")
    );
    let none = "<dictionary> {\n  ExceptionsList : <array> {\n    0 : HTTPEnable : 1\n  }\n  FTPPassive : 1\n}\n";
    assert_eq!(scutil_proxy(none), None);
    assert_eq!(scutil_proxy(""), None);
    assert_eq!(scutil_exceptions(both), ["*.local", "169.254/16"]);
    assert!(scutil_exceptions("<dictionary> {\n  HTTPEnable : 1\n}\n").is_empty());
}

/// D-105: the system's exceptions reach the proxy (a review finding).
#[test]
fn system_proxy_exceptions() {
    assert_eq!(
        gsettings_list("['localhost', '127.0.0.0/8', '*.corp.example']\n"),
        ["localhost", "127.0.0.0/8", "*.corp.example"]
    );
    assert!(gsettings_list("@as []\n").is_empty());
    assert!(gsettings_list("").is_empty());

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

/// D-105: GNOME's proxy, only when set by hand.
#[test]
fn gnome_proxy_from_gsettings() {
    let https = ("'secure.corp.example'\n", "8443\n");
    let http = ("'web.corp.example'\n", "8080\n");
    assert_eq!(
        gnome_proxy("'manual'\n", https, http).as_deref(),
        Some("http://secure.corp.example:8443")
    );
    assert_eq!(
        gnome_proxy("'manual'", ("''", "0"), http).as_deref(),
        Some("http://web.corp.example:8080")
    );
    assert_eq!(gnome_proxy("'none'", https, http), None);
    assert_eq!(gnome_proxy("'auto'", https, http), None);
    assert_eq!(gnome_proxy("'manual'", ("''", "0"), ("''", "0")), None);
    assert_eq!(
        proxy_url("http://p.example/", None).as_deref(),
        Some("http://p.example")
    );
}
