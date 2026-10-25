use super::*;

use std::collections::BTreeSet;
use std::path::Path;

use regex::Regex;

/// The interface subscribes with the names Rust emits: the constants of
/// `ui/src/ipc/events.ts` are the same, by name and by value, as
/// [`ALL`], and the output of a session has the same prefix.
#[test]
fn the_interface_has_the_same_names() {
    let ts = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("ui")
        .join("src")
        .join("ipc")
        .join("events.ts");
    let text = std::fs::read_to_string(&ts).unwrap();
    let re = Regex::new(r#"(?m)^export const ([A-Z_]+) = "([^"]+)";"#).unwrap();
    let theirs: BTreeSet<String> = re
        .captures_iter(&text)
        .map(|c| format!("{}={}", &c[1], &c[2]))
        .collect();
    let rs = include_str!("../../../tauri/src/ipc/events.rs");
    let re = Regex::new(r#"(?m)^pub const ([A-Z_]+): &str = "([^"]+)";"#).unwrap();
    let ours: BTreeSet<String> = re
        .captures_iter(rs)
        .map(|c| format!("{}={}", &c[1], &c[2]))
        .collect();
    assert_eq!(ours.len(), ALL.len(), "every constant is in ALL");
    assert_eq!(ours, theirs);
    assert!(text.contains("return `pty://${String(id)}`;"));
    assert_eq!(pty_output(7), "pty://7");
}

/// No two events share a name.
#[test]
fn names_are_distinct() {
    let set: BTreeSet<_> = ALL.iter().collect();
    assert_eq!(set.len(), ALL.len());
}
