// The window of a detached terminal session (terminal-<id>, D-149): the
// panel with its one tab, filling the window. Rust gives this window only
// its own session (pty_sessions) and its events; the language comes from
// the window's start script, as the editor windows' (editor/terminal.rs).
import "../shell/freeze";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "@fontsource-variable/geist/wght.css";
import "../theme/theme.css";
import { setLang, type Lang, LANGS } from "../i18n";
import { tauriPty } from "../ipc/pty";
import { monoFamily, monoReady } from "./font";
import { TerminalPanel } from "./panel";
import { useTerminals } from "./state";

declare global {
  interface Window {
    __ATOLLO_LANG__?: string;
  }
}

function DetachedTerminal() {
  const terminals = useTerminals(tauriPty, true);
  return (
    <div className="flex h-full flex-col bg-vsc-panel">
      {terminals.sessions.length > 0 && <TerminalPanel terminals={terminals} backend={tauriPty} detached />}
    </div>
  );
}

const lang = LANGS.find((l) => l.id === window.__ATOLLO_LANG__)?.id;
if (lang) setLang(lang satisfies Lang);
const root = document.getElementById("root");
if (root) {
  // The tab's labels in Geist, the terminal's cells in Geist Mono (the
  // view waits for it too).
  void Promise.allSettled([document.fonts.load('13px "Geist Variable"'), monoReady(monoFamily())]).then(() => {
    createRoot(root).render(
      <StrictMode>
        <DetachedTerminal />
      </StrictMode>,
    );
  });
}
