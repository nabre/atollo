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
        let rt = root.join(r"shared\Microsoft.NETCore.App").join(v);
        fs::create_dir_all(&rt).unwrap();
        fs::write(rt.join("coreclr.dll"), b"x").unwrap();
    }
    fs::create_dir_all(eight.join(r"sdk\8.0.415")).unwrap();
    link_dotnet(&[ten.clone(), eight.clone()]).unwrap();
    assert!(ten
        .join(r"shared\Microsoft.NETCore.App\8.0.21\coreclr.dll")
        .is_file());
    assert!(ten.join(r"sdk\8.0.415").is_dir());
    assert!(eight
        .join(r"shared\Microsoft.NETCore.App\10.0.3\coreclr.dll")
        .is_file());
    // Again: nothing new, no error.
    link_dotnet(&[ten.clone(), eight.clone()]).unwrap();

    // Removing a version's folder leaves the others' files alone
    // (junctions are not followed), then its junctions go.
    crate::net::archive::remove_dir(&ten).unwrap();
    assert!(eight
        .join(r"shared\Microsoft.NETCore.App\8.0.21\coreclr.dll")
        .is_file());
    link_dotnet(std::slice::from_ref(&eight)).unwrap();
    assert!(!eight.join(r"shared\Microsoft.NETCore.App\10.0.3").exists());
    assert!(eight.join(r"sdk\8.0.415").is_dir());
}
