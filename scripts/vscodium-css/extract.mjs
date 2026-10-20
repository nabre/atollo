// The part of VSCodium's workbench CSS that Atollo mirrors, vendored in
// docs/brand/vscodium-<major.minor>/workbench-ref.css: the rules of the parts
// the app imitates (RULES below), the sizes the workbench registers, and the
// --vscode-* keys Atollo reads that this VSCodium knows. The theme
// generator (docs/brand/themes.py) and the theme test
// (test/ui/theme/theme.test.ts) take the measures and the known keys from it.
//
// The source is the editor Atollo ships: the reh-web server pinned in
// tauri/catalog/components.json ("vscodium-server", version and win32-x64
// SHA-256). Its out/vs/code/browser/workbench/workbench.css is the whole
// workbench's CSS; workbench.js beside it holds the color and size
// registries. Code-OSS and VSCodium are MIT: the reference keeps the CSS's
// copyright banner and the license.
//
//   node scripts/vscodium-css/extract.mjs                  (downloads the pinned archive)
//   node scripts/vscodium-css/extract.mjs <reh-web.tar.gz> (an archive already downloaded)
//   node scripts/vscodium-css/extract.mjs <workbench.css>  (an archive already extracted)
//
// An archive must have the catalog's SHA-256; an extracted workbench.css
// must sit in its server folder (out/vs/code/browser/workbench/, with
// workbench.js beside it and package.json at the root, of the pinned
// version). Node 18 or later, no dependencies. Run it again when the
// catalog pins another editor or the app reads new --vscode-* keys (it
// fails on a key that VSCodium does not know), then
// `python docs/brand/themes.py`.
import { createHash } from "node:crypto";
import {
  createReadStream,
  createWriteStream,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  renameSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, relative, resolve } from "node:path";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";
import { createGunzip } from "node:zlib";

const repo = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const TARGET = "win32-x64";
const WORKBENCH = "out/vs/code/browser/workbench/";

// The rules kept: every rule with one selector that matches, in the order
// of the source, under the heading of its part. Each part is the one of
// the VSCodium alignment that reads them.
const RULES = [
  [
    "Workbench: shadows, the modern UI's floating parts",
    [
      /^\.monaco-workbench$/,
      /^\.modern-ui\.monaco-workbench$/,
      /^\.monaco-workbench\.no-shadows$/,
      /^\.monaco-workbench\.floating-panels \.part\.(sidebar|auxiliarybar|panel|editor)$/,
      /^\.monaco-workbench\.modern-ui\.floating-panels \.part\.editor$/,
    ],
  ],
  [
    "Title bar",
    [
      /^\.monaco-workbench \.part\.titlebar$/,
      /^\.monaco-workbench \.part\.titlebar>\.titlebar-container$/,
      /^\.monaco-workbench \.part\.titlebar>\.titlebar-container>\.titlebar-left>\.window-appicon(\.codicon)?$/,
      /^\.monaco-workbench(\.web)? \.part\.titlebar( \.titlebar-right)? \.window-controls-container$/,
      /^\.monaco-workbench:not\(\.web\):not\(\.mac\) \.part\.titlebar \.window-controls-container$/,
      /^\.monaco-workbench \.part\.titlebar(\.light)? \.window-controls-container>\.window-icon(:hover|\.window-close:hover)?$/,
    ],
  ],
  [
    "Activity bar",
    [
      /^\.monaco-workbench \.part\.activitybar$/,
      /^\.monaco-workbench\.floating-panels \.part\.activitybar/,
      /^\.monaco-workbench \.activitybar>\.content :not\(\.monaco-menu\)>\.monaco-action-bar \.action-label(\.codicon)?$/,
      /^\.modern-ui \.activitybar(\.compact)?>\.content/,
      /^\.monaco-workbench \.activitybar>\.content :not\(\.monaco-menu\)>\.monaco-action-bar \.badge \.badge-content$/,
      /^\.modern-ui\.monaco-workbench \.activitybar>\.content :not\(\.monaco-menu\)>\.monaco-action-bar \.badge \.badge-content$/,
    ],
  ],
  [
    "Status bar",
    [
      /^\.monaco-workbench \.part\.statusbar$/,
      /^\.monaco-workbench \.part\.statusbar>\.items-container>\.statusbar-item(>\.statusbar-item-label)?$/,
      /^\.monaco-workbench\.floating-panels \.part\.statusbar$/,
      /^\.modern-ui \.part\.statusbar>\.items-container>\.statusbar-item(>\.statusbar-item-label)?$/,
      /^\.modern-ui \.part\.statusbar>\.items-container>\.statusbar-item\.left\.first-visible-item>\.statusbar-item-label$/,
    ],
  ],
  [
    "Part titles and the panel's tabs",
    [
      /^\.monaco-workbench \.part>\.title(>\.title-label)?$/,
      /^\.modern-ui\.monaco-workbench \.part>\.title$/,
      /^\.modern-ui\.monaco-workbench \.part\.basepanel:is\(\.bottom,\.top,\.left,\.right\) \.composite\.title$/,
      /^\.monaco-workbench \.pane-composite-part \.part\.panel>\.title>\.composite-bar-container>\.composite-bar>\.monaco-action-bar \.action-item(\.checked)? \.active-item-indicator(:before)?$/,
    ],
  ],
  [
    "Lists, view headers, scroll bars",
    [
      /^\.monaco-list-row$/,
      /^\.monaco-pane-view$/,
      /^\.monaco-pane-view \.pane>\.pane-header$/,
      /^\.modern-ui \.monaco-pane-view$/,
      /^\.modern-ui \.monaco-pane-view \.pane>\.pane-header(:hover|:before)?$/,
      /^\.modern-ui \.monaco-pane-view \.pane:not\(\.chat-viewpane-container\)>\.pane-header$/,
      /^\.modern-ui \.monaco-pane-view \.pane-body:not\(\.chat-viewpane\) \.monaco-list-row$/,
      /^\.modern-ui :is\(\.part\.sidebar,\.part\.auxiliarybar\) \.monaco-pane-view \.pane>\.pane-header$/,
      /^\.monaco-scrollable-element>\.scrollbar(>\.slider(:hover|\.active)?)?$/,
      /^\.modern-ui \.monaco-scrollable-element>\.scrollbar>\.slider$/,
    ],
  ],
  [
    "Buttons, inputs, select boxes, dialogs",
    [
      /^\.monaco-text-button(\.small|:focus)?$/,
      /^\.monaco-inputbox$/,
      /^\.monaco-inputbox>\.ibwrapper>\.input$/,
      /^\.monaco-select-box$/,
      /^\.monaco-action-bar \.action-item \.monaco-select-box$/,
      /^\.settings-editor>\.settings-body \.settings-tree-container \.setting-item-contents \.monaco-select-box$/,
      /^\.monaco-select-box-dropdown-container$/,
      /^\.monaco-dialog-modal-block$/,
      /^\.monaco-dialog-box$/,
      /^\.monaco-dialog-box \.dialog-message-row \.dialog-message-container \.dialog-message(-detail)?$/,
      /^\.monaco-dialog-box>\.dialog-buttons-row$/,
      /^\.modern-ui-notifications-dialogs \.monaco-dialog-box/,
    ],
  ],
  [
    "Editor tabs, extension header, settings editor",
    [
      /^\.monaco-workbench \.part\.editor>\.content \.editor-group-container>\.title \.tabs-container>\.tab$/,
      /^\.monaco-workbench \.part\.editor>\.content \.editor-group-container\.active>\.title \.tabs-container>\.tab$/,
      /^\.modern-ui-tabs\.monaco-workbench \.part\.editor>\.content \.editor-group-container>\.title\.tabs$/,
      /^\.modern-ui-tabs\.monaco-workbench \.part\.editor \.tabs-and-actions-container:not\(\.wrapping\) \.tabs-container>\.tab$/,
      /^\.extension-editor>\.header$/,
      /^\.extension-editor>\.header>\.icon-container \.extension-icon \.icon$/,
      /^\.extension-editor>\.header>\.details>\.title>\.name$/,
      /^\.extension-editor>\.header>\.details>\.subtitle>\.subtitle-entry:not\(:empty\):not\(\.last-non-empty\)$/,
      /^\.extension-editor>\.body>\.navbar$/,
      /^\.settings-editor>\.settings-body \.settings-toc-container$/,
      /^\.settings-editor>\.settings-body \.settings-tree-container \.setting-item-contents(\.is-configured)? \.setting-item-modified-indicator$/,
    ],
  ],
  ["Codicons", [/^\.codicon\[class\*=codicon-\]$/, /^\.monaco-workbench \.codicon\[class\*=codicon-\]$/]],
];

// The webview's font variables (docs/brand/themes.py, FONTS): VS Code gives
// them to a webview, the workbench has none of them.
const WEBVIEW = /^--vscode-(editor-)?font-/;

// The license of Code-OSS (github.com/microsoft/vscode, LICENSE.txt), the
// source of VSCodium; VSCodium is MIT too (product.json, licenseUrl).
const LICENSE = `MIT License

Copyright (c) 2015 - present Microsoft Corporation

All rights reserved.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.`;

/** The pinned editor: version and the win32-x64 archive's SHA-256. */
function pinned() {
  const catalog = JSON.parse(readFileSync(join(repo, "tauri/catalog/components.json"), "utf8"));
  const component = catalog.components.find((c) => c.id === "vscodium-server");
  const sha256 = component.sha256ByTarget?.[TARGET] ?? component.sha256;
  const name = `vscodium-reh-web-${TARGET}-${component.version}.tar.gz`;
  const url = `https://github.com/VSCodium/vscodium/releases/download/${component.version}/${name}`;
  return { version: component.version, sha256, name, url };
}

async function sha256Of(path) {
  const hash = createHash("sha256");
  await pipeline(createReadStream(path), hash);
  return hash.digest("hex");
}

async function download({ url, name }) {
  const dir = join(tmpdir(), "atollo-vscodium");
  mkdirSync(dir, { recursive: true });
  const path = join(dir, name);
  if (!existsSync(path)) {
    console.log(`downloading ${url}`);
    const res = await fetch(url);
    if (!res.ok || !res.body) throw new Error(`${url}: HTTP ${res.status}`);
    await pipeline(Readable.fromWeb(res.body), createWriteStream(`${path}.part`));
    renameSync(`${path}.part`, path);
  }
  return path;
}

/** The files `wanted` accepts (paths inside the archive), read from a
 * .tar.gz as a stream: ustar headers, GNU long names, pax paths. */
async function readArchive(path, wanted) {
  const found = new Map();
  let pending = Buffer.alloc(0);
  let skip = 0;
  let take = null; // { name, left, pad, chunks, meta }
  let nextName = null;
  for await (const chunk of createReadStream(path).pipe(createGunzip())) {
    pending = pending.length ? Buffer.concat([pending, chunk]) : chunk;
    for (;;) {
      if (take) {
        const n = Math.min(take.left, pending.length);
        take.chunks.push(pending.subarray(0, n));
        pending = pending.subarray(n);
        take.left -= n;
        if (take.left > 0) break;
        const data = Buffer.concat(take.chunks);
        if (take.meta === "L") nextName = data.toString("utf8").replace(/\0.*$/s, "");
        else if (take.meta === "x") nextName = /\d+ path=([^\n]*)\n/.exec(data.toString("utf8"))?.[1] ?? nextName;
        else if (take.meta === null) found.set(take.name, data);
        skip = take.pad;
        take = null;
      } else if (skip > 0) {
        const n = Math.min(skip, pending.length);
        pending = pending.subarray(n);
        skip -= n;
        if (skip > 0) break;
      } else {
        if (pending.length < 512) break;
        const header = pending.subarray(0, 512);
        pending = pending.subarray(512);
        if (header.every((b) => b === 0)) continue;
        const field = (at, len) => header.subarray(at, at + len).toString("utf8").replace(/\0.*$/s, "");
        const size = parseInt(field(124, 12).trim() || "0", 8);
        const type = field(156, 1) || "0";
        const prefix = field(345, 155);
        let name = nextName ?? (prefix ? `${prefix}/${field(0, 100)}` : field(0, 100));
        name = name.replace(/^\.\//, "");
        const pad = Math.ceil(size / 512) * 512 - size;
        if (type === "L" || type === "x") {
          take = { name, left: size, pad, chunks: [], meta: type };
          continue;
        }
        nextName = null;
        if (type === "g" || !wanted(name)) skip = size + pad;
        else take = { name, left: size, pad, chunks: [], meta: null };
      }
    }
  }
  return found;
}

/** The server's files Atollo needs, from an archive or an extracted folder. */
async function sources(arg, pin) {
  const files = [`${WORKBENCH}workbench.css`, `${WORKBENCH}workbench.js`, "package.json"];
  // The built-in extensions' manifests: the colors they contribute (git's
  // gitDecoration.*, for one) are theme keys as the registry's.
  const manifest = /^extensions\/[^/]+\/package\.json$/;
  let read;
  let extensions;
  if (!arg || arg.endsWith(".tar.gz") || arg.endsWith(".tgz")) {
    const archive = arg ? resolve(arg) : await download(pin);
    const sha = await sha256Of(archive);
    if (sha !== pin.sha256) throw new Error(`${archive}: SHA-256 ${sha}, the catalog has ${pin.sha256}`);
    const found = await readArchive(archive, (name) => files.includes(name) || manifest.test(name));
    read = (name) => {
      const data = found.get(name);
      if (!data) throw new Error(`${basename(archive)}: no ${name}`);
      return data.toString("utf8");
    };
    extensions = [...found.keys()].filter((name) => manifest.test(name)).map(read);
  } else {
    const root = resolve(dirname(arg), relative(WORKBENCH, ""));
    read = (name) => readFileSync(join(root, name), "utf8");
    extensions = readdirSync(join(root, "extensions"), { withFileTypes: true })
      .filter((e) => e.isDirectory() && existsSync(join(root, "extensions", e.name, "package.json")))
      .map((e) => read(`extensions/${e.name}/package.json`));
  }
  const version = JSON.parse(read("package.json")).version;
  if (version !== pin.version) throw new Error(`VSCodium ${version}, the catalog pins ${pin.version}`);
  const contributed = extensions.flatMap((text) => JSON.parse(text).contributes?.colors ?? []).map((c) => c.id);
  return { css: read(files[0]), js: read(files[1]), contributed };
}

/** Index after the comment, string or plain character at `i`. */
function stepOver(css, i) {
  if (css.startsWith("/*", i)) {
    const end = css.indexOf("*/", i + 2);
    return end < 0 ? css.length : end + 2;
  }
  const c = css[i];
  if (c === '"' || c === "'") {
    let j = i + 1;
    while (j < css.length && css[j] !== c) j += css[j] === "\\" ? 2 : 1;
    return j + 1;
  }
  return i + 1;
}

/** The style rules and at-rules of the CSS, the conditional groups
 * (@media, @supports, @container, @layer) opened: [{ context, prelude, body }]. */
function parseRules(css) {
  const out = [];
  const walk = (start, end, context) => {
    let i = start;
    while (i < end) {
      let j = i;
      while (j < end && css[j] !== "{" && css[j] !== ";" && css[j] !== "}") j = stepOver(css, j);
      if (j >= end || css[j] !== "{") {
        i = j + 1;
        continue;
      }
      let depth = 1;
      let k = j + 1;
      while (k < end && depth > 0) {
        if (css[k] === "{") depth++;
        else if (css[k] === "}") depth--;
        k = depth > 0 ? stepOver(css, k) : k;
      }
      const prelude = css
        .slice(i, j)
        .replace(/\/\*[\s\S]*?\*\//g, "")
        .trim();
      if (/^@(media|supports|container|layer)\b/.test(prelude)) walk(j + 1, k, [...context, prelude]);
      else out.push({ context, prelude, body: css.slice(j + 1, k) });
      i = k + 1;
    }
  };
  walk(0, css.length, []);
  return out;
}

/** A selector list split at its top-level commas. */
function selectors(prelude) {
  const out = [];
  let depth = 0;
  let from = 0;
  for (let i = 0; i < prelude.length; i++) {
    const c = prelude[i];
    if (c === "(" || c === "[") depth++;
    else if (c === ")" || c === "]") depth--;
    else if (c === "," && depth === 0) {
      out.push(prelude.slice(from, i).trim());
      from = i + 1;
    }
  }
  out.push(prelude.slice(from).trim());
  return out.map((s) => s.replace(/\s+/g, " "));
}

/** The keys VSCodium registers in workbench.js: the colors (registerColor,
 * found by "foreground") and the sizes (registerSize, by "bodyFontSize")
 * with their values, as the workbench names their variables (every dot a
 * dash). */
function registry(js) {
  const callee = (first) => {
    const m = new RegExp(`([A-Za-z_$][\\w$]*)\\("${first.replace(".", "\\.")}",`).exec(js);
    if (!m) throw new Error(`workbench.js: no registration of ${first}`);
    return m[1].replace(/\$/g, "\\$");
  };
  const colors = new Set();
  for (const m of js.matchAll(new RegExp(`(?<![\\w$.])${callee("foreground")}\\("([A-Za-z][\\w.]*)",`, "g"))) {
    colors.add(`--vscode-${m[1].replaceAll(".", "-")}`);
  }
  const sizes = new Map();
  const size = callee("bodyFontSize");
  const sizeCall = new RegExp(`(?<![\\w$.])${size}\\("([A-Za-z][\\w.]*)",[A-Za-z_$][\\w$]*\\((\\d+),"([a-z%]*)"\\)`, "g");
  for (const m of js.matchAll(sizeCall)) {
    if (m[1].startsWith("agents.")) continue;
    sizes.set(`--vscode-${m[1].replaceAll(".", "-")}`, `${m[2]}${m[3]}`);
  }
  if (colors.size < 500 || sizes.size < 20) throw new Error(`workbench.js: ${colors.size} colors, ${sizes.size} sizes`);
  return { colors, sizes };
}

/** Files under a folder, recursively. */
function walkFiles(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? walkFiles(join(dir, e.name)) : [join(dir, e.name)],
  );
}

/** The --vscode-* keys Atollo's interface reads or sets (ui/src, but the
 * webview's whole table in theme-vscode.css), every dot a dash. */
function atolloKeys() {
  const keys = new Set();
  const src = join(repo, "ui/src");
  for (const file of walkFiles(src)) {
    if (!/\.(css|tsx?)$/.test(file) || basename(file) === "theme-vscode.css") continue;
    for (const m of readFileSync(file, "utf8").matchAll(/--vscode-[\w\\.-]*\w/g)) {
      const name = m[0].replaceAll("\\.", ".").replaceAll(".", "-");
      if (!WEBVIEW.test(name)) keys.add(name);
    }
  }
  return keys;
}

async function main() {
  const pin = pinned();
  const { css, js, contributed } = await sources(process.argv[2], pin);
  const banner = /^\/\*![\s\S]*?\*\//.exec(css)?.[0];
  if (!banner) throw new Error("workbench.css: no copyright banner");
  const all = parseRules(css);
  const { colors, sizes } = registry(js);
  for (const id of contributed) colors.add(`--vscode-${id.replaceAll(".", "-")}`);

  // Every --vscode-* variable the workbench's CSS reads or declares.
  const inCss = new Set(css.match(/--vscode-[\w-]*\w/g));
  const known = (name) => colors.has(name) || sizes.has(name) || inCss.has(name);
  const keys = [...atolloKeys()].sort();
  const unknown = keys.filter((k) => !known(k));

  const used = new Set();
  const groups = RULES.map(([title, patterns]) => {
    const lines = [];
    all.forEach((rule, index) => {
      // A rule with nested rules (CSS nesting) styles what is inside a
      // part, not the part: left out.
      if (used.has(index) || rule.body.includes("{")) return;
      const match =
        rule.prelude.startsWith("@font-face")
          ? title === "Codicons" && /font-family:\s*codicon\b/.test(rule.body)
          : selectors(rule.prelude).some((s) => patterns.some((p) => p.test(s)));
      if (!match) return;
      used.add(index);
      let text = `${rule.prelude}{${rule.body}}`;
      for (const at of [...rule.context].reverse()) text = `${at}{${text}}`;
      lines.push(text);
    });
    return `/* ${title} */\n${lines.join("\n")}`;
  });

  const [major, minor] = pin.version.split(".");
  const outDir = join(repo, `docs/brand/vscodium-${major}.${minor}`);
  const sizeLines = [...sizes].map(([n, v]) => `  ${n}: ${v};`).join("\n");
  const keyLines = keys
    .filter(known)
    .map((k) => `   ${k}`)
    .join("\n");
  const text = `${banner}
/* VSCodium ${pin.version} (${pin.name}, SHA-256 ${pin.sha256}),
   ${WORKBENCH}workbench.css: the rules Atollo mirrors, unchanged, one per
   line, and from workbench.js the sizes the workbench registers.
   Written by scripts/vscodium-css/extract.mjs: do not edit, run the script.

${LICENSE.replace(/^/gm, "   ").replace(/^ +$/gm, "")}
*/

/* The sizes VSCodium registers (registerSize), the same in every theme. */
:root {
${sizeLines}
}

/* The --vscode-* keys Atollo reads or sets that VSCodium ${pin.version} knows:
   registered in workbench.js or used by workbench.css.
${keyLines}
*/

${groups.join("\n\n")}
`;
  mkdirSync(outDir, { recursive: true });
  const out = join(outDir, "workbench-ref.css");
  writeFileSync(out, text.replace(/\r\n/g, "\n"));
  console.log(`${relative(repo, out)}: ${used.size} rules, ${sizes.size} sizes, ${keys.length - unknown.length} keys`);
  if (unknown.length) {
    console.error(`Atollo reads keys VSCodium ${pin.version} does not know:\n  ${unknown.join("\n  ")}`);
    process.exitCode = 1;
  }
}

main().catch((e) => {
  console.error(e.message);
  process.exit(1);
});
