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

/// A Windows kit on any host, with the kit's PowerShell and Git Bash.
fn windows_kit(name: &str) -> (TempDir, Kit) {
    kit_on(
        name,
        crate::kit::Target::WIN32_X64,
        &["pwsh/pwsh.exe", "git/bin/bash.exe"],
    )
}

#[test]
fn profiles_rewritten_user_ones_kept() {
    let (_t, kit) = windows_kit("cfg-profiles");
    let path = settings_path(&kit);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r#"{"terminal.integrated.profiles.windows":{"Mine":{"path":"x.exe"},"Atollo":{"path":"old"},"Codex":{"path":"gone"}},"editor.fontSize":15,"files.dialog.defaultPath":"D:\\mine"}"#,
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
    assert!(slashed(&p["Atollo"]["path"]).ends_with("win32-x64/pwsh/pwsh.exe"));
    assert!(slashed(&p["Atollo - Git Bash"]["path"]).ends_with("git/bin/bash.exe"));
    assert!(slashed(&p["Atollo - Git Bash"]["args"][1]).ends_with("data/kit/bashrc"));
    assert!(
        p.get("Codex").is_none(),
        "agent not installed: profile removed"
    );
    assert!(p["PowerShell"].is_null() && p["Command Prompt"].is_null());
    assert_eq!(s["terminal.integrated.defaultProfile.windows"], "Atollo");
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
    let (_t, kit) = windows_kit("cfg-default");
    let path = settings_path(&kit);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r#"{"terminal.integrated.defaultProfile.windows":"Atollo - Git Bash"}"#,
    )
    .unwrap();
    write(&kit, Path::new(r"C:\Windows")).unwrap();
    assert_eq!(
        read(&kit)["terminal.integrated.defaultProfile.windows"],
        "Atollo - Git Bash"
    );
    fs::write(
        &path,
        r#"{"terminal.integrated.defaultProfile.windows":"PowerShell"}"#,
    )
    .unwrap();
    write(&kit, Path::new(r"C:\Windows")).unwrap();
    assert_eq!(
        read(&kit)["terminal.integrated.defaultProfile.windows"],
        "Atollo",
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

fn kit_on(name: &str, target: crate::kit::Target, installed: &[&str]) -> (TempDir, Kit) {
    let (tmp, kit) = kit_with(name, &[]);
    let kit = kit.for_target(target);
    for rel in installed {
        let f = crate::fsx::join_rel(kit.apps(), rel);
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(f, b"").unwrap();
    }
    (tmp, kit)
}

/// The profile names of `section` that are not hidden (`null`), sorted.
fn shown(s: &Map<String, Value>, section: &str) -> Vec<String> {
    let mut names: Vec<String> = s[section]
        .as_object()
        .unwrap()
        .iter()
        .filter(|(_, v)| !v.is_null())
        .map(|(k, _)| k.clone())
        .collect();
    names.sort();
    names
}

fn slashed(v: &Value) -> String {
    v.as_str().unwrap().replace('\\', "/")
}

#[test]
fn linux_profiles_bash_of_the_system_no_cmd() {
    use crate::kit::Target;
    // Git of the kit has no shell on Linux: its key is git/bin/git.
    let (_t, kit) = kit_on(
        "cfg-linux",
        Target::LINUX_X64,
        &["git/bin/git", "claude/claude"],
    );
    write(&kit, Path::new(r"C:\Windows")).unwrap();
    let s = read(&kit);
    assert!(s.get("terminal.integrated.profiles.windows").is_none());
    assert_eq!(
        shown(&s, "terminal.integrated.profiles.linux"),
        ["Claude Code", "Atollo - Bash"]
    );
    let p = &s["terminal.integrated.profiles.linux"];
    assert_eq!(p["Atollo - Bash"]["path"], "/bin/bash");
    assert_eq!(p["Atollo - Bash"]["args"][0], "--rcfile");
    assert!(slashed(&p["Atollo - Bash"]["args"][1]).ends_with("data/kit/bashrc"));
    assert_eq!(p["Atollo - Bash"]["args"][2], "-i");
    assert!(slashed(&p["Claude Code"]["path"]).ends_with("linux-x64/claude/claude"));
    assert!(p["bash"].is_null() && p["zsh"].is_null() && p["pwsh"].is_null());
    let text = serde_json::to_string(&s).unwrap();
    assert!(
        !text.contains("cmd.exe") && !text.contains("agy.cmd"),
        "{text}"
    );
    assert_eq!(
        s["terminal.integrated.defaultProfile.linux"], "Atollo - Bash",
        "no PowerShell of the kit: bash"
    );
    assert_eq!(
        s["terminal.integrated.automationProfile.linux"]["path"],
        "/bin/bash"
    );
    assert!(s["terminal.integrated.env.linux"]["NODE_OPTIONS"]
        .as_str()
        .unwrap()
        .ends_with("/data/kit/loopback.js\""));
    assert!(s.get("terminal.integrated.useWslProfiles").is_none());
}

#[test]
fn macos_profiles_pwsh_default_zsh_and_agy() {
    use crate::kit::Target;
    let (_t, kit) = kit_on(
        "cfg-mac",
        Target::DARWIN_ARM64,
        &["pwsh/pwsh", "antigravity/agy", "codex/bin/codex"],
    );
    let s = merge(&Map::new(), &kit, Path::new(r"C:\Windows"));
    assert_eq!(
        shown(&s, "terminal.integrated.profiles.osx"),
        [
            "Antigravity",
            "Codex",
            "Atollo",
            "Atollo - Bash",
            "Atollo - zsh"
        ]
    );
    let p = &s["terminal.integrated.profiles.osx"];
    assert!(slashed(&p["Atollo"]["path"]).ends_with("darwin-arm64/pwsh/pwsh"));
    assert_eq!(p["Atollo - zsh"]["path"], "/bin/zsh");
    // Antigravity through the kit's agy (sh): it carries the login.
    let agy = slashed(&p["Antigravity"]["path"]);
    assert!(agy.ends_with("data/kit/bin/agy"), "{agy}");
    // The shells macOS lists in /etc/shells are hidden, as on Windows.
    for shell in ["bash", "zsh", "sh", "csh", "tcsh", "ksh", "dash"] {
        assert!(p[shell].is_null(), "{shell} shown");
        assert!(p.get(shell).is_some(), "{shell} not hidden");
    }
    assert_eq!(s["terminal.integrated.defaultProfile.osx"], "Atollo");
}

#[test]
fn windows_profiles_unchanged_by_the_unix_ones() {
    let (_t, kit) = windows_kit("cfg-win-only");
    let s = merge(&Map::new(), &kit, Path::new(r"C:\Windows"));
    assert_eq!(
        shown(&s, "terminal.integrated.profiles.windows"),
        ["Atollo", "Atollo - Git Bash", "Atollo - cmd"]
    );
    // Joined by the host's separator off Windows: compared with slashes.
    assert_eq!(
        slashed(&s["terminal.integrated.profiles.windows"]["Atollo - cmd"]["path"]),
        "C:/Windows/System32/cmd.exe"
    );
    let hidden = &s["terminal.integrated.profiles.windows"];
    assert!(hidden["PowerShell"].is_null() && hidden["Command Prompt"].is_null());
    assert!(s
        .keys()
        .all(|k| !k.ends_with(".linux") && !k.ends_with(".osx")));
    assert_eq!(s["terminal.integrated.useWslProfiles"], false);
}

#[test]
fn each_system_keeps_the_others_settings() {
    use crate::kit::Target;
    let (_t, win) = kit_on("cfg-shared", Target::WIN32_X64, &["pwsh/pwsh.exe"]);
    write(&win, Path::new(r"C:\Windows")).unwrap();
    let linux = win.for_target(Target::LINUX_ARM64);
    write(&linux, Path::new(r"C:\Windows")).unwrap();
    let s = read(&linux);
    assert_eq!(s["terminal.integrated.defaultProfile.windows"], "Atollo");
    assert!(s["terminal.integrated.profiles.windows"]["Atollo - cmd"].is_object());
    assert!(s["terminal.integrated.profiles.linux"]
        .get("Atollo - cmd")
        .is_none());
    assert_eq!(
        s["terminal.integrated.defaultProfile.linux"],
        "Atollo - Bash"
    );
    // The user's default kept when it is a profile of that system.
    let mut user = s.clone();
    user.insert(
        "terminal.integrated.defaultProfile.linux".into(),
        json!("Mine"),
    );
    let mut mine = user["terminal.integrated.profiles.linux"]
        .as_object()
        .unwrap()
        .clone();
    mine.insert("Mine".into(), json!({ "path": "/usr/bin/fish" }));
    user.insert(
        "terminal.integrated.profiles.linux".into(),
        Value::Object(mine),
    );
    let again = merge(&user, &linux, Path::new(r"C:\Windows"));
    assert_eq!(again["terminal.integrated.defaultProfile.linux"], "Mine");
    assert_eq!(
        again["terminal.integrated.profiles.linux"]["Mine"]["path"],
        "/usr/bin/fish"
    );
}
