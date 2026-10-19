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
  const langLinks = langs
    .map((l) => (l === lang ? `<span aria-current="page" lang="${l}">${l.toUpperCase()}</span>` : `<a href="${root}${pathOf(l)}" hreflang="${l}" lang="${l}" title="${strings[l].langName}">${l.toUpperCase()}</a>`))
    .join("");
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
