use super::*;
use crate::setup::{Catalog, Group};

#[test]
fn the_package_is_fixed_by_the_catalog() {
    let c = Catalog::get().find("conpty").unwrap();
    assert_eq!(c.group, Group::App);
    assert!(c.def);
    assert_eq!(c.targets, ["win32-x64", "win32-arm64"]);
    assert!(c.is_available_on(Target::WIN32_ARM64));
    assert!(!c.is_available_on(Target::LINUX_X64));
    assert!(!c.is_available_on(Target::DARWIN_ARM64));
    let version = c.version.as_deref().unwrap();
    let sha = c.sha256_for(Target::WIN32_ARM64).unwrap();
    assert_eq!(sha.len(), 64);
    assert_eq!(
        c.sha256_for(Target::WIN32_X64),
        Some(sha),
        "one package for both"
    );
    let (url, name) = package(version);
    assert_eq!(
        url,
        format!("https://api.nuget.org/v3-flatcontainer/microsoft.windows.console.conpty/{version}/microsoft.windows.console.conpty.{version}.nupkg")
    );
    assert!(super::super::is_plain_file_name(&name));
    // The terminal looks for both files in its folder.
    assert_eq!(c.key, "conpty/conpty.dll");
    assert_eq!(c.keys, ["conpty/OpenConsole.exe"]);
}

#[test]
fn each_architecture_takes_its_own_files() {
    let [dll, host] = files_for(Target::WIN32_X64);
    assert_eq!(
        dll,
        (
            "runtimes/win-x64/native/conpty.dll".to_owned(),
            "conpty.dll"
        )
    );
    assert_eq!(
        host,
        (
            "build/native/runtimes/x64/OpenConsole.exe".to_owned(),
            "OpenConsole.exe"
        )
    );
    let [dll, host] = files_for(Target::WIN32_ARM64);
    assert_eq!(dll.0, "runtimes/win-arm64/native/conpty.dll");
    assert_eq!(host.0, "build/native/runtimes/arm64/OpenConsole.exe");
    assert!(ConPty.custom(Target::WIN32_ARM64));
    assert!(!ConPty.custom(Target::LINUX_X64));
}
