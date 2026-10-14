//! Editor windows: one per opened project, all on the same local server.
//!
//! Rules (DECISIONS.md D-06, D-10, D-14):
//! - labels `editor-<n>`: no capability, so no IPC to the app;
//! - navigation only to the server's origin; any other `http`/`https` link
//!   opens in the system browser, other schemes are blocked;
//! - `window.open` and "New Window" on the server's origin open another
//!   editor window built here, with the same rules;
//! - WebView2 data in `data\webview2` (shared with the app window);
//! - browser accelerator keys and context menu off, so Ctrl+W, Ctrl+N,
//!   Ctrl+Tab, F11 reach the editor.

use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result};
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Runtime, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings3;
use windows::core::Interface;

use crate::jobs;

pub const LABEL_PREFIX: &str = "editor-";

static NEXT: AtomicU32 = AtomicU32::new(1);

/// Only the server, exactly: scheme, host and port.
pub fn same_origin(url: &Url, port: u16) -> bool {
    url.scheme() == "http" && url.host_str() == Some("127.0.0.1") && url.port() == Some(port)
}

/// The editor stays on its project: a page of the server is allowed only
/// with `folder=` the project's folder. Another folder, a workspace file or
/// an empty window (`ew=`) is refused, also in a new window.
pub fn in_project(url: &Url, folder: &Path) -> bool {
    let want = format!("/{}", folder.display().to_string().replace('\\', "/"));
    let norm = |s: &str| s.trim_end_matches('/').to_lowercase();
    let mut found = false;
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "folder" if norm(&value) == norm(&want) => found = true,
            "folder" | "workspace" | "ew" => return false,
            _ => {}
        }
    }
    found
}

/// What to do with a link the editor wants to leave for.
#[derive(Debug, PartialEq, Eq)]
pub enum Leave {
    Browser,
    Block,
}

pub fn classify_external(url: &Url) -> Leave {
    match url.scheme() {
        "http" | "https" if url.host_str().is_some() => Leave::Browser,
        _ => Leave::Block,
    }
}

/// Opens `url` in the default browser, outside the app's job.
pub fn open_in_browser(url: &Url) -> Result<()> {
    if classify_external(url) != Leave::Browser {
        anyhow::bail!("blocked link scheme: {}", url.scheme());
    }
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    let rundll = Path::new(&system_root).join(r"System32\rundll32.exe");
    jobs::spawn_detached(
        Command::new(rundll).args(["url.dll,FileProtocolHandler", url.as_str()]),
        0,
    )
    .map(|_| ())
}

/// An editor window on the project in `folder`, which it cannot leave.
pub fn open<R: Runtime>(
    app: &AppHandle<R>,
    url: &str,
    port: u16,
    title: &str,
    folder: &Path,
    data_dir: &Path,
) -> Result<()> {
    let target: Url = url.parse().context("editor URL")?;
    let label = format!("{LABEL_PREFIX}{}", NEXT.fetch_add(1, Ordering::Relaxed));
    let handle = app.clone();
    let data = data_dir.to_path_buf();
    let (project, project_new) = (folder.to_path_buf(), folder.to_path_buf());
    let name = title.to_owned();
    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(target))
        .title(format!("{title} - Atollo"))
        .inner_size(1280.0, 820.0)
        .min_inner_size(640.0, 400.0)
        .data_directory(data_dir.to_path_buf())
        .zoom_hotkeys_enabled(false)
        .on_navigation(move |u| {
            if same_origin(u, port) {
                let ok = in_project(u, &project);
                if !ok {
                    crate::log::warn("editor: blocked navigation out of the project");
                }
                return ok;
            }
            let ok = u.scheme() == "about";
            if !ok {
                if classify_external(u) == Leave::Browser {
                    let _ = open_in_browser(u);
                } else {
                    crate::log::warn(&format!("editor: blocked navigation to {}", u.scheme()));
                }
            }
            ok
        })
        .on_page_load(|w, payload| {
            let mut url = payload.url().clone();
            url.set_query(None);
            crate::log::info(&format!(
                "editor {}: {:?} {url}",
                w.label(),
                payload.event()
            ));
        })
        .on_new_window(move |u, _features| {
            if same_origin(&u, port) && !in_project(&u, &project_new) {
                crate::log::warn("editor: blocked new window out of the project");
            } else if same_origin(&u, port) {
                let (app, data, url) = (handle.clone(), data.clone(), u.to_string());
                let (folder, name) = (project_new.clone(), name.clone());
                // Not on the WebView2 callback: building a window there would deadlock.
                std::thread::spawn(move || {
                    if let Err(e) = open(&app, &url, port, &name, &folder, &data) {
                        crate::log::error(&format!("editor: new window: {e:#}"));
                    }
                });
            } else if classify_external(&u) == Leave::Browser {
                let _ = open_in_browser(&u);
            } else {
                crate::log::warn(&format!("editor: blocked new window to {}", u.scheme()));
            }
            NewWindowResponse::Deny
        })
        .build()
        .context("editor window")?;
    quiet_webview(&window, "editor");
    Ok(())
}

/// Browser accelerator keys, context menu and status bar off, so Ctrl+W,
/// Ctrl+N, Ctrl+F, F5... reach the page. `what` names the window in the log.
pub fn quiet_webview<R: Runtime>(window: &WebviewWindow<R>, what: &'static str) {
    let applied = window.with_webview(move |webview| {
        let result = (|| -> windows::core::Result<()> {
            // SAFETY: COM calls on the live controller of this webview.
            unsafe {
                let settings = webview
                    .controller()
                    .CoreWebView2()?
                    .Settings()?
                    .cast::<ICoreWebView2Settings3>()?;
                settings.SetAreBrowserAcceleratorKeysEnabled(false)?;
                settings.SetAreDefaultContextMenusEnabled(false)?;
                settings.SetIsStatusBarEnabled(false)?;
            }
            Ok(())
        })();
        if let Err(e) = result {
            crate::log::warn(&format!("{what}: webview settings: {e}"));
        }
    });
    if let Err(e) = applied {
        crate::log::warn(&format!("{what}: webview settings: {e}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_stays_in_its_project() {
        let u = |s: &str| s.parse::<Url>().unwrap();
        let dir = Path::new(r"E:\Atollo (1)\projects\app & co");
        let page = crate::editor::url(7003, "tok", Some(dir));
        assert!(in_project(&u(&page), dir));
        // after the server's redirect: the token goes, the folder stays
        assert!(in_project(
            &u("http://127.0.0.1:7003/?folder=/e:/Dev%20Kit%20(1)/projects/app%20%26%20co/"),
            dir
        ));
        for out in [
            "http://127.0.0.1:7003/",
            "http://127.0.0.1:7003/?ew=true",
            "http://127.0.0.1:7003/?folder=/E:/Dev%20Kit%20(1)/projects",
            "http://127.0.0.1:7003/?folder=/E:/Dev%20Kit%20(1)/projects/app%20%26%20co/sub",
            "http://127.0.0.1:7003/?folder=/C:/Users",
            "http://127.0.0.1:7003/?workspace=/E:/x.code-workspace",
        ] {
            assert!(!in_project(&u(out), dir), "{out}");
        }
        let both = format!("{page}&ew=true");
        assert!(!in_project(&u(&both), dir));
    }

    #[test]
    fn origin_and_links() {
        let u = |s: &str| s.parse::<Url>().unwrap();
        assert!(same_origin(&u("http://127.0.0.1:7003/x?y"), 7003));
        assert!(!same_origin(&u("http://127.0.0.1:7004/"), 7003));
        assert!(!same_origin(&u("http://localhost:7003/"), 7003));
        assert!(!same_origin(&u("https://127.0.0.1:7003/"), 7003));
        assert_eq!(
            classify_external(&u("https://github.com/x")),
            Leave::Browser
        );
        assert_eq!(classify_external(&u("http://example.com")), Leave::Browser);
        for bad in [
            "file:///C:/Windows/win.ini",
            "javascript:alert(1)",
            "vscodium://x",
            "ms-settings:display",
        ] {
            assert_eq!(classify_external(&u(bad)), Leave::Block, "{bad}");
        }
    }
}
