use super::*;
use crate::install::Dest;

/// The download target of MySQL on each kit target: x64 on Windows ARM64
/// (`x64Only`), native elsewhere.
fn mysql_download_target(t: Target) -> Target {
    let c = crate::setup::Catalog::get().find("mysql").unwrap();
    if c.is_emulated_on(t) {
        t.with_arch(Arch::X64)
    } else {
        t
    }
}

/// The lines of the catalog (8.4 and 9.7 LTS), each with its version and a
/// SHA-256 for every download target; asset names and URLs per target.
#[test]
fn mysql_pinned_lines_on_every_target() {
    let pins = mysql_pins();
    assert_eq!(pins["8.4"].version, "8.4.11");
    assert_eq!(pins["9.7"].version, "9.7.2");
    let files = [
        (Target::WIN32_X64, "mysql-9.7.2-winx64.zip", Kind::Zip),
        (Target::WIN32_ARM64, "mysql-9.7.2-winx64.zip", Kind::Zip),
        (
            Target::DARWIN_ARM64,
            "mysql-9.7.2-macos15-arm64.tar.gz",
            Kind::TarGz,
        ),
        (
            Target::DARWIN_X64,
            "mysql-9.7.2-macos15-x86_64.tar.gz",
            Kind::TarGz,
        ),
        (
            Target::LINUX_X64,
            "mysql-9.7.2-linux-glibc2.28-x86_64-minimal.tar.xz",
            Kind::TarXz,
        ),
        (
            Target::LINUX_ARM64,
            "mysql-9.7.2-linux-glibc2.28-aarch64.tar.xz",
            Kind::TarXz,
        ),
    ];
    for (t, file, kind) in files {
        let dl = mysql_download_target(t);
        assert_eq!(mysql_lines_for(pins, dl), ["9.7", "8.4"], "{}", t.as_str());
        let r = mysql_release_for(pins, dl, "9.7").unwrap();
        assert_eq!(r.version, "9.7.2");
        let p = &r.parts[0];
        assert_eq!(p.file_name, file, "{}", t.as_str());
        assert_eq!(p.kind, kind);
        assert_eq!(
            p.url,
            format!("https://cdn.mysql.com/Downloads/MySQL-9.7/{file}")
        );
        assert_eq!(p.dest, Dest::Own);
        match &p.expect {
            Expect::Sha256(s) => {
                assert_eq!(s, &pins["9.7"].sha256_by_target[dl.as_str()]);
                assert_eq!(s.len(), 64);
            }
            other => panic!("{}: {other:?}", t.as_str()),
        }
        let old = mysql_release_for(pins, dl, "8.4").unwrap();
        assert_eq!(
            old.parts[0].url,
            mysql_url("8.4", &mysql_file(dl, "8.4.11").0)
        );
    }
    // No pin, no download: Windows ARM64 itself has none (x64 runs there).
    assert!(mysql_release_for(pins, Target::WIN32_ARM64, "9.7").is_err());
    assert!(mysql_release_for(pins, Target::LINUX_X64, "9.6").is_err());
}

/// A line whose version is not of the line, or whose hash is not one, is
/// never offered or downloaded; a file with another hash is refused.
#[test]
fn mysql_refuses_other_hashes() {
    let catalog: Value = serde_json::from_str(&format!(
        r#"{{"components":[{{"id":"mysql","sha256ByLine":{{
            "9.7":{{"version":"9.7.2","sha256ByTarget":{{"linux-x64":"{a}","win32-x64":"zz"}}}},
            "8.4":{{"version":"9.7.2","sha256ByTarget":{{"linux-x64":"{a}"}}}}}}}}]}}"#,
        a = "a".repeat(64)
    ))
    .unwrap();
    let pins = mysql_pins_in(&catalog);
    assert_eq!(mysql_lines_for(&pins, Target::LINUX_X64), ["9.7"]);
    assert!(mysql_lines_for(&pins, Target::DARWIN_ARM64).is_empty());
    assert!(mysql_release_for(&pins, Target::WIN32_X64, "9.7").is_err());
    let r = mysql_release_for(&pins, Target::LINUX_X64, "9.7").unwrap();
    let tmp = crate::kit::tests::TempDir::new("mysql-hash");
    let file = tmp.0.join(&r.parts[0].file_name);
    fs::write(&file, b"not the build").unwrap();
    assert!(crate::net::verify::check(&file, &r.parts[0].expect).is_err());
    assert!(!file.exists(), "a file with another hash is deleted");
    assert!(mysql_pins_in(&serde_json::json!({})).is_empty());
}

/// D-110: the test suite, debug symbols and debug builds go.
#[test]
fn mysql_strips_what_the_kit_never_runs() {
    let tmp = crate::kit::tests::TempDir::new("mysql-strip");
    let t = &tmp.0;
    for f in [
        "bin/mysqld.exe",
        "bin/mysqld.pdb",
        "bin/mysqld-debug",
        "lib/plugin/debug/x.so",
        "lib/plugin/keep.so",
        "mysql-test/t/a.test",
        "share/english/errmsg.sys",
    ] {
        let p = crate::common::fsx::join_rel(t, f);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, b"x").unwrap();
    }
    strip_mysql(t).unwrap();
    for gone in [
        "bin/mysqld.pdb",
        "bin/mysqld-debug",
        "lib/plugin/debug",
        "mysql-test",
    ] {
        assert!(!crate::common::fsx::join_rel(t, gone).exists(), "{gone}");
    }
    for kept in [
        "bin/mysqld.exe",
        "lib/plugin/keep.so",
        "share/english/errmsg.sys",
    ] {
        assert!(crate::common::fsx::join_rel(t, kept).exists(), "{kept}");
    }
}
