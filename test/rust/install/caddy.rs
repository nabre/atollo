use super::*;

use crate::setup::catalog::{translate_rel, Catalog};

/// The archives of Caddy 2.11.7 for the four targets, as published on
/// GitHub (checked with `gh api repos/caddyserver/caddy/releases/latest`
/// on 17/10/2026).
const ASSETS_2_11_7: [(Target, &str); 4] = [
    (Target::WIN32_X64, "caddy_2.11.7_windows_amd64.zip"),
    (Target::DARWIN_ARM64, "caddy_2.11.7_mac_arm64.tar.gz"),
    (Target::LINUX_X64, "caddy_2.11.7_linux_amd64.tar.gz"),
    (Target::LINUX_ARM64, "caddy_2.11.7_linux_arm64.tar.gz"),
];

/// Other assets of the same release that must never be taken (the
/// Windows ARM64 and Intel Mac builds too, D-176).
const DECOYS: [&str; 11] = [
    "caddy_2.11.7_windows_arm64.zip",
    "caddy_2.11.7_mac_amd64.tar.gz",
    "caddy_2.11.7_linux_amd64.deb",
    "caddy_2.11.7_linux_amd64.tar.gz.sig",
    "caddy_2.11.7_linux_amd64.sbom",
    "caddy_2.11.7_linux_armv7.tar.gz",
    "caddy_2.11.7_freebsd_amd64.tar.gz",
    "caddy_2.11.7_windows_amd64.zip.sig",
    "caddy_2.11.7_buildable-artifact.tar.gz",
    "caddy_2.11.7_checksums.txt",
    "caddy_2.11.7_src.tar.gz",
];

fn asset(name: &str, digest: Option<&str>) -> GhAsset {
    GhAsset {
        name: name.to_owned(),
        browser_download_url: format!("https://example.invalid/{name}"),
        digest: digest.map(str::to_owned),
    }
}

/// A release of `version` with every archive and decoy; `digest` gives the
/// archives GitHub's SHA-256, `checksums` adds the checksum file.
fn release(version: &str, prerelease: bool, digest: bool, checksums: bool) -> GhRelease {
    let d = format!("sha256:{}", "a".repeat(64));
    let mut assets: Vec<GhAsset> = ASSETS_2_11_7
        .iter()
        .map(|(_, n)| n.replace("2.11.7", version))
        .chain(DECOYS.iter().map(|n| n.replace("2.11.7", version)))
        .filter(|n| !n.ends_with("_checksums.txt"))
        .map(|n| asset(&n, digest.then_some(d.as_str())))
        .collect();
    if checksums {
        assets.push(asset(&caddy_checksums_name(version), None));
    }
    GhRelease {
        tag_name: format!("v{version}"),
        assets,
        prerelease,
    }
}

#[test]
fn asset_for_each_target() {
    for (target, name) in ASSETS_2_11_7 {
        let (pat, kind) = caddy_asset_for(target);
        let re = pattern(&pat).unwrap();
        assert!(re.is_match(name), "{target}: {pat} misses {name}");
        let want = if target.is_windows() {
            Kind::Zip
        } else {
            Kind::TarGz
        };
        assert_eq!(kind, want, "{target}");
        // Never another system's or architecture's build, nor a decoy.
        for (other, other_name) in ASSETS_2_11_7 {
            if other != target {
                assert!(!re.is_match(other_name), "{target} takes {other_name}");
            }
        }
        for decoy in DECOYS {
            assert!(!re.is_match(decoy), "{target} takes {decoy}");
        }
    }
}

/// Every target has its own build: Caddy is never emulated.
#[test]
fn native_on_every_target() {
    let caddy = Catalog::get().find("caddy").unwrap();
    for target in Target::ALL {
        assert!(caddy.is_available_on(target), "{target}");
        assert!(!caddy.is_emulated_on(target), "{target}");
    }
}

#[test]
fn catalog_entry() {
    let caddy = Catalog::get().find("caddy").unwrap();
    assert!(matches!(caddy.group, crate::setup::Group::Languages));
    assert_eq!(caddy.home, "https://caddyserver.com");
    assert!(caddy.versioned);
    assert_eq!(caddy.key, "caddy/caddy.exe");
    assert_eq!(caddy.folder.as_deref(), Some("caddy"));
    assert_eq!(caddy.check, ["caddy/caddy.exe", "version"]);
    assert!(crate::install::installer("caddy").is_some());
    for target in Target::ALL {
        let key = translate_rel(target, &caddy.key);
        if target.is_windows() {
            assert_eq!(key, "caddy/caddy.exe", "{target}");
        } else {
            assert_eq!(key, "caddy/caddy", "{target}");
        }
    }
}

/// Lines of the real `caddy_2.10.2_checksums.txt` (SHA-512).
const CHECKSUMS_2_10_2: &str = "\
747df7ee74de188485157a383633a1a963fd9233b71fbb4a69ddcbcc589ce4e2cc82dacf5dbbe136cb51d17e14c59daeb5d9bc92487610b0f3b93680b2646546  caddy_2.10.2_linux_amd64.tar.gz
0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f  caddy_2.10.2_linux_amd64.deb
6ce061a690312ab38367df3c5d5f89a2e4a263e7300d300d87356211bb81e79b15933e6d6203e03fbf26f15cc0311f264805f336147dbdd24938d84b57a4421c  caddy_2.10.2_linux_arm64.tar.gz
3b8a4b24054bddd96cae8da5c34e48c19105be29859d6f09e37344b7e188881cfd6b0a91af3f3e8f1d10987153e6bad70d1752c967e1360cc0041fc7736076ec  caddy_2.10.2_mac_arm64.tar.gz
88347875f0cd4b5e26bb39cd1f359613f932d54158d560e03244004d1ba6e61aae0cd625ba7c913bd46df096ef973fef2249396b0bb81143414378cb4447aeb8  caddy_2.10.2_windows_amd64.zip
";

#[test]
fn hash_from_the_checksum_file() {
    assert_eq!(
        caddy_hash_in(CHECKSUMS_2_10_2, "caddy_2.10.2_windows_amd64.zip"),
        Some(Expect::Sha512(
            "88347875f0cd4b5e26bb39cd1f359613f932d54158d560e03244004d1ba6e61aae0cd625ba7c913bd46df096ef973fef2249396b0bb81143414378cb4447aeb8".into()
        ))
    );
    assert_eq!(
        caddy_hash_in(CHECKSUMS_2_10_2, "caddy_2.10.2_linux_amd64.tar.gz"),
        Some(Expect::Sha512(
            "747df7ee74de188485157a383633a1a963fd9233b71fbb4a69ddcbcc589ce4e2cc82dacf5dbbe136cb51d17e14c59daeb5d9bc92487610b0f3b93680b2646546".into()
        ))
    );
    // Not listed, or only a prefix of a listed name.
    assert_eq!(
        caddy_hash_in(CHECKSUMS_2_10_2, "caddy_2.10.2_mac_amd64.tar.gz"),
        None
    );
    assert_eq!(
        caddy_hash_in(CHECKSUMS_2_10_2, "caddy_2.10.2_linux_amd64"),
        None
    );
    // A SHA-256 line, in upper case or with the binary marker.
    let sha = "AB".repeat(32);
    assert_eq!(
        caddy_hash_in(&format!("{sha} *caddy_x.zip\n"), "caddy_x.zip"),
        Some(Expect::Sha256(sha.to_ascii_lowercase()))
    );
    // Neither length, or not hex: no hash.
    assert_eq!(caddy_hash_in("abcd  caddy_x.zip", "caddy_x.zip"), None);
    assert_eq!(
        caddy_hash_in(&format!("{}  caddy_x.zip", "g".repeat(64)), "caddy_x.zip"),
        None
    );
    assert_eq!(caddy_hash_in("", "caddy_x.zip"), None);
    assert_eq!(caddy_checksums_name("2.10.2"), "caddy_2.10.2_checksums.txt");
}

#[test]
fn stable_and_verifiable_releases_newest_first() {
    let all = vec![
        release("2.9.1", false, false, true),
        release("2.11.0-beta.2", true, true, true),
        release("2.11.7", false, true, true),
        release("2.10.2", false, true, true),
        // Neither a digest nor a checksum file: never offered.
        release("2.8.4", false, false, false),
        release("2.11.6", false, true, false),
    ];
    for target in Target::ALL {
        let stable = caddy_stable(all.clone(), target).unwrap();
        let tags: Vec<&str> = stable.iter().map(|r| r.tag_name.as_str()).collect();
        assert_eq!(
            tags,
            ["v2.11.7", "v2.11.6", "v2.10.2", "v2.9.1"],
            "{target}"
        );
        assert_eq!(
            crate::install::lang::release_lines(&stable),
            ["2.11", "2.10", "2.9"],
            "{target}"
        );
    }
}

/// A release without the build of a target is not offered there.
#[test]
fn release_without_the_build_of_a_target() {
    let mut rel = release("2.11.7", false, true, true);
    rel.assets.retain(|a| !a.name.contains("_mac_arm64"));
    let none = caddy_stable(vec![rel.clone()], Target::DARWIN_ARM64).unwrap();
    assert!(none.is_empty());
    let some = caddy_stable(vec![rel], Target::LINUX_X64).unwrap();
    assert_eq!(some.len(), 1);
}

#[test]
fn verifiable_by_digest_or_checksum_file() {
    let name = "caddy_2.9.1_windows_amd64.zip";
    let with_file = release("2.9.1", false, false, true);
    let a = with_file.assets.iter().find(|a| a.name == name).unwrap();
    assert!(caddy_verifiable(&with_file, a));
    let bare = release("2.9.1", false, false, false);
    let a = bare.assets.iter().find(|a| a.name == name).unwrap();
    assert!(!caddy_verifiable(&bare, a));
    let digest = release("2.11.7", false, true, false);
    let a = digest
        .assets
        .iter()
        .find(|a| a.name == "caddy_2.11.7_windows_amd64.zip")
        .unwrap();
    assert!(caddy_verifiable(&digest, a));
    assert_eq!(caddy_version("v2.11.7"), "2.11.7");
}
