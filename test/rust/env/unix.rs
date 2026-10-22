use super::*;
use crate::env::tests::*;
use crate::env::*;
use crate::kit::tests::TempDir;

/// D-112: with C and C++ and the kit's Zig on macOS and Linux, CC, CXX,
/// AR and RANLIB are the shims of `cc/bin`, on the PATH. Without Zig the
/// shims would fail and hide the system's compiler: neither the PATH nor
/// the variables. The LLVM of an older kit: on the PATH, no variables (they
/// would name programs it lacks). Nothing on Windows.
#[test]
fn cc_variables_on_unix_with_cc_and_zig() {
    use crate::kit::Target;

    let tmp = TempDir::new("env-cc");
    let zig = crate::setup::Catalog::get().find("zig").unwrap();
    for target in [
        Target::LINUX_X64,
        Target::LINUX_ARM64,
        Target::DARWIN_ARM64,
        Target::DARWIN_X64,
    ] {
        let bin_of = |kit: &Kit| crate::fsx::join_rel(kit.apps(), "cc/bin");
        let on_path = |env: &KitEnv, kit: &Kit| {
            env.path_entries()
                .iter()
                .any(|e| Path::new(e) == bin_of(kit))
        };
        let no_vars = |env: &KitEnv| CC_VARS.iter().all(|(n, _)| env.get(n).is_none());

        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        let bare = KitEnv::build(&kit, &Settings::default(), "en", &host());
        assert!(no_vars(&bare) && !on_path(&bare, &kit), "{target}");
        // The shims without Zig.
        touch(&kit, &CC_SHIMS);
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        assert!(no_vars(&env), "{target} without zig");
        assert!(!on_path(&env, &kit), "{target} without zig");
        // With the kit's Zig.
        touch(&kit, &["zig/0.15/zig"]);
        zig.save_version(&kit, "0.15.2").unwrap();
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        let bin = bin_of(&kit);
        for (name, tool) in CC_VARS {
            assert_eq!(
                env.get(name).map(Path::new),
                Some(bin.join(tool).as_path()),
                "{target} {name}"
            );
        }
        assert!(on_path(&env, &kit), "{target}");

        // An older kit with the LLVM tarball: `clang`, no shims.
        let old = Kit::at_target(tmp.0.join(format!("old-{target}")), target);
        touch(&old, &["cc/bin/clang", "cc/bin/llvm-ar", "zig/0.15/zig"]);
        zig.save_version(&old, "0.15.2").unwrap();
        let env = KitEnv::build(&old, &Settings::default(), "en", &host());
        assert!(no_vars(&env), "{target} old LLVM");
        assert!(on_path(&env, &old), "{target} old LLVM");
    }
    let win = Kit::at_target(tmp.0.join("win"), Target::WIN32_X64);
    touch(&win, &["cc/bin/clang.exe", "zig/0.15/zig.exe"]);
    let env = KitEnv::build(&win, &Settings::default(), "en", &host());
    for (name, _) in CC_VARS {
        assert_eq!(env.get(name), None, "{name}");
    }
    let win_bin = crate::fsx::join_rel(win.apps(), "cc/bin");
    assert!(env.path_entries().iter().any(|e| Path::new(e) == win_bin));
}

/// Each variable of C and C++ names a program on disk: a `cc/bin` with
/// some shims missing gets only the others.
#[test]
fn cc_variables_only_for_programs_on_disk() {
    let bin = Path::new("K").join("cc").join("bin");
    let all = unix_cc_env(&bin, true, |_| true);
    assert!(all.on_path);
    assert_eq!(
        all.vars,
        CC_VARS
            .iter()
            .map(|(n, t)| (*n, bin.join(t)))
            .collect::<Vec<_>>()
    );
    let no_ranlib = unix_cc_env(&bin, true, |p| !p.ends_with("ranlib"));
    let names: Vec<&str> = no_ranlib.vars.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, ["CC", "CXX", "AR"]);
    assert_eq!(unix_cc_env(&bin, false, |_| true), UnixCc::default());
    assert_eq!(unix_cc_env(&bin, true, |_| false), UnixCc::default());
}

/// On a Mac with the Command Line Tools rustc links with Apple's `cc`,
/// never the shim; nowhere else, and not without the tools.
#[test]
fn macos_rust_linker_with_command_line_tools() {
    use crate::kit::Target;

    for t in Target::ALL {
        assert_eq!(macos_rust_linker(t, false), None, "{t}");
        let got = macos_rust_linker(t, true);
        assert_eq!(got.is_some(), t.os.is_macos(), "{t}");
    }
    assert_eq!(
        macos_rust_linker(Target::DARWIN_ARM64, true),
        Some((
            "CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER".to_owned(),
            "/usr/bin/cc"
        ))
    );
    assert_eq!(
        macos_rust_linker(Target::DARWIN_X64, true).map(|(n, _)| n),
        Some("CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER".to_owned())
    );
    for dir in XCODE_DEVELOPER_DIRS {
        assert!(dir.starts_with('/') && !dir.ends_with('/'), "{dir}");
    }
}

/// D-112: the linker of Rust's gnullvm host on Windows ARM64 only, the
/// aarch64 wrapper of llvm-mingw first.
#[test]
fn rust_linker_on_windows_arm64() {
    use crate::kit::Target;

    let bin = Path::new("K").join("cc").join("bin");
    let all = |_: &Path| true;
    assert_eq!(
        arm64_rust_linker(&bin, all),
        Some(bin.join("aarch64-w64-mingw32-clang.exe"))
    );
    let clang_only =
        |p: &Path| p.ends_with("clang.exe") && !p.ends_with("aarch64-w64-mingw32-clang.exe");
    assert_eq!(
        arm64_rust_linker(&bin, clang_only),
        Some(bin.join("clang.exe"))
    );
    assert_eq!(arm64_rust_linker(&bin, |_| false), None);

    let var = "CARGO_TARGET_AARCH64_PC_WINDOWS_GNULLVM_LINKER";
    let tmp = TempDir::new("env-rust-arm64");
    let parts = ["cc/bin/clang.exe", "cc/bin/aarch64-w64-mingw32-clang.exe"];
    let arm = Kit::at_target(tmp.0.join("arm"), Target::WIN32_ARM64);
    let env = KitEnv::build(&arm, &Settings::default(), "en", &host());
    assert_eq!(env.get(var), None, "no cc, no linker");
    touch(&arm, &parts);
    let env = KitEnv::build(&arm, &Settings::default(), "en", &host());
    assert_eq!(
        env.get(var).map(Path::new),
        Some(crate::fsx::join_rel(arm.apps(), parts[1]).as_path())
    );
    // Windows x64 unchanged.
    let x64 = Kit::at_target(tmp.0.join("x64"), Target::WIN32_X64);
    touch(&x64, &parts);
    let env = KitEnv::build(&x64, &Settings::default(), "en", &host());
    assert_eq!(env.get(var), None);
}

/// D-112 hook: a Git with `libexec/git-core` (dugite-native) on macOS and
/// Linux gets dugite's variables; Linux its CA bundle when it has one; never
/// PREFIX; nothing on Windows.
#[test]
fn git_variables_for_a_dugite_layout() {
    use crate::kit::Os;

    let git = Path::new("/kit/data/apps/linux-x64/git");
    let all = |_: &Path| true;
    let names = |os| -> Vec<&str> {
        git_unix_env(os, git, all)
            .into_iter()
            .map(|(n, _)| n)
            .collect()
    };
    assert_eq!(
        names(Os::Linux),
        [
            "GIT_EXEC_PATH",
            "GIT_TEMPLATE_DIR",
            "GIT_CONFIG_SYSTEM",
            "GIT_SSL_CAINFO"
        ]
    );
    assert_eq!(
        names(Os::Darwin),
        ["GIT_EXEC_PATH", "GIT_TEMPLATE_DIR", "GIT_CONFIG_SYSTEM"]
    );
    assert!(names(Os::Win32).is_empty());
    let env: Vec<(&str, PathBuf)> = git_unix_env(Os::Linux, git, all);
    for (name, rel) in [
        ("GIT_EXEC_PATH", "libexec/git-core"),
        ("GIT_TEMPLATE_DIR", "share/git-core/templates"),
        ("GIT_CONFIG_SYSTEM", "etc/gitconfig"),
        ("GIT_SSL_CAINFO", "ssl/cacert.pem"),
    ] {
        let value = &env.iter().find(|(n, _)| *n == name).unwrap().1;
        assert_eq!(value, &crate::fsx::join_rel(git, rel), "{name}");
    }
    // Without libexec (a Git of another layout) or without the CA bundle.
    assert!(git_unix_env(Os::Linux, git, |_| false).is_empty());
    let no_ca = |p: &Path| !p.ends_with("cacert.pem");
    assert_eq!(git_unix_env(Os::Linux, git, no_ca).len(), 3);

    // Through the kit: a Linux kit with `git/libexec/git-core`.
    use crate::kit::Target;
    let tmp = TempDir::new("env-git-dugite");
    let kit = Kit::at_target(tmp.0.join("Kit"), Target::LINUX_X64);
    touch(&kit, &["git/bin/git"]);
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    assert_eq!(env.get("GIT_EXEC_PATH"), None);
    fs::create_dir_all(crate::fsx::join_rel(kit.apps(), "git/libexec/git-core")).unwrap();
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    assert_eq!(
        env.get("GIT_EXEC_PATH").map(Path::new),
        Some(crate::fsx::join_rel(kit.apps(), "git/libexec/git-core").as_path())
    );
    assert_eq!(env.get("PREFIX"), None);
}

/// D-112 hook: GEM_HOME of the default Ruby line on macOS and Linux, its
/// `bin` on the PATH after Ruby's; Windows unchanged.
#[test]
fn gem_home_per_system_and_line() {
    use crate::kit::Target;

    assert_eq!(
        gem_home(Path::new("D"), Target::LINUX_X64, "3.4"),
        Path::new("D")
            .join("ruby-gems")
            .join("linux-x64")
            .join("3.4")
    );
    let tmp = TempDir::new("env-gem-home");
    let ruby = crate::setup::Catalog::get().find("ruby").unwrap();
    for target in [Target::LINUX_X64, Target::DARWIN_ARM64, Target::WIN32_X64] {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        let bare = KitEnv::build(&kit, &Settings::default(), "en", &host());
        assert_eq!(bare.get("GEM_HOME"), None, "{target}");
        let key = if target.is_windows() {
            "ruby/3.4/bin/ruby.exe"
        } else {
            "ruby/3.4/bin/ruby"
        };
        touch(&kit, &[key]);
        ruby.save_version(&kit, "3.4.7").unwrap();
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        if target.is_windows() {
            assert_eq!(env.get("GEM_HOME"), None);
            continue;
        }
        let gems = gem_home(&kit.data(), target, "3.4");
        assert_eq!(
            env.get("GEM_HOME").map(Path::new),
            Some(gems.as_path()),
            "{target}"
        );
        let entries = env.path_entries();
        let at = |p: &Path| entries.iter().position(|e| Path::new(e) == p);
        let ruby_bin = crate::fsx::join_rel(kit.apps(), "ruby/3.4/bin");
        assert!(at(&ruby_bin).is_some(), "{target} {entries:?}");
        assert_eq!(
            at(&gems.join("bin")),
            at(&ruby_bin).map(|i| i + 1),
            "{target}"
        );
        assert_eq!(
            env.get("GEMRC").map(Path::new),
            Some(gemrc_path(&kit.data()).as_path())
        );
    }
}

/// Two Ruby lines on disk: the kit uses the newest (3.4) while the default
/// is 3.3, so GEM_HOME and its `bin` are 3.4's, next to 3.4's Ruby, never
/// a mix of the two lines.
#[test]
fn gem_home_follows_the_line_the_kit_uses() {
    use crate::kit::Target;

    let tmp = TempDir::new("env-gem-lines");
    let ruby = crate::setup::Catalog::get().find("ruby").unwrap();
    for target in [Target::LINUX_ARM64, Target::DARWIN_X64] {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        touch(&kit, &["ruby/3.3/bin/ruby", "ruby/3.4/bin/ruby"]);
        ruby.save_version(&kit, "3.3.9").unwrap();
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        let gems = gem_home(&kit.data(), target, "3.4");
        assert_eq!(
            env.get("GEM_HOME").map(Path::new),
            Some(gems.as_path()),
            "{target}"
        );
        let entries = env.path_entries();
        let at = |p: &Path| entries.iter().position(|e| Path::new(e) == p);
        let ruby_bin = crate::fsx::join_rel(kit.apps(), "ruby/3.4/bin");
        assert_eq!(
            at(&gems.join("bin")),
            at(&ruby_bin).map(|i| i + 1),
            "{target} {entries:?}"
        );
        let old = crate::fsx::join_rel(kit.apps(), "ruby/3.3");
        let old_gems = gem_home(&kit.data(), target, "3.3");
        assert!(
            entries
                .iter()
                .all(|e| !Path::new(e).starts_with(&old) && !Path::new(e).starts_with(&old_gems)),
            "{target} {entries:?}"
        );
    }
}

/// The GEM_HOME of a Ruby folder, for the swap of a line: by the folder's
/// name, never on Windows or for a folder not named after a line.
#[test]
fn gem_home_of_a_ruby_folder() {
    use crate::kit::Target;

    let data = Path::new("D");
    let dir = Path::new("D").join("apps").join("linux-x64").join("ruby");
    assert_eq!(
        gem_home_for_dir(data, Target::LINUX_X64, &dir.join("3.3")),
        Some(gem_home(data, Target::LINUX_X64, "3.3"))
    );
    assert_eq!(gem_home_for_dir(data, Target::LINUX_X64, &dir), None);
    assert_eq!(
        gem_home_for_dir(data, Target::WIN32_X64, &dir.join("3.3")),
        None
    );
}

/// The kit's gemrc turns on env shebangs, is written once and then left
/// to the user; `create_dirs` writes it where GEMRC points.
#[test]
fn gemrc_written_once_with_env_shebang() {
    let tmp = TempDir::new("env-gemrc");
    let rc = gemrc_path(&tmp.0);
    assert_eq!(rc, tmp.0.join("ruby-gems").join("gemrc"));
    assert!(GEMRC.lines().any(|l| l == "gem: --env-shebang"));
    assert!(GEMRC
        .lines()
        .all(|l| l.starts_with('#') || l.starts_with("gem: ")));
    write_gemrc(&rc).unwrap();
    assert_eq!(fs::read_to_string(&rc).unwrap(), GEMRC);
    fs::write(&rc, "gem: --no-document\n").unwrap();
    write_gemrc(&rc).unwrap();
    assert_eq!(fs::read_to_string(&rc).unwrap(), "gem: --no-document\n");

    let mut env = KitEnv::default();
    let other = tmp.0.join("other").join("gemrc");
    env.set_path("GEMRC", &other);
    env.create_dirs().unwrap();
    assert_eq!(fs::read_to_string(&other).unwrap(), GEMRC);
}

/// D-112: ICU's library found by its versioned name in a library folder.
#[test]
fn libicu_lookup() {
    let tmp = TempDir::new("env-libicu");
    let (with, without) = (tmp.0.join("with"), tmp.0.join("without"));
    fs::create_dir_all(&with).unwrap();
    fs::create_dir_all(&without).unwrap();
    fs::write(without.join("libicuuc.a"), b"").unwrap();
    fs::write(without.join("libc.so.6"), b"").unwrap();
    assert!(!has_libicu(std::slice::from_ref(&without)));
    assert!(!has_libicu(&[tmp.0.join("missing")]));
    fs::write(with.join("libicuuc.so.74"), b"").unwrap();
    assert!(has_libicu(&[without.clone(), with]));
    let dirs = linux_lib_dirs("aarch64");
    assert_eq!(dirs[0], Path::new("/usr/lib/aarch64-linux-gnu"));
    assert!(dirs.contains(&PathBuf::from("/usr/lib64")));
}

/// D-112: the invariant mode on a Linux kit when the PC has no ICU, with
/// or without .NET on disk (the final check of `prepare` uses the
/// environment built before .NET was installed); never on Windows or macOS
/// (where .NET carries or finds ICU).
#[test]
fn dotnet_invariant_off_linux_never() {
    use crate::kit::Target;

    let tmp = TempDir::new("env-dotnet");
    for (target, key) in [
        (Target::WIN32_X64, "dotnet/dotnet.exe"),
        (Target::DARWIN_ARM64, "dotnet/dotnet"),
    ] {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        touch(&kit, &[key]);
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        assert_eq!(
            env.get("DOTNET_SYSTEM_GLOBALIZATION_INVARIANT"),
            None,
            "{target}"
        );
    }
    // Linux, before and after .NET: set exactly when this PC has no ICU.
    let kit = Kit::at_target(tmp.0.join("linux"), Target::LINUX_X64);
    for installed in [false, true] {
        if installed {
            touch(&kit, &["dotnet/dotnet"]);
        }
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        assert_eq!(
            env.get("DOTNET_SYSTEM_GLOBALIZATION_INVARIANT"),
            (!libicu_present()).then_some("1"),
            "installed {installed}"
        );
    }
}
