//! The output of a run: its lines without colours, the address a web
//! server prints, and the commands and variables moved to the local network.

use std::io::Read;
use std::sync::LazyLock;

use regex::Regex;

use crate::detect::Kind;

use super::{Event, Runner, State};

pub(super) const MAX_LINES: usize = 3000;

pub(super) static ANSI: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r"\x1b\[[0-?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[=>()][0-9A-Za-z]?")
        .ok()
});

pub(super) static LOCAL_URL: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r#"https?://(?:localhost|127\.0\.0\.1|0\.0\.0\.0|\[::1?\]):(\d{2,5})[^\s'"<>)]*"#)
        .ok()
});

/// Text without colours, split in lines (`\r\n`, `\n`, `\r`); the last,
/// incomplete piece stays in `rest`.
pub(crate) fn split_lines(rest: &mut String, chunk: &str) -> Vec<String> {
    rest.push_str(chunk);
    let mut lines: Vec<String> = rest.split(['\n', '\r']).map(str::to_owned).collect();
    *rest = lines.pop().unwrap_or_default();
    // "\r\n" leaves empty pieces between \r and \n: dropped.
    let had_crlf = chunk.contains("\r\n");
    if had_crlf {
        lines.retain(|l| !l.is_empty());
    }
    if rest.len() > 4000 {
        lines.push(std::mem::take(rest));
    }
    lines
        .into_iter()
        .map(|l| match ANSI.as_ref() {
            Some(re) => re.replace_all(&l, "").into_owned(),
            None => l,
        })
        .collect()
}

/// The text of the bytes read so far from a pipe, taken out of `pending`,
/// but an incomplete UTF-8 sequence at its end: a character split across
/// two reads is kept for the next one, never shown as U+FFFD. Invalid bytes
/// elsewhere are replaced.
pub(crate) fn take_utf8(pending: &mut Vec<u8>) -> String {
    let valid = match std::str::from_utf8(pending) {
        Ok(_) => pending.len(),
        Err(e) if e.error_len().is_none() => e.valid_up_to(),
        Err(_) => pending.len(),
    };
    let text = String::from_utf8_lossy(&pending[..valid]).into_owned();
    pending.drain(..valid);
    text
}

/// The loopback address the actions found by the app listen on.
pub(super) const LOCAL: &str = "127.0.0.1";

/// The flags (and the words) after which an address is the one a server
/// binds to, as the detectors write them (`--host` of Vite, uvicorn and
/// Flask, `-H` of Next and Gatsby, `-a` of http-server, `--address` of
/// Trunk, `--server.address` of Streamlit, `-l tcp://` of serve, `-S` of
/// PHP's server, `runserver` of Django) and as the usual servers take them
/// (`--bind` and `-b` of Gunicorn and Hugo, `--ip` of Jupyter, `--hostname`
/// and `--listen`).
pub(super) const BIND_FLAGS: [&str; 13] = [
    "--host",
    "-H",
    "--hostname",
    "-a",
    "--address",
    "--server.address",
    "--bind",
    "-b",
    "--ip",
    "--listen",
    "-l",
    "-S",
    "runserver",
];

/// The programs that connect to a server and never listen: their `--host`
/// (`mysql --host=127.0.0.1`) names the server they reach, which 0.0.0.0
/// would not be on Windows.
pub(super) const CLIENTS: [&str; 12] = [
    "mysql",
    "mysqldump",
    "mysqladmin",
    "mariadb",
    "psql",
    "pg_dump",
    "mongosh",
    "mongo",
    "mongodump",
    "redis-cli",
    "curl",
    "wget",
];

/// The variables that name the address a server binds to: the detectors
/// set `GRADIO_SERVER_NAME`, the others are the ones of Node's servers
/// (`HOST`, Nuxt's and Nitro's), of Flask, uvicorn and Streamlit. Any
/// other variable holding 127.0.0.1 (`DB_HOST`, `REDIS_HOST`) is a server
/// the program connects to, and stays.
pub(super) const BIND_KEYS: [&str; 8] = [
    "GRADIO_SERVER_NAME",
    "HOST",
    "NUXT_HOST",
    "NITRO_HOST",
    "FLASK_RUN_HOST",
    "UVICORN_HOST",
    "STREAMLIT_SERVER_ADDRESS",
    "BIND_ADDRESS",
];

/// A command found by the app listens on 127.0.0.1; with the local network
/// allowed, on every interface. Only the address a server binds to moves:
/// the one after a bind flag (`--host 127.0.0.1`, `--host=127.0.0.1`,
/// `-l tcp://127.0.0.1:{port}`, `php -S 127.0.0.1:{port}`, `runserver
/// 127.0.0.1:{port}`). Every other 127.0.0.1 (`mysql -h 127.0.0.1`, `curl
/// 127.0.0.1:8000`, the host of a URL) is one the program connects to, and
/// 0.0.0.0 cannot be connected to on Windows, so it stays (BUG-V7-10).
/// Neither do longer addresses such as 127.0.0.10. The actions already
/// saved in `panel.json` keep working: they hold the same forms.
pub(super) fn on_network(s: String, lan: bool) -> String {
    if !lan {
        return s;
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s.as_str();
    while let Some(at) = rest.find(LOCAL) {
        let (before, after) = (&rest[..at], &rest[at + LOCAL.len()..]);
        out.push_str(before);
        let prev = out.chars().next_back();
        let longer = prev.is_some_and(|c| c.is_ascii_digit() || c == '.')
            || after.starts_with(|c: char| c.is_ascii_digit());
        let bind = !longer && after_bind_flag(&out) && !in_client(&out);
        out.push_str(if bind { "0.0.0.0" } else { LOCAL });
        rest = after;
    }
    out.push_str(rest);
    out
}

/// `before` (the command up to an address) ends with a bind flag and its
/// separator: a space or `=`, an optional quote, and `tcp://` for serve's
/// `-l`. A `//` of any other scheme is a URL, which a program connects to.
/// A bind variable set on the command line (`HOST=127.0.0.1 npm run dev`)
/// counts too.
pub(super) fn after_bind_flag(before: &str) -> bool {
    let s = before.strip_suffix(['"', '\'']).unwrap_or(before);
    let s = s.strip_suffix("tcp://").unwrap_or(s);
    if let Some(s) = s.strip_suffix('=') {
        let flag = s.rsplit(char::is_whitespace).next().unwrap_or(s);
        BIND_FLAGS.contains(&flag) || BIND_KEYS.contains(&flag)
    } else if s.ends_with(char::is_whitespace) {
        let flag = s
            .trim_end()
            .rsplit(char::is_whitespace)
            .next()
            .unwrap_or("");
        BIND_FLAGS.contains(&flag)
    } else {
        false
    }
}

/// The part of the command `before` is in (after the last `&&`, `||`,
/// `;` or `|`) runs a client of a server, whose address is never a bind.
pub(super) fn in_client(before: &str) -> bool {
    let part = before
        .rsplit(['&', '|', ';', '\n'])
        .next()
        .unwrap_or(before);
    let program = part.split_whitespace().next().unwrap_or("");
    let program = program.trim_matches(['"', '\'']);
    let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    let name = name.strip_suffix(".exe").unwrap_or(name);
    CLIENTS.contains(&name)
}

/// A variable of an action in LAN mode: a bind variable (`BIND_KEYS`) that
/// is 127.0.0.1, with or without a port, becomes 0.0.0.0; every other
/// value stays, since `DB_HOST=127.0.0.1` names a server to connect to
/// (BUG-V7-10).
pub(super) fn env_on_network(key: &str, value: String, lan: bool) -> String {
    if !lan || !BIND_KEYS.contains(&key) {
        return value;
    }
    match value.strip_prefix(LOCAL) {
        Some("") => "0.0.0.0".to_owned(),
        Some(port)
            if port
                .strip_prefix(':')
                .is_some_and(|p| p.bytes().all(|b| b.is_ascii_digit())) =>
        {
            format!("0.0.0.0{port}")
        }
        _ => value,
    }
}

impl Runner {
    pub(super) fn read_stream(&self, key: &str, run: u64, mut stream: Box<dyn Read + Send>) {
        let mut buf = vec![0u8; 16 * 1024];
        let mut pending: Vec<u8> = Vec::new();
        let mut rest = String::new();
        loop {
            let n = match stream.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            pending.extend_from_slice(&buf[..n]);
            let text = take_utf8(&mut pending);
            let lines = split_lines(&mut rest, &text);
            if !lines.is_empty() {
                self.append_to(key, Some(run), lines, true);
            }
        }
        let tail = String::from_utf8_lossy(&pending).into_owned() + &rest;
        let tail = tail.trim_end_matches(['\r', '\n']).to_owned();
        if !tail.is_empty() {
            self.append_to(key, Some(run), vec![tail], true);
        }
    }

    /// `scan`: look for the address a web server prints (not in the app's
    /// own lines, which may hold one in the command).
    /// A line of the kit (`[Atollo] …`) in the log of the run `key`.
    pub fn note(&self, key: &str, text: &str) {
        self.append(key, vec![format!("[Atollo] {text}")], false);
    }

    pub(super) fn append(&self, key: &str, lines: Vec<String>, scan: bool) {
        self.append_to(key, None, lines, scan);
    }

    /// Like [`Self::append`], only while the entry is still the one of
    /// `run` (`None`: whichever run it is).
    pub(super) fn append_to(&self, key: &str, run: Option<u64>, lines: Vec<String>, scan: bool) {
        let seq = {
            let mut entries = self.entries();
            let Some(e) = entries
                .get_mut(key)
                .filter(|e| run.is_none_or(|r| e.run == r))
            else {
                return;
            };
            e.lines.extend(lines.iter().cloned());
            if e.lines.len() > MAX_LINES {
                let extra = e.lines.len() - MAX_LINES;
                e.lines.drain(..extra);
            }
            if scan
                && e.info.kind == Kind::Web
                && e.info.state == State::Starting
                && e.actual_port.is_none()
            {
                if let Some(re) = LOCAL_URL.as_ref() {
                    for line in &lines {
                        let Some(c) = re.captures(line) else { continue };
                        let Some(p) = c.get(1).and_then(|m| m.as_str().parse::<u16>().ok()) else {
                            continue;
                        };
                        if Some(p) == e.info.port {
                            continue;
                        }
                        e.actual_port = Some(p);
                        let url = c[0].trim_end_matches(['.', ',', ';', ':', '!', '?']);
                        let url = ["0.0.0.0", "[::1]", "[::]", "127.0.0.1"]
                            .iter()
                            .fold(url.to_owned(), |u, h| u.replacen(h, "localhost", 1));
                        e.detected_url = Some(url);
                        break;
                    }
                }
            }
            self.number_lines(lines.len())
        };
        (self.notify)(Event::Log {
            key: key.to_owned(),
            lines,
            seq,
        });
    }
}
