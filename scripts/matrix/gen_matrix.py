"""Tool x system x architecture matrix of the catalog, as one HTML page.

python scripts/matrix/gen_matrix.py <repo> docs/app/MATRICE-STRUMENTI.html

Availability comes from tauri/catalog/components.json with the logic of
Component::is_available_on (tauri/src/setup/catalog.rs); sizes and sources
are those of docs/app/CATALOG.md, kept here by hand.
"""
import json, sys, html, subprocess, datetime

repo = sys.argv[1]
out = sys.argv[2]
cat = json.load(open(f"{repo}/tauri/catalog/components.json", encoding="utf-8"))["components"]
commit = subprocess.run(["git", "-C", repo, "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()

TARGETS = [
    ("win32-x64", "Windows", "x64"),
    ("win32-arm64", "Windows", "ARM64"),
    ("darwin-arm64", "macOS", "Apple Silicon"),
    ("darwin-x64", "macOS", "Intel"),
    ("linux-x64", "Linux", "x64"),
    ("linux-arm64", "Linux", "ARM64"),
]
GROUPS = {
    "required": "Obbligatori",
    "app": "App",
    "agents": "Agenti",
    "languages": "Linguaggi e strumenti",
    "databases": "Database",
}
MB = {
    "gh": 40, "pwsh": 110, "vscodium-server": 250, "claude": 250, "codex": 440, "antigravity": 190,
    "git": 350, "python": 70, "node": 110, "uv": 35, "php": 100, "apache": 45, "nginx": 10, "caddy": 45,
    "rust": 920, "go": 250, "java": 300, "kotlin": 80, "android": 600, "dotnet": 250, "dart": 200,
    "zig": 200, "julia": 500, "bun": 90, "deno": 100, "typst": 50, "tinytex": 280, "pnpm": 10,
    "cmake": 100, "cc": 720, "ruby": 270, "r": 1100, "mysql": 520, "mongodb": 570,
    "phpmyadmin": 60, "sqlite": 15,
}
# Fonte su Windows / su macOS e Linux (CATALOG.md §1 «Fonti per sistema» e §2)
SRC = {
    "claude": ("npm, binario nativo", "npm, binario nativo"),
    "codex": ("npm, binario nativo", "npm, binario nativo"),
    "antigravity": ("manifesto Google, exe", "manifesto Google, .tar.gz"),
    "git": ("PortableGit (git-for-windows)", "dugite-native"),
    "python": ("python-build-standalone", "python-build-standalone"),
    "node": ("nodejs.org", "nodejs.org"),
    "uv": ("GitHub astral-sh/uv", "GitHub astral-sh/uv"),
    "gh": ("GitHub cli/cli", "GitHub cli/cli"),
    "php": ("windows.php.net + VC++ locale", "static-php-cli"),
    "apache": ("Apache Lounge (modulo PHP)", "build del kit, rilascio httpd-v<v> (php-fpm)"),
    "nginx": ("nginx.org", "—"),
    "caddy": ("GitHub caddyserver/caddy", "GitHub caddyserver/caddy"),
    "rust": ("rustup (gnu su x64, gnullvm su ARM64)", "rustup"),
    "go": ("go.dev", "go.dev"),
    "java": ("Adoptium Temurin + Maven + Gradle", "Adoptium Temurin + Maven + Gradle"),
    "kotlin": ("GitHub JetBrains/kotlin", "GitHub JetBrains/kotlin"),
    "android": ("cmdline-tools Google", "cmdline-tools Google"),
    "dotnet": ("dotnet release-metadata", "dotnet release-metadata"),
    "dart": ("dart-archive", "dart-archive"),
    "zig": ("ziglang.org", "ziglang.org"),
    "julia": ("julialang-s3", "julialang-s3"),
    "bun": ("GitHub oven-sh/bun", "GitHub oven-sh/bun"),
    "deno": ("GitHub denoland/deno", "GitHub denoland/deno"),
    "typst": ("GitHub typst/typst", "GitHub typst/typst"),
    "tinytex": ("GitHub rstudio/tinytex-releases", "GitHub rstudio/tinytex-releases"),
    "pnpm": ("registro npm", "registro npm"),
    "cmake": ("GitHub Kitware/CMake + Ninja", "GitHub Kitware/CMake + Ninja"),
    "cc": ("llvm-mingw (ucrt)", "shim su zig cc + GNU make (conda-forge)"),
    "ruby": ("RubyInstaller2", "conda-forge ruby"),
    "r": ("conda-forge r-base", "conda-forge r-base"),
    "mysql": ("cdn.mysql.com 8.4 / 9.7", "cdn.mysql.com 8.4 / 9.7"),
    "mongodb": ("build MongoDB", "macOS: build MongoDB; Linux: conda-forge"),
    "phpmyadmin": ("phpmyadmin.net", "phpmyadmin.net"),
    "sqlite": ("sqlite.org", "sqlite.org; linux-arm64: conda-forge"),
    "pwsh": ("GitHub PowerShell/PowerShell", "GitHub PowerShell/PowerShell"),
    "vscodium-server": ("VSCodium reh-web, versione fissata", "VSCodium reh-web, versione fissata"),
}
NOT_RELOCATABLE = {  # D-92: reinstallati se il kit cambia cartella
    "r": {"all"}, "ruby": {"unix"}, "mongodb": {"linux"}, "cc": {"unix"},
    "sqlite": {"linux-arm64"},
}

byid = {c["id"]: c for c in cat}


def base_status(c, t):
    if c.get("targets") and t not in c["targets"]:
        return ("no", "non nel catalogo per questo sistema")
    if t in c.get("unavailable", {}):
        return ("no", c["unavailable"][t])
    if t == "win32-arm64" and c.get("noArm64"):
        return ("no", "nessuna build per Windows ARM64")
    if t == "win32-arm64" and c.get("x64Only"):
        return ("emu", "build x64 in emulazione (Prism)")
    if t == "darwin-arm64" and c.get("rosetta"):
        return ("emu", "build x64 con Rosetta 2")
    return ("ok", "build nativa")


def needs_on(c, t):
    return list(c.get("needs", [])) + list(c.get("needsByTarget", {}).get(t, []))


def status(c, t):
    s = base_status(c, t)
    if s[0] == "no":
        return s
    for n in needs_on(c, t):
        if n in byid and base_status(byid[n], t)[0] == "no":
            return ("dep", f"richiede {byid[n]['name']}, non disponibile qui")
    return s


def nonreloc(cid, t):
    r = NOT_RELOCATABLE.get(cid, set())
    return ("all" in r or t in r or ("unix" in r and not t.startswith("win"))
            or ("linux" in r and t.startswith("linux")))


LABEL = {"ok": "Nativo", "emu": "x64 emulato", "no": "Non disponibile", "dep": "Bloccato"}
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
            notes = [why]
            ndeps = c.get("needsByTarget", {}).get(t)
            if ndeps and k != "no":
                notes.append("qui serve anche " + ", ".join(byid[d]["name"] for d in ndeps))
                extra += '<i class="tag">+dip</i>'
            if k in ("ok", "emu") and nonreloc(cid, t):
                notes.append("non rilocabile: reinstallato se il kit cambia cartella (D-92)")
                extra += '<i class="tag">conda</i>'
            tip = html.escape("; ".join(notes))
            cells.append(f'<td class="s-{k}" title="{tip}"><span class="pill"><b>{ICON[k]}</b>{LABEL[k]}</span>{extra}</td>')
        deps = ", ".join(byid[n]["name"] for n in c.get("needs", []) if n in byid) or ""
        w, u = SRC.get(cid, ("", ""))
        rows.append(
            f'<tr data-group="{g}" data-search="{html.escape((cid + " " + c["name"] + " " + w + " " + u).lower())}">'
            f'<th scope="row"><span class="nm">{html.escape(c["name"])}</span><code>{cid}</code></th>'
            f'<td class="num">{MB.get(cid, "")}</td>'
            + "".join(cells)
            + f'<td class="dep">{html.escape(deps)}</td>'
            f'<td class="src"><span>Win:</span> {html.escape(w)}<br><span>Unix:</span> {html.escape(u)}</td></tr>'
        )

head_cells = "".join(
    f'<th class="tg"><span class="os">{os}</span><span class="ar">{ar}</span><code>{t}</code></th>' for t, os, ar in TARGETS
)
summary = "".join(
    f'<div class="card"><div class="cos">{os} <em>{ar}</em></div><code>{t}</code>'
    f'<div class="bar">'
    + "".join(f'<span class="b-{k}" style="flex:{counts[t][k]}"></span>' for k in ("ok", "emu", "dep", "no") if counts[t][k])
    + f'</div><div class="nums"><span class="c-ok">{counts[t]["ok"]} nativi</span>'
    f'<span class="c-emu">{counts[t]["emu"]} emulati</span>'
    f'<span class="c-no">{counts[t]["no"] + counts[t]["dep"]} assenti</span></div></div>'
    for t, os, ar in TARGETS
)
today = datetime.date.today().strftime("%d/%m/%Y")

page = f"""<!doctype html>
<html lang="it">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Matrice strumenti per sistema</title>
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
<h1>Matrice strumenti × sistema operativo × architettura</h1>
<p class="sub">{len(cat)} componenti di <code>tauri/catalog/components.json</code> · 6 target · commit <code>{commit}</code> · generata il {today}</p>
<div class="cards">{summary}</div>
<div class="tools">
  <input id="q" type="search" placeholder="Cerca componente o fonte…" aria-label="Cerca">
  <button data-f="all" aria-pressed="true">Tutti</button>
  {''.join(f'<button data-f="{g}" aria-pressed="false">{GROUPS[g]}</button>' for g in order)}
</div>
<div class="legend">
  <span>● <b style="color:var(--ok)">Nativo</b>: build per l'architettura</span>
  <span>◐ <b style="color:var(--emu)">x64 emulato</b>: build x64 su Windows ARM64 (Prism)</span>
  <span>◌ <b style="color:var(--dep)">Bloccato</b>: manca una dipendenza su quel target</span>
  <span>○ Non disponibile</span>
  <span><i class="tag">+dip</i> dipendenza in più su quel target</span>
  <span><i class="tag">conda</i> non rilocabile (D-92)</span>
  <span>Passa il mouse su una cella per il motivo.</span>
</div>
<div class="wrap">
<table>
<thead><tr><th>Componente</th><th>MB<br>(x64)</th>{head_cells}<th>Dipende da</th><th>Fonte</th></tr></thead>
<tbody>
{chr(10).join(rows)}
</tbody>
</table>
</div>
<ul class="notes">
  <li><b>Windows ARM64</b>: PHP, Apache, Nginx, MySQL, MongoDB, Android SDK, Julia, TinyTeX e l'editor usano la build x64 in emulazione; R non ha build. Rust usa l'host <code>aarch64-pc-windows-gnullvm</code> con llvm-mingw di C/C++ come linker.</li>
  <li><b>Apache</b>: su Windows Apache Lounge con il modulo PHP; su macOS e Linux la build del kit con <code>php-fpm</code> (D-116), offerta lì quando il primo rilascio <code>httpd-v2.4.69-1</code> è fissato nel catalogo. Si propone solo per i progetti con regole proprie nel <code>.htaccess</code>; Caddy è il server web predefinito su tutti i sistemi.</li>
  <li><b>Nginx</b>: solo Windows (D-111), da togliere dopo la migrazione dei pool a Caddy (issue #4).</li>
  <li><b>MySQL</b> è l'unico server SQL del kit (D-117): MariaDB non si offre più, i suoi dati passano a MySQL al primo avvio; phpMyAdmin apre MySQL su tutti i sistemi.</li>
  <li><b>Android SDK</b>: Google non pubblica platform-tools e build-tools per Linux ARM64.</li>
  <li><b>C/C++</b> su macOS e Linux: shim su <code>zig cc</code> del kit (richiede Zig); i framework Apple vogliono Xcode CLT.</li>
  <li>Fonte dei dati: <code>components.json</code> (campi <code>targets</code>, <code>unavailable</code>, <code>noArm64</code>, <code>x64Only</code>, <code>rosetta</code>, <code>needs</code>, <code>needsByTarget</code>) con la logica di <code>Component::is_available_on</code> in <code>tauri/src/setup/catalog.rs</code>; dimensioni e fonti da <code>docs/app/CATALOG.md</code>.</li>
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
open(out, "w", encoding="utf-8").write(page)
for t, os, ar in TARGETS:
    print(t, counts[t])
