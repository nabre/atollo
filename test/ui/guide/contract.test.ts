// The Guide's contract: the app draws every widget of guide/core/widgets.ts
// and nothing else, guide/ imports nothing from outside guide/ but React,
// and the chapters carry no classes or styles that only the app's CSS
// knows.
import { describe, expect, it } from "vitest";
import { WIDGET_NAMES } from "@guide/core/widgets";
import { APP_WIDGETS } from "@/guide/register";

const raw = import.meta.glob<string>(["../../../guide/**/*.ts", "../../../guide/**/*.tsx"], {
  query: "?raw",
  import: "default",
  eager: true,
});

/** A relative import of `from` (a glob's key) as a path from the test. */
function resolve(from: string, spec: string): string {
  const parts = from.split("/").slice(0, -1);
  for (const p of spec.split("/")) {
    if (p === "..") parts.pop();
    else if (p !== ".") parts.push(p);
  }
  return parts.join("/");
}

describe("guide contract", () => {
  it("finds the Guide's sources", () => {
    expect(Object.keys(raw).length).toBeGreaterThan(10);
    expect(Object.keys(raw).some((p) => p.includes("/guide/content/"))).toBe(true);
  });

  it("has an app widget for every name of the contract, and no other", () => {
    expect(Object.keys(APP_WIDGETS).sort()).toEqual([...WIDGET_NAMES].sort());
  });

  it("imports nothing from outside guide/ but React", () => {
    for (const [path, code] of Object.entries(raw)) {
      for (const m of code.matchAll(/^\s*(?:import|export)[^;]*?from\s+"([^"]+)"/gm)) {
        const spec = m[1] ?? "";
        const inside = spec.startsWith(".") && resolve(path, spec).startsWith("../../../guide/");
        expect(spec === "react" || inside, `${path}: ${spec}`).toBe(true);
      }
    }
  });

  it("leaves classes and styles to the renderers", () => {
    for (const [path, code] of Object.entries(raw)) {
      expect(code, path).not.toMatch(/\bclassName=|\bstyle=\{/);
    }
  });
});
