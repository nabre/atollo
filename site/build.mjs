// Builds the landing page into _site/: one page per language (Italian at the
// root, the others in en/, fr/, de/, as the READMEs), the brand files from
// docs/brand (the wordmark, never the lockup: symbol and word both start
// with an A) and three screenshots from docs/app/images. No dependencies:
// node site/build.mjs. Published by .github/workflows/site.yml.
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
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

rmSync(out, { recursive: true, force: true });
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
  const values = { ...Object.fromEntries(Object.entries(strings[lang]).map(([k, v]) => [k, escape(v)])), lang, root, base, path: pathOf(lang), langPath: pathOf(lang), alternates, langLinks, toolsMatrix: matrixHtml(catalog, lang, strings[lang]), componentCount: String(catalog.components.length) };
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
const images = join(repo, "docs", "app", "images");
const shots = { "shot-projects.png": "05-progetto-avvio-log.png", "shot-agents.png": "10-agenti-quote-statistiche.png", "shot-tools.png": "13-strumenti-catalogo-matrice.png" };
for (const [to, from] of Object.entries(shots)) cpSync(join(images, from), join(out, "assets", to));

console.log(`_site: ${langs.length} pages`);
