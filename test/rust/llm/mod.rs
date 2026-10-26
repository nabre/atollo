use super::*;

#[test]
fn ids_are_lowercase_names() {
    assert_eq!(
        slug("Qwen3-Coder-30B-A3B-Instruct-Q4_K_M"),
        "qwen3-coder-30b-a3b-instruct-q4_k_m"
    );
    assert_eq!(slug("a b/c"), "a-b-c");
    assert_eq!(slug("--x.."), "x");
    assert!(valid_id("gpt-oss-20b"));
    assert!(!valid_id("Gpt"));
    assert!(!valid_id(""));
    assert!(!valid_id("a]b"));
}

/// A token file that cannot be read (here a folder in its place) is an
/// error for a repository of Hugging Face, and is left out for a download
/// that needs no token.
#[test]
fn an_unreadable_token_does_not_block_a_public_download() {
    let tmp = crate::kit::tests::TempDir::new("llm-token");
    let kit = Kit::at(tmp.0.clone(), crate::test_support::kit_arch());
    kit.prepare().unwrap();
    assert_eq!(hf_token_if_readable(&kit), None);
    std::fs::create_dir_all(hf_token_file(&kit)).unwrap();
    assert!(hf_token(&kit).is_err());
    assert_eq!(hf_token_if_readable(&kit), None);
    std::fs::remove_dir(hf_token_file(&kit)).unwrap();
    std::fs::write(hf_token_file(&kit), "hf_abc\n").unwrap();
    assert_eq!(hf_token_if_readable(&kit).as_deref(), Some("hf_abc"));
}
