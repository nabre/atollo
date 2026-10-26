use super::*;
use crate::kit::tests::TempDir;

fn kit_with(name: &str, installed: &[&str]) -> (TempDir, Kit) {
    let tmp = TempDir::new(name);
    let kit = Kit::at(tmp.0.join("k (1) & co"), crate::test_support::kit_arch());
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

fn kit_on(name: &str, target: crate::kit::Target, installed: &[&str]) -> (TempDir, Kit) {
    let (tmp, kit) = kit_with(name, &[]);
    let kit = kit.for_target(target);
    for rel in installed {
        let f = crate::common::fsx::join_rel(kit.apps(), rel);
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

fn args_of(p: &Value) -> Vec<String> {
    p["args"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|a| a.as_str().unwrap().replace('\\', "/"))
        .collect()
}

/// The request client a profile runs (`launch.ps1` or `launch.sh`), if any.
fn client_of(p: &Value) -> Option<&'static str> {
    let args = args_of(p);
    if args.iter().any(|a| a.ends_with("data/kit/launch.ps1")) {
        Some("launch.ps1")
    } else if args.iter().any(|a| a.ends_with("data/kit/launch.sh")) {
        Some("launch.sh")
    } else {
        None
    }
}

/// D-179 (§13 item 10): only Atollo's profiles, rewritten at every start:
/// the user's own profiles go, and so do those an earlier kit left under
/// other names; the PC's detected ones are hidden; the user's other
/// settings stay.
#[test]
fn only_atollos_profiles() {
    let (_t, kit) = windows_kit("cfg-profiles");
    let path = settings_path(&kit);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let old_pwsh = kit.apps().join(r"pwsh\7.5\pwsh.exe");
    let old = json!({
        "terminal.integrated.profiles.windows": {
            "Mine": {"path": "x.exe"},
            "Atollo": {"path": "old"},
            "Codex": {"path": "gone"},
            "Old shell": {"path": old_pwsh, "overrideName": true},
        },
        "terminal.integrated.defaultProfile.windows": "Mine",
        "editor.fontSize": 15,
        "files.dialog.defaultPath": r"D:\mine",
    });
    fs::write(&path, old.to_string()).unwrap();
    assert_eq!(
        write(&kit, Path::new(r"C:\Windows"), false).unwrap(),
        Outcome::Written
    );
    let s = read(&kit);
    assert_eq!(
        shown(&s, "terminal.integrated.profiles.windows"),
        ["Atollo", "Atollo - Git Bash", "Atollo - cmd"],
        "without the VM no agent profile; nobody else's"
    );
    let p = &s["terminal.integrated.profiles.windows"];
    for hidden in [
        "PowerShell",
        "Windows PowerShell",
        "Command Prompt",
        "Git Bash",
    ] {
        assert!(p[hidden].is_null() && p.get(hidden).is_some(), "{hidden}");
    }
    assert_eq!(
        s["terminal.integrated.defaultProfile.windows"], "Atollo",
        "the user's default was not one of Atollo's"
    );
    assert_eq!(s["editor.fontSize"], 15);
    assert_eq!(s["files.dialog.defaultPath"], r"D:\mine", "user value kept");
    assert!(s["terminal.integrated.env.windows"]["NODE_OPTIONS"]
        .as_str()
        .unwrap()
        .ends_with("/data/kit/loopback.js\""));
    assert!(s["git.defaultCloneDirectory"]
        .as_str()
        .unwrap()
        .ends_with("projects"));
    assert_eq!(
        write(&kit, Path::new(r"C:\Windows"), false).unwrap(),
        Outcome::Unchanged
    );
    // A profile the user adds again is gone at the next start.
    let mut again = read(&kit);
    again["terminal.integrated.profiles.windows"]
        .as_object_mut()
        .unwrap()
        .insert("Mine".into(), json!({"path": "x.exe"}));
    fs::write(&path, Value::Object(again).to_string()).unwrap();
    assert_eq!(
        write(&kit, Path::new(r"C:\Windows"), false).unwrap(),
        Outcome::Written
    );
    assert!(read(&kit)["terminal.integrated.profiles.windows"]
        .get("Mine")
        .is_none());
}

/// D-179 (E2, §6.2): the terminals are level 1, the kit's shells on the PC
/// as they are, never through a request: no `launch`, no `vm.ps1`.
#[test]
fn terminals_are_level_one() {
    let (_t, kit) = windows_kit("cfg-terminals");
    let s = merge(&Map::new(), &kit, Path::new(r"C:\Windows"), false);
    let p = &s["terminal.integrated.profiles.windows"];
    assert!(slashed(&p["Atollo"]["path"]).ends_with("win32-x64/pwsh/pwsh.exe"));
    assert_eq!(
        args_of(&p["Atollo"]),
        args_of(&json!({"args": pwsh_args(&kit)}))
    );
    assert!(args_of(&p["Atollo"])
        .last()
        .unwrap()
        .ends_with("data/kit/profile.ps1"));
    assert!(slashed(&p["Atollo - Git Bash"]["path"]).ends_with("git/bin/bash.exe"));
    assert_eq!(args_of(&p["Atollo - Git Bash"])[0], "--rcfile");
    // Joined by the host's separator off Windows: compared with slashes.
    assert_eq!(
        slashed(&p["Atollo - cmd"]["path"]),
        "C:/Windows/System32/cmd.exe"
    );
    assert_eq!(args_of(&p["Atollo - cmd"]), ["/d", "/k"]);
    for name in ["Atollo", "Atollo - Git Bash", "Atollo - cmd"] {
        assert_eq!(client_of(&p[name]), None, "{name}");
        assert!(!p[name].to_string().contains("vm.ps1"), "{name}");
    }
    assert_eq!(s["terminal.integrated.useWslProfiles"], false);
    assert!(s
        .keys()
        .all(|k| !k.ends_with(".linux") && !k.ends_with(".osx")));
}

/// D-179 (E11, E12, §6.3, §6.7): the tasks (`automationProfile`) and the
/// agents' profiles go through the request client: `launch.ps1` with the
/// kit's PowerShell, `launch.sh` with Git Bash on Windows without it, with
/// the system's sh on macOS and Linux; each with its origin.
#[test]
fn tasks_and_agents_go_through_launch() {
    use crate::kit::Target;
    let (_t, kit) = kit_on("cfg-launch", Target::WIN32_X64, &["pwsh/pwsh.exe"]);
    guest_agent(&kit, "pi");
    let s = merge(&Map::new(), &kit, Path::new(r"C:\Windows"), true);
    let p = &s["terminal.integrated.profiles.windows"];
    let tasks = &s["terminal.integrated.automationProfile.windows"];
    assert_eq!(client_of(tasks), Some("launch.ps1"));
    assert!(slashed(&tasks["path"]).ends_with("pwsh/pwsh.exe"));
    let task_args = args_of(tasks);
    assert_eq!(task_args[task_args.len() - 2..], ["-Origin", "editorTask"]);
    assert!(!task_args.contains(&"-NoExit".to_owned()));
    for (name, id) in [("Claude Code", "claude"), ("Pi", "pi")] {
        assert_eq!(client_of(&p[name]), Some("launch.ps1"), "{name}");
        let args = args_of(&p[name]);
        assert_eq!(
            args[args.len() - 4..],
            ["-Origin", "editorAgentProfile", "-Agent", id],
            "{name}"
        );
    }
    assert_eq!(client_of(&p["Codex"]), Some("launch.ps1"), "brought by the VM");
    assert!(p.get("Antigravity").is_none(), "no build for the VM's Linux");

    // Without the kit's PowerShell: Git Bash with launch.sh.
    let (_t, kit) = kit_on("cfg-launch-sh", Target::WIN32_X64, &["git/bin/bash.exe"]);
    let s = merge(&Map::new(), &kit, Path::new(r"C:\Windows"), true);
    let tasks = &s["terminal.integrated.automationProfile.windows"];
    assert_eq!(client_of(tasks), Some("launch.sh"));
    assert!(slashed(&tasks["path"]).ends_with("git/bin/bash.exe"));
    assert_eq!(args_of(tasks)[1..], ["editorTask"]);
    let claude = &s["terminal.integrated.profiles.windows"]["Claude Code"];
    assert_eq!(args_of(claude)[1..], ["editorAgentProfile", "claude"]);

    // macOS with the kit's PowerShell, Linux without.
    let (_t, mac) = kit_on("cfg-launch-mac", Target::DARWIN_ARM64, &["pwsh/pwsh"]);
    let s = merge(&Map::new(), &mac, Path::new(r"C:\Windows"), true);
    assert_eq!(
        client_of(&s["terminal.integrated.automationProfile.osx"]),
        Some("launch.ps1")
    );
    assert_eq!(
        client_of(&s["terminal.integrated.profiles.osx"]["Codex"]),
        Some("launch.ps1")
    );
    let (_t, linux) = kit_on("cfg-launch-linux", Target::LINUX_X64, &[]);
    let s = merge(&Map::new(), &linux, Path::new(r"C:\Windows"), true);
    let tasks = &s["terminal.integrated.automationProfile.linux"];
    assert_eq!(tasks["path"], SH);
    assert_eq!(client_of(tasks), Some("launch.sh"));
    assert_eq!(
        args_of(&s["terminal.integrated.profiles.linux"]["Claude Code"])[1..],
        ["editorAgentProfile", "claude"]
    );
}

/// E18 (§13 item 7): a Windows kit with neither its PowerShell nor its Git
/// Bash runs no agent and no task on the PC: no agent profile, and the
/// task profile says why and fails, the task's line commented out.
#[test]
fn without_a_client_no_task_or_agent_on_the_pc() {
    use crate::kit::Target;
    let (_t, kit) = kit_on("cfg-no-client", Target::WIN32_X64, &["claude/claude.exe"]);
    assert_eq!(client(&kit), None);
    let s = merge(&Map::new(), &kit, Path::new(r"C:\Windows"), true);
    assert_eq!(
        shown(&s, "terminal.integrated.profiles.windows"),
        ["Atollo - cmd"]
    );
    let tasks = &s["terminal.integrated.automationProfile.windows"];
    assert!(slashed(&tasks["path"]).ends_with("WindowsPowerShell/v1.0/powershell.exe"));
    let args = args_of(tasks);
    assert_eq!(args[..3], ["-NoLogo", "-NoProfile", "-Command"]);
    let say = &args[3];
    assert!(
        say.contains(&NO_CLIENT.replace('\'', "''")) && say.ends_with("exit 1 #"),
        "{say}"
    );
    // Between PowerShell's quotes: each `'` of the text doubled.
    let inside = &say[say.find("('").unwrap() + 2..say.find("');").unwrap()];
    assert_eq!(inside.replace("''", ""), NO_CLIENT.replace('\'', ""));
    assert!(
        s.get("terminal.integrated.defaultProfile.windows")
            .is_none(),
        "no Atollo profile to default to: VSCodium's own choice among the shown"
    );
}

#[test]
fn user_default_profile_kept_when_atollos() {
    let (_t, kit) = windows_kit("cfg-default");
    let path = settings_path(&kit);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r#"{"terminal.integrated.defaultProfile.windows":"Atollo - Git Bash"}"#,
    )
    .unwrap();
    write(&kit, Path::new(r"C:\Windows"), false).unwrap();
    assert_eq!(
        read(&kit)["terminal.integrated.defaultProfile.windows"],
        "Atollo - Git Bash"
    );
    fs::write(
        &path,
        r#"{"terminal.integrated.defaultProfile.windows":"PowerShell"}"#,
    )
    .unwrap();
    write(&kit, Path::new(r"C:\Windows"), false).unwrap();
    assert_eq!(
        read(&kit)["terminal.integrated.defaultProfile.windows"],
        "Atollo",
        "hidden PC profile replaced"
    );
}

/// A file with comments and trailing commas is read without them, its
/// other keys kept; one that is still no JSON is moved aside and the keys
/// Atollo owns are written anew (§13 item 10).
#[test]
fn an_unreadable_file_is_rewritten() {
    let (_t, kit) = windows_kit("cfg-comments");
    let path = settings_path(&kit);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let text = "{\n  // mine, \"quoted\"\n  \"a\": \"x // not a comment\", /* b */\n  \"terminal.integrated.profiles.windows\": {\"Mine\": {\"path\": \"x.exe\"},},\n}";
    fs::write(&path, text).unwrap();
    assert_eq!(
        write(&kit, Path::new(r"C:\Windows"), false).unwrap(),
        Outcome::Written
    );
    let s = read(&kit);
    assert_eq!(s["a"], "x // not a comment");
    assert!(s["terminal.integrated.profiles.windows"]
        .get("Mine")
        .is_none());
    assert_eq!(s["terminal.integrated.defaultProfile.windows"], "Atollo");

    fs::write(&path, "{ not json").unwrap();
    let Outcome::Replaced(aside) = write(&kit, Path::new(r"C:\Windows"), false).unwrap() else {
        panic!("not replaced")
    };
    assert_eq!(fs::read_to_string(&aside).unwrap(), "{ not json");
    assert!(aside
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("settings.json.bak-"));
    let s = read(&kit);
    assert_eq!(
        client_of(&s["terminal.integrated.automationProfile.windows"]),
        Some("launch.ps1")
    );
    fs::write(&path, "[1, 2]").unwrap();
    assert!(matches!(
        write(&kit, Path::new(r"C:\Windows"), false).unwrap(),
        Outcome::Replaced(_)
    ));
    assert_eq!(strip_jsonc(r#"{"a":"\"//",}"#), r#"{"a":"\"//"}"#);
}

#[test]
fn geist_mono_by_default_user_font_kept() {
    let (_t, kit) = kit_with("cfg-fonts", &[]);
    write(&kit, Path::new(r"C:\Windows"), false).unwrap();
    let s = read(&kit);
    assert_eq!(s["editor.fontFamily"], EDITOR_FONT);
    assert_eq!(s["terminal.integrated.fontFamily"], EDITOR_FONT);
    assert_eq!(s["editor.fontLigatures"], false);
    assert!(EDITOR_FONT.starts_with("'Geist Mono',"));
    assert!(EDITOR_FONT.ends_with("monospace"));

    let path = settings_path(&kit);
    fs::write(
        &path,
        r#"{"editor.fontFamily":"Fira Code","editor.fontLigatures":true}"#,
    )
    .unwrap();
    write(&kit, Path::new(r"C:\Windows"), false).unwrap();
    let s = read(&kit);
    assert_eq!(s["editor.fontFamily"], "Fira Code");
    assert_eq!(s["editor.fontLigatures"], true);
    assert_eq!(s["terminal.integrated.fontFamily"], EDITOR_FONT);
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
    write(&kit, Path::new(r"C:\Windows"), false).unwrap();
    let s = read(&kit);
    let colors = s["workbench.colorCustomizations"].as_object().unwrap();
    assert_eq!(colors.len(), 1, "the user's own kept: {colors:?}");
    assert_eq!(colors["editor.background"], "#000000");
    assert_eq!(s["workbench.colorTheme"], "Atollo Dark");
    assert_eq!(s["workbench.experimental.modernUI"], true);
    assert_eq!(s["workbench.preferredDarkColorTheme"], "Atollo Dark");
    assert_eq!(s["workbench.preferredLightColorTheme"], "Solarized Light");

    fs::write(&path, r#"{"workbench.preferredLightColorTheme":"Default Light Modern","workbench.colorCustomizations":{"[Default Dark Modern]":{}}}"#).unwrap();
    write(&kit, Path::new(r"C:\Windows"), false).unwrap();
    let s = read(&kit);
    assert_eq!(s["workbench.preferredLightColorTheme"], "Atollo Light");
    assert!(s.get("workbench.colorCustomizations").is_none());
}

#[test]
fn linux_profiles_bash_of_the_system_no_cmd() {
    use crate::kit::Target;
    // Git of the kit has no shell on Linux: its key is git/bin/git.
    let (_t, kit) = kit_on("cfg-linux", Target::LINUX_X64, &["git/bin/git"]);
    write(&kit, Path::new(r"C:\Windows"), true).unwrap();
    let s = read(&kit);
    assert!(s.get("terminal.integrated.profiles.windows").is_none());
    assert_eq!(
        shown(&s, "terminal.integrated.profiles.linux"),
        ["Atollo - Bash", "Claude Code", "Codex"]
    );
    let p = &s["terminal.integrated.profiles.linux"];
    assert_eq!(p["Atollo - Bash"]["path"], "/bin/bash");
    assert_eq!(p["Atollo - Bash"]["args"][0], "--rcfile");
    assert!(slashed(&p["Atollo - Bash"]["args"][1]).ends_with("data/kit/bashrc"));
    assert_eq!(p["Atollo - Bash"]["args"][2], "-i");
    assert_eq!(p["Claude Code"]["path"], SH);
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
    assert!(s["terminal.integrated.env.linux"]["NODE_OPTIONS"]
        .as_str()
        .unwrap()
        .ends_with("/data/kit/loopback.js\""));
    assert!(s.get("terminal.integrated.useWslProfiles").is_none());
}

#[test]
fn macos_profiles_pwsh_default_zsh_and_agents() {
    use crate::kit::Target;
    // Antigravity for the PC (an earlier Atollo's) has no profile; its
    // build for the VM's Linux has.
    let (_t, kit) = kit_on(
        "cfg-mac",
        Target::DARWIN_ARM64,
        &["pwsh/pwsh", "antigravity/agy"],
    );
    let s = merge(&Map::new(), &kit, Path::new(r"C:\Windows"), true);
    assert!(s["terminal.integrated.profiles.osx"]
        .get("Antigravity")
        .is_none());
    guest_agent(&kit, "antigravity");
    let s = merge(&Map::new(), &kit, Path::new(r"C:\Windows"), true);
    assert_eq!(
        shown(&s, "terminal.integrated.profiles.osx"),
        [
            "Antigravity",
            "Atollo",
            "Atollo - Bash",
            "Atollo - zsh",
            "Claude Code",
            "Codex"
        ]
    );
    let p = &s["terminal.integrated.profiles.osx"];
    assert!(slashed(&p["Atollo"]["path"]).ends_with("darwin-arm64/pwsh/pwsh"));
    assert_eq!(p["Atollo - zsh"]["path"], "/bin/zsh");
    assert_eq!(client_of(&p["Atollo - zsh"]), None, "level 1");
    // No agent through the kit's launchers on the PC any more: the
    // request client, for each.
    for name in ["Antigravity", "Claude Code", "Codex"] {
        assert_eq!(client_of(&p[name]), Some("launch.ps1"), "{name}");
        assert!(!p[name].to_string().contains("data/kit/agents"), "{name}");
    }
    // The shells macOS lists in /etc/shells are hidden, as on Windows.
    for shell in ["bash", "zsh", "sh", "csh", "tcsh", "ksh", "dash"] {
        assert!(p[shell].is_null(), "{shell} shown");
        assert!(p.get(shell).is_some(), "{shell} not hidden");
    }
    assert_eq!(s["terminal.integrated.defaultProfile.osx"], "Atollo");
}

/// `id`'s build for the VM's Linux of `kit`'s system, on disk.
fn guest_agent(kit: &Kit, id: &str) {
    let guest = kit.for_target(crate::isolation::vm::guest_target(kit.target()).unwrap());
    for path in crate::setup::Catalog::get()
        .find(id)
        .unwrap()
        .key_paths(&guest)
    {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"").unwrap();
    }
}

/// LEVELS.md §7 (I6): without the VM no agent profile at all, whatever is
/// installed; with it, Claude Code and Codex (the `sbx` images bring them)
/// and the agents whose build for the VM's Linux is in the kit, never one
/// installed for the PC. A default on an agent's profile falls back to
/// «Atollo»; the profiles come back at the editor's next start.
#[test]
fn no_agent_profile_without_the_vm() {
    use crate::kit::Target;
    let (_t, kit) = kit_on(
        "cfg-guest-agent",
        Target::WIN32_X64,
        &["pwsh/pwsh.exe", "pi/pi.exe", "claude/claude.exe"],
    );
    guest_agent(&kit, "antigravity");
    let agents = ["Antigravity", "Claude Code", "Codex", "Pi"];
    let off = merge(&Map::new(), &kit, Path::new(r"C:\Windows"), false);
    let p = &off["terminal.integrated.profiles.windows"];
    for name in agents {
        assert!(p.get(name).is_none(), "{name}");
    }
    let on = merge(&Map::new(), &kit, Path::new(r"C:\Windows"), true);
    let p = &on["terminal.integrated.profiles.windows"];
    for name in ["Antigravity", "Claude Code", "Codex"] {
        assert_eq!(client_of(&p[name]), Some("launch.ps1"), "{name}");
    }
    assert!(p.get("Pi").is_none(), "Pi for the PC only: no profile");

    let path = settings_path(&kit);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r#"{"terminal.integrated.defaultProfile.windows":"Claude Code"}"#,
    )
    .unwrap();
    write(&kit, Path::new(r"C:\Windows"), true).unwrap();
    assert_eq!(
        read(&kit)["terminal.integrated.defaultProfile.windows"],
        "Claude Code"
    );
    write(&kit, Path::new(r"C:\Windows"), false).unwrap();
    let s = read(&kit);
    assert_eq!(s["terminal.integrated.defaultProfile.windows"], "Atollo");
    assert!(s["terminal.integrated.profiles.windows"]
        .get("Claude Code")
        .is_none());
}

#[test]
fn each_system_keeps_the_others_settings() {
    use crate::kit::Target;
    let (_t, win) = kit_on("cfg-shared", Target::WIN32_X64, &["pwsh/pwsh.exe"]);
    write(&win, Path::new(r"C:\Windows"), false).unwrap();
    let linux = win.for_target(Target::LINUX_ARM64);
    write(&linux, Path::new(r"C:\Windows"), false).unwrap();
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
    // A default of the user's on a profile of the user's: both go.
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
    let again = merge(&user, &linux, Path::new(r"C:\Windows"), false);
    assert_eq!(
        again["terminal.integrated.defaultProfile.linux"],
        "Atollo - Bash"
    );
    assert!(again["terminal.integrated.profiles.linux"]
        .get("Mine")
        .is_none());
    assert!(
        again["terminal.integrated.profiles.windows"]["Atollo - cmd"].is_object(),
        "Windows' settings left as they were"
    );
}
