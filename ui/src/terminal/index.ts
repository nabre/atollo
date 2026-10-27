// The app's terminal on every system: the sessions (state.ts), shown in a
// panel of the main window (panel.tsx, loaded lazily with xterm.js) or in
// a window of their own (window.tsx, terminal.html), one view each
// (view.tsx). The Terminals page, the kit consoles saved (page.tsx), is
// loaded lazily by App (D-182), never through here.
export { tauriPty, type PtyBackend, type PtyChunk, type PtyInfo } from "../ipc/pty";
export { useTerminals, type Terminals } from "./state";
