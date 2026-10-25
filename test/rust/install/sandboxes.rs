use super::*;

/// One asset per target Docker builds for, none elsewhere.
#[test]
fn every_supported_target_has_its_asset() {
    let t = |os, arch| asset_for(Target::new(os, arch)).map(|(name, kind, _)| (name, kind));
    assert_eq!(
        t(Os::Win32, Arch::X64),
        Some(("DockerSandboxes.msi", Kind::Msi))
    );
    assert_eq!(
        t(Os::Darwin, Arch::Arm64),
        Some(("DockerSandboxes-darwin.tar.gz", Kind::TarGz))
    );
    assert_eq!(
        t(Os::Linux, Arch::X64),
        Some(("DockerSandboxes-linux-amd64.tar.gz", Kind::TarGz))
    );
    assert_eq!(
        t(Os::Linux, Arch::Arm64),
        Some(("DockerSandboxes-linux-arm64.tar.gz", Kind::TarGz))
    );
    assert_eq!(t(Os::Win32, Arch::Arm64), None);
    assert_eq!(t(Os::Darwin, Arch::X64), None);
}

/// The Linux tarball laid out as its `install.sh` does.
#[test]
fn the_linux_tarball_is_laid_out_as_install_sh() {
    let tmp = crate::test_support::TempDir::new("sandboxes-layout");
    let tree = tmp.0.as_path();
    for name in [
        "sbx",
        "libsailor.so",
        "containerd-shim-nerdbox-v1",
        "containerd-shim-nerdbox-gpu-v1",
        "mkfs.erofs",
        "mkfs.ext4",
        "nerdbox-kernel-x86_64",
        "nerdbox-rootfs-x86_64.erofs",
        "install.sh",
        "LICENSE",
    ] {
        fs::write(tree.join(name), name).unwrap();
    }
    lay_out_linux(tree).unwrap();
    for rel in [
        "bin/sbx",
        "libexec/lib/libsailor.so",
        "libexec/containerd-shim-nerdbox-v1",
        "libexec/containerd-shim-nerdbox-gpu-v1",
        "libexec/mkfs.erofs",
        "libexec/mkfs.ext4",
        "libexec/nerdbox-kernel-x86_64",
        "libexec/nerdbox-rootfs-x86_64.erofs",
        "install.sh",
        "LICENSE",
    ] {
        assert!(crate::common::fsx::join_rel(tree, rel).is_file(), "{rel}");
    }
    assert!(!tree.join("sbx").exists());
}
