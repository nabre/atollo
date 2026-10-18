use super::*;
use crate::agents::sync::run;
use crate::agents::sync::tests::kit;
use std::fs;

#[test]
fn servers_flow_between_claude_shared_and_others() {
    let (_t, kit, p) = kit("mcp");
    fs::create_dir_all(&p.claude).unwrap();
    fs::write(
        p.claude_json(),
        r#"{"mcpServers":{"fs":{"command":"npx","args":["-y","@x/fs"]},"web":{"type":"http","url":"https://e.x/mcp","headers":{"Authorization":"Bearer ${TOKEN}"}}},"other":1}"#,
    )
    .unwrap();
    let r = run(&kit, None);
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let shared = load(&p).unwrap();
    assert!(
        shared.contains_key("fs") && shared.contains_key("web"),
        "Claude's servers shared"
    );
    let agy: Value = serde_json::from_str(
        &fs::read_to_string(p.gemini.join("config").join("mcp_config.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(agy["mcpServers"]["fs"]["command"], "npx");
    assert_eq!(agy["mcpServers"]["web"]["serverUrl"], "https://e.x/mcp");
    let toml = fs::read_to_string(p.codex.join("config.toml")).unwrap();
    assert!(toml.contains("[mcp_servers.\"fs\"]"));
    assert!(toml.contains("bearer_token_env_var = \"TOKEN\""));

    // Removed from Claude: removed from the shared list and Antigravity.
    fs::write(p.claude_json(), r#"{"mcpServers":{"web":{"type":"http","url":"https://e.x/mcp","headers":{"Authorization":"Bearer ${TOKEN}"}}},"other":1}"#).unwrap();
    run(&kit, None);
    assert!(!load(&p).unwrap().contains_key("fs"));
    let agy: Value = serde_json::from_str(
        &fs::read_to_string(p.gemini.join("config").join("mcp_config.json")).unwrap(),
    )
    .unwrap();
    assert!(agy["mcpServers"].get("fs").is_none());
    let claude: Value =
        serde_json::from_str(&fs::read_to_string(p.claude_json()).unwrap()).unwrap();
    assert_eq!(claude["other"], 1, "Claude's other settings kept");
}

#[test]
fn antigravity_own_servers_left_alone() {
    let (_t, kit, p) = kit("mcp-agy");
    fs::create_dir_all(p.gemini.join("config")).unwrap();
    fs::write(
        p.gemini.join("config").join("mcp_config.json"),
        r#"{"mcpServers":{"mine":{"command":"x"}}}"#,
    )
    .unwrap();
    fs::create_dir_all(&p.agents).unwrap();
    fs::write(p.mcp_file(), r#"{"mcpServers":{"kit":{"command":"k"}}}"#).unwrap();
    run(&kit, None);
    let agy: Value = serde_json::from_str(
        &fs::read_to_string(p.gemini.join("config").join("mcp_config.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(agy["mcpServers"]["mine"]["command"], "x");
    assert_eq!(agy["mcpServers"]["kit"]["command"], "k");
}
