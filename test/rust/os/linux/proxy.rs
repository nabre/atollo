use super::*;

/// D-105: GNOME's proxy, only when set by hand.
#[test]
fn gnome_proxy_from_gsettings() {
    assert_eq!(
        gsettings_list("['localhost', '127.0.0.0/8', '*.corp.example']\n"),
        ["localhost", "127.0.0.0/8", "*.corp.example"]
    );
    assert!(gsettings_list("@as []\n").is_empty());
    assert!(gsettings_list("").is_empty());

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
