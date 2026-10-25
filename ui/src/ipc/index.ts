// The interface's only way to Rust (IPC), one file per domain: the same
// split as tauri/src/ipc/. Commands through the helpers of core.ts, which
// stay inside this folder; the events' names in events.ts. Pages import from
// "@/ipc" (or "../ipc"); the terminal imports pty.ts by itself, so that the
// window of a detached session (terminal.html) carries no other domain.
export { browserPreview } from "./core";
export * from "./app";
export * from "./window";
export * from "./editing";
export * from "./setup";
export * from "./versions";
export * from "./agents";
export * from "./services";
export * from "./llm";
export * from "./databases";
export * from "./projects";
export * from "./orphans";
export * from "./frameworks";
export * from "./runs";
export * from "./github";
export * from "./servers";
export * from "./terminals";
