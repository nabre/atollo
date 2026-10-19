"""Tool x system x architecture matrix of the catalog, as one HTML page.

python -I scripts/matrix/gen_matrix.py [<repo> [<out>]]
(default: this repository and docs/app/TOOLS-MATRIX.html)

The page is in English, as the documentation. Everything comes from the
code: availability, emulation, needs and sizes from
tauri/catalog/components.json with the logic of
Component::is_available_on (tauri/src/setup/catalog.rs), its Italian
reasons in English from tauri/i18n/catalog.json (as site/matrix.mjs); the retired
components from RETIRED in tauri/src/versions/disk.rs; sources and the
targets where a component is a conda environment that does not follow the
kit from scripts/matrix/sources.json, which test/rust/install/mod.rs checks
against the installers. The page has no date or commit: running the script
again without changes gives the same file.
"""
import html
import json
import os
import re
import sys

here = os.path.dirname(os.path.abspath(__file__))
repo = sys.argv[1] if len(sys.argv) > 1 else os.path.dirname(os.path.dirname(here))
out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(repo, "docs", "app", "TOOLS-MATRIX.html")


def read(*parts):
    with open(os.path.join(repo, *parts), encoding="utf-8") as f:
        return f.read()


cat = json.loads(read("tauri", "catalog", "components.json"))["components"]
SRC = json.loads(read("scripts", "matrix", "sources.json"))["components"]
I18N = json.loads(read("tauri", "i18n", "catalog.json"))


def en(it):
    """The catalog's English for an Italian string of components.json."""
    return I18N.get(it, {}).get("en", it)


missing = sorted({c["id"] for c in cat} ^ set(SRC))
if missing:
    sys.exit(f"scripts/matrix/sources.json and the catalog differ: {', '.join(missing)}")
retired = re.search(r"const RETIRED: \[&str; \d+\] = \[([^\]]*)\]", read("tauri", "src", "versions", "disk.rs"))
RETIRED = re.findall(r'"([^"]+)"', retired.group(1)) if retired else []

TARGETS = [
    ("win32-x64", "Windows", "x64"),
    ("win32-arm64", "Windows", "ARM64"),
    ("darwin-arm64", "macOS", "Apple Silicon"),
    ("darwin-x64", "macOS", "Intel"),
    ("linux-x64", "Linux", "x64"),
    ("linux-arm64", "Linux", "ARM64"),
]
# The app's own English names of the groups (ui/src/locales/en.json).
GROUPS = {
    "required": "Required",
    "app": "App",
    "agents": "Agents",
    "languages": "Languages and tools",
    "databases": "Databases",
}

byid = {c["id"]: c for c in cat}


def base_status(c, t):
    if c.get("targets") and t not in c["targets"]:
        return ("no", SRC[c["id"]].get("notOffered", "the catalog does not offer it on this system"))
    if t in c.get("unavailable", {}):
        return ("no", en(c["unavailable"][t]))
    if t == "win32-arm64" and c.get("noArm64"):
        return ("no", "no build for Windows ARM64, not even under emulation")
    if t == "win32-arm64" and c.get("x64Only"):
        return ("emu", "x64 build under emulation (Prism)")
    if t == "darwin-arm64" and c.get("rosetta"):
        return ("emu", "x64 build with Rosetta 2")
    return ("ok", "native build")


def needs_on(c, t):
    return list(c.get("needs", [])) + list(c.get("needsByTarget", {}).get(t, []))


def status(c, t):
    s = base_status(c, t)
    if s[0] == "no":
        return s
    for n in needs_on(c, t):
        if n in byid and base_status(byid[n], t)[0] == "no":
            return ("dep", f"needs {byid[n]['name']}, not available here")
    return s


def nonreloc(cid, t):
    return t in SRC[cid].get("notRelocatable", [])


def names(ids):
    return ", ".join(byid[i]["name"] for i in ids)


def where(ts):
    ts = set(ts)
    out = []
    for os_, prefix in (("macOS", "darwin-"), ("Linux", "linux-")):
        both = {t for t, _, _ in TARGETS if t.startswith(prefix)}
        if both <= ts:
            out.append(os_)
            ts -= both
    out = [" and ".join(out)] if out else []
    out += [f"{os_} {ar}" for t, os_, ar in TARGETS if t in ts]
    return ", ".join(out)


# Notes, from the same catalog.
notes = []
emu = [c["id"] for c in cat if c.get("x64Only")]
noarm = [c["id"] for c in cat if c.get("noArm64")]
arm_needs = [(c["id"], c["needsByTarget"]["win32-arm64"]) for c in cat if "win32-arm64" in c.get("needsByTarget", {})]
line = f"<b>Windows ARM64</b>: {html.escape(names(emu))} use the x64 build under emulation"
if noarm:
    line += f"; {html.escape(names(noarm))} has no build"
for cid, deps in arm_needs:
    line += f"; {html.escape(byid[cid]['name'])} also needs {html.escape(names(deps))}"
notes.append(line + ".")
rosetta = [c["id"] for c in cat if c.get("rosetta")]
notes.append(
    "<b>macOS Apple Silicon</b>: "
    + (f"{html.escape(names(rosetta))} with Rosetta 2." if rosetta else "all native, no component needs Rosetta 2.")
)
for c in cat:
    for t, why in c.get("unavailable", {}).items():
        notes.append(f"<b>{html.escape(c['name'])}</b> on {where([t])}: {html.escape(en(why))}.")
    if c.get("targets"):
        off = [t for t, _, _ in TARGETS if t not in c["targets"]]
        why = SRC[c["id"]].get("notOffered", "the catalog does not offer it")
        notes.append(f"<b>{html.escape(c['name'])}</b> is missing on {where(off)}: {html.escape(why)}.")
for c in cat:
    extra = {t: d for t, d in c.get("needsByTarget", {}).items() if t != "win32-arm64"}
    if extra:
        ts = sorted(extra, key=[t for t, _, _ in TARGETS].index)
        deps = sorted({d for v in extra.values() for d in v})
        notes.append(f"<b>{html.escape(c['name'])}</b> on {where(ts)} also needs {html.escape(names(deps))}.")
conda = [(cid, s["notRelocatable"]) for cid, s in SRC.items() if s.get("notRelocatable")]
notes.append(
    "<b>Not relocatable</b> (conda environments, <i class=\"tag\">conda</i>): "
    + "; ".join(f"{html.escape(byid[cid]['name'])} on {where(ts)}" for cid, ts in conda)
    + ". When the kit changes folder they are created again from the packages kept in the kit, without the network (D-92, D-118); everything else follows the kit."
)
if RETIRED:
    notes.append(
        f"<b>Removed from the catalog</b> (<code>RETIRED</code>, folders removed at startup): {html.escape(', '.join(RETIRED))}."
        " MySQL is the only SQL server (D-117), Caddy the default web server (D-120)."
    )
notes.append(
    "Source of the data: <code>tauri/catalog/components.json</code> (<code>targets</code>, <code>unavailable</code>, "
    "<code>noArm64</code>, <code>x64Only</code>, <code>rosetta</code>, <code>needs</code>, <code>needsByTarget</code>, "
    "<code>mb</code>) with the logic of <code>Component::is_available_on</code>; sources and non-relocatable components from "
    "<code>scripts/matrix/sources.json</code>, checked against the installers by <code>cargo test</code>."
)
NOTES = "\n".join(f"  <li>{n}</li>" for n in notes)

LABEL = {"ok": "Native", "emu": "x64 emulated", "no": "Not available", "dep": "Blocked"}
ICON = {"ok": "●", "emu": "◐", "no": "○", "dep": "◌"}

rows = []
counts = {t[0]: {"ok": 0, "emu": 0, "no": 0, "dep": 0} for t in TARGETS}
order = ["required", "app", "agents", "languages", "databases"]
for g in order:
    items = [c for c in cat if c["group"] == g]
    rows.append(f'<tr class="grp" data-group="{g}"><th colspan="{len(TARGETS) + 4}">{GROUPS[g]} <span>{len(items)}</span></th></tr>')
    for c in items:
        cid = c["id"]
        cells = []
        for t, _, _ in TARGETS:
            k, why = status(c, t)
            counts[t][k] += 1
            extra = ""
            tips = [why]
            ndeps = c.get("needsByTarget", {}).get(t)
            if ndeps and k != "no":
                tips.append("here it also needs " + names(ndeps))
                extra += '<i class="tag">+dep</i>'
            if k in ("ok", "emu") and nonreloc(cid, t):
                tips.append("conda environment, not relocatable: created again from the kept packages if the kit changes folder (D-92, D-118)")
                extra += '<i class="tag">conda</i>'
            tip = html.escape("; ".join(tips))
            cells.append(f'<td class="s-{k}" title="{tip}"><span class="pill"><b>{ICON[k]}</b>{LABEL[k]}</span>{extra}</td>')
        deps = names(n for n in c.get("needs", []) if n in byid)
        w, u = SRC[cid]["windows"], SRC[cid]["unix"]
        rows.append(
            f'<tr data-group="{g}" data-search="{html.escape((cid + " " + c["name"] + " " + w + " " + u).lower())}">'
            f'<th scope="row"><span class="nm">{html.escape(c["name"])}</span><code>{cid}</code></th>'
            f'<td class="num">{c.get("mb", "")}</td>'
            + "".join(cells)
            + f'<td class="dep">{html.escape(deps)}</td>'
            f'<td class="src"><span>Win:</span> {html.escape(w)}<br><span>Unix:</span> {html.escape(u)}</td></tr>'
        )

head_cells = "".join(
    f'<th class="tg"><span class="os">{os_}</span><span class="ar">{ar}</span><code>{t}</code></th>' for t, os_, ar in TARGETS
)
summary = "".join(
    f'<div class="card"><div class="cos">{os_} <em>{ar}</em></div><code>{t}</code>'
    f'<div class="bar">'
    + "".join(f'<span class="b-{k}" style="flex:{counts[t][k]}"></span>' for k in ("ok", "emu", "dep", "no") if counts[t][k])
    + f'</div><div class="nums"><span class="c-ok">{counts[t]["ok"]} native</span>'
    f'<span class="c-emu">{counts[t]["emu"]} emulated</span>'
    f'<span class="c-no">{counts[t]["no"] + counts[t]["dep"]} missing</span></div></div>'
    for t, os_, ar in TARGETS
)

page = f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Atollo tools matrix by system</title>
<meta name="description" content="Every component of Atollo's catalog on Windows, macOS and Linux, x64 and ARM64: native, emulated or not available, with its source.">
<link rel="canonical" href="https://nabre.github.io/atollo/#tools">
<style>
:root {{
  --bg:#f7f7f5; --panel:#fff; --ink:#1d1f23; --mute:#6b7079; --line:#e3e3df; --head:#efefeb;
  --ok:#1f7a4d; --ok-bg:#e3f3ea; --emu:#9a5b00; --emu-bg:#fbefd9; --no:#8a8f98; --no-bg:#f0f0ee;
  --dep:#a3364a; --dep-bg:#f8e4e8; --accent:#2f5bd3;
}}
@media (prefers-color-scheme: dark) {{ :root:not([data-theme="light"]) {{
  --bg:#141518; --panel:#1c1d21; --ink:#e6e6e3; --mute:#9a9ea6; --line:#2d2f35; --head:#23252a;
  --ok:#5fd39a; --ok-bg:#163326; --emu:#f0b45a; --emu-bg:#3a2a12; --no:#8a8f98; --no-bg:#24262b;
  --dep:#f08a9c; --dep-bg:#3b1d24; --accent:#7f9cff; }} }}
:root[data-theme="dark"] {{
  --bg:#141518; --panel:#1c1d21; --ink:#e6e6e3; --mute:#9a9ea6; --line:#2d2f35; --head:#23252a;
  --ok:#5fd39a; --ok-bg:#163326; --emu:#f0b45a; --emu-bg:#3a2a12; --no:#8a8f98; --no-bg:#24262b;
  --dep:#f08a9c; --dep-bg:#3b1d24; --accent:#7f9cff; }}
* {{ box-sizing:border-box; }}
body {{ margin:0; background:var(--bg); color:var(--ink); font:14px/1.45 system-ui,-apple-system,"Segoe UI",sans-serif; }}
main {{ max-width:1400px; margin:0 auto; padding:28px 16px 48px; }}
h1 {{ font-size:24px; margin:0 0 4px; letter-spacing:-.01em; }}
.sub {{ color:var(--mute); margin:0 0 20px; }}
code {{ font:12px ui-monospace,Consolas,monospace; color:var(--mute); }}
.cards {{ display:grid; grid-template-columns:repeat(auto-fill,minmax(200px,1fr)); gap:10px; margin-bottom:20px; }}
.card {{ background:var(--panel); border:1px solid var(--line); border-radius:10px; padding:12px; }}
.cos {{ font-weight:600; }} .cos em {{ font-style:normal; color:var(--mute); font-weight:500; }}
.bar {{ display:flex; height:8px; border-radius:4px; overflow:hidden; margin:10px 0 6px; gap:2px; }}
.b-ok {{ background:var(--ok); }} .b-emu {{ background:var(--emu); }} .b-no {{ background:var(--no); opacity:.5; }} .b-dep {{ background:var(--dep); }}
.nums {{ display:flex; gap:10px; font-size:12px; flex-wrap:wrap; }}
.c-ok {{ color:var(--ok); }} .c-emu {{ color:var(--emu); }} .c-no {{ color:var(--mute); }}
.tools {{ display:flex; gap:8px; flex-wrap:wrap; align-items:center; margin-bottom:12px; }}
.tools input {{ flex:1 1 220px; padding:8px 10px; border:1px solid var(--line); border-radius:8px; background:var(--panel); color:var(--ink); font:inherit; }}
.tools button {{ padding:7px 12px; border:1px solid var(--line); border-radius:8px; background:var(--panel); color:var(--ink); cursor:pointer; font:inherit; }}
.tools button[aria-pressed="true"] {{ border-color:var(--accent); color:var(--accent); }}
.legend {{ display:flex; gap:14px; flex-wrap:wrap; font-size:12px; color:var(--mute); margin:0 0 12px; }}
.wrap {{ overflow-x:auto; background:var(--panel); border:1px solid var(--line); border-radius:10px; }}
table {{ border-collapse:collapse; width:100%; min-width:1100px; }}
th, td {{ border-bottom:1px solid var(--line); padding:7px 9px; text-align:left; vertical-align:middle; }}
thead th {{ position:sticky; top:0; background:var(--head); font-size:12px; z-index:1; }}
th.tg {{ text-align:center; }} th.tg span {{ display:block; }} .os {{ font-weight:600; font-size:13px; }} .ar {{ color:var(--mute); font-weight:500; }}
tbody th[scope=row] {{ font-weight:500; white-space:nowrap; }} tbody th .nm {{ display:block; }}
tr.grp th {{ background:var(--bg); font-size:12px; text-transform:uppercase; letter-spacing:.05em; color:var(--mute); padding-top:14px; }}
tr.grp th span {{ font-weight:400; margin-left:6px; }}
td.num {{ text-align:right; color:var(--mute); font-variant-numeric:tabular-nums; }}
td[class^="s-"] {{ text-align:center; white-space:nowrap; cursor:help; }}
.pill {{ display:inline-flex; gap:5px; align-items:center; padding:2px 8px; border-radius:999px; font-size:12px; }}
.pill b {{ font-weight:400; }}
.s-ok .pill {{ background:var(--ok-bg); color:var(--ok); }}
.s-emu .pill {{ background:var(--emu-bg); color:var(--emu); }}
.s-no .pill {{ background:var(--no-bg); color:var(--no); }}
.s-dep .pill {{ background:var(--dep-bg); color:var(--dep); }}
.tag {{ display:inline-block; font-style:normal; font-size:10px; margin-left:4px; padding:0 5px; border:1px solid var(--line); border-radius:4px; color:var(--mute); }}
td.dep {{ color:var(--mute); font-size:12px; }}
td.src {{ font-size:12px; min-width:230px; }} td.src span {{ color:var(--mute); }}
tr.hide {{ display:none; }}
.notes {{ margin-top:22px; font-size:13px; color:var(--mute); }} .notes li {{ margin:4px 0; }}
</style>
</head>
<body>
<main>
<h1>Tools × operating system × architecture matrix</h1>
<p class="sub">{len(cat)} components of <code>tauri/catalog/components.json</code> · 6 targets · generated by <code>scripts/matrix/gen_matrix.py</code></p>
<div class="cards">{summary}</div>
<div class="tools">
  <input id="q" type="search" placeholder="Search a component or source…" aria-label="Search">
  <button data-f="all" aria-pressed="true">All</button>
  {''.join(f'<button data-f="{g}" aria-pressed="false">{GROUPS[g]}</button>' for g in order)}
</div>
<div class="legend">
  <span>● <b style="color:var(--ok)">Native</b>: a build for the architecture</span>
  <span>◐ <b style="color:var(--emu)">x64 emulated</b>: x64 build on Windows ARM64 (Prism)</span>
  <span>◌ <b style="color:var(--dep)">Blocked</b>: a dependency is missing on that target</span>
  <span>○ Not available</span>
  <span><i class="tag">+dep</i> one more dependency on that target</span>
  <span><i class="tag">conda</i> not relocatable, created again from the kept packages (D-92, D-118)</span>
  <span>Hover a cell for the reason.</span>
</div>
<div class="wrap">
<table>
<thead><tr><th>Component</th><th>MB<br>(x64)</th>{head_cells}<th>Depends on</th><th>Source</th></tr></thead>
<tbody>
{chr(10).join(rows)}
</tbody>
</table>
</div>
<ul class="notes">
{NOTES}
</ul>
</main>
<script>
const q = document.getElementById('q'); let f = 'all';
const rows = [...document.querySelectorAll('tbody tr')];
function apply() {{
  const s = q.value.trim().toLowerCase();
  const vis = new Set();
  rows.forEach(r => {{
    if (r.classList.contains('grp')) return;
    const ok = (f === 'all' || r.dataset.group === f) && (!s || r.dataset.search.includes(s));
    r.classList.toggle('hide', !ok); if (ok) vis.add(r.dataset.group);
  }});
  rows.filter(r => r.classList.contains('grp')).forEach(r => r.classList.toggle('hide', !vis.has(r.dataset.group)));
}}
q.addEventListener('input', apply);
document.querySelectorAll('.tools button').forEach(b => b.addEventListener('click', () => {{
  f = b.dataset.f;
  document.querySelectorAll('.tools button').forEach(x => x.setAttribute('aria-pressed', x === b));
  apply();
}}));
</script>
</body>
</html>
"""
with open(out, "w", encoding="utf-8", newline="\n") as f:
    f.write(page)
for t, _, _ in TARGETS:
    print(t, counts[t])
