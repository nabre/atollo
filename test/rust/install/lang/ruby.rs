use super::*;
use serde_json::json;

#[test]
fn ruby_highest_version_and_build() {
    let digest = format!("sha256:{}", "0".repeat(64));
    let rel = |pre: bool, names: &[&str]| json!({"prerelease": pre, "assets": names.iter().map(|n| json!({"name": n, "digest": digest})).collect::<Vec<_>>()});
    let releases = vec![
        rel(
            false,
            &[
                "rubyinstaller-3.3.9-1-x64.7z",
                "rubyinstaller-3.3.9-1-arm.7z",
            ],
        ),
        rel(
            false,
            &[
                "rubyinstaller-3.4.5-1-x64.7z",
                "rubyinstaller-3.4.5-2-x64.7z",
            ],
        ),
        rel(true, &["rubyinstaller-3.5.0-1-x64.7z"]),
    ];
    assert_eq!(
        pick_ruby_line(&releases, "x64", None).map(|(v, _)| v),
        Some("3.4.5-2".into())
    );
    assert_eq!(
        pick_ruby_line(&releases, "arm", None).map(|(v, _)| v),
        Some("3.3.9-1".into())
    );
    // Without GitHub's SHA-256 (older builds): not offered.
    let old =
        vec![json!({"prerelease": false, "assets": [{"name": "rubyinstaller-3.1.7-1-x64.7z"}]})];
    assert!(ruby_lines(&old, "x64").is_empty());
}

/// WP-2.2: conda-forge on macOS and Linux (micromamba, not movable),
/// RubyInstaller on Windows (unchanged).
#[test]
fn ruby_from_conda_off_windows() {
    for t in Target::ALL {
        assert_eq!(Ruby.custom(t), !t.is_windows(), "{t}");
        assert_eq!(Ruby.relocatable(t), t.is_windows(), "{t}");
    }
    // The conda-forge subdirs of the four Unix targets (checked on
    // api.anaconda.org: ruby 4.0.x and 3.4.x on all four).
    let verified = ["osx-arm64", "osx-64", "linux-64", "linux-aarch64"];
    for t in Target::ALL.into_iter().filter(|t| !t.is_windows()) {
        assert!(verified.contains(&t.conda_subdir()), "{t}");
    }
}

/// The lines and the version of a line from conda-forge's list (newest
/// first, as `conda_versions` gives it).
#[test]
fn ruby_conda_lines_and_pick() {
    let versions: Vec<String> = ["4.0.7", "4.0.6", "3.4.8", "3.4.7", "3.3.6"]
        .iter()
        .map(|v| (*v).to_owned())
        .collect();
    assert_eq!(ruby_conda_lines(&versions), ["4.0", "3.4", "3.3"]);
    assert_eq!(
        ruby_conda_pick(&versions, None).map(String::as_str),
        Some("4.0.7")
    );
    assert_eq!(
        ruby_conda_pick(&versions, Some("3.4")).map(String::as_str),
        Some("3.4.8")
    );
    assert_eq!(ruby_conda_pick(&versions, Some("3.2")), None);
    assert!(ruby_conda_lines(&[]).is_empty());
    // The end-of-life lines conda-forge still has (2.3 to 3.1) are not
    // offered: from 3.2 on, as RubyInstaller on Windows.
    let old: Vec<String> = ["3.2.9", "3.1.6", "2.7.8", "2.3.8", "10.0.1"]
        .iter()
        .map(|v| (*v).to_owned())
        .collect();
    assert_eq!(ruby_conda_lines(&old), ["3.2", "10.0"]);
    assert_eq!(ruby_conda_spec("4.0.7"), "ruby=4.0.7");
    assert_eq!(
        crate::install::native::micromamba_create_args(Path::new("/kit/ruby/4.0"), &["ruby=4.0.7"])
            .last()
            .map(String::as_str),
        Some("ruby=4.0.7")
    );
}

/// T2.2.4: conda-forge's `etc/gemrc` (ruby 4.0.7, linux-64) sends gems
/// into the environment over `GEM_HOME`: its `gemhome:` goes, the rest
/// stays.
#[test]
fn conda_gemrc_leaves_gem_home_to_the_kit() {
    let gemrc = "gemhome: /kit/ruby/4.0/share/rubygems\n:update_sources: true\n";
    assert_eq!(
        gemrc_without_gemhome(gemrc).as_deref(),
        Some(":update_sources: true\n")
    );
    assert_eq!(gemrc_without_gemhome(":update_sources: true\n"), None);
    assert_eq!(gemrc_without_gemhome(""), None);
}

/// T2.2.4: the build tools of conda's `rbconfig.rb` (lines of ruby 4.0.7
/// for linux-64; the macOS triplet of osx-arm64) under their plain names,
/// so a native gem builds with the kit's or the system's `cc`.
#[test]
fn conda_rbconfig_uses_plain_tool_names() {
    let text = concat!(
        "  CONFIG[\"PKG_CONFIG\"] = \"x86_64-conda-linux-gnu-pkg-config\"\n",
        "  CONFIG[\"CPP\"] = \"x86_64-conda-linux-gnu-cpp\"\n",
        "  CONFIG[\"STRIP\"] = \"x86_64-conda-linux-gnu-strip -S -x\"\n",
        "  CONFIG[\"CXX\"] = \"x86_64-conda-linux-gnu-c++\"\n",
        "  CONFIG[\"CC\"] = \"x86_64-conda-linux-gnu-cc\"\r\n",
        "  CONFIG[\"CC_VERSION_MESSAGE\"] = \"x86_64-conda-linux-gnu-cc (conda-forge gcc 15.3.0-7)\"\n",
        "  CONFIG[\"build\"] = \"x86_64-conda-linux-gnu\"\n",
        "  CONFIG[\"LDSHARED\"] = \"$(CC) -shared\"\n",
    );
    let want = concat!(
        "  CONFIG[\"PKG_CONFIG\"] = \"pkg-config\"\n",
        "  CONFIG[\"CPP\"] = \"$(CC) -E\"\n",
        "  CONFIG[\"STRIP\"] = \"strip -S -x\"\n",
        "  CONFIG[\"CXX\"] = \"c++\"\n",
        "  CONFIG[\"CC\"] = \"cc\"\r\n",
        "  CONFIG[\"CC_VERSION_MESSAGE\"] = \"x86_64-conda-linux-gnu-cc (conda-forge gcc 15.3.0-7)\"\n",
        "  CONFIG[\"build\"] = \"x86_64-conda-linux-gnu\"\n",
        "  CONFIG[\"LDSHARED\"] = \"$(CC) -shared\"\n",
    );
    assert_eq!(rbconfig_with_plain_tools(text).as_deref(), Some(want));
    assert_eq!(rbconfig_with_plain_tools(want), None);
    assert_eq!(
        rbconfig_with_plain_tools("  CONFIG[\"CC\"] = \"arm64-apple-darwin20.0.0-clang\"\n")
            .as_deref(),
        Some("  CONFIG[\"CC\"] = \"clang\"\n")
    );
    assert_eq!(
        rbconfig_with_plain_tools("  CONFIG[\"AR\"] = \"aarch64-conda-linux-gnu-ar\"\n").as_deref(),
        Some("  CONFIG[\"AR\"] = \"ar\"\n")
    );
    // osx-arm64 ruby 4.0.6: conda's llvm-nm is not in the Xcode tools.
    assert_eq!(
        rbconfig_with_plain_tools("  CONFIG[\"NM\"] = \"llvm-nm --no-llvm-bc\"\n").as_deref(),
        Some("  CONFIG[\"NM\"] = \"nm\"\n")
    );
    assert_eq!(
        rbconfig_with_plain_tools("  CONFIG[\"NM\"] = \"llvm-nm\"\n").as_deref(),
        Some("  CONFIG[\"NM\"] = \"nm\"\n")
    );
    assert_eq!(
        rbconfig_with_plain_tools("  CONFIG[\"NM_FLAGS\"] = \"llvm-nm\"\n"),
        None
    );
}
