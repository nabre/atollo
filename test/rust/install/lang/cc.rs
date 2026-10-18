use super::*;

/// The version saved on macOS and Linux carries make's version, read back
/// for the conda spec; anything else (an LLVM of an older kit) is not one.
#[test]
fn unix_version_round_trip() {
    assert_eq!(unix_version("4.4.1"), "zig cc, make 4.4.1");
    assert_eq!(make_version(&unix_version("4.4.1")), Some("4.4.1"));
    assert_eq!(make_version("llvmorg-20.1.7"), None);
    assert_eq!(make_version("zig cc, make "), None);
    assert_eq!(make_version("zig cc, make 4.4.1rc1"), None);
}

/// Every name build systems look for is a shim, each a POSIX sh script
/// with LF endings and no Windows path; the compilers call `zig cc` and
/// `zig c++`, the tools `zig ar` and `zig ranlib`, make its own folder.
#[test]
fn shims_cover_compilers_tools_and_make() {
    let shims = zig_shims();
    let names: Vec<&str> = shims.iter().map(|(n, _)| *n).collect();
    assert_eq!(
        names,
        ["cc", "gcc", "clang", "c++", "g++", "clang++", "ar", "ranlib", "make"]
    );
    let get = |name: &str| &shims.iter().find(|(n, _)| *n == name).unwrap().1;
    for (name, script) in &shims {
        assert!(script.starts_with("#!/bin/sh\n"), "{name}");
        assert!(!script.contains('\r'), "{name}");
        assert!(!script.contains('\\'), "{name}");
        assert!(!script.contains("@CMD@"), "{name}");
        assert!(!script.contains("@ZIG@"), "{name}");
        assert!(script.ends_with("\"$@\"\n"), "{name}");
    }
    for name in ["cc", "gcc", "clang"] {
        assert!(get(name).contains("\nexec zig cc \"$@\"\n"), "{name}");
    }
    for name in ["c++", "g++", "clang++"] {
        assert!(get(name).contains("\nexec zig c++ \"$@\"\n"), "{name}");
    }
    assert!(get("ar").contains("\nexec zig ar \"$@\"\n"));
    assert!(get("ranlib").contains("\nexec zig ranlib \"$@\"\n"));
    assert!(get("make").contains("\nexec \"$(dirname \"$0\")/../make/bin/make\" \"$@\"\n"));
    // Zig rejects Clang's and Rust's triples: the compilers translate them.
    assert!(get("cc").contains("--target=*) set -- \"$@\" -target"));
}

/// The shims run with sh: triples translated, other arguments untouched
/// and in order (also with spaces), `zig` from the PATH.
#[test]
#[cfg_attr(windows, ignore = "runs the shims with /bin/sh")]
fn shims_translate_target_triples() {
    use crate::kit::tests::TempDir;

    let tmp = TempDir::new("cc-shims");
    let bin = tmp.0.join("bin");
    write_zig_shims(&bin).unwrap();
    // A fake zig that prints its arguments one per line.
    let fake = tmp.0.join("fake");
    fs::create_dir_all(&fake).unwrap();
    let zig = fake.join("zig");
    fs::write(&zig, "#!/bin/sh\nfor a do echo \"$a\"; done\n").unwrap();
    crate::router::fs::set_executable(&zig).unwrap();
    let path = format!("{}:/usr/bin:/bin", fake.display());
    let run = |shim: &str, args: &[&str]| -> Vec<String> {
        let out = std::process::Command::new(bin.join(shim))
            .args(args)
            .env("PATH", &path)
            .output()
            .unwrap();
        assert!(out.status.success(), "{shim} {args:?}");
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_owned)
            .collect()
    };
    assert_eq!(
        run("cc", &["--target=x86_64-unknown-linux-gnu", "-c", "a b.c"]),
        ["cc", "-target", "x86_64-linux-gnu", "-c", "a b.c"]
    );
    assert_eq!(
        run(
            "c++",
            &["-O2", "-target", "arm64-apple-macosx14.0", "x.cpp"]
        ),
        ["c++", "-O2", "-target", "aarch64-macos", "x.cpp"]
    );
    assert_eq!(
        run("clang", &["--target=aarch64-unknown-linux-musl"]),
        ["cc", "-target", "aarch64-linux-musl"]
    );
    assert_eq!(
        run("gcc", &["--target=x86_64-apple-darwin"]),
        ["cc", "-target", "x86_64-macos"]
    );
    assert_eq!(run("ar", &["rcs", "lib.a"]), ["ar", "rcs", "lib.a"]);
    // No Zig: a clear error, not a "not found" of the shell.
    let out = std::process::Command::new(bin.join("cc"))
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(127));
    assert!(String::from_utf8_lossy(&out.stderr).contains("needs Zig"));

    // No Zig on the PATH but one in the kit (`zig/<line>/zig` next to
    // `cc/`): that one, whatever the PATH (an environment built before Zig
    // was installed).
    let apps = tmp.0.join("apps");
    let kit_bin = apps.join("cc").join("bin");
    write_zig_shims(&kit_bin).unwrap();
    let line = apps.join("zig").join("0.17");
    fs::create_dir_all(&line).unwrap();
    fs::copy(&zig, line.join("zig")).unwrap();
    crate::router::fs::set_executable(&line.join("zig")).unwrap();
    for (shim, first) in [("cc", "cc"), ("ranlib", "ranlib")] {
        let out = std::process::Command::new(kit_bin.join(shim))
            .arg("x")
            .env("PATH", "/nonexistent")
            .output()
            .unwrap();
        assert!(out.status.success(), "{shim}");
        assert_eq!(
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .collect::<Vec<_>>(),
            [first, "x"]
        );
    }
}

/// Without Zig on the PATH the shims take the kit's newest line, compared
/// as numbers (0.14 after 0.9, not the last in glob order), skip folders
/// that are not lines, and fall back to a Zig of the older layout.
#[test]
#[cfg_attr(windows, ignore = "runs the shims with /bin/sh")]
fn shims_take_the_newest_kit_zig() {
    use crate::kit::tests::TempDir;

    let tmp = TempDir::new("cc-shims-newest");
    let apps = tmp.0.join("apps");
    let bin = apps.join("cc").join("bin");
    write_zig_shims(&bin).unwrap();
    let zig_at = |dir: &Path, name: &str| {
        fs::create_dir_all(dir).unwrap();
        let zig = dir.join("zig");
        fs::write(&zig, format!("#!/bin/sh\necho {name}\n")).unwrap();
        crate::router::fs::set_executable(&zig).unwrap();
    };
    let run = || -> String {
        let out = std::process::Command::new(bin.join("cc"))
            .env("PATH", "/nonexistent")
            .output()
            .unwrap();
        assert!(out.status.success());
        String::from_utf8_lossy(&out.stdout).trim().to_owned()
    };
    let zig = apps.join("zig");
    zig_at(&zig, "old-layout");
    assert_eq!(run(), "old-layout");
    for line in ["0.9", "0.14", "0.13", "next"] {
        zig_at(&zig.join(line), line);
    }
    assert_eq!(run(), "0.14");
    zig_at(&zig.join("1.0"), "1.0");
    assert_eq!(run(), "1.0");
}

/// Xcode Command Line Tools are mentioned on macOS only, and only when
/// `xcode-select -p` printed no folder.
#[test]
fn xcode_note_on_macos_without_tools() {
    use crate::kit::Target;

    for t in Target::ALL {
        let mac = t.os == Os::Darwin;
        assert_eq!(xcode_note(t, None).is_some(), mac, "{t}");
        assert_eq!(xcode_note(t, Some("  ")).is_some(), mac, "{t}");
        assert!(
            xcode_note(t, Some("/Library/Developer/CommandLineTools")).is_none(),
            "{t}"
        );
    }
}

/// llvm-mingw on Windows (shared path, moves with the kit), shims and a
/// conda environment elsewhere (installed again when the kit moves).
#[test]
fn custom_installer_off_windows_only() {
    use crate::kit::Target;

    for t in Target::ALL {
        assert_eq!(Cc.custom(t), !t.is_windows(), "{t}");
        assert_eq!(Cc.relocatable(t), t.is_windows(), "{t}");
    }
}
