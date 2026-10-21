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
