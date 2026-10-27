// One poll per command for the whole window: answers with the same data
// render nothing, the parts that show it share one read at the pace of the
// fastest, the timer stops while the window is hidden, and the answer of an
// older read never replaces a newer one.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook } from "@testing-library/react";
import { createPoll, same, usePoll } from "@/shared/poll";

/** A promise settled by the test. */
function deferred<T>() {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

const settle = () => act(async () => {});

/** The window minimized (true) or shown again, as the WebView tells it. */
function setHidden(hidden: boolean) {
  Object.defineProperty(document, "hidden", { configurable: true, get: () => hidden });
  act(() => {
    document.dispatchEvent(new Event("visibilitychange"));
  });
}

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  setHidden(false);
});

describe("same", () => {
  it("compares IPC answers by their data", () => {
    expect(same({ a: [1, { b: null }] }, { a: [1, { b: null }] })).toBe(true);
    expect(same({ a: [1] }, { a: [2] })).toBe(false);
    expect(same(null, {})).toBe(false);
    expect(same(3, 3)).toBe(true);
    expect(same([], null)).toBe(false);
  });
});

describe("usePoll", () => {
  it("shares one read between the parts, and renders nothing when the data is the same", async () => {
    const load = vi.fn(() => Promise.resolve({ items: [1, 2] }));
    const poll = createPoll(load);
    let renders = 0;
    const a = renderHook(() => {
      renders++;
      return usePoll(poll, null, { onError: "keep" });
    });
    const b = renderHook(() => usePoll(poll, null, { onError: "keep" }));
    await settle();
    // The second part found a read on its way: no other.
    expect(load).toHaveBeenCalledTimes(1);
    expect(a.result.current.data).toEqual({ items: [1, 2] });
    expect(b.result.current.data).toBe(a.result.current.data);
    const before = renders;
    const first = a.result.current.data;
    act(() => {
      b.result.current.reload();
    });
    await settle();
    expect(load).toHaveBeenCalledTimes(2);
    expect(a.result.current.data).toBe(first);
    expect(renders).toBe(before);
  });

  it("reads at the pace of the fastest part, stops with the last one and forgets", () => {
    vi.useFakeTimers();
    const load = vi.fn(() => Promise.resolve(1));
    const poll = createPoll(load);
    const slow = renderHook(() => usePoll(poll, 0, { onError: "keep", every: 4000 }));
    expect(load).toHaveBeenCalledTimes(1);
    return (async () => {
      await act(async () => {
        await vi.advanceTimersByTimeAsync(4000);
      });
      expect(load).toHaveBeenCalledTimes(2);
      const fast = renderHook(() => usePoll(poll, 0, { onError: "keep", every: 1000 }));
      expect(load).toHaveBeenCalledTimes(3);
      await act(async () => {
        await vi.advanceTimersByTimeAsync(3000);
      });
      expect(load).toHaveBeenCalledTimes(6);
      fast.unmount();
      await act(async () => {
        await vi.advanceTimersByTimeAsync(3000);
      });
      expect(load).toHaveBeenCalledTimes(6);
      await act(async () => {
        await vi.advanceTimersByTimeAsync(1000);
      });
      expect(load).toHaveBeenCalledTimes(7);
      slow.unmount();
      await act(async () => {
        await vi.advanceTimersByTimeAsync(20000);
      });
      expect(load).toHaveBeenCalledTimes(7);
      expect(poll.get().value).toBeUndefined();
    })();
  });

  it("sends no timer read while one is on its way", async () => {
    vi.useFakeTimers();
    const slow = deferred<number>();
    const load = vi.fn(() => slow.promise);
    const poll = createPoll(load);
    renderHook(() => usePoll(poll, 0, { onError: "keep", every: 1000 }));
    await act(async () => {
      await vi.advanceTimersByTimeAsync(5000);
    });
    expect(load).toHaveBeenCalledTimes(1);
    slow.resolve(1);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(load).toHaveBeenCalledTimes(2);
  });

  it("stops while the window is hidden and reads at once when it shows", async () => {
    vi.useFakeTimers();
    const load = vi.fn(() => Promise.resolve(1));
    const poll = createPoll(load);
    renderHook(() => usePoll(poll, 0, { onError: "keep", every: 1000 }));
    await settle();
    setHidden(true);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(10000);
    });
    expect(load).toHaveBeenCalledTimes(1);
    setHidden(false);
    expect(load).toHaveBeenCalledTimes(2);
  });

  it("drops the answer of a read older than a refresh or an action's answer", async () => {
    const reads = [deferred<string>(), deferred<string>(), deferred<string>()];
    let call = 0;
    const poll = createPoll(() => reads[call++]?.promise ?? Promise.resolve("?"));
    const { result } = renderHook(() => usePoll(poll, "", { onError: "show" }));
    let fresh: Promise<boolean> = Promise.resolve(false);
    act(() => {
      fresh = result.current.retry();
    });
    reads[1]?.resolve("new");
    await settle();
    expect(await fresh).toBe(true);
    reads[0]?.resolve("old");
    await settle();
    expect(result.current.data).toBe("new");
    act(() => {
      result.current.reload();
    });
    act(() => {
      result.current.setData("action");
    });
    reads[2]?.reject(new Error("late"));
    await settle();
    expect(result.current.data).toBe("action");
    expect(result.current.error).toBeNull();
  });

  it("shows the error only when asked, and clears it on a read that works", async () => {
    let fail = true;
    const poll = createPoll(() => (fail ? Promise.reject(new Error("no")) : Promise.resolve(1)));
    const shown = renderHook(() => usePoll(poll, 0, { onError: "show" }));
    const kept = renderHook(() => usePoll(poll, 0, { onError: "keep" }));
    await settle();
    expect(shown.result.current.error).toBe("Error: no");
    expect(kept.result.current.error).toBeNull();
    fail = false;
    await act(async () => {
      await shown.result.current.retry();
    });
    expect(shown.result.current.error).toBeNull();
    expect(kept.result.current.data).toBe(1);
  });
});
