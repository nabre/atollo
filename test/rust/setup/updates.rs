use super::*;

use crate::kit::Arch;
use crate::test_support::prepared_kit;

/// The kit given by ATOLLO_TEST_KIT asked for real: each installed
/// component's source answers, and what is behind is listed.
#[test]
fn real_check_of_a_kit() {
    let Ok(root) = std::env::var("ATOLLO_TEST_KIT") else {
        eprintln!("ATOLLO_TEST_KIT not set: skipped");
        return;
    };
    let kit = Kit::at(std::path::PathBuf::from(root), Arch::detect());
    let env = KitEnv::build(
        &kit,
        &crate::common::settings::Settings::default(),
        "en",
        &crate::env::Host::capture(),
    );
    let latest = check(&kit, &env, None).unwrap();
    assert!(latest.checked > 0);
    for c in watched(&kit) {
        eprintln!(
            "{}: installed {:?}, latest {:?}",
            c.id,
            c.installed_version(&kit),
            latest.versions.get(&c.id)
        );
    }
    for u in pending(&kit, &crate::levels::VmState::On).items {
        eprintln!(
            "new: {} {} -> {} ({})",
            u.id, u.installed, u.latest, u.notes
        );
    }
}

#[test]
fn versions_compared_by_numbers() {
    assert!(newer("2.10.0", "2.9.4"));
    assert!(newer("24.11.1", "24.11.0"));
    assert!(newer("1.0.1", "1.0"));
    assert!(!newer("2.9.4", "2.10.0"));
    assert!(!newer("3.13.1", "3.13.1"));
    // A source that answers an older version (a rollback) is no update.
    assert!(!newer("8.3.9", "8.4.1"));
}

#[test]
fn notes_of_a_version() {
    let mut c = Catalog::get().find("gh").unwrap().clone();
    c.notes = Some("https://github.com/cli/cli/releases/tag/v{version}".into());
    assert_eq!(
        notes_url(&c, "2.103.0"),
        "https://github.com/cli/cli/releases/tag/v2.103.0"
    );
    c.notes = None;
    assert_eq!(notes_url(&c, "2.103.0"), c.home);
}

/// Only the installed components newer at their source are listed; nothing
/// is asked to list them.
#[test]
fn pending_updates() {
    let (_tmp, kit) = prepared_kit("updates");
    assert!(stale(&kit));
    let empty = pending(&kit, &crate::levels::VmState::On);
    assert_eq!(empty.checked, None);
    assert!(empty.items.is_empty());

    // gh installed at 2.100.0: its version, then its key in that version's
    // folder.
    let gh = Catalog::get().find("gh").unwrap();
    gh.save_version(&kit, "2.100.0").unwrap();
    for p in gh.key_paths(&kit) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "x").unwrap();
    }
    assert!(gh.is_installed(&kit));
    let latest = Latest {
        checked: now(),
        versions: [
            ("gh".to_owned(), "2.103.0".to_owned()),
            // Not installed: not listed.
            ("caddy".to_owned(), "9.9.9".to_owned()),
        ]
        .into(),
    };
    crate::common::fsx::write_json(&file(&kit), &latest).unwrap();
    assert!(!stale(&kit));
    let list = pending(&kit, &crate::levels::VmState::On);
    assert_eq!(list.checked, Some(latest.checked));
    assert_eq!(list.items.len(), 1);
    let u = &list.items[0];
    assert_eq!(
        (u.id.as_str(), u.installed.as_str(), u.latest.as_str()),
        ("gh", "2.100.0", "2.103.0")
    );
    assert!(u.notes.starts_with("https://"));
    // Up to date: nothing.
    gh.save_version(&kit, "2.103.0").unwrap();
    assert!(pending(&kit, &crate::levels::VmState::On).items.is_empty());
}
