use super::*;
use crate::kit::tests::TempDir;

#[test]
fn php_ini_from_template() {
    let template = ";extension_dir = \"ext\"\r\n;extension=curl\r\n;extension=ldap\r\n;date.timezone =\r\n;curl.cainfo =\r\n;openssl.cafile=\r\n";
    let ini = php_ini(template).unwrap();
    assert!(
        ini.contains("extension_dir = \"${CP_PHP}\\ext\"\r\n"),
        "{ini}"
    );
    assert!(ini.contains("\r\nextension=curl\r\n"));
    assert!(
        ini.contains(";extension=ldap"),
        "not in the list: stays off"
    );
    assert!(ini.contains("date.timezone = Europe/Rome"));
    assert!(ini.contains("curl.cainfo = \"${CP_PHP}\\extras\\ssl\\cacert.pem\""));
    assert!(ini.contains("openssl.cafile = \"${CP_PHP}\\extras\\ssl\\cacert.pem\""));
    // The php.ini of an older kit follows the PHP in use too.
    assert_eq!(
        php_ini_paths("extension_dir = \"${CP_APPS}\\php\\ext\"\r\n"),
        "extension_dir = \"${CP_PHP}\\ext\"\r\n"
    );
}

/// PHP before 7.2 names its extensions `php_curl.dll`, before 8.0 GD is
/// `gd2`: each line's template gets the same extensions on.
#[test]
fn php_ini_from_older_templates() {
    let ini = php_ini(";extension=php_curl.dll\r\n;extension=php_gd2.dll\r\n;extension=php_ldap.dll\r\n;extension=gd2\r\n").unwrap();
    assert_eq!(
        ini,
        "extension=php_curl.dll\r\nextension=php_gd2.dll\r\n;extension=php_ldap.dll\r\nextension=gd2\r\n"
    );
}

/// A PHP folder of `kit` (its key for the kit's target: `php.exe` on
/// Windows, `bin/php` elsewhere) with its `php.ini-development`.
fn fake_php(kit: &crate::kit::Kit, dir: &Path, template: &str) {
    let php = crate::setup::Catalog::get().find("php").unwrap();
    let key = php.key_for(kit.target());
    let file = crate::common::fsx::join_rel(dir, key.strip_prefix("php/").unwrap());
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, b"").unwrap();
    fs::write(dir.join("php.ini-development"), template).unwrap();
}

/// Defect 10: one php.ini per line of PHP, from that line's template; the
/// single php.ini of an older kit becomes the default line's, with the
/// user's changes. A Windows kit on every host: that php.ini only existed
/// there (the Unix layout: `php_ini_shared_kit_both_orders`).
#[test]
fn one_php_ini_per_line() {
    use crate::kit::{Kit, Target};
    use crate::setup::Catalog;

    let tmp = TempDir::new("php-ini-lines");
    let kit = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    kit.prepare().unwrap();
    let data = kit.data();
    let php = Catalog::get().find("php").unwrap();
    let version = "8.4.13 (Composer 2.8.12)";
    let main = crate::versions::install_dir(&kit, php, version);
    fake_php(&kit, &main, ";extension=curl\r\n");
    php.save_version(&kit, version).unwrap();
    let old = crate::versions::saved_dir(&kit, php, "5.6");
    fake_php(&kit, &old, ";extension=php_curl.dll\r\n");
    crate::versions::write_version(&old, "5.6.40").unwrap();
    let legacy = data.join("php").join("php.ini");
    fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    fs::write(&legacy, "memory_limit = 1G\r\n").unwrap();

    assert_eq!(php_ini_dir(&data, &main), data.join("php").join("8.4"));
    assert_eq!(php_ini_dir(&data, &old), data.join("php").join("5.6"));
    assert_eq!(
        php_ini_dir(&data, &kit.apps().join("php")),
        data.join("php"),
        "an older kit's layout"
    );

    migrate_php_ini(&kit).unwrap();
    let main_ini = data.join("php").join("8.4").join("php.ini");
    let old_ini = data.join("php").join("5.6").join("php.ini");
    assert_eq!(
        fs::read_to_string(&main_ini).unwrap(),
        "memory_limit = 1G\r\n"
    );
    assert!(!legacy.exists());
    assert_eq!(
        fs::read_to_string(&old_ini).unwrap(),
        "extension=php_curl.dll\r\n"
    );

    // Then they are the user's: never written again.
    fs::write(&old_ini, "mine").unwrap();
    migrate_php_ini(&kit).unwrap();
    ensure_php_ini(&data, &old).unwrap();
    assert_eq!(fs::read_to_string(&old_ini).unwrap(), "mine");
    assert_eq!(
        fs::read_to_string(&main_ini).unwrap(),
        "memory_limit = 1G\r\n"
    );

    // The environment points at the php.ini of the PHP it names.
    let mut env = crate::env::KitEnv::default();
    env.set_php(&data, &old);
    let ini_dir = data.join("php").join("5.6");
    assert_eq!(env.get("PHPRC"), Some(ini_dir.to_str().unwrap()));
    assert_eq!(
        env.get("CP_PHPRC_FWD").unwrap(),
        ini_dir.to_str().unwrap().replace('\\', "/")
    );
}

/// The folder of the php.ini for each of the four targets: Windows keeps
/// `data/php/<line>`, macOS and Linux get `data/php/<target>/<line>`.
#[test]
fn php_ini_dir_per_target() {
    use crate::kit::{Kit, Target};
    let tmp = TempDir::new("php-ini-targets");
    for target in Target::ALL {
        let kit = Kit::at_target(tmp.0.join("kit"), target);
        let data = kit.data();
        let dir = kit.apps().join("php").join("8.5");
        let want = if target.os == crate::kit::Os::Win32 {
            data.join("php").join("8.5")
        } else {
            data.join("php").join(target.as_str()).join("8.5")
        };
        assert_eq!(php_ini_dir(&data, &dir), want, "{}", target.as_str());
    }
    // A kit folder named like a target does not count: only below data/.
    let odd = Kit::at_target(tmp.0.join("linux-x64"), Target::WIN32_X64);
    let dir = odd.apps().join("php").join("8.4");
    assert_eq!(
        php_ini_dir(&odd.data(), &dir),
        odd.data().join("php").join("8.4")
    );
}

/// Review of D-112: one kit with PHP 8.5 for Windows and for Linux gets two
/// php.ini, in either order of install: Windows with its extension= lines,
/// Linux with none. The single php.ini of an older kit stays Windows'.
#[test]
fn php_ini_shared_kit_both_orders() {
    use crate::kit::{Kit, Target};
    for windows_first in [true, false] {
        let tmp = TempDir::new("php-ini-shared");
        let win = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
        let linux = win.for_target(Target::LINUX_X64);
        win.prepare().unwrap();
        let data = win.data();
        let win_dir = win.apps().join("php").join("8.5");
        let linux_dir = linux.apps().join("php").join("8.5");
        fake_php(&win, &win_dir, ";extension=curl\r\n");
        fake_php(&linux, &linux_dir, "");
        fs::remove_file(linux_dir.join("php.ini-development")).unwrap();
        let order = if windows_first {
            [&win_dir, &linux_dir]
        } else {
            [&linux_dir, &win_dir]
        };
        for dir in order {
            ensure_php_ini(&data, dir).unwrap();
        }
        let win_ini = fs::read_to_string(data.join("php").join("8.5").join("php.ini")).unwrap();
        let linux_ini = fs::read_to_string(
            data.join("php")
                .join("linux-x64")
                .join("8.5")
                .join("php.ini"),
        )
        .unwrap();
        assert_eq!(
            win_ini, "extension=curl\r\n",
            "windows_first={windows_first}"
        );
        assert_eq!(linux_ini, php_ini_unix(), "windows_first={windows_first}");
        assert!(!linux_ini
            .lines()
            .any(|l| l.trim_start().starts_with("extension")));

        // PHPRC of each system names its own folder.
        let mut env = crate::env::KitEnv::default();
        env.set_php(&data, &linux_dir);
        let want = data.join("php").join("linux-x64").join("8.5");
        assert_eq!(env.get("PHPRC"), Some(want.to_str().unwrap()));
    }

    // An older kit's data/php/php.ini never becomes a Unix PHP's.
    let tmp = TempDir::new("php-ini-legacy-unix");
    let linux = Kit::at_target(tmp.0.clone(), Target::LINUX_X64);
    linux.prepare().unwrap();
    let php = crate::setup::Catalog::get().find("php").unwrap();
    let version = "8.5.8 (Composer 2.8.12)";
    let main = crate::versions::install_dir(&linux, php, version);
    fake_php(&linux, &main, "");
    fs::remove_file(main.join("php.ini-development")).unwrap();
    php.save_version(&linux, version).unwrap();
    let legacy = linux.data().join("php").join("php.ini");
    fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    fs::write(&legacy, "extension=curl\r\n").unwrap();
    migrate_php_ini(&linux).unwrap();
    assert_eq!(fs::read_to_string(&legacy).unwrap(), "extension=curl\r\n");
    let ini = php_ini_dir(&linux.data(), &main).join("php.ini");
    assert_eq!(fs::read_to_string(ini).unwrap(), php_ini_unix());
}

/// D-112: the Unix `php.ini`, no `extension=` line (built-in extensions),
/// the CA bundle with `/`, the time zone of the Windows one; written once
/// for a folder of static-php-cli (no template), then the user's.
#[test]
fn php_ini_for_unix() {
    let ini = php_ini_unix();
    assert!(
        !ini.lines().any(|l| l.trim_start().starts_with("extension")),
        "{ini}"
    );
    assert!(!ini.contains('\\') && !ini.contains('\r'));
    assert!(ini.contains("date.timezone = Europe/Rome\n"));
    assert!(ini.contains("curl.cainfo = \"${CP_PHP}/extras/ssl/cacert.pem\"\n"));
    assert!(ini.contains("openssl.cafile = \"${CP_PHP}/extras/ssl/cacert.pem\"\n"));
    assert!(ini.contains("display_errors = On\n") && ini.contains("error_reporting = E_ALL\n"));
    // Where the user looks (review of D-112, issue #4): the kit's build has
    // pdo_sqlite, the bulk build only SQLite through PDO.
    assert!(ini.contains("own build") && ini.contains("has pdo_sqlite"));
    assert!(ini.contains("not list pdo_sqlite") && ini.contains("ext-pdo_sqlite"));
    assert!(ini
        .lines()
        .filter(|l| l.contains("pdo_sqlite"))
        .all(|l| l.starts_with(';')));
    // The Windows one keeps its `\` and its extensions.
    let win = php_ini(";extension=curl\n;curl.cainfo =\n").unwrap();
    assert!(win.contains("extension=curl") && win.contains(r"\extras\ssl\cacert.pem"));

    let tmp = TempDir::new("php-ini-unix");
    let data = tmp.0.join("data");
    let dir = tmp.0.join("apps").join("php").join("8.4");
    fs::create_dir_all(dir.join("bin")).unwrap();
    fs::write(dir.join("bin").join("php"), b"elf").unwrap();
    ensure_php_ini(&data, &dir).unwrap();
    let path = data.join("php").join("8.4").join("php.ini");
    assert_eq!(fs::read_to_string(&path).unwrap(), ini);
    fs::write(&path, "; mine\n").unwrap();
    ensure_php_ini(&data, &dir).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "; mine\n");
}
