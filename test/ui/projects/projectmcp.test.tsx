// A project's MCP servers and editor settings (BUG-18r): a failed read is
// shown, with "Riprova" to read again, instead of an empty box. The focus
// is not lost when what has it goes: after Riprova, after Salva.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { EditorSettingsView, McpView } from "../../../ui/src/api";
import { ProjectMcpBox } from "../../../ui/src/projectmcp";
import { EditorSettingsBox } from "../../../ui/src/editorsettings";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const MCP: McpView = {
  servers: [{ name: "playwright", kind: "stdio", command: "npx", args: ["@playwright/mcp"], url: null, env: {} }],
  readOnly: false,
};

const SETTINGS: EditorSettingsView = {
  settings: { formatOnSave: true, tabSize: 4, insertSpaces: true, hidden: ["**/vendor"] },
  exists: true,
  readOnly: false,
};

/** `cmd` fails the first time, then answers `value`. */
function failOnce(cmd: string, value: unknown) {
  let failed = false;
  invoke.mockImplementation((c) => {
    if (c !== cmd) return Promise.reject(new Error(`unexpected ${c}`));
    if (failed) return Promise.resolve(value);
    failed = true;
    return Promise.reject(new Error("file non leggibile"));
  });
}

describe("ProjectMcpBox", () => {
  it("shows a failed read and reads again on Riprova", async () => {
    failOnce("project_mcp", MCP);
    render(<ProjectMcpBox project="sito" />);
    expect((await screen.findByRole("alert")).textContent).toContain("file non leggibile");
    expect(screen.queryByText("playwright")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Riprova" }));
    expect(await screen.findByText("playwright")).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();
    expect(invoke).toHaveBeenCalledTimes(2);
    expect(invoke).toHaveBeenLastCalledWith("project_mcp", { project: "sito" });
  });
});

describe("EditorSettingsBox", () => {
  it("shows a failed read and reads again on Riprova", async () => {
    failOnce("project_editor_settings", SETTINGS);
    render(<EditorSettingsBox project="sito" />);
    expect((await screen.findByRole("alert")).textContent).toContain("file non leggibile");
    fireEvent.click(screen.getByRole("button", { name: "Riprova" }));
    expect(await screen.findByRole("heading", { name: "Impostazioni dell'editor per questo progetto" })).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByRole("button", { name: "Salva" })).toBeTruthy();
  });

  it("is busy while it reads again, and puts the focus on the heading once the read works", async () => {
    let fail = true;
    const reads: Array<() => void> = [];
    invoke.mockImplementation((c) => {
      if (c !== "project_editor_settings") return Promise.reject(new Error(`unexpected ${c}`));
      if (reads.length === 0 && fail) return Promise.reject(new Error("file non leggibile"));
      // The next reads wait for the test.
      return new Promise((resolve, reject) => {
        reads.push(() => {
          if (fail) reject(new Error("file non leggibile"));
          else resolve(SETTINGS);
        });
      });
    });
    render(<EditorSettingsBox project="sito" />);
    await screen.findByRole("alert");
    // The first read failed; the next ones wait.
    reads.push(() => undefined);
    const retry = screen.getByRole("button", { name: "Riprova" });
    retry.focus();
    fireEvent.click(retry);
    // Busy while it reads, even when it fails again with the same error.
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Riprova…" }).getAttribute("aria-busy")).toBe("true");
    });
    reads.at(-1)?.();
    const again = await screen.findByRole("button", { name: "Riprova" });
    expect(again.hasAttribute("aria-busy")).toBe(false);
    expect(screen.getByRole("alert").textContent).toContain("file non leggibile");
    fail = false;
    again.focus();
    fireEvent.click(again);
    await waitFor(() => {
      expect(reads.length).toBe(3);
    });
    reads.at(-1)?.();
    const heading = await screen.findByRole("heading", { name: "Impostazioni dell'editor per questo progetto" });
    await waitFor(() => {
      expect(document.activeElement).toBe(heading);
    });
  });

  it("keeps the focus on Salva once saved, while the form starts again from the file", async () => {
    let settings = SETTINGS;
    invoke.mockImplementation((c) => {
      if (c === "project_editor_settings") return Promise.resolve(settings);
      if (c === "project_editor_settings_set") {
        // The file read after the save is another: a new form.
        settings = { ...SETTINGS, settings: { ...SETTINGS.settings, tabSize: 2 } };
        return Promise.resolve(undefined);
      }
      return Promise.reject(new Error(`unexpected ${c}`));
    });
    render(<EditorSettingsBox project="sito" />);
    const save = await screen.findByRole("button", { name: "Salva" });
    save.focus();
    fireEvent.submit(save.closest("form") as HTMLFormElement);
    expect(await screen.findByText("Salvato.")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("project_editor_settings_set", expect.anything());
    await waitFor(() => {
      const now = screen.getByRole("button", { name: "Salva" });
      expect(now).not.toBe(save);
      expect(document.activeElement).toBe(now);
    });
  });
});
