use super::*;

/// The tools sandbox is needed only to prepare the VM's Linux from a PC
/// that cannot run it: Windows and macOS for their VM's Linux, never a
/// Linux PC for its own system, never another system than the VM's.
#[test]
fn the_tools_sandbox_prepares_only_the_vms_linux_from_another_system() {
    assert!(needed_for(Target::WIN32_X64, Target::LINUX_X64));
    assert!(needed_for(Target::DARWIN_ARM64, Target::LINUX_ARM64));
    assert!(!needed_for(Target::WIN32_X64, Target::LINUX_ARM64));
    assert!(!needed_for(Target::WIN32_X64, Target::WIN32_X64));
    assert!(!needed_for(Target::WIN32_X64, Target::DARWIN_ARM64));
    // The same answers on every runner: a Linux one does not change them.
    assert!(!needed_for(Target::LINUX_X64, Target::LINUX_X64));
    assert!(!needed_for(Target::LINUX_ARM64, Target::LINUX_ARM64));
}

/// It mounts the kit's Linux tools and the global packages of npm, the
/// workspace first: the very folders the projects' VMs mount read-only.
#[test]
fn the_tools_sandbox_writes_the_folders_the_vms_read() {
    let kit = Kit::at_target(PathBuf::from("/kit"), Target::LINUX_X64);
    let roots = mounts(&kit);
    assert_eq!(roots[0], kit.apps());
    assert_eq!(roots[1], kit.data().join("npm-global"));
    assert_eq!(roots.len(), 2);
}

/// A path in a mounted folder becomes the VM's (the drive in lower case,
/// `/`), the folder itself too; a path elsewhere, or a folder that only
/// starts with the same letters, is not one.
#[test]
fn a_path_of_a_mounted_folder_is_the_vms() {
    let roots = [
        PathBuf::from(r"D:\Kit\data\apps\linux-x64"),
        PathBuf::from(r"D:\Kit\data\npm-global"),
    ];
    let word = |w: &str| guest_word(true, &roots, w);
    assert_eq!(
        word(r"D:\Kit\data\apps\linux-x64\rust\cargo\bin\rustup").as_deref(),
        Some("/d/Kit/data/apps/linux-x64/rust/cargo/bin/rustup")
    );
    assert_eq!(
        word(r"d:\kit\DATA\apps\linux-x64").as_deref(),
        Some("/d/kit/DATA/apps/linux-x64"),
        "Windows paths ignore case"
    );
    assert_eq!(
        word("D:/Kit/data/npm-global/bin").as_deref(),
        Some("/d/Kit/data/npm-global/bin")
    );
    assert_eq!(word(r"D:\Kit\data\apps\linux-x64-old\x"), None);
    assert_eq!(word(r"C:\Users\me\AppData\Local\Temp"), None);
    assert_eq!(word("yes"), None);
    assert_eq!(word("--profile"), None);
    // macOS and Linux: the same path in the VM.
    let unix = [PathBuf::from("/Users/me/Kit/data/apps/linux-arm64")];
    assert_eq!(
        guest_word(false, &unix, "/Users/me/Kit/data/apps/linux-arm64/go").as_deref(),
        Some("/Users/me/Kit/data/apps/linux-arm64/go")
    );
    assert_eq!(guest_word(false, &unix, "/usr/bin/tar"), None);
}
