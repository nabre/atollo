use super::*;

#[test]
fn mongodb_major_release_only() {
    let current: Value = serde_json::from_str(&format!(
        r#"{{"versions":[
              {{"version":"9.1.0-rc0","production_release":false,"downloads":[]}},
              {{"version":"8.3.13","production_release":true,"downloads":[]}},
              {{"version":"9.0.4","production_release":true,"downloads":[
                {{"target":"windows","arch":"x86_64","edition":"enterprise","archive":{{"url":"https://x/e.zip","sha256":"{e}"}}}},
                {{"target":"windows","arch":"x86_64","edition":"base","archive":{{"url":"https://fastdl.mongodb.org/windows/mongodb-windows-x86_64-9.0.4.zip","sha256":"{b}"}}}}]}}]}}"#,
        e = "e".repeat(64),
        b = "b".repeat(64)
    ))
    .unwrap();
    let (v, url, sha) = mongodb_zip(&current, None).unwrap();
    assert_eq!(v, "9.0.4");
    assert!(url.ends_with("mongodb-windows-x86_64-9.0.4.zip"));
    assert_eq!(sha, "b".repeat(64));
    assert_eq!(mongodb_zip(&current, Some("9.0")).unwrap().0, "9.0.4");
    assert!(
        mongodb_zip(&current, Some("8.0")).is_err(),
        "8.3 is a rapid release"
    );
    assert_eq!(mongodb_lines(&current), ["9.0"]);
}

#[test]
fn mongodb_tools_from_release_json() {
    let j: Value = serde_json::from_str(&format!(
        r#"{{"versions":[{{"version":"100.19.1","downloads":[
              {{"name":"ubuntu2204","arch":"x86_64","archive":{{"url":"https://x/u.tgz","sha256":"{u}"}}}},
              {{"name":"windows","arch":"x86_64","archive":{{"url":"https://fastdl.mongodb.org/tools/db/mongodb-database-tools-windows-x86_64-100.19.1.zip","sha256":"{w}"}}}}]}}]}}"#,
        u = "u".repeat(64),
        w = "w".repeat(64)
    ))
    .unwrap();
    let (v, url, sha) = mongodb_tools_zip(&j).unwrap();
    assert_eq!(v, "100.19.1");
    assert!(url.ends_with("windows-x86_64-100.19.1.zip"));
    assert_eq!(sha, "w".repeat(64));
}

#[test]
fn sqlite_tools_for_each_arch() {
    let page = format!(
        "<!--\nPRODUCT,3.54.0,2026/sqlite-dll-win-x64-3540000.zip,1450308,{d}\n\
             PRODUCT,3.54.0,2026/sqlite-tools-win-arm64-3540000.zip,5619849,{a}\n\
             PRODUCT,3.54.0,2026/sqlite-tools-win-x64-3540000.zip,6982216,{x}\n-->",
        d = "d".repeat(64),
        a = "a".repeat(64),
        x = "c".repeat(64)
    );
    let (v, path, sha) = sqlite_tools(&page, "x64").unwrap();
    assert_eq!(v, "3.54.0");
    assert_eq!(path, "2026/sqlite-tools-win-x64-3540000.zip");
    assert_eq!(sha, "c".repeat(64));
    assert_eq!(sqlite_tools(&page, "arm64").unwrap().2, "a".repeat(64));
}

/// phpMyAdmin's version.json (2025-10): the newest release, every language.
#[test]
fn phpmyadmin_zip_from_version_json() {
    let versions: Value = serde_json::from_str(
        r#"{"version":"5.2.3","date":"2025-10-08","releases":[
            {"version":"5.2.3","date":"2025-10-08","php_versions":">=7.2,<8.4","mysql_versions":">=5.5"},
            {"version":"4.9.11","date":"2023-02-08","php_versions":">=5.5,<8.0","mysql_versions":">=5.5"}]}"#,
    )
    .unwrap();
    let (v, url) = phpmyadmin_zip(&versions).unwrap();
    assert_eq!(v, "5.2.3");
    assert_eq!(
        url,
        "https://files.phpmyadmin.net/phpMyAdmin/5.2.3/phpMyAdmin-5.2.3-all-languages.zip"
    );
    // Only digits and dots reach the address.
    for bad in [r#"{}"#, r#"{"version":""}"#, r#"{"version":"5.2/../x"}"#] {
        assert!(
            phpmyadmin_zip(&serde_json::from_str(bad).unwrap()).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn sqlite_and_mongodb_across_targets() {
    use crate::kit::Target;

    let page = format!(
        "<!--\nPRODUCT,3.54.0,2026/sqlite-tools-osx-x64-3540000.zip,123,{m}\n\
             PRODUCT,3.54.0,2026/sqlite-tools-linux-x64-3540000.zip,456,{l}\n-->",
        m = "b".repeat(64),
        l = "c".repeat(64)
    );
    let (_, osx_path, osx_sha) = sqlite_tools_for(&page, "osx", "x64").unwrap();
    assert_eq!(osx_path, "2026/sqlite-tools-osx-x64-3540000.zip");
    assert_eq!(osx_sha, "b".repeat(64));
    let (_, lin_path, lin_sha) = sqlite_tools_for(&page, "linux", "x64").unwrap();
    assert_eq!(lin_path, "2026/sqlite-tools-linux-x64-3540000.zip");
    assert_eq!(lin_sha, "c".repeat(64));

    let current: Value = serde_json::from_str(&format!(
        r#"{{"versions":[{{"version":"9.0.4","production_release":true,"downloads":[
            {{"target":"macos","arch":"arm64","edition":"base","archive":{{"url":"https://fastdl.mongodb.org/osx/mongodb-macos-arm64-9.0.4.tgz","sha256":"{m}"}}}},
            {{"target":"ubuntu2204","arch":"aarch64","edition":"base","archive":{{"url":"https://fastdl.mongodb.org/linux/mongodb-linux-aarch64-ubuntu2204-9.0.4.tgz","sha256":"{l}"}}}}
        ]}}]}}"#,
        m = "1".repeat(64),
        l = "2".repeat(64)
    ))
    .unwrap();
    let (v, url, _, kind) = mongodb_archive_for(Target::DARWIN_ARM64, &current, None).unwrap();
    assert_eq!(v, "9.0.4");
    assert!(url.ends_with("mongodb-macos-arm64-9.0.4.tgz"));
    assert_eq!(kind, Kind::TarGz);
    let (_, url_lin, _, kind_lin) =
        mongodb_archive_for(Target::LINUX_ARM64, &current, None).unwrap();
    assert!(url_lin.ends_with("mongodb-linux-aarch64-ubuntu2204-9.0.4.tgz"));
    assert_eq!(kind_lin, Kind::TarGz);
}

/// mongosh follows MongoDB's download target (`x64Only`): the x64 build on
/// Windows ARM64, the native one everywhere else.
#[test]
fn mongosh_asset_per_target() {
    use crate::kit::Target;

    let mongodb = crate::setup::catalog::Catalog::get()
        .find("mongodb")
        .unwrap();
    let names = [
        (Target::WIN32_X64, "mongosh-2.5.9-win32-x64.zip"),
        (Target::WIN32_ARM64, "mongosh-2.5.9-win32-x64.zip"),
        (Target::DARWIN_ARM64, "mongosh-2.5.9-darwin-arm64.zip"),
        (Target::DARWIN_X64, "mongosh-2.5.9-darwin-x64.zip"),
        (Target::LINUX_X64, "mongosh-2.5.9-linux-x64.tgz"),
        (Target::LINUX_ARM64, "mongosh-2.5.9-linux-arm64.tgz"),
    ];
    for (target, name) in names {
        let download = crate::router::target::effective_target(target, mongodb);
        let (re, kind) = mongosh_asset_pattern(download);
        let re = pattern(re).unwrap();
        assert!(re.is_match(name), "{target}: {name}");
        for (other, other_name) in names {
            if other_name != name {
                assert!(!re.is_match(other_name), "{target} took {other}");
            }
        }
        let tgz = name.ends_with(".tgz");
        assert_eq!(kind == Kind::TarGz, tgz, "{target}");
    }
    // The arm64 Windows pattern is honest, in case it is ever asked for.
    let (re, _) = mongosh_asset_pattern(Target::WIN32_ARM64);
    assert!(pattern(re)
        .unwrap()
        .is_match("mongosh-2.5.9-win32-arm64.zip"));
}

/// No silent fallback to x64 for SQLite (D-108), except on Windows ARM64
/// where the x64 build runs emulated.
#[test]
fn sqlite_falls_back_to_x64_only_on_windows_arm64() {
    use crate::kit::Target;

    for t in Target::ALL {
        let want = (t == Target::WIN32_ARM64).then_some("x64");
        assert_eq!(sqlite_fallback_arch(t), want, "{t}");
    }
    let page = format!(
        "<!--\nPRODUCT,3.54.0,2026/sqlite-tools-linux-x64-3540000.zip,456,{l}\n-->",
        l = "c".repeat(64)
    );
    assert!(sqlite_tools_for(&page, "linux", "arm64").is_err());
}

/// `/etc/os-release` of real systems (Ubuntu 24.04 is the one of WSL2 on
/// the development PC; the others from the distributions' own files).
const UBUNTU_2404: &str = "PRETTY_NAME=\"Ubuntu 24.04.3 LTS\"\nNAME=\"Ubuntu\"\nVERSION_ID=\"24.04\"\nVERSION_CODENAME=noble\nID=ubuntu\nID_LIKE=debian\nUBUNTU_CODENAME=noble\n";
const DEBIAN_13: &str = "PRETTY_NAME=\"Debian GNU/Linux 13 (trixie)\"\nNAME=\"Debian GNU/Linux\"\nVERSION_ID=\"13\"\nVERSION_CODENAME=trixie\nID=debian\n";
const FEDORA_42: &str =
    "NAME=\"Fedora Linux\"\nVERSION=\"42 (Workstation Edition)\"\nID=fedora\nVERSION_ID=42\n";
const ROCKY_94: &str = "NAME=\"Rocky Linux\"\nVERSION=\"9.4 (Blue Onyx)\"\nID=\"rocky\"\nID_LIKE=\"rhel centos fedora\"\nVERSION_ID=\"9.4\"\n";
const MINT_22: &str = "NAME=\"Linux Mint\"\nVERSION=\"22 (Wilma)\"\nID=linuxmint\nID_LIKE=\"ubuntu debian\"\nVERSION_ID=\"22\"\nUBUNTU_CODENAME=noble\n";
const ARCH: &str = "NAME=\"Arch Linux\"\nID=arch\nBUILD_ID=rolling\n";
const DEBIAN_9: &str = "ID=debian\nVERSION_ID=\"9\"\n";

#[test]
fn linux_distro_from_os_release() {
    assert_eq!(linux_distro(UBUNTU_2404), Some(("ubuntu", vec![24, 4])));
    assert_eq!(linux_distro(DEBIAN_13), Some(("debian", vec![13])));
    assert_eq!(linux_distro(FEDORA_42), Some(("rhel", vec![u64::MAX])));
    assert_eq!(linux_distro(ROCKY_94), Some(("rhel", vec![9, 4])));
    assert_eq!(linux_distro(MINT_22), Some(("ubuntu", vec![24, 4])));
    assert_eq!(linux_distro(ARCH), None);
    assert_eq!(linux_distro(""), None);
}

/// D-112: the Database Tools of this PC's distribution (names of
/// release.json 100.19.1), the oldest Ubuntu build from 20.04 on when it is
/// unknown; only builds of the right CPU are offered by the caller.
#[test]
fn mongodb_tools_build_by_distribution() {
    let x64 = [
        "rhel88",
        "amazon2023",
        "amazon",
        "amazon2",
        "debian10",
        "debian11",
        "debian12",
        "debian13",
        "debian92",
        "rhel10",
        "rhel70",
        "rhel93",
        "suse12",
        "suse15",
        "suse16",
        "ubuntu1604",
        "ubuntu1804",
        "ubuntu2004",
        "ubuntu2204",
        "ubuntu2404",
        "ubuntu2604",
    ];
    let arm64 = [
        "amazon2023",
        "amazon2",
        "rhel10",
        "rhel88",
        "rhel93",
        "ubuntu1604",
        "ubuntu1804",
        "ubuntu2004",
        "ubuntu2204",
        "ubuntu2404",
        "ubuntu2604",
    ];
    let pick = |os: Option<&str>, names: &[&'static str]| mongodb_tools_linux_name(os, names);
    assert_eq!(pick(Some(UBUNTU_2404), &x64), Some("ubuntu2404"));
    assert_eq!(pick(Some(UBUNTU_2404), &arm64), Some("ubuntu2404"));
    assert_eq!(pick(Some(MINT_22), &x64), Some("ubuntu2404"));
    assert_eq!(pick(Some(DEBIAN_13), &x64), Some("debian13"));
    assert_eq!(pick(Some(FEDORA_42), &x64), Some("rhel10"));
    assert_eq!(pick(Some(ROCKY_94), &x64), Some("rhel93"));
    assert_eq!(pick(Some(ROCKY_94), &arm64), Some("rhel93"));
    assert_eq!(
        pick(Some("ID=amzn\nVERSION_ID=\"2023\"\n"), &arm64),
        Some("amazon2023")
    );
    assert_eq!(
        pick(Some("ID=\"sles\"\nVERSION_ID=\"15.6\"\n"), &x64),
        Some("suse15")
    );
    // Newer than every build of its family: the newest one.
    assert_eq!(
        pick(Some("ID=ubuntu\nVERSION_ID=\"28.04\"\n"), &x64),
        Some("ubuntu2604")
    );
    // Debian by major, as RHEL: Debian 9 (`VERSION_ID="9"`) takes
    // debian92, whose glibc it has (the Ubuntu 20.04 build needs a newer
    // one); Debian 10 its own build, not debian92.
    assert_eq!(pick(Some(DEBIAN_9), &x64), Some("debian92"));
    assert_eq!(
        pick(Some("ID=debian\nVERSION_ID=\"10\"\n"), &x64),
        Some("debian10")
    );
    // Debian 8: no build of its family that old.
    assert_eq!(
        pick(Some("ID=debian\nVERSION_ID=\"8\"\n"), &x64),
        Some("ubuntu2004")
    );
    // Debian 12 has no arm64 build; Arch and a kit prepared from another
    // system (no os-release): the oldest Ubuntu from 20.04.
    assert_eq!(pick(Some(DEBIAN_13), &arm64), Some("ubuntu2004"));
    assert_eq!(pick(Some(ARCH), &x64), Some("ubuntu2004"));
    assert_eq!(pick(None, &arm64), Some("ubuntu2004"));
    assert_eq!(pick(None, &["rhel93"]), None);
    // RHEL-like systems by major: CentOS 7 (`VERSION_ID="7"`) takes
    // rhel70, not the Ubuntu fallback; RHEL 8.2 the 8 build.
    assert_eq!(
        pick(Some("ID=\"centos\"\nVERSION_ID=\"7\"\n"), &x64),
        Some("rhel70")
    );
    assert_eq!(
        pick(Some("ID=\"rhel\"\nVERSION_ID=\"8.2\"\n"), &x64),
        Some("rhel88")
    );
    assert_eq!(
        pick(Some("ID=\"rhel\"\nVERSION_ID=\"9.0\"\n"), &arm64),
        Some("rhel93")
    );
}

#[test]
fn mongodb_tools_for_linux_follow_the_distribution() {
    use crate::kit::Target;

    let j: Value = serde_json::from_str(&format!(
        r#"{{"versions":[{{"version":"100.19.1","downloads":[
              {{"name":"ubuntu1604","arch":"x86_64","archive":{{"url":"https://x/ubuntu1604-x86_64.tgz","sha256":"{a}"}}}},
              {{"name":"ubuntu2004","arch":"x86_64","archive":{{"url":"https://x/ubuntu2004-x86_64.tgz","sha256":"{a}"}}}},
              {{"name":"debian12","arch":"x86_64","archive":{{"url":"https://x/debian12-x86_64.tgz","sha256":"{a}"}}}},
              {{"name":"rhel93","arch":"aarch64","archive":{{"url":"https://x/rhel93-aarch64.tgz","sha256":"{a}"}}}},
              {{"name":"ubuntu2004","arch":"arm64","archive":{{"url":"https://x/ubuntu2004-arm64.tgz","sha256":"{a}"}}}},
              {{"name":"ubuntu2404","arch":"arm64","archive":{{"url":"https://x/ubuntu2404-arm64.tgz","sha256":"{a}"}}}}]}}]}}"#,
        a = "a".repeat(64)
    ))
    .unwrap();
    let url = |t: Target, os: Option<&str>| mongodb_tools_for_distro(t, &j, os).unwrap().1;
    assert!(
        url(Target::LINUX_X64, Some("ID=debian\nVERSION_ID=12\n")).ends_with("debian12-x86_64.tgz")
    );
    assert!(url(Target::LINUX_X64, None).ends_with("ubuntu2004-x86_64.tgz"));
    assert!(url(Target::LINUX_ARM64, Some(ROCKY_94)).ends_with("rhel93-aarch64.tgz"));
    assert!(url(Target::LINUX_ARM64, Some(UBUNTU_2404)).ends_with("ubuntu2404-arm64.tgz"));
    // Never another CPU's build.
    assert!(url(Target::LINUX_ARM64, Some("ID=debian\nVERSION_ID=12\n"))
        .ends_with("ubuntu2004-arm64.tgz"));
    assert_eq!(
        mongodb_tools_for_distro(Target::LINUX_X64, &j, None)
            .unwrap()
            .3,
        Kind::TarGz
    );
}

/// D-112: the server on Linux from conda-forge (production lines only;
/// conda-forge has 8.3.7, 8.0.23, 7.0.34 for linux-64 and linux-aarch64 on
/// api.anaconda.org), not movable there; MongoDB's archives elsewhere.
#[test]
fn mongodb_server_from_conda_on_linux() {
    use crate::kit::Target;

    for t in Target::ALL {
        let linux = t.os == Os::Linux;
        assert_eq!(MongoDb.custom(t), linux, "{t}");
        assert_eq!(MongoDb.relocatable(t), !linux, "{t}");
    }
    let versions: Vec<String> = ["8.3.7", "8.0.23", "8.0.20", "7.0.34", "6.0.16"]
        .iter()
        .map(|v| (*v).to_owned())
        .collect();
    assert_eq!(mongodb_conda_lines(&versions), ["8.0", "7.0", "6.0"]);
    assert_eq!(
        mongodb_conda_pick(&versions, None).map(String::as_str),
        Some("8.0.23")
    );
    assert_eq!(
        mongodb_conda_pick(&versions, Some("7.0")).map(String::as_str),
        Some("7.0.34")
    );
    assert_eq!(mongodb_conda_pick(&versions, Some("8.3")), None);
    assert_eq!(
        mongodb_server_version("8.0.23+mongosh2.13.0+tools100.19.1"),
        "8.0.23"
    );
    assert_eq!(mongodb_server_version("8.0.23"), "8.0.23");
    // The catalog's key of the server is the environment's `bin/mongod`.
    let mongodb = crate::setup::catalog::Catalog::get()
        .find("mongodb")
        .unwrap();
    assert_eq!(mongodb.key_for(Target::LINUX_ARM64), "mongodb/bin/mongod");
    assert_eq!(
        mongodb.keys_for(Target::LINUX_X64),
        ["mongosh/bin/mongosh", "mongodb-tools/bin/mongodump"]
    );
}

/// D-112: SQLite on Linux ARM64 from conda-forge (sqlite 3.54.x for
/// linux-aarch64 on api.anaconda.org), available there now; sqlite.org's
/// zips elsewhere, and still no x64 build on Linux ARM64.
#[test]
fn sqlite_from_conda_on_linux_arm64() {
    use crate::kit::Target;

    for t in Target::ALL {
        let conda = t == Target::LINUX_ARM64;
        assert_eq!(Sqlite.custom(t), conda, "{t}");
        assert_eq!(Sqlite.relocatable(t), !conda, "{t}");
    }
    let sqlite = crate::setup::catalog::Catalog::get()
        .find("sqlite")
        .unwrap();
    assert!(sqlite.is_available_on(Target::LINUX_ARM64));
    assert_eq!(sqlite.unavailable_reason(Target::LINUX_ARM64), None);
    assert_eq!(sqlite.key_for(Target::LINUX_ARM64), "sqlite/sqlite3");
    assert!(SQLITE_CONDA_LAUNCHER.starts_with("#!/bin/sh\n"));
    assert!(SQLITE_CONDA_LAUNCHER.contains("exec \"$(dirname \"$0\")/bin/sqlite3\" \"$@\""));
    assert!(!SQLITE_CONDA_LAUNCHER.contains('\r'));
}

/// The launcher really starts the environment's sqlite3, from any folder
/// (Unix only: it is a shell script).
#[cfg(unix)]
#[test]
fn sqlite_conda_launcher_runs_the_environment_program() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = crate::kit::tests::TempDir::new("sqlite-launcher");
    let dir = tmp.0.join("sqlite").join("3.54");
    fs::create_dir_all(dir.join("bin")).unwrap();
    let fake = dir.join("bin").join("sqlite3");
    fs::write(&fake, "#!/bin/sh\necho \"fake $1\"\n").unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    let launcher = dir.join("sqlite3");
    fs::write(&launcher, SQLITE_CONDA_LAUNCHER).unwrap();
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).unwrap();
    let out = std::process::Command::new(&launcher)
        .arg("--version")
        .current_dir(&tmp.0)
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "fake --version"
    );
}

/// The download target of MySQL on each kit target: x64 on Windows ARM64
/// (`x64Only`), native elsewhere.
fn mysql_download_target(t: Target) -> Target {
    let c = crate::setup::catalog::Catalog::get().find("mysql").unwrap();
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
        let p = crate::fsx::join_rel(t, f);
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
        assert!(!crate::fsx::join_rel(t, gone).exists(), "{gone}");
    }
    for kept in [
        "bin/mysqld.exe",
        "lib/plugin/keep.so",
        "share/english/errmsg.sys",
    ] {
        assert!(crate::fsx::join_rel(t, kept).exists(), "{kept}");
    }
}
