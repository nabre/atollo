use super::*;
use crate::kit::tests::TempDir;

#[test]
fn gradle_finds_the_kit_jdks_only() {
    let jdks = [
        PathBuf::from(r"D:\kit\java\25.0"),
        PathBuf::from(r"D:\kit\java\11.0"),
    ];
    let text = "org.gradle.jvmargs=-Xmx2g\norg.gradle.java.installations.paths=C:/old\n";
    let out = gradle_properties(text, &jdks);
    assert!(
        out.contains("org.gradle.jvmargs=-Xmx2g"),
        "the user's lines stay"
    );
    assert!(out.contains("org.gradle.java.installations.paths=D:/kit/java/25.0,D:/kit/java/11.0"));
    assert!(out.contains("auto-detect=false"));
    assert!(!out.contains("C:/old"));
    assert_eq!(gradle_properties(&out, &jdks), out, "stable");
    assert!(!gradle_properties(&out, &[]).contains("installations"));
    let xml = maven_toolchains(&[("11.0".into(), PathBuf::from(r"D:\kit\java\11.0"))]);
    assert!(xml.contains("<version>11</version>") && xml.contains(KIT_MARK));
}

#[test]
fn dotnet_roots_share_their_versions() {
    let tmp = TempDir::new("dotnet-sbs");
    let ten = tmp.0.join("10.0");
    let eight = tmp.0.join("8.0");
    for (root, v) in [(&ten, "10.0.3"), (&eight, "8.0.21")] {
        let rt = join_rel(root, "shared/Microsoft.NETCore.App").join(v);
        fs::create_dir_all(&rt).unwrap();
        fs::write(rt.join("coreclr.dll"), b"x").unwrap();
    }
    fs::create_dir_all(join_rel(&eight, "sdk/8.0.415")).unwrap();
    link_dotnet(&[ten.clone(), eight.clone()]).unwrap();
    assert!(join_rel(&ten, "shared/Microsoft.NETCore.App/8.0.21/coreclr.dll").is_file());
    assert!(join_rel(&ten, "sdk/8.0.415").is_dir());
    assert!(join_rel(&eight, "shared/Microsoft.NETCore.App/10.0.3/coreclr.dll").is_file());
    // Again: nothing new, no error.
    link_dotnet(&[ten.clone(), eight.clone()]).unwrap();

    // Removing a version's folder leaves the others' files alone
    // (junctions are not followed), then its junctions go.
    crate::net::archive::remove_dir(&ten).unwrap();
    assert!(join_rel(&eight, "shared/Microsoft.NETCore.App/8.0.21/coreclr.dll").is_file());
    link_dotnet(std::slice::from_ref(&eight)).unwrap();
    assert!(!join_rel(&eight, "shared/Microsoft.NETCore.App/10.0.3").exists());
    assert!(join_rel(&eight, "sdk/8.0.415").is_dir());
}

/// On a disk without links (exFAT, FAT) the version is copied, marked, and
/// the copy goes when its source goes (the refusal is injected).
#[test]
fn dotnet_copies_where_links_are_refused() {
    let tmp = TempDir::new("dotnet-copy");
    let source = join_rel(&tmp.0, "8.0/sdk/8.0.415");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("dotnet.dll"), b"x").unwrap();
    let link = join_rel(&tmp.0, "10.0/sdk/8.0.415");
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    let refuse = |_: &Path, _: &Path| {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "no links here",
        ))
    };
    link_or_copy(&source, &link, refuse).unwrap();
    assert_eq!(fs::read(link.join("dotnet.dll")).unwrap(), b"x");
    assert!(!is_dir_link(&link), "a copy");
    assert!(!stale_copy(&link), "its source is there");

    // Not shared further: another root gets only 8.0's own version.
    let other = tmp.0.join("9.0");
    fs::create_dir_all(join_rel(&other, "sdk")).unwrap();
    let ten = tmp.0.join("10.0");
    fs::create_dir_all(join_rel(&ten, "sdk/10.0.100")).unwrap();
    link_dotnet(&[ten.clone(), other.clone()]).unwrap();
    assert!(join_rel(&other, "sdk/10.0.100").is_dir());
    assert!(
        !join_rel(&other, "sdk/8.0.415").exists(),
        "copies are not shared"
    );
    assert!(link.is_dir(), "a copy whose source is there stays");

    crate::net::archive::remove_dir(&tmp.0.join("8.0")).unwrap();
    assert!(stale_copy(&link));
    link_dotnet(&[ten.clone(), other.clone()]).unwrap();
    assert!(!link.exists(), "the copy of a version gone is removed");
    assert!(join_rel(&ten, "sdk/10.0.100").is_dir());

    // Any other error is one.
    let denied = |_: &Path, _: &Path| {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "denied",
        ))
    };
    let source = join_rel(&ten, "sdk/10.0.100");
    assert!(link_or_copy(&source, &join_rel(&other, "sdk/10.0.100"), denied).is_err());
}

/// A copy that fails partway (a full disk) leaves nothing: the next refresh
/// makes it again instead of keeping a broken SDK as a version of its own.
#[test]
fn a_failed_copy_leaves_no_folder() {
    let tmp = TempDir::new("dotnet-copy-fail");
    let source = join_rel(&tmp.0, "8.0/sdk/8.0.415");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("dotnet.dll"), b"x").unwrap();
    let link = join_rel(&tmp.0, "10.0/sdk/8.0.415");
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    let half = |_: &Path, to: &Path| {
        fs::create_dir_all(to)?;
        fs::write(to.join("half.dll"), b"x")?;
        Err(std::io::Error::other("disk full"))
    };
    assert!(copy_marked(&source, &link, half).is_err());
    assert!(!link.exists(), "no half copy under the version's name");
    assert!(
        !staged_path(&link).exists(),
        "the staging folder is removed"
    );

    // A copy interrupted by the app closing: its staging folder is removed
    // and the version linked, never shared as a version of its own.
    let staged = staged_path(&link);
    fs::create_dir_all(&staged).unwrap();
    fs::write(staged.join("half.dll"), b"x").unwrap();
    let ten = tmp.0.join("10.0");
    let eight = tmp.0.join("8.0");
    let nine = tmp.0.join("9.0");
    fs::create_dir_all(join_rel(&nine, "sdk")).unwrap();
    link_dotnet(&[ten.clone(), eight.clone(), nine.clone()]).unwrap();
    assert!(!staged.exists());
    assert!(join_rel(&ten, "sdk/8.0.415/dotnet.dll").is_file());
    assert!(!join_rel(&nine, "sdk/8.0.415.devkit-new").exists());
    assert!(join_rel(&nine, "sdk/8.0.415/dotnet.dll").is_file());
}

/// The mark names the source relative to the copy: a kit mounted elsewhere
/// keeps its copies; an absolute mark (older copies) still reads.
#[test]
fn copy_mark_is_relative_to_the_copy() {
    let tmp = TempDir::new("dotnet-mark");
    let source = join_rel(&tmp.0, "a/8.0/shared/Microsoft.NETCore.App/8.0.21");
    let link = join_rel(&tmp.0, "a/10.0/shared/Microsoft.NETCore.App/8.0.21");
    assert_eq!(
        copy_mark(&source, &link),
        "../../../../8.0/shared/Microsoft.NETCore.App/8.0.21"
    );
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    copy_marked(&source, &link, crate::net::archive::copy_tree).unwrap();
    assert!(link.join(COPY_MARK).is_file() && !stale_copy(&link));

    // The disk mounted at another place: the copy is not stale.
    fs::rename(tmp.0.join("a"), tmp.0.join("b")).unwrap();
    let moved = join_rel(&tmp.0, "b/10.0/shared/Microsoft.NETCore.App/8.0.21");
    assert!(!stale_copy(&moved));
    crate::net::archive::remove_dir(&join_rel(&tmp.0, "b/8.0")).unwrap();
    assert!(stale_copy(&moved));

    // An absolute mark is read as it is.
    let kept = join_rel(&tmp.0, "b/10.0");
    fs::write(moved.join(COPY_MARK), kept.display().to_string()).unwrap();
    assert!(!stale_copy(&moved));
}
