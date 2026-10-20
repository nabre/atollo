// The social card (docs/brand/social.png, 1280×640): the GitHub repository's
// social preview and the site's og:image / Twitter card. The hero's dark
// background, the wordmark alone (MARKETING.md, «Logo nelle intestazioni»),
// the category, one line of what Atollo is, and the Projects screenshot.
//
//   node site/shots/social.mjs   (after make.mjs: it uses en/shot-projects-dark.png;
//                                 playwright-core and the browser as make.mjs)
import { chromium } from "playwright-core";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const shots = dirname(fileURLToPath(import.meta.url));
const repo = join(shots, "..", "..");
// Inline: a page set from a string loads no file:// URL.
const url = (...p) => {
  const type = p.at(-1).endsWith(".png") ? "image/png" : "font/woff2";
  return `data:${type};base64,${readFileSync(join(repo, ...p)).toString("base64")}`;
};
const svg = (name) => readFileSync(join(repo, "docs", "brand", name), "utf8");

const html = `<!doctype html><html><head><meta charset="utf-8"><style>
@font-face { font-family: Geist; src: url("${url("site", "assets", "fonts", "geist-latin-wght-normal.woff2")}") format("woff2"); font-weight: 100 900; }
@font-face { font-family: "Geist Mono"; src: url("${url("site", "assets", "fonts", "geist-mono-latin-wght-normal.woff2")}") format("woff2"); font-weight: 100 900; }
* { box-sizing: border-box; margin: 0; }
body { width: 1280px; height: 640px; overflow: hidden; background: #141414; color: #cccccc; font-family: Geist, sans-serif; position: relative; }
.bg { position: absolute; inset: 0; }
.bg svg { width: 100%; height: 100%; }
.text { position: absolute; left: 72px; top: 84px; width: 560px; }
.mark svg { width: 300px; height: auto; display: block; }
.kicker { margin-top: 40px; font: 600 15px/1 "Geist Mono", monospace; letter-spacing: .14em; text-transform: uppercase; color: #6ee7b7; }
h1 { margin-top: 18px; font-size: 46px; line-height: 1.12; font-weight: 700; color: #f8fafc; letter-spacing: -.02em; }
p { margin-top: 22px; font-size: 21px; line-height: 1.45; color: #b8b8b8; }
.targets { position: absolute; left: 72px; bottom: 60px; font: 500 15px/1 "Geist Mono", monospace; color: #8a8a8a; letter-spacing: .02em; }
.shot { position: absolute; left: 680px; top: 96px; width: 760px; border-radius: 12px; overflow: hidden; border: 1px solid #2f2f2f; box-shadow: 0 30px 80px rgba(0,0,0,.55); }
.shot img { display: block; width: 100%; }
</style></head><body>
<div class="bg">${svg("hero-dark.svg")}</div>
<div class="text">
  <div class="mark">${svg("wordmark-dark.svg")}</div>
  <div class="kicker">Universal Portable Dev &amp; AI Workspace</div>
  <h1>Dev &amp; AI, all in one folder, on any machine.</h1>
  <p>Editor, 20+ languages, local databases and the Claude Code, Codex and Antigravity agents. Zero install, zero admin rights.</p>
</div>
<div class="targets">Windows · macOS · Linux — x64 &amp; ARM64 · MIT</div>
<div class="shot"><img src="${url("site", "shots", "en", "shot-projects-dark.png")}"></div>
</body></html>`;

const channel = process.env.SHOTS_BROWSER ?? (process.platform === "win32" ? "msedge" : "chrome");
const browser = await chromium.launch({ channel });
const page = await browser.newPage({ viewport: { width: 1280, height: 640 }, deviceScaleFactor: 1 });
await page.setContent(html, { waitUntil: "load" });
await page.evaluate(() => document.fonts.ready);
await page.screenshot({ path: join(repo, "docs", "brand", "social.png") });
await browser.close();
console.log("docs/brand/social.png");
