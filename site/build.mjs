// Builds the landing page into _site/: one page per language (Italian at the
// root, the others in en/, fr/, de/, as the READMEs), the brand files from
// docs/brand (the wordmark, whose last O is the symbol: MARKETING.md, «Logo
// nelle intestazioni») and the gallery's screenshots, dark and light, from
// site/shots (made by site/shots/make.mjs). No dependencies:
// node site/build.mjs. Published by .github/workflows/site.yml.
import { cpSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadCatalog, matrixHtml } from "./matrix.mjs";

const site = dirname(fileURLToPath(import.meta.url));
const repo = dirname(site);
const out = join(repo, "_site");
const base = "https://nabre.github.io/atollo/";
const langs = ["it", "en", "fr", "de"];
const pathOf = (lang) => (lang === "it" ? "" : `${lang}/`);

const template = readFileSync(join(site, "template.html"), "utf8");
const strings = Object.fromEntries(langs.map((l) => [l, JSON.parse(readFileSync(join(site, "strings", `${l}.json`), "utf8"))]));

// Same keys in every language: a missing one would show as {{key}}.
const keys = Object.keys(strings.it).sort().join();
for (const l of langs) {
  if (Object.keys(strings[l]).sort().join() !== keys) throw new Error(`site/strings/${l}.json: keys differ from it.json`);
}

// The strings may hold <code>; & and " are escaped so they fit in attributes too.
const escape = (s) => s.replaceAll("&", "&amp;").replaceAll('"', "&quot;");

const catalog = loadCatalog(repo);

// The language menu's flags, inline SVG at 3:2 (Windows draws no flag emoji):
// English is the United Kingdom's, cut to 3:2. The Union Jack's clip path
// gets a new id each time, as the menu may show it twice.
let flagIds = 0;
const flags = {
  it: () => `<svg viewBox="0 0 3 2"><path fill="#009246" d="M0 0h1v2H0z"/><path fill="#fff" d="M1 0h1v2H1z"/><path fill="#ce2b37" d="M2 0h1v2H2z"/></svg>`,
  fr: () => `<svg viewBox="0 0 3 2"><path fill="#0055a4" d="M0 0h1v2H0z"/><path fill="#fff" d="M1 0h1v2H1z"/><path fill="#ef4135" d="M2 0h1v2H2z"/></svg>`,
  de: () => `<svg viewBox="0 0 5 3" preserveAspectRatio="none"><path fill="#000" d="M0 0h5v1H0z"/><path fill="#dd0000" d="M0 1h5v1H0z"/><path fill="#ffce00" d="M0 2h5v1H0z"/></svg>`,
  en: () => {
    const id = `uk${++flagIds}`;
    return `<svg viewBox="0 0 60 30" preserveAspectRatio="xMidYMid slice"><clipPath id="${id}"><path d="M30 15h30v15zv15H0zH0V0zV0h30z"/></clipPath><path fill="#012169" d="M0 0h60v30H0z"/><path d="M0 0l60 30m0-30L0 30" stroke="#fff" stroke-width="6"/><path d="M0 0l60 30m0-30L0 30" clip-path="url(#${id})" stroke="#c8102e" stroke-width="4"/><path d="M30 0v30M0 15h60" stroke="#fff" stroke-width="10"/><path d="M30 0v30M0 15h60" stroke="#c8102e" stroke-width="6"/></svg>`;
  },
};
const flag = (l) => `<span class="flag" aria-hidden="true">${flags[l]()}</span>`;

// The app's version, in the download names (Atollo-<version>-<target>.exe,
// .github/workflows/app.yml); download.js puts the latest release's in.
const version = readFileSync(join(repo, "tauri", "Cargo.toml"), "utf8").match(/^version = "(\d+\.\d+\.\d+)"$/m)?.[1];
if (!version) throw new Error("tauri/Cargo.toml: no version");

// The folder's contents, not the folder: a local preview server started
// in _site keeps the folder itself busy on Windows.
mkdirSync(out, { recursive: true });
for (const entry of readdirSync(out)) rmSync(join(out, entry), { recursive: true, force: true });
mkdirSync(join(out, "assets"), { recursive: true });

for (const lang of langs) {
  const root = lang === "it" ? "" : "../";
  const alternates = langs
    .map((l) => `<link rel="alternate" hreflang="${l}" href="${base}${pathOf(l)}">`)
    .concat(`<link rel="alternate" hreflang="x-default" href="${base}">`)
    .join("\n");
  const item = (l) =>
    l === lang
      ? `<li><span aria-current="page" lang="${l}">${flag(l)}<b>${l.toUpperCase()}</b>${strings[l].langName}</span></li>`
      : `<li><a href="${root}${pathOf(l)}" hreflang="${l}" lang="${l}">${flag(l)}<b>${l.toUpperCase()}</b>${strings[l].langName}</a></li>`;
  const langLinks =
    `<details class="lang-menu"><summary aria-label="${escape(strings[lang].langLabel)}: ${strings[lang].langName}">${flag(lang)}<b>${lang.toUpperCase()}</b>` +
    `<svg class="chev" viewBox="0 0 10 6" aria-hidden="true"><path d="M1 1l4 4 4-4" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/></svg></summary>` +
    `<ul>${langs.map(item).join("")}</ul></details>`;
  const values = { ...Object.fromEntries(Object.entries(strings[lang]).map(([k, v]) => [k, escape(v)])), lang, root, base, path: pathOf(lang), langPath: pathOf(lang), alternates, langLinks, toolsMatrix: matrixHtml(catalog, lang, strings[lang]), componentCount: String(catalog.components.length), version };
  const html = template.replace(/\{\{(\w+)\}\}/g, (_, key) => {
    if (!(key in values)) throw new Error(`template.html: no value for {{${key}}}`);
    return values[key];
  });
  mkdirSync(join(out, pathOf(lang)), { recursive: true });
  writeFileSync(join(out, pathOf(lang), "index.html"), html);
}

cpSync(join(site, "assets"), join(out, "assets"), { recursive: true });
const brand = join(repo, "docs", "brand");
for (const f of ["wordmark-dark.svg", "wordmark-light.svg", "symbol.svg", "brand.png", "hero-dark.svg", "hero-light.svg"]) cpSync(join(brand, f), join(out, "assets", f));
const shots = join(site, "shots");
for (const f of readdirSync(shots).filter((f) => f.endsWith(".png"))) cpSync(join(shots, f), join(out, "assets", f));

console.log(`_site: ${langs.length} pages`);
