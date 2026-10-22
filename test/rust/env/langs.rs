use super::*;
use crate::env::tests::*;
use crate::env::*;

#[test]
fn languages_from_the_folder_of_their_version() {
    let windows = cfg!(windows);
    let files: &[&str] = if windows {
        &[
            "node/22.11/node.exe",
            "php/8.4/php.exe",
            "java/21.0/bin/java.exe",
        ]
    } else {
        &[
            "node/22.11/bin/node",
            "php/8.4/bin/php",
            "java/21.0/bin/java",
        ]
    };
    let (_tmp, kit) = fake_kit("env-versions", files);
    let catalog = crate::setup::Catalog::get();
    for (id, version) in [
        ("node", "v22.11.0"),
        ("php", "8.4.13"),
        ("java", "21.0.5+11"),
    ] {
        catalog
            .find(id)
            .unwrap()
            .save_version(&kit, version)
            .unwrap();
    }
    let env = KitEnv::build(&kit, &Settings::default(), "it", &host());
    let apps = kit.apps();
    let entries = env.path_entries();
    let node = apps.join("node").join("22.11");
    let node_bin = if windows { node } else { node.join("bin") };
    assert!(entries.iter().any(|e| Path::new(e) == node_bin));
    let java = apps.join("java").join("21.0");
    assert!(entries.iter().any(|e| Path::new(e) == java.join("bin")));
    let php = apps.join("php").join("8.4");
    assert_eq!(env.get("CP_PHP"), php.to_str());
    assert_eq!(env.get("JAVA_HOME"), java.to_str());
    let fwd = php.display().to_string().replace('\\', "/");
    assert_eq!(env.get("CP_PHP_FWD"), Some(fwd.as_str()));
}
