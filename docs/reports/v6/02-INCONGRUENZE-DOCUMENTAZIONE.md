# 02 — Incongruenze tra Documentazione e Applicazione (`v6`)

> **Riferimento**: `HEAD 6ecd492`. Documenti confrontati: README in 4 lingue, `docs/PRIVACY*.md`, `docs/app/{GUIDE,CATALOG,ARCHITECTURE,TESTING}.md` e gemelli `.it.md`, `CHANGELOG.md`, `DECISIONS.md`, `TOOLS-MATRIX.html`, Guida dell'app (`ui/src/guide/**`), sito (`site/`), aiuto della riga di comando.

---

## 1. Cosa torna

- **Conteggi**: 35 componenti, 63 basi + 18 aggiunte in 10 gruppi, 9 capitoli della Guida, 7 sezioni di Impostazioni, 7 sezioni del progetto, 9 pagine.
- **Piattaforme**: i 9 componenti `x64Only` in emulazione su Windows ARM64; Android assente su `linux-arm64`.
- **Versioni**: 0.5.0 in `tauri/Cargo.toml` e `ui/package.json`; editor 1.135.06055, 1845 MB del primo avvio, MySQL 8.4/9.7, PHP 8.4.26/8.5.11, Apache 2.4.69-1.
- **Collegamenti**: 633 link relativi e ancore validi; albero dei moduli di ARCHITECTURE coerente col codice.
- **File generati**: `node scripts/guide-docs.cjs --check` dice GUIDE.md e GUIDE.it.md aggiornati; `TOOLS-MATRIX.html` rigenerato è identico.
- **Traduzioni**: stesse sezioni e stessi fatti nei 4 README, nelle 4 informative e nelle 4 stringhe del sito.
- **Informativa**: percorsi, variabili, User-Agent, timer di 10 s e 20 s, interrogazione degli agenti ogni 2 minuti, `ATOLLO_NO_UPDATE`.

---

## 2. Rilievi

| ID | Grav. | Documento | Codice | Il documento dice | Il codice fa | Da correggere |
|---|:---:|---|---|---|---|---|
| `DOC-V6-01` | MEDIA | [`README.md:145-163`](../../../README.md#L145-L163) e i 3 gemelli | [`cli.rs:99-188`](../../../tauri/src/cli.rs#L99-L188), [`main.rs:461`](../../../tauri/src/main.rs#L461) | elenco dei comandi senza `versions [id]`, `env --project`, `components --target`, `prepare --line id=x.y`, `prepare --target` | li implementa e li mostra nell'aiuto; GUIDE §8.5 li elenca | README (×4), copiando GUIDE §8.5 |
| `DOC-V6-02` | MEDIA | aiuto `HELP` ([`main.rs:461`](../../../tauri/src/main.rs#L461), `tauri/i18n/catalog.json`) | [`setup/runner.rs:281`](../../../tauri/src/setup/runner.rs#L281), [`:293`](../../../tauri/src/setup/runner.rs#L293) | «installa e aggiorna i componenti scelti» | `prepare` **rimuove** anche quelli non scelti; `--only` porta anche le dipendenze (README e GUIDE lo dicono) | codice: testo dell'aiuto in 4 lingue. Chi legge solo l'aiuto può perdere componenti |
| `DOC-V6-03` | MEDIA | `GUIDE.md:996`, `:1101`, `:2360` | [`guide/topics/services.tsx:29`](../../../ui/src/guide/topics/services.tsx#L29), [`:999-1001`](../../../ui/src/guide/topics/services.tsx#L999-L1001), [`guide/sources.ts:961`](../../../ui/src/guide/sources.ts#L961) | testo italiano nella Guida inglese, francese e tedesca: «32 caratteri generati dal kit», `@altro-progetto/public`, `MySQL-<linea>` | stringhe fisse non tradotte; `lLine()` esiste a `sources.ts:76` ma non è usato | sorgenti della Guida, poi `node scripts/guide-docs.cjs` |
| `DOC-V6-04` | MEDIA | [`README.md:117`](../../../README.md#L117) (×4), `GUIDE.md:72`, `site/strings/en.json:78` | [`catalog/relations.json:28`](../../../tauri/catalog/relations.json#L28), [`commands/setup.rs:43`](../../../tauri/src/commands/setup.rs#L43), [`:212-215`](../../../tauri/src/commands/setup.rs#L212-L215) | «Windows 10 e 11 (x64 e ARM64)» | l'emulazione x64 richiede Windows 11 (build ≥ 22000): su Windows 10 ARM64 l'editor, obbligatorio, non è disponibile | documenti: «Windows ARM64 richiede Windows 11» (README ×4, sito, `start.tsx`) |
| `DOC-V6-05` | MEDIA | [`settings/about.tsx:96-107`](../../../ui/src/settings/about.tsx#L96-L107), `GUIDE.md:1779` | [`docs/PRIVACY.md:40-42`](../../PRIVACY.md#L40-L42) | «agenti, GitHub e registri parlano direttamente con i loro servizi» | Atollo stesso manda il token di ogni agente ad `api.anthropic.com`, `chatgpt.com`, `cloudcode-pa.googleapis.com` ogni 2 minuti con la pagina Agenti aperta, e usa il token GitHub con `api.github.com` | testo dell'app: un punto in più nel riepilogo privacy |
| `DOC-V6-06` | MEDIA | [`TESTING.md:117`](../../app/TESTING.md#L117), `TESTING.it.md:117` | [`test/rust/db/services/data.rs:315`](../../../test/rust/db/services/data.rs#L315) | prova `a_new_kit_needs_mysql_for_every_sql_need` | si chiama `mysql_is_the_kits_sql_server` | documenti |
| `DOC-V6-07` | BASSA | [`CATALOG.md:50`](../../app/CATALOG.md#L50) (+ `.it`) | `components.json` (`gh` senza `needs`), [`test/rust/setup/catalog.rs:126`](../../../test/rust/setup/catalog.rs#L126), commento in [`setup/catalog.rs:166-168`](../../../tauri/src/setup/catalog.rs#L166-L168) | «più Git, sempre installato» | Git non è bloccato (D-81); è nel primo avvio perché serve a Claude Code | documenti: «più Git, che serve a Claude Code»; codice: commento |
| `DOC-V6-08` | BASSA | [`CATALOG.md:159`](../../app/CATALOG.md#L159) (+ `.it`) | [`install/web.rs:855-876`](../../../tauri/src/install/web.rs#L855-L876) | `data\php\php.ini` | `data\php\<linea>\php.ini` (Windows), `data/php/<target>/<linea>/php.ini` (macOS/Linux) | documenti |
| `DOC-V6-09` | BASSA | [`ARCHITECTURE.md:151`](../../app/ARCHITECTURE.md#L151) (+ `.it`) | [`db/auth.rs:596`](../../../tauri/src/db/auth.rs#L596), [`:603`](../../../tauri/src/db/auth.rs#L603) | `data\app\db\` contiene «secrets.json, my.ini» | `mysql.cnf` e `mysql-lines\<linea>\mysql.cnf`; il `my.cnf` dei server è in `data\db\<motore>\<versione>` | documenti |
| `DOC-V6-10` | BASSA | [`README.md:120`](../../../README.md#L120) (×4) | [`docs/PRIVACY.md:76-82`](../../PRIVACY.md#L76-L82) | tracce fuori dalla cartella: solo Windows, temporanei, `gh`, Antigravity | anche accesso di Claude nel Portachiavi (macOS), dati della webview in Library (macOS), voce HKCU di rustup (rimossa dopo); con «Temporanei nel sistema» i file restano fino alla chiusura | README: aggiungere o rimandare a PRIVACY §4 |
| `DOC-V6-11` | BASSA | [`docs/PRIVACY.md:38`](../../PRIVACY.md#L38) (×4) | [`install/web.rs:105`](../../../tauri/src/install/web.rs#L105), [`:320`](../../../tauri/src/install/web.rs#L320), [`:722-728`](../../../tauri/src/install/web.rs#L722-L728), [`:1157`](../../../tauri/src/install/web.rs#L1157), [`install/core.rs:332`](../../../tauri/src/install/core.rs#L332), [`install/native.rs:555`](../../../tauri/src/install/native.rs#L555) | l'informativa dice di elencare «ogni connessione» | mancano `www.apachelounge.com`, `dl.static-php.dev`, `curl.se`, `aka.ms`, `julialang-s3.julialang.org`, `storage.googleapis.com`, `raw.githubusercontent.com`, `api.anaconda.org` | documenti (×4); meglio un elenco generato dal catalogo |
| `DOC-V6-12` | BASSA | `site/strings/en.json:105` (×4) | `PRIVACY.md:39`, README:209 | il sito si collega solo a GitHub, alle sorgenti e alle API di GitHub e degli agenti | anche Open VSX (estensioni) | stringhe del sito |
| `DOC-V6-13` | BASSA | `site/strings/en.json:31` (×4) | [`site/template.html:127-132`](../../../site/template.html#L127-L132) | «Four parts, one folder» | cinque riquadri (f1-f5) | sito |
| `DOC-V6-14` | BASSA | [`README.md:95`](../../../README.md#L95) (×4) | [`settings/sections.tsx:161`](../../../ui/src/settings/sections.tsx#L161), [`settings/conda.tsx`](../../../ui/src/settings/conda.tsx) | riga Impostazioni | mancano «File temporanei dei programmi», «Pacchetti da reinstallare senza rete», sezione «Il kit» | README (×4) |
| `DOC-V6-15` | BASSA | [`ARCHITECTURE.md:143-166`](../../app/ARCHITECTURE.md#L143-L166) | `GUIDE.md:2164-2179` | albero di `data\` | mancano `agents\`, `gh\`, `cache\`, `vscodium-server\`, `webview2\`, `antigravity\`, `php\`, `apache\`, `npm-global\` | documenti |
| `DOC-V6-16` | BASSA | [`ARCHITECTURE.md:175`](../../app/ARCHITECTURE.md#L175), [`:381`](../../app/ARCHITECTURE.md#L381) | `projects/run.rs`, `guide/topics/projects.tsx` | 1189 e 2118 righe | 1197 e 2132 | togliere i conteggi esatti, che invecchiano a ogni commit |

---

## 3. Incongruenze emerse dalla revisione del codice

Non sono errori dei documenti ma promesse che il codice non mantiene in casi limite; si chiudono correggendo il codice (documento [01](./01-BUG-SICUREZZA-PORTABILITA.md)):

| Promessa | Dove | Rilievo |
|---|---|---|
| «spaces, brackets or `&` in the name are fine» | [`README.md:69`](../../../README.md#L69) | `BUG-V6-12` (Maven), `BUG-V6-31` (apici tipografici), `PORT-V6-04` (Gradle non ASCII) |
| la stessa cartella passa da un PC all'altro | README, «Features» | `PORT-V6-01` (DACL su NTFS), `BUG-V6-03` (radice dell'unità) |
| backup e ripristino dei database del progetto | Guida, capitolo Database | `BUG-V6-01`, `BUG-V6-04`, `BUG-V6-06` |
| «Remove drive» chiude tutto | README, pagina «Remove drive» | `BUG-V6-15` |
| «Annulla» ferma subito un download | intestazione di [`net/mod.rs`](../../../tauri/src/net/mod.rs) | `BUG-V6-13` |
| il nome del progetto può avere spazi | [`kit.rs:779`](../../../tauri/src/kit.rs#L779) | `BUG-V6-08` (Caddy), `BUG-V6-07` (MySQL > 32) |

---

## 4. Strategia contro la deriva

1. **Una fonte per l'elenco dei comandi.** Generare il blocco «Command line» dei README da `HELP` (come `guide-docs.cjs` genera la Guida), con `--check` in CI. Chiude `DOC-V6-01` e `DOC-V6-02` per sempre.
2. **Host di rete generati.** Uno script che estrae gli host da `components.json` e dagli `https://` di `tauri/src/install/**` e confronta con PRIVACY: una prova fallisce se un host nuovo manca.
3. **Niente numeri che invecchiano.** Togliere i conteggi di righe da ARCHITECTURE; tenere solo quelli verificati da una prova (componenti, basi, capitoli).
4. **Prova «nessun italiano nelle lingue straniere».** In `guide-docs.cjs --check`, cercare parole italiane frequenti (`caratteri`, `progetto`, `linea`) in GUIDE.md.
5. **Nomi di prove citati.** Una prova che controlla che ogni nome di prova citato in TESTING*.md esista in `test/`.
