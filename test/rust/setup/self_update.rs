use super::*;
use crate::kit::{tests::TempDir, Arch};

fn release(tag: &str, asset: &str) -> Release {
    Release {
        tag_name: tag.into(),
        draft: false,
        prerelease: false,
        assets: vec![GhAsset {
            name: asset.into(),
            browser_download_url: String::new(),
            digest: None,
        }],
    }
}

#[test]
fn versions_and_choice() {
    assert_eq!(parse_version("app-v1.2.3"), Some((1, 2, 3)));
    assert_eq!(parse_version("1.10.0"), Some((1, 10, 0)));
    assert_eq!(parse_version("app-v1.2"), None);
    assert_eq!(parse_version("v1.2.3.4"), None);
    let mut pre = release("app-v9.0.0", "Atollo-x64.exe");
    pre.prerelease = true;
    let list = vec![
        release("app-v1.0.1", "Atollo-x64.exe"),
        release("app-v1.2.0", "Atollo-x64.exe"),
        release("app-v1.3.0", "Atollo-arm64.exe"),
        release("node-v1", "Atollo-x64.exe"),
        pre,
    ];
    assert_eq!(
        newest(&list, "1.0.0", "Atollo-x64.exe").map(|r| r.tag_name.as_str()),
        Some("app-v1.2.0")
    );
    assert!(newest(&list, "1.2.0", "Atollo-x64.exe").is_none());
    assert_eq!(
        newest(&list, "1.0.0", "Atollo-arm64.exe").map(|r| r.tag_name.as_str()),
        Some("app-v1.3.0")
    );
}

#[test]
fn swap_and_cleanup() {
    let tmp = TempDir::new("self-update");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let exe = tmp.0.join("Atollo.exe");
    fs::write(&exe, b"old").unwrap();
    // Nothing staged: nothing changes.
    assert!(!apply_staged(&kit, &exe).unwrap());
    fs::create_dir_all(update_dir(&kit)).unwrap();
    fs::write(staged_exe(&kit), b"new").unwrap();
    let hash = verify::sha256_hex(&staged_exe(&kit)).unwrap();
    let info = Staged {
        version: "999.0.0".into(),
        sha256: hash,
    };
    fs::write(staged_info(&kit), serde_json::to_string(&info).unwrap()).unwrap();
    assert_eq!(
        Updater::new(kit.clone()).status().staged.as_deref(),
        Some("999.0.0")
    );
    assert!(apply_staged(&kit, &exe).unwrap());
    assert_eq!(fs::read(&exe).unwrap(), b"new");
    assert_eq!(fs::read(tmp.0.join("Atollo.old.exe")).unwrap(), b"old");
    assert!(!staged_exe(&kit).exists() && !staged_info(&kit).exists());
    remove_old(&exe);
    assert!(!tmp.0.join("Atollo.old.exe").exists());

    // A staged file changed after the check is never used.
    fs::write(staged_exe(&kit), b"evil").unwrap();
    fs::write(staged_info(&kit), serde_json::to_string(&info).unwrap()).unwrap();
    assert!(apply_staged(&kit, &exe).is_err());
    assert_eq!(fs::read(&exe).unwrap(), b"new");
    assert!(!staged_exe(&kit).exists());
}
