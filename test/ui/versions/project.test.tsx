// The versions of a project's page (D-106): the languages
// of the project in the first zone, the other installed ones in the second,
// closed; a language the project needs and the kit lacks (p.missing) stays
// in the first zone with what to do, without a line to choose. The text
// above says the versions apply to the web server too.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { ProjectInfo, ProjectVersions as Data, VersionedTool } from "../../../ui/src/api";
import { ProjectVersions } from "../../../ui/src/versions";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => undefined),
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

function tool(id: string, name: string, installed: boolean): VersionedTool {
  return {
    id,
    name,
    installed,
    default: installed ? { line: "1.0", version: "1.0.0" } : null,
    saved: [],
    preferred: null,
    downloadable: true,
    available: ["1.0", "0.9"],
    // Just asked: the page does not ask the source again.
    checked: Math.floor(Date.now() / 1000),
    projects: {},
  };
}

const DATA: Data = {
  tools: [tool("php", "PHP", false), tool("node", "Node.js", true), tool("python", "Python", true)],
  chosen: {},
};

const P = {
  id: "shop",
  name: "shop",
  requires: ["php", "node"],
  missing: ["php"],
} as unknown as ProjectInfo;

function answer() {
  invoke.mockImplementation((cmd) => {
    if (cmd === "project_versions") return Promise.resolve(DATA);
    return Promise.reject(new Error(`unexpected ${cmd}`));
  });
}

describe("ProjectVersions", () => {
  it("keeps a missing language in the project's zone, with what to do instead of a line", async () => {
    answer();
    render(<ProjectVersions p={P} />);
    expect(await screen.findByText("Linguaggi attivi nel progetto (2)")).toBeTruthy();
    expect(
      screen.getByText(
        "Non installato: «Installa ciò che manca» in Componenti lo installa, poi qui se ne sceglie la versione.",
      ),
    ).toBeTruthy();
    // PHP has no list of lines; Node.js, installed, has its own.
    expect(document.getElementById("ver-php")).toBeNull();
    expect(document.getElementById("ver-node")).not.toBeNull();

    // Python, installed and not used by the project, is in the other zone.
    const other = screen.getByRole("button", { name: "Linguaggi non presenti nel progetto (1)" });
    expect(other.getAttribute("aria-expanded")).toBe("false");
    expect(document.getElementById("ver-python")).toBeNull();
    fireEvent.click(other);
    expect(document.getElementById("ver-python")).not.toBeNull();
  });

  it("proposes the line of the project's containers, applied only on request", async () => {
    answer();
    const set = vi.fn();
    invoke.mockImplementation((cmd, args) => {
      if (cmd === "project_versions") return Promise.resolve(DATA);
      if (cmd === "project_versions_set") {
        set(args);
        return Promise.resolve({ ...DATA, chosen: { node: "0.9" } });
      }
      return Promise.reject(new Error(`unexpected ${cmd}`));
    });
    render(<ProjectVersions p={{ ...P, missing: [], lines: { node: "0.9" } }} />);
    expect(
      await screen.findByText(/Dai container del progetto \(Compose o Dockerfile\): Node\.js 0\.9\./),
    ).toBeTruthy();
    expect(set).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Usa 0.9" }));
    await vi.waitFor(() => {
      expect(set).toHaveBeenCalledWith({ project: "shop", id: "node", line: "0.9" });
    });
    // Once chosen, the proposal goes away.
    await vi.waitFor(() => {
      expect(screen.queryByText(/Node\.js 0\.9\./)).toBeNull();
    });
  });

  it("proposes nothing when the container line is the one in use", async () => {
    answer();
    render(<ProjectVersions p={{ ...P, missing: [], lines: { node: "1.0" } }} />);
    expect(await screen.findByText("Linguaggi attivi nel progetto (2)")).toBeTruthy();
    expect(screen.queryByText(/Dai container del progetto/)).toBeNull();
  });

  it("names a container line that is not offered, without an action", async () => {
    answer();
    render(<ProjectVersions p={{ ...P, missing: [], lines: { node: "7.1" } }} />);
    expect(await screen.findByText(/Node\.js 7\.1\. Non è tra le versioni offerte\./)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Usa 7.1" })).toBeNull();
  });

  it("says the versions apply to the project's web server too", async () => {
    answer();
    render(<ProjectVersions p={{ ...P, missing: [] }} />);
    expect(await screen.findByText(/per il server web che lo serve \(Apache o Nginx, con il suo PHP\)/)).toBeTruthy();
    expect(screen.queryByText(/Non installato: «Installa ciò che manca»/)).toBeNull();
    expect(document.getElementById("ver-php")).not.toBeNull();
  });
});
