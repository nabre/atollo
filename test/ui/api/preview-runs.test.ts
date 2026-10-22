// The projects' actions in the browser preview: a start goes from «starting»
// to «running» on a port with its output, a stop ends it, and the page hears
// it through the preview's "run" events.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { previewEmit } from "../../../ui/src/api/core";
import { previewLog, previewRuns, previewStart, previewStop } from "../../../ui/src/api/preview-runs";

describe("preview runs", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("starts a sample project's action, then stops it", () => {
    const first = previewStart("sito-vetrina", "static");
    expect(first.state).toBe("starting");
    expect(first.label).toBe("sito statico (cartella del progetto)");
    expect(first.port).toBeGreaterThanOrEqual(7100);
    expect(first.url).toBe(`http://localhost:${String(first.port)}/`);
    expect(previewLog(first.key).lines[0]).toBe(`> Atollo serve . ${String(first.port)}`);
    // Started again while it runs: the same run, not a second one.
    expect(previewStart("sito-vetrina", "static")).toBe(first);

    vi.advanceTimersByTime(800);
    const running = previewRuns().find((r) => r.key === first.key);
    expect(running?.state).toBe("running");
    const log = previewLog(first.key);
    expect(log.lines.some((l) => l.includes("File server su"))).toBe(true);
    // Numbered as in Rust: the seq of a read covers its lines.
    expect(log.seq).toBeGreaterThanOrEqual(log.lines.length);

    previewStop(first.key);
    const stopped = previewRuns().find((r) => r.key === first.key);
    expect(stopped?.state).toBe("stopped");
    expect(stopped?.end).not.toBeNull();
    // A new start keeps the port, as the kit does.
    expect(previewStart("sito-vetrina", "static").port).toBe(first.port);
  });

  it("fills the port in a command action", () => {
    const run = previewStart("api-python", "py:uvicorn");
    expect(run.command).toContain(`--port ${String(run.port)}`);
    expect(run.command).not.toContain("{port}");
  });

  it("emits nothing without listeners", () => {
    expect(() => {
      previewEmit("run", { kind: "log", key: "x", lines: [], seq: 0 });
    }).not.toThrow();
  });
});
