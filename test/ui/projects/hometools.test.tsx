// The quick panel's «Strumenti»: the components this PC can get as a grid
// of tiles, with the Tools page's states and actions; the app's own parts
// and the required ones left out.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { ComponentInfo, SetupSnapshot, Updates } from "@/ipc";
import type { Frameworks } from "@/frameworks";
import { ProjectsHome, type HomeHandlers } from "@/projects/home";
import { HomeTools, TOOLS_SHOWN } from "@/projects/tools";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));
vi.mock("@tauri-apps/api/event", () => ({
  listen: () => Promise.resolve(() => undefined),
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

function tool(id: string, name: string, group: ComponentInfo["group"], extra: Partial<ComponentInfo> = {}) {
  return {
    id,
    name,
    group,
    desc: `${name}, a tool`,
    mb: 10,
    chosen: false,
    installed: false,
    present: false,
    locked: false,
    version: null,
    available: true,
    unavailableReason: null,
    notNeeded: null,
    needs: [],
    versioned: false,
    home: "https://example.org",
    ...extra,
  } satisfies ComponentInfo;
}

const ON = { chosen: true, installed: true, present: true };

const LIST: ComponentInfo[] = [
  tool("vscodium-server", "Editor", "app", { ...ON, locked: true, version: "1.0" }),
  tool("gh", "GitHub CLI", "required", { ...ON, locked: true, version: "2.83.0" }),
  tool("pwsh", "PowerShell", "required", { ...ON, locked: true, version: "7.5" }),
  tool("conpty", "ConPTY", "languages", { available: false, notNeeded: "il sistema ha la sua console" }),
  tool("claude", "Claude Code", "agents", { ...ON, version: "2.1.299" }),
  tool("node", "Node.js", "languages", { ...ON, version: "24.10.0" }),
  tool("python", "Python", "languages"),
  tool("mongodb", "MongoDB", "databases", { available: false, unavailableReason: "nessuna build per Linux ARM64" }),
];

const UPDATES: Updates = {
  checked: 1_800_000_000,
  items: [{ id: "node", name: "Node.js", installed: "24.10.0", latest: "24.11.1", notes: "" }],
};

const fw = { view: null, pieces: [] } as unknown as Frameworks;

function answer(list: ComponentInfo[]) {
  invoke.mockImplementation((c) =>
    Promise.resolve(c === "components_list" ? list : c === "updates_list" ? UPDATES : undefined),
  );
}

function home() {
  const handlers = { onTool: vi.fn() } as unknown as HomeHandlers;
  render(<ProjectsHome projects={[]} runs={{}} busy={null} status={null} agents={null} fw={fw} handlers={handlers} />);
  return handlers;
}

const tile = (name: string) =>
  screen.getByRole("button", { name: new RegExp(`^${name}:`) }).closest("li") as HTMLElement;

describe("the quick panel's «Strumenti»", () => {
  it("shows the tools as tiles in the groups of the Tools page, without the locked ones", async () => {
    answer(LIST);
    home();
    const section = within(await screen.findByRole("region", { name: "Strumenti" }));
    await section.findByText("Node.js");
    for (const name of ["Editor", "GitHub CLI", "PowerShell"]) expect(section.queryByText(name)).toBeNull();
    expect(section.getByRole("region", { name: "Agenti AI" })).toBeTruthy();
    expect(within(section.getByRole("region", { name: "Database" })).getByText("MongoDB")).toBeTruthy();
    expect(within(tile("Claude Code")).getByText("2.1.299")).toBeTruthy();
    expect(within(tile("Node.js")).getByText("24.10.0 → 24.11.1")).toBeTruthy();
    expect(within(tile("Python")).getByText("non installato")).toBeTruthy();
    expect(within(tile("MongoDB")).getByText("nessuna build per Linux ARM64")).toBeTruthy();
    expect(section.getByText("2 installati su 5")).toBeTruthy();
  });

  it("installs and updates with the Tools page's commands", async () => {
    answer(LIST);
    home();
    await screen.findByText("Python");
    fireEvent.click(within(tile("Python")).getByRole("button", { name: "Installa" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("components_install", { id: "python" });
    });
    await waitFor(() => {
      expect(within(tile("Node.js")).getByRole("button", { name: "Aggiorna ora" })).toBeTruthy();
    });
    fireEvent.click(within(tile("Node.js")).getByRole("button", { name: "Aggiorna ora" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("setup_start", { only: ["node"], updateAll: true });
    });
    // Installed and up to date: nothing to do.
    expect(within(tile("Claude Code")).queryByRole("button", { name: /Installa|Aggiorna/ })).toBeNull();
  });

  it("says «non serve» and «non disponibile» without a button", async () => {
    answer(LIST);
    home();
    await screen.findByText("ConPTY");
    const conpty = within(tile("ConPTY"));
    expect(conpty.getByText("non serve")).toBeTruthy();
    expect(conpty.getAllByRole("button")).toHaveLength(1);
    expect(within(tile("MongoDB")).getAllByRole("button")).toHaveLength(1);
  });

  it("opens the tool's page on Tools from the tile", async () => {
    answer(LIST);
    const handlers = home();
    await screen.findByText("Python");
    fireEvent.click(screen.getByRole("button", { name: /^Python: non installato/ }));
    expect(handlers.onTool).toHaveBeenCalledWith("python");
    expect(invoke).not.toHaveBeenCalledWith("components_install", expect.anything());
  });

  it("folds a long list under «Mostra tutti»", async () => {
    const many = Array.from({ length: TOOLS_SHOWN + 3 }, (_, i) =>
      tool(`t${String(i).padStart(2, "0")}`, `Tool ${String(i).padStart(2, "0")}`, "languages"),
    );
    answer(many);
    home();
    await screen.findByText("Tool 00");
    expect(screen.queryByText(`Tool ${String(TOOLS_SHOWN)}`)).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: `Mostra tutti (${String(many.length)})` }));
    expect(screen.getByText(`Tool ${String(TOOLS_SHOWN + 2)}`)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Mostra meno" }));
    expect(screen.queryByText(`Tool ${String(TOOLS_SHOWN)}`)).toBeNull();
  });
});

/** A preparation running: `steps` so far, `queue` of this run, `pending`
 * runs after it. */
function running(extra: Partial<SetupSnapshot> = {}): SetupSnapshot {
  return {
    running: true,
    cancelling: false,
    automatic: false,
    target: "win32-x64",
    total: 1,
    steps: [],
    queue: [],
    error: null,
    job: "job-1",
    pending: [],
    ...extra,
  };
}

const PYTHON_STEP = { id: "python", name: "Python", note: null, done: 40, size: 100, outcome: null };

describe("the tiles while tools install (fluid installs, §8.6)", () => {
  it("shows a tile's own download with its bar, and keeps the other tiles' buttons", async () => {
    answer(LIST);
    render(<HomeTools setup={running({ steps: [PYTHON_STEP] })} onOpen={() => undefined} />);
    await screen.findByText("Python");
    const python = within(tile("Python"));
    expect(python.getByText(/download 40%/)).toBeTruthy();
    expect(python.getByRole("progressbar", { name: "Download di Python" }).getAttribute("aria-valuenow")).toBe("40");
    expect(python.queryByRole("button", { name: "Installa" })).toBeNull();
    // Another tile is not held by Python's install: it can be asked now.
    const update = within(tile("Node.js")).getByRole("button", { name: "Aggiorna ora" });
    expect(update.hasAttribute("disabled")).toBe(false);
  });

  it("says «in coda» for a tool asked after the one running, and cancels it alone", async () => {
    answer(LIST);
    render(
      <HomeTools
        setup={running({
          steps: [{ ...PYTHON_STEP, id: "node", name: "Node.js" }],
          pending: [{ job: "job-2", title: "Installo Python", items: ["python"] }],
        })}
        onOpen={() => undefined}
      />,
    );
    await screen.findByText("Python");
    const python = within(tile("Python"));
    expect(python.getByText(/in coda/)).toBeTruthy();
    fireEvent.click(python.getByRole("button", { name: "Annulla Python" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("job_cancel", { id: "job-2" });
    });
    // The running one is cancelled with its own job.
    fireEvent.click(within(tile("Node.js")).getByRole("button", { name: "Annulla Node.js" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("job_cancel", { id: "job-1" });
    });
  });

  it("offers «Riprova» on a tile whose install failed", async () => {
    answer(LIST);
    const failed = {
      ...PYTHON_STEP,
      outcome: { kind: "failed" as const, message: "checksum", byHand: null },
    };
    render(<HomeTools setup={{ ...running({ steps: [failed] }), running: false }} onOpen={() => undefined} />);
    await screen.findByText("Python");
    fireEvent.click(within(tile("Python")).getByRole("button", { name: "Riprova" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("components_install", { id: "python" });
    });
  });

  it("reads again only the tile whose install ended: the grid is not made again", async () => {
    answer(LIST);
    const { rerender } = render(<HomeTools setup={running({ steps: [PYTHON_STEP] })} onOpen={() => undefined} />);
    await screen.findByText("Python");
    const grid = tile("Python").closest("ul");
    const nodeTile = tile("Node.js");
    invoke.mockImplementation((c, args) =>
      Promise.resolve(
        c === "components_list"
          ? (args as { ids?: string[] } | undefined)?.ids
            ? [tool("python", "Python", "languages", { ...ON, version: "3.13.1" })]
            : LIST
          : c === "updates_list"
            ? UPDATES
            : undefined,
      ),
    );
    const done = { ...PYTHON_STEP, outcome: { kind: "installed" as const, version: "3.13.1" } };
    rerender(<HomeTools setup={running({ steps: [done] })} onOpen={() => undefined} />);
    await waitFor(() => {
      expect(within(tile("Python")).getByText("3.13.1")).toBeTruthy();
    });
    expect(invoke).toHaveBeenCalledWith("components_list", { ids: ["python"] });
    const whole = invoke.mock.calls.filter(([c, a]) => c === "components_list" && a === undefined);
    expect(whole).toHaveLength(1);
    // The same list and the same tiles: nothing remounted, no scroll lost.
    expect(tile("Python").closest("ul")).toBe(grid);
    expect(tile("Node.js")).toBe(nodeTile);
  });
});
