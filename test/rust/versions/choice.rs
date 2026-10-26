use super::*;
use crate::kit::tests::TempDir;
use crate::setup::Catalog;
use crate::test_support::prepared_kit;
use crate::versions::tests::install_fake;
use crate::versions::{default_dir, install_dir, remove, saved_dir, write_version, Line};

/// The environment of a program of `project` on the PC (level 2, D-179).
fn project_env(kit: &Kit, project: &str, env: KitEnv) -> KitEnv {
    crate::levels::env::pc_environment(kit, env, crate::levels::Level::Project, Some(project), None)
}

#[test]
fn choices_rewrite_the_environment() {
    let (_tmp, kit) = prepared_kit("versions");
    let node = Catalog::get().find("node").unwrap();
    let main = install_dir(&kit, node, "v22.11.0");
    assert!(main.ends_with(std::path::Path::new("node").join("22.11")));
    install_fake(&kit, node, &main);
    node.save_version(&kit, "v22.11.0").unwrap();
    assert_eq!(default_line(&kit, node).as_deref(), Some("22.11"));
    assert_eq!(default_dir(&kit, node), main);
    let node_exe = if cfg!(windows) {
        main.join("node.exe")
    } else {
        main.join("bin").join("node")
    };
    assert_eq!(crate::setup::kit_path(&kit, "node/node.exe"), node_exe);
    assert!(node.is_installed(&kit));
    let old = saved_dir(&kit, node, "20.18");
    install_fake(&kit, node, &old);
    write_version(&old, "v20.18.1").unwrap();
    fs::create_dir_all(saved_dir(&kit, node, "18.1")).unwrap(); // no key
    assert_eq!(
        saved(&kit, node),
        [Line {
            line: "20.18".into(),
            version: Some("v20.18.1".into())
        }]
    );
    assert_eq!(dir_for(&kit, node, Some("22.11")), Some(main.clone()));
    assert_eq!(dir_for(&kit, node, Some("18.1")), Some(main.clone()));
    assert_eq!(dir_for(&kit, node, Some("20.18")), Some(old.clone()));

    // The system's PATH: `;` and `\` on Windows, `:` and `/` elsewhere; an
    // entry under the folder, one that only starts with its name, another.
    let mut env = KitEnv::default();
    let (sep, slash) = (env.path_sep(), std::path::MAIN_SEPARATOR);
    let other = if cfg!(windows) { r"C:\W" } else { "/usr/bin" };
    let path = format!(
        "{}{sep}{}{slash}x{sep}{}x{sep}{other}",
        main.display(),
        main.display(),
        main.display()
    );
    env.set("PATH", path);
    let base = env.clone();

    // Apache's PHP follows the PHP in use (its module and php.ini's
    // ext folder must be the same version).
    let mut php_env = KitEnv::default();
    php_env.set("CP_PHP", main.display().to_string());
    apply(
        &mut php_env,
        &[Swap {
            from: main.display().to_string(),
            to: old.display().to_string(),
        }],
    );
    assert_eq!(
        php_env.get("CP_PHP_FWD"),
        Some(old.display().to_string().replace('\\', "/").as_str())
    );

    // The kit uses the newest line on disk: the default, no rewrite.
    assert!(kit_choice(&kit).is_empty());
    apply_kit_choice(&kit, &mut env);
    assert_eq!(env.get("PATH"), base.get("PATH"));

    // A project on 20.18.
    set_project_choice(&kit, "app", "node", Some("20.18")).unwrap();
    let p = project_env(&kit, "app", env.clone());
    assert_eq!(
        crate::test_support::past_stand_ins(&kit, p.get("PATH")),
        format!(
            "{}{sep}{}{slash}x{sep}{}x{sep}{other}",
            old.display(),
            old.display(),
            main.display()
        ),
        "only entries under the folder"
    );
    let file = kit.project_data().join("app").join("versions.json");
    assert!(fs::read_to_string(&file).unwrap().contains("swaps"));
    let txt = kit.project_data().join("app").join("versions.txt");
    assert_eq!(
        fs::read_to_string(&txt).unwrap(),
        format!("{}\t{}\n", main.display(), old.display())
    );
    // Back to the kit's: no rewrite, no file.
    set_project_choice(&kit, "app", "node", None).unwrap();
    assert!(!file.exists() && !txt.exists());

    // A newer line downloaded with "+": the kit uses it, and a new
    // project keeps it even when newer ones come.
    let newer = saved_dir(&kit, node, "24.1");
    install_fake(&kit, node, &newer);
    write_version(&newer, "v24.1.0").unwrap();
    assert_eq!(
        kit_choice(&kit).get("node").map(String::as_str),
        Some("24.1")
    );
    pin_newest(&kit, "fresh");
    assert_eq!(
        project_choice(&kit, "fresh")
            .get("node")
            .map(String::as_str),
        Some("24.1")
    );
    // Locked by the project that uses it; free once it chooses another.
    assert!(remove(&kit, "node", "24.1").is_err());
    set_project_choice(&kit, "fresh", "node", None).unwrap();
    // What was kept for the line goes with it, not what another line has.
    let staged = crate::router::bootstrap::staged_dir(&kit, "node", Some("24.1"));
    fs::create_dir_all(&staged).unwrap();
    fs::write(staged.join("node.tar.xz"), b"node").unwrap();
    let locks = crate::install::conda::root(&kit).join("locks");
    fs::create_dir_all(&locks).unwrap();
    fs::write(locks.join("node@24.1.txt"), b"").unwrap();
    fs::write(locks.join("node@20.18.txt"), b"").unwrap();
    remove(&kit, "node", "24.1").unwrap();
    assert!(kit_choice(&kit).is_empty());
    assert!(!kit.apps().join(".bootstrap").exists());
    assert!(!locks.join("node@24.1.txt").exists());
    assert!(locks.join("node@20.18.txt").is_file());

    assert!(check("claude", Some("1.0")).is_err());
    assert!(check("node", Some(r"..\x")).is_err());
    // The default goes while another stays: the newest left takes over.
    remove(&kit, "node", "22.11").unwrap();
    assert_eq!(default_line(&kit, node).as_deref(), Some("20.18"));
    assert_eq!(node.installed_version(&kit).as_deref(), Some("v20.18.1"));
    assert_eq!(default_dir(&kit, node), old);
    assert!(saved(&kit, node).is_empty() && !main.exists());
    assert!(
        remove(&kit, "node", "20.18").is_err(),
        "the last goes with the component"
    );
}

/// A project on another line of PHP also gets that line's php.ini
/// (PHPRC, and Apache's CP_PHPRC_FWD), not the default line's.
#[test]
fn a_php_line_brings_its_php_ini() {
    let (_tmp, kit) = prepared_kit("versions-php-ini");
    let php = Catalog::get().find("php").unwrap();
    let version = "8.4.13 (Composer 2.8.12)";
    let main = install_dir(&kit, php, version);
    install_fake(&kit, php, &main);
    php.save_version(&kit, version).unwrap();
    let old = saved_dir(&kit, php, "7.4");
    install_fake(&kit, php, &old);
    write_version(&old, "7.4.33").unwrap();

    let data = kit.data();
    // Off Windows each target has its own folder of php.ini (one kit can
    // hold the same line for Windows and Unix).
    let ini_root = if kit.os().is_windows() {
        data.join("php")
    } else {
        data.join("php").join(kit.target().as_str())
    };
    let mut env = KitEnv::default();
    env.set_php(&data, &main);
    assert_eq!(env.get("PHPRC"), ini_root.join("8.4").to_str());

    set_project_choice(&kit, "app", "php", Some("7.4")).unwrap();
    let p = project_env(&kit, "app", env.clone());
    let ini = ini_root.join("7.4");
    assert_eq!(p.get("CP_PHP"), old.to_str());
    assert_eq!(p.get("PHPRC"), ini.to_str());
    assert_eq!(
        p.get("CP_PHPRC_FWD").unwrap(),
        ini.to_str().unwrap().replace('\\', "/")
    );
}

/// Off Windows a project on another Ruby line also gets that line's
/// GEM_HOME, and its bin on PATH, not the default line's.
#[test]
fn a_ruby_line_brings_its_gem_home() {
    let tmp = TempDir::new("versions-gem-home");
    let target = crate::kit::Target::LINUX_X64;
    let kit = Kit::at_target(tmp.0.clone(), target);
    kit.prepare().unwrap();
    let ruby = Catalog::get().find("ruby").unwrap();
    let main = install_dir(&kit, ruby, "3.4.7");
    install_fake(&kit, ruby, &main);
    ruby.save_version(&kit, "3.4.7").unwrap();
    let old = saved_dir(&kit, ruby, "3.3");
    install_fake(&kit, ruby, &old);
    write_version(&old, "3.3.9").unwrap();

    let data = kit.data();
    let (gems, old_gems) = (
        crate::env::gem_home(&data, target, "3.4"),
        crate::env::gem_home(&data, target, "3.3"),
    );
    let mut env = KitEnv::default();
    env.set("GEM_HOME", gems.to_str().unwrap());
    env.set("PATH", gems.join("bin").to_str().unwrap());

    set_project_choice(&kit, "app", "ruby", Some("3.3")).unwrap();
    let p = project_env(&kit, "app", env.clone());
    assert_eq!(p.get("GEM_HOME"), old_gems.to_str());
    assert_eq!(
        crate::test_support::past_stand_ins(&kit, p.get("PATH")),
        old_gems.join("bin").to_str().unwrap()
    );
}
