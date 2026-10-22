# Atollo — Valutazione Tecnica Integrale (Revisione v6)

> **Data revisione**: 21 ottobre 2026 (`HEAD 6ecd492`, rilascio `app-v0.5.0` alpha) · **Baseline precedente**: `v5` (`a8aad6c`, 20 ottobre 2026)
> **Ambito**: backend Rust/Tauri 2 ([`tauri/src/`](../../../tauri/src), 55.929 righe), frontend React 19/TypeScript ([`ui/src/`](../../../ui/src), 36.617 righe), runtime, script, catalogo (35 componenti, 63 basi), documentazione (README in 4 lingue, `docs/app/*`, Guida dell'app, sito, informativa).
>
> **Metodo**: revisione in sola lettura, divisa per sottosistema (installatori/rete/setup/versioni; progetti/server/database; agenti/editor/os/comandi; interfaccia; contratto IPC e i18n; documentazione contro codice) più una ricerca esterna su Pi e llama.cpp. I rilievi di gravità ALTA sono stati riletti nel codice prima di entrare nel report. Prove eseguite il 21/10/2026 su Windows 11 x64.

> **Stato (21/10/2026, dopo le correzioni)**: tutti i 49 rilievi del codice e le 16 incongruenze della documentazione sono **chiusi** in `main`, un commit per rilievo, e tolti dai documenti 01 e 02. Prove: `cargo test` **855 superati**, 0 falliti, 7 ignorati (due giri); interfaccia **37 file, 230 superati**; `npm run lint` pulito. Le tabelle qui sotto restano come fotografia della revisione; cosa resta da verificare a mano è nel documento [01](./01-BUG-SICUREZZA-PORTABILITA.md), §1.1.

---

## 1. Indice

| # | Documento | Contenuto |
|---|---|---|
| **00** | questo file | sintesi, stato delle prove, numeri dei rilievi |
| **01** | [`01-BUG-SICUREZZA-PORTABILITA.md`](./01-BUG-SICUREZZA-PORTABILITA.md) | rilievi `BUG/SEC/PORT/UX/TEST-V6-xx` ancora aperti (oggi nessuno), rami che li hanno chiusi, verifiche manuali |
| **02** | [`02-INCONGRUENZE-DOCUMENTAZIONE.md`](./02-INCONGRUENZE-DOCUMENTAZIONE.md) | incongruenze `DOC-V6-xx` ancora aperte (oggi nessuna), strategia contro la deriva |
| **03** | [`03-LLM-LOCALI-GGUF-E-PI.md`](./03-LLM-LOCALI-GGUF-E-PI.md) | progetto: modelli GGUF locali con llama.cpp e Pi come quarto agente; **fatto** (F0-F4, D-151, D-152; esito nel §7) |
| **04** | [`04-NUOVE-FUNZIONALITA.md`](./04-NUOVE-FUNZIONALITA.md) | 12 funzionalità semplici e di impatto, con stima di sforzo |
| **05** | [`05-PIANO-ESECUTIVO-V6.md`](./05-PIANO-ESECUTIVO-V6.md) | ondate di correzione, regole di codice per non ricadere negli stessi errori, decisioni da prendere |

---

## 2. Stato delle prove (21/10/2026)

| Suite | Comando | Esito |
|---|---|---|
| Rust | `cargo test` | **794 superati, 1 fallito, 7 ignorati** su 802. Il fallito, `projects::run::tests::ports_are_kept_and_locked`, passa 3 volte su 3 da solo: è instabile in parallelo (`TEST-V6-01`). |
| Interfaccia | `npm -w ui test` | **34 file, 219 superati** |
| Lint interfaccia | `npm -w ui run lint` | pulito |
| Contratto IPC | script di confronto UI ↔ Rust | 123 comandi, argomenti, 72 tipi di ritorno ed 8 eventi allineati; 2 tipi TS imprecisi senza effetti |
| i18n | chiavi `t()` ↔ `en/fr/de.json`, `tr()` ↔ `catalog.json` | 919/919 chiavi in ogni lingua, nessuna mancante; la prova Rust non sorveglia `cx.t`, `label()` e `HELP` |
| Documentazione | link relativi, `guide-docs.cjs --check`, matrice rigenerata | 633 link validi, Guida e matrice aggiornate |

Nota: `npx vitest run` lanciato dalla radice raccoglie i file di prova sotto `target/debug/data/...` e `.claude/worktrees/` e dà 45 file falliti. Lo script ufficiale (`npm -w ui test`) è corretto; vedi `TEST-V6-02`.

---

## 3. Sintesi

**Lo stato generale è buono.** Il contratto IPC e le traduzioni sono allineati, la documentazione segue il codice con poche eccezioni, l'estrazione degli archivi e le verifiche dei checksum reggono. I rilievi `v5` sono stati chiusi tra il 20 e il 21/10.

**Il tema di questa revisione è la perdita di dati sui percorsi d'errore.** Quasi tutti i rilievi gravi seguono lo stesso schema: si cancella o si sovrascrive *prima* di sapere che il nuovo stato esiste ed è valido.

| Schema | Rilievi |
|---|---|
| Si cancella il vecchio prima di avere il nuovo | `BUG-V6-02` (ambienti conda), `BUG-V6-09` (dati MySQL), `BUG-V6-06` (archivio MongoDB con `--drop`) |
| Un file illeggibile diventa «vuoto» e viene riscritto | `SEC-V6-01` (password dei database, profili dei pool), `PORT-V6-01` (chiave Gemini) |
| Una sostituzione di testo rompe i file | `BUG-V6-03` (kit alla radice di un'unità) |
| Una cancellazione si propaga senza copia | `BUG-V6-16` (istruzioni degli agenti) |
| Un ripristino si ferma a metà | `BUG-V6-01` (`DEFINER`), `BUG-V6-04` (BLOB), `BUG-V6-05` (pipe rotta) |

### 3.1 Rilievi per gravità

| Categoria | ALTA | MEDIA | BASSA | Totale |
|---|:---:|:---:|:---:|:---:|
| Bug (`BUG-V6-*`) | 3 | 16 | 14 | **33** |
| Sicurezza (`SEC-V6-*`) | 0 | 2 | 2 | **4** |
| Portabilità (`PORT-V6-*`) | 1 | 1 | 4 | **6** |
| UX interfaccia (`UX-V6-*`) | 0 | 2 | 1 | **3** |
| Prove (`TEST-V6-*`) | 0 | 0 | 3 | **3** |
| Documentazione (`DOC-V6-*`) | 0 | 6 | 10 | **16** |

I quattro ALTA:

1. **`BUG-V6-01`** — il ripristino MySQL di un backup del kit fallisce a metà se il database ha viste, trigger, routine o eventi (`DEFINER=root`, ripristino come utente del progetto).
2. **`BUG-V6-02`** — aggiornare R, Ruby, C/C++ o i componenti conda cancella l'ambiente funzionante prima che il nuovo sia creato; senza rete resta niente.
3. **`BUG-V6-03`** — con il kit alla radice di una chiavetta (`E:\`) che cambia lettera, la rilocazione scrive `F:\\data` e rompe i JSON degli agenti e dell'editor.
4. **`PORT-V6-01`** — i file protetti con DACL «solo proprietario» su NTFS diventano illeggibili su un altro PC o account Windows (editor che non parte, chiave Gemini «sparita»). Da confermare con due account.

### 3.2 Richieste di questa revisione

- **Modelli GGUF locali e Pi**: fattibile senza forzare l'architettura. Pi 1.3 pubblica binari per tutti e sei i target con `SHA256SUMS`, legge `AGENTS.md` e le skill, ha MCP e un provider llama.cpp integrati. llama.cpp in modalità router serve una cartella di `.gguf` con API OpenAI. Il progetto in 5 fasi è nel documento [03](./03-LLM-LOCALI-GGUF-E-PI.md).
- **Nuove funzionalità**: le più utili per l'alpha sono «Esporta diagnostica» e «Segnala un problema» precompilato, il controllo del disco del kit (file system, permessi, spazio) e il backup automatico prima di un ripristino. Elenco nel documento [04](./04-NUOVE-FUNZIONALITA.md).
