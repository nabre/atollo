// The screenshots of the site's gallery («Dentro l'app»), in the dark and
// the light theme, from the app's browser preview (sample data): 1280×820 at
// 2× (2560×1640: the site crops the title and status bars, `.crop`).
//
//   npm -w ui run dev                 (the preview on 127.0.0.1:5173)
//   node site/shots/make.mjs          (with playwright-core installed next to
//                                      it or in a folder of its own; Edge or
//                                      Chrome from the system, no download)
//
// Writes <lang>/shot-<page>-<dark|light>.png here, the app speaking each of
// the site's four languages (D-143); site/build.mjs copies them for the page
// of that language and each README shows the Projects one of its own.
// SHOTS_LANGS=en,it limits the languages, SHOTS_URL=http://127.0.0.1:5199
// takes the preview from another address.
import { chromium } from "playwright-core";
import { mkdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const langs = (process.env.SHOTS_LANGS ?? "en,it,fr,de").split(",");
const preview = process.env.SHOTS_URL ?? "http://127.0.0.1:5173";
const channel = process.env.SHOTS_BROWSER ?? (process.platform === "win32" ? "msedge" : "chrome");
const browser = await chromium.launch({ channel });

async function shoot(lang, scheme) {
  // The app's labels in that language (ui/src/locales: Italian key -> text).
  const catalog =
    lang === "it" ? {} : JSON.parse(readFileSync(join(here, "..", "..", "ui", "src", "locales", `${lang}.json`), "utf8"));
  const tr = (it) => catalog[it] ?? it;
  const out = join(here, lang);
  mkdirSync(out, { recursive: true });
  const page = await browser.newPage({ viewport: { width: 1280, height: 820 }, deviceScaleFactor: 2, colorScheme: scheme });
  await page.goto(`${preview}/?lang=${lang}`, { waitUntil: "networkidle" });
  await page.waitForTimeout(800);
  const nav = (name) => page.getByRole("navigation", { name: tr("Pagine") }).getByRole("button", { name: tr(name), exact: true });
  const settle = () => page.waitForTimeout(700);
  const activity = new RegExp(`^${tr("Attività")}`);

  // Projects: the quick panel, with the Activity panel open.
  await page.getByRole("button", { name: activity }).click();
  await settle();
  await page.screenshot({ path: `${out}/shot-projects-${scheme}.png` });
  await page.getByRole("button", { name: activity }).click();
  await settle();

  // Agents: limits and usage of Claude Code.
  await nav("Agenti").click();
  await settle();
  await page.screenshot({ path: `${out}/shot-agents-${scheme}.png` });

  // Tools: components × systems.
  await nav("Strumenti").click();
  await settle();
  await page.getByRole("button", { name: new RegExp(tr("Matrice")) }).first().click();
  await settle();
  await page.screenshot({ path: `${out}/shot-tools-${scheme}.png` });
  await page.close();
}

try {
  for (const lang of langs) for (const scheme of ["dark", "light"]) await shoot(lang, scheme);
} finally {
  await browser.close();
}
console.log(`site/shots: ${langs.join(", ")}`);
