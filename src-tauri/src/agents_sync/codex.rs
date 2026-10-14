//! Codex `config.toml`: kit blocks at the top (settings) and at the end (MCP
//! servers, trusted folders); the user's text in between is kept as it is
//! (port of `codex.js`, same block markers, so files of nabre/dev-kit read
//! the same).

use std::collections::BTreeSet;
use std::fs;

use anyhow::Result;
use serde_json::Value;

use super::mcp::{server_type, Servers};
use super::{trusted_folders, Paths, Report, KIT_RULE_TEXT};
use crate::kit::write_atomic;

const TOP_BEGIN: &str = "# >>> Dev-Kit: impostazioni del kit (non modificare questo blocco) >>>";
const TOP_END: &str = "# <<< Dev-Kit <<<";
const MCP_BEGIN: &str = "# >>> Dev-Kit: server MCP da data/agents/mcp.json (non modificare: cambia quel file o usa \"claude mcp\") >>>";
const MCP_END: &str = "# <<< Dev-Kit: server MCP <<<";
const TRUST_BEGIN: &str =
    "# >>> Dev-Kit: cartelle attendibili (projects del kit, riscritte a ogni avvio) >>>";
const TRUST_END: &str = "# <<< Dev-Kit: cartelle attendibili <<<";

/// A JSON value as TOML (JSON strings, numbers and booleans are TOML too).
fn toml(v: &Value) -> String {
    match v {
        Value::Array(a) => format!("[{}]", a.iter().map(toml).collect::<Vec<_>>().join(", ")),
        Value::Object(o) => format!(
            "{{ {} }}",
            o.iter()
                .map(|(k, x)| format!("{} = {}", Value::String(k.clone()), toml(x)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => other.to_string(),
    }
}

fn strip_block(text: &str, begin: &str, end: &str) -> String {
    let Some(i) = text.find(begin) else {
        return text.to_owned();
    };
    let Some(j) = text[i..].find(end).map(|j| j + i) else {
        return text.to_owned();
    };
    let rest = &text[j + end.len()..];
    let rest = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))
        .unwrap_or(rest);
    format!("{}{rest}", &text[..i])
}

fn one_line(s: &str) -> String {
    s.replace(['\r', '\n'], " ")
}

fn str_of(v: &Value) -> String {
    v.as_str().map_or_else(|| v.to_string(), str::to_owned)
}

fn has_var(s: &str) -> bool {
    s.find("${").is_some_and(|i| s[i..].contains('}'))
}

/// One server as a Codex table (`None` if Codex cannot use it). Codex never
/// expands `${VAR}`: translated where Codex has a key for it, otherwise
/// copied literally with a warning comment (no secret is read or written).
pub fn codex_server(name: &str, s: &Value) -> Option<String> {
    let mut lines = vec![format!("[mcp_servers.{}]", Value::String(name.to_owned()))];
    let mut literal: Vec<&str> = Vec::new();
    let check = |field: &'static str, v: &str, literal: &mut Vec<&str>| {
        if has_var(v) && !literal.contains(&field) {
            literal.push(field);
        }
    };
    let ty = server_type(s);
    if ty == "stdio" {
        let command = s.get("command")?;
        lines.push(format!("command = {}", toml(command)));
        check("command", &str_of(command), &mut literal);
        if let Some(args) = s
            .get("args")
            .and_then(Value::as_array)
            .filter(|a| !a.is_empty())
        {
            let args: Vec<Value> = args.iter().map(|a| Value::String(str_of(a))).collect();
            lines.push(format!("args = {}", toml(&Value::Array(args.clone()))));
            for a in &args {
                check("args", &str_of(a), &mut literal);
            }
        }
        if let Some(cwd) = s.get("cwd") {
            lines.push(format!("cwd = {}", toml(cwd)));
            check("cwd", &str_of(cwd), &mut literal);
        }
        let mut env = serde_json::Map::new();
        let mut pass = Vec::new();
        for (k, v) in s
            .get("env")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            let v = str_of(v);
            if v == format!("${{{k}}}") {
                pass.push(Value::String(k.clone()));
            } else {
                check("env", &v, &mut literal);
                env.insert(k.clone(), Value::String(v));
            }
        }
        if !env.is_empty() {
            lines.push(format!("env = {}", toml(&Value::Object(env))));
        }
        if !pass.is_empty() {
            lines.push(format!("env_vars = {}", toml(&Value::Array(pass))));
        }
    } else if ty == "http" || ty == "streamable-http" {
        let url = s.get("url")?;
        lines.push(format!("url = {}", toml(url)));
        check("url", &str_of(url), &mut literal);
        let mut headers = serde_json::Map::new();
        let mut from_env = serde_json::Map::new();
        for (k, v) in s
            .get("headers")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            let v = str_of(v);
            let whole = v
                .strip_prefix("${")
                .and_then(|r| r.strip_suffix('}'))
                .filter(|n| n.chars().all(|c| c.is_alphanumeric() || c == '_'));
            let bearer = v
                .strip_prefix("Bearer ${")
                .and_then(|r| r.strip_suffix('}'))
                .filter(|n| n.chars().all(|c| c.is_alphanumeric() || c == '_'));
            if let (Some(var), true) = (bearer, k.eq_ignore_ascii_case("authorization")) {
                lines.push(format!(
                    "bearer_token_env_var = {}",
                    Value::String(var.to_owned())
                ));
            } else if let Some(var) = whole {
                from_env.insert(k.clone(), Value::String(var.to_owned()));
            } else {
                check("headers", &v, &mut literal);
                headers.insert(k.clone(), Value::String(v));
            }
        }
        if !headers.is_empty() {
            lines.push(format!("http_headers = {}", toml(&Value::Object(headers))));
        }
        if !from_env.is_empty() {
            lines.push(format!(
                "env_http_headers = {}",
                toml(&Value::Object(from_env))
            ));
        }
    } else {
        return None; // sse: not supported by Codex
    }
    if !literal.is_empty() {
        lines.insert(
            0,
            format!(
                "# {}: Codex does not expand ${{...}} in {}: the text stays literal",
                one_line(name),
                literal.join(", ")
            ),
        );
    }
    Some(lines.join("\n"))
}

/// Top-level keys of a TOML text: the ones before the first table, outside
/// multi-line strings (enough for the keys the kit sets).
pub fn top_level_keys(text: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut open: Option<&str> = None;
    for line in text.lines() {
        if open.is_none() {
            if line.trim_start().starts_with('[') {
                break;
            }
            let t = line.trim_start();
            if let Some(eq) = t.find('=') {
                let k = t[..eq].trim();
                let k = k.trim_matches('"').trim_matches('\'');
                if !k.is_empty()
                    && !k.starts_with('#')
                    && k.chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
                {
                    keys.insert(k.to_owned());
                }
            }
        }
        let mut rest = line;
        while let Some(i) = rest
            .find("\"\"\"")
            .into_iter()
            .chain(rest.find("'''"))
            .min()
        {
            let q = &rest[i..i + 3];
            open = match open {
                None => Some(if q == "\"\"\"" { "\"\"\"" } else { "'''" }),
                Some(o) if o == q => None,
                o => o,
            };
            rest = &rest[i + 3..];
        }
    }
    keys
}

pub fn sync_config(p: &Paths, servers: &Servers, report: &mut Report) -> Result<()> {
    let file = p.codex.join("config.toml");
    let old = match fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let mut user = old.clone();
    for (b, e) in [
        (TOP_BEGIN, TOP_END),
        (MCP_BEGIN, MCP_END),
        (TRUST_BEGIN, TRUST_END),
        ("# >>> Claude Portable: impostazioni del kit (non modificare questo blocco) >>>", "# <<< Claude Portable <<<"),
        (
            "# >>> Claude Portable: server MCP da data/agents/mcp.json (non modificare: cambia quel file o usa \"claude mcp\") >>>",
            "# <<< Claude Portable: server MCP <<<",
        ),
    ] {
        user = strip_block(&user, b, e);
    }
    let user = user.trim().to_owned();
    let user_keys = top_level_keys(&user);
    let top: Vec<String> = [
        (
            "project_doc_fallback_filenames",
            "[\"CLAUDE.md\", \"GEMINI.md\"]".to_owned(),
        ),
        (
            "developer_instructions",
            Value::String(KIT_RULE_TEXT.to_owned()).to_string(),
        ),
        ("cli_auth_credentials_store", "\"file\"".to_owned()),
        ("check_for_update_on_startup", "false".to_owned()),
        // A login shell would load the PC's PowerShell profiles (no
        // -NoProfile): only the kit's shell, without them.
        ("allow_login_shell", "false".to_owned()),
    ]
    .into_iter()
    .filter(|(k, _)| !user_keys.contains(*k))
    .map(|(k, v)| format!("{k} = {v}"))
    .collect();

    let mut tables = Vec::new();
    for (n, s) in servers {
        let quoted = format!("[mcp_servers.{}]", Value::String(n.clone()));
        let bare = format!("[mcp_servers.{n}]");
        if user.lines().any(|l| l.trim() == quoted || l.trim() == bare) {
            continue; // defined by hand in config.toml: that one wins
        }
        match codex_server(n, s) {
            Some(t) => tables.push(t),
            None => {
                let ty = server_type(s);
                let missing = if ty == "stdio" {
                    Some("command")
                } else if ty == "http" || ty == "streamable-http" {
                    Some("url")
                } else {
                    None
                };
                tables.push(match missing {
                    Some(m) => format!("# {}: missing \"{m}\"", one_line(n)),
                    None => format!(
                        "# {}: not supported by Codex (type \"{}\")",
                        one_line(n),
                        one_line(&ty)
                    ),
                });
            }
        }
    }

    // Trusted folders, keyed by the lower-case path. A table Codex wrote for
    // the same folder is kept (a second one would be invalid TOML).
    let user_lower = user.to_lowercase();
    let mut trust = Vec::new();
    for f in trusted_folders(p) {
        let path = f.display().to_string().to_lowercase();
        let literal = format!("'{path}'");
        let basic = Value::String(path.clone()).to_string();
        if user_lower.contains(&format!("[projects.{literal}]"))
            || user_lower.contains(&format!("[projects.{}]", basic.to_lowercase()))
        {
            continue;
        }
        let key = if path.contains('\'') { basic } else { literal };
        trust.push(format!("[projects.{key}]\ntrust_level = \"trusted\""));
    }

    let mut text = format!(
        "{TOP_BEGIN}\n{}{}{TOP_END}\n",
        top.join("\n"),
        if top.is_empty() { "" } else { "\n" }
    );
    if !user.is_empty() {
        text.push_str(&format!("\n{user}\n"));
    }
    if !tables.is_empty() {
        text.push_str(&format!(
            "\n{MCP_BEGIN}\n{}\n{MCP_END}\n",
            tables.join("\n\n")
        ));
    }
    if !trust.is_empty() {
        text.push_str(&format!(
            "\n{TRUST_BEGIN}\n{}\n{TRUST_END}\n",
            trust.join("\n\n")
        ));
    }
    if text.replace("\r\n", "\n") != old.replace("\r\n", "\n") {
        fs::create_dir_all(&p.codex)?;
        write_atomic(&file, text.as_bytes())?;
        report.note("data\\home\\.codex\\config.toml updated");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents_sync::tests::kit;
    use serde_json::json;

    #[test]
    fn servers_as_tables() {
        let t = codex_server("fs", &json!({"command": "npx", "args": ["-y", 1], "env": {"A": "x", "B": "${B}", "C": "${SECRET}"}})).unwrap();
        assert!(t.contains("[mcp_servers.\"fs\"]"));
        assert!(t.contains("args = [\"-y\", \"1\"]"));
        assert!(t.contains("env_vars = [\"B\"]"));
        assert!(t.starts_with("# fs: Codex does not expand"), "{t}");
        let h = codex_server("w", &json!({"url": "https://x", "headers": {"Authorization": "Bearer ${TOK}", "X-Key": "${KEY}", "Y": "plain"}})).unwrap();
        assert!(h.contains("bearer_token_env_var = \"TOK\""));
        assert!(h.contains("env_http_headers = { \"X-Key\" = \"KEY\" }"));
        assert!(h.contains("http_headers = { \"Y\" = \"plain\" }"));
        assert!(codex_server("s", &json!({"type": "sse", "url": "x"})).is_none());
    }

    #[test]
    fn user_text_and_keys_kept() {
        let (_t, _kit, p) = kit("codex");
        fs::create_dir_all(&p.codex).unwrap();
        fs::create_dir_all(p.projects.join("app")).unwrap();
        fs::write(p.codex.join("config.toml"), "model = \"o9\"\ncheck_for_update_on_startup = true\n\n[mcp_servers.mine]\ncommand = \"m\"\n").unwrap();
        let mut servers = Servers::new();
        servers.insert("mine".into(), json!({"command": "kit"}));
        servers.insert("kit".into(), json!({"command": "k"}));
        let mut r = Report::default();
        sync_config(&p, &servers, &mut r).unwrap();
        let text = fs::read_to_string(p.codex.join("config.toml")).unwrap();
        assert!(text.starts_with(TOP_BEGIN));
        assert!(
            !text.contains("check_for_update_on_startup = false"),
            "user's key wins"
        );
        assert!(text.contains("cli_auth_credentials_store = \"file\""));
        assert!(text.contains("model = \"o9\""));
        assert!(text.contains("[mcp_servers.\"kit\"]"));
        assert!(!text.contains("command = \"kit\""), "hand-made table wins");
        let app = p.projects.join("app").display().to_string().to_lowercase();
        assert!(text.contains(&format!("[projects.'{app}']")));
        // Stable: a second run changes nothing.
        sync_config(&p, &servers, &mut r).unwrap();
        assert_eq!(
            fs::read_to_string(p.codex.join("config.toml")).unwrap(),
            text
        );
    }

    #[test]
    fn keys_before_tables_only() {
        let keys = top_level_keys("a = 1\nb = \"\"\"\nc = 2\n\"\"\"\n[t]\nd = 3\n");
        assert!(keys.contains("a") && keys.contains("b"));
        assert!(!keys.contains("c") && !keys.contains("d"));
    }
}
