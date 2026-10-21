// The settings saved as they change (D-137): a Gemini key whose save was
// refused goes again with the next change, never dropped as «unchanged».
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook } from "@testing-library/react";
import type { Settings, SettingsView } from "../../../ui/src/api";
import { useSettingsAutosave } from "../../../ui/src/settings/autosave";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
  vi.useRealTimers();
});

const SETTINGS: Settings = {
  lang: null,
  network: { httpsProxy: null, httpProxy: null, noProxy: null, extraCaCerts: null, allowLan: false },
  geminiApiKey: null,
  autoUpdate: true,
  rustToolchain: null,
  programTemp: "auto",
  theme: "system",
  keepCondaPackages: true,
  toolsTarget: null,
};
const VIEW: SettingsView = { settings: SETTINGS, geminiKeySet: false, problem: null };

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
  it("sends a refused Gemini key again with the next change, then forgets it", async () => {
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
      result.current.setGemini("AIza-1");
    });
    await wait();
    expect(result.current.error).toBe("Error: no");
    refuse = false;
    act(() => {
      result.current.set({ autoUpdate: false });
    });
    await wait();
    expect(result.current.error).toBeNull();
    expect(result.current.view?.geminiKeySet).toBe(true);
    act(() => {
      result.current.set({ autoUpdate: true });
    });
    await wait();
    expect(geminiSent()).toEqual(["AIza-1", "AIza-1", null]);
  });

  it("lets a newer key win over the refused one", async () => {
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
      result.current.setGemini("AIza-1");
    });
    await wait();
    refuse = false;
    act(() => {
      result.current.setGemini("AIza-2");
    });
    await wait();
    act(() => {
      result.current.set({ autoUpdate: false });
    });
    await wait();
    expect(geminiSent()).toEqual(["AIza-1", "AIza-2", null]);
  });
});
