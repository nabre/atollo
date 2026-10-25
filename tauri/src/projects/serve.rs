//! `Atollo.exe serve <folder> <port>`: file server for HTML/CSS/JS-only
//! projects, on 127.0.0.1 only, or on every interface with
//! `ATOLLO_ALLOW_LAN=1` (Settings).
//!
//! - only requests addressed to localhost, or with the local network
//!   allowed to an IP address (a `Host` of another name that points to the
//!   PC, DNS rebinding, cannot read the project);
//! - no hidden files or folders (`.git`, `.env`), except `.well-known`, nor
//!   names shaped like NTFS 8.3 short names (`GIT~1`), which could reach them;
//! - nothing outside the folder (`..`, links leading out);
//! - a folder without the final `/` is redirected (relative links work).

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

fn mime(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" | "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "avif" => "image/avif",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "pdf" => "application/pdf",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}

/// `localhost`, `*.localhost`, `127.0.0.1`, `[::1]`, with or without a port
/// and a final dot, in any case: the names `php -S`'s router, Caddy and
/// Apache take as local (SEC-V5-01).
pub fn is_local_host_header(host: &str) -> bool {
    let host = host.trim().to_ascii_lowercase();
    let name = if let Some(rest) = host.strip_prefix('[') {
        rest.split(']')
            .next()
            .map(|h| format!("[{h}]"))
            .unwrap_or_default()
    } else {
        host.split(':').next().unwrap_or_default().to_owned()
    };
    let name = name.trim_end_matches('.');
    matches!(name, "localhost" | "127.0.0.1" | "[::1]") || name.ends_with(".localhost")
}

fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = s.get(i + 1..i + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// A segment shaped like an NTFS 8.3 short name (`~` followed by a digit,
/// as in `GIT~1`): it could reach a hidden file (`.git`, `.env`) under
/// another name, so it is refused without asking Windows.
fn is_short_name(segment: &str) -> bool {
    segment
        .as_bytes()
        .windows(2)
        .any(|w| w[0] == b'~' && w[1].is_ascii_digit())
}

pub enum Answer {
    File(PathBuf),
    Redirect(String),
    Status(u16, &'static str),
}

/// What to answer for `target` (path and query of the request line).
pub fn resolve(root: &Path, target: &str) -> Answer {
    let (path, query) = target.split_once('?').map_or((target, ""), |(p, q)| (p, q));
    let Some(rel) = percent_decode(path) else {
        return Answer::Status(400, "Bad Request");
    };
    if !rel.starts_with('/') || rel.contains('\0') {
        return Answer::Status(400, "Bad Request");
    }
    if rel
        .split(['/', '\\'])
        .any(|s| (s.starts_with('.') && s != ".well-known") || is_short_name(s))
    {
        return Answer::Status(403, "Forbidden");
    }
    let rel_path = Path::new(rel.trim_start_matches(['/', '\\']));
    if rel_path
        .components()
        .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Answer::Status(403, "Forbidden");
    }
    let mut file = root.join(rel_path);
    if crate::platform::safe_path::is_dir(root, &file) {
        if !rel.ends_with('/') {
            let q = if query.is_empty() {
                String::new()
            } else {
                format!("?{query}")
            };
            // One leading slash: `//assets/` (or `/\assets/`) would be read
            // as the host `assets` (BUG-V6-28).
            let path = path.trim_start_matches(['/', '\\']);
            return Answer::Redirect(format!("/{path}/{q}"));
        }
        file = file.join("index.html");
    }
    // A link inside the folder is refused without ever following it (D-77).
    let Some((base, real)) =
        crate::platform::safe_path::lexical(root).zip(crate::platform::safe_path::lexical(&file))
    else {
        return Answer::Status(403, "Forbidden");
    };
    if !real.starts_with(&base) || !crate::platform::safe_path::no_links(&base, &real) {
        return Answer::Status(403, "Forbidden");
    }
    if crate::platform::safe_path::is_file(&base, &real) {
        Answer::File(real)
    } else {
        Answer::Status(404, "Not Found")
    }
}

/// A `Host` that is an IP address (v4, or v6 in brackets), with or without a
/// port: never a name.
fn is_ip_host_header(host: &str) -> bool {
    let host = host.trim();
    if let Some(rest) = host.strip_prefix('[') {
        return rest
            .split(']')
            .next()
            .is_some_and(|h| h.parse::<std::net::Ipv6Addr>().is_ok());
    }
    host.split(':')
        .next()
        .is_some_and(|h| h.parse::<Ipv4Addr>().is_ok())
}

/// Longest wait for a single read of the request head.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Total time allowed for the whole request head, however it trickles in:
/// a client sending one byte every few seconds is cut here, not after the
/// 8192 bytes of the head (hours). Shorter under test, to keep tests quick.
pub(crate) const HEAD_DEADLINE: Duration = if cfg!(test) {
    Duration::from_secs(3)
} else {
    Duration::from_secs(10)
};

/// Reads from the connection, each read waiting at most what is left
/// before `deadline` (and never more than [`READ_TIMEOUT`]).
struct DeadlineReader {
    stream: TcpStream,
    deadline: Instant,
}

impl Read for DeadlineReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let left = self.deadline.saturating_duration_since(Instant::now());
        // `set_read_timeout(Some(0))` is an error: a spent budget ends here.
        if left.is_zero() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "request head deadline",
            ));
        }
        self.stream.set_read_timeout(Some(left.min(READ_TIMEOUT)))?;
        self.stream.read(buf)
    }
}

fn handle(root: &Path, lan: bool, mut stream: TcpStream) -> Result<()> {
    let deadline = Instant::now() + HEAD_DEADLINE;
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
    let head = DeadlineReader {
        stream: stream.try_clone()?,
        deadline,
    };
    let mut reader = BufReader::new(head.take(8192));
    let mut request = String::new();
    reader.read_line(&mut request)?;
    let mut host = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("host") {
                host = v.trim().to_owned();
            }
        }
    }
    let mut parts = request.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    let head_only = method == "HEAD";
    let answer = if !is_local_host_header(&host) && !(lan && is_ip_host_header(&host)) {
        Answer::Status(403, "Forbidden")
    } else if method != "GET" && !head_only {
        Answer::Status(405, "Method Not Allowed")
    } else {
        resolve(root, target)
    };
    let (status, kind, body, extra) = match answer {
        Answer::File(path) => match crate::platform::safe_path::read(root, &path) {
            Some(bytes) => ("200 OK".to_owned(), mime(&path), bytes, String::new()),
            None => (
                "404 Not Found".to_owned(),
                "text/plain; charset=utf-8",
                b"Not Found".to_vec(),
                String::new(),
            ),
        },
        Answer::Redirect(location) => (
            "301 Moved Permanently".to_owned(),
            "text/plain; charset=utf-8",
            Vec::new(),
            format!("Location: {location}\r\n"),
        ),
        Answer::Status(code, text) => {
            println!("{code} {target}");
            (
                format!("{code} {text}"),
                "text/plain; charset=utf-8",
                text.as_bytes().to_vec(),
                String::new(),
            )
        }
    };
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n{extra}\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    if !head_only {
        stream.write_all(&body)?;
    }
    stream.flush()?;
    Ok(())
}

/// Connections served at the same time; the others are closed at once.
const MAX_CONNS: usize = 64;

struct ConnGuard(Arc<AtomicUsize>);

impl Drop for ConnGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Serves `root` on `127.0.0.1:port` (every interface with the local
/// network allowed) until the process is ended.
pub fn run(root: &Path, port: u16, lang: &str) -> Result<()> {
    let abs = if root.is_absolute() {
        root.to_path_buf()
    } else {
        std::env::current_dir()
            .with_context(|| root.display().to_string())?
            .join(root)
    };
    let root = crate::platform::safe_path::lexical(&abs)
        .filter(|r| crate::platform::safe_path::is_dir(r, r))
        .with_context(|| root.display().to_string())?;
    let lan = std::env::var("ATOLLO_ALLOW_LAN").is_ok_and(|v| v == "1");
    let ip = if lan {
        Ipv4Addr::UNSPECIFIED
    } else {
        Ipv4Addr::LOCALHOST
    };
    let listener = TcpListener::bind((ip, port)).with_context(|| format!("{ip}:{port}"))?;
    println!(
        "{}",
        crate::common::i18n::t(
            lang,
            "Sito statico: {0}   (cartella {1})",
            &[
                &format!("http://localhost:{port}/"),
                root.display().to_string().trim_start_matches(r"\\?\")
            ]
        )
    );
    serve(&root, &listener, lan);
    Ok(())
}

/// The connections of `listener`, each in a thread of its own, at most
/// `MAX_CONNS` at a time. The tests give it a listener already bound to a
/// free port: a port found free and bound again later could be taken by
/// another test's server in between (TEST-V7-02).
fn serve(root: &Path, listener: &TcpListener, lan: bool) {
    let active = Arc::new(AtomicUsize::new(0));
    for stream in listener.incoming().flatten() {
        let prev = active.fetch_add(1, Ordering::AcqRel);
        let guard = ConnGuard(Arc::clone(&active));
        if prev >= MAX_CONNS {
            continue;
        }
        let root = root.to_path_buf();
        std::thread::spawn(move || {
            let _guard = guard;
            let _ = handle(&root, lan, stream);
        });
    }
}

#[cfg(test)]
#[path = "../../../test/rust/projects/serve.rs"]
mod tests;
