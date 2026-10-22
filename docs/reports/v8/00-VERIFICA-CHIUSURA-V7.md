# Atollo — Verifica della chiusura dei rilievi `v7`

> **Data**: 23 ottobre 2026 · **Riferimento**: ramo `docs/v7-consistency` (`HEAD 71f96d5` prima di questo file), costruito su `feature/v7-guide-site`, che contiene `fix/v7-bugs`, `fix/v7-followups` e `refactor/v7-structure`; nessuno dei rami dell'ondata 2-4 è ancora in `main`. Unico rilascio pubblico: `app-v0.5.1` (21/10/2026).
> **Metodo**: per ogni rilievo dei documenti [`v7/01`](../v7/01-BUG-SICUREZZA-PORTABILITA.md)-[`04`](../v7/04-GUIDA-NEUTRA-E-SITO.md), il commit che lo cita (`git log --grep`, da `1f84370`, l'`HEAD` della revisione `v7`); poi una rilettura nel codice, non solo nel messaggio di commit, di tutti i rilievi di gravità MEDIA (nessuno era ALTA) e di un campione dei BASSA. Prove eseguite il 23/10/2026 su Windows 11 x64.

---

## 1. Sintesi

| Famiglia | Rilievi | Chiusi | Superati | Non applicabili | Aperti |
|---|:---:|:---:|:---:|:---:|:---:|
| Bug, sicurezza, portabilità (`BUG-V7-*`) | 39 | 39 | 0 | 0 | 0 |
| Struttura e duplicati (`STR-V7-*`) | 22 | 22 | 0 | 0 | 0 |
| Guida neutra e sito (`GUI-V7-*`) | 12 | 10 | 2 | 0 | 0 |
| Documentazione (`DOC-V7-*`) | 13 | 13 | 0 | 0 | 0 |
| Emersi nell'ondata 1 (`SEC-V7-01`, `TEST-V7-02`) e osservazione `TEST-V7-01` | 3 | 3 | 0 | 0 | 0 |
| **Totale** | **89** | **87** | **2** | **0** | **0** |

**Nessun rilievo `v7` resta aperto nel codice o nei documenti.** `GUI-V7-04` e `GUI-V7-05` sono superati: il renderer Markdown che descrivevano non c'è più (decisione del proprietario, D-164). Restano invece le verifiche che solo una persona o la pubblicazione possono fare (§3), e tre proposte di strategia del piano `v7` non applicate (§4): non sono rilievi.

Rilettura nel codice dei MEDIA (tutti confermati):

- `BUG-V7-01` — la ripresa parte dalla voce del manifest e riparte dai file quando il commit è cambiato (`tauri/src/llm/pull.rs`, commenti alle righe 447 e 582); `BUG-V7-05` con lo SHA giusto prima di accettare un file.
- `BUG-V7-09` — `apache::quoted_path` (`tauri/src/projects/apache.rs:37`) rifiuta `"`, `\`, a capo e `${`; la usano l'Apache del progetto e i pool (`servers/config.rs:46`).
- `BUG-V7-15` — il mutex `starting` separato da quello del server (`tauri/src/editor/mod.rs:94-98`): i comandi sincroni non aspettano più l'avvio.
- `BUG-V7-23`, `BUG-V7-25` — `ErrorLine` nella pagina Terminali e `useAction` con errore nel pannello Attività (`ui/src/terminal/page.tsx`, `ui/src/shell/activity.tsx:165`).
- `SEC-V7-01` — `launch_vars` non dà più il token di GitHub a nessuno (`tauri/src/agents/presets.rs:168-173`); il server github è `Atollo mcp github` (`agents/github_mcp.rs`, verso `api.githubcopilot.com`), con le sue prove in `test/rust/agents/`.
- `STR-V7-01..03`, `-12`, `-19..21` — `crate::ports` con le sonde, `projects/run/` e `env/` cartelle, `scripts/matrix/gen_matrix.mjs`, `ui/src/{shared,shell,servers}`, gli alias in `ui/tsconfig.json`, `test/rust/support.rs`.
- `GUI-V7-01..03`, `-10`, `-11` — `guide/core` (contratto, lingua, icone come nomi), capitoli senza import dell'app, `text.test.tsx` che fallisce senza file (`> 10` file e `core/lang.ts`), livello `site` nella CI (`app.yml`) e `site.yml`.
- `DOC-V7-01..03` — controllati sul sito costruito: `data-count="4"` dal catalogo; nota «in arrivo» nei 4 README e nelle stringhe del sito; Git non più «sempre installato» nella Guida e nel suggerimento di Strumenti.

---

## 2. Tabella dei rilievi

Stato: **chiuso** (corretto, commit citato), **superato** (la cosa descritta non esiste più), **non applicabile**, **aperto**.

### 2.1 Bug, sicurezza, portabilità (ramo `fix/v7-bugs` e sotto-rami)

| ID | Stato | Commit | Nota |
|---|---|---|---|
| `BUG-V7-01` | chiuso | `a76ffe9` | download sospeso con commit cambiato: riparte dai file del commit nuovo |
| `BUG-V7-02` | chiuso | `98b7788` | un download per cartella del modello, qualunque maiuscola del repository |
| `BUG-V7-03` | chiuso | `ad855a6` | lock del download preso in modo atomico |
| `BUG-V7-04` | chiuso | `1eec50d` | ripresa di un id con il nome dell'editore |
| `BUG-V7-05` | chiuso | `48034fd` | file della dimensione giusta accettato solo con lo SHA giusto |
| `BUG-V7-06` | chiuso | `7f906fd` | mese di un file aggiunto a mano |
| `BUG-V7-07` | chiuso | `b3da6f3` | token HF illeggibile non blocca più il catalogo |
| `BUG-V7-08` | chiuso | `9c93d3b` | repository ad accesso controllato: lo dice |
| `BUG-V7-09` | chiuso | `2b28879` | `apache::quoted_path`; anche in ARCHITECTURE (`329d29a`) |
| `BUG-V7-10` | chiuso | `1a75cc4`, `b68c04b` | modalità LAN: solo gli indirizzi d'ascolto, i client restano su 127.0.0.1 |
| `BUG-V7-11` | chiuso | `55c1238` | socket MySQL lungo nella cartella privata dei socket |
| `BUG-V7-12` | chiuso | `e6c05b8` | Apache o Caddy mancante nella lingua dell'utente |
| `BUG-V7-13` | chiuso | `b4ead23` | carattere UTF-8 diviso fra due letture |
| `BUG-V7-14` | chiuso | `de6e403` | processi uccisi per timeout raccolti (`wait`) |
| `BUG-V7-15` | chiuso | `193b075`, `9d31ecf` | lock del server dell'editor solo per guardarlo; uno stop chiude un avvio in corso |
| `BUG-V7-16` | chiuso | `4d0a900` | uscita di `setup-token` anche nella sua finestra |
| `BUG-V7-17` | chiuso | `ac2f4c2` | `TMPDIR` letto all'avvio (decisione 4); da provare a mano da Finder |
| `BUG-V7-18` | chiuso | `e833c83` | maiuscole della radice nell'id del kit solo dove il disco le ignora |
| `BUG-V7-19` | chiuso | `23ed88b` | processi del kit con le regole dei percorsi del PC |
| `BUG-V7-20` | chiuso | `8bc9c50` | aperture staccate e server dell'editor fermato raccolti |
| `BUG-V7-21` | chiuso | `2e46b7e` | timeout sulle letture del socket dell'istanza |
| `BUG-V7-22` | chiuso | `113539e` | modifica fatta durante il ricaricamento dopo un salvataggio |
| `BUG-V7-23` | chiuso | `7c5d194` | «Salva e apri»: l'errore si vede |
| `BUG-V7-24` | chiuso | `8b1d168` | solo i file dell'ultima ricerca su Hugging Face |
| `BUG-V7-25` | chiuso | `25e92e7` | i «Ferma» del pannello Attività dicono perché non riescono |
| `BUG-V7-26` | chiuso | `b60c618` | passi della preparazione per posizione |
| `BUG-V7-27` | chiuso | `e161f0a` | Server Web: lettura vecchia scartata, errore tolto |
| `BUG-V7-28` | chiuso | `62cdef0` | letture fallite non passano per dati |
| `BUG-V7-29` | chiuso | `75884d7` | interruttore della barra di stato: solo i terminali del pannello |
| `BUG-V7-30` | chiuso | `8c02414`, `6db5f10` | log di un'azione: righe tenute e unite per posizione |
| `BUG-V7-31` | chiuso | `1dea7ad` | stato dei modelli letto prima di un'azione non la annulla |
| `BUG-V7-32` | chiuso | `fdc7999` | componenti richiesti di nuovo dopo una lettura fallita |
| `BUG-V7-33` | chiuso | `9e19dea` | trascinamento di un divisore annullato |
| `BUG-V7-34` | chiuso | `1b2a767` | commento superato nel codice |
| `BUG-V7-35` | chiuso | `59f54a7` | rifiuti dei download dei modelli tradotti |
| `BUG-V7-36` | chiuso | `98dd567` | prova i18n Rust su ogni forma di chiamata |
| `BUG-V7-37` | chiuso | `6a09a56` | segnaposto neutro per il nome di una variabile `.env` |
| `BUG-V7-38` | chiuso | `23be8b4` | pagina di benvenuto di Apache neutra (decisione 5) |
| `BUG-V7-39` | chiuso | `58a70b3` | stringa del sito non usata tolta |

### 2.2 Emersi durante l'ondata 1 (ramo `fix/v7-followups`)

| ID | Stato | Commit | Nota |
|---|---|---|---|
| `SEC-V7-01` | chiuso | `c327cc2`, `39bbcba` | token di GitHub fuori dall'ambiente dell'agente, ponte `Atollo mcp github` avviato col percorso assoluto; informativa in `3d50437` |
| `TEST-V7-01` | chiuso | `a5abb36` | metà delle CPU come worker di Vitest; non `vmThreads` come proponeva la decisione 7. Il 23/10 un giro di `npm -w ui test` con `npm run lint` in parallelo ha avuto 1 file fallito, verde ai giri successivi |
| `TEST-V7-02` | chiuso | `cadae1d` | la porta del file server resta legata; riprova sugli errori di connessione |

### 2.3 Struttura e duplicati (ramo `refactor/v7-structure`)

| ID | Stato | Commit | Nota |
|---|---|---|---|
| `STR-V7-01` | chiuso | `de79d7f` | sonde delle porte in `crate::ports` |
| `STR-V7-02` | chiuso | `0e6d2dc` | `projects/run/` in parti |
| `STR-V7-03` | chiuso | `6403552` | `env/`, `KitEnv::build` in parti |
| `STR-V7-04` | chiuso | `481304f` | `projects/caddy/`: socket, conf, command |
| `STR-V7-05` | chiuso | `0f7bce6` | `install/`: flow, parts, keys |
| `STR-V7-06` | chiuso | `2043bc4` | `kit/`: target, root, names |
| `STR-V7-07` | chiuso | `b366371` | `main.rs`, `app.rs`, `cli.rs` |
| `STR-V7-08` | chiuso | `c505481` | riesportazioni del catalogo e dei file privati |
| `STR-V7-09` | chiuso | `6d65582` | riesportazioni al posto delle funzioni che inoltrano |
| `STR-V7-10` | chiuso | `db2f5e5` | un solo `tr` |
| `STR-V7-11` | chiuso | `e2592d4` | un solo orologio, `crate::time` |
| `STR-V7-12` | chiuso | `db29e3c` | matrice in JavaScript (`gen_matrix.mjs`) |
| `STR-V7-13` | chiuso | `333165b` | `scripts/lib/shots.mjs` |
| `STR-V7-14` | chiuso | `73c5ba5` | `shared/format.ts` |
| `STR-V7-15` | chiuso | `afefc9b` | motori e campi delle porte in un posto |
| `STR-V7-16` | chiuso | `56a7d83` | `DotTone` da workbench |
| `STR-V7-17` | chiuso | `421eda9` | `workbench/{buttons,inputs,indicators}` |
| `STR-V7-18` | chiuso | `78e0731` | modello della Guida in `guide/core` (ondata 3) |
| `STR-V7-19` | chiuso | `4c8807e` | `shared/`, `shell/`, `servers/`; il link di MARKETING a `brand.tsx` rotto da questo spostamento corretto in `42816b5` |
| `STR-V7-20` | chiuso | `7d4b61c` | alias `@/`, `@guide`, `@test`, un solo mock di Tauri |
| `STR-V7-21` | chiuso | `6456080` | `test/rust/support.rs` |
| `STR-V7-22` | chiuso | `ba16a1c` | prove Rust divise come i sorgenti |

### 2.4 Guida neutra e sito (ramo `feature/v7-guide-site`)

| ID | Stato | Commit | Nota |
|---|---|---|---|
| `GUI-V7-01` | chiuso | `77d0809`, `7d4cfbb` | contratto dei widget, icone come nomi |
| `GUI-V7-02` | chiuso | `d976bea` | widget solo-app con ripiego |
| `GUI-V7-03` | chiuso | `0164fc0` | collegamenti dichiarativi |
| `GUI-V7-04` | superato | `1f116f7` | renderer Markdown tolto |
| `GUI-V7-05` | superato | `1f116f7` | `guide-docs.cjs` tolto |
| `GUI-V7-06` | chiuso | `87f66a5` | nessuna classe nel testo |
| `GUI-V7-07` | chiuso | `916c5a7` | testo solo per l'app o solo per il sito |
| `GUI-V7-08` | chiuso | `78e0731` | chunk della build su `guide/` |
| `GUI-V7-09` | chiuso | `78e0731` | lint, formattazione e tipi su `guide/` |
| `GUI-V7-10` | chiuso | `78e0731` | le prove del testo falliscono senza file |
| `GUI-V7-11` | chiuso | `b9994fa` | sito costruito e controllato nella CI; il livello `site` non si è ancora visto girare (§3) |
| `GUI-V7-12` | chiuso | `561539a` | modello in partial, pagine descritte al builder |

### 2.5 Documentazione (ramo `docs/v7-consistency`)

| ID | Stato | Commit | Nota |
|---|---|---|---|
| `DOC-V7-01` | chiuso | `1c32e84` | `agentCount` dal gruppo `agents` del catalogo |
| `DOC-V7-02` | chiuso | `c2dfeca` | decisione 1: «in arrivo nel prossimo rilascio» nei 4 README e nel sito, «Non ancora rilasciato» nel CHANGELOG con tutto ciò che è entrato dopo `app-v0.5.1` (D-145..D-164 e le correzioni v7). La nota va tolta nel commit che alza la versione |
| `DOC-V7-03` | chiuso | `46d96bc` | Git arriva con Claude Code e Pi: Guida (agenti, primo avvio, tipi di Strumenti) e suggerimento di Strumenti con en/fr/de |
| `DOC-V7-04` | chiuso | `7c42bda` | «Modelli locali» e il suo server nella barra delle attività e nel contatore della Guida |
| `DOC-V7-05` | chiuso | `7dfec93` | `make.mjs` con la pagina dei modelli e Pi installato nell'anteprima; schermate della Guida (56) e del sito (24) rigenerate nelle 4 lingue |
| `DOC-V7-06` | chiuso | `3a6adbb` | otto tipi in README ×4, CATALOG, TESTING |
| `DOC-V7-07` | chiuso | `c41833d` | `diagnostics.rs`, `health.rs`, `backup.rs`, `tray.rs`, `app\diagnostics\`; niente conteggi esatti di righe |
| `DOC-V7-08` | chiuso | `3a6adbb` | ConPTY come esempio di `targets` |
| `DOC-V7-09` | chiuso | `8ee05d3` | la chiave nel `models.json` di Pi, README ×4 |
| `DOC-V7-10` | chiuso | `7c42bda` | `a2Text` del sito nelle 4 lingue |
| `DOC-V7-11` | chiuso | `8ee05d3` | «Il kit» con Salute, Copia, Esporta diagnostica, Segnala un problema |
| `DOC-V7-12` | chiuso | `773e2d0` | quattro agenti in MARKETING |
| `DOC-V7-13` | chiuso | `3a6adbb` | 1848 MB su Windows, con ConPTY (calcolato da `components.json`) |

---

## 3. Da fare a mano

1. **Pubblicare il sito** da `site.yml` dopo il merge in `main`, poi in Search Console **reinviare `sitemap.xml`** (152 pagine, di cui le pagine della Guida nuove).
2. **Ponte MCP di GitHub con i quattro agenti** (`SEC-V7-01`): con l'account GitHub del kit e il server github aggiunto, chiedere a Claude Code, Codex, Antigravity e Pi un'operazione su issue o pull request; controllare che funzioni e che `GITHUB_PERSONAL_ACCESS_TOKEN` non sia nell'ambiente dell'agente (né in quello di un server del `.mcp.json` di un progetto).
3. **Kit spostato**: spostare la cartella del kit (altro disco o altra lettera) e controllare che `agents::relocate` riscriva il percorso assoluto del ponte in `mcp.json` e che il server github riparta.
4. **CI del livello `site`** (`GUI-V7-11`): non si è ancora visto girare su GitHub; controllarlo alla prima pull request che tocca `site/`, `guide/` o `ui/public/guide/`.
5. **`TMPDIR` su macOS** (`BUG-V7-17`): avviare l'app da Finder, come diceva il piano.
6. **Schermate**: rigenerate in questo ramo (Guida e sito, 4 lingue) con l'anteprima e Edge; da riguardare a vista prima del merge, in particolare `project-new` (ora si arriva dalla base Laravel del «Pannello rapido»).
7. **Al prossimo rilascio**: togliere le note «in arrivo nel prossimo rilascio» (README ×4, `site/strings/*.json` `f4Text` e `t1Text`) e trasformare «Non ancora rilasciato» del CHANGELOG nella voce del rilascio, nello stesso commit che alza la versione.

---

## 4. Proposte del piano `v7` non applicate

Non sono rilievi: il piano ([`v7/05`](../v7/05-PIANO-ESECUTIVO-V7.md), §1, e [`v7/02`](../v7/02-INCONGRUENZE-DOCUMENTAZIONE.md), §5) suggeriva regole per non ricadere negli stessi schemi. Le correzioni le hanno fatte caso per caso:

| Proposta | Stato |
|---|---|
| Regola ESLint `no-restricted-syntax` contro `.then(…, () => undefined)` e `void` su promesse di comandi | non presente in `eslint.config.mjs`; i casi di `BUG-V7-22..32` sono corretti uno per uno con `useLoad`/`useAction` |
| `os::jobs::kill_and_reap` e `clippy::disallowed-methods` su `Child::kill` | non presenti; `BUG-V7-14` e `-20` raccolgono i processi con `wait()` sul posto o su un thread |
| Prova che ogni pagina di `App.tsx` abbia una voce in `scripts/guide-shots/make.mjs` | non scritta; in questo ramo `make.mjs` ha avuto la pagina dei modelli a mano |

---

## 5. Prove (23/10/2026, ramo `docs/v7-consistency`)

| Suite | Comando | Esito |
|---|---|---|
| Interfaccia | `npm -w ui test` | **56 file, 325 superati** (un giro con il lint in parallelo: 1 file fallito per carico, verde ripetuto da solo) |
| Lint | `npm run lint` | pulito (`cargo fmt --check`, `clippy -D warnings`, ESLint e `tsc`, Prettier) |
| Sito | `node site/build.mjs --check` | 152 pagine, sitemap e link in ordine |
| Link relativi | controllo dei link e delle ancore dei `.md` | nessun link rotto fuori da `docs/reports/` (i report sono fotografie con i percorsi di allora) |
| README | sezioni `##` | 14 in ognuna delle 4 lingue, nello stesso ordine |
| Stringhe del sito | chiavi di `site/strings/*.json` | 126, uguali nelle 4 lingue |
| Rust | `cargo test` | non rieseguito: l'ondata 4 non tocca il codice Rust né il catalogo i18n |
