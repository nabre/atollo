use super::*;

use crate::kit::tests::TempDir;
use crate::test_support::prepared_kit;

#[test]
fn free_space_levels() {
    assert_eq!(space(LOW - 1, "it").level, Level::Err);
    assert!(space(LOW - 1, "it").fix.is_some());
    assert_eq!(space(LOWISH - 1, "it").level, Level::Warn);
    assert_eq!(space(LOWISH, "it").level, Level::Ok);
    assert!(space(3 << 30, "en").text.contains("3.0"));
}

#[test]
fn risky_characters_of_a_path() {
    assert!(risky_chars(r"D:\Kit\Atollo").is_empty());
    assert!(risky_chars("/media/usb/Atollo kit-2").is_empty());
    assert_eq!(risky_chars(r"D:\R&D\O’Brien!\x"), ['!', '&', '’']);
    assert_eq!(risky_chars(r"C:\Users\Zoë\Città"), ['à', 'ë']);
    assert_eq!(risky_chars("a%b^c%"), ['%', '^']);
}

/// The pasted command never runs a piece of the path.
#[test]
fn command_to_take_the_files_back() {
    assert_eq!(
        owners_fix(false, "/media/O'Brien/kit/data").unwrap(),
        r#"sudo chown -R "$(id -un)" '/media/O'\''Brien/kit/data'"#
    );
    assert_eq!(
        owners_fix(false, "/k/$(rm -rf ~)`x`").unwrap(),
        r#"sudo chown -R "$(id -un)" '/k/$(rm -rf ~)`x`'"#
    );
    assert_eq!(
        owners_fix(true, r"D:\Kit\data").unwrap(),
        r#"takeown /f "D:\Kit\data" /r /d y && icacls "D:\Kit\data" /reset /t"#
    );
    assert!(owners_fix(true, r"D:\%PATH%\data").is_none());
}

#[test]
fn files_the_account_reads() {
    let tmp = TempDir::new("health-read");
    fs::create_dir_all(tmp.0.join("a").join("b")).unwrap();
    fs::write(tmp.0.join("a").join("b").join("x"), "x").unwrap();
    assert!(unreadable(&tmp.0, 100).is_empty());
}

/// Every check answers on a fresh kit, in its order, with a text.
#[test]
fn a_fresh_kit_is_checked() {
    let (_tmp, kit) = prepared_kit("health-kit");
    let all = check(&kit, "it");
    let ids: Vec<&str> = all.iter().map(|f| f.id).collect();
    assert_eq!(ids[0], "disk");
    // The space when the drive is known.
    assert_eq!(ids.contains(&"space"), kit.volume().is_some());
    assert!(ids.contains(&"path") && ids.contains(&"owners"));
    assert_eq!(ids.contains(&"libs"), kit.os().is_linux());
    assert!(all.iter().all(|f| !f.text.is_empty()));
    let owners = all.iter().find(|f| f.id == "owners").unwrap();
    assert_eq!(owners.level, Level::Ok);
}
