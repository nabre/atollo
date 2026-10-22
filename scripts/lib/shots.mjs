// What the screenshot scripts share (site/shots/make.mjs, site/shots/social.mjs,
// scripts/guide-shots/make.mjs) and the site's languages (site/build.mjs):
// the browser, the preview's address, the app's labels in each language and
// the activity bar's buttons. Each script imports playwright-core itself and
// passes `chromium` in, so the package still resolves next to that script.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const REPO = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

// The site's and the app's languages, English first (the site's root page).
export const LANGS = ["en", "it", "fr", "de"];

// The languages a script shoots: `env` (for example SHOTS_LANGS=en,it) or all.
export const langsFrom = (env, all = LANGS) => (process.env[env] ?? all.join(",")).split(",");

// The browser preview (npm -w ui run dev); SHOTS_URL takes it from another address.
export const PREVIEW = process.env.SHOTS_URL ?? "http://127.0.0.1:5173";

// Edge or Chrome from the system, never a download (SHOTS_BROWSER=msedge|chrome).
export const launch = (chromium) =>
  chromium.launch({ channel: process.env.SHOTS_BROWSER ?? (process.platform === "win32" ? "msedge" : "chrome") });

// The app's labels in that language (ui/src/locales: Italian key -> text).
export function catalog(lang) {
  const texts = lang === "it" ? {} : JSON.parse(readFileSync(join(REPO, "ui", "src", "locales", `${lang}.json`), "utf8"));
  return (it) => texts[it] ?? it;
}

// A new page on the preview speaking `lang`, settled after it loads.
export async function openPreview(browser, lang, options) {
  const page = await browser.newPage(options);
  await page.goto(`${PREVIEW}/?lang=${lang}`, { waitUntil: "networkidle" });
  await page.waitForTimeout(800);
  return page;
}

// The activity bar's button of a page, by its Italian label.
export const nav = (page, tr, name) =>
  page.getByRole("navigation", { name: tr("Pagine") }).getByRole("button", { name: tr(name), exact: true });
