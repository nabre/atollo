// The agents' state polled every few seconds: a poll that fails keeps what
// the last one read, so the page and the projects' agent buttons stay.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook } from "@testing-library/react";
import type { AgentInfo } from "../../../ui/src/api";
import { useAgents } from "../../../ui/src/agents";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

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
