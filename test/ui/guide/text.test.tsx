// The Guide's text: every topic has its text and renders in the four
// languages, no draft text (L) is left, and every tr() has its four
// languages with the same placeholders.
import { render } from "@testing-library/react";
import { afterAll, describe, expect, it } from "vitest";
import { setLang, type Lang } from "@/i18n";
import "@/guide/register";
import { GUIDE_GROUPS, GUIDE_TOPICS } from "@guide/core/model";
import { GuideTopicContent, TOPIC_TEXTS } from "@/guide/sections";

const raw = import.meta.glob<string>(["../../../guide/**/*.ts", "../../../guide/**/*.tsx"], {
  query: "?raw",
  import: "default",
  eager: true,
});

/** The sources without their comments, which may name L() and tr(). */
const files = Object.fromEntries(
  Object.entries(raw).map(([path, code]) => [path, code.replace(/\/\*[\s\S]*?\*\//g, "").replace(/^\s*\/\/.*$/gm, "")]),
);

// A string literal as prettier writes it: in double quotes, in single
// quotes when the text has double quotes, or a template (the sources table).
const STRING = String.raw`"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|${"`"}(?:[^${"`"}\\]|\\.)*${"`"}`;

function unquote(literal: string): string {
  if (literal.startsWith('"')) return JSON.parse(literal) as string;
  if (literal.startsWith("`")) return literal.slice(1, -1);
  const body = literal.slice(1, -1).replace(/\\'/g, "'").replace(/"/g, '\\"');
  return JSON.parse(`"${body}"`) as string;
}
const TR = new RegExp(
  String.raw`\btr\(\s*(${STRING})\s*,\s*(${STRING})\s*,\s*(${STRING})\s*,\s*(${STRING})\s*,?\s*\)`,
  "g",
);

function placeholders(text: string): string {
  return [...text.matchAll(/\{(\w+)\}/g)]
    .map((m) => m[1] ?? "")
    .sort()
    .join(",");
}

describe("guide text", () => {
  afterAll(() => {
    setLang("it");
  });

  it("has a text for every topic and a topic for every text, every chapter with topics", () => {
    expect(Object.keys(TOPIC_TEXTS).sort()).toEqual(GUIDE_TOPICS.map((x) => x.id).sort());
    for (const g of GUIDE_GROUPS)
      expect(
        GUIDE_TOPICS.some((x) => x.group === g.id),
        g.id,
      ).toBe(true);
  });

  it("reads the Guide's sources", () => {
    // guide/core and guide/content: a glob that finds nothing would pass
    // every check below.
    expect(Object.keys(files).length).toBeGreaterThan(10);
    expect(Object.keys(files).some((p) => p.endsWith("/core/lang.ts"))).toBe(true);
  });

  it("leaves no draft text", () => {
    for (const [path, code] of Object.entries(files)) {
      const calls = [...code.matchAll(/\bL\(/g)].length;
      const definition = path.endsWith("/core/lang.ts") ? 1 : 0;
      expect(calls, path).toBe(definition);
    }
  });

  it("gives every tr() four languages with the same placeholders", () => {
    for (const [path, code] of Object.entries(files)) {
      const all = [...code.matchAll(/\btr\(/g)].length - (path.endsWith("/core/lang.ts") ? 1 : 0);
      const full = [...code.matchAll(TR)];
      expect(full.length, `${path}: tr() without four string literals`).toBe(all);
      for (const m of full) {
        const texts = m.slice(1, 5).map((s) => unquote(s));
        expect(
          texts.every((s) => s.trim() !== ""),
          `${path}: ${texts[0] ?? ""}`,
        ).toBe(true);
        const ph = placeholders(texts[0] ?? "");
        for (const s of texts) expect(placeholders(s), `${path}: ${texts[0] ?? ""}`).toBe(ph);
      }
    }
  });

  it.each(["it", "en", "fr", "de"] as Lang[])("renders every topic in %s", (lang) => {
    setLang(lang);
    for (const topic of GUIDE_TOPICS) {
      const { container, unmount } = render(
        <>{GuideTopicContent({ topicId: topic.id, info: null, components: null })}</>,
      );
      expect(container.textContent.length, topic.id).toBeGreaterThan(200);
      unmount();
    }
  });
});
