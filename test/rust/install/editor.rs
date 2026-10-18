use super::*;
use crate::setup::catalog::Catalog;

#[test]
fn processes_of_the_old_editor() {
    // Paths of the system's form: the comparison uses its separator.
    let apps = if cfg!(windows) {
        Path::new(r"E:\Kit\data\apps\win32-x64").to_path_buf()
    } else {
        Path::new("/media/u/Kit/data/apps/linux-x64").to_path_buf()
    };
    let proc = |pid, rel: &[&str]| crate::os::processes::Proc {
        pid,
        parent: 0,
        name: String::new(),
        path: rel.iter().fold(apps.clone(), |p, s| p.join(s)),
    };
    let all = [
        proc(1, &["vscodium-server", "node.exe"]),
        proc(2, &["VSCODIUM-SERVER", "bin", "x.exe"]),
        proc(3, &["vscodium-server-old", "node.exe"]),
        proc(4, &["node", "node.exe"]),
    ];
    let old = apps.join("vscodium-server");
    // Windows and macOS (APFS by default) do not tell case apart, so
    // VSCODIUM-SERVER is the same folder; on Linux it is another one and
    // its processes must not be stopped by the update.
    let expected: &[u32] = if cfg!(target_os = "linux") {
        &[1]
    } else {
        &[1, 2]
    };
    assert_eq!(crate::os::processes::running_from(&all, &old), expected);
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

/// BUG-V3-09: every system the editor is downloaded for has its SHA-256
/// fixed in the catalog (no `.sha256` fetched next to the archive); on
/// Windows ARM64 the x64 build, emulated.
#[test]
fn the_editor_is_pinned_on_every_target() {
    use crate::kit::Target;

    let server = Catalog::get().find("vscodium-server").unwrap();
    for target in Target::ALL {
        let download = if server.is_emulated_on(target) {
            target.with_arch(crate::kit::Arch::X64)
        } else {
            target
        };
        let sha = server
            .sha256_for(download)
            .unwrap_or_else(|| panic!("no SHA-256 for {download}"));
        assert!(
            sha.len() == 64 && sha.chars().all(|c| c.is_ascii_hexdigit()),
            "{download}"
        );
        let r = pinned_with_sha(download, server, sha).unwrap();
        assert!(r.parts[0].file_name.contains(download.as_str()), "{target}");
    }
    assert_eq!(
        server.sha256_for(Target::WIN32_X64),
        server.sha256.as_deref()
    );
}

/// The version the catalog's `sha256ByTarget` of `vscodium-server` was
/// recorded for, with those hashes. A new editor version must come with a
/// new hash for every target: this record is renewed with them, after the
/// hashes are checked against the release's `.sha256` files.
const HASHED_VERSION: &str = "1.135.06055";
const HASHED: [(&str, &str); 5] = [
    (
        "win32-x64",
        "579121d3043b94fbd054aee3f1155b53b00dc2786995604a685fda9ba7e2b560",
    ),
    (
        "darwin-arm64",
        "87ffebc1df7bde31b9e863d38fa6121cdebe3e5f9794d65f95f062f986dcfa73",
    ),
    (
        "darwin-x64",
        "b0038f4ae482d6454bb293ae5b04d4af0834f8c7094f95dfeec3c6519d9fcb74",
    ),
    (
        "linux-x64",
        "e36cf545d526e7f2e10679e9b35a269e9ba573f9527038bf39bc26aab64d27a6",
    ),
    (
        "linux-arm64",
        "883596fdef9e8cb9aa27965b5a75ace1c7cd6de026dd7c8d96bdacd126fd4cec",
    ),
];

/// A new `vscodium-server` version without a new SHA-256 for every target
/// would download the new archive and refuse it (or, worse, a hash left
/// from the old version could be taken for the new one): the test fails
/// until every target's hash is renewed and the record above follows.
#[test]
fn a_new_editor_version_renews_every_hash() {
    let server = Catalog::get().find("vscodium-server").unwrap();
    let version = server.version.as_deref().unwrap();
    let hashes = &server.sha256_by_target;
    let recorded: std::collections::BTreeMap<String, String> = HASHED
        .iter()
        .map(|(t, s)| ((*t).to_owned(), (*s).to_owned()))
        .collect();
    if version == HASHED_VERSION {
        assert_eq!(
            hashes, &recorded,
            "vscodium-server {version}: sha256ByTarget changed without a new version"
        );
        return;
    }
    let stale: Vec<&str> = HASHED
        .iter()
        .filter(|(t, old)| hashes.get(*t).is_none_or(|s| s == old))
        .map(|(t, _)| *t)
        .collect();
    assert!(
        stale.is_empty(),
        "vscodium-server {HASHED_VERSION} -> {version}: sha256ByTarget not renewed for {stale:?}"
    );
    panic!(
        "vscodium-server {HASHED_VERSION} -> {version}: once the new sha256ByTarget are \
         checked against the release's .sha256 files, update HASHED_VERSION and HASHED"
    );
}
