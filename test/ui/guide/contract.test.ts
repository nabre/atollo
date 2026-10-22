// The Guide's contract: the app draws every widget of core/widgets.ts and
// nothing else, the chapters and the core import nothing from the app, and
// the chapters carry no classes or styles that only the app's CSS knows.
import { describe, expect, it } from "vitest";
import { WIDGET_NAMES } from "@/guide/core/widgets";
import { APP_WIDGETS } from "@/guide/register";

const raw = import.meta.glob<string>(["../../../ui/src/guide/core/**/*.ts", "../../../ui/src/guide/topics/**/*.tsx"], {
  query: "?raw",
  import: "default",
  eager: true,
});

describe("guide contract", () => {
  it("finds the Guide's sources", () => {
    expect(Object.keys(raw).length).toBeGreaterThan(10);
  });

  it("has an app widget for every name of the contract, and no other", () => {
    expect(Object.keys(APP_WIDGETS).sort()).toEqual([...WIDGET_NAMES].sort());
  });

  it("imports nothing from the app in the chapters and the core", () => {
    for (const [path, code] of Object.entries(raw)) {
      for (const m of code.matchAll(/^\s*(?:import|export)[^;]*?from\s+"([^"]+)"/gm)) {
        const spec = m[1] ?? "";
        const allowed = spec === "react" || spec.startsWith("./") || /^\.\.\/(core|model|sources)\b/.test(spec);
        expect(allowed, `${path}: ${spec}`).toBe(true);
      }
    }
  });

  it("leaves classes and styles to the renderers", () => {
    for (const [path, code] of Object.entries(raw)) {
      expect(code, path).not.toMatch(/\bclassName=|\bstyle=\{/);
    }
  });
});
