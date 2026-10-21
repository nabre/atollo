use super::*;

#[test]
fn mongodb_tools_from_release_json() {
    let j: Value = serde_json::from_str(&format!(
        r#"{{"versions":[{{"version":"100.19.1","downloads":[
              {{"name":"ubuntu2204","arch":"x86_64","archive":{{"url":"https://x/u.tgz","sha256":"{u}"}}}},
              {{"name":"windows","arch":"x86_64","archive":{{"url":"https://fastdl.mongodb.org/tools/db/mongodb-database-tools-windows-x86_64-100.19.1.zip","sha256":"{w}"}}}}]}}]}}"#,
        u = "u".repeat(64),
        w = "w".repeat(64)
    ))
    .unwrap();
    let (v, url, sha) = mongodb_tools_zip(&j).unwrap();
    assert_eq!(v, "100.19.1");
    assert!(url.ends_with("windows-x86_64-100.19.1.zip"));
    assert_eq!(sha, "w".repeat(64));
}

/// `/etc/os-release` of real systems (Ubuntu 24.04 is the one of WSL2 on
/// the development PC; the others from the distributions' own files).
const UBUNTU_2404: &str = "PRETTY_NAME=\"Ubuntu 24.04.3 LTS\"\nNAME=\"Ubuntu\"\nVERSION_ID=\"24.04\"\nVERSION_CODENAME=noble\nID=ubuntu\nID_LIKE=debian\nUBUNTU_CODENAME=noble\n";
const DEBIAN_13: &str = "PRETTY_NAME=\"Debian GNU/Linux 13 (trixie)\"\nNAME=\"Debian GNU/Linux\"\nVERSION_ID=\"13\"\nVERSION_CODENAME=trixie\nID=debian\n";
const FEDORA_42: &str =
    "NAME=\"Fedora Linux\"\nVERSION=\"42 (Workstation Edition)\"\nID=fedora\nVERSION_ID=42\n";
const ROCKY_94: &str = "NAME=\"Rocky Linux\"\nVERSION=\"9.4 (Blue Onyx)\"\nID=\"rocky\"\nID_LIKE=\"rhel centos fedora\"\nVERSION_ID=\"9.4\"\n";
const MINT_22: &str = "NAME=\"Linux Mint\"\nVERSION=\"22 (Wilma)\"\nID=linuxmint\nID_LIKE=\"ubuntu debian\"\nVERSION_ID=\"22\"\nUBUNTU_CODENAME=noble\n";
const ARCH: &str = "NAME=\"Arch Linux\"\nID=arch\nBUILD_ID=rolling\n";
const DEBIAN_9: &str = "ID=debian\nVERSION_ID=\"9\"\n";

#[test]
fn linux_distro_from_os_release() {
    assert_eq!(linux_distro(UBUNTU_2404), Some(("ubuntu", vec![24, 4])));
    assert_eq!(linux_distro(DEBIAN_13), Some(("debian", vec![13])));
    assert_eq!(linux_distro(FEDORA_42), Some(("rhel", vec![u64::MAX])));
    assert_eq!(linux_distro(ROCKY_94), Some(("rhel", vec![9, 4])));
    assert_eq!(linux_distro(MINT_22), Some(("ubuntu", vec![24, 4])));
    assert_eq!(linux_distro(ARCH), None);
    assert_eq!(linux_distro(""), None);
}

/// D-112: the Database Tools of this PC's distribution (names of
/// release.json 100.19.1), the oldest Ubuntu build from 20.04 on when it is
/// unknown; only builds of the right CPU are offered by the caller.
#[test]
fn mongodb_tools_build_by_distribution() {
    let x64 = [
        "rhel88",
        "amazon2023",
        "amazon",
        "amazon2",
        "debian10",
        "debian11",
        "debian12",
        "debian13",
        "debian92",
        "rhel10",
        "rhel70",
        "rhel93",
        "suse12",
        "suse15",
        "suse16",
        "ubuntu1604",
        "ubuntu1804",
        "ubuntu2004",
        "ubuntu2204",
        "ubuntu2404",
        "ubuntu2604",
    ];
    let arm64 = [
        "amazon2023",
        "amazon2",
        "rhel10",
        "rhel88",
        "rhel93",
        "ubuntu1604",
        "ubuntu1804",
        "ubuntu2004",
        "ubuntu2204",
        "ubuntu2404",
        "ubuntu2604",
    ];
    let pick = |os: Option<&str>, names: &[&'static str]| mongodb_tools_linux_name(os, names);
    assert_eq!(pick(Some(UBUNTU_2404), &x64), Some("ubuntu2404"));
    assert_eq!(pick(Some(UBUNTU_2404), &arm64), Some("ubuntu2404"));
    assert_eq!(pick(Some(MINT_22), &x64), Some("ubuntu2404"));
    assert_eq!(pick(Some(DEBIAN_13), &x64), Some("debian13"));
    assert_eq!(pick(Some(FEDORA_42), &x64), Some("rhel10"));
    assert_eq!(pick(Some(ROCKY_94), &x64), Some("rhel93"));
    assert_eq!(pick(Some(ROCKY_94), &arm64), Some("rhel93"));
    assert_eq!(
        pick(Some("ID=amzn\nVERSION_ID=\"2023\"\n"), &arm64),
        Some("amazon2023")
    );
    assert_eq!(
        pick(Some("ID=\"sles\"\nVERSION_ID=\"15.6\"\n"), &x64),
        Some("suse15")
    );
    // Newer than every build of its family: the newest one.
    assert_eq!(
        pick(Some("ID=ubuntu\nVERSION_ID=\"28.04\"\n"), &x64),
        Some("ubuntu2604")
    );
    // Debian by major, as RHEL: Debian 9 (`VERSION_ID="9"`) takes
    // debian92, whose glibc it has (the Ubuntu 20.04 build needs a newer
    // one); Debian 10 its own build, not debian92.
    assert_eq!(pick(Some(DEBIAN_9), &x64), Some("debian92"));
    assert_eq!(
        pick(Some("ID=debian\nVERSION_ID=\"10\"\n"), &x64),
        Some("debian10")
    );
    // Debian 8: no build of its family that old.
    assert_eq!(
        pick(Some("ID=debian\nVERSION_ID=\"8\"\n"), &x64),
        Some("ubuntu2004")
    );
    // Debian 12 has no arm64 build; Arch and a kit prepared from another
    // system (no os-release): the oldest Ubuntu from 20.04.
    assert_eq!(pick(Some(DEBIAN_13), &arm64), Some("ubuntu2004"));
    assert_eq!(pick(Some(ARCH), &x64), Some("ubuntu2004"));
    assert_eq!(pick(None, &arm64), Some("ubuntu2004"));
    assert_eq!(pick(None, &["rhel93"]), None);
    // RHEL-like systems by major: CentOS 7 (`VERSION_ID="7"`) takes
    // rhel70, not the Ubuntu fallback; RHEL 8.2 the 8 build.
    assert_eq!(
        pick(Some("ID=\"centos\"\nVERSION_ID=\"7\"\n"), &x64),
        Some("rhel70")
    );
    assert_eq!(
        pick(Some("ID=\"rhel\"\nVERSION_ID=\"8.2\"\n"), &x64),
        Some("rhel88")
    );
    assert_eq!(
        pick(Some("ID=\"rhel\"\nVERSION_ID=\"9.0\"\n"), &arm64),
        Some("rhel93")
    );
}

#[test]
fn mongodb_tools_for_linux_follow_the_distribution() {
    use crate::kit::Target;

    let j: Value = serde_json::from_str(&format!(
        r#"{{"versions":[{{"version":"100.19.1","downloads":[
              {{"name":"ubuntu1604","arch":"x86_64","archive":{{"url":"https://x/ubuntu1604-x86_64.tgz","sha256":"{a}"}}}},
              {{"name":"ubuntu2004","arch":"x86_64","archive":{{"url":"https://x/ubuntu2004-x86_64.tgz","sha256":"{a}"}}}},
              {{"name":"debian12","arch":"x86_64","archive":{{"url":"https://x/debian12-x86_64.tgz","sha256":"{a}"}}}},
              {{"name":"rhel93","arch":"aarch64","archive":{{"url":"https://x/rhel93-aarch64.tgz","sha256":"{a}"}}}},
              {{"name":"ubuntu2004","arch":"arm64","archive":{{"url":"https://x/ubuntu2004-arm64.tgz","sha256":"{a}"}}}},
              {{"name":"ubuntu2404","arch":"arm64","archive":{{"url":"https://x/ubuntu2404-arm64.tgz","sha256":"{a}"}}}}]}}]}}"#,
        a = "a".repeat(64)
    ))
    .unwrap();
    let url = |t: Target, os: Option<&str>| mongodb_tools_for_distro(t, &j, os).unwrap().1;
    assert!(
        url(Target::LINUX_X64, Some("ID=debian\nVERSION_ID=12\n")).ends_with("debian12-x86_64.tgz")
    );
    assert!(url(Target::LINUX_X64, None).ends_with("ubuntu2004-x86_64.tgz"));
    assert!(url(Target::LINUX_ARM64, Some(ROCKY_94)).ends_with("rhel93-aarch64.tgz"));
    assert!(url(Target::LINUX_ARM64, Some(UBUNTU_2404)).ends_with("ubuntu2404-arm64.tgz"));
    // Never another CPU's build.
    assert!(url(Target::LINUX_ARM64, Some("ID=debian\nVERSION_ID=12\n"))
        .ends_with("ubuntu2004-arm64.tgz"));
    assert_eq!(
        mongodb_tools_for_distro(Target::LINUX_X64, &j, None)
            .unwrap()
            .3,
        Kind::TarGz
    );
}
