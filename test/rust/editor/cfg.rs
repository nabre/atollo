use super::*;
use crate::kit::{tests::TempDir, Arch};

fn kit_with(name: &str, installed: &[&str]) -> (TempDir, Kit) {
    let tmp = TempDir::new(name);
    let kit = Kit::at(tmp.0.join("k (1) & co"), Arch::X64);
    kit.prepare().unwrap();
    for rel in installed {
        let f = kit.apps().join(rel);
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(f, b"").unwrap();
    }
    (tmp, kit)
}

fn read(kit: &Kit) -> Map<String, Value> {
    serde_json::from_str(&fs::read_to_string(settings_path(kit)).unwrap()).unwrap()
}

#[test]
fn profiles_rewritten_user_ones_kept() {
    let (_t, kit) = kit_with("cfg-profiles", &[r"pwsh\pwsh.exe", r"git\bin\bash.exe"]);
    let path = settings_path(&kit);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r#"{"terminal.integrated.profiles.windows":{"Mine":{"path":"x.exe"},"Dev-Kit":{"path":"old"},"Codex":{"path":"gone"}},"editor.fontSize":15,"files.dialog.defaultPath":"D:\\mine"}"#,
    )
    .unwrap();
    assert_eq!(
        write(&kit, Path::new(r"C:\Windows")).unwrap(),
        Outcome::Written
    );
    let s = read(&kit);
    let p = s["terminal.integrated.profiles.windows"]
        .as_object()
        .unwrap();
    assert_eq!(p["Mine"]["path"], "x.exe");
    assert!(p["Dev-Kit"]["path"]
        .as_str()
        .unwrap()
        .ends_with(r"pwsh\pwsh.exe"));
    assert!(p["Dev-Kit - Git Bash"]["args"][1]
        .as_str()
        .unwrap()
        .ends_with(r"data\kit\bashrc"));
    assert!(
        p.get("Codex").is_none(),
        "agent not installed: profile removed"
    );
    assert!(p["PowerShell"].is_null() && p["Command Prompt"].is_null());
    assert_eq!(s["terminal.integrated.defaultProfile.windows"], "Dev-Kit");
    assert!(s["terminal.integrated.env.windows"]["NODE_OPTIONS"]
        .as_str()
        .unwrap()
        .ends_with("/data/kit/loopback.js\""));
    assert_eq!(s["editor.fontSize"], 15);
    assert_eq!(s["files.dialog.defaultPath"], r"D:\mine", "user value kept");
    assert!(s["git.defaultCloneDirectory"]
        .as_str()
        .unwrap()
        .ends_with("projects"));
    assert_eq!(
        write(&kit, Path::new(r"C:\Windows")).unwrap(),
        Outcome::Unchanged
    );
}

#[test]
fn user_default_profile_kept_when_valid() {
    let (_t, kit) = kit_with("cfg-default", &[r"pwsh\pwsh.exe", r"git\bin\bash.exe"]);
    let path = settings_path(&kit);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r#"{"terminal.integrated.defaultProfile.windows":"Dev-Kit - Git Bash"}"#,
    )
    .unwrap();
    write(&kit, Path::new(r"C:\Windows")).unwrap();
    assert_eq!(
        read(&kit)["terminal.integrated.defaultProfile.windows"],
        "Dev-Kit - Git Bash"
    );
    fs::write(
        &path,
        r#"{"terminal.integrated.defaultProfile.windows":"PowerShell"}"#,
    )
    .unwrap();
    write(&kit, Path::new(r"C:\Windows")).unwrap();
    assert_eq!(
        read(&kit)["terminal.integrated.defaultProfile.windows"],
        "Dev-Kit",
        "hidden PC profile replaced"
    );
}

#[test]
fn file_with_comments_left_alone() {
    let (_t, kit) = kit_with("cfg-comments", &[]);
    let path = settings_path(&kit);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let text = "{\n  // mine\n  \"a\": 1\n}";
    fs::write(&path, text).unwrap();
    assert!(matches!(
        write(&kit, Path::new(r"C:\Windows")).unwrap(),
        Outcome::LeftAlone(_)
    ));
    assert_eq!(fs::read_to_string(&path).unwrap(), text);
}

#[test]
fn codium_themes_become_atollo_user_choice_kept() {
    let (_t, kit) = kit_with("cfg-colors", &[]);
    let path = settings_path(&kit);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r##"{"workbench.colorCustomizations":{"editor.background":"#000000","[Default Dark Modern]":{"focusBorder":"#10B981"},"[Default Light Modern]":{}},"workbench.preferredDarkColorTheme":"Dark Modern","workbench.preferredLightColorTheme":"Solarized Light"}"##,
    )
    .unwrap();
    write(&kit, Path::new(r"C:\Windows")).unwrap();
    let s = read(&kit);
    let colors = s["workbench.colorCustomizations"].as_object().unwrap();
    assert_eq!(colors.len(), 1, "the user's own kept: {colors:?}");
    assert_eq!(colors["editor.background"], "#000000");
    assert_eq!(s["workbench.colorTheme"], "Atollo Dark");
    assert_eq!(s["workbench.experimental.modernUI"], true);
    assert_eq!(s["workbench.preferredDarkColorTheme"], "Atollo Dark");
    assert_eq!(s["workbench.preferredLightColorTheme"], "Solarized Light");

    fs::write(&path, r#"{"workbench.preferredLightColorTheme":"Default Light Modern","workbench.colorCustomizations":{"[Default Dark Modern]":{}}}"#).unwrap();
    write(&kit, Path::new(r"C:\Windows")).unwrap();
    let s = read(&kit);
    assert_eq!(s["workbench.preferredLightColorTheme"], "Atollo Light");
    assert!(s.get("workbench.colorCustomizations").is_none());
}
