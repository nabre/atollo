// After `npm -w ui run build`: the Guide's chapters (guide/content/*.tsx)
// each in a chunk of its own, dist/assets/guide-<chapter>-*.js, and their
// common parts in guide-base. A pattern of ui/vite.config.ts that no longer
// matches would put them in the app's first chunk without any error
// (GUI-V7-08); this check fails instead. The ui job of app.yml runs it.
import { readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repo = dirname(dirname(fileURLToPath(import.meta.url)));
const chapters = readdirSync(join(repo, "guide", "content"))
  .filter((f) => f.endsWith(".tsx"))
  .map((f) => f.slice(0, -4));
const assets = readdirSync(join(repo, "dist", "assets")).filter((f) => f.endsWith(".js"));
const missing = [...chapters, "base"].filter(
  (name) => !assets.some((f) => f.startsWith(`guide-${name}-`)),
);
if (chapters.length === 0 || missing.length > 0) {
  console.error(`guide chunks missing: ${missing.join(", ") || "no chapters found"}`);
  process.exit(1);
}
console.log(`guide chunks: ${String(chapters.length)} chapters and guide-base`);
