// The agents' state polled every few seconds: a poll that fails keeps what
// the last one read, so the page and the projects' agent buttons stay; read
// again at once when the VM goes on or off (`levels-changed`); without the
// VM the buttons are shown disabled with the reason (LEVELS.md §7).
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, renderHook, screen } from "@testing-library/react";
import type { AgentInfo } from "@/ipc";
import { AgentButtons, useAgents } from "@/agents";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

let levelsListener: ((e: { payload: unknown }) => void) | null = null;
vi.mock("@tauri-apps/api/event", () => ({
  listen: (name: string, handler: (e: { payload: unknown }) => void) => {
    if (name === "levels-changed") levelsListener = handler;
    return Promise.resolve(() => undefined);
  },
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
  vi.useRealTimers();
});

const CLAUDE = { id: "claude", name: "Claude Code", installed: true, available: true } as AgentInfo;
const REASON = "Gli agenti girano solo nella VM, che non è attiva: sbx non ha accesso a Docker.";

describe("useAgents", () => {
  it("keeps the agents read when a later poll fails", async () => {
    vi.useFakeTimers();
    invoke.mockResolvedValueOnce([CLAUDE]);
    invoke.mockRejectedValue("not answering");
    const { result } = renderHook(useAgents);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(result.current.agents).toEqual([CLAUDE]);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(5000);
    });
    expect(invoke).toHaveBeenCalledTimes(2);
    expect(result.current.agents).toEqual([CLAUDE]);
  });

  it("reads the agents again when the VM goes on or off", async () => {
    const off = { ...CLAUDE, available: false, unavailableReason: REASON };
    invoke.mockResolvedValueOnce([off]);
    const { result } = renderHook(useAgents);
    await act(async () => {
      await Promise.resolve();
    });
    expect(result.current.agents).toEqual([off]);
    invoke.mockResolvedValueOnce([CLAUDE]);
    await act(async () => {
      levelsListener?.({ payload: { vm: "on", reason: null, agentsEnabled: true } });
      await Promise.resolve();
    });
    expect(invoke).toHaveBeenCalledTimes(2);
    expect(result.current.agents).toEqual([CLAUDE]);
  });
});

describe("AgentButtons", () => {
  it("offers Antigravity on a project, like Pi, in the project's VM (D-179)", () => {
    invoke.mockResolvedValue(null);
    const agents = [
      CLAUDE,
      { id: "codex", name: "Codex", installed: true, available: true },
      { id: "antigravity", name: "Antigravity", installed: true, available: true },
      { id: "pi", name: "Pi", installed: true, available: true },
    ] as AgentInfo[];
    render(<AgentButtons agents={agents} project="shop" />);
    for (const name of ["Claude Code", "Codex", "Antigravity", "Pi"]) {
      expect(screen.getByRole("button", { name: `Apri ${name}` })).toBeTruthy();
    }
    const agy = screen.getByRole("button", { name: "Apri Antigravity" });
    expect(agy.title).toBe("Apre Antigravity nel progetto shop");
    fireEvent.click(agy);
    expect(invoke).toHaveBeenCalledWith("agent_launch", { agent: "antigravity", project: "shop" });
  });

  it("shows every agent disabled with the reason without the VM", () => {
    invoke.mockResolvedValue(null);
    const agents = ["claude", "pi"].map((id) => ({
      id,
      name: id,
      installed: true,
      available: false,
      unavailableReason: REASON,
    })) as AgentInfo[];
    render(<AgentButtons agents={agents} project="shop" />);
    for (const id of ["claude", "pi"]) {
      const button = screen.getByRole<HTMLButtonElement>("button", { name: `Apri ${id}` });
      expect(button.disabled).toBe(true);
      expect(button.title).toBe(REASON);
      fireEvent.click(button);
    }
    expect(invoke).not.toHaveBeenCalledWith("agent_launch", expect.anything());
  });
});
