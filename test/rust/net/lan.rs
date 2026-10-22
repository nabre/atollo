use super::*;

const IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 20);

#[test]
fn site_address_on_the_network() {
    let lan = |u: &str| lan_url(u, IP);
    assert_eq!(
        lan("http://localhost:7100/").as_deref(),
        Some("http://192.168.1.20:7100/")
    );
    assert_eq!(
        lan("http://127.0.0.1:8000").as_deref(),
        Some("http://192.168.1.20:8000/")
    );
    assert_eq!(
        lan("https://LOCALHOST:5173/app?x=1#top").as_deref(),
        Some("https://192.168.1.20:5173/app?x=1#top")
    );
    assert_eq!(
        lan("http://0.0.0.0:3000?q").as_deref(),
        Some("http://192.168.1.20:3000/?q")
    );
    assert_eq!(
        lan("http://[::1]:4000/").as_deref(),
        Some("http://192.168.1.20:4000/")
    );
    assert_eq!(
        lan("http://localhost/").as_deref(),
        Some("http://192.168.1.20/")
    );
    // A pool site's name, another host, another scheme: none.
    assert_eq!(lan("http://shop.localhost:8080/"), None);
    assert_eq!(lan("http://example.com:80/"), None);
    assert_eq!(lan("ftp://localhost:21/"), None);
    assert_eq!(lan("localhost:7100"), None);
}

#[test]
fn qr_of_an_address() {
    let q = qr("http://192.168.1.20:7100/").unwrap();
    // Version 2 (25 modules) holds this address.
    assert_eq!(q.width, 25);
    assert_eq!(q.modules.len(), q.width * q.width);
    assert!(q.modules.chars().all(|c| c == '0' || c == '1'));
    // The finder pattern in the top-left corner: a dark 7-module row.
    assert_eq!(&q.modules[..7], "1111111");
    assert_eq!(q.modules.as_bytes()[7], b'0');
}

#[test]
fn nothing_without_the_local_network() {
    assert!(lan_link("http://localhost:7100/", false).is_none());
    // On: an address when this PC has a private one (not on every machine
    // of the tests).
    if let Some(ip) = lan_ip() {
        assert!(ip.is_private());
        let link = lan_link("http://localhost:7100/", true).unwrap();
        assert_eq!(link.url, format!("http://{ip}:7100/"));
    }
}
