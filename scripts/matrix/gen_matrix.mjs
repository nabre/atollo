// Tool x system x architecture matrix of the catalog, as one HTML page: the
// systems of TARGETS in site/matrix.mjs, those Docker Sandboxes runs on
// (D-176).
//
//   node scripts/matrix/gen_matrix.mjs [<repo> [<out>]]
//   (default: this repository and docs/app/TOOLS-MATRIX.html)
//
// The page is in English, as the documentation. Everything comes from the
// code: availability, Rosetta 2, needs and sizes from
// tauri/catalog/components.json with the rules of site/matrix.mjs (`status`,
// which follows Component::is_available_on of tauri/src/setup/catalog.rs:
// one copy for the site and this page), its Italian reasons in English from
// tauri/i18n/catalog.json; the retired components from RETIRED in
// tauri/src/versions/disk.rs; sources and the targets where a component is a
// conda environment that does not follow the kit from
// scripts/matrix/sources.json, which test/rust/install/mod.rs checks against
// the installers. The page has no date or commit: running the script again
// without changes gives the same file.
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadCatalog, status as statusOn, TARGETS } from "../../site/matrix.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const repo = process.argv[2] ?? join(here, "..", "..");
const out = process.argv[3] ?? join(repo, "docs", "app", "TOOLS-MATRIX.html");
const read = (...parts) => readFileSync(join(repo, ...parts), "utf8");

// As Python's html.escape: & < > " and '.
const esc = (s) =>
  String(s)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#x27;");

const { components: cat, i18n: I18N } = loadCatalog(repo);
const SRC = JSON.parse(read("scripts", "matrix", "sources.json")).components;

// The catalog's English for an Italian string of components.json.
const en = (it) => I18N[it]?.en ?? it;

const ids = new Set(cat.map((c) => c.id));
const missing = [...new Set([...ids, ...Object.keys(SRC)])].filter((id) => !ids.has(id) || !(id in SRC)).sort();
if (missing.length) {
  console.error(`scripts/matrix/sources.json and the catalog differ: ${missing.join(", ")}`);
  process.exit(1);
}
const retired = /const RETIRED: \[&str; \d+\] = \[([^\]]*)\]/.exec(read("tauri", "src", "versions", "disk.rs"));
const RETIRED = retired ? [...retired[1].matchAll(/"([^"]+)"/g)].map((m) => m[1]) : [];

// The app's own English names of the groups (ui/src/locales/en.json).
const GROUPS = {
  required: "Required",
  app: "App",
  agents: "Agents",
  ai: "Local models",
  languages: "Languages and tools",
  databases: "Databases",
};

const byid = Object.fromEntries(cat.map((c) => [c.id, c]));
const WHY = {
  notBuilt: (c) => SRC[c.id].notOffered ?? "the catalog does not offer it on this system",
  unavailable: en,
  rosetta: "x64 build with Rosetta 2",
  native: "native build",
  needs: (name) => `needs ${name}, not available here`,
};
const status = (c, t) => statusOn(c, t, byid, WHY);
const nonreloc = (cid, t) => (SRC[cid].notRelocatable ?? []).includes(t);
const names = (list) => [...list].map((i) => byid[i].name).join(", ");

function where(list) {
  const ts = new Set(list);
  let out = [];
  for (const [os, prefix] of [
    ["macOS", "darwin-"],
    ["Linux", "linux-"],
  ]) {
    const both = TARGETS.map(([t]) => t).filter((t) => t.startsWith(prefix));
    if (both.every((t) => ts.has(t))) {
      out.push(os);
      for (const t of both) ts.delete(t);
    }
  }
  out = out.length ? [out.join(" and ")] : [];
  out.push(...TARGETS.filter(([t]) => ts.has(t)).map(([, os, ar]) => `${os} ${ar}`));
  return out.join(", ");
}

// Notes, from the same catalog.
const notes = [];
const rosetta = cat.filter((c) => c.rosetta).map((c) => c.id);
notes.push(
  "<b>macOS Apple Silicon</b>: " +
    (rosetta.length ? `${esc(names(rosetta))} with Rosetta 2.` : "all native, no component needs Rosetta 2."),
);
for (const c of cat) {
  for (const [t, why] of Object.entries(c.unavailable ?? {})) {
    notes.push(`<b>${esc(c.name)}</b> on ${where([t])}: ${esc(en(why))}.`);
  }
  if (c.targets?.length) {
    const off = TARGETS.map(([t]) => t).filter((t) => !c.targets.includes(t));
    // `targets` naming every system (Docker Sandboxes, D-176) is no note.
    if (!off.length) continue;
    const why = SRC[c.id].notOffered ?? "the catalog does not offer it";
    notes.push(`<b>${esc(c.name)}</b> is missing on ${where(off)}: ${esc(why)}.`);
  }
}
const order = TARGETS.map(([t]) => t);
for (const c of cat) {
  const extra = Object.entries(c.needsByTarget ?? {});
  if (extra.length) {
    const ts = extra.map(([t]) => t).sort((a, b) => order.indexOf(a) - order.indexOf(b));
    const deps = [...new Set(extra.flatMap(([, d]) => d))].sort();
    notes.push(`<b>${esc(c.name)}</b> on ${where(ts)} also needs ${esc(names(deps))}.`);
  }
}
const conda = Object.entries(SRC).filter(([, s]) => s.notRelocatable?.length);
notes.push(
  '<b>Not relocatable</b> (conda environments, <i class="tag">conda</i>): ' +
    conda.map(([cid, s]) => `${esc(byid[cid].name)} on ${where(s.notRelocatable)}`).join("; ") +
    ". When the kit changes folder they are created again from the packages kept in the kit, without the network (D-92, D-118); everything else follows the kit.",
);
if (RETIRED.length) {
  notes.push(
    `<b>Removed from the catalog</b> (<code>RETIRED</code>, folders removed at startup): ${esc(RETIRED.join(", "))}.` +
      " MySQL is the only SQL server (D-117), Caddy the default web server (D-120).",
  );
}
notes.push(
  "Source of the data: <code>tauri/catalog/components.json</code> (<code>targets</code>, <code>unavailable</code>, " +
    "<code>rosetta</code>, <code>needs</code>, <code>needsByTarget</code>, " +
    "<code>mb</code>) with the logic of <code>Component::is_available_on</code>; sources and non-relocatable components from " +
    "<code>scripts/matrix/sources.json</code>, checked against the installers by <code>cargo test</code>.",
);
const NOTES = notes.map((n) => `  <li>${n}</li>`).join("\n");

const LABEL = { ok: "Native", emu: "Rosetta 2", no: "Not available", dep: "Blocked" };
const ICON = { ok: "●", emu: "◐", no: "○", dep: "◌" };

const rows = [];
const counts = Object.fromEntries(TARGETS.map(([t]) => [t, { ok: 0, emu: 0, no: 0, dep: 0 }]));
const groupOrder = ["required", "app", "agents", "ai", "languages", "databases"];
for (const g of groupOrder) {
  const items = cat.filter((c) => c.group === g);
  rows.push(`<tr class="grp" data-group="${g}"><th colspan="${TARGETS.length + 4}">${GROUPS[g]} <span>${items.length}</span></th></tr>`);
  for (const c of items) {
    const cid = c.id;
    const cells = TARGETS.map(([t]) => {
      const [k, why] = status(c, t);
      counts[t][k] += 1;
      let extra = "";
      const tips = [why];
      const ndeps = c.needsByTarget?.[t];
      if (ndeps?.length && k !== "no") {
        tips.push("here it also needs " + names(ndeps));
        extra += '<i class="tag">+dep</i>';
      }
      if ((k === "ok" || k === "emu") && nonreloc(cid, t)) {
        tips.push("conda environment, not relocatable: created again from the kept packages if the kit changes folder (D-92, D-118)");
        extra += '<i class="tag">conda</i>';
      }
      const tip = esc(tips.join("; "));
      return `<td class="s-${k}" title="${tip}"><span class="pill"><b>${ICON[k]}</b>${LABEL[k]}</span>${extra}</td>`;
    });
    const deps = names((c.needs ?? []).filter((n) => n in byid));
    const [w, u] = [SRC[cid].windows, SRC[cid].unix];
    rows.push(
      `<tr data-group="${g}" data-search="${esc((cid + " " + c.name + " " + w + " " + u).toLowerCase())}">` +
        `<th scope="row"><span class="nm">${esc(c.name)}</span><code>${cid}</code></th>` +
        `<td class="num">${c.mb ?? ""}</td>` +
        cells.join("") +
        `<td class="dep">${esc(deps)}</td>` +
        `<td class="src"><span>Win:</span> ${esc(w)}<br><span>Unix:</span> ${esc(u)}</td></tr>`,
    );
  }
}

const headCells = TARGETS.map(
  ([t, os, ar]) => `<th class="tg"><span class="os">${os}</span><span class="ar">${ar}</span><code>${t}</code></th>`,
).join("");
const summary = TARGETS.map(
  ([t, os, ar]) =>
    `<div class="card"><div class="cos">${os} <em>${ar}</em></div><code>${t}</code>` +
    `<div class="bar">` +
    ["ok", "emu", "dep", "no"]
      .filter((k) => counts[t][k])
      .map((k) => `<span class="b-${k}" style="flex:${counts[t][k]}"></span>`)
      .join("") +
    `</div><div class="nums"><span class="c-ok">${counts[t].ok} native</span>` +
    `<span class="c-emu">${counts[t].emu} Rosetta 2</span>` +
    `<span class="c-no">${counts[t].no + counts[t].dep} missing</span></div></div>`,
).join("");
const page = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Atollo tools matrix by system</title>
<meta name="description" content="Every component of Atollo's catalog on the systems of Docker Sandboxes (Windows x64, macOS Apple Silicon, Linux x64 and ARM64): native, with Rosetta 2 or not available, with its source.">
<link rel="canonical" href="https://nabre.github.io/atollo/#tools">
<style>
:root {
  --bg:#f7f7f5; --panel:#fff; --ink:#1d1f23; --mute:#6b7079; --line:#e3e3df; --head:#efefeb;
  --ok:#1f7a4d; --ok-bg:#e3f3ea; --emu:#9a5b00; --emu-bg:#fbefd9; --no:#8a8f98; --no-bg:#f0f0ee;
  --dep:#a3364a; --dep-bg:#f8e4e8; --accent:#2f5bd3;
}
@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) {
  --bg:#141518; --panel:#1c1d21; --ink:#e6e6e3; --mute:#9a9ea6; --line:#2d2f35; --head:#23252a;
  --ok:#5fd39a; --ok-bg:#163326; --emu:#f0b45a; --emu-bg:#3a2a12; --no:#8a8f98; --no-bg:#24262b;
  --dep:#f08a9c; --dep-bg:#3b1d24; --accent:#7f9cff; } }
:root[data-theme="dark"] {
  --bg:#141518; --panel:#1c1d21; --ink:#e6e6e3; --mute:#9a9ea6; --line:#2d2f35; --head:#23252a;
  --ok:#5fd39a; --ok-bg:#163326; --emu:#f0b45a; --emu-bg:#3a2a12; --no:#8a8f98; --no-bg:#24262b;
  --dep:#f08a9c; --dep-bg:#3b1d24; --accent:#7f9cff; }
* { box-sizing:border-box; }
body { margin:0; background:var(--bg); color:var(--ink); font:14px/1.45 system-ui,-apple-system,"Segoe UI",sans-serif; }
main { max-width:1400px; margin:0 auto; padding:28px 16px 48px; }
h1 { font-size:24px; margin:0 0 4px; letter-spacing:-.01em; }
.sub { color:var(--mute); margin:0 0 20px; }
code { font:12px ui-monospace,Consolas,monospace; color:var(--mute); }
.cards { display:grid; grid-template-columns:repeat(auto-fill,minmax(200px,1fr)); gap:10px; margin-bottom:20px; }
.card { background:var(--panel); border:1px solid var(--line); border-radius:10px; padding:12px; }
.cos { font-weight:600; } .cos em { font-style:normal; color:var(--mute); font-weight:500; }
.bar { display:flex; height:8px; border-radius:4px; overflow:hidden; margin:10px 0 6px; gap:2px; }
.b-ok { background:var(--ok); } .b-emu { background:var(--emu); } .b-no { background:var(--no); opacity:.5; } .b-dep { background:var(--dep); }
.nums { display:flex; gap:10px; font-size:12px; flex-wrap:wrap; }
.c-ok { color:var(--ok); } .c-emu { color:var(--emu); } .c-no { color:var(--mute); }
.tools { display:flex; gap:8px; flex-wrap:wrap; align-items:center; margin-bottom:12px; }
.tools input { flex:1 1 220px; padding:8px 10px; border:1px solid var(--line); border-radius:8px; background:var(--panel); color:var(--ink); font:inherit; }
.tools button { padding:7px 12px; border:1px solid var(--line); border-radius:8px; background:var(--panel); color:var(--ink); cursor:pointer; font:inherit; }
.tools button[aria-pressed="true"] { border-color:var(--accent); color:var(--accent); }
.legend { display:flex; gap:14px; flex-wrap:wrap; font-size:12px; color:var(--mute); margin:0 0 12px; }
.wrap { overflow-x:auto; background:var(--panel); border:1px solid var(--line); border-radius:10px; }
table { border-collapse:collapse; width:100%; min-width:1100px; }
th, td { border-bottom:1px solid var(--line); padding:7px 9px; text-align:left; vertical-align:middle; }
thead th { position:sticky; top:0; background:var(--head); font-size:12px; z-index:1; }
th.tg { text-align:center; } th.tg span { display:block; } .os { font-weight:600; font-size:13px; } .ar { color:var(--mute); font-weight:500; }
tbody th[scope=row] { font-weight:500; white-space:nowrap; } tbody th .nm { display:block; }
tr.grp th { background:var(--bg); font-size:12px; text-transform:uppercase; letter-spacing:.05em; color:var(--mute); padding-top:14px; }
tr.grp th span { font-weight:400; margin-left:6px; }
td.num { text-align:right; color:var(--mute); font-variant-numeric:tabular-nums; }
td[class^="s-"] { text-align:center; white-space:nowrap; cursor:help; }
.pill { display:inline-flex; gap:5px; align-items:center; padding:2px 8px; border-radius:999px; font-size:12px; }
.pill b { font-weight:400; }
.s-ok .pill { background:var(--ok-bg); color:var(--ok); }
.s-emu .pill { background:var(--emu-bg); color:var(--emu); }
.s-no .pill { background:var(--no-bg); color:var(--no); }
.s-dep .pill { background:var(--dep-bg); color:var(--dep); }
.tag { display:inline-block; font-style:normal; font-size:10px; margin-left:4px; padding:0 5px; border:1px solid var(--line); border-radius:4px; color:var(--mute); }
td.dep { color:var(--mute); font-size:12px; }
td.src { font-size:12px; min-width:230px; } td.src span { color:var(--mute); }
tr.hide { display:none; }
.notes { margin-top:22px; font-size:13px; color:var(--mute); } .notes li { margin:4px 0; }
</style>
</head>
<body>
<main>
<h1>Tools × operating system × architecture matrix</h1>
<p class="sub">${cat.length} components of <code>tauri/catalog/components.json</code> · ${TARGETS.length} targets, the systems of Docker Sandboxes · generated by <code>scripts/matrix/gen_matrix.mjs</code></p>
<div class="cards">${summary}</div>
<div class="tools">
  <input id="q" type="search" placeholder="Search a component or source…" aria-label="Search">
  <button data-f="all" aria-pressed="true">All</button>
  ${groupOrder.map((g) => `<button data-f="${g}" aria-pressed="false">${GROUPS[g]}</button>`).join("")}
</div>
<div class="legend">
  <span>● <b style="color:var(--ok)">Native</b>: a build for the architecture</span>
  <span>◐ <b style="color:var(--emu)">Rosetta 2</b>: x64 build on macOS Apple Silicon</span>
  <span>◌ <b style="color:var(--dep)">Blocked</b>: a dependency is missing on that target</span>
  <span>○ Not available</span>
  <span><i class="tag">+dep</i> one more dependency on that target</span>
  <span><i class="tag">conda</i> not relocatable, created again from the kept packages (D-92, D-118)</span>
  <span>Hover a cell for the reason.</span>
</div>
<div class="wrap">
<table>
<thead><tr><th>Component</th><th>MB<br>(x64)</th>${headCells}<th>Depends on</th><th>Source</th></tr></thead>
<tbody>
${rows.join("\n")}
</tbody>
</table>
</div>
<ul class="notes">
${NOTES}
</ul>
</main>
<script>
const q = document.getElementById('q'); let f = 'all';
const rows = [...document.querySelectorAll('tbody tr')];
function apply() {
  const s = q.value.trim().toLowerCase();
  const vis = new Set();
  rows.forEach(r => {
    if (r.classList.contains('grp')) return;
    const ok = (f === 'all' || r.dataset.group === f) && (!s || r.dataset.search.includes(s));
    r.classList.toggle('hide', !ok); if (ok) vis.add(r.dataset.group);
  });
  rows.filter(r => r.classList.contains('grp')).forEach(r => r.classList.toggle('hide', !vis.has(r.dataset.group)));
}
q.addEventListener('input', apply);
document.querySelectorAll('.tools button').forEach(b => b.addEventListener('click', () => {
  f = b.dataset.f;
  document.querySelectorAll('.tools button').forEach(x => x.setAttribute('aria-pressed', x === b));
  apply();
}));
</script>
</body>
</html>
`;
writeFileSync(out, page);
for (const [t] of TARGETS) {
  const c = counts[t];
  console.log(t, `{'ok': ${c.ok}, 'emu': ${c.emu}, 'no': ${c.no}, 'dep': ${c.dep}}`);
}
