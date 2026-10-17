// The app's terminal of macOS and Linux: the sessions (state.ts), shown in
// a panel of the main window (panel.tsx, loaded lazily with xterm.js), one
// view each (view.tsx).
export { tauriPty, type PtyBackend, type PtyChunk, type PtyInfo } from "./api";
export { useTerminals, type Terminals } from "./state";
