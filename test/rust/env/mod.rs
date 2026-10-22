use super::*;
use crate::kit::{tests::TempDir, Arch, Target};

pub(crate) fn host() -> Host {
    Host::from_pairs(
        [
            ("SystemRoot", r"C:\Windows"),
            (
                "Path",
                r"C:\Users\me\bin;C:\Windows\System32;C:\Program Files\Thing",
            ),
            ("USERPROFILE", r"C:\Users\me"),
            ("APPDATA", r"C:\Users\me\AppData\Roaming"),
            ("OneDrive", r"C:\Users\me\OneDrive"),
            ("PATHEXT", ".COM;.EXE;.BAT;.CMD"),
            ("HTTPS_PROXY", "http://proxy:8080"),
            ("CARGO_HOME", r"C:\Users\me\.cargo"),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into())),
    )
}

pub(crate) fn fake_kit(name: &str, installed: &[&str]) -> (TempDir, Kit) {
    let tmp = TempDir::new(name);
    let kit = Kit::at(tmp.0.join("My Kit (1) & co"), Arch::X64);
    for rel in installed {
        // The layout of the kit's system: `node/node.exe` is `node/bin/node`
        // on macOS and Linux.
        let file = crate::fsx::join_rel(
            kit.apps(),
            &crate::setup::catalog::translate_rel(kit.target(), rel),
        );
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, b"").unwrap();
    }
    (tmp, kit)
}

/// data/home/AppData only for a Windows kit: elsewhere the variables are
/// set but nothing creates the folders.
#[test]
fn appdata_folders_only_on_windows() {
    for (target, made) in [(Target::WIN32_X64, true), (Target::LINUX_X64, false)] {
        let tmp = TempDir::new("env-appdata");
        let kit = Kit::at_target(tmp.0.join("Kit"), target);
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        env.create_dirs().unwrap();
        let home = kit.data().join("home");
        assert!(env.get("APPDATA").is_some(), "{target:?}");
        assert_eq!(home.join("AppData").join("Roaming").is_dir(), made);
        assert_eq!(home.join("AppData").join("Local").is_dir(), made);
        assert!(home.is_dir());
    }
}

#[test]
fn profile_and_temp_inside_kit() {
    let (_tmp, kit) = fake_kit("env-profile", &[]);
    let env = KitEnv::build(&kit, &Settings::default(), "it", &host());
    let data = kit.data();
    for name in [
        "USERPROFILE",
        "HOME",
        "APPDATA",
        "LOCALAPPDATA",
        "TEMP",
        "TMP",
        "CLAUDE_CONFIG_DIR",
        "CODEX_HOME",
        "NPM_CONFIG_CACHE",
        "PIP_CACHE_DIR",
        "CARGO_HOME",
        "GOPATH",
    ] {
        let value = env.get(name).unwrap_or_else(|| panic!("{name} missing"));
        assert!(Path::new(value).starts_with(&data), "{name} = {value}");
    }
    assert_eq!(env.get("CP_USER_PROFILE"), Some(r"C:\Users\me"));
    assert_eq!(env.get("ATOLLO"), Some(kit.root().to_str().unwrap()));
    assert_eq!(env.get("CP_LANG"), Some("it"));
    assert_eq!(env.get("DISABLE_UPDATES"), Some("1"));
    // HOMEDRIVE and HOMEPATH only with a drive letter.
    assert_eq!(
        env.get("HOMEPATH")
            .map(|p| Path::new(p).ends_with(Path::new("data").join("home"))),
        cfg!(windows).then_some(true)
    );
}

#[test]
fn program_temp_decision_table() {
    use crate::os::volume::FsKind;
    let vol = |fs, drive| {
        Some(VolumeInfo {
            fs,
            drive,
            free: 1,
            noexec: false,
        })
    };
    let (data, host) = if cfg!(windows) {
        (
            Path::new(r"E:\Kit\data"),
            Path::new(r"C:\Users\me\AppData\Local\Temp\Atollo\abc"),
        )
    } else {
        (Path::new("/media/u/Kit/data"), Path::new("/tmp/Atollo/abc"))
    };
    let in_kit = data.join("tmp").join("system");
    let in_sys = host.join("system");
    let ntfs_fixed = vol(FsKind::Ntfs, DriveKind::Fixed);
    let get = |choice, v, h: Option<&Path>| program_temp_dir(choice, v, h, data, false);
    let table = [
        (ProgramTemp::Auto, ntfs_fixed, &in_kit),
        (
            ProgramTemp::Auto,
            vol(FsKind::ExFat, DriveKind::Removable),
            &in_sys,
        ),
        (
            ProgramTemp::Auto,
            vol(FsKind::Fat32, DriveKind::Fixed),
            &in_sys,
        ),
        (
            ProgramTemp::Auto,
            vol(FsKind::Ntfs, DriveKind::Removable),
            &in_sys,
        ),
        (
            ProgramTemp::Auto,
            vol(FsKind::Ntfs, DriveKind::Remote),
            &in_sys,
        ),
        (
            ProgramTemp::Auto,
            vol(FsKind::Other, DriveKind::Fixed),
            &in_sys,
        ),
        (ProgramTemp::Auto, None, &in_kit),
        (
            ProgramTemp::Kit,
            vol(FsKind::ExFat, DriveKind::Removable),
            &in_kit,
        ),
        (ProgramTemp::System, ntfs_fixed, &in_sys),
        (ProgramTemp::System, None, &in_sys),
    ];
    for (choice, v, want) in table {
        assert_eq!(&get(choice, v, Some(host)), want, "{choice:?} {v:?}");
    }
    // no host temp: always the kit
    assert_eq!(
        get(
            ProgramTemp::System,
            vol(FsKind::ExFat, DriveKind::Removable),
            None
        ),
        in_kit
    );
    // Claude's folder follows the same rule
    let claude = |c| program_temp_dir(c, ntfs_fixed, Some(host), data, true);
    assert_eq!(claude(ProgramTemp::Auto), data.join("tmp"));
    assert_eq!(claude(ProgramTemp::System), host.join("claude"));
}

#[test]
fn host_temp_order_ends_with_tmpdir() {
    let of = |pairs: &[(&str, &str)]| {
        Host::from_pairs(pairs.iter().map(|(k, v)| ((*k).into(), (*v).into())))
            .temp()
            .map(str::to_owned)
    };
    // macOS and Linux desktops set TMPDIR only (BUG-V7-17).
    assert_eq!(
        of(&[("TMPDIR", "/var/folders/x/T")]),
        Some("/var/folders/x/T".into())
    );
    assert_eq!(
        of(&[("TMPDIR", "/t"), ("TMP", "/tmp2")]),
        Some("/tmp2".into())
    );
    assert_eq!(
        of(&[("TMP", "/tmp2"), ("TEMP", "/tmp1")]),
        Some("/tmp1".into())
    );
    assert_eq!(
        of(&[("TEMP", "/tmp1"), ("CP_HOST_TEMP", "/host")]),
        Some("/host".into())
    );
    assert_eq!(of(&[]), None);
    // The environment passes on the same folder the app itself uses.
    let (_tmp, kit) = fake_kit("env-hosttemp", &[]);
    let host = Host::from_pairs([("TMPDIR".into(), "/var/folders/x/T".into())]);
    let env = KitEnv::build(&kit, &Settings::default(), "it", &host);
    assert_eq!(env.get("CP_HOST_TEMP"), Some("/var/folders/x/T"));
}

#[test]
fn system_temp_reaches_the_environment() {
    let (tmp, kit) = fake_kit("env-systemtemp", &[]);
    let kit = kit.with_host(Some(&tmp.0.join("host")));
    let s = Settings {
        program_temp: ProgramTemp::System,
        ..Settings::default()
    };
    let env = KitEnv::build(&kit, &s, "it", &host());
    let sys = kit.system_temp().unwrap();
    assert_eq!(Path::new(env.get("TEMP").unwrap()), sys.join("system"));
    assert_eq!(Path::new(env.get("TMP").unwrap()), sys.join("system"));
    assert_eq!(
        Path::new(env.get("CLAUDE_CODE_TMPDIR").unwrap()),
        sys.join("claude")
    );
}

#[test]
fn nothing_from_the_user_profile_leaks() {
    let (_tmp, kit) = fake_kit("env-leak", &["git/bin/bash.exe", "node/node.exe"]);
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    for (name, value) in env.iter() {
        if name == "CP_USER_PROFILE" || name == "CP_HOST_TEMP" {
            continue;
        }
        assert!(
            !value.contains(r"C:\Users\me"),
            "{name} leaks the real profile: {value}"
        );
    }
    assert!(env.get("OneDrive").is_none());
    assert_eq!(env.get("PATHEXT"), Some(".COM;.EXE;.BAT;.CMD"));
    assert_eq!(env.get("HTTPS_PROXY"), Some("http://proxy:8080"));
}

#[test]
fn node_options_uses_forward_slashes() {
    let (_tmp, kit) = fake_kit("env-node", &[]);
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    let opts = env.get("NODE_OPTIONS").unwrap();
    assert!(opts.starts_with("--require \"") && opts.ends_with("/data/kit/loopback.js\""));
    assert!(!opts.contains('\\'));
}

#[test]
fn settings_override_and_add() {
    let (_tmp, kit) = fake_kit("env-settings", &[]);
    let s = Settings {
        lang: Some("fr".into()),
        network: crate::settings::Network {
            https_proxy: Some("http://corp:3128".into()),
            allow_lan: true,
            ..Default::default()
        },
        gemini_api_key: Some("abc".into()),
        ..Settings::default()
    };
    let env = KitEnv::build(&kit, &s, "fr", &host());
    assert_eq!(env.get("HTTPS_PROXY"), Some("http://corp:3128"));
    assert_eq!(env.get("ATOLLO_ALLOW_LAN"), Some("1"));
    assert_eq!(env.get("ATOLLO_LANG"), Some("fr"));
    // The Gemini key reaches Antigravity only (`agents::launch_env`), never
    // every program of the kit (SEC-V6-02).
    assert_eq!(env.get("GEMINI_API_KEY"), None);
    assert!(!env.get(SETTINGS_VARS).unwrap().contains("GEMINI"));
}

#[test]
fn keys_unique_ignoring_case() {
    let (_tmp, kit) = fake_kit("env-case", &[]);
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    let mut seen = std::collections::HashSet::new();
    for (k, _) in env.iter() {
        assert!(seen.insert(k.to_ascii_uppercase()), "duplicate {k}");
    }
}

/// `Atollo env` (`cli::print_env`) and `{:?}` of the kit's environment
/// never show a secret's value.
#[test]
fn print_env_and_debug_mask_secrets() {
    let (_tmp, kit) = fake_kit("env-print", &[]);
    let s = Settings {
        network: crate::settings::Network {
            https_proxy: Some("http://me:hunter2@proxy:8080".into()),
            ..Default::default()
        },
        gemini_api_key: Some("AIzaSECRET".into()),
        ..Settings::default()
    };
    let mut env = KitEnv::build(&kit, &s, "en", &host());
    // An agent's environment (`agents::launch_env`): its secrets too.
    env.set("GEMINI_API_KEY", "AIzaSECRET");
    env.set("GH_TOKEN", "ghp_SECRET");
    env.set("CLAUDE_CODE_OAUTH_TOKEN", "sk-ant-oat01-SECRET");
    let mut out = Vec::new();
    crate::cli::print_env(&env, &mut out).unwrap();
    let out = String::from_utf8(out).unwrap();
    for line in [
        "GEMINI_API_KEY=***",
        "GH_TOKEN=***",
        "CLAUDE_CODE_OAUTH_TOKEN=***",
        "HTTPS_PROXY=http://me:***@proxy:8080",
    ] {
        assert!(out.lines().any(|l| l == line), "{line}");
    }
    assert!(out.lines().any(|l| l.starts_with("PATH=") && l.len() > 5));
    for debug in [format!("{env:?}"), format!("{env:#?}")] {
        for secret in ["SECRET", "hunter2"] {
            assert!(!out.contains(secret) && !debug.contains(secret), "{secret}");
        }
        assert!(debug.contains("GH_TOKEN"));
    }
}

#[test]
fn unix_target_builds_colon_path_and_xdg_dirs() {
    use crate::kit::Target;

    let tmp = TempDir::new("env-unix");
    let kit = Kit::at_target(tmp.0.join("Kit"), Target::DARWIN_ARM64);
    for rel in [
        "git/bin/git",
        "node/bin/node",
        "python/bin/python3",
        "pwsh/pwsh",
        "go/bin/go",
        "tinytex/bin/universal-darwin/tlmgr",
    ] {
        let file = crate::fsx::join_rel(kit.apps(), rel);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, b"").unwrap();
    }
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    let path = env.get("PATH").unwrap();
    assert!(path.contains(':'), "Unix target must join PATH with ':'");
    let entries = env.path_entries();
    let n = entries.len();
    // macOS: Homebrew after the kit's programs (D-105).
    assert_eq!(
        &entries[n - 6..],
        [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/usr/bin",
            "/bin",
            "/usr/sbin",
            "/sbin"
        ]
    );
    let apps = kit.apps();
    assert!(entries
        .iter()
        .any(|e| Path::new(e) == crate::fsx::join_rel(&apps, "node/bin")));
    assert!(entries
        .iter()
        .any(|e| Path::new(e) == crate::fsx::join_rel(&apps, "python/bin")));
    assert!(entries
        .iter()
        .any(|e| Path::new(e) == crate::fsx::join_rel(&apps, "tinytex/bin/universal-darwin")));
    for xdg in [
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
        "XDG_CACHE_HOME",
        "XDG_STATE_HOME",
        "TMPDIR",
    ] {
        let v = env.get(xdg).unwrap_or_else(|| panic!("{xdg} missing"));
        assert!(Path::new(v).starts_with(kit.data()), "{xdg} = {v}");
    }
}

/// The environment of a Linux and a macOS kit (D-105), as the table of
/// Windows above: the desktop session kept, no Windows-only variable, the
/// system folders after the kit's.
#[test]
fn linux_and_macos_environment_table() {
    use crate::kit::Target;

    let host = Host::from_pairs(
        [
            ("HOME", "/home/me"),
            ("PATH", "/home/me/.local/bin:/usr/bin"),
            ("DISPLAY", ":0"),
            ("WAYLAND_DISPLAY", "wayland-0"),
            ("XAUTHORITY", "/run/user/1000/.mutter-Xwaylandauth.X1"),
            ("XDG_CURRENT_DESKTOP", "GNOME"),
            ("XDG_SESSION_TYPE", "wayland"),
            ("XDG_CONFIG_HOME", "/home/me/.config"),
            ("LANG", "it_IT.UTF-8"),
        ]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into())),
    );
    for (target, system) in [
        (
            Target::LINUX_X64,
            &["/usr/local/bin", "/usr/bin", "/bin", "/usr/sbin", "/sbin"][..],
        ),
        (
            Target::DARWIN_ARM64,
            &[
                "/opt/homebrew/bin",
                "/usr/local/bin",
                "/usr/bin",
                "/bin",
                "/usr/sbin",
                "/sbin",
            ][..],
        ),
    ] {
        let tmp = TempDir::new(&format!("env-table-{target}"));
        let kit = Kit::at_target(tmp.0.join("Kit"), target);
        let env = KitEnv::build(&kit, &Settings::default(), "it", &host);
        for (name, value) in [
            ("DISPLAY", ":0"),
            ("WAYLAND_DISPLAY", "wayland-0"),
            ("XAUTHORITY", "/run/user/1000/.mutter-Xwaylandauth.X1"),
            ("XDG_CURRENT_DESKTOP", "GNOME"),
            ("XDG_SESSION_TYPE", "wayland"),
            ("LANG", "it_IT.UTF-8"),
            ("CP_USER_PROFILE", "/home/me"),
        ] {
            assert_eq!(env.get(name), Some(value), "{target} {name}");
        }
        // Windows only; a host path with a drive letter (this test on
        // Windows) is not one on these systems either.
        for name in [
            "GCM_CREDENTIAL_STORE",
            "GCM_DPAPI_STORE_PATH",
            "HOMEDRIVE",
            "HOMEPATH",
        ] {
            assert_eq!(env.get(name), None, "{target} {name}");
        }
        let xdg = env.get("XDG_CONFIG_HOME").unwrap();
        assert!(Path::new(xdg).starts_with(kit.data()), "{target} {xdg}");
        let entries = env.path_entries();
        assert_eq!(&entries[entries.len() - system.len()..], system, "{target}");
        assert!(!entries.contains(&"/home/me/.local/bin"), "{target}");
    }
}

/// Windows keeps Git Credential Manager's DPAPI store.
#[test]
fn credential_store_on_windows_only() {
    let (_tmp, kit) = fake_kit("env-gcm", &[]);
    let kit = Kit::at_target(kit.root().to_path_buf(), crate::kit::Target::WIN32_X64);
    let env = KitEnv::build(&kit, &Settings::default(), "it", &host());
    assert_eq!(env.get("GCM_CREDENTIAL_STORE"), Some("dpapi"));
    assert!(env.get("GCM_DPAPI_STORE_PATH").is_some());
}

/// Files under `data/apps/<target>` of `kit` (forward slashes).
pub(crate) fn touch(kit: &Kit, rels: &[&str]) {
    for rel in rels {
        let file = crate::fsx::join_rel(kit.apps(), rel);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, b"").unwrap();
    }
}

/// The shims install/lang/cc.rs writes in `cc/bin` on macOS and Linux.
pub(crate) const CC_SHIMS: [&str; 9] = [
    "cc/bin/cc",
    "cc/bin/gcc",
    "cc/bin/clang",
    "cc/bin/c++",
    "cc/bin/g++",
    "cc/bin/clang++",
    "cc/bin/ar",
    "cc/bin/ranlib",
    "cc/bin/make",
];

/// Atollo started from a kit console: what that console got from the
/// settings is not the PC's (D-137).
#[test]
fn settings_of_a_kit_console_are_not_the_hosts() {
    let (_tmp, kit) = fake_kit("env-settings-vars", &[]);
    let s = Settings {
        lang: Some("de".into()),
        ..Settings::default()
    };
    let env = KitEnv::build(&kit, &s, "de", &host());
    let marker = env.get(SETTINGS_VARS).unwrap();
    assert!(marker.split(';').any(|n| n == "ATOLLO_LANG"), "{marker}");
    // The PC's own proxy is not the settings'.
    assert!(!marker.split(';').any(|n| n == "HTTPS_PROXY"), "{marker}");

    let mut console = Host::from_pairs(
        env.iter()
            .map(|(k, v)| (k.to_owned().into(), v.to_owned().into())),
    );
    let gone = console.forget_kit_settings();
    assert!(gone.iter().any(|n| n == "ATOLLO_LANG"));
    assert!(console.get("ATOLLO_LANG").is_none());
    assert!(console.get(SETTINGS_VARS).is_none());
    assert_eq!(console.get("HTTPS_PROXY"), Some("http://proxy:8080"));
    // A setting emptied since: the language is the system's again.
    assert_eq!(
        crate::i18n::resolve(None, console.get("ATOLLO_LANG"), Some("it-IT")),
        "it"
    );
}
