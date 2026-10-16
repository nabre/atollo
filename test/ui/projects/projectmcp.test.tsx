// A project's MCP servers and editor settings (BUG-18r): a failed read is
// shown, with "Riprova" to read again, instead of an empty box.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
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
});
