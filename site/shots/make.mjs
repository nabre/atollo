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
import { mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { catalog, langsFrom, launch, nav as navOf, openPreview } from "../../scripts/lib/shots.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const langs = langsFrom("SHOTS_LANGS");
const browser = await launch(chromium);

async function shoot(lang, scheme) {
  const tr = catalog(lang);
  const out = join(here, lang);
  mkdirSync(out, { recursive: true });
  const page = await openPreview(browser, lang, {
    viewport: { width: 1280, height: 820 },
    deviceScaleFactor: 2,
    colorScheme: scheme,
  });
  const nav = (name) => navOf(page, tr, name);
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
