use super::*;
use crate::kit::{tests::TempDir, Arch};

fn host() -> Host {
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

fn fake_kit(name: &str, installed: &[&str]) -> (TempDir, Kit) {
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
fn path_is_kit_then_system_only() {
    let windows = cfg!(windows);
    let (_tmp, kit) = fake_kit(
        "env-path",
        &[
            "git/bin/bash.exe",
            "node/node.exe",
            "python/python.exe",
            "pwsh/pwsh.exe",
            "go/bin/go.exe",
        ],
    );
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    let entries = env.path_entries();
    let system: Vec<String> = if windows {
        vec![
            r"C:\Windows\System32".into(),
            r"C:\Windows\System32\Wbem".into(),
            r"C:\Windows\System32\WindowsPowerShell\v1.0".into(),
        ]
    } else {
        system_path(kit.os())
            .iter()
            .map(|s| s.to_string())
            .collect()
    };
    let n = entries.len() - system.len();
    assert_eq!(&entries[n..], system);
    for entry in &entries[..n] {
        assert!(Path::new(entry).starts_with(kit.data()), "{entry}");
    }
    let apps = kit.apps();
    let pos = |p: PathBuf| {
        entries
            .iter()
            .position(|e| Path::new(e) == p)
            .unwrap_or_else(|| panic!("{} missing", p.display()))
    };
    // Node and Python have their programs in `bin` off Windows; Git is
    // `cmd` on Windows, `bin` elsewhere.
    let bin = |d: &str| {
        if windows {
            apps.join(d)
        } else {
            apps.join(d).join("bin")
        }
    };
    let git = apps.join("git").join(if windows { "cmd" } else { "bin" });
    assert!(pos(apps.join("claude")) < pos(bin("python")));
    assert!(pos(bin("node")) < pos(apps.join("go").join("bin")));
    assert!(pos(apps.join("go").join("bin")) < pos(git));
    assert!(
        entries
            .iter()
            .all(|e| !Path::new(e).starts_with(apps.join("java"))),
        "java not installed"
    );
    let bash = Path::new("git").join("bin").join("bash.exe");
    assert_eq!(
        env.get("CLAUDE_CODE_GIT_BASH_PATH")
            .map(|p| Path::new(p).ends_with(&bash)),
        windows.then_some(true)
    );
    assert!(env.get("JAVA_HOME").is_none());
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
    assert_eq!(env.get("GEMINI_API_KEY"), Some("abc"));
    assert_eq!(env.get("ATOLLO_LANG"), Some("fr"));
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

#[test]
fn secrets_are_masked() {
    assert_eq!(
        mask_secrets("GEMINI_API_KEY=abc123 next"),
        "GEMINI_API_KEY=*** next"
    );
    assert_eq!(
        mask_secrets("url ?tkn=deadbeef&folder=x"),
        "url ?tkn=***&folder=x"
    );
    assert_eq!(mask_secrets("GH_TOKEN=\"x y\" ok"), "GH_TOKEN=\"***\" ok");
    assert_eq!(mask_secrets("SECRET='a b"), "SECRET='***'");
    assert_eq!(
        mask_secrets("PATH=C:\\x MY_PASSWORD=p"),
        "PATH=C:\\x MY_PASSWORD=***"
    );
    assert_eq!(mask_secrets("no secrets here"), "no secrets here");
}

/// What `Atollo env` and `KitEnv`'s `Debug` show: every `*_TOKEN`,
/// `*_KEY`, secret and password masked, a proxy's password too; the rest
/// as it is.
#[test]
fn secret_values_are_masked_by_name() {
    for name in [
        "CLAUDE_CODE_OAUTH_TOKEN",
        "GH_TOKEN",
        "github_token",
        "GEMINI_API_KEY",
        "ATOLLO_FCGI_KEY",
        "CLIENT_SECRET",
        "DB_PASSWORD",
        "MYSQL_PASSWD",
        "tkn",
    ] {
        assert!(is_secret(name), "{name}");
        assert_eq!(masked(name, "sk-ant-oat01-x"), "***", "{name}");
    }
    for name in [
        "PATH",
        "HOME",
        "ATOLLO_LANG",
        "GCM_CREDENTIAL_STORE",
        "TERM",
    ] {
        assert!(!is_secret(name), "{name}");
    }
    assert_eq!(masked("PATH", "/a:/b"), "/a:/b");
    assert_eq!(
        masked("HTTPS_PROXY", "http://me:p%40ss@proxy:8080/x"),
        "http://me:***@proxy:8080/x"
    );
    assert_eq!(
        masked("HTTP_PROXY", "http://token@proxy:8080"),
        "http://token:***@proxy:8080"
    );
    assert_eq!(
        masked("HTTP_PROXY", "http://proxy:8080/a@b"),
        "http://proxy:8080/a@b"
    );
    assert_eq!(
        masked("HTTP_PROXY", "http://proxy:8080"),
        "http://proxy:8080"
    );
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
fn languages_from_the_folder_of_their_version() {
    let windows = cfg!(windows);
    let files: &[&str] = if windows {
        &[
            "node/22.11/node.exe",
            "php/8.4/php.exe",
            "java/21.0/bin/java.exe",
        ]
    } else {
        &[
            "node/22.11/bin/node",
            "php/8.4/bin/php",
            "java/21.0/bin/java",
        ]
    };
    let (_tmp, kit) = fake_kit("env-versions", files);
    let catalog = crate::setup::catalog::Catalog::get();
    for (id, version) in [
        ("node", "v22.11.0"),
        ("php", "8.4.13"),
        ("java", "21.0.5+11"),
    ] {
        catalog
            .find(id)
            .unwrap()
            .save_version(&kit, version)
            .unwrap();
    }
    let env = KitEnv::build(&kit, &Settings::default(), "it", &host());
    let apps = kit.apps();
    let entries = env.path_entries();
    let node = apps.join("node").join("22.11");
    let node_bin = if windows { node } else { node.join("bin") };
    assert!(entries.iter().any(|e| Path::new(e) == node_bin));
    let java = apps.join("java").join("21.0");
    assert!(entries.iter().any(|e| Path::new(e) == java.join("bin")));
    let php = apps.join("php").join("8.4");
    assert_eq!(env.get("CP_PHP"), php.to_str());
    assert_eq!(env.get("JAVA_HOME"), java.to_str());
    let fwd = php.display().to_string().replace('\\', "/");
    assert_eq!(env.get("CP_PHP_FWD"), Some(fwd.as_str()));
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

/// Backlog (a): on macOS and Linux `python` runs the kit's `python3`,
/// through a script in a folder that only the Unix PATH has, right after
/// the kit's Python; Windows has no such folder.
#[test]
fn python_on_the_unix_path() {
    use crate::kit::Target;

    let tmp = TempDir::new("env-python-shim");
    let kit = Kit::at_target(tmp.0.join("Kit"), Target::LINUX_X64);
    let python = crate::fsx::join_rel(kit.apps(), "python/bin/python3");
    fs::create_dir_all(python.parent().unwrap()).unwrap();
    fs::write(&python, b"").unwrap();
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    let shims = kit.kit_files().join("shims");
    let entries = env.path_entries();
    let at = |p: &Path| entries.iter().position(|e| Path::new(e) == p);
    let bin = crate::fsx::join_rel(kit.apps(), "python/bin");
    assert!(at(&bin).is_some(), "{entries:?}");
    assert_eq!(at(&shims), at(&bin).map(|i| i + 1), "{entries:?}");
    env.create_dirs().unwrap();
    let script = fs::read_to_string(shims.join("python")).unwrap();
    assert_eq!(script, PYTHON_SHIM);
    assert!(script.starts_with("#!/bin/sh\n") && !script.contains('\r'));
    assert!(script.contains("exec python3 \"$@\""));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(shims.join("python"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o755);
    }
    // Unchanged: left as it is.
    env.create_dirs().unwrap();
    // The same script without its exec bits (a copy that drops modes):
    // they come back.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file = shims.join("python");
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
        env.create_dirs().unwrap();
        let mode = fs::metadata(&file).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o755);
    }

    // Without the kit's Python: no shim on the PATH.
    let bare = Kit::at_target(tmp.0.join("Bare"), Target::LINUX_X64);
    let env = KitEnv::build(&bare, &Settings::default(), "en", &host());
    let bare_shims = bare.kit_files().join("shims");
    assert!(!env
        .path_entries()
        .iter()
        .any(|e| Path::new(e) == bare_shims));

    // Windows: python.exe itself, never the shim folder.
    let (_tmp, win) = fake_kit("env-python-win", &["python/python.exe"]);
    let win = Kit::at_target(win.root().to_path_buf(), Target::WIN32_X64);
    let env = KitEnv::build(&win, &Settings::default(), "en", &host());
    let win_shims = win.kit_files().join("shims");
    assert!(!env.path_entries().iter().any(|e| Path::new(e) == win_shims));
}

/// Files under `data/apps/<target>` of `kit` (forward slashes).
fn touch(kit: &Kit, rels: &[&str]) {
    for rel in rels {
        let file = crate::fsx::join_rel(kit.apps(), rel);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, b"").unwrap();
    }
}

/// The shims install/lang/cc.rs writes in `cc/bin` on macOS and Linux.
const CC_SHIMS: [&str; 9] = [
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

/// D-112: with C and C++ and the kit's Zig on macOS and Linux, CC, CXX,
/// AR and RANLIB are the shims of `cc/bin`, on the PATH. Without Zig the
/// shims would fail and hide the system's compiler: neither the PATH nor
/// the variables. The LLVM of an older kit: on the PATH, no variables (they
/// would name programs it lacks). Nothing on Windows.
#[test]
fn cc_variables_on_unix_with_cc_and_zig() {
    use crate::kit::Target;

    let tmp = TempDir::new("env-cc");
    let zig = crate::setup::catalog::Catalog::get().find("zig").unwrap();
    for target in [
        Target::LINUX_X64,
        Target::LINUX_ARM64,
        Target::DARWIN_ARM64,
        Target::DARWIN_X64,
    ] {
        let bin_of = |kit: &Kit| crate::fsx::join_rel(kit.apps(), "cc/bin");
        let on_path = |env: &KitEnv, kit: &Kit| {
            env.path_entries()
                .iter()
                .any(|e| Path::new(e) == bin_of(kit))
        };
        let no_vars = |env: &KitEnv| CC_VARS.iter().all(|(n, _)| env.get(n).is_none());

        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        let bare = KitEnv::build(&kit, &Settings::default(), "en", &host());
        assert!(no_vars(&bare) && !on_path(&bare, &kit), "{target}");
        // The shims without Zig.
        touch(&kit, &CC_SHIMS);
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        assert!(no_vars(&env), "{target} without zig");
        assert!(!on_path(&env, &kit), "{target} without zig");
        // With the kit's Zig.
        touch(&kit, &["zig/0.15/zig"]);
        zig.save_version(&kit, "0.15.2").unwrap();
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        let bin = bin_of(&kit);
        for (name, tool) in CC_VARS {
            assert_eq!(
                env.get(name).map(Path::new),
                Some(bin.join(tool).as_path()),
                "{target} {name}"
            );
        }
        assert!(on_path(&env, &kit), "{target}");

        // An older kit with the LLVM tarball: `clang`, no shims.
        let old = Kit::at_target(tmp.0.join(format!("old-{target}")), target);
        touch(&old, &["cc/bin/clang", "cc/bin/llvm-ar", "zig/0.15/zig"]);
        zig.save_version(&old, "0.15.2").unwrap();
        let env = KitEnv::build(&old, &Settings::default(), "en", &host());
        assert!(no_vars(&env), "{target} old LLVM");
        assert!(on_path(&env, &old), "{target} old LLVM");
    }
    let win = Kit::at_target(tmp.0.join("win"), Target::WIN32_X64);
    touch(&win, &["cc/bin/clang.exe", "zig/0.15/zig.exe"]);
    let env = KitEnv::build(&win, &Settings::default(), "en", &host());
    for (name, _) in CC_VARS {
        assert_eq!(env.get(name), None, "{name}");
    }
    let win_bin = crate::fsx::join_rel(win.apps(), "cc/bin");
    assert!(env.path_entries().iter().any(|e| Path::new(e) == win_bin));
}

/// Each variable of C and C++ names a program on disk: a `cc/bin` with
/// some shims missing gets only the others.
#[test]
fn cc_variables_only_for_programs_on_disk() {
    let bin = Path::new("K").join("cc").join("bin");
    let all = unix_cc_env(&bin, true, |_| true);
    assert!(all.on_path);
    assert_eq!(
        all.vars,
        CC_VARS
            .iter()
            .map(|(n, t)| (*n, bin.join(t)))
            .collect::<Vec<_>>()
    );
    let no_ranlib = unix_cc_env(&bin, true, |p| !p.ends_with("ranlib"));
    let names: Vec<&str> = no_ranlib.vars.iter().map(|(n, _)| *n).collect();
    assert_eq!(names, ["CC", "CXX", "AR"]);
    assert_eq!(unix_cc_env(&bin, false, |_| true), UnixCc::default());
    assert_eq!(unix_cc_env(&bin, true, |_| false), UnixCc::default());
}

/// On a Mac with the Command Line Tools rustc links with Apple's `cc`,
/// never the shim; nowhere else, and not without the tools.
#[test]
fn macos_rust_linker_with_command_line_tools() {
    use crate::kit::Target;

    for t in Target::ALL {
        assert_eq!(macos_rust_linker(t, false), None, "{t}");
        let got = macos_rust_linker(t, true);
        assert_eq!(got.is_some(), t.os.is_macos(), "{t}");
    }
    assert_eq!(
        macos_rust_linker(Target::DARWIN_ARM64, true),
        Some((
            "CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER".to_owned(),
            "/usr/bin/cc"
        ))
    );
    assert_eq!(
        macos_rust_linker(Target::DARWIN_X64, true).map(|(n, _)| n),
        Some("CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER".to_owned())
    );
    for dir in XCODE_DEVELOPER_DIRS {
        assert!(dir.starts_with('/') && !dir.ends_with('/'), "{dir}");
    }
}

/// D-112: the linker of Rust's gnullvm host on Windows ARM64 only, the
/// aarch64 wrapper of llvm-mingw first.
#[test]
fn rust_linker_on_windows_arm64() {
    use crate::kit::Target;

    let bin = Path::new("K").join("cc").join("bin");
    let all = |_: &Path| true;
    assert_eq!(
        arm64_rust_linker(&bin, all),
        Some(bin.join("aarch64-w64-mingw32-clang.exe"))
    );
    let clang_only =
        |p: &Path| p.ends_with("clang.exe") && !p.ends_with("aarch64-w64-mingw32-clang.exe");
    assert_eq!(
        arm64_rust_linker(&bin, clang_only),
        Some(bin.join("clang.exe"))
    );
    assert_eq!(arm64_rust_linker(&bin, |_| false), None);

    let var = "CARGO_TARGET_AARCH64_PC_WINDOWS_GNULLVM_LINKER";
    let tmp = TempDir::new("env-rust-arm64");
    let parts = ["cc/bin/clang.exe", "cc/bin/aarch64-w64-mingw32-clang.exe"];
    let arm = Kit::at_target(tmp.0.join("arm"), Target::WIN32_ARM64);
    let env = KitEnv::build(&arm, &Settings::default(), "en", &host());
    assert_eq!(env.get(var), None, "no cc, no linker");
    touch(&arm, &parts);
    let env = KitEnv::build(&arm, &Settings::default(), "en", &host());
    assert_eq!(
        env.get(var).map(Path::new),
        Some(crate::fsx::join_rel(arm.apps(), parts[1]).as_path())
    );
    // Windows x64 unchanged.
    let x64 = Kit::at_target(tmp.0.join("x64"), Target::WIN32_X64);
    touch(&x64, &parts);
    let env = KitEnv::build(&x64, &Settings::default(), "en", &host());
    assert_eq!(env.get(var), None);
}

/// D-112 hook: a Git with `libexec/git-core` (dugite-native) on macOS and
/// Linux gets dugite's variables; Linux its CA bundle when it has one; never
/// PREFIX; nothing on Windows.
#[test]
fn git_variables_for_a_dugite_layout() {
    use crate::kit::Os;

    let git = Path::new("/kit/data/apps/linux-x64/git");
    let all = |_: &Path| true;
    let names = |os| -> Vec<&str> {
        git_unix_env(os, git, all)
            .into_iter()
            .map(|(n, _)| n)
            .collect()
    };
    assert_eq!(
        names(Os::Linux),
        [
            "GIT_EXEC_PATH",
            "GIT_TEMPLATE_DIR",
            "GIT_CONFIG_SYSTEM",
            "GIT_SSL_CAINFO"
        ]
    );
    assert_eq!(
        names(Os::Darwin),
        ["GIT_EXEC_PATH", "GIT_TEMPLATE_DIR", "GIT_CONFIG_SYSTEM"]
    );
    assert!(names(Os::Win32).is_empty());
    let env: Vec<(&str, PathBuf)> = git_unix_env(Os::Linux, git, all);
    for (name, rel) in [
        ("GIT_EXEC_PATH", "libexec/git-core"),
        ("GIT_TEMPLATE_DIR", "share/git-core/templates"),
        ("GIT_CONFIG_SYSTEM", "etc/gitconfig"),
        ("GIT_SSL_CAINFO", "ssl/cacert.pem"),
    ] {
        let value = &env.iter().find(|(n, _)| *n == name).unwrap().1;
        assert_eq!(value, &crate::fsx::join_rel(git, rel), "{name}");
    }
    // Without libexec (a Git of another layout) or without the CA bundle.
    assert!(git_unix_env(Os::Linux, git, |_| false).is_empty());
    let no_ca = |p: &Path| !p.ends_with("cacert.pem");
    assert_eq!(git_unix_env(Os::Linux, git, no_ca).len(), 3);

    // Through the kit: a Linux kit with `git/libexec/git-core`.
    use crate::kit::Target;
    let tmp = TempDir::new("env-git-dugite");
    let kit = Kit::at_target(tmp.0.join("Kit"), Target::LINUX_X64);
    touch(&kit, &["git/bin/git"]);
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    assert_eq!(env.get("GIT_EXEC_PATH"), None);
    fs::create_dir_all(crate::fsx::join_rel(kit.apps(), "git/libexec/git-core")).unwrap();
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    assert_eq!(
        env.get("GIT_EXEC_PATH").map(Path::new),
        Some(crate::fsx::join_rel(kit.apps(), "git/libexec/git-core").as_path())
    );
    assert_eq!(env.get("PREFIX"), None);
}

/// D-112 hook: GEM_HOME of the default Ruby line on macOS and Linux, its
/// `bin` on the PATH after Ruby's; Windows unchanged.
#[test]
fn gem_home_per_system_and_line() {
    use crate::kit::Target;

    assert_eq!(
        gem_home(Path::new("D"), Target::LINUX_X64, "3.4"),
        Path::new("D")
            .join("ruby-gems")
            .join("linux-x64")
            .join("3.4")
    );
    let tmp = TempDir::new("env-gem-home");
    let ruby = crate::setup::catalog::Catalog::get().find("ruby").unwrap();
    for target in [Target::LINUX_X64, Target::DARWIN_ARM64, Target::WIN32_X64] {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        let bare = KitEnv::build(&kit, &Settings::default(), "en", &host());
        assert_eq!(bare.get("GEM_HOME"), None, "{target}");
        let key = if target.is_windows() {
            "ruby/3.4/bin/ruby.exe"
        } else {
            "ruby/3.4/bin/ruby"
        };
        touch(&kit, &[key]);
        ruby.save_version(&kit, "3.4.7").unwrap();
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        if target.is_windows() {
            assert_eq!(env.get("GEM_HOME"), None);
            continue;
        }
        let gems = gem_home(&kit.data(), target, "3.4");
        assert_eq!(
            env.get("GEM_HOME").map(Path::new),
            Some(gems.as_path()),
            "{target}"
        );
        let entries = env.path_entries();
        let at = |p: &Path| entries.iter().position(|e| Path::new(e) == p);
        let ruby_bin = crate::fsx::join_rel(kit.apps(), "ruby/3.4/bin");
        assert!(at(&ruby_bin).is_some(), "{target} {entries:?}");
        assert_eq!(
            at(&gems.join("bin")),
            at(&ruby_bin).map(|i| i + 1),
            "{target}"
        );
        assert_eq!(
            env.get("GEMRC").map(Path::new),
            Some(gemrc_path(&kit.data()).as_path())
        );
    }
}

/// Two Ruby lines on disk: the kit uses the newest (3.4) while the default
/// is 3.3, so GEM_HOME and its `bin` are 3.4's, next to 3.4's Ruby, never
/// a mix of the two lines.
#[test]
fn gem_home_follows_the_line_the_kit_uses() {
    use crate::kit::Target;

    let tmp = TempDir::new("env-gem-lines");
    let ruby = crate::setup::catalog::Catalog::get().find("ruby").unwrap();
    for target in [Target::LINUX_ARM64, Target::DARWIN_X64] {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        touch(&kit, &["ruby/3.3/bin/ruby", "ruby/3.4/bin/ruby"]);
        ruby.save_version(&kit, "3.3.9").unwrap();
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        let gems = gem_home(&kit.data(), target, "3.4");
        assert_eq!(
            env.get("GEM_HOME").map(Path::new),
            Some(gems.as_path()),
            "{target}"
        );
        let entries = env.path_entries();
        let at = |p: &Path| entries.iter().position(|e| Path::new(e) == p);
        let ruby_bin = crate::fsx::join_rel(kit.apps(), "ruby/3.4/bin");
        assert_eq!(
            at(&gems.join("bin")),
            at(&ruby_bin).map(|i| i + 1),
            "{target} {entries:?}"
        );
        let old = crate::fsx::join_rel(kit.apps(), "ruby/3.3");
        let old_gems = gem_home(&kit.data(), target, "3.3");
        assert!(
            entries
                .iter()
                .all(|e| !Path::new(e).starts_with(&old) && !Path::new(e).starts_with(&old_gems)),
            "{target} {entries:?}"
        );
    }
}

/// The GEM_HOME of a Ruby folder, for the swap of a line: by the folder's
/// name, never on Windows or for a folder not named after a line.
#[test]
fn gem_home_of_a_ruby_folder() {
    use crate::kit::Target;

    let data = Path::new("D");
    let dir = Path::new("D").join("apps").join("linux-x64").join("ruby");
    assert_eq!(
        gem_home_for_dir(data, Target::LINUX_X64, &dir.join("3.3")),
        Some(gem_home(data, Target::LINUX_X64, "3.3"))
    );
    assert_eq!(gem_home_for_dir(data, Target::LINUX_X64, &dir), None);
    assert_eq!(
        gem_home_for_dir(data, Target::WIN32_X64, &dir.join("3.3")),
        None
    );
}

/// The kit's gemrc turns on env shebangs, is written once and then left
/// to the user; `create_dirs` writes it where GEMRC points.
#[test]
fn gemrc_written_once_with_env_shebang() {
    let tmp = TempDir::new("env-gemrc");
    let rc = gemrc_path(&tmp.0);
    assert_eq!(rc, tmp.0.join("ruby-gems").join("gemrc"));
    assert!(GEMRC.lines().any(|l| l == "gem: --env-shebang"));
    assert!(GEMRC
        .lines()
        .all(|l| l.starts_with('#') || l.starts_with("gem: ")));
    write_gemrc(&rc).unwrap();
    assert_eq!(fs::read_to_string(&rc).unwrap(), GEMRC);
    fs::write(&rc, "gem: --no-document\n").unwrap();
    write_gemrc(&rc).unwrap();
    assert_eq!(fs::read_to_string(&rc).unwrap(), "gem: --no-document\n");

    let mut env = KitEnv::default();
    let other = tmp.0.join("other").join("gemrc");
    env.set_path("GEMRC", &other);
    env.create_dirs().unwrap();
    assert_eq!(fs::read_to_string(&other).unwrap(), GEMRC);
}

/// D-112: ICU's library found by its versioned name in a library folder.
#[test]
fn libicu_lookup() {
    let tmp = TempDir::new("env-libicu");
    let (with, without) = (tmp.0.join("with"), tmp.0.join("without"));
    fs::create_dir_all(&with).unwrap();
    fs::create_dir_all(&without).unwrap();
    fs::write(without.join("libicuuc.a"), b"").unwrap();
    fs::write(without.join("libc.so.6"), b"").unwrap();
    assert!(!has_libicu(std::slice::from_ref(&without)));
    assert!(!has_libicu(&[tmp.0.join("missing")]));
    fs::write(with.join("libicuuc.so.74"), b"").unwrap();
    assert!(has_libicu(&[without.clone(), with]));
    let dirs = linux_lib_dirs("aarch64");
    assert_eq!(dirs[0], Path::new("/usr/lib/aarch64-linux-gnu"));
    assert!(dirs.contains(&PathBuf::from("/usr/lib64")));
}

/// D-112: the invariant mode on a Linux kit when the PC has no ICU, with
/// or without .NET on disk (the final check of `prepare` uses the
/// environment built before .NET was installed); never on Windows or macOS
/// (where .NET carries or finds ICU).
#[test]
fn dotnet_invariant_off_linux_never() {
    use crate::kit::Target;

    let tmp = TempDir::new("env-dotnet");
    for (target, key) in [
        (Target::WIN32_X64, "dotnet/dotnet.exe"),
        (Target::DARWIN_ARM64, "dotnet/dotnet"),
    ] {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        touch(&kit, &[key]);
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        assert_eq!(
            env.get("DOTNET_SYSTEM_GLOBALIZATION_INVARIANT"),
            None,
            "{target}"
        );
    }
    // Linux, before and after .NET: set exactly when this PC has no ICU.
    let kit = Kit::at_target(tmp.0.join("linux"), Target::LINUX_X64);
    for installed in [false, true] {
        if installed {
            touch(&kit, &["dotnet/dotnet"]);
        }
        let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
        assert_eq!(
            env.get("DOTNET_SYSTEM_GLOBALIZATION_INVARIANT"),
            (!libicu_present()).then_some("1"),
            "installed {installed}"
        );
    }
}

/// D-125: a conda environment puts its `kit-bin` on the PATH, the
/// launchers of the component's programs, never its `bin` with those of
/// its dependencies; `bin` while it has no `kit-bin` (installed before).
#[test]
fn conda_environments_give_only_their_programs() {
    use crate::kit::Target;

    let tmp = TempDir::new("env-conda-launchers");
    let kit = Kit::at_target(tmp.0.join("Kit"), Target::LINUX_X64);
    touch(
        &kit,
        &[
            "ruby/4.0/bin/ruby",
            "ruby/4.0/kit-bin/ruby",
            "mongodb/8.0/bin/mongod",
            "mongodb/8.0/kit-bin/mongod",
            "r/4.6/bin/R",
        ],
    );
    let catalog = crate::setup::catalog::Catalog::get();
    for (id, version) in [("ruby", "4.0.7"), ("mongodb", "8.0.23"), ("r", "4.6.1")] {
        catalog
            .find(id)
            .unwrap()
            .save_version(&kit, version)
            .unwrap();
    }
    let env = KitEnv::build(&kit, &Settings::default(), "en", &host());
    let entries = env.path_entries();
    let has = |rel: &str| {
        let p = crate::fsx::join_rel(kit.apps(), rel);
        entries.iter().any(|e| Path::new(e) == p)
    };
    assert!(
        has("ruby/4.0/kit-bin") && !has("ruby/4.0/bin"),
        "{entries:?}"
    );
    assert!(
        has("mongodb/8.0/kit-bin") && !has("mongodb/8.0/bin"),
        "{entries:?}"
    );
    assert!(has("r/4.6/bin"), "R without launchers yet: {entries:?}");
}

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
