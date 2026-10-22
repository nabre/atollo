# Atollo — Valutazione Tecnica Integrale (Revisione v7)

> **Data revisione**: 22 ottobre 2026 (`HEAD 1f84370`, versione `0.5.1`; unico rilascio pubblico `app-v0.5.1`) · **Baseline precedente**: `v6` (`6ecd492`, 21 ottobre 2026; rilievi chiusi in `main`)
> **Ambito**: backend Rust/Tauri 2 ([`tauri/src/`](../../../tauri/src), 62.983 righe), frontend React 19/TypeScript ([`ui/src/`](../../../ui/src), 44.494 righe), Guida dell'app, sito ([`site/`](../../../site)), script, prove ([`test/`](../../../test)), catalogo (39 componenti, 63 basi + 18 aggiunte, 5 modelli), documentazione (README in 4 lingue, `docs/app/*`, informativa, `MARKETING.md`). Novità rispetto alla `v6`: modelli GGUF locali (D-151), Pi come quarto agente (D-152), terminale dell'app su ogni sistema (D-148, D-149); la dimensione «documentazione» ha coperto anche la diagnostica (D-153, `eb3cb57`, entrata durante la revisione). Il merge `c256fb6` (D-154, copie di sicurezza dei database) è successivo e non è coperto.
>
> **Metodo**: revisione in sola lettura, divisa in 8 dimensioni: installatori e modelli (`rust-install`), progetti/db/server (`rust-projects`), core/editor/os (`rust-core`), interfaccia (`ui`), contratto IPC e i18n (`ipc-i18n`), documentazione contro codice (`docs`), struttura e duplicati (`structure`), progetto della Guida neutra (`guide-design`). Sulle prime sei un **verificatore avversario** ha riletto ogni rilievo nel codice cercando di confutarlo: **2 rilievi di `rust-projects` sono stati confutati** e tolti; per altri la verifica ha abbassato la gravità o ristretto lo scenario (lo dice la riga «Verifica» nel documento 01). Le dimensioni `structure` e `guide-design` sono proposte di progetto, non verificate allo stesso modo. Prove eseguite il 22/10/2026 su Windows 11 x64.

---

## 1. Indice

| # | Documento | Contenuto |
|---|---|---|
| **00** | questo file | sintesi, stato delle prove, numeri dei rilievi |
| **01** | [`01-BUG-SICUREZZA-PORTABILITA.md`](./01-BUG-SICUREZZA-PORTABILITA.md) | i 39 rilievi `BUG-V7-xx`, nei quattro gruppi d'esecuzione A-D; osservazione `TEST-V7-01`; punti controllati senza rilievi |
| **02** | [`02-INCONGRUENZE-DOCUMENTAZIONE.md`](./02-INCONGRUENZE-DOCUMENTAZIONE.md) | le 13 incongruenze `DOC-V7-xx`; la decisione sulle funzioni non ancora rilasciate (`DOC-V7-02`) |
| **03** | [`03-STRUTTURA-DUPLICATI-ALIAS.md`](./03-STRUTTURA-DUPLICATI-ALIAS.md) | i 22 rilievi `STR-V7-xx`; struttura di destinazione di `ui/src`; alias `@/`, `@guide`, `@test`; riesportazioni Rust |
| **04** | [`04-GUIDA-NEUTRA-E-SITO.md`](./04-GUIDA-NEUTRA-E-SITO.md) | i 12 rilievi `GUI-V7-xx`; progetto della Guida in `guide/` con due renderer (app e sito), **senza più Markdown**; D-164 |
| **05** | [`05-PIANO-ESECUTIVO-V7.md`](./05-PIANO-ESECUTIVO-V7.md) | quattro ondate (bug, struttura, Guida e sito, documenti), regole per non ricadere negli stessi schemi, decisioni da prendere |

---

## 2. Stato delle prove (22/10/2026, `HEAD 1f84370`)

| Suite | Comando | Esito |
|---|---|---|
| Rust | `cargo test` | **938 superati, 0 falliti, 7 ignorati** |
| Interfaccia | `npm -w ui test` | **42 file, 272 superati** (vedi `TEST-V7-01` qui sotto) |
| Lint interfaccia | `npm -w ui run lint` | pulito (ESLint e `tsc --noEmit` su `test/ui`) |
| Documenti della Guida | `node scripts/guide-docs.cjs --check` | GUIDE.md e GUIDE.it.md aggiornati |
| Contratto IPC | script di confronto UI ↔ Rust | 147 comandi registrati, tutti chiamati dall'interfaccia e viceversa; argomenti allineati; eventi (`editor-status`, `framework`, `github`, `setup`, `run`, `close-requested`, `open-project`, `llm-pull`, `pty-*`) allineati tra `emit_to` e `subscribe`; 53 tipi TS con la stessa forma della struct Rust omonima |
| i18n | `t()` ↔ `en/fr/de.json`, catalogo Rust, `tr()` della Guida, stringhe del sito | nessuna chiave mancante; 423 chiavi Rust tutte usate; 2042 `tr()` della Guida senza testo nella lingua sbagliata; il sito ha 118 chiavi uguali nelle 4 lingue. Buchi: `BUG-V7-36` (prova Rust), `BUG-V7-12`, `-35`, `-37`, `-38` (testi fuori dal catalogo) |
| Documentazione | link relativi e ancore, conteggi, CLI | nessun link rotto; conteggi corretti (39 componenti, 9 capitoli, 10 pagine); aiuto della CLI uguale ai README |

**`TEST-V7-01`** (osservazione, non un rilievo): il primo giro di `npm -w ui test`, con la macchina molto carica, ha dato 15 errori «Failed to start forks worker» / «Timeout waiting for worker». Sono timeout del pool di Vitest sotto carico, non prove fallite: il giro successivo è verde (42 file, 272 superati). Vitest segnala che jsdom è stato creato 27 volte (41% del tempo); `pool: "vmThreads"` è il rimedio se succede anche in CI ([05](./05-PIANO-ESECUTIVO-V7.md), decisione 7).

---

## 3. Sintesi

**Lo stato generale è buono e migliore della `v6`.** Nessun rilievo di gravità ALTA, nessuna perdita di dati: gli schemi della `v6` (cancellare prima di avere il nuovo, file illeggibile riscritto vuoto) non si ripresentano. Il contratto IPC e le traduzioni sono allineati, i link dei documenti tengono, le prove sono tutte verdi. Le parti più nuove (modelli locali, Pi, terminale dell'app) raccolgono la maggior parte dei rilievi del codice.

| Schema | Rilievi |
|---|---|
| La ripresa di un download si fida di una lettura nuova del repository invece che del manifest | `BUG-V7-01`, `-04`, `-05` |
| Lock deboli o con chiavi diverse per la stessa cosa | `BUG-V7-02`, `-03`, `-15` |
| Processi Unix mai raccolti, regole dei percorsi per sistema ignorate | `BUG-V7-14`, `-17`, `-18`, `-19`, `-20`, `-21` |
| Interfaccia: errori inghiottiti e letture concorrenti senza sequenza | `BUG-V7-22`, `-23`, `-24`, `-25`, `-27`, `-28`, `-30`, `-31`, `-32` |
| Valori esterni scritti in una configurazione senza escape | `BUG-V7-09` (Apache), `BUG-V7-10` (modalità LAN) |
| Testi rivolti all'utente fuori dal catalogo | `BUG-V7-12`, `-35`, `-37`, `-38`, e la prova che non li vede (`-36`) |
| Documenti con numeri e elenchi scritti a mano | `DOC-V7-01`, `-04`, `-06`, `-07`, `-10`, `-13` |
| Documenti avanti rispetto al rilascio | `DOC-V7-02` (decisione per il proprietario) |

### 3.1 Rilievi per famiglia e gravità

| Famiglia | ALTA | MEDIA | BASSA | Totale |
|---|:---:|:---:|:---:|:---:|
| Bug, sicurezza, portabilità (`BUG-V7-*`) | 0 | 5 | 34 | **39** |
| Documentazione (`DOC-V7-*`) | 0 | 3 | 10 | **13** |
| Struttura e duplicati (`STR-V7-*`) | 0 | 7 | 15 | **22** |
| Guida neutra e sito (`GUI-V7-*`) | 0 | 6 | 6 | **12** |
| **Totale** | **0** | **21** | **65** | **86** |

Più l'osservazione `TEST-V7-01`. Dei 12 `GUI-V7-*`, due (`GUI-V7-04`, `-05`, MEDIA e BASSA) sono **superati** dalla decisione di togliere il renderer Markdown. Dei 39 `BUG-V7-*`, per categoria: 19 bug, 7 UX, 5 i18n, 3 sicurezza, 3 portabilità, 1 prove, 1 commento superato nel codice (`BUG-V7-34`).

I cinque MEDIA del codice:

1. **`BUG-V7-01`** — «Riprendi» di un modello Hugging Face unisce i byte del commit vecchio a quelli del nuovo: diversi GB scaricati per un checksum sbagliato, o un manifest che dichiara il file sbagliato.
2. **`BUG-V7-09`** — su macOS/Linux un nome di cartella con `"` e a capo, da un repository clonato, inietta direttive nell'`httpd.conf` del progetto o di un pool condiviso.
3. **`BUG-V7-15`** — durante un avvio lento dell'editor la finestra si blocca fino a ~90 s: comandi sincroni sul thread principale aspettano il mutex del server.
4. **`BUG-V7-23`** — «Salva e apri» su un terminale nuovo smonta l'editor a metà azione e l'errore di avvio si perde.
5. **`BUG-V7-25`** — i pulsanti «Ferma» del pannello Attività inghiottono ogni errore.

### 3.2 Richieste di questa revisione

- **Struttura**: i file più grandi di Rust (`projects/run.rs` 1256 righe, `env.rs` 1289) e dell'interfaccia (`workbench/controls.tsx` 878, 20 file sciolti nella radice di `ui/src`) si dividono senza cambiare comportamento, con `pub use` in Rust e gli alias `@/`, `@guide`, `@test` in TypeScript (circa 115 relativi profondi in meno). Documento [03](./03-STRUTTURA-DUPLICATI-ALIAS.md).
- **Guida neutra e sito**: il contenuto si sposta in `guide/` alla radice, con un contratto di widget e due renderer, l'app e il sito (`/guide/<argomento>/` in inglese, `/it/guide/…`, `/fr/guide/…`, `/de/guide/…`: 148 pagine con ricerca, hreflang e sitemap). **Per decisione del proprietario il renderer Markdown si toglie**: spariscono `scripts/guide-docs.cjs`, `docs/app/GUIDE*.md`, la prova `docs.test.ts` e il passo `--check` della CI; i link puntano a https://nabre.github.io/atollo/guide/. Nuova decisione D-164. Documento [04](./04-GUIDA-NEUTRA-E-SITO.md).
- **Piano**: quattro ondate su quattro rami (`fix/v7-bugs` con i gruppi A-D in worktree paralleli, `refactor/v7-structure`, `feature/v7-guide-site`, `docs/v7-consistency`), un commit per rilievo, merge solo con le prove verdi. Documento [05](./05-PIANO-ESECUTIVO-V7.md).

### 3.3 Decisioni per il proprietario

1. **`DOC-V7-02`**: segnare Pi, modelli locali e diagnostica come «in arrivo nel prossimo rilascio» in README e sito finché non esce un rilascio (proposta), oppure tagliare il rilascio prima di pubblicare il sito.
2. **Schermate della Guida** in `guide/shots/` (facoltativo, dopo l'ondata 3).
3. Il numero della decisione sulla Guida è **D-164**, non D-153: D-153 (diagnostica) e D-154 (copie di sicurezza) sono già in `main`. Il report proponeva D-155; D-155..D-163 sono poi andati alle funzioni entrate prima della Guida (correzione dell'ondata 4).
