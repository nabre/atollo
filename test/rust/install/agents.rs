use super::*;
use crate::kit::Target;

/// Antigravity 1.4: a `.tar.gz` with one binary named `antigravity` on
/// macOS and Linux, the `.exe` itself on Windows (Google's manifests of
/// 18/10/2026).
#[test]
fn antigravity_download_by_url() {
    let base =
        "https://storage.googleapis.com/antigravity-public/antigravity-cli/1.4.2-4835981233356800";
    for (url, target) in [
        (
            format!("{base}/linux-x64/cli_linux_x64.tar.gz"),
            Target::LINUX_X64,
        ),
        (
            format!("{base}/darwin-arm/cli_mac_arm64.tar.gz"),
            Target::DARWIN_ARM64,
        ),
    ] {
        assert_eq!(
            antigravity_download(&url, "1.4.2", target),
            (
                Kind::TarGz,
                "agy-1.4.2.tar.gz".to_owned(),
                "antigravity".to_owned()
            ),
            "{url}"
        );
    }
    let exe = format!("{base}/windows-x64/cli_windows_x64.exe");
    assert_eq!(
        antigravity_download(&exe, "1.4.2", Target::WIN32_X64),
        (
            Kind::File,
            "agy-1.4.2.exe".to_owned(),
            "agy-1.4.2.exe".to_owned()
        )
    );
    // A plain binary on Unix (the format before 1.4) is still taken as is.
    assert_eq!(
        antigravity_download(&format!("{base}/linux-x64/agy"), "1.4.2", Target::LINUX_X64),
        (Kind::File, "agy-1.4.2".to_owned(), "agy-1.4.2".to_owned())
    );
}

/// Pi 1.3 publishes one archive per target: a zip on Windows, a tar.gz
/// elsewhere (release v1.3.0 of earendil-works/pi).
#[test]
fn pi_archive_per_target() {
    let names = [
        "pi-windows-x64.zip",
        "pi-windows-arm64.zip",
        "pi-darwin-arm64.tar.gz",
        "pi-darwin-x64.tar.gz",
        "pi-linux-x64.tar.gz",
        "pi-linux-arm64.tar.gz",
        "pi-1.3.0-source.tar.gz",
        "SHA256SUMS",
    ];
    for target in Target::ALL {
        let (re, kind) = pi_asset(target);
        let re = regex::Regex::new(&re).unwrap();
        let found: Vec<&str> = names.iter().copied().filter(|n| re.is_match(n)).collect();
        assert_eq!(found.len(), 1, "{target}: {found:?}");
        assert_eq!(kind == Kind::Zip, target.is_windows(), "{target}");
    }
}
