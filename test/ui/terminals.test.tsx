// The Terminals page: removing a terminal asks first, and Esc answers no
// (UX-V5-02).
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { TerminalsView } from "../../ui/src/api";
import { TerminalsPage } from "../../ui/src/terminals";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

const VIEW: TerminalsView = {
  terminals: [
    {
      id: "legacy-php",
      name: "PHP 7.4 legacy",
      dir: null,
      tools: { php: "7.4" },
      builtin: false,
      missing: [],
    },
  ],
  tools: [{ id: "php", name: "PHP", group: "languages", lines: ["8.4", "7.4"], versions: {} }],
  projects: "E:\\Atollo\\projects",
};

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

describe("TerminalsPage", () => {
  it("closes the question to remove a terminal with Esc, and removes it once confirmed", async () => {
    invoke.mockImplementation((cmd) => {
      if (cmd === "terminals_list") return Promise.resolve(VIEW);
      if (cmd === "terminal_remove") return Promise.resolve(null);
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<TerminalsPage />);
    fireEvent.click(await screen.findByRole("button", { name: "Elimina" }));
    const ask = screen.getByRole("alertdialog", { name: "Elimina il terminale" });
    fireEvent.keyDown(ask, { key: "Escape" });
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(invoke).not.toHaveBeenCalledWith("terminal_remove", expect.anything());
    fireEvent.click(screen.getByRole("button", { name: "Elimina" }));
    fireEvent.click(within(screen.getByRole("alertdialog")).getByRole("button", { name: "Elimina" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("terminal_remove", { id: "legacy-php" });
    });
  });
});
