use super::*;
use crate::setup::catalog::Catalog;

#[test]
fn processes_of_the_old_editor() {
    let proc = |pid, path: &str| crate::os::processes::Proc {
        pid,
        parent: 0,
        name: String::new(),
        path: path.into(),
    };
    let all = [
        proc(1, r"E:\Kit\data\apps\win32-x64\vscodium-server\node.exe"),
        proc(2, r"E:\Kit\data\apps\win32-x64\VSCODIUM-SERVER\bin\x.exe"),
        proc(
            3,
            r"E:\Kit\data\apps\win32-x64\vscodium-server-old\node.exe",
        ),
        proc(4, r"E:\Kit\data\apps\win32-x64\node\node.exe"),
    ];
    let old = Path::new(r"E:\Kit\data\apps\win32-x64\vscodium-server");
    assert_eq!(crate::os::processes::running_from(&all, old), [1, 2]);
}

#[test]
fn the_editor_is_pinned() {
    let server = Catalog::get().find("vscodium-server").unwrap();
    let r = pinned(server).unwrap();
    let version = server.version.as_deref().unwrap();
    assert_eq!(r.version, version);
    assert_eq!(r.parts.len(), 1);
    assert_eq!(
        r.parts[0].url,
        format!("https://github.com/VSCodium/vscodium/releases/download/{version}/vscodium-reh-web-win32-x64-{version}.tar.gz")
    );
    let Expect::Sha256(sha) = &r.parts[0].expect else {
        panic!("sha256")
    };
    assert!(sha.len() == 64 && sha.chars().all(|c| c.is_ascii_hexdigit()));
}
