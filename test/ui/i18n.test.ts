// Every text passed to t() in the sources is in the English, French and
// German catalogs, with the same {n} placeholders, and every catalog key is
// still used. The sources are read through Vite's import.meta.glob (raw), so
// the test needs no Node types.
import { describe, expect, it } from "vitest";
import { CATALOGS } from "../../ui/src/i18n";

const files = import.meta.glob<string>(["../../ui/src/**/*.ts", "../../ui/src/**/*.tsx", "!../../ui/src/i18n.ts"], {
  query: "?raw",
  import: "default",
  eager: true,
});

/** The double-quoted literals passed as first argument of t(…), unescaped. */
function sources(code: string): string[] {
  const out: string[] = [];
  for (const m of code.matchAll(/\bt\(\s*"((?:[^"\\\n]|\\.)*)"/g)) {
    out.push(JSON.parse(`"${m[1] ?? ""}"`) as string);
  }
  return out;
}

function placeholders(text: string): string[] {
  return [...new Set([...text.matchAll(/\{(\d+)\}/g)].map((m) => m[1] ?? ""))].sort();
}

const used = new Set(Object.values(files).flatMap(sources));

describe("i18n catalogs", () => {
  it("finds the texts of the sources", () => {
    expect(Object.keys(files)).toContain("../../ui/src/App.tsx");
    expect(sources('t(\n  "A \\"b\\" {0}",\n x)')).toEqual(['A "b" {0}']);
    expect(used.size).toBeGreaterThan(100);
  });

  for (const [lang, catalog] of Object.entries(CATALOGS)) {
    it(`${lang}: every text is translated, with the same placeholders`, () => {
      const problems: string[] = [];
      for (const source of used) {
        const text = catalog[source];
        if (text === undefined) problems.push(`missing: ${source}`);
        else if (!text.trim()) problems.push(`empty: ${source}`);
        else if (placeholders(text).join() !== placeholders(source).join()) problems.push(`placeholders: ${source}`);
      }
      expect(problems).toEqual([]);
    });

    it(`${lang}: every key is used`, () => {
      expect(Object.keys(catalog).filter((k) => !used.has(k))).toEqual([]);
    });
  }
});
