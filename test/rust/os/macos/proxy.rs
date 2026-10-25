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
