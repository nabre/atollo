# 02 — Incongruenze tra Documentazione e Applicazione (`v7`)

> **Riferimento**: `HEAD 1f84370`; la dimensione «documentazione» ha coperto anche `eb3cb57` (diagnostica, D-153), entrato in `main` durante la revisione. Documenti confrontati: README in 4 lingue, `docs/PRIVACY*.md`, `docs/app/{GUIDE,CATALOG,ARCHITECTURE,TESTING}.md` e gemelli `.it.md`, testa di `CHANGELOG.md` e `DECISIONS.md` (D-135, D-151..153), `docs/brand/MARKETING.md`, Guida dell'app (`ui/src/guide/**`), sito (`site/template.html`, `site/strings/*.json`), aiuto della riga di comando.

---

## 1. Cosa torna

- **Conteggi**: 39 componenti, 63 basi + 18 aggiunte, 9 capitoli della Guida, 10 pagine, 9 componenti `x64Only`, 5 modelli nel catalogo.
- **Collegamenti**: nessun link relativo o ancora rotto in README, `docs/*.md` e `docs/app/*.md`.
- **File generati**: `node scripts/guide-docs.cjs --check` dice GUIDE.md e GUIDE.it.md aggiornati (lo script viene tolto nell'ondata 3, vedi [04](./04-GUIDA-NEUTRA-E-SITO.md)).
- **Pi e llama.cpp**: sorgenti d'installazione dei documenti uguali a `install/agents.rs` e `install/llama.rs`.
- **Riga di comando**: l'aiuto (`main.rs:495`) e il blocco dei README coincidono, compresi `models`, `llm` e `diagnostics`.
- **Traduzioni**: in ARCHITECTURE, CATALOG e TESTING i nomi di codice EN e IT coincidono (cambiano solo i segnaposto tradotti); struttura dei 4 README e chiavi del sito allineate.
- **Marchio**: nessun nome di prima fuori dai report; l'unico riscontro in DECISIONS è `w64devkit`, un prodotto di terzi.

---

## 2. Rilievi aperti

| ID | Grav. | Documento | Codice | Il documento dice | Il codice fa | Da correggere |
|---|:---:|---|---|---|---|---|
| `DOC-V7-01` | MEDIA | [`site/template.html:117`](../../../site/template.html#L117) | [`tauri/src/agents/mod.rs:44`](../../../tauri/src/agents/mod.rs#L44) | «3 AI agents with a single configuration» (4 lingue), con `data-count="3"` fisso; la tessera poco sotto dice «Four AI agents» | `IDS: [&str; 4]` = claude, codex, antigravity, pi | `site/build.mjs`: `agentCount` calcolato dal catalogo (come `componentCount`, riga 198) e `{{agentCount}}` nel modello |
| `DOC-V7-02` | MEDIA | [`README.md:105`](../../../README.md#L105) (×4), sito | `app-v0.5.1` (21/10), `tauri/Cargo.toml:3` | Pi, «Local models» e la diagnostica presentati come presenti; «How to use» punta a `Atollo-0.5.1-win32-x64.exe` | il commit di Pi (`7308ca3`, 22/10) non è antenato di `app-v0.5.1`; la versione è ancora 0.5.1 | **decisione del proprietario**, §3 |
| `DOC-V7-03` | MEDIA | [`ui/src/guide/topics/agents.tsx:157`](../../../ui/src/guide/topics/agents.tsx#L157) (4 lingue), `GUIDE.md:1233`; [`ui/src/components/model.ts:47`](../../../ui/src/components/model.ts#L47) e `locales/{en,fr,de}.json:899` | [`tauri/src/setup/catalog.rs:168`](../../../tauri/src/setup/catalog.rs#L168), `components.json` | «Git, che usano tutti, è già tra i componenti sempre installati»; suggerimento «Sempre installati: l'editor, la shell del kit, GitHub CLI e Git» | Git non è bloccato (D-81), `def=false`; solo Claude Code e Pi hanno `needs: [git]` | Guida: «Claude Code e Pi usano Git, che il kit installa con loro»; testo di `model.ts:47` e locali; rigenerare GUIDE*.md (finché lo script esiste) |
| `DOC-V7-04` | BASSA | [`ui/src/guide/topics/start.tsx:1019`](../../../ui/src/guide/topics/start.tsx#L1019), `GUIDE.md:240` | [`ui/src/App.tsx:85`](../../../ui/src/App.tsx#L85), [`ui/src/activity.tsx:189`](../../../ui/src/activity.tsx#L189) | barra delle attività senza «Modelli locali»; il numero sull'icona Progetti conta azioni, versioni dei database e pool | la pagina `models` sta tra Agenti e Database; il conteggio include `(llm ? 1 : 0)` | Guida §1.3 (4 lingue): aggiungere «Modelli locali» e «il server dei modelli locali» |
| `DOC-V7-05` | BASSA | [`ui/public/guide/en/agents.webp`](../../../ui/public/guide/en/agents.webp) e tutte le `ui/public/guide/<lang>/*.webp`, `site/shots/<lang>/*.png` | [`ui/src/agents/page.tsx:258`](../../../ui/src/agents/page.tsx#L258), [`scripts/guide-shots/make.mjs`](../../../scripts/guide-shots/make.mjs) | immagini del rilascio 0.5.0 (`91b3821`): 9 icone, «v0.5.0», «Claude Code, Codex and Antigravity share…» | l'app ha Pi e la pagina dei modelli; `make.mjs` non ha la pagina `models`, la Guida §4.4 non ha immagine | `make.mjs`: pagina `models` e dati d'anteprima con Pi; rigenerare Guida e sito nelle 4 lingue |
| `DOC-V7-06` | BASSA | [`README.md:95`](../../../README.md#L95) (×4), [`CATALOG.md:56`](../../app/CATALOG.md#L56) (+ `.it`), [`TESTING.md:305`](../../app/TESTING.md#L305) (+ `.it`) | [`ui/src/components/model.ts:9`](../../../ui/src/components/model.ts#L9) | sette tipi in Strumenti | otto: c'è anche «Visualizzatori di database» (phpMyAdmin, mongo-express, D-147); la Guida lo dice già | aggiungere il tipo nei README, CATALOG e TESTING |
| `DOC-V7-07` | BASSA | [`ARCHITECTURE.md:205`](../../app/ARCHITECTURE.md#L205) (+ `.it`), albero alle righe 67-150, riga 417 | `tauri/src/diagnostics.rs`, `tauri/src/tray.rs`, `install/mod.rs` (961 righe), `editor/pty.rs` (974) | albero senza `diagnostics.rs` (D-153) e `tray.rs` (D-145), dati senza `app\diagnostics\`; «oggi solo `install\mod.rs` supera le 900 righe»; «956» | due moduli in più; anche `editor/pty.rs` supera le 900 righe | aggiungere i due moduli e la cartella; togliere i conteggi esatti di righe (già proposto in v6, §3.3) |
| `DOC-V7-08` | BASSA | [`CATALOG.md:68`](../../app/CATALOG.md#L68) (+ `.it`) | `components.json` (`conpty`) | colonna d'esempio di `targets`: «nessuno oggi» | `conpty` ha `targets: ["win32-x64", "win32-arm64"]` (D-148) | usare ConPTY come esempio |
| `DOC-V7-09` | BASSA | [`README.md:123`](../../../README.md#L123) (×4) | [`docs/PRIVACY.md:84`](../../PRIVACY.md#L84), `tauri/src/agents/sync/pi.rs:142` | `LLAMA_API_KEY` va solo agli agenti, mai a terminali, script o editor | la chiave è scritta anche in `data\home\.pi\agent\models.json` perché `pi` funzioni dai terminali del kit | README (×4): dire che la chiave sta anche nel `models.json` di Pi (solo proprietario) |
| `DOC-V7-10` | BASSA | [`site/strings/en.json:54`](../../../site/strings/en.json#L54) (×4, `a2Text`) | [`ui/src/App.tsx:85`](../../../ui/src/App.tsx#L85) | elenco delle pagine senza i modelli locali | la pagina «Modelli locali» esiste; `MARKETING.md:90` la cita già | aggiungere «local models» dopo «agents» nelle 4 lingue |
| `DOC-V7-11` | BASSA | [`README.md:97`](../../../README.md#L97) (×4) | [`ui/src/settings/about.tsx:59`](../../../ui/src/settings/about.tsx#L59) | «**The kit**, read only» | il gruppo contiene «Esporta diagnostica» e «Segnala un problema» (D-153); lo stesso README alla riga 109 rimanda lì | «Il kit: cartelle, disco, piattaforma e versioni, con Esporta diagnostica e Segnala un problema» |
| `DOC-V7-12` | BASSA | [`docs/brand/MARKETING.md:15`](../../brand/MARKETING.md#L15), riga 235 | `agents/mod.rs:44` | «sincronizzazione di 3 agenti AI» | quattro agenti; lo stesso file dice «4 Agenti» (riga 72) e «Quattro agenti AI» (riga 192) | «4 agenti AI (Claude Code, Codex, Antigravity, Pi)» |
| `DOC-V7-13` | BASSA | [`CATALOG.md:54`](../../app/CATALOG.md#L54) (+ `.it`) | `components.json` | primo avvio di «1845 MB … su ogni sistema» | 1848 MB su Windows: `conpty` (3 MB, `def=true`) c'è solo lì | «1845 MB (1848 su Windows, con ConPTY)» o togliere la cifra esatta |

---

## 3. `DOC-V7-02`: funzioni descritte prima del rilascio

**Situazione.** L'unico rilascio pubblico è `app-v0.5.1` (21/10/2026). Pi (D-152), i modelli locali (D-151) e la diagnostica (D-153) sono in `main` ma in nessun rilascio; `tauri/Cargo.toml` e `ui/package.json` dicono ancora 0.5.1. README e sito li presentano come presenti e mandano a scaricare il rilascio 0.5.1, che non li contiene; la sezione «Still to do» non lo dice. Che la documentazione di `main` sia avanti rispetto al rilascio è normale; il problema è che il link di download promette cose che il file scaricato non ha.

**Correzione proposta.** Finché non esce un rilascio con queste funzioni, segnarle come **«in arrivo nel prossimo rilascio»** nei 4 README e nelle stringhe del sito (una nota breve accanto a Pi, «Modelli locali» e alla diagnostica, e una riga in «Still to do»), e aprire nel `CHANGELOG.md` una voce «Non ancora rilasciato» che le raccolga. Quando il rilascio esce, la nota si toglie nello stesso commit che alza la versione.

**Decisione per il proprietario.** Due strade, da scegliere prima dell'ondata 4:

| Strada | Pro | Contro |
|---|---|---|
| **A. Nota «in arrivo»** nei documenti finché non c'è il rilascio (proposta) | nessun rilascio affrettato; documenti onesti subito | va ricordato di togliere la nota |
| **B. Tagliare il prossimo rilascio** (per esempio `0.6.0`) prima di pubblicare il sito | documenti e download coincidono senza note | richiede di chiudere prima le ondate 1-3, o di rilasciare con i rilievi aperti |

---

## 4. Incongruenze emerse dalla revisione del codice

Non sono errori dei documenti ma promesse che il codice non mantiene in casi limite; si chiudono correggendo il codice (documento [01](./01-BUG-SICUREZZA-PORTABILITA.md)):

| Promessa | Dove | Rilievo |
|---|---|---|
| «ogni file scaricato e controllato» per i modelli | Guida, capitolo Modelli locali; [`PRIVACY.md`](../../PRIVACY.md) | `BUG-V7-01`, `BUG-V7-05` |
| la stessa cartella passa da un PC o account all'altro | README, «Features» | `BUG-V7-07` (token HF illeggibile blocca il catalogo) |
| interfaccia nelle 4 lingue | README | `BUG-V7-12`, `BUG-V7-35`, `BUG-V7-37`, `BUG-V7-38` |
| «LLAMA_API_KEY solo agli agenti» | README | `DOC-V7-09` (si corregge il testo, non il codice) |

---

## 5. Strategia contro la deriva

Le proposte della `v6` (§3) restano valide; tre rilievi di questa revisione le confermano:

1. **Numeri calcolati, non scritti.** `DOC-V7-01` (3 agenti), `DOC-V7-07` (righe) e `DOC-V7-13` (MB) sono cifre fisse invecchiate. Nel sito, ogni numero esce da `build.mjs` e dal catalogo; nei documenti, solo i conteggi che una prova verifica.
2. **Elenchi delle pagine da una fonte.** `DOC-V7-04` e `DOC-V7-10` ripetono a mano l'elenco delle pagine di `App.tsx`. La Guida pubblicata sul sito ([04](./04-GUIDA-NEUTRA-E-SITO.md)) toglie un'altra copia: GUIDE.md sparisce.
3. **Schermate rigenerate a ogni pagina nuova.** `DOC-V7-05`: una prova in `test/ui` che controlli che ogni pagina di `App.tsx` abbia una voce in `scripts/guide-shots/make.mjs`.
