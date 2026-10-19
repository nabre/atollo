// Builds the landing page into _site/: one page per language (English at the
// root, the others in it/, fr/, de/, as the READMEs), what search engines
// read (sitemap.xml with every language's alternates, for Search Console:
// a robots.txt here, under /atollo/, would be read by none; the JSON-LD of
// the app in each page), a redirect from the old en/ to the root, the brand
// files from docs/brand (the wordmark, whose last O is the symbol:
// MARKETING.md, «Logo nelle intestazioni») and the gallery's screenshots,
// dark and light, from
// site/shots (made by site/shots/make.mjs). The icons are react-icons', as
// in the app, drawn here into plain SVG: the page runs no React. After
// npm ci (the ui workspace's react, react-dom and react-icons):
// node site/build.mjs. Published by .github/workflows/site.yml.
import { cpSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { RiArrowDownSLine, RiComputerLine, RiMoonLine, RiSunLine } from "react-icons/ri";
import { loadCatalog, matrixHtml } from "./matrix.mjs";

const site = dirname(fileURLToPath(import.meta.url));
const repo = dirname(site);
const out = join(repo, "_site");
const base = "https://nabre.github.io/atollo/";
// English first: the root page, x-default and the keys every language must have.
const langs = ["en", "it", "fr", "de"];
const pathOf = (lang) => (lang === "en" ? "" : `${lang}/`);
// og:locale of each page.
const locales = { en: "en_US", it: "it_IT", fr: "fr_FR", de: "de_DE" };
// The words the page is found by, in English on every page (as the GitHub
// topics): the meta keywords and the JSON-LD's keywords.
const keywords = [
  "portable development environment", "portable dev environment", "zero install", "no admin rights",
  "USB development environment", "AI agents", "Claude Code", "Codex", "Antigravity", "MCP servers",
  "VSCodium", "portable VS Code", "toolchain manager", "runtime manager", "version manager",
  "MySQL", "MongoDB", "SQLite", "phpMyAdmin", "Caddy", "Apache", "PHP", "Node.js", "Python", "Rust", "Go", "Java",
  "Windows", "macOS", "Linux", "ARM64", "Tauri",
].join(", ");

const template = readFileSync(join(site, "template.html"), "utf8");
const strings = Object.fromEntries(langs.map((l) => [l, JSON.parse(readFileSync(join(site, "strings", `${l}.json`), "utf8"))]));

// Same keys in every language: a missing one would show as {{key}}.
const keys = Object.keys(strings.en).sort().join();
for (const l of langs) {
  if (Object.keys(strings[l]).sort().join() !== keys) throw new Error(`site/strings/${l}.json: keys differ from en.json`);
}

// The strings may hold <code>; & and " are escaped so they fit in attributes too.
const escape = (s) => s.replaceAll("&", "&amp;").replaceAll('"', "&quot;");

const catalog = loadCatalog(repo);

// A react-icons icon as SVG markup, hidden from screen readers (the control
// around it carries the label); its size comes from site.css.
const icon = (Icon, props = {}) => renderToStaticMarkup(createElement(Icon, { "aria-hidden": true, ...props }));
const icons = {
  iconSystem: icon(RiComputerLine, { "data-icon": "system" }),
  iconLight: icon(RiSunLine, { "data-icon": "light" }),
  iconDark: icon(RiMoonLine, { "data-icon": "dark" }),
};
const chevron = icon(RiArrowDownSLine, { className: "chev" });

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

// The app as search engines describe it (schema.org SoftwareApplication),
// in the page's language; "<" escaped so no string can close the script.
const jsonLd = (lang) =>
  JSON.stringify({
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: "Atollo",
    alternateName: "Atollo — Universal Portable Dev & AI Workspace",
    description: strings[lang].description.replace(/<[^>]+>/g, ""),
    url: `${base}${pathOf(lang)}`,
    inLanguage: lang,
    applicationCategory: "DeveloperApplication",
    applicationSubCategory: "Portable development environment",
    operatingSystem: "Windows 10, Windows 11, macOS 12+, Linux (glibc 2.35+)",
    processorRequirements: "x64, ARM64",
    softwareVersion: version,
    downloadUrl: "https://github.com/nabre/atollo/releases/latest",
    installUrl: "https://github.com/nabre/atollo/releases/latest",
    codeRepository: "https://github.com/nabre/atollo",
    license: "https://opensource.org/licenses/MIT",
    isAccessibleForFree: true,
    offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
    image: `${base}assets/brand.png`,
    screenshot: `${base}assets/shot-projects-dark.png`,
    author: { "@type": "Person", name: "nabre", url: "https://github.com/nabre" },
    keywords,
  }).replaceAll("<", "\\u003c");

for (const lang of langs) {
  const root = lang === "en" ? "" : "../";
  const alternates = langs
    .map((l) => `<link rel="alternate" hreflang="${l}" href="${base}${pathOf(l)}">`)
    .concat(`<link rel="alternate" hreflang="x-default" href="${base}">`)
    .join("\n");
  const ogLocales = [`<meta property="og:locale" content="${locales[lang]}">`]
    .concat(langs.filter((l) => l !== lang).map((l) => `<meta property="og:locale:alternate" content="${locales[l]}">`))
    .join("\n");
  const item = (l) =>
    l === lang
      ? `<li><span aria-current="page" lang="${l}">${flag(l)}<b>${l.toUpperCase()}</b>${strings[l].langName}</span></li>`
      : `<li><a href="${root}${pathOf(l)}" hreflang="${l}" lang="${l}">${flag(l)}<b>${l.toUpperCase()}</b>${strings[l].langName}</a></li>`;
  const langLinks =
    `<details class="lang-menu"><summary aria-label="${escape(strings[lang].langLabel)}: ${strings[lang].langName}">${flag(lang)}<b>${lang.toUpperCase()}</b>` +
    `${chevron}</summary>` +
    `<ul>${langs.map(item).join("")}</ul></details>`;
  const values = { ...Object.fromEntries(Object.entries(strings[lang]).map(([k, v]) => [k, escape(v)])), lang, root, base, path: pathOf(lang), langPath: pathOf(lang), alternates, ogLocales, keywords: escape(keywords), jsonLd: jsonLd(lang), langLinks, ...icons, toolsMatrix: matrixHtml(catalog, lang, strings[lang]), componentCount: String(catalog.components.length), version };
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

// The English page lived in en/ until it moved to the root: old links land
// there, and search engines learn the new address.
mkdirSync(join(out, "en"), { recursive: true });
writeFileSync(
  join(out, "en", "index.html"),
  `<!doctype html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n<title>Atollo</title>\n<link rel="canonical" href="${base}">\n<meta name="robots" content="noindex">\n<meta http-equiv="refresh" content="0; url=${base}">\n</head>\n<body><a href="${base}">Atollo</a></body>\n</html>\n`,
);

// Every language's page with its alternates, as the <link>s in the pages:
// submitted by hand in Google Search Console and Bing Webmaster Tools.
const today = new Date().toISOString().slice(0, 10);
const urlEntry = (lang) =>
  `  <url>\n    <loc>${base}${pathOf(lang)}</loc>\n    <lastmod>${today}</lastmod>\n` +
  langs.map((l) => `    <xhtml:link rel="alternate" hreflang="${l}" href="${base}${pathOf(l)}"/>\n`).join("") +
  `    <xhtml:link rel="alternate" hreflang="x-default" href="${base}"/>\n  </url>\n`;
writeFileSync(
  join(out, "sitemap.xml"),
  `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">\n${langs.map(urlEntry).join("")}</urlset>\n`,
);

console.log(`_site: ${langs.length} pages`);
