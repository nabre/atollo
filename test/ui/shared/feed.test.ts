// A value kept by Rust's events: the events of a frame change it once, a
// component renders only when the part it chose changes, and the first
// read does not undo an event that came before its answer.
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { createFeed, useFeed } from "@/shared/feed";

afterEach(() => {
  cleanup();
});

interface Progress {
  running: boolean;
  percent: number;
}

/** A feed of progress events sent by the test, its first read answered by
 * the test too. */
function progressFeed() {
  let send: (p: Progress) => void = () => undefined;
  let answer: (p: Progress) => void = () => undefined;
  const listen = vi.fn((handler: (p: Progress) => void) => {
    send = handler;
    return Promise.resolve(() => {
      send = () => undefined;
    });
  });
  const feed = createFeed<Progress, Progress>({
    initial: { running: false, percent: 0 },
    listen,
    apply: (_, p) => p,
    read: () =>
      new Promise<Progress>((resolve) => {
        answer = resolve;
      }).then((p) => (current) => (current.running ? current : p)),
  });
  return {
    feed,
    listen,
    send: (p: Progress) => {
      send(p);
    },
    answer: (p: Progress) => {
      answer(p);
    },
  };
}

describe("useFeed", () => {
  it("changes once a frame and renders a part only when it changes", async () => {
    const { feed, send } = progressFeed();
    let whole = 0;
    let part = 0;
    const all = renderHook(() => {
      whole++;
      return useFeed(feed);
    });
    const running = renderHook(() => {
      part++;
      return useFeed(feed, (p) => p.running);
    });
    const [w0, p0] = [whole, part];
    act(() => {
      send({ running: true, percent: 1 });
      send({ running: true, percent: 2 });
      send({ running: true, percent: 3 });
    });
    await waitFor(() => {
      expect(all.result.current.percent).toBe(3);
    });
    expect(whole).toBe(w0 + 1);
    expect(running.result.current).toBe(true);
    expect(part).toBe(p0 + 1);
    act(() => {
      send({ running: true, percent: 50 });
    });
    await waitFor(() => {
      expect(all.result.current.percent).toBe(50);
    });
    expect(part).toBe(p0 + 1);
  });

  it("keeps an event newer than the first read", async () => {
    const { feed, send, answer } = progressFeed();
    const { result } = renderHook(() => useFeed(feed));
    act(() => {
      send({ running: true, percent: 10 });
    });
    await act(async () => {
      answer({ running: false, percent: 0 });
      await Promise.resolve();
    });
    expect(result.current).toEqual({ running: true, percent: 10 });
  });

  it("listens once for every component, and stops with the last", async () => {
    const { feed, listen } = progressFeed();
    const a = renderHook(() => useFeed(feed));
    const b = renderHook(() => useFeed(feed));
    expect(listen).toHaveBeenCalledTimes(1);
    a.unmount();
    b.unmount();
    renderHook(() => useFeed(feed));
    await waitFor(() => {
      expect(listen).toHaveBeenCalledTimes(2);
    });
  });
});
