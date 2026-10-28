// «VM del progetto» (LEVELS.md §8.1, §8.2, §8.5): each sandbox of the
// project with its state pill, its mounts rw/ro and the PC's ports; the
// tools table after «Verifica strumenti»; the four actions, «Ricrea»
// asked in the page first; without the VM, the reason and the notice.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { ProjectVm } from "@/projects/vm";
import type { SandboxReport, VmReport } from "@/ipc/levels";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

function sandbox(over: Partial<SandboxReport>): SandboxReport {
  return {
    name: "atollo-p-shop-0123",
    agent: null,
    state: "running",
    lastUsed: null,
    mounts: [
      { host: "D:\\kit\\projects\\shop", guest: "/d/kit/projects/shop", mode: "rw" },
      { host: "D:\\kit\\data\\apps\\linux-x64", guest: "/d/kit/data/apps/linux-x64", mode: "ro" },
    ],
    hostPorts: [{ port: 3307, what: "mysql shop" }],
    info: null,
    sessions: [],
    tools: null,
    note: null,
    ...over,
  };
}

const REPORT: VmReport = {
  project: "shop",
  vm: "on",
  reason: null,
  sandboxes: [
    sandbox({ sessions: ["shop · bash"] }),
    sandbox({ name: "atollo-a-claude-shop-0123", agent: "claude", state: "stopped", mounts: [], hostPorts: [] }),
    sandbox({ name: "atollo-a-codex-shop-0123", agent: "codex", state: "missing", mounts: [], hostPorts: [] }),
  ],
  github: false,
};

function answer(report: VmReport, more: Record<string, unknown> = {}) {
  invoke.mockImplementation((cmd: string) => {
    if (cmd === "project_vm_report") return Promise.resolve(report);
    if (cmd in more) return Promise.resolve(more[cmd]);
    return Promise.resolve([]);
  });
}

describe("ProjectVm", () => {
  it("lists the project's sandbox, then each agent's, with its state, mounts and ports", async () => {
    answer(REPORT);
    render(<ProjectVm project="shop" />);
    const own = await screen.findByRole("region", { name: "Sandbox del progetto" });
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_vm_report", { project: "shop", check: false });
    });
    const regions = screen
      .getAllByRole("region")
      .map((r) => r.getAttribute("aria-label"))
      .filter((l) => l !== null);
    expect(regions).toEqual(["Sandbox del progetto", "Sandbox di Claude Code", "Sandbox di Codex"]);
    expect(within(own).getByText("accesa").closest("[data-tone]")?.getAttribute("data-tone")).toBe("ready");
    const claude = screen.getByRole("region", { name: "Sandbox di Claude Code" });
    expect(within(claude).getByText("spenta").closest("[data-tone]")?.getAttribute("data-tone")).toBe("off");
    const codex = screen.getByRole("region", { name: "Sandbox di Codex" });
    expect(within(codex).getByText("non c'è")).toBeTruthy();
    const mounts = within(own).getByRole("list", { name: "Cartelle montate" });
    expect(within(mounts).getByText("lettura e scrittura")).toBeTruthy();
    expect(within(mounts).getByText("sola lettura")).toBeTruthy();
    expect(within(mounts).getByText("/d/kit/projects/shop")).toBeTruthy();
    expect(within(own).getByText(/3307 \(mysql shop\)/)).toBeTruthy();
    expect(within(own).getByText(/Sessioni in corso: shop · bash/)).toBeTruthy();
    expect(screen.getByText("GitHub nella VM: no")).toBeTruthy();
  });

  it("shows the tools after «Verifica strumenti», each verdict in words", async () => {
    const checked = sandbox({
      tools: [
        {
          id: "node",
          name: "Node.js",
          chosen: "22",
          command: "node --version",
          path: "/d/kit/data/apps/linux-x64/node/22/bin/node",
          found: "22.11.0",
          verdict: "ok",
          output: "v22.11.0",
        },
        {
          id: "python",
          name: "Python",
          chosen: "3.13",
          command: "python3 --version",
          path: "/d/kit/data/apps/linux-x64/python/3.12/bin/python3",
          found: "3.12.7",
          verdict: "otherLine",
          output: "Python 3.12.7",
        },
        {
          id: "go",
          name: "Go",
          chosen: "1.23",
          command: "go version",
          path: null,
          found: null,
          verdict: "missing",
          output: "",
        },
      ],
    });
    answer(REPORT, { project_vm_check: checked });
    render(<ProjectVm project="shop" />);
    const own = await screen.findByRole("region", { name: "Sandbox del progetto" });
    fireEvent.click(within(own).getByRole("button", { name: "Verifica strumenti" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_vm_check", { project: "shop", agent: null });
    });
    const table = await within(own).findByRole("table", { name: "Strumenti nella VM" });
    const rows = within(table).getAllByRole("row").slice(1);
    expect(rows.map((r) => r.textContent)).toEqual([
      "Node.js2222.11.0✓ come scelto",
      "Python3.133.12.7✗ atteso 3.13, trovato 3.12.7",
      "Go1.23—✗ manca",
    ]);
  });

  it("runs an action on the sandbox it belongs to", async () => {
    answer(REPORT);
    render(<ProjectVm project="shop" />);
    const own = await screen.findByRole("region", { name: "Sandbox del progetto" });
    fireEvent.click(within(own).getByRole("button", { name: "Aggiorna strumenti" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_vm_action", { project: "shop", agent: null, action: "refresh" });
    });
    const claude = screen.getByRole("region", { name: "Sandbox di Claude Code" });
    // No session in it: «Riavvia» runs at once.
    fireEvent.click(within(claude).getByRole("button", { name: "Riavvia" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_vm_action", {
        project: "shop",
        agent: "claude",
        action: "restart",
      });
    });
  });

  it("asks before «Ricrea», saying what is lost, what stays and the sessions it stops", async () => {
    answer(REPORT);
    render(<ProjectVm project="shop" />);
    const own = await screen.findByRole("region", { name: "Sandbox del progetto" });
    fireEvent.click(within(own).getByRole("button", { name: "Ricrea" }));
    const dialog = await screen.findByRole("alertdialog");
    expect(within(dialog).getByText("Ricreare Sandbox del progetto?")).toBeTruthy();
    expect(within(dialog).getByText(/Si perde: la home della VM/)).toBeTruthy();
    expect(within(dialog).getByText(/Resta: il progetto, i suoi database/)).toBeTruthy();
    expect(within(dialog).getByText("Si fermano: shop · bash")).toBeTruthy();
    fireEvent.click(within(dialog).getByRole("button", { name: "Annulla" }));
    await waitFor(() => {
      expect(screen.queryByRole("alertdialog")).toBeNull();
    });
    expect(invoke).not.toHaveBeenCalledWith("project_vm_action", expect.anything());

    fireEvent.click(within(own).getByRole("button", { name: "Ricrea" }));
    const again = await screen.findByRole("alertdialog");
    fireEvent.click(within(again).getByRole("button", { name: "Ricrea" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_vm_action", { project: "shop", agent: null, action: "recreate" });
    });
  });

  it("asks before «Riavvia» when sessions run in the sandbox", async () => {
    answer(REPORT);
    render(<ProjectVm project="shop" />);
    const own = await screen.findByRole("region", { name: "Sandbox del progetto" });
    fireEvent.click(within(own).getByRole("button", { name: "Riavvia" }));
    const dialog = await screen.findByRole("alertdialog");
    expect(within(dialog).getByText("Si fermano: shop · bash")).toBeTruthy();
    expect(invoke).not.toHaveBeenCalledWith("project_vm_action", expect.anything());
    fireEvent.click(within(dialog).getByRole("button", { name: "Riavvia" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_vm_action", { project: "shop", agent: null, action: "restart" });
    });
  });

  it("without the VM: the reason, the notice, no action", async () => {
    answer({ ...REPORT, vm: "off", reason: "Hyper-V è spento." });
    render(<ProjectVm project="shop" />);
    expect(await screen.findByText("Hyper-V è spento.")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Apri l'avviso VM" })).toBeTruthy();
    const own = screen.getByRole("region", { name: "Sandbox del progetto" });
    for (const name of ["Riavvia", "Ricrea", "Aggiorna strumenti", "Verifica strumenti"]) {
      const button = within(own).getByRole("button", { name });
      expect(button.hasAttribute("disabled") || button.getAttribute("aria-disabled") === "true").toBe(true);
    }
  });
});
