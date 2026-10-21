use super::*;

use crate::kit::tests::TempDir;
use crate::kit::Arch;

#[test]
fn devices_of_llama_server() {
    let text = "ggml_vulkan: Found 1 Vulkan devices:\nAvailable devices:\n  Vulkan0: NVIDIA GeForce RTX 2070 (8220 MiB, 7451 MiB free)\n  MTL0: Apple M2 Pro (21845 MiB, 21845 MiB free)\n";
    assert_eq!(
        parse_devices(text),
        [
            Gpu {
                name: "NVIDIA GeForce RTX 2070".into(),
                memory: 8220 * 1024 * 1024
            },
            Gpu {
                name: "Apple M2 Pro".into(),
                memory: 21845 * 1024 * 1024
            }
        ]
    );
    assert!(parse_devices("Available devices:\n  (none)\n").is_empty());
}

#[test]
fn loaded_models_of_the_router() {
    let models = serde_json::json!({ "data": [
        { "id": "a", "status": { "value": "unloaded" } },
        { "id": "b", "status": { "value": "loaded" } },
        { "id": "c" }
    ]});
    assert_eq!(loaded_in(&models), ["b"]);
}

/// Only 127.0.0.1, the key from its file, never on the command line, one
/// model at a time, released when idle.
#[test]
fn the_server_listens_on_this_pc_only() {
    let tmp = TempDir::new("llm-args");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    let a = args(&kit, 9931);
    let after = |flag: &str| a[a.iter().position(|x| x == flag).unwrap() + 1].clone();
    assert_eq!(after("--host"), "127.0.0.1");
    assert_eq!(after("--port"), "9931");
    assert_eq!(after("--models-max"), "1");
    assert_eq!(
        after("--api-key-file"),
        settings::key_file(&kit).display().to_string()
    );
    assert!(!a.iter().any(|x| x == "--api-key"));
    assert_eq!(
        after("--models-preset"),
        crate::llm::presets::path(&kit).display().to_string()
    );
}

/// Without llama.cpp, or without a model, it says what to do instead of
/// starting.
#[test]
fn a_start_needs_the_server_and_a_model() {
    let tmp = TempDir::new("llm-start");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let llm = Llm::new(kit.clone());
    let err = llm.start(&KitEnv::default(), "it").unwrap_err();
    assert!(err.to_string().contains("llama.cpp"), "{err:#}");
    assert_eq!(llm.running(), 0);
    let info = llm.status();
    assert_eq!(info.state, State::Error);
    assert!(!info.installed);
    assert_eq!(info.url, "http://127.0.0.1:9931");
    llm.stop();
    assert_eq!(llm.status().state, State::Stopped);
}
