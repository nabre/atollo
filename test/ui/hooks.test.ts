import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook } from "@testing-library/react";
import { useAction, useLoad, type LoadOptions } from "../../ui/src/hooks";

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

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe("useLoad", () => {
  it("reads on mount and keeps the value", async () => {
    const load = vi.fn(() => Promise.resolve(3));
    const { result } = renderHook(() => useLoad(load, 0, { onError: "keep" }));
    expect(result.current.data).toBe(0);
    await settle();
    expect(load).toHaveBeenCalledTimes(1);
    expect(result.current.data).toBe(3);
    expect(result.current.error).toBeNull();
  });

  it("keeps the value on error with keep", async () => {
    let fail = false;
    const load = vi.fn(() => (fail ? Promise.reject(new Error("no")) : Promise.resolve(1)));
    const { result } = renderHook(() => useLoad(load, 0, { onError: "keep" }));
    await settle();
    fail = true;
    act(() => {
      result.current.reload();
    });
    await settle();
    expect(result.current.data).toBe(1);
    expect(result.current.error).toBeNull();
  });

  it("sets the fallback on error", async () => {
    const load = vi.fn(() => Promise.reject(new Error("no")));
    const { result } = renderHook(() => useLoad<number[] | null>(load, null, { onError: { fallback: [] } }));
    await settle();
    expect(result.current.data).toEqual([]);
    expect(result.current.error).toBeNull();
  });

  it("shows the error, keeps the value, clears it on the next success", async () => {
    let fail = false;
    const load = vi.fn(() => (fail ? Promise.reject(new Error("no")) : Promise.resolve(1)));
    const { result } = renderHook(() => useLoad(load, 0, { onError: "show" }));
    await settle();
    fail = true;
    act(() => {
      result.current.reload();
    });
    await settle();
    expect(result.current.data).toBe(1);
    expect(result.current.error).toBe("Error: no");
    fail = false;
    act(() => {
      result.current.reload();
    });
    await settle();
    expect(result.current.error).toBeNull();
  });

  it("retry resolves to whether the read worked", async () => {
    let fail = true;
    const load = vi.fn(() => (fail ? Promise.reject(new Error("no")) : Promise.resolve(1)));
    const { result } = renderHook(() => useLoad(load, 0, { onError: "show" }));
    await settle();
    let ok: boolean | undefined;
    await act(async () => {
      ok = await result.current.retry();
    });
    expect(ok).toBe(false);
    expect(result.current.error).toBe("Error: no");
    fail = false;
    await act(async () => {
      ok = await result.current.retry();
    });
    expect(ok).toBe(true);
    expect(result.current.data).toBe(1);
    expect(result.current.error).toBeNull();
  });

  it("reads again when load or key change, not when the policy does", async () => {
    const a = vi.fn(() => Promise.resolve("a"));
    const b = vi.fn(() => Promise.resolve("b"));
    const { result, rerender } = renderHook(
      ({ load, key, onError }: { load: () => Promise<string>; key: number; onError: LoadOptions<string>["onError"] }) =>
        useLoad(load, "", { onError, key }),
      { initialProps: { load: a, key: 0, onError: "keep" } },
    );
    await settle();
    rerender({ load: a, key: 0, onError: "show" });
    await settle();
    expect(a).toHaveBeenCalledTimes(1);
    rerender({ load: a, key: 1, onError: "show" });
    await settle();
    expect(a).toHaveBeenCalledTimes(2);
    rerender({ load: b, key: 1, onError: "show" });
    await settle();
    expect(b).toHaveBeenCalledTimes(1);
    expect(result.current.data).toBe("b");
  });

  it("reads nothing while disabled", async () => {
    const load = vi.fn(() => Promise.resolve(1));
    const { result, rerender } = renderHook(({ enabled }) => useLoad(load, 0, { onError: "keep", enabled }), {
      initialProps: { enabled: false },
    });
    act(() => {
      result.current.reload();
    });
    await settle();
    expect(load).not.toHaveBeenCalled();
    rerender({ enabled: true });
    await settle();
    expect(load).toHaveBeenCalledTimes(1);
    expect(result.current.data).toBe(1);
  });

  it("reads every few milliseconds when asked", () => {
    vi.useFakeTimers();
    const load = vi.fn(() => Promise.resolve(1));
    const { unmount } = renderHook(() => useLoad(load, 0, { onError: "keep", every: 1000 }));
    expect(load).toHaveBeenCalledTimes(1);
    act(() => {
      vi.advanceTimersByTime(2500);
    });
    expect(load).toHaveBeenCalledTimes(3);
    unmount();
    act(() => {
      vi.advanceTimersByTime(5000);
    });
    expect(load).toHaveBeenCalledTimes(3);
  });

  it("setData changes the value between reads", async () => {
    const { result } = renderHook(() => useLoad(() => Promise.resolve(1), 0, { onError: "keep" }));
    await settle();
    act(() => {
      result.current.setData((n) => n + 1);
    });
    expect(result.current.data).toBe(2);
  });

  it("sets nothing once unmounted", async () => {
    const answer = deferred<number>();
    const failure = deferred<number>();
    const seen: number[] = [];
    const ok = renderHook(() => {
      const r = useLoad(() => answer.promise, 0, { onError: "keep" });
      seen.push(r.data);
      return r;
    });
    const ko = renderHook(() => useLoad(() => failure.promise, 0, { onError: { fallback: 9 } }));
    const renders = seen.length;
    ok.unmount();
    ko.unmount();
    answer.resolve(1);
    failure.reject(new Error("no"));
    await settle();
    expect(seen.length).toBe(renders);
    expect(ok.result.current.data).toBe(0);
    expect(ko.result.current.data).toBe(0);
  });

  it("ignores out-of-order responses from earlier loads", async () => {
    const first = deferred<number>();
    const second = deferred<number>();
    let call = 0;
    const load = vi.fn(() => (++call === 1 ? first.promise : second.promise));
    const { result } = renderHook(() => useLoad(load, 0, { onError: "show" }));
    act(() => {
      result.current.reload();
    });
    second.resolve(2);
    await settle();
    expect(result.current.data).toBe(2);
    first.resolve(1);
    await settle();
    expect(result.current.data).toBe(2);
  });
});

describe("useAction", () => {
  it("keeps busy until the latest overlapping action settles", async () => {
    const first = deferred<undefined>();
    const second = deferred<undefined>();
    const { result } = renderHook(() => useAction());
    act(() => {
      result.current.run("first", () => first.promise);
    });
    expect(result.current.busy).toBe("first");
    act(() => {
      result.current.run("second", () => second.promise);
    });
    expect(result.current.busy).toBe("second");
    first.resolve(undefined);
    await settle();
    expect(result.current.busy).toBe("second");
    second.resolve(undefined);
    await settle();
    expect(result.current.busy).toBeNull();
  });
});
