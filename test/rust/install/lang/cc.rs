use super::*;

fn release(tag: &str, prerelease: bool, assets: &[&str]) -> GhRelease {
    GhRelease {
        tag_name: tag.into(),
        assets: assets
            .iter()
            .map(|name| GhAsset {
                name: (*name).into(),
                browser_download_url: format!("https://example.invalid/{tag}/{name}"),
                digest: Some(format!("sha256:{}", "a".repeat(64))),
            })
            .collect(),
        prerelease,
    }
}

/// BUG-V3-05: the newest stable release that has the archive of the
/// system, not only the latest (no macOS x64 after 20.1.7, macOS ARM64
/// missing from some releases); never a release candidate.
#[test]
fn llvm_takes_the_newest_release_with_the_archive() {
    use crate::kit::Target;

    let releases = [
        release(
            "llvmorg-23.1.0",
            false,
            &[
                "LLVM-23.1.0-Linux-X64.tar.xz",
                "LLVM-23.1.0-Linux-ARM64.tar.xz",
            ],
        ),
        release(
            "llvmorg-24.1.0-rc1",
            true,
            &["LLVM-24.1.0-rc1-macOS-X64.tar.xz"],
        ),
        release(
            "llvmorg-22.1.8",
            false,
            &[
                "LLVM-22.1.8-Linux-X64.tar.xz",
                "LLVM-22.1.8-macOS-ARM64.tar.xz",
            ],
        ),
        release(
            "llvmorg-20.1.7",
            false,
            &[
                "LLVM-20.1.7-macOS-ARM64.tar.xz",
                "LLVM-20.1.7-macOS-X64.tar.xz",
            ],
        ),
    ];
    let pick = |t: Target| {
        let re = pattern(llvm_asset(t)).unwrap();
        newest_llvm(&releases, &re).map(|(r, a)| (r.tag_name.clone(), a.name.clone()))
    };
    assert_eq!(
        pick(Target::LINUX_X64),
        Some((
            "llvmorg-23.1.0".into(),
            "LLVM-23.1.0-Linux-X64.tar.xz".into()
        ))
    );
    assert_eq!(
        pick(Target::LINUX_ARM64).unwrap().1,
        "LLVM-23.1.0-Linux-ARM64.tar.xz"
    );
    assert_eq!(
        pick(Target::DARWIN_ARM64),
        Some((
            "llvmorg-22.1.8".into(),
            "LLVM-22.1.8-macOS-ARM64.tar.xz".into()
        ))
    );
    assert_eq!(
        pick(Target::DARWIN_X64),
        Some((
            "llvmorg-20.1.7".into(),
            "LLVM-20.1.7-macOS-X64.tar.xz".into()
        ))
    );

    // An archive GitHub gives no SHA-256 for is never taken.
    let mut unchecked = release("llvmorg-21.1.8", false, &["LLVM-21.1.8-macOS-X64.tar.xz"]);
    unchecked.assets[0].digest = None;
    let re = pattern(llvm_asset(Target::DARWIN_X64)).unwrap();
    assert!(newest_llvm(&[unchecked], &re).is_none());
}

/// The archive names of each system match only that system's archive.
#[test]
fn llvm_asset_names_per_target() {
    use crate::kit::Target;

    let all = [
        "LLVM-23.1.3-Linux-ARM64.tar.xz",
        "LLVM-23.1.3-Linux-X64.tar.xz",
        "LLVM-23.1.3-macOS-ARM64.tar.xz",
        "LLVM-20.1.7-macOS-X64.tar.xz",
        "LLVM-23.1.3-macOS-ARM64.tar.xz.jsonl",
        "LLVM-23.1.3-macOS-ARM64.tar.zst",
    ];
    for (target, expected) in [
        (Target::LINUX_ARM64, "LLVM-23.1.3-Linux-ARM64.tar.xz"),
        (Target::LINUX_X64, "LLVM-23.1.3-Linux-X64.tar.xz"),
        (Target::DARWIN_ARM64, "LLVM-23.1.3-macOS-ARM64.tar.xz"),
        (Target::DARWIN_X64, "LLVM-20.1.7-macOS-X64.tar.xz"),
    ] {
        let re = pattern(llvm_asset(target)).unwrap();
        let hits: Vec<&str> = all.iter().copied().filter(|n| re.is_match(n)).collect();
        assert_eq!(hits, [expected], "{target}");
    }
}

/// When 20.1.7 leaves the last 100 releases, macOS x64 still gets it,
/// from the fixed tag, archive and SHA-256; no other system is pinned.
#[test]
fn llvm_is_pinned_on_macos_x64_only() {
    use crate::kit::Target;

    let r = llvm_pinned(Target::DARWIN_X64).unwrap();
    assert_eq!(r.version, "llvmorg-20.1.7");
    assert_eq!(r.parts.len(), 1);
    assert_eq!(
        r.parts[0].url,
        "https://github.com/llvm/llvm-project/releases/download/llvmorg-20.1.7/LLVM-20.1.7-macOS-X64.tar.xz"
    );
    assert!(pattern(llvm_asset(Target::DARWIN_X64))
        .unwrap()
        .is_match(&r.parts[0].file_name));
    let Expect::Sha256(sha) = &r.parts[0].expect else {
        panic!("sha256")
    };
    assert!(sha.len() == 64 && sha.chars().all(|c| c.is_ascii_hexdigit()));
    for t in Target::ALL {
        if t != Target::DARWIN_X64 {
            assert!(llvm_pinned(t).is_none(), "{t}");
        }
    }
}
