use super::*;
use crate::kit::Target;

fn release(tag: &str, names: &[&str]) -> GhRelease {
    GhRelease {
        tag_name: tag.into(),
        assets: names
            .iter()
            .map(|name| GhAsset {
                name: (*name).into(),
                browser_download_url: format!("https://example.invalid/{tag}/{name}"),
                digest: Some(format!("sha256:{}", "a".repeat(64))),
            })
            .collect(),
        prerelease: false,
    }
}

/// T2.7.1: the asset names of rstudio/tinytex-releases v2026.10 (checked
/// with `gh api` on 17/10/2026), the layout since v2026.04: TinyTeX-1 of
/// each system, never TinyTeX-0/-2, the bundle without a version or the
/// musl build.
#[test]
fn tinytex_new_asset_names() {
    let v = "v2026.10";
    let rel = release(
        v,
        &[
            "installer-unix-v2026.10.tar.gz",
            "TinyTeX-0-darwin-v2026.10.tar.xz",
            "TinyTeX-0-linux-x86_64-v2026.10.tar.xz",
            "TinyTeX-1-darwin-v2026.10.tar.xz",
            "TinyTeX-1-linux-arm64-v2026.10.tar.xz",
            "TinyTeX-1-linux-x86_64-v2026.10.tar.xz",
            "TinyTeX-1-linuxmusl-x86_64-v2026.10.tar.xz",
            "TinyTeX-1-tar-v2026.10.gz",
            "TinyTeX-1-windows-v2026.10.exe",
            "TinyTeX-2-linux-x86_64-v2026.10.tar.xz",
            "TinyTeX-darwin-v2026.10.tar.xz",
            "TinyTeX-linux-x86_64-v2026.10.tar.xz",
            "TinyTeX-v2026.10.tgz",
            "TinyTeX-v2026.10.zip",
        ],
    );
    for (target, name, kind) in [
        (
            Target::WIN32_X64,
            "TinyTeX-1-windows-v2026.10.exe",
            Kind::SevenZip,
        ),
        (
            Target::WIN32_ARM64,
            "TinyTeX-1-windows-v2026.10.exe",
            Kind::SevenZip,
        ),
        (
            Target::DARWIN_ARM64,
            "TinyTeX-1-darwin-v2026.10.tar.xz",
            Kind::TarXz,
        ),
        (
            Target::DARWIN_X64,
            "TinyTeX-1-darwin-v2026.10.tar.xz",
            Kind::TarXz,
        ),
        (
            Target::LINUX_X64,
            "TinyTeX-1-linux-x86_64-v2026.10.tar.xz",
            Kind::TarXz,
        ),
        (
            Target::LINUX_ARM64,
            "TinyTeX-1-linux-arm64-v2026.10.tar.xz",
            Kind::TarXz,
        ),
    ] {
        let (asset, k, inner) = tinytex_pick(&rel, target).unwrap();
        assert_eq!((asset.name.as_str(), k), (name, kind), "{target}");
        let expected = if target.is_windows() {
            Inner::Named("TinyTeX".into())
        } else {
            Inner::Single
        };
        assert_eq!(inner, expected, "{target}");
    }
}

/// A release with only the names before v2026.04 (v2026.03.02) still
/// installs on macOS and Linux; Windows keeps its one name.
#[test]
fn tinytex_old_asset_names_as_fallback() {
    let rel = release(
        "v2026.03.02",
        &[
            "TinyTeX-1-arm64-v2026.03.02.tar.gz",
            "TinyTeX-1-v2026.03.02.tar.gz",
            "TinyTeX-1-v2026.03.02.tgz",
            "TinyTeX-1-v2026.03.02.zip",
        ],
    );
    let pick = |t| tinytex_pick(&rel, t).map(|(a, k, _)| (a.name.clone(), k));
    assert_eq!(
        pick(Target::DARWIN_ARM64),
        Some(("TinyTeX-1-v2026.03.02.tgz".into(), Kind::TarGz))
    );
    assert_eq!(
        pick(Target::LINUX_X64),
        Some(("TinyTeX-1-v2026.03.02.tar.gz".into(), Kind::TarGz))
    );
    assert_eq!(
        pick(Target::LINUX_ARM64),
        Some(("TinyTeX-1-arm64-v2026.03.02.tar.gz".into(), Kind::TarGz))
    );
    assert_eq!(pick(Target::WIN32_X64), None);
    // The older Linux names of the kit (x86_64-linux, aarch64-linux).
    let older = release(
        "v2025.01",
        &[
            "TinyTeX-1-x86_64-linux-v2025.01.tar.gz",
            "TinyTeX-1-aarch64-linux-v2025.01.tar.xz",
        ],
    );
    assert_eq!(
        tinytex_pick(&older, Target::LINUX_X64).unwrap().0.name,
        "TinyTeX-1-x86_64-linux-v2025.01.tar.gz"
    );
    assert_eq!(
        tinytex_pick(&older, Target::LINUX_ARM64).unwrap().1,
        Kind::TarXz
    );
}

/// T2.7.1: tlmgr inside the TinyTeX folder, in the binary folder of each
/// system (before, on macOS and Linux, `bin/windows/tlmgr`).
#[test]
fn tlmgr_in_the_binary_folder_of_each_system() {
    for (target, rel) in [
        (Target::WIN32_X64, "bin/windows/tlmgr.bat"),
        (Target::WIN32_ARM64, "bin/windows/tlmgr.bat"),
        (Target::DARWIN_ARM64, "bin/universal-darwin/tlmgr"),
        (Target::DARWIN_X64, "bin/universal-darwin/tlmgr"),
        (Target::LINUX_X64, "bin/x86_64-linux/tlmgr"),
        (Target::LINUX_ARM64, "bin/aarch64-linux/tlmgr"),
    ] {
        assert_eq!(tlmgr_rel(target), rel, "{target}");
    }
}
