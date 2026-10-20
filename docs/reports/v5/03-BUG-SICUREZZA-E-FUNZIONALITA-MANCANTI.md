# 03 — Bug, Sicurezza e Funzionalità Mancanti (`v5`)

> **Riferimento Codice**: `HEAD a8aad6c` (20/10/2026) · **Baseline**: `v4` su `18698f9` (17/10/2026) · **Rilievi `v4` verificati**: **31** (`SEC-V4-01..04`, `BUG-V4-01..13`, `PORT-V4-01..08`, `UX-V4-01..06`) · **Nuovi rilievi `v5`**: **9** (`1` Sicurezza `SEC-V5-01`, `5` Bug `BUG-V5-01..05`, `1` Portabilità `PORT-V5-01`, `2` UX `UX-V5-01..02`).
>
> Metodo: ogni rilievo `v5` è stato riletto nel codice di `a8aad6c` prima di essere scritto; sono comportamenti letti nel codice, non riprodotti a runtime. Il confronto con il rapporto precedente è in [`../v4/03-BUG-SICUREZZA-E-FUNZIONALITA-MANCANTI.md`](../v4/03-BUG-SICUREZZA-E-FUNZIONALITA-MANCANTI.md).

---

## 1. Riepilogo

### 1.1 Stato dei rilievi `v4`

| Categoria | Risolti | Parziali | Aperti | Totale |
|---|:---:|:---:|:---:|:---:|
| **Sicurezza (`SEC-V4-*`)** | 2 | 1 | 1 | **4** |
| **Bug (`BUG-V4-*`)** | 6 | 3 | 4 | **13** |
| **Portabilità (`PORT-V4-*`)** | 8 | 0 | 0 | **8** |
| **UX (`UX-V4-*`)** | 4 | 1 | 1 | **6** |
| **TOTALE** | **20** | **5** | **6** | **31** |

Tutti i quattro rilievi critici del `v4` (`PORT-V4-01..04`) sono chiusi. `BUG-V4-10` è contato fra i risolti con un residuo teorico (sez. 2.2). I 6 aperti e i 5 parziali confluiscono nei rilievi `v5` della sezione 3 oppure, dove non c'è un nuovo ID, nel piano esecutivo ([`07-PIANO-ESECUTIVO-V5.md`](07-PIANO-ESECUTIVO-V5.md)).

### 1.2 Nuovi rilievi `v5` per severità

| Categoria | Critica | Alta | Media | Bassa | Totale |
|---|:---:|:---:|:---:|:---:|:---:|
| **Sicurezza (`SEC-V5-*`)** | 0 | 0 | 0 | 1 | **1** |
| **Bug (`BUG-V5-*`)** | 0 | 1 | 2 | 2 | **5** |
| **Portabilità (`PORT-V5-*`)** | 0 | 0 | 0 | 1 | **1** |
| **UX (`UX-V5-*`)** | 0 | 0 | 1 | 1 | **2** |
| **TOTALE `v5`** | **0** | **1** | **3** | **5** | **9** |

Nessun rilievo critico. Sette dei nove rilievi `v5` sono residui di rilievi `v4` non chiusi del tutto; due sono nuovi (`BUG-V5-03`, nato con l'identità Git di D-137, e `PORT-V5-01`, nato con il server di sviluppo di D-138).

---

## 2. Verifica dei rilievi `v4`

Legenda: **Risolto** / **Parziale** / **Aperto**. La colonna «Seguito» indica dove il residuo è trattato.

### 2.1 Sicurezza (`SEC-V4-01..04`)

| ID | Stato | Evidenza in `a8aad6c` | Seguito |
|---|:---:|---|---|
| **`SEC-V4-01`** PHP eseguito negli alias | **Risolto** | `ALIAS_NO_RUN = r"\.(?:php\|phtml\|phar\|phps\|inc\|php[3-8])$"` ([`tauri/src/projects/apache.rs:273`](../../../tauri/src/projects/apache.rs#L273)); `<FilesMatch> Require all denied` in ogni `<Directory>` degli alias e `AllowOverride` senza `FileInfo` ([`apache.rs:285-301`](../../../tauri/src/projects/apache.rs#L285-L301)), usato anche nei pool ([`servers/config.rs:65`](../../../tauri/src/servers/config.rs#L65)). Caddy: alias solo statici, `@script path_regexp` → 404 ([`projects/caddy.rs:509-510`](../../../tauri/src/projects/caddy.rs#L509-L510)). Nginx rimosso (D-120). Test in `test/rust/projects/{apache,caddy}.rs`, `test/rust/servers/mod.rs`. | — |
| **`SEC-V4-02`** router `php -S`: LAN, `Host` vuoto, router di progetto | **Parziale** | (3) risolto: il `router.php` del progetto gira dopo i controlli del kit via `-d atollo.router=` ([`detect/mod.rs:300-318`](../../../tauri/src/detect/mod.rs#L300-L318), commit `6dd5f00`). (2) `Host` vuoto accettato di proposito (`if ($host !== '')`, [`runtime/php-router.php:24`](../../../runtime/php-router.php#L24)), scelta documentata in [`DECISIONS.md:853`](../../app/DECISIONS.md#L853). (1) **ancora aperto**: il router non legge mai `ATOLLO_ALLOW_LAN` e risponde 421 a ogni `Host` non locale ([`php-router.php:23-32`](../../../runtime/php-router.php#L23-L32)). | `BUG-V5-01` |
| **`SEC-V4-03`** `serve.rs` rifiuta `*.localhost` | **Aperto** | `matches!(name.as_str(), "localhost" \| "127.0.0.1" \| "[::1]")` senza `.localhost` ([`tauri/src/projects/serve.rs:56-67`](../../../tauri/src/projects/serve.rs#L56-L67)). | `SEC-V5-01` |
| **`SEC-V4-04`** `FCGI_WEB_SERVER_ADDRS`, vHost di default dei pool | **Risolto** | `.env("FCGI_WEB_SERVER_ADDRS", "127.0.0.1")` più chiave per avvio `ATOLLO_FCGI_KEY` ([`projects/fastcgi.rs:167-182`](../../../tauri/src/projects/fastcgi.rs#L167-L182), commit `33a502a`); primo vHost `refused.invalid` con `Require all denied` nei pool Apache fuori LAN ([`servers/config.rs:114-128`](../../../tauri/src/servers/config.rs#L114-L128), commit `109a19b`); Caddy `respond 421` ([`projects/caddy.rs:547-549`](../../../tauri/src/projects/caddy.rs#L547-L549)). | — |

### 2.2 Bug (`BUG-V4-01..13`)

| ID | Stato | Evidenza in `a8aad6c` | Seguito |
|---|:---:|---|---|
| **`BUG-V4-01`** `site_mode` vs `scan` | **Risolto** | `SiteMode::Auto` per `None`/`""`/`"auto"`, `Dedicated` solo per `DEDICATED_SERVER` ([`projects/run.rs:124-138`](../../../tauri/src/projects/run.rs#L124-L138)); `pool_of` da una scansione nuova, ripiego sul server del sito con una nota ([`commands/runs.rs:37-85`](../../../tauri/src/commands/runs.rs#L37-L85); commit `1b5b173`, `4412524`). | — |
| **`BUG-V4-02`** pool ignorano `conf.d` del progetto | **Aperto** | Il vHost di pool Apache ([`servers/config.rs:49-72`](../../../tauri/src/servers/config.rs#L49-L72)) e il blocco Caddy di pool ([`config.rs:166-189`](../../../tauri/src/servers/config.rs#L166-L189)) non includono `data/projects/<p>/{apache,caddy}/conf.d`; il server dedicato sì ([`projects/apache.rs:389`](../../../tauri/src/projects/apache.rs#L389)). | `BUG-V5-05` |
| **`BUG-V4-03`** porte incrociate, `php.ini` condiviso | **Parziale** | `php.ini` per linea: `data/php/<line>` su Windows, `data/php/<target>/<line>` su Unix, con migrazione ([`install/web.rs:855-878, 919-937`](../../../tauri/src/install/web.rs#L855-L878); commit `624918b`). Porte: `save_profile` controlla solo gli altri pool di `servers.json` e `p < 1024` ([`servers/pools.rs:533-566`](../../../tauri/src/servers/pools.rs#L533-L566)). | `BUG-V5-05` |
| **`BUG-V4-04`** race `Services` / `services.json` | **Aperto** | `Services` ha come Mutex solo `running` ed `errors` ([`db/services/mod.rs:162-163`](../../../tauri/src/db/services/mod.rs#L162-L163)); `start_engine` controlla `running`, rilascia il lock e `launch` inserisce il processo più tardi ([`mod.rs:567-576, 663-675`](../../../tauri/src/db/services/mod.rs#L567-L576)); `keep_port` legge-modifica-scrive senza lock ([`mod.rs:602-610`](../../../tauri/src/db/services/mod.rs#L602-L610)). | `BUG-V5-02` |
| **`BUG-V4-05`** scambio porte MySQL al cambio di linea predefinita | **Aperto** | `line_ports` dà sempre la porta del server alla linea principale e registra solo le altre ([`mod.rs:195-230`](../../../tauri/src/db/services/mod.rs#L195-L230)); `keep_port` solo se `!main` ([`mod.rs:578-583`](../../../tauri/src/db/services/mod.rs#L578-L583)). | `BUG-V5-02` |
| **`BUG-V4-06`** `drop_db` solo sulla linea corrente | **Risolto** | `drop_db` scorre `project_drop_lines` e cancella su ogni linea installata, avvisa per quelle disinstallate ([`mod.rs:951-971`](../../../tauri/src/db/services/mod.rs#L951-L971); commit `3379fca`). | — |
| **`BUG-V4-07`** browser DB e admin solo linea primaria | **Risolto** | `running_at(engine, line)` / `sql_tool_at(..., line, port)` ([`db/browse.rs:155-181`](../../../tauri/src/db/browse.rs#L155-L181)); phpMyAdmin e login per versione (commit `fa148c3`; D-107). | — |
| **`BUG-V4-08`** `<exe>-<pid>.cnf` condiviso fra thread | **Aperto** (ambito ridotto) | Nome con il solo PID: `format!("{exe}-{}.cnf", std::process::id())` ([`db/services/tools.rs:175`](../../../tauri/src/db/services/tools.rs#L175)) e `{exe}-{pid}.yaml` per MongoDB ([`tools.rs:197`](../../../tauri/src/db/services/tools.rs#L197)). Riguarda ora solo dump/ripristini concorrenti. | `BUG-V5-04` |
| **`BUG-V4-09`** `requires` forza Apache su Laravel | **Risolto** | `apache` solo se `php::apache_offered(...)` sulle regole proprie del `.htaccess`, altrimenti Caddy ([`detect/mod.rs:571-578`](../../../tauri/src/detect/mod.rs#L571-L578)); D-111, D-116. | — |
| **`BUG-V4-10`** `env_apply` tronca `.env`; ordine nel wizard | **Risolto** (residuo teorico) | `env_source` legge `.env`, altrimenti `.env.example` ([`db/auth.rs:879-883`](../../../tauri/src/db/auth.rs#L879-L883)); `env_apply` riscrive quel testo con `env_set_text` ([`auth.rs:843-901`](../../../tauri/src/db/auth.rs#L843-L901)). L'ordine resta `env_apply` ([`commands/frameworks.rs:332`](../../../tauri/src/commands/frameworks.rs#L332)) prima di `env_set` ([`frameworks.rs:355`](../../../tauri/src/commands/frameworks.rs#L355)), ma nessuna base di [`tauri/catalog/frameworks.json`](../../../tauri/catalog/frameworks.json) definisce chiavi `DB_*`/`DATABASE*`: il rischio resta solo per una base utente. | Piano, WP-R |
| **`BUG-V4-11`** cache `WebServers` non invalidata | **Parziale** | `invalidate()` solo in `project_action_save`, `project_action_remove`, `project_site` ([`commands/projects.rs:122, 161, 227`](../../../tauri/src/commands/projects.rs#L122)); non in `project_delete` ([`projects.rs:59`](../../../tauri/src/commands/projects.rs#L59)), `project_clone` ([`projects.rs:83`](../../../tauri/src/commands/projects.rs#L83)), `github_clone`, `frameworks_create`. Ritardo limitato da `LIST_TTL = 15 s` ([`servers/mod.rs:30`](../../../tauri/src/servers/mod.rs#L30)); `pool_of` usa sempre una scansione nuova. | Piano, WP-R |
| **`BUG-V4-12`** tag immagine con registry e porta | **Risolto** | `image.rsplit('/').next()` poi `split_once(':')` sull'ultimo segmento ([`detect/container.rs:57-58`](../../../tauri/src/detect/container.rs#L57-L58)). | — |
| **`BUG-V4-13`** terminale con linea disinstallata | **Parziale** | Modulo spostato in [`tauri/src/editor/terminals.rs`](../../../tauri/src/editor/terminals.rs): «a line falls back to the newest» ([`terminals.rs:14-15`](../../../tauri/src/editor/terminals.rs#L14-L15)), `choice_env` in `env` ([`terminals.rs:430-449`](../../../tauri/src/editor/terminals.rs#L430-L449)). La segnalazione all'utente copre solo gli strumenti spariti (`missing`, [`terminals.rs:73, 200`](../../../tauri/src/editor/terminals.rs#L73)), non la linea sostituita. | Piano, WP-R |

### 2.3 Portabilità (`PORT-V4-01..08`)

| ID | Stato | Evidenza in `a8aad6c` |
|---|:---:|---|
| **`PORT-V4-01`** `copy_tree` perde i symlink | **Risolto** | `copy_entries` raccoglie i link (`read_link`) e `finish_links_here` li ricrea su Unix ([`net/archive.rs:852-887`](../../../tauri/src/net/archive.rs#L852-L887)). |
| **`PORT-V4-02`** rilocazione Unix | **Risolto** | `Kit::old_root_file` con `old-root-darwin.txt` / `old-root-linux.txt` ([`kit.rs:569-571`](../../../tauri/src/kit.rs#L569-L571)), letto da [`agents/relocate.rs:524-526`](../../../tauri/src/agents/relocate.rs#L524-L526); `substitute` cambia la maiuscola solo con una lettera di unità (`has_drive_letter`, [`relocate.rs:138-160`](../../../tauri/src/agents/relocate.rs#L138-L160)). |
| **`PORT-V4-03`** `bin\agy`, `${var,,}` | **Risolto** | `("bin/agy", ...)` con `router::fs::write_executable` / `set_executable` ([`setup/kit_files.rs:28-29, 130-137`](../../../tauri/src/setup/kit_files.rs#L28-L29)); [`runtime/bin/agy:6-10`](../../../runtime/bin/agy#L6-L10) sceglie `agy.exe` o `agy`; [`runtime/bashrc:5`](../../../runtime/bashrc#L5) vieta `${var,,}`, `declare -A`, `mapfile` (bash 3.2); nessuna occorrenza di `,,}` in `runtime/`. |
| **`PORT-V4-04`** VSCodium server solo Windows | **Risolto** | Profili terminale per sistema `terminal.integrated.<name>.<os_key>` ([`editor/cfg.rs:274`](../../../tauri/src/editor/cfg.rs#L274)); `node.exe` / `out\server-main.js` restano solo nei commenti ([`editor/mod.rs:15, 87`](../../../tauri/src/editor/mod.rs#L15)). |
| **`PORT-V4-05`** `.exe` e `php/php.exe` in `db/*` | **Risolto** | `translate_rel` mappa `php/php.exe` → `php/bin/php` e toglie `.exe/.cmd/.bat` fuori da Windows ([`setup/catalog.rs:262-292`](../../../tauri/src/setup/catalog.rs#L262-L292)), usata da `kit_path` ([`catalog.rs:303-309`](../../../tauri/src/setup/catalog.rs#L303-L309)) per [`db/phpmyadmin.rs:195`](../../../tauri/src/db/phpmyadmin.rs#L195); nomi dei programmi DB da `DbEngine` (D-110). |
| **`PORT-V4-06`** backslash nei join | **Risolto** | Test su tutto il crate «No separator written inside a joined path» ([`tauri/src/fsx.rs:142-145`](../../../tauri/src/fsx.rs#L142-L145) → [`test/rust/paths.rs`](../../../test/rust/paths.rs)); job `test-unix` su 4 target ([`.github/workflows/app.yml:211-300`](../../../.github/workflows/app.yml#L211-L300)). |
| **`PORT-V4-07`** Julia/Bun ARM64, `RunLock` Unix, pin tauri | **Risolto** | Julia `x64Only: true` ([`components.json:23`](../../../tauri/catalog/components.json#L23)); Bun con asset nativo `aarch64` ([`install/lang/js.rs:18-29`](../../../tauri/src/install/lang/js.rs#L18-L29)); `RunLock` Unix con `flock` su `data/app/prepare.lock` ([`setup/runner.rs:141-200`](../../../tauri/src/setup/runner.rs#L141-L200)); `tauri = "=2.12.1"` ([`tauri/Cargo.toml:24`](../../../tauri/Cargo.toml#L24)). |
| **`PORT-V4-08`** offline Git/R Unix, stack web Unix | **Risolto** | Git dugite-native (D-112), conda offline (D-118, [`install/conda.rs`](../../../tauri/src/install/conda.rs)), Apache del kit (D-116), PHP del kit (D-133), Caddy (D-111), phpMyAdmin sui 6 target (D-117). Catalogo 35/35 su cinque target, 34/35 su `linux-arm64` (manca solo `android`, [`components.json:19`](../../../tauri/catalog/components.json#L19)). |

### 2.4 UX (`UX-V4-01..06`)

| ID | Stato | Evidenza in `a8aad6c` | Seguito |
|---|:---:|---|---|
| **`UX-V4-01`** barra di stato e Attività ignorano pool e linee MySQL secondarie | **Aperto** | `live` conta solo le esecuzioni ([`ui/src/App.tsx:104, 505, 532`](../../../ui/src/App.tsx#L104)); `liveServices = services.filter(s => s.state !== "stopped")` guarda solo la linea principale ([`ui/src/activity.tsx:49`](../../../ui/src/activity.tsx#L49)) anche se `ServiceInfo` ha `others` ([`db/services/mod.rs:445-455`](../../../tauri/src/db/services/mod.rs#L445-L455), [`ui/src/api/services.ts:26`](../../../ui/src/api/services.ts#L26)); nessun pool web nel pannello; nessuna azione «Ferma tutto» in `ui/src`. | `UX-V5-01` |
| **`UX-V4-02`** siti sganciati, nome azione | **Risolto** | `AttachedSite.action`, `server_name`, `detached` ([`servers/mod.rs:78-95`](../../../tauri/src/servers/mod.rs#L78-L95)); [`ui/src/servers.tsx:160-193`](../../../ui/src/servers.tsx#L160-L193) mostra `detached` e `site.serverName` (commit `b58fdd7`). | — |
| **`UX-V4-03`** etichetta «Apache dedicato», dominio statico | **Risolto** | Nessuna «Apache dedicato» in `ui/src/projects`; nome dal pool (`serverName`) con ripiego `<label>.localhost` ([`ui/src/projects/site.tsx:16, 61`](../../../ui/src/projects/site.tsx#L16)). | — |
| **`UX-V4-04`** `DbVersionSelect` nascosto | **Risolto** | `if (!versions \|\| (versions.lines.length < 2 && missing === null)) return null;` ([`ui/src/projects/databases.tsx:133`](../../../ui/src/projects/databases.tsx#L133); commit `04dfb88`). | — |
| **`UX-V4-05`** `Project.lines` non esposto | **Risolto** | `lines?: Record<string, string>` ([`ui/src/api/projects.ts:50`](../../../ui/src/api/projects.ts#L50)); la UI propone la linea dei container (commit `09e489a`). | — |
| **`UX-V4-06`** conferme e `onCancel` | **Parziale** | Il pulsante separato «elimina database» non c'è più: i DB si cancellano dal piano di eliminazione del progetto ([`api/projects.ts:217-253`](../../../ui/src/api/projects.ts#L217-L253)). Restano: `<InlineConfirm>` senza `onCancel` ([`ui/src/terminals.tsx:258`](../../../ui/src/terminals.tsx#L258)); «Elimina profilo» del pool esegue `serverRemove` al primo clic ([`ui/src/servers.tsx:137, 524-528`](../../../ui/src/servers.tsx#L137)). | `UX-V5-02` |

---

## 3. Catalogo dei nuovi rilievi `v5`

### 3.1 Sicurezza

#### `SEC-V5-01` [BASSA] — Il server statico rifiuta `<progetto>.localhost` con 403 (residuo di `SEC-V4-03`)
- **File e Righe**: [`tauri/src/projects/serve.rs:55-67`](../../../tauri/src/projects/serve.rs#L55-L67) (`is_local_host_header`), usato in [`serve.rs:227`](../../../tauri/src/projects/serve.rs#L227).
- **Analisi Tecnica**: la funzione accetta solo `matches!(name.as_str(), "localhost" | "127.0.0.1" | "[::1]")`. Il router `php -S` accetta i nomi che finiscono in `.localhost` ([`runtime/php-router.php:28-29`](../../../runtime/php-router.php#L28-L29)), e così Caddy e Apache. La severità è bassa perché l'effetto è un rifiuto in più, non un accesso in più.
- **Scenario di guasto**: l'utente apre `http://blog.localhost:<porta>` su un'azione «sito statico»: il server risponde 403, mentre la stessa URL funziona con `php -S`, Caddy e Apache.
- **Correzione Proposta**: `matches!(...) || name.ends_with(".localhost")`, con un test in [`test/rust/projects/serve.rs`](../../../test/rust/projects/serve.rs) per `a.localhost`, `a.localhost:8000`, `A.LOCALHOST.` e per un nome che **non** deve passare (`localhost.evil.com`).

### 3.2 Bug

#### `BUG-V5-01` [MEDIA] — In modalità LAN le azioni `php -S` rispondono 421 ai dispositivi della rete (residuo di `SEC-V4-02.1`)
- **File e Righe**:
  - [`runtime/php-router.php:23-32`](../../../runtime/php-router.php#L23-L32) (controllo dell'`Host`)
  - [`tauri/src/projects/run.rs:485-487`](../../../tauri/src/projects/run.rs#L485-L487) (`lan`, `listen`) e `on_network` ([`run.rs:1173-1180`](../../../tauri/src/projects/run.rs#L1173-L1180))
  - [`tauri/src/env.rs:875-878`](../../../tauri/src/env.rs#L875-L878) (`ATOLLO_ALLOW_LAN=1` nell'ambiente del kit)
- **Analisi Tecnica**: con «Permetti ai server dei progetti in rete» attivo, `env.rs` mette `ATOLLO_ALLOW_LAN=1` nell'ambiente e `run.rs` sostituisce `127.0.0.1` con `0.0.0.0` nel comando, quindi `php -S` ascolta su ogni interfaccia. Il router però non legge mai `ATOLLO_ALLOW_LAN` (nessuna occorrenza nel file) e risponde 421 a ogni `Host` diverso da `localhost`, `127.0.0.1`, `[::1]` e `*.localhost`. Caddy e Apache invece saltano il 421 in LAN ([`servers/config.rs:114-133`](../../../tauri/src/servers/config.rs#L114-L133); test in [`test/rust/projects/caddy.rs`](../../../test/rust/projects/caddy.rs)), e anche il server statico ammette un IP come `Host` in LAN ([`serve.rs:227`](../../../tauri/src/projects/serve.rs#L227)).
- **Scenario di guasto**: l'utente attiva la rete locale, avvia un progetto PHP con l'azione `php -S` e apre `http://192.168.1.20:8000` dal telefono: riceve `421 Misdirected Request`. La funzione «condividi in LAN» non funziona proprio per il server più semplice.
- **Correzione Proposta**: nel router, saltare il controllo dell'`Host` quando `getenv('ATOLLO_ALLOW_LAN') === '1'` (tenere invariati i controlli su dotfile e percorso); test in [`test/rust/detect/php.rs`](../../../test/rust/detect/php.rs) che avvia il router con e senza la variabile e un `Host: 192.168.1.20:8000`.

#### `BUG-V5-02` [ALTA] — Nessun lock sulle operazioni dei database: doppio `mysqld` sulla stessa cartella, `services.json` perso, porte scambiate (residuo di `BUG-V4-04` e `BUG-V4-05`)
- **File e Righe**:
  - [`tauri/src/db/services/mod.rs:160-165`](../../../tauri/src/db/services/mod.rs#L160-L165) (`struct Services`: solo `running` ed `errors` come Mutex)
  - [`mod.rs:550-597`](../../../tauri/src/db/services/mod.rs#L550-L597) (`start_engine`), [`mod.rs:602-610`](../../../tauri/src/db/services/mod.rs#L602-L610) (`keep_port`), [`mod.rs:612-675`](../../../tauri/src/db/services/mod.rs#L612-L675) (`launch`)
  - [`mod.rs:191-230`](../../../tauri/src/db/services/mod.rs#L191-L230) (`line_ports`)
- **Analisi Tecnica**:
  1. **Avvio doppio**: `start_engine` prende il lock di `running`, vede che la chiave non c'è, **rilascia** il lock; poi `is_port_free`, `keep_port`, `launch` (che prepara la cartella dati, avvia il processo e solo allora inserisce in `running`). Due chiamate concorrenti per la stessa linea possono passare entrambe il controllo prima che una delle due inserisca il processo. `is_port_free` riduce la finestra solo dopo che il primo `mysqld` ha aperto la porta.
  2. **Scritture perse**: `keep_port` fa `load_settings` → modifica → `write_json` senza lock; `write_atomic` serializza solo il rename ([`fsx.rs:79-91`](../../../tauri/src/fsx.rs#L79-L91)), non il ciclo lettura-modifica-scrittura.
  3. **Scambio di porte**: `line_ports` dà sempre la porta del server alla linea principale (`out.insert(m, port)`) e `keep_port` si chiama solo se `!main`: la linea principale non ha mai una porta salvata.
- **Scenario di guasto**:
  - Due progetti che chiedono MySQL 8.4 si avviano insieme (Pannello rapido, avvio automatico): due `mysqld` sulla stessa `data/db/mysql/<line>`; il secondo fallisce sul lock di InnoDB o, peggio, i due si contendono i file.
  - Il progetto A usa MySQL 9.x (principale, 3306) e il B usa 8.4 (3307). L'utente sceglie 8.4 come linea del server: 8.4 passa a 3306, 9.x non ha porta salvata e prende la prima libera (3307): le stringhe di connessione nei `.env` di entrambi i progetti puntano al server sbagliato.
- **Correzione Proposta**: un `op_lock: Mutex<()>` per motore tenuto per tutto `start_engine` / `stop_engine`; un lock per il ciclo su `services.json` (come `Settings::update`, [`tauri/src/settings.rs:247-264`](../../../tauri/src/settings.rs#L247-L264)); salvare in `ports` anche la porta della linea principale e in `line_ports` dare precedenza alla porta salvata di ogni linea, usando la porta del server solo per una linea che non ne ha. Test in [`test/rust/db/services/mod.rs`](../../../test/rust/db/services/mod.rs): `line_ports` prima e dopo il cambio di principale dà le stesse porte; due thread su `keep_port` non perdono voci.

#### `BUG-V5-03` [BASSA] — `kit_safe_value` cancella le voci `safe.directory` dell'utente per cartelle non presenti
- **File e Righe**: [`tauri/src/github/mod.rs:214-222`](../../../tauri/src/github/mod.rs#L214-L222) (`kit_safe_value`), [`github/mod.rs:225-241`](../../../tauri/src/github/mod.rs#L225-L241) (`sync_safe_directory`); il comportamento è fissato dal test [`test/rust/github/mod.rs:48`](../../../test/rust/github/mod.rs#L48) (`kit_safe_value("Z:/gone/kit/*")` vero).
- **Analisi Tecnica**: D-137 limita `safe.directory` alla cartella del kit (`<kit>/*`). Per togliere le voci scritte prima, `kit_safe_value` considera «del kit» `*` e ogni valore `X/*` dove `X` **non esiste** oppure contiene `data/app`. `sync_safe_directory` fa `--unset-all` e riscrive solo le voci che non sono «del kit». Una voce dell'utente della forma `X/*` per una cartella che in quel momento non c'è viene quindi tolta.
- **Scenario di guasto**: l'utente ha aggiunto nel `.gitconfig` del kit `safe.directory = E:/repos/*` per un disco esterno. Avvia Atollo con il disco staccato: la voce sparisce; ricollegato il disco, Git rifiuta i repository con «dubious ownership».
- **Correzione Proposta**: togliere solo `*`, la radice attuale e le radici precedenti note al kit (`Kit::old_root_file`, [`tauri/src/kit.rs:569-571`](../../../tauri/src/kit.rs#L569-L571)), oppure marcare le voci scritte dal kit; aggiornare il test della riga 48. Programmato subito dopo questo rapporto, insieme al rilascio 0.4.0 alpha.

#### `BUG-V5-04` [BASSA] — File di login temporanei dei dump con il solo PID nel nome (residuo di `BUG-V4-08`)
- **File e Righe**: [`tauri/src/db/services/tools.rs:175`](../../../tauri/src/db/services/tools.rs#L175) (`<exe>-<pid>.cnf`), [`tools.rs:197`](../../../tauri/src/db/services/tools.rs#L197) (`<exe>-<pid>.yaml` di MongoDB).
- **Analisi Tecnica**: tutti i thread di Atollo hanno lo stesso PID. Due strumenti dello stesso tipo (per esempio due `mysqldump` o due `mongorestore`) avviati insieme scrivono lo stesso file di login, e il primo che finisce lo cancella mentre l'altro può ancora leggerlo. Il contatore che risolve lo stesso problema esiste già per `write_atomic` (`TMP_SEQ`, [`tauri/src/fsx.rs:80-81`](../../../tauri/src/fsx.rs#L80-L81)).
- **Scenario di guasto**: backup di due progetti avviati uno subito dopo l'altro: il secondo client trova il file già cancellato (o con utente e password dell'altro progetto) e fallisce l'accesso.
- **Correzione Proposta**: `static LOGIN_SEQ: AtomicU64` e nome `<exe>-<pid>-<seq>.cnf` / `.yaml`; test che due chiamate consecutive danno percorsi diversi.

#### `BUG-V5-05` [MEDIA] — I siti nei pool ignorano il `conf.d` del progetto; porte dei pool non controllate contro DB, progetti ed editor (residuo di `BUG-V4-02` e `BUG-V4-03`)
- **File e Righe**:
  - [`tauri/src/servers/config.rs:49-72`](../../../tauri/src/servers/config.rs#L49-L72) (vHost Apache del sito nel pool), [`config.rs:166-189`](../../../tauri/src/servers/config.rs#L166-L189) (`caddy_site_conf`); il server dedicato include il `conf.d` del progetto ([`projects/apache.rs:389`](../../../tauri/src/projects/apache.rs#L389)), il pool solo il proprio ([`config.rs:139`](../../../tauri/src/servers/config.rs#L139))
  - [`tauri/src/servers/pools.rs:533-566`](../../../tauri/src/servers/pools.rs#L533-L566) (`save_profile`)
- **Analisi Tecnica**:
  1. Un sito spostato da server dedicato a pool perde in silenzio le direttive che l'utente ha scritto in `data/projects/<p>/apache/conf.d/*.conf` (o il suo equivalente Caddy).
  2. `save_profile` rifiuta solo `p < 1024` e una porta già data a un altro pool in `servers.json`; non controlla MySQL/MongoDB (`services.json`), le porte assegnate ai progetti (`Runner::set_port`, [`projects/run.rs:416`](../../../tauri/src/projects/run.rs#L416)) né l'intervallo dell'editor `7000..=7099` ([`editor/mod.rs:47`](../../../tauri/src/editor/mod.rs#L47)). Allo stesso modo `Services::set` / `set_line_port` ([`db/services/mod.rs:818, 834`](../../../tauri/src/db/services/mod.rs#L818)) non conoscono i pool.
- **Scenario di guasto**: (1) un `conf.d` con un `RewriteRule` o un header CORS funziona con il server dedicato e smette di funzionare dopo «Sposta nel pool». (2) Un pool salvato sulla porta 3307 non parte quando la linea MySQL secondaria è avviata, oppure impedisce a quella linea di partire, con un errore generico di porta occupata.
- **Correzione Proposta**: `IncludeOptional "<project>/apache/conf.d/*.conf"` nel vHost del pool e `import` del `conf.d` del progetto nel blocco Caddy (solo se la cartella esiste, come per i pool, [`projects/caddy.rs:641`](../../../tauri/src/projects/caddy.rs#L641)); una funzione comune `ports_taken(kit)` (pool, DB, progetti, editor) usata da `save_profile`, `Runner::set_port`, `Services::set` e `set_line_port`, con messaggio che nomina il proprietario della porta.

### 3.3 Portabilità

#### `PORT-V5-01` [BASSA, solo sviluppo] — Il server di sviluppo Vite può non accorgersi della fine del genitore su Windows
- **File e Righe**: [`scripts/dev-ui.mjs:103-114`](../../../scripts/dev-ui.mjs#L103-L114).
- **Analisi Tecnica**: il controllo ogni secondo confronta `process.ppid` con il valore iniziale e prova `process.kill(parent, 0)`. Su Windows `process.ppid` non cambia quando il genitore muore, e `kill(pid, 0)` riesce se nel frattempo il PID è stato riassegnato a un altro processo.
- **Scenario di guasto**: raro: chiusa l'app durante `npm run dev`, il PID del genitore viene riusato prima del controllo successivo e Vite resta vivo sulla porta 5173 (lo stesso sintomo che D-138 voleva eliminare). Non riguarda gli utenti dell'app pubblicata.
- **Correzione Proposta**: accettabile così; in alternativa uscire anche alla chiusura dello stdin ereditato dal genitore (`process.stdin.on("end", ...)`).

### 3.4 UX

#### `UX-V5-01` [MEDIA] — Pool web attivi e linee MySQL secondarie invisibili in barra di stato e Attività (residuo di `UX-V4-01`)
- **File e Righe**: [`ui/src/App.tsx:104, 175`](../../../ui/src/App.tsx#L104) (badge «Progetti avviati»), [`App.tsx:505, 532`](../../../ui/src/App.tsx#L505) (barra di stato), [`ui/src/activity.tsx:49`](../../../ui/src/activity.tsx#L49) (`liveServices`).
- **Analisi Tecnica**: il conteggio considera solo le esecuzioni dei progetti; il pannello Attività filtra i servizi sullo stato della linea principale e non legge `others` ([`ui/src/api/services.ts:26`](../../../ui/src/api/services.ts#L26)); i pool web attivi non compaiono affatto. Manca un'azione «Ferma tutto».
- **Scenario di guasto**: con la linea MySQL principale ferma e la 8.4 avviata da un progetto, oppure con un pool Caddy acceso, la barra di stato mostra 0 e Attività non elenca nulla: l'utente chiude il progetto convinto che non giri niente, mentre porte e processi restano occupati fino all'uscita dall'app.
- **Correzione Proposta**: includere `s.others.filter(o => o.state !== "stopped")` e i pool attivi (`serversList`) in `liveServices` e nel conteggio; un'azione «Ferma tutto» nel pannello. Test in `test/ui` per il conteggio con primaria ferma e secondaria attiva.

#### `UX-V5-02` [BASSA] — Escape non chiude la conferma del terminale; profilo di pool eliminato senza conferma (residuo di `UX-V4-06`)
- **File e Righe**: [`ui/src/terminals.tsx:257-287`](../../../ui/src/terminals.tsx#L257-L287) (`<InlineConfirm>` senza `onCancel`), [`ui/src/servers.tsx:137`](../../../ui/src/servers.tsx#L137) e [`servers.tsx:524-530`](../../../ui/src/servers.tsx#L524-L530) (`onRemove` → `serverRemove`); `InlineConfirm` chiama `onCancel` su Esc ([`ui/src/workbench/dialog.tsx:154-180`](../../../ui/src/workbench/dialog.tsx#L154-L180)).
- **Analisi Tecnica**: la conferma del terminale ha il pulsante «Annulla» ma non passa `onCancel`, quindi Esc non fa nulla, a differenza delle altre conferme. «Elimina profilo» del pool esegue `serverRemove` al primo clic, senza domanda.
- **Scenario di guasto**: un clic accidentale su «Elimina profilo» cancella porta, versione e opzioni del pool; chi usa la tastiera non riesce a chiudere la conferma del terminale con Esc.
- **Correzione Proposta**: `onCancel={() => setConfirmRemove(false)}` in `terminals.tsx`; un `InlineConfirm` prima di `serverRemove` in `servers.tsx`; test in [`test/ui/servers.test.tsx`](../../../test/ui/servers.test.tsx) e in un test del terminale.

### 3.5 Tabella riassuntiva `v5`

| ID | Severità | File principale | Origine | Sintesi |
|---|:---:|---|---|---|
| **`BUG-V5-02`** | **ALTA** | [`db/services/mod.rs:160-230, 550-610`](../../../tauri/src/db/services/mod.rs#L550-L610) | `BUG-V4-04/05` | Nessun lock su avvio DB e `services.json`; porte scambiate al cambio di linea principale. |
| **`BUG-V5-01`** | MEDIA | [`runtime/php-router.php:23-32`](../../../runtime/php-router.php#L23-L32) | `SEC-V4-02.1` | `php -S` in LAN risponde 421 a ogni IP. |
| **`BUG-V5-05`** | MEDIA | [`servers/config.rs:49-72, 166-189`](../../../tauri/src/servers/config.rs#L49-L72), [`servers/pools.rs:533-566`](../../../tauri/src/servers/pools.rs#L533-L566) | `BUG-V4-02/03` | `conf.d` del progetto ignorato nei pool; porte dei pool non controllate. |
| **`UX-V5-01`** | MEDIA | [`ui/src/activity.tsx:49`](../../../ui/src/activity.tsx#L49), [`ui/src/App.tsx:104`](../../../ui/src/App.tsx#L104) | `UX-V4-01` | Pool e linee MySQL secondarie invisibili; manca «Ferma tutto». |
| **`SEC-V5-01`** | BASSA | [`projects/serve.rs:55-67`](../../../tauri/src/projects/serve.rs#L55-L67) | `SEC-V4-03` | `*.localhost` rifiutato dal server statico. |
| **`BUG-V5-03`** | BASSA | [`github/mod.rs:214-241`](../../../tauri/src/github/mod.rs#L214-L241) | nuovo (D-137) | `safe.directory` dell'utente rimosso se la cartella non c'è. |
| **`BUG-V5-04`** | BASSA | [`db/services/tools.rs:175, 197`](../../../tauri/src/db/services/tools.rs#L175) | `BUG-V4-08` | File di login dei dump con il solo PID. |
| **`UX-V5-02`** | BASSA | [`ui/src/terminals.tsx:258`](../../../ui/src/terminals.tsx#L258), [`ui/src/servers.tsx:524-530`](../../../ui/src/servers.tsx#L524-L530) | `UX-V4-06` | Esc non chiude la conferma; profilo pool eliminato senza conferma. |
| **`PORT-V5-01`** | BASSA (sviluppo) | [`scripts/dev-ui.mjs:103-114`](../../../scripts/dev-ui.mjs#L103-L114) | nuovo (D-138) | Vite può sopravvivere all'app su Windows per PID riusato. |

### 3.6 Punti controllati senza rilievi

- `settings_set` conserva `tools_target`, `other` e la chiave Gemini dal disco sotto `Settings::update` ([`commands/app.rs:216-261`](../../../tauri/src/commands/app.rs#L216-L261)): l'autosave della pagina non sovrascrive «Piattaforma».
- Percorso del PEM relativo al kit risolto al caricamento ([`settings.rs:166-170`](../../../tauri/src/settings.rs#L166-L170)).
- `git config --global` usa l'HOME del kit ([`env.rs:402-433`](../../../tauri/src/env.rs#L402-L433)): nessun residuo sul PC.
- Permessi `pty_*` solo alla finestra `main` ([`tauri/permissions/pty.toml`](../../../tauri/permissions/pty.toml), [`capabilities/main.json:4-6`](../../../tauri/capabilities/main.json#L4-L6)).
- CSP invariata rispetto al `v4` ([`tauri/tauri.conf.json:22`](../../../tauri/tauri.conf.json#L22)).
- Socket admin di Caddy privati, mai TCP `:2019` ([`projects/caddy.rs:14-19, 173-215`](../../../tauri/src/projects/caddy.rs#L14-L19)).

---

## 4. Funzionalità mancanti e lavoro aperto

Fonti: sezione «Still to do» del [`README.md:99-108`](../../../README.md#L99-L108), issue GitHub aperte #5 e #6, [`DECISIONS.md:1077`](../../app/DECISIONS.md#L1077), rilievi sopra.

| # | Funzionalità / lavoro | Stato in `a8aad6c` | Fonte |
|---|---|---|---|
| F-1 | **Prova dei rilasci su macchine pulite** (macOS Apple Silicon e Intel; Linux x64 e ARM64 su Ubuntu 24.04, Fedora 42, Debian 13; AppImage con e senza FUSE, chiavette exFAT o `noexec`) | Costruiti e avviati solo in CI | [`README.md:103-106`](../../../README.md#L103-L106), issue #5 |
| F-2 | **Accessi portabili degli agenti su un Mac reale** (Portachiavi, `/usr/bin/security`, D-115) | Da provare | [`README.md:105`](../../../README.md#L105), issue #6 |
| F-3 | **Aggiornamento automatico da una versione all'altra su macOS e Linux** | Non provato | [`README.md:107`](../../../README.md#L107) |
| F-4 | **Apache (D-116) e PHP del kit (D-133) fuori Windows** | Nel catalogo, provati solo in CI; macOS e Linux ARM64 «non ancora con un kit vero» | [`README.md:107`](../../../README.md#L107), [`DECISIONS.md:1077`](../../app/DECISIONS.md#L1077) |
| F-5 | **Firma del codice**: notarizzazione Apple (oggi firma ad hoc, serve `xattr -dr com.apple.quarantine`); firma SignPath su Windows (0.3.0 senza firma, SmartScreen può avvisare) | Da attivare | [`README.md:108`](../../../README.md#L108), D-113 |
| F-6 | **«Ferma tutto»** e vista unica di esecuzioni, pool e linee DB attive | Assente | `UX-V5-01` |
| F-7 | **Registro comune delle porte** (pool, DB, progetti, editor) | Assente | `BUG-V5-05` |
| F-8 | **Allineamento grafico a VSCodium** (15 differenze: schede del pannello, barra delle attività, liste, intestazioni, scrollbar, pulsanti, editor delle estensioni e delle impostazioni, barra del titolo, schede dell'editor, dialoghi, barra di stato, campi, icone codicon, token di misura) | Approvato, fasi 0-5 pianificate | [`07-PIANO-ESECUTIVO-V5.md`](07-PIANO-ESECUTIVO-V5.md) |
| F-9 | **Android SDK su Linux ARM64** | Non disponibile: Google non pubblica platform-tools e build-tools per Linux ARM64 | [`components.json:19`](../../../tauri/catalog/components.json#L19) |

F-9 è un limite esterno, non lavoro pianificato. F-1..F-5 richiedono hardware o account esterni (Apple Developer, SignPath) e non si chiudono dal solo codice.
