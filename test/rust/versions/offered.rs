use super::*;
use crate::kit::{tests::TempDir, Arch};
use crate::setup::catalog::Catalog;
use crate::versions::tests::install_fake;
use crate::versions::{install_dir, saved_dir, tools};

#[test]
fn stale_lists_are_asked_for_again() {
    let tmp = TempDir::new("versions-stale");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    // Installed or not, every list that can be asked for is missing.
    let missing = stale_offers(&kit);
    assert!(missing.iter().any(|id| id == "mariadb"));
    assert!(missing.iter().any(|id| id == "mongodb"));

    save_offered(&kit, "mariadb", &["11.8".into()]).unwrap();
    assert!(!stale_offers(&kit).iter().any(|id| id == "mariadb"));

    // Older than a day: asked for again.
    let mut all = offered(&kit);
    all.get_mut("mariadb").unwrap().checked -= STALE_SECS + 1;
    crate::fsx::write_json(&offered_file(&kit), &all).unwrap();
    assert!(stale_offers(&kit).iter().any(|id| id == "mariadb"));

    // Apache's list of a kit with one folder per patch: asked for again.
    let mut all = offered(&kit);
    all.insert(
        "apache".into(),
        Offered {
            lines: vec!["2.4.69".into()],
            checked: all["mariadb"].checked + STALE_SECS,
        },
    );
    crate::fsx::write_json(&offered_file(&kit), &all).unwrap();
    assert!(stale_offers(&kit).iter().any(|id| id == "apache"));
    save_offered(&kit, "apache", &["2.4.69".into(), "2.4".into()]).unwrap();
    assert_eq!(offered(&kit)["apache"].lines, ["2.4"]);
}

#[test]
fn offered_lines_and_downloads() {
    let tmp = TempDir::new("versions-offered");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let node = Catalog::get().find("node").unwrap();

    // Nothing installed: nothing to download.
    assert!(!to_download(&kit, "node", "20.18").unwrap());

    let main = install_dir(&kit, node, "v22.11.0");
    install_fake(&kit, node, &main);
    node.save_version(&kit, "v22.11.0").unwrap();
    let old = saved_dir(&kit, node, "20.18");
    install_fake(&kit, node, &old);
    assert!(!to_download(&kit, "node", "22.11").unwrap(), "the default");
    assert!(!to_download(&kit, "node", "20.18").unwrap(), "saved");
    assert!(to_download(&kit, "node", "18.20").unwrap());
    assert!(to_download(&kit, "node", "x").is_err());

    // Kept per component, valid lines only, newest first.
    save_offered(&kit, "node", &["20.18".into(), "24.1".into(), "bad".into()]).unwrap();
    save_offered(&kit, "claude", &["1.0".into()]).unwrap();
    let all = offered(&kit);
    assert_eq!(all.keys().collect::<Vec<_>>(), ["node"]);
    assert_eq!(all["node"].lines, ["24.1", "20.18"]);
    assert!(all["node"].checked > 0);

    crate::projects::settings::update(&kit, "app", |s| {
        s.versions.insert("node".into(), "20.18".into());
    });
    let tool = tools(&kit).into_iter().find(|t| t.id == "node").unwrap();
    assert_eq!(tool.available, ["24.1", "20.18"]);
    assert!(tool.checked.is_some());
    assert_eq!(tool.projects.get("20.18").map(Vec::len), Some(1));
}
