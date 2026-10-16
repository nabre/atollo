use super::*;
use crate::kit::tests::TempDir;

/// Microsoft Edge: embedded signature, present on Windows 10 and 11.
fn edge() -> Option<std::path::PathBuf> {
    ["ProgramFiles(x86)", "ProgramFiles"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .map(|p| Path::new(&p).join(r"Microsoft\Edge\Application\msedge.exe"))
        .find(|p| p.is_file())
}

#[test]
fn signed_by_microsoft() {
    let Some(edge) = edge() else { return };
    let verdict = check(
        &edge,
        &Regex::new("O=Microsoft Corporation").unwrap(),
        false,
    )
    .unwrap();
    let (Verdict::Valid { subject } | Verdict::RevocationUnknown { subject }) = verdict;
    assert!(subject.contains("Microsoft"), "{subject}");
    assert!(
        check(&edge, &Regex::new("O=Anthropic").unwrap(), false).is_err(),
        "wrong publisher"
    );
}

#[test]
fn unsigned_and_wrong_publisher_are_refused() {
    let tmp = TempDir::new("sig");
    let f = tmp.0.join("fake.exe");
    std::fs::write(&f, b"MZ not really").unwrap();
    assert!(check(&f, &Regex::new("Anthropic").unwrap(), false).is_err());
}
