// The app's terminal on every system (tauri/src/editor/pty.rs, the kit's
// ConPTY on Windows; commands in tauri/src/commands/terminals.rs): sessions
// opened by the Terminal and agent buttons, shown by the panel of the main
// window (D-109, D-148).
import { act, query, subscribe } from "../api/core";

export interface PtyInfo {
  id: number;
  /** The folder's name and what runs ("app · bash", "app · claude"). */
  title: string;
  dir: string;
  /** False once its program ended: the tab stays until it is closed. */
  running: boolean;
  /** The exit code, when it ended with one. */
  code: number | null;
}

/** Output of a session: the chunk `seq`, or for a backlog everything kept
 * up to the chunk `seq`. */
export interface PtyChunk {
  seq: number;
  data: string;
}

/** What the terminal panel needs from Rust; a fake one in the tests. */
export interface PtyBackend {
  sessions(): Promise<PtyInfo[]>;
  backlog(id: number): Promise<PtyChunk>;
  write(id: number, data: string): Promise<void>;
  /** Bytes that are not text (xterm's onBinary: some mouse reports), one
   * character per byte. */
  writeBinary(id: number, data: string): Promise<void>;
  resize(id: number, cols: number, rows: number): Promise<void>;
  close(id: number): Promise<void>;
  onOpened(handler: (info: PtyInfo) => void): Promise<() => void>;
  onExit(handler: (info: PtyInfo) => void): Promise<() => void>;
  onOutput(id: number, handler: (chunk: PtyChunk) => void): Promise<() => void>;
}

export const tauriPty: PtyBackend = {
  sessions: () => query<PtyInfo[]>("pty_sessions", []),
  backlog: (id) => query<PtyChunk>("pty_backlog", { seq: 0, data: "" }, { id }),
  write: (id, data) => act("pty_write", { id, data }),
  writeBinary: (id, data) => act("pty_write", { id, data, binary: true }),
  resize: (id, cols, rows) => act("pty_resize", { id, cols, rows }),
  close: (id) => act("pty_close", { id }),
  onOpened: (handler) => subscribe<PtyInfo>("pty-opened", handler),
  onExit: (handler) => subscribe<PtyInfo>("pty-exit", handler),
  onOutput: (id, handler) => subscribe<PtyChunk>(`pty://${String(id)}`, handler),
};
