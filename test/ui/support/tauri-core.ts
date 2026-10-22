import { invoke as mock } from "./tauri";

// What the app imports from "@tauri-apps/api/core" in the tests (see
// tauri.ts): every call reaches the mock with both arguments, so a test
// asserts ("cmd", undefined) for a command without arguments; and the app
// always believes it runs in Tauri.
export const invoke = (cmd: string, args?: unknown) => mock(cmd, args);
export const isTauri = () => true;
