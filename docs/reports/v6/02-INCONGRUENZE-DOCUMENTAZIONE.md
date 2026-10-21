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

## 2. Rilievi aperti

**Nessuno.** Le 16 incongruenze `DOC-V6-01..16` sono state corrette il 21/10/2026 nel ramo `docs/v6-consistency`, una per commit, insieme ai testi che descrivono i comportamenti cambiati dalle correzioni (token solo agli agenti, fiducia al primo avvio di un agente, niente stato «API key» da variabile d'ambiente per Claude Code e Codex). Il testo completo è nel commit `4cfc6aa`.

Anche le promesse che il codice non manteneva in casi limite (`&` e apici nel percorso del kit, la stessa cartella su un altro PC, backup e ripristino dei database, «Remove drive», «Annulla» durante un download, nomi di progetto con spazi) sono mantenute: i rilievi del documento [01](./01-BUG-SICUREZZA-PORTABILITA.md) che le riguardavano sono chiusi.

---

## 3. Strategia contro la deriva (proposte, non ancora fatte)

1. **Una fonte per l'elenco dei comandi.** Generare il blocco «Command line» dei README da `HELP` (come `guide-docs.cjs` genera la Guida), con `--check` in CI, così README e aiuto non si separano più.
2. **Host di rete generati.** Uno script che estrae gli host da `components.json` e dagli `https://` di `tauri/src/install/**` e confronta con PRIVACY: una prova fallisce se un host nuovo manca.
3. **Niente numeri che invecchiano.** Togliere i conteggi di righe da ARCHITECTURE; tenere solo quelli verificati da una prova (componenti, basi, capitoli).
4. **Prova «nessun italiano nelle lingue straniere».** In `guide-docs.cjs --check`, cercare parole italiane frequenti (`caratteri`, `progetto`, `linea`) in GUIDE.md.
5. **Nomi di prove citati.** Una prova che controlla che ogni nome di prova citato in TESTING*.md esista in `test/`.
