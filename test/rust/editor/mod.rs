use super::*;

#[test]
fn url_encodes_odd_folders() {
    let u = url(
        7001,
        "ab",
        Some(Path::new(r"E:\Dev Kit (1) & co\projects\già")),
    );
    assert_eq!(
        u,
        "http://127.0.0.1:7001/?tkn=ab&folder=/E:/Dev%20Kit%20%281%29%20%26%20co/projects/gi%C3%A0"
    );
    assert_eq!(url(7001, "ab", None), "http://127.0.0.1:7001/?tkn=ab");
}

#[test]
fn url_of_a_unix_folder_has_one_leading_slash() {
    let u = url(7001, "ab", Some(Path::new("/home/me/Dev Kit/projects/app")));
    assert_eq!(
        u,
        "http://127.0.0.1:7001/?tkn=ab&folder=/home/me/Dev%20Kit/projects/app"
    );
    let mac = url(7001, "ab", Some(Path::new("/Users/me/Kit/projects/già")));
    assert_eq!(
        mac,
        "http://127.0.0.1:7001/?tkn=ab&folder=/Users/me/Kit/projects/gi%C3%A0"
    );
    assert_eq!(folder_param(Path::new(r"C:\x\y")), "/C:/x/y");
    assert_eq!(folder_param(Path::new("/x/y")), "/x/y");
}

#[test]
fn tokens_are_random_hex() {
    let a = random_token().unwrap();
    assert_eq!(a.len(), 64);
    assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    assert_ne!(a, random_token().unwrap());
}

/// Processes whose program lives under `dir`.
#[cfg(windows)]
fn processes_under(dir: &Path) -> Vec<u32> {
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    let dir = dir.to_string_lossy().to_lowercase();
    let mut out = Vec::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap();
        let mut e = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut ok = Process32FirstW(snap, &mut e).is_ok();
        while ok {
            if let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, e.th32ProcessID) {
                let mut buf = [0u16; 1024];
                let mut len = buf.len() as u32;
                if QueryFullProcessImageNameW(
                    h,
                    PROCESS_NAME_WIN32,
                    windows::core::PWSTR(buf.as_mut_ptr()),
                    &mut len,
                )
                .is_ok()
                {
                    let path = String::from_utf16_lossy(&buf[..len as usize]).to_lowercase();
                    if path.starts_with(&dir) {
                        out.push(e.th32ProcessID);
                    }
                }
                let _ = windows::Win32::Foundation::CloseHandle(h);
            }
            ok = Process32NextW(snap, &mut e).is_ok();
        }
        let _ = windows::Win32::Foundation::CloseHandle(snap);
    }
    out
}

/// The real server (prompt, "editor.rs" test): needs a kit with the
/// editor installed, given by DEVKIT_TEST_KIT (CI runs it after prepare).
#[cfg(windows)]
#[test]
fn real_server_token_redirect_and_stop() {
    let Ok(root) = std::env::var("DEVKIT_TEST_KIT") else {
        eprintln!("DEVKIT_TEST_KIT not set: skipped");
        return;
    };
    let kit = Kit::at(PathBuf::from(root), crate::kit::Arch::detect());
    kit.prepare().unwrap();
    crate::setup::kit_files::write_all(&kit).unwrap();
    let env = KitEnv::build(
        &kit,
        &crate::settings::Settings::default(),
        "en",
        &crate::env::Host::capture(),
    );
    let editor = Editor::new(kit.clone(), env);
    let (port, token) = editor.ensure_running().unwrap();
    assert!(PORTS.contains(&port));
    let agent = local_agent();
    let no_redirect = ureq::Agent::config_builder()
        .proxy(None)
        .http_status_as_error(false)
        .max_redirects(0)
        .build()
        .new_agent();
    let denied = agent
        .get(&format!("http://127.0.0.1:{port}/"))
        .call()
        .unwrap();
    assert_eq!(denied.status().as_u16(), 403, "no token: refused");
    let ok = no_redirect
        .get(&url(port, &token, Some(&kit.projects())))
        .call()
        .unwrap();
    assert!(
        ok.status().is_redirection(),
        "token: redirect, got {}",
        ok.status()
    );
    assert!(
        ok.headers()
            .get_all("set-cookie")
            .iter()
            .any(|c| c.to_str().unwrap_or("").contains(&token)),
        "cookie set"
    );
    // The workbench page, with the cookie the redirect set.
    let page = no_redirect
        .get(&format!("http://127.0.0.1:{port}/"))
        .header("Cookie", format!("vscode-tkn={token}"))
        .call()
        .unwrap()
        .body_mut()
        .read_to_string()
        .unwrap()
        .replace("&quot;", "\"");
    assert!(
        page.contains("\"enableWorkspaceTrust\":true"),
        "workspace trust on (D-67)"
    );
    assert!(
        !fs::read_to_string(kit.logs().join("editor.log"))
            .unwrap()
            .contains(&token),
        "token masked in log"
    );
    assert!(!processes_under(&server_dir(&kit)).is_empty());
    editor.stop();
    drop(editor);
    let start = Instant::now();
    while !processes_under(&server_dir(&kit)).is_empty() {
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "server processes left"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(
        agent
            .get(&format!("http://127.0.0.1:{port}/"))
            .call()
            .is_err(),
        "port closed"
    );
}

#[test]
fn ports_in_range() {
    let p = free_ports().next().unwrap();
    assert!(PORTS.contains(&p));
}
