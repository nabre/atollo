import { describe, expect, it } from "vitest";
import vscodeCss from "./theme-vscode.css?raw";
import tokensCss from "./theme-colors.css?raw";

// The components' sources (one module per file, as the app imports them).
const sources = import.meta.glob<string>(
  ["../../node_modules/@vscode-elements/elements/dist/**/*.js", "!**/bundled.js"],
  { query: "?raw", import: "default", eager: true },
);

/** Name -> value of the custom properties declared in a piece of CSS. */
function declared(css: string) {
  const out = new Map<string, string>();
  for (const m of css.matchAll(/(--[\w\\.-]+):\s*([^;]+);/g)) {
    out.set((m[1] ?? "").replaceAll("\\.", "."), (m[2] ?? "").trim());
  }
  return out;
}

// theme-vscode.css: the fonts for both themes, then dark (the default) and
// light in its media query, as written by docs/brand/themes.py.
const [fontsCss = "", rest = ""] = vscodeCss.split(/\n:root,\n/);
const [darkCss = "", lightCss = ""] = rest.split("@media (prefers-color-scheme: light)");
const fonts = declared(fontsCss);
const themes = { dark: declared(darkCss), light: declared(lightCss) };

const used = new Set<string>();
for (const source of Object.values(sources)) {
  for (const m of source.matchAll(/var\(\s*(--vscode-[\w-]+)/g)) used.add(m[1] ?? "");
}

// Keys VS Code leaves unset outside high contrast (UNSET in themes.py).
const unset = [
  "--vscode-contrastBorder",
  "--vscode-list-hoverForeground",
  "--vscode-menu-selectionBorder",
  "--vscode-toolbar-hoverOutline",
];

describe("theme-vscode.css", () => {
  it("reads the components' sources", () => {
    expect(Object.keys(sources).length).toBeGreaterThan(30);
    expect(used.size).toBeGreaterThan(50);
  });

  it("declares the fonts and the same names in both themes", () => {
    expect(fonts.get("--vscode-font-family")).toBe('"Segoe WPC", "Segoe UI", sans-serif');
    expect(fonts.get("--vscode-editor-font-family")).toBe('Consolas, "Courier New", monospace');
    expect(themes.dark.size).toBeGreaterThan(700);
    expect([...themes.light.keys()]).toEqual([...themes.dark.keys()]);
  });

  for (const [kind, values] of Object.entries(themes)) {
    it(`defines every variable the components read, ${kind}`, () => {
      const missing = [...used].filter((name) => !fonts.has(name) && !values.has(name));
      expect(missing).toEqual([]);
      const empty = [...used].filter((name) => values.get(name) === "initial");
      expect(empty.filter((name) => !unset.includes(name))).toEqual([]);
    });
  }

  it("gives every --vsc-* token a color in both themes", () => {
    const aliases = [...declared(tokensCss)].filter(([name]) => name.startsWith("--vsc-"));
    expect(aliases.length).toBeGreaterThan(100);
    const colorless = aliases.filter(([, value]) => {
      const target = /^var\((--vscode-[\w.-]+)\)$/.exec(value)?.[1] ?? "";
      return Object.values(themes).some((values) => !/^(#|rgba?\()/.test(values.get(target) ?? ""));
    });
    expect(colorless).toEqual([]);
  });
});
