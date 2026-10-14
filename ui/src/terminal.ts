// Console window of the kit (terminal.html, windows `terminal-<n>`): xterm.js
// on the process that Rust started on a pseudo console (src-tauri/src/terminal.rs).
// The window reaches only its own session: three commands, by window label
// (DECISIONS.md D-13, D-14).
//
// Keys like Windows Terminal: Ctrl+C copies when text is selected, otherwise
// it goes to the program; Ctrl+V and Ctrl+Shift+V paste; Ctrl+Shift+C copies;
// right click copies the selection or pastes.
// No freeze.ts here: xterm.js assigns `toString` on its objects and does
// not load with a frozen Object.prototype. The page parses no outside data.
import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Terminal, type ITheme } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";

// Terminal colors of VSCodium's "Default Dark Modern" / "Default Light Modern".
const DARK: ITheme = {
  background: "#181818",
  foreground: "#CCCCCC",
  cursor: "#CCCCCC",
  selectionBackground: "#264F78",
  black: "#000000",
  red: "#CD3131",
  green: "#0DBC79",
  yellow: "#E5E510",
  blue: "#2472C8",
  magenta: "#BC3FBC",
  cyan: "#11A8CD",
  white: "#E5E5E5",
  brightBlack: "#666666",
  brightRed: "#F14C4C",
  brightGreen: "#23D18B",
  brightYellow: "#F5F543",
  brightBlue: "#3B8EEA",
  brightMagenta: "#D670D6",
  brightCyan: "#29B8DB",
  brightWhite: "#E5E5E5",
};

const LIGHT: ITheme = {
  background: "#F8F8F8",
  foreground: "#3B3B3B",
  cursor: "#3B3B3B",
  selectionBackground: "#ADD6FF",
  black: "#000000",
  red: "#CD3131",
  green: "#107C10",
  yellow: "#949800",
  blue: "#0451A5",
  magenta: "#BC05BC",
  cyan: "#0598BC",
  white: "#555555",
  brightBlack: "#666666",
  brightRed: "#CD3131",
  brightGreen: "#14CE14",
  brightYellow: "#B5BA00",
  brightBlue: "#0451A5",
  brightMagenta: "#BC05BC",
  brightCyan: "#0598BC",
  brightWhite: "#A5A5A5",
};

const light = window.matchMedia("(prefers-color-scheme: light)");
const theme = () => (light.matches ? LIGHT : DARK);

function paint(t: ITheme) {
  document.documentElement.style.background = t.background ?? "";
  document.body.style.background = t.background ?? "";
}

const host = document.getElementById("terminal");
if (host) {
  document.body.style.margin = "0";
  document.body.style.overflow = "hidden";
  host.style.position = "absolute";
  host.style.inset = "4px 0 0 8px";

  const term = new Terminal({
    fontFamily: '"Cascadia Mono", Consolas, "Courier New", monospace',
    fontSize: 14,
    cursorBlink: true,
    scrollback: 10000,
    allowProposedApi: false,
    theme: theme(),
  });
  paint(theme());
  light.addEventListener("change", () => {
    term.options.theme = theme();
    paint(theme());
  });
  const fit = new FitAddon();
  term.loadAddon(fit);
  term.open(host);
  fit.fit();
  new ResizeObserver(() => {
    fit.fit();
  }).observe(host);

  const copy = () => {
    const text = term.getSelection();
    if (text) {
      void navigator.clipboard.writeText(text);
      term.clearSelection();
    }
    return Boolean(text);
  };
  const paste = () => {
    navigator.clipboard.readText().then(
      (text) => {
        term.paste(text);
      },
      () => undefined,
    );
  };
  term.attachCustomKeyEventHandler((e) => {
    if (e.type !== "keydown" || !e.ctrlKey || e.altKey) return true;
    const key = e.key.toLowerCase();
    if (key === "c" && (e.shiftKey || term.hasSelection())) {
      copy();
      e.preventDefault();
      return false;
    }
    if (key === "v") {
      paste();
      e.preventDefault();
      return false;
    }
    return true;
  });
  host.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    if (!copy()) paste();
  });

  if (!isTauri()) {
    term.write("Anteprima nel browser: il terminale funziona nella finestra di Atollo.\r\n");
  } else {
    void connect(term, fit);
  }
  term.focus();
}

async function connect(term: Terminal, fit: FitAddon) {
  const win = getCurrentWindow();
  let exited = false;

  // Input in order, one call at a time: keys typed meanwhile go together.
  let queued = "";
  let sending = false;
  const send = async () => {
    if (sending) return;
    sending = true;
    while (queued) {
      const data = queued;
      queued = "";
      await invoke("terminal_input", { data }).catch(() => undefined);
    }
    sending = false;
  };
  term.onData((data) => {
    if (exited) {
      void win.close();
      return;
    }
    queued += data;
    void send();
  });
  term.onResize(({ cols, rows }) => {
    void invoke("terminal_resize", { cols, rows }).catch(() => undefined);
  });

  const output = new Channel<ArrayBuffer>();
  output.onmessage = (data) => {
    term.write(new Uint8Array(data));
  };
  const exit = new Channel<number>();
  exit.onmessage = (code) => {
    exited = true;
    term.options.cursorBlink = false;
    term.write(
      `\r\n\x1b[90m[Processo terminato (codice ${String(code)}). Premi un tasto per chiudere la finestra.]\x1b[0m`,
    );
  };

  try {
    const title = await invoke<string>("terminal_attach", { output, exit });
    term.onTitleChange((t) => {
      void win.setTitle(`${t || title} - Atollo`).catch(() => undefined);
    });
    fit.fit();
    await invoke("terminal_resize", { cols: term.cols, rows: term.rows });
  } catch (e) {
    term.write(`\x1b[31mTerminale non disponibile: ${String(e)}\x1b[0m\r\n`);
  }
}
