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
