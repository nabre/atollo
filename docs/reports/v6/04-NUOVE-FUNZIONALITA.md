# 04 — Nuove Funzionalità Semplici e di Impatto (`v6`)

> Criteri: (1) **semplici**, cioè costruite su pezzi che il kit ha già; (2) **di impatto** per la fase alpha, che chiede persone che provino Atollo su PC diversi e segnalino cosa non va ([`README.md`](../../../README.md), «Still to do»); (3) **coerenti** con i principi del kit: portabile, niente servizi di sistema, niente telemetria nascosta.
> Prima di proporle ho controllato nel codice che non esistessero già (nessuna corrispondenza per diagnostica esportabile, segnalazione precompilata, QR, backup prima del ripristino, palette dei comandi, preset MCP).

---

## 1. Tabella

| # | Funzionalità | Impatto | Sforzo | Si appoggia a |
|---|---|:---:|:---:|---|
| **N-1** | **Esporta diagnostica** (zip) | Alto | S | `data/logs`, `Atollo env`, `check`, `os::volume` |
| **N-2** | **Segnala un problema** con issue GitHub precompilata | Alto | S | N-1, `api.github.com` già usato |
| **N-3** | **Controllo del disco del kit** («Salute del kit») | Alto | M | `os::volume` (FS, `noexec`, `max_file_size`), `PORT-V6-01` |
| **N-4** | **Backup automatico prima di un ripristino** e prima di cambiare versione del DB | Alto | S | `db/services/backups.rs` |
| **N-5** | **Verifica dell'integrità dei componenti** | Medio-alto | M | hash già noti al download, `check` |
| **N-6** | **QR del sito in rete locale** | Medio | S | modalità LAN già presente |
| **N-7** | **Server MCP pronti** (un clic) | Medio-alto | S | `data/agents/mcp.json`, sync verso tutti gli agenti |
| **N-8** | **Palette dei comandi** (Ctrl+Shift+P) | Medio | M | Ctrl+P dei progetti già esistente |
| **N-9** | **Riepilogo degli aggiornamenti** dei componenti | Medio | S | «Aggiorna tutto», catalogo |
| **N-10** | **Salva progetto come base** del wizard | Medio | M | 63 basi in `frameworks.json`, wizard |
| **N-11** | **Copia stringa di connessione / `.env`** per DB e siti | Medio | S | credenziali per progetto già generate |
| **N-12** | **Esporta / importa il kit** (backup guidato di `projects` + `data` senza `apps`/`cache`/`models`) | Medio | M | la regola di backup del README |

S = fino a 2 giorni, M = 3-5 giorni.

---

## 2. Dettaglio

### N-1 — Esporta diagnostica
- **Cosa**: un pulsante in Impostazioni › Il kit e un comando `Atollo.exe diagnostics` che crea `atollo-diagnostica-<data>.zip` con: versione e target, `Atollo env` **con i segreti mascherati**, esito di `Atollo check`, componenti e versioni, file system del kit (FAT/exFAT/NTFS/APFS/ext4, `noexec`), ultimi N KB di `app.log` ed `editor.log`, porte del registro.
- **Perché**: il README chiede agli alpha tester di segnalare cosa va storto; oggi devono raccogliere a mano le informazioni.
- **Attenzione**: elenco di variabili da mascherare con una prova che fallisce se un valore di `data/app/secrets/*` compare nello zip. Richiede la rotazione del log di `BUG-V6-17`.

### N-2 — Segnala un problema
- **Cosa**: apre `https://github.com/nabre/atollo/issues/new?template=bug.yml&title=…&body=…` con sistema, versione e i passi da compilare; lo zip di N-1 si allega a mano (GitHub non accetta allegati via URL). Più un modello `.github/ISSUE_TEMPLATE/bug.yml`.
- **Perché**: abbassa il costo di una segnalazione a un clic. Nessun dato parte senza che l'utente prema «Invia» su GitHub.

### N-3 — Salute del kit
- **Cosa**: una sezione (e `Atollo.exe check --kit`) che segnala, con la correzione:
  - file system FAT32 → limite di 4 GB per file (modelli GGUF, dump grandi), nessun collegamento;
  - unità montata `noexec` (Linux) → i programmi non partono;
  - **file con DACL di un altro account** (`PORT-V6-01`) → «questi file sono di un altro utente: ripristina i permessi» con un pulsante;
  - spazio libero sotto una soglia; percorso con caratteri a rischio (`&`, apici tipografici, non ASCII) per Maven/Gradle/PowerShell;
  - librerie di sistema mancanti su Linux (`libaio`, `libnuma`, `libvulkan1`).
- **Perché**: trasforma rilievi di portabilità in un messaggio chiaro al primo avvio su un PC nuovo, invece di un errore oscuro più tardi.

### N-4 — Backup prima di un ripristino
- **Cosa**: ogni ripristino crea prima una copia `prima-del-ripristino-<stamp>` dello stato attuale; lo stesso prima di cambiare la versione di un server DB. Conservazione: le ultime 3.
- **Perché**: rende innocui `BUG-V6-01`, `BUG-V6-04` e `BUG-V6-06` anche prima della loro correzione, e protegge da un ripristino del file sbagliato.

### N-5 — Verifica dell'integrità dei componenti
- **Cosa**: alla fine di un'installazione salvare un manifesto (percorso, dimensione, hash veloce) in `data/apps/<target>/<id>/.manifest`; «Verifica» confronta e propone di reinstallare i componenti danneggiati.
- **Perché**: le chiavette USB si danneggiano; oggi `check` vede solo se il programma risponde, non se un file di una libreria è corrotto.

### N-6 — QR del sito in rete locale
- **Cosa**: con la rete locale permessa, accanto all'indirizzo `http://192.168.x.y:porta` del progetto un QR da inquadrare col telefono. Generazione locale (libreria QR in Rust o TS, nessun servizio esterno).
- **Perché**: provare un sito sul telefono è il motivo principale per attivare la LAN; oggi l'indirizzo va ricopiato a mano.

### N-7 — Server MCP pronti
- **Cosa**: in Agenti › Server MCP un elenco di server comuni (filesystem limitato al progetto, GitHub con il token dell'account del kit, Playwright, fetch, il database del progetto) con «Aggiungi»: il kit scrive `data/agents/mcp.json` e la sync li porta a tutti gli agenti (e a Pi). Le dipendenze arrivano da sole (`npx` → Node, `uvx` → uv), come già accade.
- **Perché**: oggi si modifica il JSON a mano; è la cosa che più differenzia «agenti configurati una volta per tutti».

### N-8 — Palette dei comandi
- **Cosa**: Ctrl+Shift+P apre un elenco filtrabile delle azioni (avvia/ferma progetto, apri editor, avvia agente nel progetto, avvia/ferma DB, apri pagina, «Ferma tutto»). Riusa il componente di Ctrl+P.
- **Perché**: chi viene da VSCodium la cerca subito; l'aspetto del workbench (D-142) la fa aspettare.

### N-9 — Riepilogo degli aggiornamenti
- **Cosa**: un badge sulla pagina Strumenti con il numero di componenti che hanno una versione nuova (controllo al massimo una volta al giorno, rispettando le impostazioni di privacy) e un elenco «cosa cambia» con il collegamento alle note di rilascio.
- **Perché**: oggi l'aggiornamento è «Aggiorna tutto» alla cieca.

### N-10 — Salva progetto come base
- **Cosa**: dal menu del progetto, «Salva come base»: copia i file scelti (rispettando `.gitignore`) in `data/frameworks/mine/<nome>` e la base compare nel wizard sotto «Le mie basi».
- **Perché**: riusare la propria struttura (Laravel con i pacchetti di sempre, un monorepo) è comune e oggi richiede un clone a mano.

### N-11 — Copia stringa di connessione
- **Cosa**: nella sezione Database del progetto, pulsanti «Copia URL» (`mysql://utente:password@127.0.0.1:porta/db`, `mongodb://…`), «Copia blocco .env» e «Scrivi in .env» (con conferma e senza toccare le altre righe).
- **Perché**: è il passo successivo naturale a «database, utente e password per ogni progetto».

### N-12 — Esporta / importa il kit
- **Cosa**: una procedura guidata che copia `projects\` e `data\` senza `apps\`, `cache\` e `models\` in una cartella o in uno zip, con i database fermati prima (dump coerente), e l'inverso. Sulla destinazione i componenti si riscaricano.
- **Perché**: il README descrive la regola del backup a parole; farla fare all'app evita copie di un MySQL acceso.

---

## 3. Ordine consigliato

1. **N-1 + N-2** insieme: servono subito agli alpha tester e aiutano a trovare i bug dei sistemi non ancora provati (issue #5, #6).
2. **N-4**: protezione dei dati a basso costo, da fare insieme all'ondata 1 di correzioni.
3. **N-3**: si scrive insieme alle correzioni di portabilità (`PORT-V6-01`, `BUG-V6-12`, `BUG-V6-31`).
4. **N-7, N-11, N-6**: piccoli e visibili.
5. **N-5, N-8, N-9, N-10, N-12**: dopo la parte LLM/Pi.
