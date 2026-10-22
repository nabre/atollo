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
// The Guide's pages, one per topic and language, come from site/guide.mjs
// (D-164). `--check` also checks the pages, the sitemap and the links.
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { RiArrowDownSLine, RiComputerLine, RiMoonLine, RiSunLine } from "react-icons/ri";
import { guidePages, topicPath } from "./guide.mjs";
import { loadCatalog, matrixHtml } from "./matrix.mjs";
import { LANGS } from "../scripts/lib/shots.mjs";

const site = dirname(fileURLToPath(import.meta.url));
const repo = dirname(site);
const out = join(repo, "_site");
const base = "https://nabre.github.io/atollo/";
// English first: the root page, x-default and the keys every language must have.
const langs = LANGS;
const pathOf = (lang) => (lang === "en" ? "" : `${lang}/`);
// og:locale of each page.
const locales = { en: "en_US", it: "it_IT", fr: "fr_FR", de: "de_DE" };
// The words the page is found by, in English on every page (as the GitHub
// topics): the meta keywords and the JSON-LD's keywords.
const keywords = [
  "portable development environment",
  "portable dev environment",
  "zero install",
  "no admin rights",
  "USB development environment",
  "AI agents",
  "Claude Code",
  "Codex",
  "Antigravity",
  "Pi",
  "local LLM",
  "llama.cpp",
  "GGUF",
  "MCP servers",
  "VSCodium",
  "portable VS Code",
  "toolchain manager",
  "runtime manager",
  "version manager",
  "MySQL",
  "MongoDB",
  "SQLite",
  "phpMyAdmin",
  "Caddy",
  "Apache",
  "PHP",
  "Node.js",
  "Python",
  "Rust",
  "Go",
  "Java",
  "Windows",
  "macOS",
  "Linux",
  "ARM64",
  "Tauri",
].join(", ");

// The pages' models: a page (landing.html, guide.html) includes the
// partials (partials/head.html, header.html, footer.html) with {{> name}},
// then its values fill the {{key}}s.
const partials = Object.fromEntries(
  ["head", "header", "footer"].map((n) => [n, readFileSync(join(site, "partials", `${n}.html`), "utf8").replace(/\n$/, "")]),
);
const model = (file) =>
  readFileSync(join(site, file), "utf8").replace(/\{\{> (\w+)\}\}/g, (_, n) => {
    if (!(n in partials)) throw new Error(`${file}: no partial ${n}`);
    return partials[n];
  });
const landing = model("landing.html");
const strings = Object.fromEntries(
  langs.map((l) => [l, JSON.parse(readFileSync(join(site, "strings", `${l}.json`), "utf8"))]),
);

// Same keys in every language: a missing one would show as {{key}}.
const keys = Object.keys(strings.en).sort().join();
for (const l of langs) {
  if (Object.keys(strings[l]).sort().join() !== keys)
    throw new Error(`site/strings/${l}.json: keys differ from en.json`);
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
  it: () =>
    `<svg viewBox="0 0 3 2"><path fill="#009246" d="M0 0h1v2H0z"/><path fill="#fff" d="M1 0h1v2H1z"/><path fill="#ce2b37" d="M2 0h1v2H2z"/></svg>`,
  fr: () =>
    `<svg viewBox="0 0 3 2"><path fill="#0055a4" d="M0 0h1v2H0z"/><path fill="#fff" d="M1 0h1v2H1z"/><path fill="#ef4135" d="M2 0h1v2H2z"/></svg>`,
  de: () =>
    `<svg viewBox="0 0 5 3" preserveAspectRatio="none"><path fill="#000" d="M0 0h5v1H0z"/><path fill="#dd0000" d="M0 1h5v1H0z"/><path fill="#ffce00" d="M0 2h5v1H0z"/></svg>`,
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
const appLd = (lang) => ({
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
    image: `${base}assets/social.png`,
    screenshot: `${base}assets/shots/${lang}/shot-projects-dark.png`,
    author: { "@type": "Person", name: "nabre", url: "https://github.com/nabre" },
    keywords,
  });
const jsonLd = (lang) => JSON.stringify(appLd(lang)).replaceAll("<", "\\u003c");

// The root page (English, x-default) follows the system's language: the
// first of navigator.languages the site speaks, English if none, unless a
// language was picked in the menu (site.js keeps it in localStorage). Only
// the root page moves: the others are asked for by their address.
const langDetect =
  `<script>try{var l=localStorage.getItem("atollo-lang");if(!l){var s=${JSON.stringify(langs)};` +
  `var n=navigator.languages&&navigator.languages.length?navigator.languages:[navigator.language||""];` +
  `for(var i=0;i<n.length&&!l;i++){var p=String(n[i]).toLowerCase().split("-")[0];if(s.indexOf(p)>=0)l=p}}` +
  `if(l&&l!=="en"&&${JSON.stringify(langs.filter((l) => l !== "en"))}.indexOf(l)>=0)` +
  `location.replace(new URL(l+"/"+location.search+location.hash,location.href))}catch(e){}</script>`;

// A page of the site in one language: `path(l)` is its address in language
// l (from the site's root: "", "it/", "guide/isolation/"), so the language
// menu, canonical, og:url, the hreflang alternates and the sitemap all
// follow it; `values` fill its model's own {{key}}s and may replace the
// common ones (title, description, jsonLd...). (GUI-V7-12)
function renderPage({ lang, path, model: html, name, values: own }) {
  const here = path(lang);
  const root = "../".repeat(here.split("/").length - 1);
  const alternates = langs
    .map((l) => `<link rel="alternate" hreflang="${l}" href="${base}${path(l)}">`)
    .concat(`<link rel="alternate" hreflang="x-default" href="${base}${path("en")}">`)
    .join("\n");
  const ogLocales = [`<meta property="og:locale" content="${locales[lang]}">`]
    .concat(langs.filter((l) => l !== lang).map((l) => `<meta property="og:locale:alternate" content="${locales[l]}">`))
    .join("\n");
  const item = (l) =>
    l === lang
      ? `<li><span aria-current="page" lang="${l}">${flag(l)}<b>${l.toUpperCase()}</b>${strings[l].langName}</span></li>`
      : `<li><a href="${root}${path(l)}" hreflang="${l}" lang="${l}">${flag(l)}<b>${l.toUpperCase()}</b>${strings[l].langName}</a></li>`;
  const langLinks =
    `<details class="lang-menu"><summary aria-label="${escape(strings[lang].langLabel)}: ${strings[lang].langName}">${flag(lang)}<b>${lang.toUpperCase()}</b>` +
    `${chevron}</summary>` +
    `<ul>${langs.map(item).join("")}</ul></details>`;
  const values = {
    ...Object.fromEntries(Object.entries(strings[lang]).map(([k, v]) => [k, escape(v)])),
    lang,
    root,
    base,
    path: here,
    langPath: pathOf(lang),
    // The way back to the landing page, before the navigation's #anchors:
    // nothing on the landing page itself.
    home: here === pathOf(lang) ? "" : `${root}${pathOf(lang)}`,
    alternates,
    ogLocales,
    ogType: "website",
    keywords: escape(keywords),
    jsonLd: jsonLd(lang),
    langLinks,
    langDetect: "",
    headExtra: "",
    guideCurrent: "",
    ...icons,
    componentCount: String(catalog.components.length),
    // The agents are the catalog's "agents" group, like agents::IDS in
    // Rust: never a number written by hand (DOC-V7-01).
    agentCount: String(catalog.components.filter((c) => c.group === "agents").length),
    version,
    ...own,
  };
  const page = html.replace(/\{\{(\w+)\}\}/g, (_, key) => {
    if (!(key in values)) throw new Error(`${name}: no value for {{${key}}}`);
    return values[key];
  });
  mkdirSync(join(out, here), { recursive: true });
  writeFileSync(join(out, here, "index.html"), page);
}

/** Every page written, for the sitemap: its address in each language. */
const pages = [];

for (const lang of langs) {
  renderPage({
    lang,
    path: pathOf,
    model: landing,
    name: "landing.html",
    values: {
      langDetect: lang === "en" ? langDetect : "",
      toolsMatrix: matrixHtml(catalog, lang, strings[lang]),
    },
  });
}
pages.push(pathOf);

// The Guide, one page per topic and language (site/guide.mjs, D-164).
const guide = guidePages({ langs, strings, pathOf, out, model: model("guide.html"), base, jsonLdApp: appLd });
for (const page of guide.pages) renderPage(page);
for (const id of guide.topics) pages.push((l) => topicPath(pathOf, l, id));

cpSync(join(site, "assets"), join(out, "assets"), { recursive: true });
// Files that must sit at the site's root, as they are: the Google Search
// Console ownership check of https://nabre.github.io/atollo/ (keep it, or the
// property loses its verification).
cpSync(join(site, "root"), out, { recursive: true });
const brand = join(repo, "docs", "brand");
for (const f of [
  "wordmark-dark.svg",
  "wordmark-light.svg",
  "symbol.svg",
  "brand.png",
  "social.png",
  "hero-dark.svg",
  "hero-light.svg",
])
  cpSync(join(brand, f), join(out, "assets", f));
// The gallery's screenshots, in the page's language (D-143).
for (const l of langs) {
  const shots = join(site, "shots", l);
  mkdirSync(join(out, "assets", "shots", l), { recursive: true });
  for (const f of readdirSync(shots).filter((f) => f.endsWith(".png")))
    cpSync(join(shots, f), join(out, "assets", "shots", l, f));
}

// The English page lived in en/ until it moved to the root: old links land
// there, and search engines learn the new address.
mkdirSync(join(out, "en"), { recursive: true });
writeFileSync(
  join(out, "en", "index.html"),
  `<!doctype html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n<title>Atollo</title>\n<link rel="canonical" href="${base}">\n<meta name="robots" content="noindex">\n<meta http-equiv="refresh" content="0; url=${base}">\n</head>\n<body><a href="${base}">Atollo</a></body>\n</html>\n`,
);

// Every page in every language with its alternates, as the <link>s in the
// pages: submitted by hand in Google Search Console and Bing Webmaster Tools.
const today = new Date().toISOString().slice(0, 10);
const urlEntry = (path, lang) =>
  `  <url>\n    <loc>${base}${path(lang)}</loc>\n    <lastmod>${today}</lastmod>\n` +
  langs.map((l) => `    <xhtml:link rel="alternate" hreflang="${l}" href="${base}${path(l)}"/>\n`).join("") +
  `    <xhtml:link rel="alternate" hreflang="x-default" href="${base}${path("en")}"/>\n  </url>\n`;
writeFileSync(
  join(out, "sitemap.xml"),
  `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">\n${pages.flatMap((p) => langs.map((l) => urlEntry(p, l))).join("")}</urlset>\n`,
);

console.log(`_site: ${pages.length * langs.length} pages`);

// --check (the ui job of app.yml): the site built, then checked without
// publishing anything: every page in every language is there and in the
// sitemap, no {{key}} is left, and every relative link and image of the
// pages leads to a file of _site.
if (process.argv.includes("--check")) {
  const problems = [];
  const sitemap = readFileSync(join(out, "sitemap.xml"), "utf8");
  for (const path of pages) {
    for (const l of langs) {
      const file = join(out, path(l), "index.html");
      if (!existsSync(file)) {
        problems.push(`missing ${path(l)}index.html`);
        continue;
      }
      if (!sitemap.includes(`<loc>${base}${path(l)}</loc>`)) problems.push(`not in sitemap.xml: ${path(l)}`);
      const html = readFileSync(file, "utf8");
      if (/\{\{[\w> ]+\}\}/.test(html)) problems.push(`${path(l)}: a {{key}} is left`);
      for (const m of html.matchAll(/\s(?:href|src|srcset)="([^"]*)"/g)) {
        const link = (m[1] ?? "").split(/[?#]/)[0];
        if (link === "" || /^[a-z][\w+.-]*:/i.test(link)) continue; // an anchor, another site
        const target = join(out, path(l), link);
        const found = existsSync(target) && (!statSync(target).isDirectory() || existsSync(join(target, "index.html")));
        if (!found) problems.push(`${path(l)}: broken link ${link}`);
      }
    }
  }
  if (problems.length > 0) {
    console.error(problems.join("\n"));
    process.exit(1);
  }
  console.log(`check: ${pages.length * langs.length} pages, sitemap and links in order`);
}
