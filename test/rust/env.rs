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
        let file = kit.apps().join(rel);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, b"").unwrap();
    }
    (tmp, kit)
}

#[test]
#[cfg_attr(
    not(windows),
    ignore = "HOMEDRIVE and HOMEPATH exist only with a drive letter"
)]
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
    assert_eq!(env.get("DEVKIT"), Some(kit.root().to_str().unwrap()));
    assert_eq!(env.get("CP_LANG"), Some("it"));
    assert_eq!(env.get("DISABLE_UPDATES"), Some("1"));
    assert_eq!(
        env.get("HOMEPATH").map(|p| p.ends_with(r"\data\home")),
        Some(true)
    );
}

#[test]
#[cfg_attr(
    not(windows),
    ignore = "Windows paths with backslashes in the expected values"
)]
fn program_temp_decision_table() {
    use crate::os::volume::FsKind;
    let vol = |fs, drive| Some(VolumeInfo { fs, drive, free: 1 });
    let data = Path::new(r"E:\Kit\data");
    let host = Path::new(r"C:\Users\me\AppData\Local\Temp\Dev-Kit\abc");
    let in_kit = PathBuf::from(r"E:\Kit\data\tmp\system");
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
#[cfg_attr(not(windows), ignore = "Windows PATH: System32 and git/cmd")]
fn path_is_kit_then_system_only() {
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
    let n = entries.len();
    assert_eq!(
        &entries[n - 3..],
        [
            r"C:\Windows\System32",
            r"C:\Windows\System32\Wbem",
            r"C:\Windows\System32\WindowsPowerShell\v1.0"
        ]
    );
    for entry in &entries[..n - 3] {
        assert!(Path::new(entry).starts_with(kit.data()), "{entry}");
    }
    let apps = kit.apps();
    let pos = |p: PathBuf| {
        entries
            .iter()
            .position(|e| Path::new(e) == p)
            .unwrap_or_else(|| panic!("{} missing", p.display()))
    };
    assert!(pos(apps.join("claude")) < pos(apps.join("python")));
    assert!(pos(apps.join("node")) < pos(apps.join(r"go\bin")));
    assert!(pos(apps.join(r"go\bin")) < pos(apps.join(r"git\cmd")));
    assert!(
        entries
            .iter()
            .all(|e| !Path::new(e).starts_with(apps.join("java"))),
        "java not installed"
    );
    assert_eq!(
        env.get("CLAUDE_CODE_GIT_BASH_PATH")
            .map(|p| p.ends_with(r"git\bin\bash.exe")),
        Some(true)
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
    assert_eq!(env.get("DEVKIT_ALLOW_LAN"), Some("1"));
    assert_eq!(env.get("GEMINI_API_KEY"), Some("abc"));
    assert_eq!(env.get("DEVKIT_LANG"), Some("fr"));
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

#[test]
#[cfg_attr(
    not(windows),
    ignore = "Windows layout: backslash folders, .exe keys and ';' in PATH"
)]
fn languages_from_the_folder_of_their_version() {
    let (_tmp, kit) = fake_kit(
        "env-versions",
        &[
            r"node\22.11\node.exe",
            r"php\8.4\php.exe",
            r"java\21.0\bin\java.exe",
        ],
    );
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
    let path = env.get("PATH").unwrap();
    assert!(path
        .split(';')
        .any(|e| Path::new(e) == apps.join(r"node\22.11")));
    assert!(path
        .split(';')
        .any(|e| Path::new(e) == apps.join(r"java\21.0\bin")));
    assert_eq!(env.get("CP_PHP"), apps.join(r"php\8.4").to_str());
    assert_eq!(env.get("JAVA_HOME"), apps.join(r"java\21.0").to_str());
    let fwd = apps
        .join(r"php\8.4")
        .display()
        .to_string()
        .replace('\\', "/");
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
    assert_eq!(
        &entries[n - 5..],
        ["/usr/local/bin", "/usr/bin", "/bin", "/usr/sbin", "/sbin"]
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
