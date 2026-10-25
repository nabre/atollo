use super::*;

const NOBLE: &str = "PRETTY_NAME=\"Ubuntu 24.04.1 LTS\"\nNAME=\"Ubuntu\"\nVERSION_ID=\"24.04\"\nID=ubuntu\nID_LIKE=debian\n";

#[test]
fn ubuntu_24_04_needs_kvm_only() {
    assert!(gaps_of(Some(NOBLE), Kvm::Ready).is_empty());
    assert_eq!(gaps_of(Some(NOBLE), Kvm::Missing), vec![Gap::KvmMissing]);
    assert_eq!(gaps_of(Some(NOBLE), Kvm::Denied), vec![Gap::KvmDenied]);
}

#[test]
fn older_ubuntu_and_derivatives_are_not_supported() {
    let jammy = "PRETTY_NAME=\"Ubuntu 22.04.4 LTS\"\nVERSION_ID=\"22.04\"\nID=ubuntu\n";
    assert_eq!(
        gaps_of(Some(jammy), Kvm::Ready),
        vec![Gap::Distribution("Ubuntu 22.04.4 LTS".into())]
    );
    let mint = "PRETTY_NAME=\"Linux Mint 22\"\nVERSION_ID=\"22\"\nID=linuxmint\nID_LIKE=\"ubuntu debian\"\n";
    assert_eq!(
        gaps_of(Some(mint), Kvm::Ready),
        vec![Gap::Distribution("Linux Mint 22".into())]
    );
    let plucky = "VERSION_ID=\"25.04\"\nID=ubuntu\n";
    assert!(gaps_of(Some(plucky), Kvm::Ready).is_empty());
}

/// The kit's shim has a profile when one of AppArmor's files names its
/// path.
#[test]
fn a_profile_names_the_shim() {
    let tmp = crate::test_support::TempDir::new("virt-apparmor");
    let dir = tmp.0.as_path();
    let shim =
        Path::new("/media/u/KIT/data/apps/linux-x64/sandboxes/libexec/containerd-shim-nerdbox-v1");
    assert!(!profiled(dir, shim));
    std::fs::write(dir.join("other"), "/usr/bin/x {}\n").unwrap();
    assert!(!profiled(dir, shim));
    std::fs::write(
        dir.join("atollo-sbx-shim"),
        format!(
            "abi <abi/3.0>,\n\"{}\" flags=(unconfined) {{\n}}\n",
            shim.display()
        ),
    )
    .unwrap();
    assert!(profiled(dir, shim));
}
