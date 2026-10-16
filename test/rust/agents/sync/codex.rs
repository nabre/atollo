use super::*;
use crate::agents::sync::tests::kit;
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
