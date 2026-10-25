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
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result};
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, Runtime, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
#[cfg(windows)]
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings3;
#[cfg(windows)]
use windows::core::Interface;

use crate::platform::jobs;

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
        crate::common::log::warn(&format!("editor {}: {what:?}: {e}", window.label()));
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
const LOGO_32: &[u8] = include_bytes!("../../../ui/src/assets/logo-32.png");
const LOGO_256: &[u8] = include_bytes!("../../../ui/src/assets/logo-256.png");

/// The editor's name where VSCodium shows its own: window title, About
/// (D-72). Merged by the workbench over its product, as `productConfiguration`.
const PRODUCT: &str = r#"{"nameShort":"Atollo","nameLong":"Atollo"}"#;

/// Atollo's typefaces in the editor (MARKETING.md, section 6): Geist for the
/// workbench, Geist Mono for the text and the terminal (`cfg.rs` names it in
/// `editor.fontFamily`). The same files as the site's (site/assets/fonts,
/// OFL-1.1, from @fontsource-variable 5.3.0): latin, latin-ext and, for
/// Geist Mono, the box-drawing block of the terminal's TUIs. The server's
/// page takes fonts only from its own origin or `blob:` (VSCodium's CSP),
/// and the server cannot serve the kit's files: the script builds them from
/// their bytes with the FontFace API, which loads no URL and no CSP limits.
const LATIN: &str = "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD";
const LATIN_EXT: &str = "U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF";
const SYMBOLS: &str = "U+2000-2001,U+2004-2008,U+200A,U+23B8-23BD,U+2500-259F";
const FONT_FILES: &[(&str, &[u8], &str)] = &[
    (
        "Geist",
        include_bytes!("../../../site/assets/fonts/geist-latin-wght-normal.woff2"),
        LATIN,
    ),
    (
        "Geist",
        include_bytes!("../../../site/assets/fonts/geist-latin-ext-wght-normal.woff2"),
        LATIN_EXT,
    ),
    (
        "Geist Mono",
        include_bytes!("../../../site/assets/fonts/geist-mono-latin-wght-normal.woff2"),
        LATIN,
    ),
    (
        "Geist Mono",
        include_bytes!("../../../site/assets/fonts/geist-mono-latin-ext-wght-normal.woff2"),
        LATIN_EXT,
    ),
    (
        "Geist Mono",
        include_bytes!("../../../site/assets/fonts/geist-mono-symbols2-wght-normal.woff2"),
        SYMBOLS,
    ),
];

/// The workbench's font, with the system's behind it (CJK, a font that
/// failed to load).
const WORKBENCH_FONT: &str = "'Geist', system-ui, 'Segoe UI', sans-serif";

/// `[[family, base64, unicode-range], ...]` for the script.
fn font_faces() -> String {
    use base64::Engine;
    let faces: Vec<(&str, String, &str)> = FONT_FILES
        .iter()
        .map(|(family, bytes, range)| {
            let data = base64::engine::general_purpose::STANDARD.encode(bytes);
            (*family, data, *range)
        })
        .collect();
    serde_json::to_string(&faces).unwrap_or_else(|_| "[]".into())
}

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
fn controls_script(lang: &str, port: u16, project: &str, hidden: &[String]) -> String {
    let label = |it: &str| {
        serde_json::to_string(&crate::common::i18n::t(lang, it, &[])).unwrap_or_default()
    };
    let (logo32, logo256) = (data_uri(LOGO_32), data_uri(LOGO_256));
    let hidden_key = serde_json::to_string(&hidden_key(project)).unwrap_or_default();
    let project = serde_json::to_string(project).unwrap_or_default();
    let hidden = serde_json::to_string(hidden).unwrap_or_else(|_| "[]".into());
    let stamp = now_ms();
    let fonts = font_faces();
    let (sans, mono) = (
        serde_json::to_string(WORKBENCH_FONT).unwrap_or_default(),
        serde_json::to_string(crate::editor::cfg::EDITOR_FONT).unwrap_or_default(),
    );
    format!(
        r##"(() => {{
  if (window.top !== window) return;
  // Geist and Geist Mono, before the workbench measures any text.
  if (document.fonts && window.FontFace) {{
    for (const [family, data, range] of {fonts}) {{
      try {{
        const bytes = Uint8Array.from(atob(data), (ch) => ch.charCodeAt(0));
        document.fonts.add(new FontFace(family, bytes, {{ weight: "100 900", unicodeRange: range }}));
      }} catch (e) {{}}
    }}
  }}
  const sans = {sans};
  const mono = {mono};
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
  // Extensions the project does not use (D-71): off in this window's
  // workspace, written in the workspace's storage of the workbench before
  // it reads it (the first transaction on the database wins). Which ones the
  // kit turned off is kept apart: the user's own choices stay, and one the
  // project wants again is turned back on. Atollo sends a newer list while
  // the window is open (localStorage), used at the next reload.
  const hidden = () => {{
    let ids = {hidden};
    try {{
      const s = JSON.parse(localStorage.getItem({hidden_key}));
      if (s && s.t > {stamp} && Array.isArray(s.ids)) ids = s.ids;
    }} catch (e) {{}}
    return ids.map((i) => String(i).toLowerCase());
  }};
  const turnOff = (db) => {{
    const tx = db.transaction("ItemTable", "readwrite");
    const store = tx.objectStore("ItemTable");
    const KEY = "extensionsIdentifiers/disabled", MINE = "atollo/disabled";
    const parse = (v) => {{ try {{ const x = JSON.parse(v); return Array.isArray(x) ? x : []; }} catch (e) {{ return []; }} }};
    const now = store.get(KEY), mine = store.get(MINE);
    mine.onsuccess = () => {{
      const was = new Set(parse(mine.result).map((i) => String(i).toLowerCase()));
      const keep = parse(now.result)
        .filter((e) => e && typeof e.id === "string" && !was.has(e.id.toLowerCase()));
      const user = new Set(keep.map((e) => e.id.toLowerCase()));
      const add = hidden().filter((i) => !user.has(i));
      store.put(JSON.stringify(keep.concat(add.map((id) => ({{ id }})))), KEY);
      store.put(JSON.stringify(add), MINE);
    }};
  }};
  const idbOpen = indexedDB.open.bind(indexedDB);
  indexedDB.open = function (name, ...rest) {{
    const req = idbOpen(name, ...rest);
    if (typeof name === "string" && name.startsWith("vscode-web-state-db-")
        && !name.startsWith("vscode-web-state-db-global")) {{
      // Before the workbench's own handler, set after this returns.
      req.addEventListener("success", () => {{ try {{ turnOff(req.result); }} catch (e) {{}} }});
    }}
    return req;
  }};
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
    // The workbench's own font is per system (.windows, .mac, .linux, also
    // by :lang): Geist over all of them, as the app; the text and the
    // terminal take theirs from the settings (cfg.rs).
    style.textContent = ".monaco-workbench{{font-family:" + sans + "!important;"
      + "--monaco-monospace-font:" + mono + "}}"
      + ".monaco-workbench.web .part.titlebar .titlebar-right "
      + ".window-controls-container{{width:138px}}"
      + "#atollo-titlebar{{position:fixed;top:0;left:0;right:0;height:30px;z-index:2147483647;"
      + "display:flex;font:12px " + sans + ";user-select:none;"
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

/// Where a window keeps the newer list of its hidden extensions.
fn hidden_key(project: &str) -> String {
    format!("atollo-hidden:{project}")
}

use crate::common::time::now_ms;

/// Sends each open editor window the extensions its project hides now: a
/// language or a choice changed. They apply at the window's next reload.
pub fn refresh_hidden<R: Runtime>(app: &AppHandle<R>, kit: &crate::kit::Kit) {
    let open: Vec<(String, std::path::PathBuf)> = OPEN
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .map(|(l, d)| (l.clone(), d.clone()))
        .collect();
    for (label, dir) in open {
        let Some(window) = app.get_webview_window(&label) else {
            continue;
        };
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let value =
            serde_json::json!({ "t": now_ms(), "ids": crate::editor::ext::hidden_of(kit, &dir) });
        let key = serde_json::to_string(&hidden_key(&name)).unwrap_or_default();
        let value = serde_json::to_string(&value.to_string()).unwrap_or_default();
        let _ = window.eval(format!(
            "try {{ localStorage.setItem({key}, {value}); }} catch (e) {{}}"
        ));
    }
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
    let choice = crate::projects::settings::load(kit, &name).extensions;
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
pub fn watch_projects(
    kit: crate::kit::Kit,
    editor: std::sync::Arc<crate::editor::Editor>,
    on_change: std::sync::Arc<dyn Fn() + Send + Sync>,
) {
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
                        crate::common::log::info(&format!(
                            "editor: {} changed, extensions synced",
                            dir.display()
                        ));
                        on_change();
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
        .map(|id| crate::editor::ext::name_of(id))
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
    in_project_on(crate::kit::Os::current(), url, folder)
}

/// [`in_project`] for a folder of `os`.
pub fn in_project_on(os: crate::kit::Os, url: &Url, folder: &Path) -> bool {
    let want = super::folder_param_on(os, folder);
    // A Windows path (`/C:/...`) is matched in any case, as Windows does;
    // a Unix one exactly: on Linux `/home/a/App` is another folder.
    let fold = os.is_windows();
    let norm = |s: &str| {
        let s = s.trim_end_matches('/');
        if fold {
            s.to_lowercase()
        } else {
            s.to_owned()
        }
    };
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
    jobs::open_url(url.as_str())
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
        .initialization_script(controls_script(
            lang,
            port,
            title,
            &crate::editor::ext::hidden_of(&app.state::<crate::ipc::AppState>().kit, folder),
        ))
        .data_directory(data_dir.to_path_buf())
        // Light or dark as the settings say (D-137); VSCodium follows it.
        .theme(
            crate::common::settings::Settings::load(&app.state::<crate::ipc::AppState>().kit)
                .settings
                .theme
                .window(),
        )
        .zoom_hotkeys_enabled(false)
        .on_navigation(move |u| {
            if same_origin(u, port) {
                let ok = in_project(u, &project);
                if !ok {
                    crate::common::log::warn("editor: blocked navigation out of the project");
                }
                return ok;
            }
            let ok = u.scheme() == "about";
            if !ok {
                if classify_external(u) == Leave::Browser {
                    let _ = open_in_browser(u);
                } else {
                    crate::common::log::warn(&format!(
                        "editor: blocked navigation to {}",
                        u.scheme()
                    ));
                }
            }
            ok
        })
        .on_page_load(|w, payload| {
            sync_maximized(&w);
            let mut url = payload.url().clone();
            url.set_query(None);
            crate::common::log::info(&format!(
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
                crate::common::log::warn("editor: blocked new window out of the project");
            } else if same_origin(&u, port) {
                let (app, data, url) = (handle.clone(), data.clone(), u.to_string());
                let (folder, name) = (project_new.clone(), name.clone());
                // Not on the WebView2 callback: building a window there would deadlock.
                std::thread::spawn(move || {
                    if let Err(e) = open(&app, &url, port, &name, &folder, &data, lang) {
                        crate::common::log::error(&format!("editor: new window: {e:#}"));
                    }
                });
            } else if classify_external(&u) == Leave::Browser {
                let _ = open_in_browser(&u);
            } else {
                crate::common::log::warn(&format!("editor: blocked new window to {}", u.scheme()));
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

/// WebView2 settings: nothing to change in WKWebView or WebKitGTK.
#[cfg(not(windows))]
pub fn quiet_webview<R: Runtime>(_window: &WebviewWindow<R>, _what: &'static str) {}

/// Browser accelerator keys, context menu and status bar off, so Ctrl+W,
/// Ctrl+N, Ctrl+F, F5... reach the page. `what` names the window in the log.
#[cfg(windows)]
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
            crate::common::log::warn(&format!("{what}: webview settings: {e}"));
        }
    });
    if let Err(e) = applied {
        crate::common::log::warn(&format!("{what}: webview settings: {e}"));
    }
}

#[cfg(test)]
#[path = "../../../test/rust/editor/window.rs"]
mod tests;
