// «Account degli agenti» (D-185): one row per agent with the account the
// project uses; choosing another saves it for the project, «Nuovo
// account…» makes one and picks it; Antigravity has none.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { AgentAccountsSection } from "@/projects/accounts";
import type { AgentAccounts } from "@/ipc/agents";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const ALL: AgentAccounts[] = [
  {
    agent: "claude",
    supported: true,
    accounts: [
      { name: "default", signedIn: false, usedBy: ["blog"] },
      { name: "lavoro", signedIn: true, usedBy: ["shop"] },
    ],
  },
  { agent: "codex", supported: true, accounts: [{ name: "default", signedIn: false, usedBy: ["shop"] }] },
  { agent: "antigravity", supported: false, accounts: [{ name: "default", signedIn: false, usedBy: [] }] },
];

function answer() {
  invoke.mockImplementation((cmd: string) => {
    if (cmd === "agent_accounts") return Promise.resolve(ALL);
    if (cmd === "project_agent_accounts") return Promise.resolve({ claude: "lavoro", codex: "default" });
    return Promise.resolve(undefined);
  });
}

describe("AgentAccountsSection", () => {
  it("shows each agent's account and saves another choice", async () => {
    answer();
    render(<AgentAccountsSection project="shop" />);
    const claude =
      await screen.findByLabelText<HTMLElementTagNameMap["vscode-single-select"]>("Account di Claude Code");
    expect(claude.querySelectorAll("vscode-option")).toHaveLength(2);
    expect(claude.value).toBe("lavoro");
    expect(screen.getByText("connesso")).toBeTruthy();
    expect(screen.getByText(/Nessun account separato/)).toBeTruthy();
    fireEvent.change(claude, { target: { value: "default" } });
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_agent_account_set", {
        project: "shop",
        agent: "claude",
        name: "default",
      });
    });
  });

  it("makes a new account and picks it for the project", async () => {
    answer();
    render(<AgentAccountsSection project="shop" />);
    await screen.findByLabelText("Account di Codex");
    const adds = screen.getAllByRole("button", { name: "Nuovo account…" });
    fireEvent.click(adds[1] as HTMLElement);
    fireEvent.change(await screen.findByLabelText("Nome del nuovo account"), { target: { value: " cliente " } });
    fireEvent.click(screen.getByRole("button", { name: "Crea" }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("agent_account_create", { agent: "codex", name: "cliente" });
    });
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("project_agent_account_set", {
        project: "shop",
        agent: "codex",
        name: "cliente",
      });
    });
  });
});
