// The settings saved as they change (D-137): the Gemini key is not theirs
// (the AI page saves it, D-165), a refused change tries again with the next
// one, a read that works clears the error of the failed one.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { SAMPLE_SETTINGS } from "@/ipc";
import { useSettingsAutosave } from "@/settings/autosave";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
  vi.useRealTimers();
});

// The browser preview's settings: no Gemini key saved.
const VIEW = SAMPLE_SETTINGS;
const SETTINGS = VIEW.settings;

/** The Gemini key of each settings_set, in order. */
function geminiSent(): (string | null)[] {
  return invoke.mock.calls
    .filter(([cmd]) => cmd === "settings_set")
    .map(([, a]) => (a as { gemini: string | null }).gemini);
}

/** Lets the wait after a change pass, and the save run. */
const wait = () =>
  act(async () => {
    await vi.advanceTimersByTimeAsync(600);
  });

describe("useSettingsAutosave", () => {
  it("never sends a Gemini key: every save keeps the one saved (D-165)", async () => {
    vi.useFakeTimers();
    invoke.mockImplementation((cmd) => {
      if (cmd === "settings_get") return Promise.resolve(VIEW);
      return Promise.resolve();
    });
    const { result } = renderHook(() => useSettingsAutosave());
    await wait();
    act(() => {
      result.current.set({ autoUpdate: false });
    });
    await wait();
    act(() => {
      result.current.set({ autoUpdate: true });
    });
    await wait();
    expect(geminiSent()).toEqual([null, null]);
  });

  it("tries a refused change again with the next one", async () => {
    vi.useFakeTimers();
    let refuse = true;
    invoke.mockImplementation((cmd) => {
      if (cmd === "settings_get") return Promise.resolve(VIEW);
      if (cmd === "settings_set") return refuse ? Promise.reject(new Error("no")) : Promise.resolve();
      return Promise.resolve();
    });
    const { result } = renderHook(() => useSettingsAutosave());
    await wait();
    act(() => {
      result.current.set({ autoUpdate: false });
    });
    await wait();
    expect(result.current.error).toBe("Error: no");
    refuse = false;
    act(() => {
      result.current.set({ theme: "dark" });
    });
    await wait();
    expect(result.current.error).toBeNull();
    expect(result.current.status).toBe("saved");
  });

  it("clears a failed read when the next one works", async () => {
    let fail = true;
    invoke.mockImplementation((cmd) => {
      if (cmd === "settings_get") return fail ? Promise.reject(new Error("letto male")) : Promise.resolve(VIEW);
      return Promise.resolve();
    });
    const { result } = renderHook(() => useSettingsAutosave());
    await waitFor(() => {
      expect(result.current.loadError).toBe("Error: letto male");
    });
    expect(result.current.s).toBeNull();
    fail = false;
    let ok = false;
    await act(async () => {
      ok = await result.current.retry();
    });
    expect(ok).toBe(true);
    expect(result.current.loadError).toBeNull();
    expect(result.current.s).toEqual(SETTINGS);
    expect(result.current.view).toEqual(VIEW);
  });
});
