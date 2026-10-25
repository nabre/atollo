// A project's MCP servers and editor settings (BUG-18r): a failed read is
// shown, with "Riprova" to read again, instead of an empty box. The focus
// is not lost when what has it goes: after Riprova, after Salva.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { SAMPLE_PRESETS, type EditorSettingsView, type McpServer, type McpView } from "@/ipc";
import { KitMcpBox, ProjectMcpBox } from "@/projects/mcp";
import { EditorSettingsBox } from "@/projects/editorsettings";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const MCP: McpView = {
  servers: [
    { name: "playwright", kind: "stdio", command: "npx", args: ["@playwright/mcp"], url: null, env: {}, headers: {} },
  ],
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
    render(<ProjectMcpBox project="sito" engines={[]} />);
    expect((await screen.findByRole("alert")).textContent).toContain("file non leggibile");
    expect(screen.queryByText("playwright")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Riprova" }));
    expect(await screen.findByText("playwright")).toBeTruthy();
    expect(screen.queryByRole("alert")).toBeNull();
    const reads = invoke.mock.calls.filter(([c]) => c === "project_mcp");
    expect(reads).toEqual([
      ["project_mcp", { project: "sito" }],
      ["project_mcp", { project: "sito" }],
    ]);
  });

  it("offers the project's ready servers, those of its databases only", async () => {
    let servers: McpServer[] = [];
    invoke.mockImplementation((c, args) => {
      if (c === "mcp_presets") return Promise.resolve(SAMPLE_PRESETS);
      if (c === "project_mcp") return Promise.resolve({ servers, readOnly: false });
      if (c === "project_mcp_set") {
        servers = (args as { servers: McpServer[] }).servers;
        return Promise.resolve(undefined);
      }
      return Promise.reject(new Error(`unexpected ${c}`));
    });
    render(<ProjectMcpBox project="sito" engines={["mysql"]} />);
    expect(await screen.findByText("Legge e scrive i file, solo nella cartella del progetto.")).toBeTruthy();
    expect(screen.getByText("Interroga il database MySQL del progetto (DBHub).")).toBeTruthy();
    // No MongoDB database, no MongoDB server; the kit's are on the Agents page.
    expect(screen.queryByText("Interroga il database MongoDB del progetto.")).toBeNull();
    expect(screen.queryByText("playwright")).toBeNull();
    const row = screen.getByText("Interroga il database MySQL del progetto (DBHub).").closest("li") as HTMLElement;
    fireEvent.click(within(row).getByRole("button", { name: "Aggiungi" }));
    await waitFor(() => {
      expect(servers.map((s) => s.name)).toEqual(["mysql"]);
    });
    expect(servers[0]?.env).toEqual({ DSN: "${DSN}" });
    // Added: listed, no longer offered.
    await waitFor(() => {
      expect(screen.queryByText("Interroga il database MySQL del progetto (DBHub).")).toBeNull();
    });
    expect(screen.getByText("npx -y @bytebase/dbhub --transport stdio")).toBeTruthy();
  });
});

describe("KitMcpBox", () => {
  it("lists the kit's servers and adds a ready one to data\\agents\\mcp.json", async () => {
    let servers: McpServer[] = SAMPLE_PRESETS.filter((p) => p.id === "playwright").map((p) => p.server);
    invoke.mockImplementation((c, args) => {
      if (c === "mcp_presets") return Promise.resolve(SAMPLE_PRESETS);
      if (c === "kit_mcp") return Promise.resolve({ servers, readOnly: false });
      if (c === "kit_mcp_set") {
        servers = (args as { servers: McpServer[] }).servers;
        return Promise.resolve(undefined);
      }
      return Promise.reject(new Error(`unexpected ${c}`));
    });
    render(<KitMcpBox />);
    expect(await screen.findByRole("heading", { name: "Server MCP del kit" })).toBeTruthy();
    expect(
      await screen.findByText("Issue, pull request e repository su GitHub, con l'account GitHub del kit."),
    ).toBeTruthy();
    // Playwright is there already; the project's ones are on its page.
    expect(screen.queryByText("Un browser che l'agente apre e usa: pagine, clic, moduli, schermate.")).toBeNull();
    expect(screen.queryByText("Legge e scrive i file, solo nella cartella del progetto.")).toBeNull();
    const row = screen
      .getByText("Issue, pull request e repository su GitHub, con l'account GitHub del kit.")
      .closest("li") as HTMLElement;
    fireEvent.click(within(row).getByRole("button", { name: "Aggiungi" }));
    await waitFor(() => {
      expect(servers.map((s) => s.name)).toEqual(["playwright", "github"]);
    });
    // The app, by its absolute path, reads the token itself: no token, not
    // even a variable of it (SEC-V7-01).
    expect(servers[1]?.command).toBe("E:\\Atollo\\Atollo.exe");
    expect(servers[1]?.args).toEqual(["mcp", "github"]);
    expect(JSON.stringify(servers[1])).not.toContain("GITHUB_PERSONAL_ACCESS_TOKEN");
    const playwright = (await screen.findByText("playwright")).closest("li") as HTMLElement;
    fireEvent.click(within(playwright).getByRole("button", { name: "Togli" }));
    await waitFor(() => {
      expect(servers.map((s) => s.name)).toEqual(["github"]);
    });
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
