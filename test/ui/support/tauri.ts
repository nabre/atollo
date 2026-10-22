import { vi } from "vitest";

// The Tauri core as the tests see it. A test file mocks the module with
//   vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));
// and drives `invoke`, imported from here, with mockImplementation; Vitest
// gives every test file its own copy of both modules.
export const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
