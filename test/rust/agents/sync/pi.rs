use super::*;

use crate::kit::tests::TempDir;
use crate::kit::Arch;
use crate::llm::local::{write_manifest, Entry, Manifest};

fn kit() -> (TempDir, Kit) {
    let tmp = TempDir::new("sync-pi");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    (tmp, kit)
}

/// One complete model on disk, `id`, of the catalog's `catalog`.
fn add_model(kit: &Kit, id: &str, catalog: Option<&str>) {
    let dir = crate::llm::models_dir(kit).join("o--r");
    fs::create_dir_all(&dir).unwrap();
    let file = format!("{id}.gguf");
    fs::write(dir.join(&file), b"x").unwrap();
    let mut manifest = crate::llm::local::read_manifest(&dir).unwrap();
    manifest.models.push(Entry {
        id: id.into(),
        name: id.into(),
        repo: "o/r".into(),
        commit: "c".repeat(40),
        files: vec![crate::llm::catalog::ModelFile {
            name: file,
            sha256: "0".repeat(64),
            bytes: 1,
        }],
        catalog: catalog.map(str::to_owned),
        ctx: Some(8192),
        complete: true,
    });
    write_manifest(
        &dir,
        &Manifest {
            models: manifest.models,
        },
    )
    .unwrap();
}

#[test]
fn the_kit_block_keeps_the_users_text() {
    assert_eq!(with_block("", "rule"), format!("{BEGIN}\nrule\n{END}\n"));
    let mine = "My own rule.\n";
    let once = with_block(mine, "rule");
    assert!(once.starts_with("My own rule.\n\n"));
    let twice = with_block(&once, "new rule");
    assert!(twice.starts_with("My own rule.\n\n"));
    assert!(twice.contains("new rule") && !twice.contains("\nrule\n"));
    assert_eq!(twice.matches(BEGIN).count(), 1);
}

/// Without a choice and without a provider of the cloud, the first model
/// of the catalog's order; a choice wins; a provider of the cloud and no
/// choice: none.
#[test]
fn the_model_pi_uses() {
    let (_tmp, kit) = kit();
    assert_eq!(local_model(&kit), None);
    add_model(&kit, "mine", None);
    add_model(&kit, "gpt-oss-20b", Some("gpt-oss-20b"));
    add_model(&kit, "qwen2.5-coder-7b", Some("qwen2.5-coder-7b"));
    assert_eq!(local_model(&kit).as_deref(), Some("qwen2.5-coder-7b"));
    crate::llm::settings::update(&kit, |s| {
        s.pi_model = Some("mine".into());
        Ok(())
    })
    .unwrap();
    assert_eq!(local_model(&kit).as_deref(), Some("mine"));
    crate::llm::settings::update(&kit, |s| {
        s.pi_model = Some("gone".into());
        Ok(())
    })
    .unwrap();
    assert_eq!(local_model(&kit), None);
    crate::llm::settings::update(&kit, |s| {
        s.pi_model = None;
        Ok(())
    })
    .unwrap();
    let pi = dir(&kit.home());
    fs::create_dir_all(&pi).unwrap();
    fs::write(pi.join("auth.json"), r#"{"anthropic":{"type":"oauth"}}"#).unwrap();
    assert_eq!(local_model(&kit), None);
}

/// models.json gets the provider of the local models (every model, the
/// key, the server's port) and keeps the user's; with no model it goes.
#[test]
fn models_json_has_the_local_provider() {
    let (_tmp, kit) = kit();
    let p = Paths::of(&kit);
    fs::create_dir_all(&p.pi).unwrap();
    fs::write(
        p.pi.join("models.json"),
        r#"{"providers":{"ollama":{"baseUrl":"http://localhost:11434/v1"}}}"#,
    )
    .unwrap();
    add_model(&kit, "m1", None);
    sync_models(&kit, &p).unwrap();
    let cfg: Value = crate::fsx::read_json(&p.pi.join("models.json")).unwrap();
    let ours = &cfg["providers"][PROVIDER];
    assert_eq!(ours["baseUrl"], "http://127.0.0.1:9931/v1");
    assert_eq!(ours["api"], "openai-completions");
    assert_eq!(
        ours["apiKey"].as_str(),
        crate::llm::settings::key(&kit).as_deref()
    );
    assert_eq!(ours["models"][0]["id"], "m1");
    assert_eq!(ours["models"][0]["contextWindow"], 8192);
    assert!(cfg["providers"]["ollama"].is_object());
    // No model left: the provider goes, the user's stays.
    fs::remove_dir_all(crate::llm::models_dir(&kit)).unwrap();
    sync_models(&kit, &p).unwrap();
    let cfg: Value = crate::fsx::read_json(&p.pi.join("models.json")).unwrap();
    assert!(cfg["providers"].get(PROVIDER).is_none());
    assert!(cfg["providers"]["ollama"].is_object());
}

/// settings.json: the local model as the default while the user has no
/// provider of theirs; trust "ask" unless set; the user's own keys kept.
#[test]
fn settings_follow_the_local_model() {
    let (_tmp, kit) = kit();
    let p = Paths::of(&kit);
    fs::create_dir_all(&p.pi).unwrap();
    fs::write(
        p.pi.join("settings.json"),
        r#"{"theme":"dark","defaultProjectTrust":"never"}"#,
    )
    .unwrap();
    add_model(&kit, "m1", None);
    sync_settings(&kit, &p).unwrap();
    let cfg: Value = crate::fsx::read_json(&p.pi.join("settings.json")).unwrap();
    assert_eq!(cfg["defaultProvider"], PROVIDER);
    assert_eq!(cfg["defaultModel"], "m1");
    assert_eq!(cfg["defaultProjectTrust"], "never");
    assert_eq!(cfg["theme"], "dark");
    // The user chose another provider: left as it is.
    let mut cfg = cfg;
    cfg["defaultProvider"] = json!("anthropic");
    cfg["defaultModel"] = json!("claude-sonnet-5");
    write_json(&p.pi.join("settings.json"), &cfg).unwrap();
    sync_settings(&kit, &p).unwrap();
    let cfg: Value = crate::fsx::read_json(&p.pi.join("settings.json")).unwrap();
    assert_eq!(cfg["defaultProvider"], "anthropic");
    assert_eq!(cfg["defaultModel"], "claude-sonnet-5");
}

#[test]
fn a_fresh_settings_file_asks_for_trust() {
    let (_tmp, kit) = kit();
    let p = Paths::of(&kit);
    sync_settings(&kit, &p).unwrap();
    let cfg: Value = crate::fsx::read_json(&p.pi.join("settings.json")).unwrap();
    assert_eq!(cfg["defaultProjectTrust"], "ask");
    assert!(cfg.get("defaultProvider").is_none());
}

/// MCP: stdio and HTTP servers in Pi's format, SSE left out with a
/// warning, a server added in Pi kept.
#[test]
fn mcp_servers_in_pis_format() {
    let (_tmp, kit) = kit();
    let p = Paths::of(&kit);
    fs::create_dir_all(&p.pi).unwrap();
    fs::write(
        p.pi.join("mcp.json"),
        r#"{"mcpServers":{"own":{"command":"x"}}}"#,
    )
    .unwrap();
    let servers: Servers = serde_json::from_value(json!({
        "fs": { "command": "npx", "args": ["-y", "server-fs"], "env": { "A": "1" } },
        "docs": { "type": "http", "url": "https://example.com/mcp", "headers": { "X": "y" } },
        "old": { "type": "sse", "url": "https://example.com/sse" }
    }))
    .unwrap();
    let mut report = Report::default();
    sync_mcp(&p, &servers, &mut report).unwrap();
    let cfg: Value = crate::fsx::read_json(&p.pi.join("mcp.json")).unwrap();
    let list = &cfg["mcpServers"];
    assert_eq!(list["fs"]["command"], "npx");
    assert_eq!(list["fs"]["env"]["A"], "1");
    assert_eq!(list["docs"]["url"], "https://example.com/mcp");
    assert!(list["docs"].get("type").is_none());
    assert!(list.get("old").is_none());
    assert_eq!(list["own"]["command"], "x");
    assert!(report.warnings.iter().any(|w| w.contains("old")));
    // Removed from the kit: removed from Pi; the user's stays.
    sync_mcp(&p, &Servers::new(), &mut Report::default()).unwrap();
    let cfg: Value = crate::fsx::read_json(&p.pi.join("mcp.json")).unwrap();
    assert!(cfg["mcpServers"].get("fs").is_none());
    assert_eq!(cfg["mcpServers"]["own"]["command"], "x");
}

/// A project opened from Atollo is trusted; the user's own decision for it
/// stays.
#[test]
fn a_project_is_trusted_once() {
    let (_tmp, kit) = kit();
    let p = Paths::of(&kit);
    let project = kit.projects().join("shop");
    fs::create_dir_all(&project).unwrap();
    trust(&p, &project).unwrap();
    let cfg: Value = crate::fsx::read_json(&p.pi.join("trust.json")).unwrap();
    let key = trust_key(&project);
    assert_eq!(cfg[&key], true);
    assert!(!key.starts_with(r"\\?\"));
    let mut cfg = cfg;
    cfg[&key] = json!(false);
    write_json(&p.pi.join("trust.json"), &cfg).unwrap();
    trust(&p, &project).unwrap();
    let cfg: Value = crate::fsx::read_json(&p.pi.join("trust.json")).unwrap();
    assert_eq!(cfg[&key], false);
}
