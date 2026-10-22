use super::*;

use std::collections::BTreeMap;
use std::io::Read;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::{Arc, Mutex};

/// One request the fake server received.
#[derive(Debug, Clone)]
struct Seen {
    method: String,
    headers: BTreeMap<String, String>,
    body: String,
}

/// A fake MCP server (Streamable HTTP): `initialize` with a session, a
/// notification accepted (202), `tools/list` as a stream of events,
/// `tools/call` failing (500), then the session closed (DELETE).
fn fake_server() -> (String, Arc<Mutex<Vec<Seen>>>) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let url = format!("http://{}/mcp/", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut first = String::new();
            reader.read_line(&mut first).unwrap();
            let method = first.split(' ').next().unwrap_or_default().to_owned();
            let mut headers = BTreeMap::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let line = line.trim_end();
                if line.is_empty() {
                    break;
                }
                let (k, v) = line.split_once(':').unwrap();
                headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_owned());
            }
            let len: usize = headers
                .get("content-length")
                .map_or(0, |l| l.parse().unwrap());
            let mut body = vec![0; len];
            reader.read_exact(&mut body).unwrap();
            let body = String::from_utf8(body).unwrap();
            let (status, extra, content) = if method == "DELETE" {
                ("200 OK", "", String::new())
            } else if body.contains("\"initialize\"") {
                (
                    "200 OK",
                    "Content-Type: application/json\r\nMcp-Session-Id: s-1\r\n",
                    r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-06-18"}}"#
                        .to_owned(),
                )
            } else if body.contains("notifications/initialized") {
                ("202 Accepted", "", String::new())
            } else if body.contains("tools/list") {
                (
                    "200 OK",
                    "Content-Type: text/event-stream\r\n",
                    ": a comment\r\nevent: message\r\ndata: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\"}\r\n\r\nid: 7\ndata: {\"jsonrpc\":\"2.0\",\ndata: \"id\":2,\"result\":{\"tools\":[]}}\n\n"
                        .to_owned(),
                )
            } else {
                ("500 Internal Server Error", "", "boom".to_owned())
            };
            let done = method == "DELETE";
            log.lock().unwrap().push(Seen {
                method,
                headers,
                body,
            });
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\n{extra}Content-Length: {}\r\nConnection: close\r\n\r\n{content}",
                content.len()
            );
            if done {
                break;
            }
        }
    });
    (url, seen)
}

/// SEC-V7-01: the agent talks stdio, the bridge adds the token on its way to
/// GitHub only, keeps the session, turns a stream into lines and answers a
/// failed request with an error instead of silence.
#[test]
fn bridge_carries_messages_with_the_token() {
    let (url, seen) = fake_server();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .proxy(None)
        .build()
        .new_agent();
    let input = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        "",
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        "not json",
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"x"}}"#,
    ]
    .join("\n");
    let mut out = Vec::new();
    bridge(&agent, &url, "t0k3n", input.as_bytes(), &mut out).unwrap();
    let lines: Vec<Value> = String::from_utf8(out)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 5, "{lines:?}");
    assert_eq!(lines[0]["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(lines[1]["method"], "notifications/progress");
    assert_eq!(lines[2]["id"], 2);
    assert_eq!(lines[2]["result"]["tools"], json!([]));
    assert_eq!(lines[3]["error"]["code"], -32700);
    assert_eq!(lines[4]["id"], 3);
    assert!(
        lines[4]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("HTTP 500"),
        "{}",
        lines[4]
    );
    // The token never reaches the agent's side.
    assert!(!lines.iter().any(|l| l.to_string().contains("t0k3n")));

    let seen = seen.lock().unwrap().clone();
    let methods: Vec<&str> = seen.iter().map(|s| s.method.as_str()).collect();
    assert_eq!(methods, ["POST", "POST", "POST", "POST", "DELETE"]);
    for s in &seen {
        assert_eq!(s.headers["authorization"], "Bearer t0k3n", "{s:?}");
    }
    assert!(!seen[0].headers.contains_key("mcp-session-id"));
    assert!(seen[0].headers["accept"].contains("text/event-stream"));
    for s in &seen[1..] {
        assert_eq!(s.headers["mcp-session-id"], "s-1", "{s:?}");
    }
    for s in &seen[1..4] {
        assert_eq!(s.headers["mcp-protocol-version"], "2025-06-18", "{s:?}");
    }
    assert!(seen[3].body.contains("tools/call"));
}

/// Events of a stream: `data:` lines joined, the rest ignored, the last
/// event given even without the blank line that ends it.
#[test]
fn events_of_a_stream() {
    let text = "event: message\r\ndata: a\r\ndata:b\r\n\r\n: ping\n\nretry: 5\ndata: c";
    let mut got = Vec::new();
    events(text.as_bytes(), |d| {
        got.push(d);
        Ok(())
    })
    .unwrap();
    assert_eq!(got, ["a\nb", "c"]);
}
