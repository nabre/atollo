// Work done once a frame: many asks, one flush; and still done when no
// frame comes (a hidden window), after a while.
import { afterEach, describe, expect, it, vi } from "vitest";
import { frameBatch } from "@/shared/frame";

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe("frameBatch", () => {
  it("flushes once at the next frame, however many asks", () => {
    const frames: FrameRequestCallback[] = [];
    vi.stubGlobal("requestAnimationFrame", (cb: FrameRequestCallback) => frames.push(cb));
    vi.stubGlobal("cancelAnimationFrame", () => undefined);
    const flush = vi.fn();
    const batch = frameBatch(flush);
    batch.ask();
    batch.ask();
    batch.ask();
    expect(frames).toHaveLength(1);
    frames[0]?.(0);
    expect(flush).toHaveBeenCalledTimes(1);
    batch.ask();
    expect(frames).toHaveLength(2);
  });

  it("flushes after a while when no frame comes, and not twice", () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    const frames: FrameRequestCallback[] = [];
    vi.stubGlobal("requestAnimationFrame", (cb: FrameRequestCallback) => frames.push(cb));
    vi.stubGlobal("cancelAnimationFrame", () => undefined);
    const flush = vi.fn();
    const batch = frameBatch(flush);
    batch.ask();
    vi.advanceTimersByTime(499);
    expect(flush).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(flush).toHaveBeenCalledTimes(1);
    // The frame that comes late finds nothing to do: it was cancelled.
    batch.ask();
    frames[1]?.(0);
    vi.advanceTimersByTime(1000);
    expect(flush).toHaveBeenCalledTimes(2);
  });

  it("cancel drops the flush asked", () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "requestAnimationFrame", "cancelAnimationFrame"] });
    const flush = vi.fn();
    const batch = frameBatch(flush);
    batch.ask();
    batch.cancel();
    vi.advanceTimersByTime(1000);
    expect(flush).not.toHaveBeenCalled();
  });
});
