// The Gemini API key on Antigravity's page: saved and removed alone, by
// Rust at once (`gemini_key_set`), never by sending back the settings read
// here, which would undo a change of the Settings page saved meanwhile.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SAMPLE_SETTINGS } from "@/ipc";
import { GeminiKey } from "@/agents/gemini";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

describe("GeminiKey", () => {
  it("removes the saved key alone, and then says there is none", async () => {
    invoke.mockImplementation((c) =>
      Promise.resolve(c === "settings_get" ? { ...SAMPLE_SETTINGS, geminiKeySet: true } : undefined),
    );
    const done = vi.fn();
    render(<GeminiKey onDone={done} />);
    fireEvent.click(await screen.findByText("Togli la chiave"));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("gemini_key_set", { key: "" });
    });
    expect(invoke).not.toHaveBeenCalledWith("settings_set", expect.anything());
    await waitFor(() => {
      expect(screen.queryByText("Togli la chiave")).toBeNull();
    });
    expect(done).toHaveBeenCalled();
  });
});
