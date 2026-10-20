import { describe, expect, it } from "vitest";
import vscodeCss from "../../../ui/src/theme/theme-vscode.css?raw";
import tokensCss from "../../../ui/src/theme/theme-colors.css?raw";
import sizesCss from "../../../ui/src/theme/theme-sizes.css?raw";
// VSCodium 1.135's workbench, vendored by scripts/vscodium-css/extract.mjs.
import referenceCss from "../../../docs/brand/vscodium-1.135/workbench-ref.css?raw";

// The components' sources (one module per file, as the app imports them).
const sources = import.meta.glob<string>(
  ["../../../node_modules/@vscode-elements/elements/dist/**/*.js", "!**/bundled.js"],
  { query: "?raw", import: "default", eager: true },
);

// The app's own sources, but the webview's whole table (theme-vscode.css).
const appSources = import.meta.glob<string>(["../../../ui/src/**/*.{css,ts,tsx}", "!**/theme-vscode.css"], {
  query: "?raw",
  import: "default",
  eager: true,
});

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
    // Geist and Geist Mono (@fontsource-variable, main.tsx), the system's behind.
    expect(fonts.get("--vscode-font-family")).toBe('"Geist Variable", "Geist", system-ui, "Segoe UI", sans-serif');
    expect(fonts.get("--vscode-editor-font-family")).toBe(
      '"Geist Mono Variable", "Geist Mono", ui-monospace, Consolas, monospace',
    );
    expect(fonts.get("--vscode-font-size")).toBe("13px");
    expect(fonts.get("--vscode-editor-font-feature-settings")).toBe('"liga" off, "calt" off');
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

/** The value of a property in the reference's rules for a selector list
 * (one rule per line, as extract.mjs writes them): the first one setting
 * it, VSCodium's own value before the rules that scale it with a font size. */
function referenceValue(selector: string, property: string) {
  const values = referenceCss
    .split("\n")
    .filter((line) => line.startsWith(`${selector}{`))
    .map((line) => new RegExp(`[{;]${property}:\\s*([^;}]+)`).exec(line)?.[1])
    .filter((value) => value !== undefined);
  return (values[0] ?? "").replace(/\s*!important$/, "").trim();
}

/** Prettier's numbers, as theme-sizes.css keeps them: 0.08, not .08. */
const prettier = (value: string) => value.replace(/(?<![\d.])\.(\d)/g, "0.$1");

const sizes = declared(sizesCss);

describe("theme-sizes.css", () => {
  it("declares every size once, for every theme", () => {
    // Only :root blocks and Tailwind's: no theme, media query or
    // data-theme can leave dark, light or the system's theme without them.
    const blocks = sizesCss.match(/^\S.*\{$/gm) ?? [];
    expect(blocks).toEqual([":root {", ":root {", "@theme inline {"]);
    const names = [...sizesCss.matchAll(/^\s+(--[\w-]+):/gm)].map((m) => m[1]);
    expect(new Set(names).size).toBe(names.length);
    const referenced = [...sizesCss.matchAll(/var\((--[\w-]+)\)/g)].map((m) => m[1] ?? "");
    expect(referenced.filter((name) => !sizes.has(name))).toEqual([]);
  });

  it("gives every measure token a Tailwind utility", () => {
    const tokens = [...sizes.keys()].filter((name) => name.startsWith("--vsc-size-"));
    expect(tokens.length).toBeGreaterThanOrEqual(12);
    for (const token of tokens) {
      expect(sizes.get(token.replace("--vsc-size-", "--spacing-vsc-"))).toBe(`var(${token})`);
    }
    for (const name of ["--spacing-vsc-size40", "--radius-vsc-small", "--shadow-vsc-xl", "--text-vsc-body-small"]) {
      expect(sizes.has(name)).toBe(true);
    }
  });

  it("has VSCodium 1.135's sizes", () => {
    const root = /^:root \{\n([\s\S]*?)^\}/m.exec(referenceCss)?.[1] ?? "";
    const registry = declared(root);
    expect(registry.size).toBeGreaterThan(30);
    for (const [name, value] of registry) expect(sizes.get(name)).toBe(value);
    // The modern UI's shadows: the workbench's, the small ones turned off.
    expect(sizes.get("--vscode-shadow-xl")).toBe(prettier(referenceValue(".monaco-workbench", "--vscode-shadow-xl")));
    expect(sizes.get("--vscode-shadow-sm")).toBe(referenceValue(".modern-ui.monaco-workbench", "--vscode-shadow-sm"));
    // The measures VSCodium's CSS gives.
    expect(sizes.get("--pane-header-size")).toBe(referenceValue(".modern-ui .monaco-pane-view", "--pane-header-size"));
    expect(sizes.get("--editor-group-tab-height")).toBe(
      referenceValue(
        ".modern-ui-tabs.monaco-workbench .part.editor>.content .editor-group-container>.title.tabs",
        "--editor-group-tab-height",
      ),
    );
    expect(sizes.get("--vsc-size-statusbar")).toBe(referenceValue(".monaco-workbench .part.statusbar", "height"));
    expect(sizes.get("--vsc-size-window-control")).toBe(
      referenceValue(".monaco-workbench .part.titlebar .window-controls-container>.window-icon", "width"),
    );
    expect(sizes.get("--vsc-size-title")).toBe(
      referenceValue(
        ".modern-ui.monaco-workbench .part>.title,.modern-ui.monaco-workbench .part>.header-or-footer",
        "height",
      ),
    );
  });
});

describe("VSCodium 1.135", () => {
  // The keys it knows that Atollo reads, as the reference lists them.
  const list = /that VSCodium [\d.]+ knows:([\s\S]*?)\*\//.exec(referenceCss)?.[1] ?? "";
  const known = new Set(list.match(/--vscode-[\w-]+/g));
  // The webview's fonts (FONTS in themes.py): VS Code gives them to a
  // webview, the workbench has none.
  const webview = /^--vscode-(editor-)?font-/;

  it("knows every --vscode-* key Atollo reads", () => {
    expect(known.size).toBeGreaterThan(100);
    const keys = new Set<string>();
    for (const source of Object.values(appSources)) {
      for (const m of source.matchAll(/--vscode-[\w\\.-]*\w/g)) {
        keys.add(m[0].replaceAll("\\.", ".").replaceAll(".", "-"));
      }
    }
    expect(keys.size).toBeGreaterThan(100);
    expect([...keys].filter((key) => !webview.test(key) && !known.has(key))).toEqual([]);
  });

  for (const [kind, values] of Object.entries(themes)) {
    it(`colors the activity bar's items with the keys it reads, ${kind}`, () => {
      for (const state of ["active", "hover"]) {
        for (const part of ["Background", "Foreground"]) {
          expect(values.get(`--vscode-modernActivityBar-${state}${part}`)).toMatch(/^(#|rgba?\()/);
        }
      }
      expect(values.get("--vscode-surface-border")).toMatch(/^(#|rgba?\()/);
      expect([...values.keys()].filter((name) => name.includes("modernActivityBarItem"))).toEqual([]);
    });
  }
});
