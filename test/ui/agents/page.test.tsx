// The agents' state polled every few seconds: a poll that fails keeps what
// the last one read, so the page and the projects' agent buttons stay.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, render, renderHook, screen } from "@testing-library/react";
import type { AgentInfo } from "@/ipc";
import { AgentButtons, useAgents } from "@/agents";
import { invoke } from "@test/support/tauri";

vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri-core"));

afterEach(() => {
  cleanup();
  invoke.mockReset();
  vi.useRealTimers();
});

const CLAUDE = { id: "claude", name: "Claude Code", installed: true } as AgentInfo;

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
});

describe("AgentButtons", () => {
  it("never offers Antigravity on a project: it cannot run in the VM (D-178)", () => {
    const agents = [
      CLAUDE,
      { id: "codex", name: "Codex", installed: true },
      { id: "antigravity", name: "Antigravity", installed: true },
      { id: "pi", name: "Pi", installed: true },
    ] as AgentInfo[];
    render(<AgentButtons agents={agents} project="shop" />);
    for (const name of ["Claude Code", "Codex", "Pi"]) {
      expect(screen.getByRole("button", { name: `Apri ${name}` })).toBeTruthy();
    }
    expect(screen.queryByRole("button", { name: "Apri Antigravity" })).toBeNull();
  });
});
