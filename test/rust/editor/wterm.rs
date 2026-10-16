use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn merge_keeps_user_parts() {
    let tmp = TempDir::new("wt");
    let kit = Kit::at(tmp.0.join("k & (1)"), Arch::X64);
    kit.prepare().unwrap();
    let pwsh = pwsh(&kit);
    fs::create_dir_all(pwsh.parent().unwrap()).unwrap();
    fs::write(&pwsh, b"").unwrap();
    let existing: Map<String, Value> = serde_json::from_str(&format!(
        r##"{{"profiles":{{"list":[{{"guid":"{PROFILE_PWSH}","name":"old"}},{{"guid":"{{x}}","name":"Mine"}},{{"name":"Windows PowerShell"}}]}},"schemes":[{{"name":"{SCHEME_DARK}","background":"#000"}},{{"name":"Mine"}}],"copyOnSelect":true}}"##
    ))
    .unwrap();
    let m = merge(&existing, &kit);
    let list = m["profiles"]["list"].as_array().unwrap();
    assert_eq!(list[0]["name"], "Atollo");
    let line = list[0]["commandline"].as_str().unwrap();
    assert!(line.contains("profile.ps1"), "{line}");
    assert!(line.starts_with('"'), "path with spaces is quoted: {line}");
    assert!(list.iter().any(|p| p["name"] == "Mine"));
    assert_eq!(list.iter().filter(|p| p["guid"] == PROFILE_PWSH).count(), 1);
    assert_eq!(
        list.iter()
            .find(|p| p["name"] == "Windows PowerShell")
            .unwrap()["hidden"],
        true
    );
    let schemes = m["schemes"].as_array().unwrap();
    assert_eq!(
        schemes.iter().filter(|s| s["name"] == SCHEME_DARK).count(),
        1
    );
    assert_eq!(
        schemes.iter().find(|s| s["name"] == SCHEME_DARK).unwrap()["background"],
        "#181818"
    );
    assert!(schemes.iter().any(|s| s["name"] == "Mine"));
    assert_eq!(m["copyOnSelect"], true);
    assert_eq!(m["defaultProfile"], PROFILE_PWSH);
    assert_eq!(
        m["profiles"]["defaults"]["compatibility.reloadEnvironmentVariables"],
        false
    );
    assert_eq!(merge(&m, &kit), m, "stable");
}

#[test]
fn settings_of_the_user_are_never_lost() {
    let tmp = TempDir::new("wt-settings");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let path = settings_path(&kit);
    // missing: created
    write_settings(&kit).unwrap();
    assert!(fs::read_to_string(&path).unwrap().contains("Atollo"));
    // not readable as UTF-8 (UTF-16 from Notepad) or not plain JSON: untouched
    let utf16: Vec<u8> = "\u{feff}{\"a\":1}"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    for bytes in [utf16, b"// mine\n{\"a\":1}".to_vec()] {
        fs::write(&path, &bytes).unwrap();
        assert!(write_settings(&kit).is_err());
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn semicolons_are_escaped() {
    assert_eq!(wt_arg(r"D:\a;b\c"), r"D:\a\;b\c");
    assert_eq!(wt_arg(r"D:\a b"), r"D:\a b");
}

#[test]
fn build_of_windows_is_read() {
    assert!(windows_build().is_some_and(|b| b >= 10240));
}

/// The pipe's DACL has one entry, for its owner (not Everyone's read).
#[test]
fn pipe_is_owner_only() {
    use windows::Win32::Security::Authorization::{
        ConvertSecurityDescriptorToStringSecurityDescriptorW, GetSecurityInfo, SE_KERNEL_OBJECT,
    };
    use windows::Win32::Security::DACL_SECURITY_INFORMATION;
    let pipe = create_pipe(&format!("Atollo-Console-acl-{}", std::process::id())).unwrap();
    let mut sd = PSECURITY_DESCRIPTOR::default();
    let mut text = windows::core::PWSTR::null();
    // SAFETY: valid pipe handle; both outputs LocalAlloc'd, freed below.
    let sddl = unsafe {
        GetSecurityInfo(
            pipe.0,
            SE_KERNEL_OBJECT,
            DACL_SECURITY_INFORMATION,
            None,
            None,
            None,
            None,
            Some(&mut sd),
        )
        .ok()
        .unwrap();
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut text,
            None,
        )
        .unwrap();
        let s = text.to_string().unwrap();
        LocalFree(Some(HLOCAL(text.0.cast())));
        LocalFree(Some(HLOCAL(sd.0)));
        s
    };
    assert!(sddl.starts_with("D:P"), "{sddl}");
    assert_eq!(sddl.matches("(A;").count(), 1, "{sddl}");
    assert!(!sddl.contains(";WD)") && !sddl.contains(";AN)"), "{sddl}");
}

/// The handshake with a real client: the kit's environment arrives, and
/// a client that is not the kit's PowerShell is told apart by its id.
#[test]
fn pipe_sends_the_environment() {
    let name = format!("Atollo-Console-test-{}", std::process::id());
    let pipe = create_pipe(&name).unwrap();
    let path = format!(r"\\.\pipe\{name}");
    let client = std::thread::spawn(move || {
        use std::io::Read;
        let mut f = fs::OpenOptions::new().read(true).open(path).unwrap();
        let mut text = String::new();
        f.read_to_string(&mut text).unwrap();
        text
    });
    // In the test the client is this process: what the timeout gives.
    assert!(accept(&pipe, &name).is_err());
    let mut env = KitEnv::default();
    env.set("ATOLLO_PROBE", "a;b \"c\"");
    send(&pipe, &env).unwrap();
    drop(pipe);
    let got: Map<String, Value> = serde_json::from_str(&client.join().unwrap()).unwrap();
    assert_eq!(got["ATOLLO_PROBE"], "a;b \"c\"");
}

/// Real Windows Terminal windows (D-68): the second goes to the running
/// terminal, and its tab still gets the kit's environment, runs in a folder with a space and a `;`, and the app
/// sees it end. Needs a kit with the terminal, given by ATOLLO_TEST_KIT.
#[test]
fn real_terminal_gets_the_kit_environment() {
    let Ok(root) = std::env::var("ATOLLO_TEST_KIT") else {
        eprintln!("ATOLLO_TEST_KIT not set: skipped");
        return;
    };
    let kit = Kit::at(PathBuf::from(root), Arch::detect());
    kit.prepare().unwrap();
    crate::setup::kit_files::write_all(&kit).unwrap();
    assert!(usable(&kit), "terminal and PowerShell installed");
    let mut env = KitEnv::build(
        &kit,
        &crate::settings::Settings::default(),
        "en",
        &crate::env::Host::capture(),
    );
    env.set("ATOLLO_PROBE", "kit-env");
    let dir = kit.data().join(r"tmp\wterm test;1");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let script = dir.join("probe.cmd");
    fs::write(
        &script,
        "@echo %ATOLLO_PROBE%> ran.txt
",
    )
    .unwrap();
    // A shell first: the probe's window goes to the Windows Terminal
    // already running, which has its own environment.
    let shell = open(&kit, &env, &dir, None).unwrap();
    let console = open(&kit, &env, &dir, Some(&script)).unwrap();
    let start = std::time::Instant::now();
    let waiter = std::thread::spawn(move || console.wait());
    while !waiter.is_finished() {
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "console still open"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    let ran = fs::read_to_string(dir.join("ran.txt")).unwrap();
    assert_eq!(ran.trim(), "kit-env");
    crate::os::sessions::stop(&[shell.id()]);
    shell.wait();
    // The shell's tab stays on "process exited": the test's terminal
    // would keep cargo's output pipe open.
    let terminal: Vec<u32> = crate::os::processes::running_under(&exe(&kit).with_file_name(""))
        .iter()
        .map(|p| p.pid)
        .collect();
    crate::os::sessions::stop(&terminal);
    let _ = fs::remove_dir_all(&dir);
}
