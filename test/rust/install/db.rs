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
