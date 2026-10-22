// The kit's components behind the frameworks' tags, asked once per session:
// a read that fails is asked again by the next page, not kept as empty.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, renderHook, waitFor } from "@testing-library/react";
import type { ComponentInfo } from "../../../ui/src/api";
import { useComponents } from "../../../ui/src/frameworks/model";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  isTauri: () => true,
}));

afterEach(() => {
  cleanup();
  invoke.mockReset();
});

const NODE = { id: "node", name: "Node.js" } as ComponentInfo;

describe("useComponents", () => {
  it("asks again after a failed read, then keeps the answer", async () => {
    invoke.mockRejectedValueOnce(new Error("not ready"));
    const first = renderHook(useComponents);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledTimes(1);
    });
    expect(first.result.current).toEqual([]);
    first.unmount();

    invoke.mockResolvedValue([NODE]);
    const second = renderHook(useComponents);
    await waitFor(() => {
      expect(second.result.current).toEqual([NODE]);
    });
    second.unmount();

    const third = renderHook(useComponents);
    await waitFor(() => {
      expect(third.result.current).toEqual([NODE]);
    });
    expect(invoke).toHaveBeenCalledTimes(2);
  });
});
