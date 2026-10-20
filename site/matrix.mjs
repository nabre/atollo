// The component × system matrix of the site, from the app's own catalog:
// tauri/catalog/components.json for what exists where, tauri/i18n/catalog.json
// for the descriptions and reasons in en/fr/de. Availability follows
// Component::is_available_on (tauri/src/setup/catalog.rs), as
// scripts/matrix/gen_matrix.py does for docs/app/TOOLS-MATRIX.html.
import { readFileSync } from "node:fs";
import { join } from "node:path";

export const TARGETS = [
  ["win32-x64", "Windows", "x64"],
  ["win32-arm64", "Windows", "ARM64"],
  ["darwin-arm64", "macOS", "Apple Silicon"],
  ["darwin-x64", "macOS", "Intel"],
  ["linux-x64", "Linux", "x64"],
  ["linux-arm64", "Linux", "ARM64"],
];

// Finer kinds than the catalog's groups (where "languages" holds almost
// everything). A component not listed here falls back to its group's kind.
export const KINDS = ["system", "agents", "languages", "tools", "web", "databases"];
export const KIND_OF = {
  pwsh: "system", gh: "system", git: "system", "vscodium-server": "system",
  uv: "tools", pnpm: "tools", cmake: "tools", android: "tools", typst: "tools", tinytex: "tools",
  apache: "web", caddy: "web",
};
export const KIND_OF_GROUP = { required: "system", app: "system", agents: "agents", languages: "languages", databases: "databases" };

const MARK = { ok: "✓", emu: "≈", no: "✕", dep: "✕" };

const escape = (s) => String(s).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;");

export function loadCatalog(repo) {
  const components = JSON.parse(readFileSync(join(repo, "tauri", "catalog", "components.json"), "utf8")).components;
  const i18n = JSON.parse(readFileSync(join(repo, "tauri", "i18n", "catalog.json"), "utf8"));
  return { components, i18n };
}

// The status of c on target t, without its needs: [kind, reason].
function baseStatus(c, t, s) {
  if (c.targets && !c.targets.includes(t)) return ["no", s.mxNotBuilt];
  if (c.unavailable && t in c.unavailable) return ["no", { catalog: c.unavailable[t] }];
  if (t === "win32-arm64" && c.noArm64) return ["no", s.mxNoArm64];
  if (t === "win32-arm64" && c.x64Only) return ["emu", s.mxEmulated];
  return ["ok", s.mxNative];
}

function status(c, t, byId, s) {
  const base = baseStatus(c, t, s);
  if (base[0] === "no") return base;
  const needs = [...(c.needs || []), ...((c.needsByTarget || {})[t] || [])];
  for (const n of needs) {
    if (byId[n] && baseStatus(byId[n], t, s)[0] === "no") return ["dep", s.mxNeeds.replace("{0}", byId[n].name)];
  }
  return base;
}

export function matrixHtml({ components, i18n }, lang, s) {
  const tr = (it) => (lang === "it" ? it : (i18n[it] && i18n[it][lang]) || it);
  const reason = (r) => (typeof r === "string" ? r : tr(r.catalog));
  const byId = Object.fromEntries(components.map((c) => [c.id, c]));
  const kindOf = (c) => KIND_OF[c.id] || KIND_OF_GROUP[c.group] || "tools";

  const osHead = [["Windows", 2], ["macOS", 2], ["Linux", 2]].map(([os, n]) => `<th colspan="${n}" class="os-head">${os}</th>`).join("");
  // data-target lets assets/download.js mark the visitor's column.
  const archHead = TARGETS.map(([t, , arch]) => `<th class="arch" title="${t}" data-target="${t}">${escape(arch)}<span class="mx-here">${escape(s.mxHere)}</span></th>`).join("");

  const body = KINDS.map((kind) => {
    const items = components.filter((c) => kindOf(c) === kind);
    if (!items.length) return "";
    const rows = items.map((c) => {
      const cells = TARGETS.map(([t, os, arch]) => {
        const [k, why] = status(c, t, byId, s);
        const tip = `${os} ${arch}: ${reason(why)}`;
        return `<td class="mx-${k}" data-target="${t}" title="${escape(tip)}"><span aria-hidden="true">${MARK[k]}</span><span class="sr">${escape(tip)}</span></td>`;
      }).join("");
      const name = c.home ? `<a href="${escape(c.home)}">${escape(c.name)}</a>` : escape(c.name);
      return `<tr><th scope="row"><span class="mx-name">${name}</span><span class="mx-desc">${escape(tr(c.desc))}</span></th>${cells}</tr>`;
    }).join("\n");
    return `<tbody><tr class="mx-kind"><th colspan="${TARGETS.length + 1}" scope="rowgroup">${escape(s[`mxKind_${kind}`])} <span>${items.length}</span></th></tr>\n${rows}</tbody>`;
  }).join("\n");

  return `<div class="mx-scroll"><table class="mx">
<thead><tr><th rowspan="2" class="mx-corner">${escape(s.mxComponent)}</th>${osHead}</tr><tr>${archHead}</tr></thead>
${body}
</table></div>
<p class="mx-legend"><span class="mx-ok">✓</span> ${escape(s.mxLegendOk)} <span class="mx-emu">≈</span> ${escape(s.mxLegendEmu)} <span class="mx-no">✕</span> ${escape(s.mxLegendNo)}</p>`;
}
