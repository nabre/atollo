// The app's terminal on every system: the sessions (state.ts), shown in a
// panel of the main window (panel.tsx, loaded lazily with xterm.js) or in
// a window of their own (window.tsx, terminal.html), one view each
// (view.tsx); the Terminals page, the kit consoles saved (page.tsx).
export { tauriPty, type PtyBackend, type PtyChunk, type PtyInfo } from "./api";
export { useTerminals, type Terminals } from "./state";
export { TerminalsPage } from "./page";
