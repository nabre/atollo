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
//!   Ctrl+Tab, F11 reach the editor;
//! - no Windows frame: the editor's own title bar is the window's (D-69).
//!   Its drag region (`-webkit-app-region: drag`) moves the window through
//!   WebView2's non-client regions, which wry turns on; a script puts the
//!   three Windows buttons in the editor's empty `window-controls-container`.
//!   A click opens `/__atollo/window/<action>` on the server's origin as a
//!   new window: refused here and done on this window, still without IPC.

use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result};
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, Runtime, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings3;
use windows::core::Interface;

use crate::jobs;

pub const LABEL_PREFIX: &str = "editor-";

static NEXT: AtomicU32 = AtomicU32::new(1);

/// Where the title bar buttons send their clicks (never served).
const CONTROL_PATH: &str = "/__atollo/window/";

/// A button of the editor's title bar.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Control {
    Minimize,
    ToggleMaximize,
    Close,
}

/// The button a URL of the server's origin stands for, if any.
pub fn control(url: &Url, port: u16) -> Option<Control> {
    if !same_origin(url, port) {
        return None;
    }
    match url.path().strip_prefix(CONTROL_PATH)? {
        "minimize" => Some(Control::Minimize),
        "maximize" => Some(Control::ToggleMaximize),
        "close" => Some(Control::Close),
        _ => None,
    }
}

fn apply<R: Runtime>(window: &WebviewWindow<R>, what: Control) {
    let done = match what {
        Control::Minimize => window.minimize(),
        Control::ToggleMaximize => match window.is_maximized() {
            Ok(true) => window.unmaximize(),
            _ => window.maximize(),
        },
        // Same path as the Windows button: CloseRequested, then closed.
        Control::Close => window.close(),
    };
    if let Err(e) = done {
        crate::log::warn(&format!("editor {}: {what:?}: {e}", window.label()));
    }
}

/// Tells the title bar whether the window is maximized (restore icon).
pub fn sync_maximized<R: Runtime>(window: &WebviewWindow<R>) {
    let maximized = window.is_maximized().unwrap_or(false);
    let _ = window.eval(format!(
        "window.__atolloMaximized && window.__atolloMaximized({maximized})"
    ));
}

/// Defaults of the editor's title bar, as in the desktop VSCodium: the menus
/// in the bar (the web default is "compact", in the activity bar). They are
/// "application" settings, which `Machine\settings.json` cannot set (D-66):
/// they go in the page's `configurationDefaults`; the user's settings win.
const TITLE_BAR_DEFAULTS: &str = r#"{"window.menuBarVisibility":"classic"}"#;

/// Atollo's logo (the app icon, ui/src/assets) in the editor's title bar
/// and on the empty editor (D-72).
const LOGO_32: &[u8] = include_bytes!("../../ui/src/assets/logo-32.png");
const LOGO_256: &[u8] = include_bytes!("../../ui/src/assets/logo-256.png");

/// The editor's name where VSCodium shows its own: window title, About
/// (D-72). Merged by the workbench over its product, as `productConfiguration`.
const PRODUCT: &str = r#"{"nameShort":"Atollo","nameLong":"Atollo"}"#;

fn data_uri(png: &[u8]) -> String {
    use base64::Engine;
    format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    )
}

/// The script that sets the title bar defaults and fills the editor's title
/// bar with the Windows buttons, with the editor's own classes, so its
/// colors (codicons, red on close).
fn controls_script(lang: &str, port: u16, project: &str) -> String {
    let label =
        |it: &str| serde_json::to_string(&crate::i18n::t(lang, it, &[])).unwrap_or_default();
    let (logo32, logo256) = (data_uri(LOGO_32), data_uri(LOGO_256));
    let project = serde_json::to_string(project).unwrap_or_default();
    format!(
        r##"(() => {{
  if (window.top !== window) return;
  // Before the workbench reads its configuration: module scripts run after
  // the document turns "interactive".
  const defaults = {TITLE_BAR_DEFAULTS};
  const product = {PRODUCT};
  const configure = () => {{
    const el = document.getElementById("vscode-workbench-web-configuration");
    if (!el || el.dataset.atollo) return;
    el.dataset.atollo = "1";
    try {{
      const c = JSON.parse(el.getAttribute("data-settings"));
      c.configurationDefaults = {{ ...defaults, ...(c.configurationDefaults || {{}}) }};
      c.productConfiguration = {{ ...(c.productConfiguration || {{}}), ...product }};
      el.setAttribute("data-settings", JSON.stringify(c));
    }} catch (e) {{}}
  }};
  document.addEventListener("readystatechange", configure);
  const open = window.open.bind(window);
  // The server's origin, also on WebView2's error page (server down).
  const send = (what) => open("http://127.0.0.1:{port}{CONTROL_PATH}" + what, "_blank");
  const text = {{
    min: {min}, max: {max}, restore: {restore}, close: {close},
    ext: {ext}, restart: {restart}, later: {later}, tagline: {tagline},
  }};
  let maximized = false;
  const maxButtons = [];
  const paint = () => {{
    for (const b of maxButtons) {{
      if (b.dataset.glyph) b.textContent = maximized ? "" : "";
      else {{
        b.classList.toggle("codicon-chrome-maximize", !maximized);
        b.classList.toggle("codicon-chrome-restore", maximized);
      }}
      const t = maximized ? text.restore : text.max;
      b.title = t;
      b.setAttribute("aria-label", t);
    }}
  }};
  window.__atolloMaximized = (m) => {{ maximized = !!m; paint(); }};
  const add = (box, cls, t, what, glyph) => {{
    const b = document.createElement("div");
    b.className = cls;
    if (glyph) {{ b.dataset.glyph = "1"; b.textContent = glyph; }}
    b.title = t;
    b.setAttribute("role", "button");
    b.setAttribute("aria-label", t);
    b.addEventListener("click", (e) => {{ e.stopPropagation(); send(what); }});
    box.appendChild(b);
    if (what === "maximize") maxButtons.push(b);
    return b;
  }};
  // In the editor's own title bar, as the desktop VSCodium draws them.
  const fill = () => {{
    const box = document.querySelector(
      ".monaco-workbench .part.titlebar .titlebar-right .window-controls-container");
    if (!box || box.dataset.atollo) return;
    box.dataset.atollo = "1";
    const icon = "window-icon codicon ";
    add(box, icon + "window-minimize codicon-chrome-minimize", text.min, "minimize");
    add(box, icon + "window-max-restore codicon-chrome-maximize", text.max, "maximize");
    add(box, icon + "window-close codicon-chrome-close", text.close, "close");
    paint();
  }};
  // Without a visible title bar (error page, title bar turned off), a plain
  // one: the window has no Windows frame and must still move and close.
  let fallback = null;
  let missing = 0;
  const check = () => {{
    const bar = document.querySelector(".monaco-workbench .part.titlebar");
    const shown = (bar && bar.offsetHeight > 0) || document.fullscreenElement;
    missing = shown ? 0 : missing + 1;
    if (shown && fallback) {{
      maxButtons.splice(maxButtons.indexOf(fallback.max), 1);
      fallback.remove();
      fallback = null;
    }} else if (missing >= 3 && !fallback && document.body) {{
      fallback = document.createElement("div");
      fallback.id = "atollo-titlebar";
      const drag = document.createElement("div");
      drag.className = "drag";
      drag.textContent = document.title;
      fallback.appendChild(drag);
      add(fallback, "btn", text.min, "minimize", "");
      fallback.max = add(fallback, "btn", text.max, "maximize", "");
      add(fallback, "btn close", text.close, "close", "");
      document.body.appendChild(fallback);
      paint();
    }}
  }};
  // Welcome on the empty editor: VSCodium's watermark (logo, then its
  // shortcuts) with Atollo's logo, name, project and what the kit is.
  const welcome = () => {{
    for (const w of document.querySelectorAll(".editor-group-watermark")) {{
      if (w.querySelector(".atollo-welcome")) continue;
      const box = document.createElement("div");
      box.className = "atollo-welcome";
      const line = (cls, value) => {{
        const d = document.createElement("div");
        d.className = cls;
        d.textContent = value;
        box.appendChild(d);
      }};
      line("name", "Atollo");
      line("project", {project});
      line("tagline", text.tagline);
      const logo = w.querySelector(".letterpress");
      if (logo) logo.after(box); else w.prepend(box);
    }}
  }};
  // Extensions installed by Atollo while the editor is open: a notice in
  // the editor's own colors offers to restart them (a page reload: the
  // page cannot run the editor's commands, D-14). Names grow while open.
  const installed = new Set();
  let notice = null;
  window.__atolloExtensions = (names) => {{
    for (const n of names) installed.add(n);
    const list = [...installed].join(", ");
    if (notice) {{
      notice.msg.textContent = text.ext.replace("{{0}}", list);
      return;
    }}
    notice = document.createElement("div");
    notice.id = "atollo-notice";
    notice.setAttribute("role", "alert");
    notice.msg = document.createElement("div");
    notice.msg.className = "msg";
    notice.msg.textContent = text.ext.replace("{{0}}", list);
    const icon = document.createElement("span");
    icon.className = "codicon codicon-extensions";
    const head = document.createElement("div");
    head.className = "head";
    head.append(icon, notice.msg);
    const actions = document.createElement("div");
    actions.className = "actions";
    const button = (label, cls, act) => {{
      const b = document.createElement("button");
      b.className = cls;
      b.textContent = label;
      b.addEventListener("click", act);
      actions.appendChild(b);
    }};
    const close = () => {{ notice.remove(); notice = null; installed.clear(); }};
    button(text.later, "secondary", close);
    button(text.restart, "primary", () => location.reload());
    notice.append(head, actions);
    (document.querySelector(".monaco-workbench") || document.body).appendChild(notice);
  }};
  const start = () => {{
    const style = document.createElement("style");
    style.textContent = ".monaco-workbench.web .part.titlebar .titlebar-right "
      + ".window-controls-container{{width:138px}}"
      + "#atollo-titlebar{{position:fixed;top:0;left:0;right:0;height:30px;z-index:2147483647;"
      + "display:flex;font:12px 'Segoe UI',sans-serif;user-select:none;"
      + "background:var(--vscode-titleBar-activeBackground,#1f1f1f);"
      + "color:var(--vscode-titleBar-activeForeground,#ccc)}}"
      + "#atollo-titlebar .drag{{flex:1;padding-left:12px;line-height:30px;overflow:hidden;"
      + "white-space:nowrap;text-overflow:ellipsis;-webkit-app-region:drag}}"
      + "#atollo-titlebar .btn{{width:46px;display:flex;align-items:center;justify-content:center;"
      + "font:10px 'Segoe Fluent Icons','Segoe MDL2 Assets';-webkit-app-region:no-drag}}"
      + "#atollo-titlebar .btn:hover{{background:#ffffff1a}}"
      + "#atollo-titlebar .close:hover{{background:#e81123e6;color:#fff}}"
      + ".monaco-workbench .part.titlebar>.titlebar-container>.titlebar-left"
      + ">.window-appicon:not(.codicon){{background-image:url({logo32})!important}}"
      + ".monaco-workbench .part.editor>.content .editor-group-container"
      + " .editor-group-watermark .letterpress{{background-image:url({logo256})!important;"
      + "max-width:128px!important}}"
      + ".atollo-welcome{{text-align:center;margin:12px 0 24px;"
      + "color:var(--vscode-editorWatermark-foreground,var(--vscode-descriptionForeground))}}"
      + ".atollo-welcome .name{{font-size:26px;font-weight:600;"
      + "color:var(--vscode-foreground)}}"
      + ".atollo-welcome .project{{font-size:15px;margin-top:4px}}"
      + ".atollo-welcome .tagline{{font-size:13px;margin-top:8px;opacity:.85}}"
      + "#atollo-notice{{position:fixed;right:12px;bottom:34px;width:440px;z-index:3000;"
      + "box-sizing:border-box;padding:12px 14px;border-radius:6px;font-size:13px;"
      + "background:var(--vscode-notifications-background,#252526);"
      + "color:var(--vscode-notifications-foreground,#ccc);"
      + "border:1px solid var(--vscode-notificationToast-border,transparent);"
      + "box-shadow:0 0 8px 2px var(--vscode-widget-shadow,#0000005c)}}"
      + "#atollo-notice .head{{display:flex;gap:8px;align-items:flex-start;line-height:1.4}}"
      + "#atollo-notice .codicon{{color:var(--vscode-notificationsInfoIcon-foreground,#3794ff);"
      + "margin-top:2px}}"
      + "#atollo-notice .actions{{display:flex;justify-content:flex-end;gap:8px;margin-top:10px}}"
      + "#atollo-notice button{{border:none;border-radius:2px;padding:4px 10px;font:inherit;"
      + "cursor:pointer}}"
      + "#atollo-notice .primary{{background:var(--vscode-button-background,#0e639c);"
      + "color:var(--vscode-button-foreground,#fff)}}"
      + "#atollo-notice .primary:hover{{background:var(--vscode-button-hoverBackground,#1177bb)}}"
      + "#atollo-notice .secondary{{background:var(--vscode-button-secondaryBackground,#3a3d41);"
      + "color:var(--vscode-button-secondaryForeground,#fff)}}";
    document.head.appendChild(style);
    fill();
    welcome();
    new MutationObserver(() => {{ fill(); welcome(); }})
      .observe(document.body, {{ childList: true, subtree: true }});
    check();
    setInterval(check, 500);
  }};
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", start);
  else start();
}})();"##,
        min = label("Riduci a icona"),
        max = label("Ingrandisci"),
        restore = label("Ripristina"),
        close = label("Chiudi"),
        ext = label("Atollo ha installato nuove estensioni: {0}. Riavvia le estensioni perché partano tutte."),
        restart = label("Riavvia le estensioni"),
        later = label("Più tardi"),
        tagline = label("Ambiente di sviluppo portabile: linguaggi, agenti ed editor in una cartella."),
    )
}

/// The project folder of each open editor window, by label.
static OPEN: std::sync::Mutex<std::collections::BTreeMap<String, std::path::PathBuf>> =
    std::sync::Mutex::new(std::collections::BTreeMap::new());

/// The projects open in the editor (each once).
pub fn open_projects() -> Vec<std::path::PathBuf> {
    let open = OPEN.lock().unwrap_or_else(|e| e.into_inner());
    let mut dirs: Vec<std::path::PathBuf> = open.values().cloned().collect();
    dirs.sort();
    dirs.dedup();
    dirs
}

/// An editor window closed.
pub fn closed(label: &str) {
    OPEN.lock().unwrap_or_else(|e| e.into_inner()).remove(label);
}

/// What decides the extensions of a project: its languages, the tools it
/// requires and its own choice (D-71).
fn extension_inputs(
    kit: &crate::kit::Kit,
    dir: &Path,
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let choice = crate::project_settings::load(kit, &name).extensions;
    (
        crate::detect::languages(dir),
        crate::detect::requires(dir),
        choice.added,
        choice.removed,
    )
}

/// Every `WATCH_EVERY`, the projects open in the editor are read again: a
/// new language (the first `.py` written in the editor) or a changed choice
/// installs its extensions while the editor is open. Bounded reads (detect:
/// 3 levels, 3000 files). A change seen while an install runs is kept and
/// tried again at the next round.
pub fn watch_projects(kit: crate::kit::Kit, editor: std::sync::Arc<crate::editor::Editor>) {
    const WATCH_EVERY: std::time::Duration = std::time::Duration::from_secs(15);
    std::thread::spawn(move || {
        let mut seen = std::collections::BTreeMap::new();
        loop {
            std::thread::sleep(WATCH_EVERY);
            let open = open_projects();
            seen.retain(|dir, _| open.contains(dir));
            for dir in open {
                let now = extension_inputs(&kit, &dir);
                match seen.get(&dir) {
                    // Opening the project already synced.
                    None => {
                        seen.insert(dir, now);
                    }
                    Some(before) if *before != now => {
                        crate::log::info(&format!(
                            "editor: {} changed, extensions synced",
                            dir.display()
                        ));
                        if editor.sync_extensions() {
                            seen.insert(dir, now);
                        }
                    }
                    Some(_) => {}
                }
            }
        }
    });
}

/// Tells the editor windows which extensions Atollo just installed.
pub fn notify_installed<R: Runtime>(app: &AppHandle<R>, ids: &[String]) {
    let names: Vec<String> = ids
        .iter()
        .map(|id| crate::editor_ext::name_of(id))
        .collect();
    let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".into());
    for (label, window) in app.webview_windows() {
        if label.starts_with(LABEL_PREFIX) {
            let _ = window.eval(format!(
                "window.__atolloExtensions && window.__atolloExtensions({json})"
            ));
        }
    }
}

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
    lang: &'static str,
) -> Result<()> {
    let target: Url = url.parse().context("editor URL")?;
    let label = format!("{LABEL_PREFIX}{}", NEXT.fetch_add(1, Ordering::Relaxed));
    let handle = app.clone();
    let data = data_dir.to_path_buf();
    let (project, project_new) = (folder.to_path_buf(), folder.to_path_buf());
    let name = title.to_owned();
    let this = label.clone();
    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(target))
        .title(format!("{title} - Atollo"))
        .inner_size(1280.0, 820.0)
        .min_inner_size(640.0, 400.0)
        // The editor's title bar is the window's (D-69).
        .decorations(false)
        .shadow(true)
        .initialization_script(controls_script(lang, port, title))
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
            sync_maximized(&w);
            let mut url = payload.url().clone();
            url.set_query(None);
            crate::log::info(&format!(
                "editor {}: {:?} {url}",
                w.label(),
                payload.event()
            ));
        })
        .on_new_window(move |u, _features| {
            if let Some(what) = control(&u, port) {
                if let Some(w) = handle.get_webview_window(&this) {
                    // Not on the WebView2 callback, as for a new window below.
                    std::thread::spawn(move || apply(&w, what));
                }
            } else if same_origin(&u, port) && !in_project(&u, &project_new) {
                crate::log::warn("editor: blocked new window out of the project");
            } else if same_origin(&u, port) {
                let (app, data, url) = (handle.clone(), data.clone(), u.to_string());
                let (folder, name) = (project_new.clone(), name.clone());
                // Not on the WebView2 callback: building a window there would deadlock.
                std::thread::spawn(move || {
                    if let Err(e) = open(&app, &url, port, &name, &folder, &data, lang) {
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
    OPEN.lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(label.clone(), folder.to_path_buf());
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
    fn title_bar_buttons() {
        let u = |s: &str| s.parse::<Url>().unwrap();
        let at = |p: &str| control(&u(&format!("http://127.0.0.1:7003{p}")), 7003);
        assert_eq!(at("/__atollo/window/minimize"), Some(Control::Minimize));
        assert_eq!(
            at("/__atollo/window/maximize"),
            Some(Control::ToggleMaximize)
        );
        assert_eq!(at("/__atollo/window/close"), Some(Control::Close));
        assert_eq!(at("/__atollo/window/close/x"), None);
        assert_eq!(at("/__atollo/window/"), None);
        assert_eq!(at("/?folder=/e:/p"), None);
        let other = u("http://127.0.0.1:7004/__atollo/window/close");
        assert_eq!(control(&other, 7003), None);
        let js = controls_script("en", 7003, "my app");
        assert!(js.contains(r#"close: "Close""#), "{js}");
        assert!(js.contains(r#"restart: "Restart extensions""#), "{js}");
        assert!(js.contains("window.__atolloExtensions = "), "{js}");
        assert!(js.contains(r#"c.productConfiguration = "#), "{js}");
        assert!(js.contains(r#"line("project", "my app")"#), "{js}");
        assert!(js.contains("data:image/png;base64,iVBOR"), "logo embedded");
        assert!(
            js.contains(r#"{"window.menuBarVisibility":"classic"}"#),
            "{js}"
        );
        assert!(
            js.contains(r#""http://127.0.0.1:7003/__atollo/window/" + what"#),
            "{js}"
        );
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
