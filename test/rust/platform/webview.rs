use super::*;
use crate::kit::{tests::TempDir, Target};

#[test]
fn webkit_package_by_distribution() {
    let ubuntu = "NAME=\"Ubuntu\"\nID=ubuntu\nID_LIKE=debian\nVERSION_ID=\"24.04\"\n";
    assert_eq!(
        webkit_install_command(ubuntu),
        Some("sudo apt install libwebkit2gtk-4.1-0")
    );
    let mint = "ID=linuxmint\nID_LIKE=\"ubuntu debian\"\n";
    assert_eq!(
        webkit_install_command(mint),
        Some("sudo apt install libwebkit2gtk-4.1-0")
    );
    assert_eq!(
        webkit_install_command("ID=fedora\n"),
        Some("sudo dnf install webkit2gtk4.1")
    );
    assert_eq!(
        webkit_install_command("ID=\"rocky\"\nID_LIKE=\"rhel centos fedora\"\n"),
        Some("sudo dnf install webkit2gtk4.1")
    );
    assert_eq!(
        webkit_install_command("ID=manjaro\nID_LIKE=arch\n"),
        Some("sudo pacman -S webkit2gtk-4.1")
    );
    assert_eq!(
        webkit_install_command("ID=\"opensuse-tumbleweed\"\nID_LIKE=\"opensuse suse\"\n"),
        Some("sudo zypper install libwebkit2gtk-4_1-0")
    );
    assert_eq!(webkit_install_command("ID=nixos\n"), None);
}

#[test]
fn missing_web_view_message() {
    let tmp = TempDir::new("webview-msg");
    let linux = Kit::at_target(tmp.0.join("L"), Target::LINUX_X64);
    let text = missing_message(&linux, Some("ID=ubuntu\n"), "it");
    assert!(
        text.contains("WebKitGTK 4.1") && text.contains("sudo apt install"),
        "{text}"
    );
    let unknown = missing_message(&linux, None, "it");
    assert!(unknown.contains(WEBKITGTK), "{unknown}");
    assert_ne!(missing_message(&linux, None, "en"), unknown);
    // Windows: the WebView2 message of before.
    let win = Kit::at_target(tmp.0.join("W"), Target::WIN32_X64);
    let text = missing_message(&win, None, "it");
    assert!(text.starts_with("Manca Microsoft Edge WebView2"), "{text}");
    assert!(
        text.contains(DOWNLOAD_URL) && text.contains("x64"),
        "{text}"
    );
}
