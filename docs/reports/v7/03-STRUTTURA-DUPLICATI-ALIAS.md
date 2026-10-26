# 03 — Struttura, Duplicati e Alias (`v7`)

> **Riferimento**: `HEAD 1f84370`. Misure prese con `grep` e conteggio delle righe su ogni file `.rs/.ts/.tsx/.mjs/.cjs/.py/.sh` di `tauri/src`, `ui/src`, `test`, `site` e `scripts`, e lettura delle configurazioni che decidono gli alias (`ui/tsconfig.json`, `test/ui/tsconfig.json`, `ui/vite.config.ts`, `vitest.config.ts`, `eslint.config.mjs`, il risolutore di `scripts/guide-docs.cjs`).
> Nessuno di questi rilievi cambia il comportamento: sono refactoring da fare con le prove verdi prima e dopo, un commit per rilievo, nel ramo `refactor/v7-structure` (ondata 2 del [piano](./05-PIANO-ESECUTIVO-V7.md)). Nessun rilievo ripete la `v6`.

---

## 1. Tabella riassuntiva

| ID | Grav. | Tipo | Dove | Titolo |
|---|:---:|---|---|---|
| `STR-V7-01` | MEDIA | struttura | [`tauri/src/projects/run.rs:225`](../../../tauri/src/projects/run.rs#L225) | Le sonde `responds`/`is_port_free` stanno nel runner dei progetti ma servono a db e llm |
| `STR-V7-02` | MEDIA | struttura | [`tauri/src/projects/run.rs:509`](../../../tauri/src/projects/run.rs#L509) | `projects/run.rs` (1256 righe): `Runner::start` da solo è ~300 righe |
| `STR-V7-03` | MEDIA | struttura | [`tauri/src/env.rs:324`](../../../tauri/src/env.rs#L324) | `KitEnv::build` è una funzione di ~580 righe |
| `STR-V7-04` | BASSA | struttura | [`tauri/src/projects/caddy.rs:101`](../../../tauri/src/projects/caddy.rs#L101) | `caddy.rs` (864) mescola socket privati, Caddyfile e riga di comando |
| `STR-V7-05` | BASSA | struttura | [`tauri/src/install/mod.rs:691`](../../../tauri/src/install/mod.rs#L691) | `install/mod.rs` (961) tiene insieme registro, flusso e parti |
| `STR-V7-06` | BASSA | struttura | [`tauri/src/kit.rs:156`](../../../tauri/src/kit.rs#L156) | `kit.rs` (806): `Os`/`Arch`/`Target` e ricerca della radice in sottomoduli |
| `STR-V7-07` | BASSA | struttura | [`tauri/src/main.rs:70`](../../../tauri/src/main.rs#L70) | `main.rs` (822): CLI e costruzione dell'app in moduli propri |
| `STR-V7-08` | BASSA | alias | [`tauri/src/setup/mod.rs:8`](../../../tauri/src/setup/mod.rs#L8) | `crate::setup::catalog::…` scritto per intero 83 volte, `crate::router::fs::…` 31 |
| `STR-V7-09` | BASSA | duplicato | [`tauri/src/agents/sync/mod.rs:107`](../../../tauri/src/agents/sync/mod.rs#L107) | Funzioni che inoltrano soltanto, dove basterebbe un `pub use` |
| `STR-V7-10` | BASSA | duplicato | [`tauri/src/detect/mod.rs:220`](../../../tauri/src/detect/mod.rs#L220) | Quattro `fn tr` identiche attorno a `crate::i18n::t` |
| `STR-V7-11` | BASSA | duplicato | [`tauri/src/projects/run.rs:184`](../../../tauri/src/projects/run.rs#L184) | 17 calcoli a mano del tempo epoch, quattro copie private di `now_ms` |
| `STR-V7-12` | MEDIA | duplicato | [`scripts/matrix/gen_matrix.py:70`](../../../scripts/matrix/gen_matrix.py#L70) | Regole di disponibilità dei componenti scritte tre volte: Rust, `site/matrix.mjs`, `gen_matrix.py` |
| `STR-V7-13` | BASSA | duplicato | [`site/shots/make.mjs:21`](../../../site/shots/make.mjs#L21) | Tre script Playwright ripetono avvio del browser, catalogo delle lingue e navigazione |
| `STR-V7-14` | BASSA | duplicato | [`ui/src/projects/delete.tsx:16`](../../../ui/src/projects/delete.tsx#L16) | `size()` identica in due file; formattatori di byte sparsi |
| `STR-V7-15` | BASSA | duplicato | [`ui/src/databases/page.tsx:169`](../../../ui/src/databases/page.tsx#L169) | Mappa dei nomi dei motori e validazione della porta ripetute |
| `STR-V7-16` | BASSA | duplicato | [`ui/src/workbench/controls.tsx:867`](../../../ui/src/workbench/controls.tsx#L867) | Unione dei toni di `Dot` riscritta in cinque punti |
| `STR-V7-17` | BASSA | struttura | [`ui/src/workbench/controls.tsx:307`](../../../ui/src/workbench/controls.tsx#L307) | `controls.tsx` (878) riunisce pulsanti, campi e indicatori |
| `STR-V7-18` | BASSA | struttura | [`ui/src/guide/model.ts:148`](../../../ui/src/guide/model.ts#L148) | `model.ts` della Guida mescola tipi e ~755 righe di dati; `inside.tsx` contiene componenti |
| `STR-V7-19` | MEDIA | struttura | [`ui/src/servers.tsx:291`](../../../ui/src/servers.tsx#L291) | 20 file sciolti nella radice di `ui/src` |
| `STR-V7-20` | MEDIA | alias | [`ui/tsconfig.json:2`](../../../ui/tsconfig.json#L2) | Alias `@/`, `@guide`, `@test` al posto dei relativi profondi |
| `STR-V7-21` | MEDIA | duplicato | [`test/rust/servers/mod.rs:595`](../../../test/rust/servers/mod.rs#L595) | Preparazione del kit ripetuta 119 volte nelle prove Rust; `fake_server` e `wait_for` copiati |
| `STR-V7-22` | BASSA | struttura | [`test/rust/servers/mod.rs:27`](../../../test/rust/servers/mod.rs#L27) | `test/rust/servers/mod.rs` (2433 righe) copre pool, processi e configurazione insieme |

---

## 2. Rust

### 2.1 File grandi da dividere (sottomoduli con `pub use`, percorsi invariati)

Regola comune: il file diventa una cartella `x/mod.rs` con sottomoduli, e `mod.rs` riesporta con `pub use` tutto ciò che oggi è pubblico, così nessun `crate::…` esterno cambia. La divisione segue i confini già marcati dai commenti.

| ID | File (righe) | Divisione proposta |
|---|---|---|
| `STR-V7-02` | `projects/run.rs` (1256) | `run/{mod.rs` (State, RunInfo, Event, Entry, `Runner::new/list/stop/clear/rename`), `ports.rs` (Ports, ProjectPorts, `load_ports`, `saved_ports`, `assign_port`, `set_port`, `save_ports`), `output.rs` (ANSI, LOCAL_URL, `split_lines`, `read_stream`, `append*`, `on_network`), `start.rs` (`start`, `finish`, `watch_port`, `ready`), `pool.rs` (`attach_pool`, `pooled`, `reconcile_pools`, `wait_with_php`, SiteMode)`}`. 12 riferimenti esterni a `crate::projects::run` restano validi |
| `STR-V7-03` | `env.rs` (1289) | `env/{mod.rs` (Host, KitEnv, `build` che chiama una funzione privata per sezione), `langs.rs` (Python, Node, PHP, Rust, Gradle, R), `unix.rs` (CC_VARS, `unix_cc_env`, `macos_rust_linker`, `git_unix_env`, `gem_home`, `linux_lib_dirs`, `has_libicu`), `path.rs` (PATH), `secrets.rs` (`is_secret`, `masked`, `mask_secrets`)`}`. Le sezioni oggi sono solo commenti: agenti 376-404, linguaggi 462-568, variabili per OS (D-112) 568-642, PATH 642-727, Android 727, conda 782, database 806, proxy e certificati 859-897 |
| `STR-V7-04` | `projects/caddy.rs` (864) | `caddy/{mod.rs, socket.rs` (PrivateSocket, `socket_bases`, `private_leaf`, `sweep_stale_leaves`, `admin_socket`; codice sensibile, isolato), `conf.rs` (`global_options`, `site_block`, `project_conf`, `pool_conf`, `resolve_aliases`), `command.rs` (`run_args`, `reload_args`, `caddy_command`)`}` |
| `STR-V7-05` | `install/mod.rs` (961) | `mod.rs` tiene tipi, tratti e registro (Ctx, Dest, Sign, Part, Release, Installer); `flow.rs` (`work_dir`, `fetch`, `install`, `install_now`, `relocate`, `install_line*`), `parts.rs` (`install_parts`, `fetch_part`, `place_part`, `put_file`), `keys.rs` (`check_key*`, `target_of`, `arch_of`) |
| `STR-V7-06` | `kit.rs` (806) | `kit/{mod.rs` (Kit, KitError), `target.rs` (Os, Arch, Target: ~330 righe importate quasi ovunque), `root.rs` (`root_from`, bundle e AppImage, `launch_problem`), `names.rs` (`validate_project_name`)`}` |
| `STR-V7-07` | `main.rs` (822) | aiuti della CLI (`models`, `version_lines`, `help_text`, `program_name`, `help`) in `cli.rs`; `build_app`, `start_missing_required`, `open_main_window` in `app.rs`; `main.rs` tiene i `mod`, `main()`, `run()` e `fatal()` |

### 2.2 Funzioni nel posto sbagliato

- **`STR-V7-01`** — `responds`, `can_listen` e `is_port_free` (`projects/run.rs:225-242`) sono importati da `db/services/mod.rs:55`, `db/services/process.rs:19` e `llm/service.rs:27`: database e modelli dipendono dal runner dei progetti per due sonde TCP generiche. **Correzione**: spostarle in `crate::ports` (che già possiede le porte), con `pub use crate::ports::{is_port_free, responds};` in `run.rs` per i percorsi esistenti. `editor::free_ports` (`editor/mod.rs:340`, solo IPv4) resta com'è, perché cambiarla cambierebbe il comportamento; un commento ne spiega la differenza.

### 2.3 Riesportazioni e duplicati

- **`STR-V7-08`** — `setup/mod.rs` dichiara `pub mod catalog;` senza riesportare: `crate::setup::catalog::Component` compare 34 volte, `::Catalog` 26, `::kit_path` 23, più 60 righe `use crate::setup::catalog…` in 78 file. `router/mod.rs` non riesporta e `crate::router::fs::*` compare 31 volte. **Riesportazioni da aggiungere**:

  ```rust
  // tauri/src/setup/mod.rs
  pub use catalog::{kit_path, Catalog, Choice, Component, Group};

  // tauri/src/router/mod.rs
  pub use fs::{restrict_dir_to_owner, restrict_to_owner, set_executable, write_executable, write_private};
  ```

  Nessuna API cambia; i 114 percorsi lunghi si accorciano gradualmente (`crate::setup::Component`, `crate::router::write_private`).
- **`STR-V7-09`** — funzioni che inoltrano soltanto: `agents/sync/mod.rs:107` e `:119` (`read_json`/`write_json` → `fsx`), `agents/stats/mod.rs:280`, `editor/mod.rs:360` e `:367` (`restrict_*` → `router::fs`), `detect/overrides.rs:189` (`valid_domain` → `projects::alias`). **Correzione**: `pub(crate) use crate::router::fs::{restrict_to_owner, restrict_dir_to_owner};` in `editor/mod.rs`, `pub use crate::projects::alias::valid_domain;` in `detect/overrides.rs`; per i JSON, o annotare il tipo nei punti di chiamata o documentarli come specializzazioni su `Value`.
- **`STR-V7-10`** — `fn tr(lang, source, args) { crate::i18n::t(lang, source, args) }` identica in `detect/mod.rs:220`, `github/mod.rs:57`, `projects/mod.rs:149`, `projects/run.rs:1221`; sei moduli usano già `use crate::i18n::t;`. **Correzione**: `use crate::i18n::t as tr;`. `commands/mod.rs:293` (prende `&AppState`) e `tray.rs:78` sono diverse e restano.
- **`STR-V7-11`** — `now_ms` privata in `agents/stats/util.rs:20` (i64), `editor/window.rs:452` (u128), `projects/run.rs:184` (u64), `setup/self_update.rs:698` (String); `SystemTime::now().duration_since(UNIX_EPOCH)` 17 volte con larghezze e ripieghi diversi. **Correzione**: `pub fn now_ms() -> u64` e `pub fn now_secs() -> u64` in `fsx.rs` (o un modulo `crate::time`), con conversioni dove serve i64/u128.

### 2.4 Prove Rust

- **`STR-V7-21`** — `TempDir::new(..); Kit::at(..); kit.prepare().unwrap();` ripetuto 119 volte in 44 file, con copie private di `fn kit()` in 5 file; `fake_server` identica in `servers/mod.rs:595` e `db/services/data.rs:11`; `wait_for` identica in `editor/conpty.rs:67` e `editor/pty.rs:474`; `TempDir` raggiunto come `crate::kit::tests::TempDir` 124 volte. **Correzione**: `test/rust/support.rs`, montato una volta da `main.rs` con `#[cfg(test)] #[path = "../../test/rust/support.rs"] mod test_support;`, con `TempDir`, `prepared_kit(name)`, `kit_with_files(..)`, `fake_server(..)`, `wait_for(..)`; nelle prove del kit resta `pub use crate::test_support::TempDir;` per il percorso vecchio.
- **`STR-V7-22`** — `test/rust/servers/mod.rs` (2433 righe, 46 prove, il file più grande del repository) mescola profili, configurazioni dei pool, processi e cache, mentre `tauri/src/servers` ha cinque file. **Correzione**: `test/rust/servers/{mod.rs` (fixture comuni), `pools.rs`, `process.rs`, `config.rs}`, ognuno montato dal suo sorgente con `#[path]`; dividere `test/rust/env.rs` (1160 righe) lungo i sottomoduli di `STR-V7-03`.

---

## 3. Script e sito

- **`STR-V7-12`** — le regole di disponibilità per target stanno in `Component::is_available_on` (`tauri/src/setup/catalog.rs`), in `site/matrix.mjs:40-48` e in `scripts/matrix/gen_matrix.py:70-88`; le due copie dicono nel commento di «seguire» il Rust. Una regola nuova va portata a mano in due linguaggi di script e `TOOLS-MATRIX.html` può staccarsi dal sito. **Correzione**: portare `gen_matrix.py` in `scripts/matrix/gen_matrix.mjs` che importa `loadCatalog`, `TARGETS`, `status` da `site/matrix.mjs` (o da un `scripts/lib/catalog.mjs` comune).
- **`STR-V7-13`** — `site/shots/make.mjs:21-35`, `scripts/guide-shots/make.mjs:23-40` e `site/shots/social.mjs:47-48` ripetono la scelta del browser (`SHOTS_BROWSER`, `msedge`/`chrome`), `SHOTS_URL`, il caricamento di `ui/src/locales/<lang>.json` con `tr`, l'aiuto `nav(name)` e l'elenco delle lingue (anche in `site/build.mjs:26`). **Correzione**: `scripts/lib/shots.mjs` con `launch()`, `PREVIEW`, `LANGS`, `catalog(lang)`, `openPreview(..)`, `nav(page, tr)`. Va fatto prima di spostare `ui/src` (il percorso dei locali è scritto in tre posti) e prima di rigenerare le schermate dell'ondata 4.

---

## 4. Interfaccia

### 4.1 Duplicati

- **`STR-V7-14`** — `size(bytes)` è identica carattere per carattere in `databases/backups.tsx:11` e `projects/delete.tsx:16`; simili `settings/conda.tsx:12` (`megabytes`), `models/page.tsx:61` (`gb`), `components/model.ts:134` (`sizeText`, MB in base 1000). **Correzione**: `ui/src/shared/format.ts` con `formatBytes`, `gb`, `megabytes` e `sizeText`.
- **`STR-V7-15`** — `ENGINE_NAMES` in `projects/delete.tsx:14`, `projects/orphans.tsx:14` (e `projects/needs.tsx:45`); la validazione `Number.isInteger(v) && v >= 1024 && v <= 65535` con il suggerimento «Una porta fra 1024 e 65535.» due volte in `databases/page.tsx` (166-169, 268-271) e in `projects/run.tsx:317`. **Correzione**: esportare `ENGINE_NAMES` da `databases/index.ts`; `usePortField(initial)`, `PORT_MIN`, `PORT_MAX` e `validPort(n)` in `shared/common.tsx`.
- **`STR-V7-16`** — `"ok" | "warn" | "err" | "off"` scritto in `workbench/controls.tsx:867`, `activity.tsx:124`, `agents/page.tsx:60`, `databases/page.tsx:59`, `models/page.tsx:109`. **Correzione**: `export type DotTone` da `workbench`.

### 4.2 File grandi

- **`STR-V7-17`** — `workbench/controls.tsx` (878): pulsanti 108-307, campi 307-733, indicatori 733-878. **Correzione**: `workbench/{buttons,inputs,indicators}.tsx`; `workbench/index.ts` riesporta gli stessi nomi, gli 80+ importatori non cambiano.
- **`STR-V7-18`** — `guide/model.ts` ha tipi e aiuti (13-107) più ~755 righe di dati (`GUIDE_TOPICS`, 148-903); `topics/inside.tsx` definisce `diskTree` e il componente `SourcesTable` (266-404); i file dei capitoli vanno da 1000 a 2135 righe. **Si chiude con l'ondata 3**: il progetto della Guida neutra ([04](./04-GUIDA-NEUTRA-E-SITO.md)) separa `guide/core/model.ts` e `guide/content/`, e fa di `SourcesTable` un widget del contratto. La divisione dei capitoli più grandi per argomento (`content/projects/{index.ts,new.tsx,run.tsx,…}`) è facoltativa e va fatta dopo lo spostamento, mai prima: aggiornerebbe due volte le espressioni dei chunk in `ui/vite.config.ts:36-38`.

### 4.3 `STR-V7-19` — Struttura di destinazione di `ui/src`

Oggi la radice di `ui/src` ha 20 file sciolti: pagine (`servers.tsx`, `terminals.tsx`), pezzi della cornice (`titlebar.tsx`, `activity.tsx`, `eject.tsx`) e aiuti condivisi (`common.tsx` con 47 importatori, `hooks.ts` 40, `i18n.ts` 82) uno accanto all'altro. `terminals.tsx` (la pagina) accanto a `terminal/` (il pannello) e `tags.tsx` accanto a `frameworks/tags.tsx` si confondono facilmente. Le cartelle esistenti seguono già lo schema `<area>/{page.tsx,index.ts}`.

```text
ui/src/
  main.tsx, App.tsx              restano nella radice
  i18n.ts                        resta nella radice (vite.config.ts:31 si aspetta src/locales)
  locales/                       invariata
  shared/                        aiuti senza pagina
    common.tsx  hooks.ts  tags.tsx  logos.tsx  brand.tsx
    format.ts                    nuovo (STR-V7-14)
  shell/                         cornice dell'app
    titlebar.tsx  activity.tsx  eject.tsx  update.ts  freeze.ts  freezeproto.ts
  servers/                       nuova area
    page.tsx                     da servers.tsx
    index.ts
  terminal/
    page.tsx                     da terminals.tsx; TerminalsPage esportata da terminal/index.ts
    …                            pannello, vista, stato come oggi
  components/
    relations.tsx  byhand.tsx  setup.tsx   da radice: i loro consumatori sono qui (più App, activity, versions)
  projects/
    extensions.tsx               da radice (ProjectExtensions; components/explorer importa ExtensionsMatrixSection)
  agents/ api/ databases/ frameworks/ github/ guide/ models/ settings/ versions/ workbench/ theme/ assets/   invariate
```

Le prove in `test/ui` seguono gli spostamenti: `servers.test.tsx` → `servers/`, `terminals.test.tsx` → `terminal/`, le prove di `activity`, `freezeproto`, `hooks`, `i18n` e `logos` → `shell/` o `shared/`. Con gli alias del §5 lo spostamento è meccanico: si cambia il percorso nell'import `@/…` e nient'altro.

---

## 5. `STR-V7-20` — Alias

**Situazione.** `ui/tsconfig.json` non ha `paths` e `ui/vite.config.ts` non ha `resolve.alias`. In `ui/src` ci sono 344 import `../` e 17 `../../` (16 in `guide/topics`, più `api/app.ts:8` → `../../package.json`). In `test/ui` ci sono 98 riferimenti a `(../)+ui/src` (72 `../../../ui/src`, 15 `../../ui/src`, più import dinamici come `import("../../ui/src/App")`), 3 a `../../../tauri`, 2 a `../../../docs`, 1 a `../../../site/matrix.mjs`. Il blocco `vi.mock("@tauri-apps/api/core", …)` con `const invoke = vi.fn()` è ripetuto in 21 file di prova.

**Alias proposti.**

| Alias | Punta a | Uso |
|---|---|---|
| `@/` | `ui/src/` | ogni import dell'app che sale di più di un livello; tutti gli import delle prove verso l'app |
| `@guide/` | `ui/src/guide/` nell'ondata 2; **`guide/`** (radice) dall'ondata 3 | la Guida dall'app e dalle prove |
| `@test/` | `test/ui/` | aiuti condivisi delle prove (`@test/support/tauri`) |

```jsonc
// ui/tsconfig.json
"compilerOptions": {
  "baseUrl": ".",
  "paths": {
    "@/*": ["src/*"],
    "@guide/*": ["src/guide/*"],      // ondata 3: ["../guide/*"]
    "@test/*": ["../test/ui/*"]
  }
}
```

```ts
// ui/vite.config.ts, unito alle resolve.conditions della modalità di prova (riga 15)
resolve: {
  alias: {
    "@": fileURLToPath(new URL("./src", import.meta.url)),
    "@guide": fileURLToPath(new URL("./src/guide", import.meta.url)), // ondata 3: "../guide"
    "@test": fileURLToPath(new URL("../test/ui", import.meta.url)),
  },
},
```

- `test/ui/tsconfig.json` eredita i `paths` (si risolvono rispetto al file che li definisce); `vitest.config.ts` estende la configurazione di Vite e non serve altro; ESLint usa `projectService`, quindi bastano i `paths` (non c'è un plugin d'import da configurare).
- **Prove**: `test/ui/support/tauri.ts` esporta il finto `invoke`; i 21 file usano `vi.mock("@tauri-apps/api/core", () => import("@test/support/tauri"))`.
- **Vincolo con lo script della Guida**: finché `scripts/guide-docs.cjs` esiste, il suo `req()` risolve gli specificatori con `path.resolve` e un `@/i18n` finirebbe in silenzio nel proxy dei marcatori, rompendo GUIDE*.md. Nell'ondata 2 quindi **non si convertono gli import dentro `ui/src/guide`**; lo script sparisce nell'ondata 3 ([04](./04-GUIDA-NEUTRA-E-SITO.md)), e il contenuto spostato in `guide/` importa solo dentro `guide/` (regola imposta dal caricatore del sito).
- **Effetto stimato**: tutti i 17 `../../` di `ui/src` e circa 98 riferimenti profondi delle prove diventano `@/…` (circa 115 relativi profondi in meno). I 344 `../` singoli possono restare o passare a `@/` poco alla volta, quando si tocca il file.

### 5.1 Rust: riesportazioni al posto degli alias

Rust non ha alias di percorso; l'equivalente sono i `pub use` nei `mod.rs`, già elencati: `setup/mod.rs` e `router/mod.rs` (`STR-V7-08`), le riesportazioni dei sottomoduli nuovi (`STR-V7-02..07`, che tengono stabili i percorsi `crate::…`), `projects/run.rs` verso `crate::ports` (`STR-V7-01`) e i wrapper sostituiti (`STR-V7-09`, `STR-V7-10`).

---

## 6. Controllati senza rilievo

- `setup/runner.rs` (872), `editor/pty.rs` (974), `setup/catalog.rs` (807), `agents/relocate.rs` (787) e `servers/process.rs` (772) hanno confini chiari (modulo Unix, varianti di avvio, `RunLock`, piano) ma non sono stati confrontati riga per riga: nessun rilievo a parte.
- Coppie con lo stesso nome ma comportamento diverso, quindi non duplicati: `alive` (`agents/sync/mod.rs:347` e `os/sessions.rs:268`, che gestisce gli zombie) e `valid_id` (quattro insiemi di regole diversi).
- I file dei capitoli della Guida sono grandi per il testo in quattro lingue: la loro divisione è nel documento [04](./04-GUIDA-NEUTRA-E-SITO.md).
- Nessun problema di marchio: l'unico riscontro dei nomi di prima fuori dai report è una riga di storia del nome in `docs/app/DECISIONS.md`.
