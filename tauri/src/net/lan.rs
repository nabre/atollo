//! A project's site from the phone (report v6/04, N-6, DECISIONS.md D-158):
//! with «Rete locale» on, the site's address with this PC's IP on the local
//! network, and its QR, made here (no service on the internet).

use std::net::{IpAddr, Ipv4Addr, UdpSocket};

use serde::Serialize;

/// This PC's address on the local network: the one of the route out (a UDP
/// "connect" sends nothing; 192.0.2.1 is a documentation address), only
/// when it is a private one (192.168.x, 10.x, 172.16-31.x).
pub fn lan_ip() -> Option<Ipv4Addr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect((Ipv4Addr::new(192, 0, 2, 1), 9)).ok()?;
    match socket.local_addr().ok()?.ip() {
        IpAddr::V4(ip) if ip.is_private() => Some(ip),
        _ => None,
    }
}

/// `url` as the other devices reach it: this PC's host (`localhost`,
/// `127.0.0.1`, `0.0.0.0`) replaced by `ip`, port and path kept. `None` for
/// any other host: a pool site's name (`shop.localhost`) is chosen by the
/// server from the name, which the phone cannot send.
pub fn lan_url(url: &str, ip: Ipv4Addr) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    if !matches!(scheme, "http" | "https") {
        return None;
    }
    let (authority, path) = match rest.find(['/', '?', '#']) {
        Some(i) => rest.split_at(i),
        None => (rest, ""),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) if !p.is_empty() && p.parse::<u16>().is_ok() => (h, Some(p)),
        _ => (authority, None),
    };
    if !matches!(
        host.to_ascii_lowercase().as_str(),
        "localhost" | "127.0.0.1" | "0.0.0.0" | "[::1]" | "[::]"
    ) {
        return None;
    }
    let port = port.map(|p| format!(":{p}")).unwrap_or_default();
    let slash = if path.starts_with('/') { "" } else { "/" };
    Some(format!("{scheme}://{ip}{port}{slash}{path}"))
}

/// A QR as its modules, drawn by the page: `width` × `width`, row by row,
/// `1` dark and `0` light; without the quiet zone (the page adds it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Qr {
    pub width: usize,
    pub modules: String,
}

pub fn qr(text: &str) -> Option<Qr> {
    let code = qrcode::QrCode::new(text.as_bytes()).ok()?;
    Some(Qr {
        width: code.width(),
        modules: code
            .to_colors()
            .iter()
            .map(|c| if *c == qrcode::Color::Dark { '1' } else { '0' })
            .collect(),
    })
}

/// The site's address on the local network with its QR.
#[derive(Debug, Clone, Serialize)]
pub struct LanLink {
    pub url: String,
    pub qr: Qr,
}

/// `url`'s address on the local network and its QR, when `allowed`
/// («Rete locale») and the PC has a private address.
pub fn lan_link(url: &str, allowed: bool) -> Option<LanLink> {
    if !allowed {
        return None;
    }
    let url = lan_url(url, lan_ip()?)?;
    let qr = qr(&url)?;
    Some(LanLink { url, qr })
}

#[cfg(test)]
#[path = "../../../test/rust/net/lan.rs"]
mod tests;
