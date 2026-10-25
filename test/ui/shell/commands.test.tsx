// The command palette (Ctrl+Shift+P, N-8): the kit's actions found by a
// few words, Enter runs the first, a failure is told after it closed.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { SAMPLE_AGENTS, SAMPLE_PROJECTS, SAMPLE_SERVICES, type RunInfo } from "@/ipc";
import type { Running } from "@/shell/activity";
import { CommandPalette } from "@/shell/commands";
import { filterItems } from "@/shared/palette";
import { codicon } from "@/workbench";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

function running(count: number): Running {
  return {
    liveRuns: [],
    lines: [],
    pools: [],
    llm: null,
    count,
    setServices: vi.fn(),
    reloadServices: vi.fn(),
    reloadPools: vi.fn(),
    reloadLlm: vi.fn(),
  };
}

function answers(extra: Record<string, unknown> = {}) {
  invoke.mockImplementation((c) => {
    if (c === "projects_list") return Promise.resolve(SAMPLE_PROJECTS);
    if (c === "services_list") return Promise.resolve(SAMPLE_SERVICES);
    if (c in extra) {
      const v = extra[c];
      return v instanceof Error ? Promise.reject(v) : Promise.resolve(v);
    }
    return Promise.resolve(undefined);
  });
}

function palette(props: Partial<Parameters<typeof CommandPalette>[0]> = {}) {
  const onPage = vi.fn();
  const onError = vi.fn();
  const onClose = vi.fn();
  render(
    <CommandPalette
      pages={[{ id: "databases", label: "Database", icon: codicon("database") }]}
      agents={SAMPLE_AGENTS}
      editor={null}
      runs={{}}
      running={running(0)}
      onPage={onPage}
      onOpenProject={vi.fn()}
      onError={onError}
      onClose={onClose}
      {...props}
    />,
  );
  return { onPage, onError, onClose };
}

const type = (text: string) => {
  const dialog = within(screen.getByRole("dialog", { name: "Comandi" }));
  fireEvent.change(dialog.getByLabelText("Comandi"), { target: { value: text } });
};

describe("command palette", () => {
  it("starts a project found by two words", async () => {
    answers();
    const { onClose } = palette();
    await screen.findByText("Avvia sito-vetrina");
    type("avvia vetrina");
    expect(screen.getAllByRole("option")).toHaveLength(1);
    fireEvent.keyDown(screen.getByRole("dialog", { name: "Comandi" }), { key: "Enter" });
    expect(onClose).toHaveBeenCalled();
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("run_start", expect.objectContaining({ project: "sito-vetrina" }));
    });
  });

  it("stops a running action, a database, everything", async () => {
    answers();
    const runs: Record<string, RunInfo> = {
      "sito-vetrina|static": { key: "sito-vetrina|static", state: "running" } as RunInfo,
    };
    palette({ runs, running: running(2) });
    expect(await screen.findByText("Ferma sito-vetrina")).toBeTruthy();
    expect(screen.queryByText("Avvia sito-vetrina")).toBeNull();
    // MySQL runs in the sample.
    expect(await screen.findByText("Ferma MySQL")).toBeTruthy();
    expect(screen.getByText("Ferma tutto")).toBeTruthy();
  });

  it("has «Ferma tutto» only when something runs, the editor only when installed", async () => {
    answers();
    palette();
    await screen.findByText("Avvia sito-vetrina");
    expect(screen.queryByText("Ferma tutto")).toBeNull();
    expect(screen.queryByText("Apri sito-vetrina nell'editor")).toBeNull();
    // An agent installed, in each project.
    expect(screen.getByText("Claude Code in sito-vetrina")).toBeTruthy();
  });

  it("goes to a page", async () => {
    answers();
    const { onPage } = palette();
    type("vai database");
    fireEvent.click(await screen.findByText("Vai a Database"));
    expect(onPage).toHaveBeenCalledWith("databases");
  });

  it("tells a failure after it closed", async () => {
    answers({ agent_launch: new Error("Claude Code non è installato") });
    const { onError } = palette();
    fireEvent.click(await screen.findByText("Claude Code in sito-vetrina"));
    await waitFor(() => {
      expect(onError).toHaveBeenCalledWith(expect.stringContaining("non è installato"));
    });
  });
});

describe("filterItems", () => {
  it("keeps the items with every word, in any order and case", () => {
    const item = (label: string, search?: string) => ({ key: label, icon: null, label, search, run: () => undefined });
    const items = [item("Avvia sito"), item("Ferma sito"), item("Apri", "apri sito html")];
    expect(filterItems(items, "SITO avvia").map((i) => i.key)).toEqual(["Avvia sito"]);
    expect(filterItems(items, "html").map((i) => i.key)).toEqual(["Apri"]);
    expect(filterItems(items, "  ")).toHaveLength(3);
  });
});
