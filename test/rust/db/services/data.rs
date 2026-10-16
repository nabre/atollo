use super::*;
use crate::db::services::server_lines;
use crate::kit::{tests::TempDir, Arch};

/// `id` installed at `version`, the default folder of its line.
fn fake_server(kit: &Kit, id: &str, version: &str) {
    let c = Catalog::get().find(id).unwrap();
    let dir = crate::versions::install_dir(kit, c, version);
    let folder = c.folder.as_deref().unwrap();
    for k in std::iter::once(&c.key).chain(&c.keys) {
        if let Some(rel) = k.strip_prefix(folder) {
            let file = dir.join(rel.trim_start_matches('/').replace('/', "\\"));
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, b"").unwrap();
        }
    }
    c.save_version(kit, version).unwrap();
}

#[test]
fn each_version_has_its_data() {
    let tmp = TempDir::new("services-lines");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    // Data of an older kit, then the server's own folder per version.
    fake_server(&kit, "mariadb", "11.8.3");
    let root = kit.data().join(r"db\mariadb");
    fs::create_dir_all(root.join("mysql")).unwrap();
    fs::write(root.join("ibdata1"), b"x").unwrap();
    migrate_data(&kit);
    assert!(root.join(r"11.8\mysql").is_dir() && !root.join("mysql").exists());
    assert_eq!(data_dir(&kit, "mariadb"), root.join("11.8"));

    // A newer version starts with a copy; the old data stays.
    let c = Catalog::get().find("mariadb").unwrap();
    let newer = crate::versions::saved_dir(&kit, c, "12.3");
    fs::create_dir_all(newer.join("bin")).unwrap();
    for k in ["mariadbd.exe", "mariadb.exe"] {
        fs::write(newer.join("bin").join(k), b"").unwrap();
    }
    assert_eq!(server_line(&kit, "mariadb").as_deref(), Some("12.3"));
    assert_eq!(server_lines(&kit, "mariadb"), ["12.3", "11.8"]);
    assert_eq!(
        seed(&kit, "mariadb", "12.3").unwrap().as_deref(),
        Some("11.8")
    );
    assert!(root.join(r"12.3\ibdata1").is_file() && root.join(r"11.8\ibdata1").is_file());
    assert_eq!(
        seed(&kit, "mariadb", "12.3").unwrap(),
        None,
        "only the first time"
    );
    // Back to the older version: its own data, a setting of the server.
    let s = Services::new(kit.clone());
    s.set_line("mariadb", Some("11.8")).unwrap();
    assert_eq!(data_dir(&kit, "mariadb"), root.join("11.8"));
    assert!(
        s.set_line("mariadb", Some("10.6")).is_err(),
        "not installed"
    );
    assert_eq!(
        bin(&kit, "mariadb", "mariadbd.exe")
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap(),
        "11.8"
    );

    // MongoDB: one major at a time.
    let mongo = kit.data().join(r"db\mongodb");
    fs::create_dir_all(mongo.join("7.0")).unwrap();
    fs::write(mongo.join(r"7.0\WiredTiger"), b"").unwrap();
    assert_eq!(
        seed(&kit, "mongodb", "9.0").unwrap(),
        None,
        "7.0 cannot jump to 9.0"
    );
    assert_eq!(
        seed(&kit, "mongodb", "8.0").unwrap().as_deref(),
        Some("7.0")
    );
}
