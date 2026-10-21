// The terminal's monospace font (font.ts, D-149): a view opens once Geist
// Mono's faces are loaded, and one opened on a fallback measures again.
import { afterEach, describe, expect, it, vi } from "vitest";
import { monoLoaded, monoReady, remeasured, whenMonoLoads } from "../../../ui/src/terminal/font";

const FAMILY = '"Geist Mono Variable", "Geist Mono", ui-monospace, Consolas, monospace';

/** document.fonts as the browser's: faces that load when asked. */
function fakeFonts(loads: boolean) {
  let loaded = false;
  const listeners = new Set<() => void>();
  const asked: string[] = [];
  const fonts = {
    check: () => loaded,
    load: (font: string, text: string) => {
      asked.push(`${font} | ${text}`);
      if (loads) loaded = true;
      return Promise.resolve([]);
    },
    addEventListener: (_: string, h: () => void) => listeners.add(h),
    removeEventListener: (_: string, h: () => void) => listeners.delete(h),
  };
  Object.defineProperty(document, "fonts", { value: fonts, configurable: true });
  return {
    asked,
    listeners,
    arrive() {
      loaded = true;
      listeners.forEach((h) => {
        h();
      });
    },
  };
}

afterEach(() => {
  Reflect.deleteProperty(document, "fonts");
  vi.useRealTimers();
});

describe("terminal font", () => {
  it("waits for nothing without the font loading API", async () => {
    expect(monoLoaded(FAMILY)).toBe(true);
    await expect(monoReady(FAMILY)).resolves.toBe(true);
  });

  it("loads normal and bold with the letters and the box drawing", async () => {
    const fake = fakeFonts(true);
    expect(monoLoaded(FAMILY)).toBe(false);
    await expect(monoReady(FAMILY)).resolves.toBe(true);
    expect(fake.asked).toHaveLength(2);
    expect(fake.asked[0]).toMatch(/^13px "Geist Mono Variable"/);
    expect(fake.asked[1]).toMatch(/^bold 13px /);
    expect(fake.asked[0]).toContain("─");
  });

  it("gives up after the cap, then tells when the faces arrive", async () => {
    vi.useFakeTimers();
    const fake = fakeFonts(false);
    const ready = monoReady(FAMILY, 100);
    await vi.advanceTimersByTimeAsync(100);
    await expect(ready).resolves.toBe(false);
    const arrived = vi.fn();
    whenMonoLoads(FAMILY, arrived);
    fake.arrive();
    expect(arrived).toHaveBeenCalledOnce();
    expect(fake.listeners.size).toBe(0);
  });

  it("writes the stack another way, so xterm.js measures again", () => {
    const once = remeasured(FAMILY);
    expect(once).not.toBe(FAMILY);
    expect(once.trim()).toBe(FAMILY);
    expect(remeasured(once)).toBe(FAMILY);
  });
});
