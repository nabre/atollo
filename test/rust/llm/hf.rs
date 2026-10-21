use super::*;

use serde_json::json;

fn lfs(path: &str, size: u64, c: char) -> Value {
    json!({ "type": "file", "path": path, "size": size,
            "lfs": { "oid": c.to_string().repeat(64), "size": size } })
}

#[test]
fn specs_name_a_repository() {
    assert_eq!(
        parse_spec("unsloth/Qwen3-GGUF:Q4_K_M").unwrap(),
        ("unsloth/Qwen3-GGUF".into(), Some("Q4_K_M".into()))
    );
    assert_eq!(parse_spec(" a/b ").unwrap(), ("a/b".into(), None));
    assert!(parse_spec("nobody").is_err());
    assert!(parse_spec("a/../b").is_err());
}

#[test]
fn quantizations_in_file_names() {
    assert_eq!(
        quant_of("Qwen3-Coder-30B-A3B-Instruct-Q4_K_M").as_deref(),
        Some("Q4_K_M")
    );
    assert_eq!(quant_of("gpt-oss-20b-MXFP4").as_deref(), Some("MXFP4"));
    assert_eq!(
        quant_of("qwen2.5-coder-7b-instruct-q4_k_m").as_deref(),
        Some("Q4_K_M")
    );
    assert_eq!(quant_of("model-UD-IQ2_XXS").as_deref(), Some("UD-IQ2_XXS"));
    assert_eq!(quant_of("model-Q8_0").as_deref(), Some("Q8_0"));
    assert_eq!(quant_of("model-bf16").as_deref(), Some("BF16"));
    assert_eq!(quant_of("model"), None);
}

/// A tree becomes the models one can download: the parts of a model
/// together (all of them, or none), the projectors and the other files
/// left out, smallest first.
#[test]
fn a_tree_becomes_choices() {
    let tree = json!([
        { "type": "directory", "path": "Q8_0" },
        { "type": "file", "path": "README.md", "size": 10 },
        lfs("m-Q4_K_M.gguf", 400, 'a'),
        lfs("m-Q2_K.gguf", 200, 'b'),
        lfs("mmproj-F16.gguf", 50, 'c'),
        lfs("Q8_0/m-Q8_0-00001-of-00002.gguf", 300, 'd'),
        lfs("Q8_0/m-Q8_0-00002-of-00002.gguf", 300, 'e'),
        lfs("BF16/m-BF16-00001-of-00003.gguf", 300, 'f'),
        { "type": "file", "path": "no-hash.gguf", "size": 5 },
        lfs("bad name.gguf", 1, 'g'),
    ]);
    let c = choices(&tree);
    let ids: Vec<&str> = c.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["m-q2_k", "m-q4_k_m", "m-q8_0"]);
    let q8 = &c[2];
    assert_eq!(q8.bytes, 600);
    assert_eq!(q8.quant.as_deref(), Some("Q8_0"));
    assert_eq!(q8.files[0].name, "Q8_0/m-Q8_0-00001-of-00002.gguf");
    assert_eq!(q8.files[1].sha256, "e".repeat(64));
}

#[test]
fn a_choice_is_picked_by_its_quantization() {
    let tree = json!([lfs("m-Q4_K_M.gguf", 4, 'a'), lfs("m-Q8_0.gguf", 8, 'b')]);
    let info = RepoInfo {
        repo: "o/m-GGUF".into(),
        commit: "c".repeat(40),
        license: None,
        choices: choices(&tree),
    };
    assert_eq!(pick(&info, Some("q8_0")).unwrap().id, "m-q8_0");
    assert_eq!(pick(&info, Some("m-q4_k_m")).unwrap().id, "m-q4_k_m");
    let err = pick(&info, None).unwrap_err().to_string();
    assert!(err.contains("Q4_K_M, Q8_0"), "{err}");
    assert!(pick(&info, Some("Q2_K")).is_err());
}

#[test]
fn file_addresses_are_at_the_commit() {
    assert_eq!(
        file_url("o/r", "abc", "Q8_0/m Q8.gguf"),
        "https://huggingface.co/o/r/resolve/abc/Q8_0/m%20Q8.gguf"
    );
}
