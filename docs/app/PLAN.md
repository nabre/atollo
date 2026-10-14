# Atollo App: piano (fase 0, revisione 3)

Stato: **approvato** (14/10/2026). Fasi 1-6 completate; prossima: fase 7 (componente `app`, workflow di release `app-v*`, documentazione in quattro lingue, release `app-v1.0.0`).
Base letta: `nabre/atollo` al commit `a61a354` (main, 13/10/2026).
Scelte non ovvie: [DECISIONS.md](DECISIONS.md). Domande aperte: §11.

## Indicazioni dell'utente che cambiano il prompt

| Data | Indicazione | Effetto |
| --- | --- | --- |
| 13/10 | 100% portabile, nessuna richiesta di amministratore o di permessi; si usano solo le cartelle accanto all'exe | requisiti 1-3 confermati e resi assoluti |
| 14/10 | Tutta la logica di gestione in Rust, con le chiamate di sistema necessarie | niente script PowerShell, `cmd` o Node per la gestione |
| 14/10 | Il terminale nell'editor è esclusivo del kit: scaricare il terminale migliore e metterci tutto il necessario | componente `pwsh` (§6); console come finestre dell'app con terminale integrato (DECISIONS.md D-44) |
| 14/10 | Tutta la logica nel file portabile; una sola cartella `data` con i file scaricati, lo stato, i contenuti e le impostazioni | struttura nuova: **`Atollo.exe` + `data\` + `projects\`** (§2) |
| 14/10 | `projects\` deve stare nella radice | Q2 chiusa |
| 14/10 | Prodotto **separato**, in un altro repository; `nabre/atollo` è solo un riferimento (linguaggi, agenti, comportamenti) | Q1 e Q3 chiuse; catalogo in [CATALOG.md](CATALOG.md) |
| 14/10 | Nessun modulo PowerShell in più; documenti in italiano | Q5 e Q7 chiuse |

Requisiti del prompt **superati** da queste indicazioni:
- "Riusa, non riscrivere" gli script del kit → tutto in Rust dentro l'exe.
- Requisito 11 "il menu attuale resta" e `setup\Windows.cmd` di ripiego → non applicabile: è un prodotto nuovo, il Atollo resta com'è nel suo repository.
- Modifiche a `nabre/atollo` (componenti nuovi nel suo `components.json`, estensione, `README` in quattro lingue, `checks.yml`) → nessuna: il nuovo repository ha il suo catalogo, la sua CI e la sua documentazione.
- Radice riconosciuta da `setup\components.json` → riconosciuta da `Atollo.exe` stesso; `data\` creata al primo avvio.
- `apps\` e `setup\` nella radice → `apps\` sotto `data\`, `setup\` dentro l'exe; `projects\` resta nella radice.
- Aggiornamento di `setup\` da `main` (`Update-Setup.ps1`) → si aggiorna solo l'exe.
- `data\` condivisa con macOS e Linux → l'app è solo Windows; `data\` è sua.

## 1. Cosa resta valido di quanto ho letto nel kit

Il kit attuale diventa la **specifica di riferimento**: il comportamento si porta in Rust, regola per regola, con riferimento al file d'origine nei commenti.

| Comportamento | Origine | Destinazione Rust |
| --- | --- | --- |
| Ambiente portabile (~90 variabili, PATH del kit) | `setup/windows/_env.cmd` | `env.rs` |
| Rilocazione dopo spostamento o cambio di lettera | `_env.cmd:238-267`, `setup/agents/moved.js` | `relocate.rs` |
| Catalogo componenti (27 + 3 nuovi) | `setup/components.json` | `catalog.rs`, dati incorporati |
| Download, verifica (SHA-256/512, npm, Authenticode), estrazione, scambio cartelle | `_common.ps1`, `_prepare-*.ps1`, `_languages.ps1`, `_workarea.ps1` | `net.rs`, `verify.rs`, `archive.rs`, `install/*.rs` |
| Allineamento di istruzioni, skill, MCP, regole, cartelle attendibili | `setup/agents/sync.js` + 8 moduli | `agents_sync/*.rs` |
| Accesso Antigravity fuori dal Gestore credenziali | `agy-keyring.ps1` | `keyring.rs` |
| Server su 127.0.0.1 per Node | `setup/node/loopback.js` | file incorporato, scritto in `data\kit\` (Node lo richiede come file) |
| Espulsione e altre copie aperte | `Eject.ps1`, `Test-Sessions.ps1` | `sessions.rs` |
| Estensioni consigliate, `@programma`, proposta una volta sola, CodeLLDB | `setup/vscodium/*`, `_vscodium-extensions.ps1` | `editor_ext.rs` |
| Profili del terminale, `claudeCode.environmentVariables` | `setup/extensions/atollo/kit.js:163-220` | `editor_cfg.rs` |
| Regole dei nomi dei progetti, rilevatori | `core/projects.js:37-44`, `core/detectors` | `projects.rs`, `detect/*.rs` |
| Testi it/en/fr/de | `setup/lang/parts/*.json` | cataloghi incorporati |

Bug trovato e da non riportare: rimozione di Typst e TinyTeX mai eseguita (`_prepare-migrate.ps1:128`).

## 2. Struttura su disco

```
Atollo\                         qualsiasi cartella, anche con spazi, ( ) e &
  Atollo.exe                    tutta la logica, i testi, il catalogo, le estensioni del kit
  projects\                      i progetti (contenuti), creata al primo avvio
  data\                          creata al primo avvio; tutto il resto è qui
    apps\win32-x64\              programmi scaricati (o win32-arm64)
      git\ node\ python\ pwsh\ vscodium-server\ claude\ codex\ antigravity\ …
      webview2\                  facoltativo: runtime WebView2 "Fixed Version"
    home\                        profilo portabile: .claude .codex .gemini .gitconfig .ssh AppData\
    agents\                      istruzioni, skill, MCP condivisi dai tre agenti
    cache\  tmp\                 cache dei pacchetti, temporanei (anche quelli del sistema)
    app\                         settings.json, components.txt, versions\, logs\, update\
    kit\                         file che i programmi vogliono su disco (loopback.js, profile.ps1, bashrc), riscritti dall'exe a ogni nuova versione
    webview2\                    dati di WebView2
    vscodium-server\             dati del server dell'editor e token
    … le altre cartelle dati di _env.cmd (npm-global, go, gradle, php, apache…)
```

Regole:
- L'exe riconosce la sua radice dalla propria posizione. Se `data\` manca, la crea (primo avvio). Se la cartella non è scrivibile, finestra di messaggio e uscita.
- Ogni file scritto da app, programmi e agenti finisce in `data\`. Eccezioni ammesse solo quelle del README attuale (tracce di Windows; `gh` senza `--insecure-storage`; Gestore credenziali per Antigravity mentre `agy` è aperto, con persistenza di sola sessione), elencate in `DECISIONS.md`.
- Per fare un backup basta copiare `projects\` e `data\` (meno `data\apps\` e `data\cache\`, che si riscaricano).

## 3. Architettura

```
Atollo.exe (Tauri 2, dentro un Job Object)
 ├─ finestra "main"    interfaccia Svelte, unica con capability e comandi
 ├─ finestre "editor"  http://127.0.0.1:<7000-7099>, nessuna capability
 ├─ finestre "terminal" terminal.html (xterm.js), solo i comandi della propria console
 ├─ motore Rust        tutto il resto (§4)
 └─ figli, tutti nel job:
     codium-server · pwsh / Git Bash / cmd · claude / codex / agy (su pseudo console)
```

Riga di comando dello stesso exe, senza finestra (D-21): `Atollo.exe env`, `prepare --only …`, `sync`, `eject --list`, `agent claude <cartella>`. Serve a profili del terminale, test di integrazione e aggiornamento dell'exe. Esempio: il profilo "Claude Code" del terminale lancia `Atollo.exe agent claude .`, che allinea gli agenti prima e dopo.

Avvio (finestra entro 1 s): radice → istanza singola per kit → ambiente del processo dentro `data\` → Job Object → WebView2 (sistema, fissa in `data\apps\…\webview2\`, o messaggio) → finestra `main` → in background: rilocazione, salvataggio accesso Antigravity in sospeso, allineamento agenti, altre copie del kit aperte, controllo aggiornamento.

## 4. Moduli Rust (`src-tauri\src\`)

| Modulo | Responsabilità |
| --- | --- |
| `main.rs` | avvio, finestre, comandi Tauri, riga di comando, chiusura |
| `kit.rs` | radice, piattaforma, percorsi, primo avvio, validazione di percorsi e nomi |
| `env.rs` | ambiente del kit; PATH = kit + `System32`, `Wbem`, `WindowsPowerShell\v1.0`; impostazioni di rete da `data\app\settings.json` (proxy, certificati, chiave Gemini, lingua, rete locale) |
| `relocate.rs` | radice precedente in `data\app\root.txt`; sostituzione nei file di agenti, editor, Python, uv, `.venv` (regole di `moved.js`) |
| `jobs.rs` | Job Object, avvio con argomenti a vettore, sotto-job annullabili, log in streaming, uscita dal job per browser ed Esplora risorse |
| `net.rs` | HTTPS con certificati di Windows, proxy, 3 tentativi, `.part`, API e redirect di GitHub |
| `verify.rs` | SHA-256, SHA-512, integrità npm, firma Authenticode con editore atteso |
| `archive.rs` | zip, tar.gz, 7z, cab; scambio cartelle con ripristino; "file in uso" |
| `catalog.rs` | catalogo incorporato, `data\app\components.txt`, `needs` a cascata, versioni installate |
| `install/*.rs` | un file per famiglia: `core` (git, node, uv, gh, python), `agents`, `web` (php, apache, VC++), `rust`, `lang` (gli altri), `editor` (server, pwsh) |
| `setup_runner.rs` | ordine dei passi, una preparazione alla volta, eventi, annullamento al termine del passo |
| `agents.rs` | stato (installato, versione, accesso), lancio, uscita |
| `agents_sync/*.rs` | istruzioni, skill (giunzioni NTFS, copia su FAT32/exFAT), MCP per Claude/Codex/Antigravity, regole, cartelle attendibili, skill di progetto |
| `keyring.rs` | accesso Antigravity: `CredEnumerateW`, `CredWriteW` (sessione), `CredDeleteW`; file in `data\antigravity\` |
| `editor.rs` | server web: porta, token, avvio, salute, riavvio, finestre `editor` |
| `editor_cfg.rs` | impostazioni, profili del terminale esclusivi, variabili per Claude Code |
| `editor_ext.rs` | estensioni consigliate (`@programma`, proposta una volta sola, CodeLLDB); nessuna estensione propria (Q4 = a) |
| `terminal.rs` | console del kit: pseudo console (`portable-pty`) e finestre `terminal-<n>` con xterm.js (D-44…D-46) |
| `projects.rs`, `detect/*.rs` | elenco, creazione, clone con `gh`, rilevamento del tipo, dei linguaggi e delle azioni (`npm run dev`, `php -S`, `cargo run`…), `panel.json` |
| `run.rs` | avvio e arresto delle azioni dei progetti: porte 7100-7999 ricordate in `data\app\ports.json`, attesa del server, log, apertura nel browser (D-50) |
| `serve.rs` | server di file per i siti statici, `Atollo.exe serve <cartella> <porta>`, al posto di `static-server.js` (D-51) |
| `stats.rs` | (fase successiva) limiti d'uso degli agenti, come le statistiche del Atollo |
| `sessions.rs` | processi del kit aperti, altre copie del kit, espulsione |
| `self_update.rs` | nuova versione dell'exe: download in `data\app\update\`, SHA-256, scambio alla chiusura |
| `i18n.rs` | lingua e cataloghi incorporati |
| `settings.rs` | `data\app\settings.json` (al posto di `proxy.cmd`) |
| `instance.rs` | istanza singola per kit (mutex + pipe) |
| `webview.rs` | WebView2 di sistema o fisso, messaggio se manca |
| `kit_files.rs` | file incorporati scritti in `data\kit\` |
| `cli.rs`, `commands.rs`, `log.rs` | riga di comando, comandi Tauri, log con segreti mascherati |

Dipendenze previste: `serde`, `serde_json`, `tokio`, `windows`, `anyhow`, `thiserror`, `ureq` + `rustls-platform-verifier`, `sha2`, `base64`, `zip`, `tar`, `flate2`, `sevenz-rust2`, `toml_edit`, `regex`. Motivi in D-22.

## 5. Interfaccia

React + TypeScript `strict` + Tailwind + react-icons (Codicons), con il tema di VSCodium (D-02, D-27). Pagine: Primo avvio, Home, Progetti, Agenti, Componenti, Impostazioni, Rimuovi l'unità. Barra di stato. Tutte le funzioni del menu attuale hanno una pagina (tabella di corrispondenza nella revisione 1, invariata). "Proxy e impostazioni" diventa un modulo nella pagina Impostazioni (niente più `proxy.cmd` da modificare a mano).

## 6. Terminale esclusivo del kit

| Componente | Cosa | Perché |
| --- | --- | --- |
| `pwsh` | PowerShell 7 portabile (zip ufficiale, x64 e ARM64, SHA-256) | shell predefinita dell'editor; PSReadLine incluso; isolabile dal PC |
| (nessuno) | console come finestre dell'app: xterm.js + pseudo console di `portable-pty` (D-44); al posto di Windows Terminal dal 14/10 | console fuori dall'editor (agenti, Terminale); multipiattaforma; nel job. Ripiego `conhost` su Windows 10 prima della 1809 |
| `conpty` (proposto) | `conpty.dll` + `OpenConsole.exe` dal pacchetto NuGet di Microsoft (D-47) | ConPTY aggiornato: Claude Code riceve i tasti; da provare |
| `git` | Git Bash (già nel catalogo) | shell dei comandi di Claude |

Profili nell'editor (riscritti a ogni avvio; profili di sistema nascosti; quelli dell'utente conservati): **Atollo** (pwsh, predefinito, anche per le attività), Atollo - Git Bash, Atollo - cmd, Claude Code / Codex / Antigravity (se installati).

Isolamento: ambiente del kit; `PSModulePath` solo kit; profilo del PC mai caricato (`-NoProfile` + profilo del kit da `data\kit\`); telemetria e controllo aggiornamenti di PowerShell spenti; cronologia in `data\home\`. Il terminale vede tutti gli strumenti scelti in Componenti, con cache in `data\`.

## 7. Editor integrato

Server web di VSCodium in `data\apps\…\vscodium-server\`. Avvio su `127.0.0.1`, porta 7000-7099, token in file, `--server-data-dir data\vscodium-server`, telemetria spenta. Finestre `editor` con cartella dati `data\webview2\`, navigazione solo verso il server, nuove finestre intercettate, tasti del browser e menu contestuale disattivati.

**Editor pulito** (Q4 = a, 14/10): nessuna estensione propria nell'editor. Dentro ci sono VSCodium, le estensioni consigliate e i terminali del kit. Progetti, avvio dei server, agenti e componenti stanno nella finestra principale dell'app, scritti in Rust. Le impostazioni che nel Atollo scriveva l'estensione (profili del terminale, `files.dialog.defaultPath`, `git.defaultCloneDirectory`, `claudeCode.environmentVariables`) le scrive `editor_cfg.rs` prima di ogni avvio.

Prototipo all'inizio della fase dell'editor: dove il server web tiene le impostazioni utente, nome dello script del server, `--install-extension` sul server, chiusura con file non salvati.

## 8. Sicurezza

Invariata rispetto al prompt: comandi solo per `main` (`AppManifest::commands` + capability + controllo dell'etichetta); finestre `editor` senza IPC; CSP stretta; server solo su `127.0.0.1` con token; argomenti a vettore; nomi validati; download con checksum o firma; segreti mascherati nei log; manifesto `asInvoker`.

## 9. Importazione da un Atollo

Esclusa dalla prima versione (Q6 = no, 14/10). I formati di `data\app\` restano comunque semplici e documentati, così si potrà aggiungere in seguito.

## 10. Fasi

| Fase | Contenuto | Consegna verificabile |
| --- | --- | --- |
| 0. Piano | questo documento | piano approvato |
| 1. Scheletro | progetto Tauri, `kit.rs`, `env.rs`, `jobs.rs`, istanza singola, log, finestra vuota, `Atollo.exe env`, CI x64 + ARM64 | exe che parte da chiavetta, crea solo `data\`, nulla fuori |
| 2. Motore di installazione | `net`, `verify`, `archive`, `catalog`, `setup_runner`; componenti `pwsh`, `terminal` (tolto il 14/10, D-44), `git`, `node`, `vscodium-server` | `Atollo.exe prepare --only pwsh,terminal` installa e verifica |
| 3. Editor e terminale | `editor*`, `terminal.rs`, finestre `editor`, capability, estensioni consigliate | progetto aperto nell'editor con terminale Atollo esclusivo; nessuna IPC dalle finestre `editor` |
| 4. Tutti i componenti | altri componenti, rimozione, verifica finale; pagine Componenti e Primo avvio | ogni linguaggio installato e rimosso dall'app |
| 5. Agenti | `agents.rs`, `agents_sync`, `keyring.rs`, `relocate.rs`; pagine Home e Agenti | tre agenti avviati, istruzioni e skill allineate, kit spostato che funziona |
| 6. Progetti e rifinitura | `projects.rs`, `detect`, `run.rs` (avvio dei progetti con porta e anteprima), `sessions.rs`, `self_update.rs`, Impostazioni, Rimuovi l'unità, quattro lingue, accessibilità | checklist manuale completa |
| 7. Rilascio | documentazione in quattro lingue, release `app-v1.0.0` | `Atollo.exe` scaricato da solo in una cartella vuota arriva all'editor senza passaggi manuali |

## 11. Repository

Radice del nuovo repository: `D:\Projects\atollo` (oggi `Prompt.md` e `docs\app\`). Struttura proposta:

```
Cargo.toml  Cargo.lock        workspace
src-tauri\                    exe (moduli §4), catalogo e testi incorporati
ui\                           interfaccia React + TypeScript + Tailwind (D-27)
kit-files\                    file scritti in data\kit\ (loopback.js, profilo PowerShell, bashrc)
docs\app\                     piano, decisioni, catalogo, prove, novità
.github\workflows\app.yml     CI e release
```

Nome del prodotto: **Atollo** (exe `Atollo.exe`), deciso il 14/10. Da quel repository l'exe scarica i propri aggiornamenti.

**Conflitto sul nome del repository** (verificato il 14/10): `github.com/nabre/atollo` esiste già ed è il Atollo attuale, pubblico. Due repository dello stesso account non possono avere lo stesso nome. In più, i kit già distribuiti si aggiornano da soli leggendo `nabre/atollo` (`Update-Setup.ps1`): se quel nome passasse a un altro progetto, al prossimo avvio scaricherebbero file sbagliati. Rinominare il vecchio repository non basta: GitHub smette di reindirizzare il vecchio nome appena se ne crea uno nuovo uguale. Alternative in Q10.

## 12. Domande aperte

Chiuse il 14/10: Q1 prodotto separato · Q2 `projects\` nella radice · Q3 repository nuovo · Q4 editor pulito · Q5 nessun modulo PowerShell in più · Q6 nessuna importazione · Q7 documenti in italiano · Q9 nome "Atollo".

10. ~~Nome del repository~~: chiusa (14/10), **`nabre/atollo`**. Non ancora creato su GitHub: si crea quando l'utente lo chiede.
11. **ConPTY aggiornato** (14/10, D-47): provare `conpty.dll` + `OpenConsole.exe` di Microsoft con Claude Code (serve il permesso dell'utente per eseguire i binari scaricati); se risolve, aggiungere il componente obbligatorio `conpty`, altrimenti cercare altrove la causa.

12. **VSCodium desktop nel catalogo** (14/10): con l'editor integrato non serve; resta come componente facoltativo, non predefinito, per chi vuole l'editor come programma separato (CATALOG.md). Da decidere se toglierlo del tutto.
13. **Commit della fase 6** (14/10): nel working tree le modifiche della fase 6 sono mescolate a quelle del terminale integrato (D-44…D-48), anche negli stessi file (`commands.rs`, `main.rs`, `build.rs`, `api.ts`, `App.test.tsx`). Scelte: un solo commit; due commit separati dividendo a mano i file condivisi; oppure attendere la decisione sul ConPTY (domanda 11).

Piano approvato il 14/10/2026. Fasi 1-6 completate: vedi [CHANGELOG.md](CHANGELOG.md); prove in [TESTING.md](TESTING.md).
