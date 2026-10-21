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
    let verdict = check(&edge, "Microsoft Corporation", false).unwrap();
    let (Verdict::Valid { subject } | Verdict::RevocationUnknown { subject }) = verdict;
    assert!(subject.contains("Microsoft"), "{subject}");
    // The organization exactly: a part of it, or of the subject, is not
    // enough (SEC-V6-03).
    for wrong in [
        "Anthropic, PBC",
        "Microsoft",
        "O=Microsoft Corporation",
        "Redmond",
    ] {
        assert!(check(&edge, wrong, false).is_err(), "{wrong}");
    }
}

#[test]
fn unsigned_and_wrong_publisher_are_refused() {
    let tmp = TempDir::new("sig");
    let f = tmp.0.join("fake.exe");
    std::fs::write(&f, b"MZ not really").unwrap();
    assert!(check(&f, "Anthropic, PBC", false).is_err());
}

/// A certificate with the publisher's name in another attribute (`OU=`,
/// `CN=`) has another organization: refused (SEC-V6-03).
#[test]
fn the_organization_must_be_the_publisher() {
    assert!(is_publisher(
        Some("Microsoft Corporation"),
        "Microsoft Corporation"
    ));
    assert!(!is_publisher(Some("Evil Ltd"), "Microsoft Corporation"));
    assert!(!is_publisher(
        Some("Microsoft Corporation Evil"),
        "Microsoft Corporation"
    ));
    assert!(!is_publisher(Some("Anthropic, PBC"), "Anthropic"));
    assert!(!is_publisher(None, "Anthropic, PBC"));
    assert!(!is_publisher(Some(""), ""));
}
