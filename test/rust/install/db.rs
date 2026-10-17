use super::*;

#[test]
fn mariadb_newest_lts_and_its_zip() {
    let index: Value = serde_json::from_str(
        r#"{"major_releases":[
              {"release_id":"13.0","release_status":"Stable","release_support_type":"Rolling"},
              {"release_id":"12.3","release_status":"Stable","release_support_type":"Long Term Support"},
              {"release_id":"11.8","release_status":"Stable","release_support_type":"Long Term Support"},
              {"release_id":"13.2","release_status":"Preview","release_support_type":null}]}"#,
    )
    .unwrap();
    assert_eq!(mariadb_major(&index).unwrap(), "12.3");
    let latest: Value = serde_json::from_str(&format!(
        r#"{{"releases":{{"12.3.4":{{"files":[
              {{"file_name":"repo/","os":null,"cpu":null,"package_type":null,"checksum":{{}}}},
              {{"file_name":"mariadb-12.3.4-winx64-debugsymbols.zip","os":"Windows","cpu":"x86_64","package_type":"ZIP file","checksum":{{"sha256sum":"{a}"}}}},
              {{"file_name":"mariadb-12.3.4-winx64.zip","os":"Windows","cpu":"x86_64","package_type":"ZIP file","checksum":{{"sha256sum":"{b}"}}}}]}}}}}}"#,
        a = "a".repeat(64),
        b = "b".repeat(64)
    ))
    .unwrap();
    let (v, name, sha) = mariadb_zip(&latest).unwrap();
    assert_eq!(v, "12.3.4");
    assert_eq!(name, "mariadb-12.3.4-winx64.zip");
    assert_eq!(sha, "b".repeat(64));
}

/// BUG-V3-04: on Linux only a Linux file of the right CPU, the
/// `linux-systemd` build first, never debug symbols; macOS has no build.
#[test]
fn mariadb_linux_tarball_and_no_macos() {
    use crate::kit::Target;

    let latest: Value = serde_json::from_str(&format!(
        r#"{{"releases":{{"11.4.13":{{"files":[
              {{"file_name":"mariadb-11.4.13-macos-x86_64.tar.gz","os":"macOS","cpu":"x86_64","package_type":"gzipped tar file","checksum":{{"sha256sum":"{m}"}}}},
              {{"file_name":"mariadb-11.4.13-linux-x86_64-debugsymbols.tar.gz","os":"Linux","cpu":"x86_64","package_type":"gzipped tar file","checksum":{{"sha256sum":"{d}"}}}},
              {{"file_name":"mariadb-11.4.13-linux-glibc_214-x86_64.tar.gz","os":"Linux","cpu":"x86_64","package_type":"gzipped tar file","checksum":{{"sha256sum":"{g}"}}}},
              {{"file_name":"mariadb-11.4.13-linux-systemd-aarch64.tar.gz","os":"Linux","cpu":"aarch64","package_type":"gzipped tar file","checksum":{{"sha256sum":"{a}"}}}},
              {{"file_name":"mariadb-11.4.13-linux-systemd-x86_64.tar.gz","os":"Linux","cpu":"x86_64","package_type":"gzipped tar file","checksum":{{"sha256sum":"{s}"}}}}]}}}}}}"#,
        m = "1".repeat(64),
        d = "2".repeat(64),
        g = "3".repeat(64),
        a = "4".repeat(64),
        s = "5".repeat(64)
    ))
    .unwrap();
    let (v, name, sha, kind) = mariadb_archive_for(Target::LINUX_X64, &latest).unwrap();
    assert_eq!(v, "11.4.13");
    assert_eq!(name, "mariadb-11.4.13-linux-systemd-x86_64.tar.gz");
    assert_eq!(sha, "5".repeat(64));
    assert_eq!(kind, Kind::TarGz);
    let arm = mariadb_archive_for(Target::LINUX_ARM64, &latest).unwrap();
    assert_eq!(arm.1, "mariadb-11.4.13-linux-systemd-aarch64.tar.gz");
    for t in [Target::DARWIN_X64, Target::DARWIN_ARM64] {
        assert!(mariadb_archive_for(t, &latest).is_err(), "{t}");
    }

    // Without a systemd build, another Linux one (never macOS).
    let only_glibc: Value = serde_json::json!({"releases": {"11.4.13": {"files": [
        {"file_name": "mariadb-11.4.13-macos-x86_64.tar.gz", "os": "macOS", "cpu": "x86_64",
         "package_type": "gzipped tar file", "checksum": {"sha256sum": "1".repeat(64)}},
        {"file_name": "mariadb-11.4.13-linux-glibc_214-x86_64.tar.gz", "os": "Linux", "cpu": "x86_64",
         "package_type": "gzipped tar file", "checksum": {"sha256sum": "3".repeat(64)}}]}}});
    let (_, name, _, _) = mariadb_archive_for(Target::LINUX_X64, &only_glibc).unwrap();
    assert_eq!(name, "mariadb-11.4.13-linux-glibc_214-x86_64.tar.gz");
}

/// MariaDB publishes Windows x64 and Linux x86_64 builds only: the
/// catalog offers it there (x64 emulated on Windows ARM64), and the page
/// says "not available" elsewhere.
#[test]
fn mariadb_targets_follow_the_published_builds() {
    use crate::kit::Target;
    use crate::setup::catalog::Catalog;

    let c = Catalog::get().find("mariadb").unwrap();
    for t in [Target::WIN32_X64, Target::WIN32_ARM64, Target::LINUX_X64] {
        assert!(c.is_available_on(t), "{t}");
    }
    for t in [
        Target::DARWIN_ARM64,
        Target::DARWIN_X64,
        Target::LINUX_ARM64,
    ] {
        assert!(!c.is_available_on(t), "{t}");
    }
}

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
fn mariadb_lines_are_the_stable_lts() {
    let index = serde_json::json!({"major_releases": [
        {"release_id": "13.0", "release_status": "Stable", "release_support_type": "Short Term Support"},
        {"release_id": "12.3", "release_status": "Stable", "release_support_type": "Long Term Support"},
        {"release_id": "10.11", "release_status": "Stable", "release_support_type": "Long Term Support"},
        {"release_id": "11.8", "release_status": "Stable", "release_support_type": "Long Term Support"},
        {"release_id": "13.1", "release_status": "RC", "release_support_type": "Long Term Support"}
    ]});
    assert_eq!(mariadb_lines(&index), ["12.3", "11.8", "10.11"]);
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

/// T2.5.1: the Database Tools of this PC's distribution (names of
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
    // Debian 12 has no arm64 build; Arch, an old Debian and a kit prepared
    // from another system (no os-release): the oldest Ubuntu from 20.04.
    assert_eq!(pick(Some(DEBIAN_13), &arm64), Some("ubuntu2004"));
    assert_eq!(pick(Some(ARCH), &x64), Some("ubuntu2004"));
    assert_eq!(pick(Some(DEBIAN_9), &x64), Some("ubuntu2004"));
    assert_eq!(pick(None, &arm64), Some("ubuntu2004"));
    assert_eq!(pick(None, &["rhel93"]), None);
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

/// T2.5.1: the server on Linux from conda-forge (production lines only;
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

/// T2.5.3: SQLite on Linux ARM64 from conda-forge (sqlite 3.54.x for
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
