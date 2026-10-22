use super::super::sqlite::sqlite_tools_for;
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

    let mongodb = crate::setup::Catalog::get().find("mongodb").unwrap();
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
    let mongodb = crate::setup::Catalog::get().find("mongodb").unwrap();
    assert_eq!(mongodb.key_for(Target::LINUX_ARM64), "mongodb/bin/mongod");
    assert_eq!(
        mongodb.keys_for(Target::LINUX_X64),
        ["mongosh/bin/mongosh", "mongodb-tools/bin/mongodump"]
    );
}
