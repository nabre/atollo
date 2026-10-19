// The screenshots of the in-app guide and of docs/app/GUIDE*.md
// (ui/public/guide/<lang>/*.webp, D-136): the browser preview on its sample
// data, in the light theme, 1280×800 at 1.25× (1600×1000), without the
// preview's badge, in each
// of the four languages (?lang=, the sample data's engine texts too).
//
//   npm -w ui run dev                    (the preview on 127.0.0.1:5173)
//   node scripts/guide-shots/make.mjs    (playwright-core next to it or in a
//                                         folder of its own, as site/shots;
//                                         Edge or Chrome from the system;
//                                         Python with Pillow for the webp)
//
// GUIDE_SHOTS_LANGS=en,it limits the languages.
import { chromium } from "playwright-core";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const langs = (process.env.GUIDE_SHOTS_LANGS ?? "it,en,fr,de").split(",");
const channel = process.env.SHOTS_BROWSER ?? (process.platform === "win32" ? "msedge" : "chrome");
const browser = await chromium.launch({ channel });
const png = mkdtempSync(join(tmpdir(), "guide-shots-"));

async function shoot(lang) {
  // The app's labels in that language (ui/src/locales: Italian key -> text).
  const catalog =
    lang === "it" ? {} : JSON.parse(readFileSync(join(repo, "ui", "src", "locales", `${lang}.json`), "utf8"));
  const tr = (it) => catalog[it] ?? it;
  const page = await browser.newPage({
    viewport: { width: 1280, height: 800 },
    deviceScaleFactor: 1.25,
    colorScheme: "light",
  });
  await page.goto(`http://127.0.0.1:5173/?lang=${lang}`, { waitUntil: "networkidle" });
  await page.waitForTimeout(800);
  await page.getByText(tr("Anteprima nel browser: dati di esempio"), { exact: true }).evaluate((el) => {
    el.style.visibility = "hidden";
  });
  const settle = () => page.waitForTimeout(700);
  const nav = async (name) => {
    await page
      .getByRole("navigation", { name: tr("Pagine") })
      .getByRole("button", { name: tr(name), exact: true })
      .click();
    await settle();
  };
  const shot = async (name) => {
    await page.screenshot({ path: join(png, `${lang}-${name}.png`) });
  };
  const tab = async (name) => {
    await page.getByText(tr(name), { exact: true }).first().click();
    await settle();
  };

  // Projects: the quick panel.
  await shot("projects-home");

  // A new project from a framework: Laravel chosen.
  await page
    .getByRole("button", { name: tr("Web con PHP") })
    .first()
    .click();
  await settle();
  await shot("project-new");

  // A project's page, tab by tab.
  await page.getByText("sito-vetrina", { exact: true }).first().click();
  await settle();
  await shot("project-run");
  await tab("Sito web");
  await shot("project-site");
  await tab("Componenti");
  await shot("project-versions");
  await tab("Database");
  await shot("project-db");

  for (const [name, label] of [
    ["agents", "Agenti"],
    ["databases", "Database"],
    ["servers", "Server Web"],
    ["terminals", "Terminali"],
    ["tools", "Strumenti"],
    ["settings", "Impostazioni"],
    ["eject", "Rimuovi l'unità"],
  ]) {
    await nav(label);
    await shot(name);
  }
  await page.close();
}

try {
  for (const lang of langs) await shoot(lang);
} finally {
  await browser.close();
}

// PNG to webp, as the guide shows them (Pillow, quality 85).
const convert = `
import glob, os, sys
from PIL import Image
src, out = sys.argv[1], sys.argv[2]
for f in glob.glob(os.path.join(src, "*.png")):
    lang, name = os.path.basename(f)[:-4].split("-", 1)
    os.makedirs(os.path.join(out, lang), exist_ok=True)
    Image.open(f).save(os.path.join(out, lang, name + ".webp"), "WEBP", quality=85, method=6)
`;
const done = spawnSync("python", ["-c", convert, png, join(repo, "ui", "public", "guide")], { stdio: "inherit" });
rmSync(png, { recursive: true, force: true });
if (done.status !== 0) throw new Error("webp conversion failed (Python with Pillow)");
console.log(`ui/public/guide: ${langs.join(", ")}`);
