// The component × system matrix of the site, from the app's own catalog:
// tauri/catalog/components.json for what exists where, tauri/i18n/catalog.json
// for the descriptions and reasons in en/fr/de. Availability follows
// Component::is_available_on (tauri/src/setup/catalog.rs):
// `status` is written once here, and scripts/matrix/gen_matrix.mjs uses it
// too for docs/app/TOOLS-MATRIX.html.
import { readFileSync } from "node:fs";
import { join } from "node:path";

// The four systems Docker Sandboxes runs on, the only ones Atollo supports
// (D-176): Docker builds sbx for no other.
export const TARGETS = [
  ["win32-x64", "Windows", "x64"],
  ["darwin-arm64", "macOS", "Apple Silicon"],
  ["linux-x64", "Linux", "x64"],
  ["linux-arm64", "Linux", "ARM64"],
];

// Finer kinds than the catalog's groups (where "languages" holds almost
// everything). A component not listed here falls back to its group's kind.
export const KINDS = ["system", "agents", "ai", "languages", "tools", "web", "databases", "dbviewers"];
export const KIND_OF = {
  pwsh: "system", gh: "system", git: "system", "vscodium-server": "system", sandboxes: "system",
  uv: "tools", pnpm: "tools", cmake: "tools", android: "tools", typst: "tools", tinytex: "tools",
  apache: "web", caddy: "web",
  phpmyadmin: "dbviewers", "mongo-express": "dbviewers",
};
export const KIND_OF_GROUP = { required: "system", app: "system", agents: "agents", ai: "ai", languages: "languages", databases: "databases" };

const MARK = { ok: "✓", no: "✕", dep: "✕", skip: "–" };

const escape = (s) => String(s).replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;");

export function loadCatalog(repo) {
  const components = JSON.parse(readFileSync(join(repo, "tauri", "catalog", "components.json"), "utf8")).components;
  const i18n = JSON.parse(readFileSync(join(repo, "tauri", "i18n", "catalog.json"), "utf8"));
  return { components, i18n };
}

// The status of c on target t, without its needs: [kind, reason]. `why`
// gives the reasons: notBuilt(c), unavailable and notNeeded (the catalog's
// Italian text), native, needs(name). "skip": not needed there, the system
// does its job (`notNeeded`, ConPTY on macOS and Linux, D-177): never
// missing.
export function baseStatus(c, t, why) {
  if (c.notNeeded && t in c.notNeeded) return ["skip", why.notNeeded(c.notNeeded[t])];
  if (c.targets && !c.targets.includes(t)) return ["no", why.notBuilt(c)];
  if (c.unavailable && t in c.unavailable) return ["no", why.unavailable(c.unavailable[t])];
  return ["ok", why.native];
}

// The status of c on target t: "dep" when a component it needs there is
// not available there.
export function status(c, t, byId, why) {
  const base = baseStatus(c, t, why);
  if (base[0] !== "ok") return base;
  const needs = [...(c.needs || []), ...((c.needsByTarget || {})[t] || [])];
  for (const n of needs) {
    if (byId[n] && baseStatus(byId[n], t, why)[0] === "no") return ["dep", why.needs(byId[n].name)];
  }
  return base;
}

export function matrixHtml({ components, i18n }, lang, s) {
  const tr = (it) => (lang === "it" ? it : (i18n[it] && i18n[it][lang]) || it);
  const why = {
    notBuilt: () => s.mxNotBuilt,
    unavailable: tr,
    notNeeded: (it) => s.mxNotNeeded.replace("{0}", tr(it)),
    native: s.mxNative,
    needs: (name) => s.mxNeeds.replace("{0}", name),
  };
  const byId = Object.fromEntries(components.map((c) => [c.id, c]));
  const kindOf = (c) => KIND_OF[c.id] || KIND_OF_GROUP[c.group] || "tools";

  const osHead = ["Windows", "macOS", "Linux"].map((os) => [os, TARGETS.filter(([, o]) => o === os).length]).map(([os, n]) => `<th colspan="${n}" class="os-head">${os}</th>`).join("");
  // data-target lets assets/download.js mark the visitor's column.
  const archHead = TARGETS.map(([t, , arch]) => `<th class="arch" title="${t}" data-target="${t}">${escape(arch)}<span class="mx-here">${escape(s.mxHere)}</span></th>`).join("");

  const body = KINDS.map((kind) => {
    const items = components.filter((c) => kindOf(c) === kind);
    if (!items.length) return "";
    const rows = items.map((c) => {
      const cells = TARGETS.map(([t, os, arch]) => {
        const [k, reason] = status(c, t, byId, why);
        const tip = `${os} ${arch}: ${reason}`;
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
<p class="mx-legend"><span class="mx-ok">✓</span> ${escape(s.mxLegendOk)} <span class="mx-skip">–</span> ${escape(s.mxLegendSkip)} <span class="mx-no">✕</span> ${escape(s.mxLegendNo)}</p>`;
}
