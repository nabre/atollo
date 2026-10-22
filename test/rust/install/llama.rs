use super::*;

use crate::setup::catalog::Catalog;

#[test]
fn one_archive_per_target() {
    let t = |s: &str| Target::parse(s).unwrap();
    assert_eq!(
        asset_for("b11669", t("win32-x64")).0,
        "llama-b11669-bin-win-vulkan-x64.zip"
    );
    assert_eq!(
        asset_for("b11669", t("win32-arm64")).0,
        "llama-b11669-bin-win-vulkan-arm64.zip"
    );
    assert_eq!(
        asset_for("b11669", t("darwin-arm64")).0,
        "llama-b11669-bin-macos-arm64.tar.gz"
    );
    assert_eq!(
        asset_for("b11669", t("darwin-x64")).0,
        "llama-b11669-bin-macos-x64.tar.gz"
    );
    assert_eq!(
        asset_for("b11669", t("linux-arm64")).0,
        "llama-b11669-bin-ubuntu-vulkan-arm64.tar.gz"
    );
    // The zip has the programs at its root, the tar.gz in `llama-<build>/`.
    assert_eq!(asset_for("b1", t("win32-x64")).2, Inner::Root);
    assert_eq!(asset_for("b1", t("linux-x64")).2, Inner::Single);
}

#[test]
fn only_a_build_is_a_version() {
    assert!(valid_version("b11669"));
    assert!(!valid_version("b"));
    assert!(!valid_version("v0.7.0"));
    assert!(!valid_version("b1/../x"));
}

/// The catalog fixes the build and a SHA-256 for every target it offers.
#[test]
fn the_catalog_fixes_build_and_hashes() {
    let c = Catalog::get().find("llama").unwrap();
    assert!(valid_version(c.version.as_deref().unwrap()));
    for t in crate::kit::Target::ALL {
        let sha = c.sha256_for(t).unwrap_or_else(|| panic!("{t}"));
        assert_eq!(sha.len(), 64, "{t}");
        assert!(sha.bytes().all(|b| b.is_ascii_hexdigit()), "{t}");
    }
}
