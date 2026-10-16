use super::*;
use crate::kit::tests::TempDir;
use std::fs;

fn stdio(name: &str) -> McpServer {
    McpServer {
        name: name.into(),
        kind: "stdio".into(),
        command: Some("npx".into()),
        args: vec![
            "-y".into(),
            "@modelcontextprotocol/server-filesystem".into(),
            ".".into(),
        ],
        ..Default::default()
    }
}

#[test]
fn servers_in_mcp_json_and_codex() {
    let tmp = TempDir::new("project-mcp");
    let d = tmp.0.join("p");
    fs::create_dir_all(d.join(".codex")).unwrap();
    fs::write(d.join(".codex").join("config.toml"), "model = \"x\"\n").unwrap();
    fs::write(
        d.join(".mcp.json"),
        r#"{"other":1,"mcpServers":{"fs":{"command":"old","timeout":30}}}"#,
    )
    .unwrap();
    assert_eq!(get(&d).servers[0].command.as_deref(), Some("old"));
    let web = McpServer {
        name: "docs".into(),
        kind: "http".into(),
        url: Some("https://example.com/mcp".into()),
        ..Default::default()
    };
    set(&d, &[stdio("fs"), web.clone()]).unwrap();
    let o: Value = serde_json::from_str(&fs::read_to_string(d.join(".mcp.json")).unwrap()).unwrap();
    assert_eq!(o["other"], 1, "other keys stay");
    assert_eq!(o["mcpServers"]["fs"]["command"], "npx");
    assert_eq!(
        o["mcpServers"]["fs"]["timeout"], 30,
        "keys of a server stay"
    );
    assert_eq!(o["mcpServers"]["docs"]["type"], "http");
    let toml = fs::read_to_string(d.join(".codex").join("config.toml")).unwrap();
    assert!(toml.starts_with("model = \"x\"\n"), "{toml}");
    assert!(
        toml.contains("[mcp_servers.\"fs\"]") || toml.contains("[mcp_servers.fs]"),
        "{toml}"
    );
    assert_eq!(get(&d).servers.len(), 2);
    // Removed: the Codex block goes, the user's line stays.
    set(&d, &[]).unwrap();
    let toml = fs::read_to_string(d.join(".codex").join("config.toml")).unwrap();
    assert_eq!(toml, "model = \"x\"\n");
    let o: Value = serde_json::from_str(&fs::read_to_string(d.join(".mcp.json")).unwrap()).unwrap();
    assert!(o.get("mcpServers").is_none());
    // Refused.
    assert!(set(
        &d,
        &[McpServer {
            name: "a b".into(),
            ..stdio("x")
        }]
    )
    .is_err());
    assert!(set(&d, &[stdio("x"), stdio("x")]).is_err());
    assert!(set(
        &d,
        &[McpServer {
            url: Some("ftp://x".into()),
            ..web
        }]
    )
    .is_err());
}
