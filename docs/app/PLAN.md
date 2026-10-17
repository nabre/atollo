# Dev-Kit App: piano (fase 0, revisione 3)

Stato: **approvato** (14/10/2026). Fasi 1-6 completate, con le correzioni successive in [CHANGELOG.md](CHANGELOG.md). Fase 7 in corso: job di release `app-v*` con firma facoltativa nella CI (`.github/workflows/app.yml`) e README in quattro lingue fatti; resta la release `app-v1.0.0`.
Base letta: `nabre/dev-kit` al commit `a61a354` (main, 13/10/2026).
Scelte non ovvie: [DECISIONS.md](DECISIONS.md). Domande aperte: §12.

## Indicazioni dell'utente che cambiano il prompt

| Data | Indicazione | Effetto |
| --- | --- | --- |
| 13/10 | 100% portabile, nessuna richiesta di amministratore o di permessi; si usano solo le cartelle accanto all'exe | requisiti 1-3 confermati e resi assoluti |
| 14/10 | Tutta la logica di gestione in Rust, con le chiamate di sistema necessarie | niente script PowerShell, `cmd` o Node per la gestione |
| 14/10 | Il terminale nell'editor è esclusivo del kit: scaricare il terminale migliore e metterci tutto il necessario | componente `pwsh` (§6); console fuori dall'editor nella console classica di Windows (DECISIONS.md D-57, prima D-44) |
| 14/10 | Tutta la logica nel file portabile; una sola cartella `data` con i file scaricati, lo stato, i contenuti e le impostazioni | struttura nuova: **`Dev-Kit.exe` + `data\` + `projects\`** (§2) |
| 14/10 | `projects\` deve stare nella radice | Q2 chiusa |
| 14/10 | Prodotto **separato**, in un altro repository; `nabre/dev-kit` è solo un riferimento (linguaggi, agenti, comportamenti) | Q1 e Q3 chiuse; catalogo in [CATALOG.md](CATALOG.md) |
| 14/10 | Nessun modulo PowerShell in più; documenti in italiano | Q5 e Q7 chiuse |
| 14/10 | Solo strumenti del kit o componenti di Windows, mai programmi del PC; via xterm.js | console in `conhost.exe` di Windows (D-57) |
| 14/10 | L'editor fa parte dell'app, non è un componente da scegliere | gruppo `app` del catalogo, versione fissata a ogni rilascio (D-58) |
| 14/10 | Obbligatori solo PowerShell 7 e GitHub CLI; poi: componenti di sistema GitHub CLI, terminale ed editor, con le loro dipendenze obbligatorie | Git sempre presente perché serve a GitHub CLI; Node.js e uv installati quando servono (D-59) |
| 14/10 | Terminale ed editor saldamente collegati alla cartella del progetto; nessun altro luogo per l'utente | editor, console e agenti solo in un progetto, editor bloccato sulla sua cartella (D-62) |
| 14/10 | Impostazioni del kit per ogni progetto in `data`, file `dev-settings.json` | `data\app\projects\<progetto>\dev-settings.json` (D-63) |
| 14/10 | Barra di Windows della finestra spenta, barra dell'interfaccia | solo finestra principale (D-64) |
| 14/10 | Loghi ufficiali degli strumenti; etichette del codice di ogni progetto; solo gli agenti installati | D-65 |
| 14/10 | Gestione del terminale nel kit: predefinito il terminale del kit, portabile | Windows Terminal portabile (componente `terminal`), la console classica nelle Impostazioni (D-68); tolto il 16/10 (D-95) |
| 16/10 | Windows Terminal non è necessario: toglierlo del tutto | componente `terminal`, impostazione «Terminale» e gruppo `os` tolti; console solo nella console classica di Windows (D-95) |

Requisiti del prompt **superati** da queste indicazioni:
- "Riusa, non riscrivere" gli script del kit → tutto in Rust dentro l'exe.
- Requisito 11 "il menu attuale resta" e `setup\Windows.cmd` di ripiego → non applicabile: è un prodotto nuovo, il Dev-Kit resta com'è nel suo repository.
- Modifiche a `nabre/dev-kit` (componenti nuovi nel suo `components.json`, estensione, `README` in quattro lingue, `checks.yml`) → nessuna: il nuovo repository ha il suo catalogo, la sua CI e la sua documentazione.
- Radice riconosciuta da `setup\components.json` → riconosciuta da `Dev-Kit.exe` stesso; `data\` creata al primo avvio.
- `apps\` e `setup\` nella radice → `apps\` sotto `data\`, `setup\` dentro l'exe; `projects\` resta nella radice.
- Aggiornamento di `setup\` da `main` (`Update-Setup.ps1`) → si aggiorna solo l'exe.
- `data\` condivisa con macOS e Linux → l'app è solo Windows; `data\` è sua.

## 1. Cosa resta valido di quanto ho letto nel kit

Il kit attuale diventa la **specifica di riferimento**: il comportamento si porta in Rust, regola per regola, con riferimento al file d'origine nei commenti.

| Comportamento | Origine | Destinazione Rust |
| --- | --- | --- |
| Ambiente portabile (~90 variabili, PATH del kit) | `setup/windows/_env.cmd` | `env.rs` |
| Rilocazione dopo spostamento o cambio di lettera | `_env.cmd:238-267`, `setup/agents/moved.js` | `agents/relocate.rs` |
| Catalogo componenti (i 27 del Dev-Kit senza VSCodium desktop, più `pwsh` e l'editor) | `setup/components.json` | `setup/catalog.rs`, dati incorporati |
| Download, verifica (SHA-256/512, npm, Authenticode), estrazione, scambio cartelle | `_common.ps1`, `_prepare-*.ps1`, `_languages.ps1`, `_workarea.ps1` | `net/`, `install/` |
| Allineamento di istruzioni, skill, MCP, regole, cartelle attendibili | `setup/agents/sync.js` + 8 moduli | `agents/sync/` |
| Accesso Antigravity fuori da Gestione credenziali | `agy-keyring.ps1` | `agents/keyring.rs` |
| Server su 127.0.0.1 per Node | `setup/node/loopback.js` | file incorporato, scritto in `data\kit\` (Node lo richiede come file) |
| Espulsione e altre copie aperte | `Eject.ps1`, `Test-Sessions.ps1` | `os/sessions.rs` |
| Estensioni consigliate, `@programma`, proposta una volta sola, CodeLLDB | `setup/vscodium/*`, `_vscodium-extensions.ps1` | `editor/ext.rs` |
| Profili del terminale (`claudeCode.environmentVariables` non serve più: D-42) | `setup/extensions/dev-kit/kit.js:163-220` | `editor/cfg.rs` |
| Regole dei nomi dei progetti, rilevatori | `core/projects.js:37-44`, `core/detectors` | `projects/`, `detect/` |
| Testi it/en/fr/de | `setup/lang/parts/*.json` | cataloghi incorporati |

Bug trovato e da non riportare: rimozione di Typst e TinyTeX mai eseguita (`_prepare-migrate.ps1:128`).

## 2. Struttura su disco

```
Dev-Kit\                         qualsiasi cartella, anche con spazi, ( ) e &
  Dev-Kit.exe                    tutta la logica, i testi, il catalogo, le estensioni del kit
  projects\                      i progetti (contenuti), creata al primo avvio
  data\                          creata al primo avvio; tutto il resto è qui
    apps\win32-x64\              programmi scaricati (o win32-arm64)
      git\ node\ python\ pwsh\ vscodium-server\ claude\ codex\ antigravity\ tools\ …
      webview2\                  facoltativo: runtime WebView2 "Fixed Version"
    home\                        profilo portabile: .claude .codex .gemini .gitconfig .ssh AppData\
    agents\                      istruzioni, skill, MCP condivisi dai tre agenti
    antigravity\                 accesso di Antigravity salvato nel kit
    cache\  tmp\                 cache dei pacchetti, temporanei (anche quelli del sistema)
    app\                         settings.json, components.txt, versions\, logs\, update\, ports.json,
                                 root.txt, old-root.txt, extensions.txt, extensions-offered.txt,
                                 projects\<progetto>\dev-settings.json (impostazioni del kit per progetto)
    kit\                         file che i programmi vogliono su disco (loopback.js, profile.ps1, bashrc, bin\agy.cmd), riscritti dall'exe a ogni nuova versione
    webview2\                    dati di WebView2
    vscodium-server\             dati del server dell'editor e token
    … le altre cartelle dati di _env.cmd (npm-global, go, gradle, php, apache…)
```

Regole:
- L'exe riconosce la sua radice dalla propria posizione. Se `data\` manca, la crea (primo avvio). Se la cartella non è scrivibile, finestra di messaggio e uscita.
- Ogni file scritto da app, programmi e agenti finisce in `data\`. Eccezioni ammesse solo quelle del README attuale (tracce di Windows; `gh` senza `--insecure-storage`, cioè un accesso fatto in una console e non dalla pagina GitHub, D-70; Gestione credenziali per Antigravity mentre `agy` è aperto, con persistenza di sola sessione), elencate in `DECISIONS.md`.
- Per fare un backup basta copiare `projects\` e `data\` (meno `data\apps\` e `data\cache\`, che si riscaricano).

## 3. Architettura

```
Dev-Kit.exe (Tauri 2, dentro un Job Object)
 ├─ finestra "main"    interfaccia React, unica con capability e comandi
 ├─ finestre "editor"  http://127.0.0.1:<7000-7099>, nessuna capability
 ├─ motore Rust        tutto il resto (§4)
 └─ figli, tutti nel job:
     codium-server · console in conhost.exe (pwsh del kit, claude / codex / agy) · azioni dei progetti · installazioni
```

Riga di comando dello stesso exe, senza finestra (D-21, `cli.rs`):
- `env`: l'ambiente del kit;
- `components`: il catalogo, con la scelta e le versioni installate;
- `check`: la verifica finale dei programmi installati;
- `prepare [--only a,b] [--skip a,b] [--update-all] [--force]`: installa, aggiorna e rimuove i componenti;
- `sync [--project <cartella>] [--verbose]`: allinea gli agenti;
- `keyring restore|save`: accesso di Antigravity, usati da `data\kit\bin\agy.cmd`;
- `serve <cartella> <porta>`: server di file dei siti statici, usato da `projects/run.rs`;
- `version`, `help`.

Servono a `agy.cmd`, alle azioni dei progetti, alla CI e alle prove a mano.

Avvio (`main.rs`): riga di comando letta → radice → creazione di `data\`, controllo di scrittura, spostamento del kit (`root.txt`) → log, pulizia di `Dev-Kit.old.exe`, versione dell'app → impostazioni → ambiente del kit, applicato al processo → file in `data\kit\` → se c'è un comando, lo esegue ed esce → istanza singola per kit → WebView2 (sistema, fissa in `data\apps\…\webview2\`, o messaggio) → Job Object → finestra `main` → poi: installazione degli obbligatori e dell'editor mancanti o di un'altra versione; in background accesso Antigravity in sospeso, allineamento agenti (con la rilocazione), sorvegliante degli agenti; controllo aggiornamento dopo 20 s. Le altre copie del kit aperte le cerca la Home.

## 4. Moduli Rust (`tauri\src\`)

Una cartella per area; l'elenco dei file, i livelli e le regole comuni per file, processi, comandi ed errori sono in [ARCHITECTURE.md](ARCHITECTURE.md).

| Modulo | Responsabilità |
| --- | --- |
| `main.rs`, `cli.rs`, `log.rs` | avvio, finestre, riga di comando, chiusura; log con segreti mascherati |
| `kit.rs` | radice, piattaforma, percorsi, primo avvio, spostamento, validazione di percorsi e nomi |
| `env.rs` | ambiente del kit; PATH = programmi del kit presenti + `System32`, `Wbem`, `WindowsPowerShell\v1.0`; `PSModulePath` senza i moduli del PC; impostazioni di rete da `data\app\settings.json` (proxy, certificati, chiave Gemini, lingua, rete locale); ricostruito dopo ogni installazione e a ogni salvataggio delle impostazioni |
| `fsx.rs` | file del kit: lettura senza BOM, JSON mancante, vuoto o danneggiato, scrittura atomica con la cartella creata |
| `settings.rs`, `i18n.rs` | `data\app\settings.json` (al posto di `proxy.cmd`); lingua e cataloghi incorporati |
| `os\` | Job Object, avvio con l'ambiente del kit, console `conhost.exe`, `.cmd`/`.bat`, uscita dal job per browser ed Esplora risorse (`jobs`); processi del PC per cartella; programmi del kit aperti, altre copie, «Rimuovi l'unità» (`sessions`); istanza singola per kit (mutex + pipe); WebView2 di sistema o fisso; lettura dentro un progetto senza seguire collegamenti (`safe_path`, D-77) |
| `net\` | HTTPS con il TLS e i certificati di Windows, proxy, 3 tentativi, `.part`, API di GitHub (`digest` delle asset); SHA-256, SHA-512, integrità npm (`verify`); Authenticode con editore atteso (`signature`); zip, tar.gz, 7z, scambio cartelle con ripristino, "file in uso" (`archive`) |
| `setup\` | catalogo incorporato (`keys`, `noArm64`, gruppo `app` con versione fissata), `data\app\components.txt`, `needs` a cascata; cosa serve a un agente, a un clone o a un'azione (D-59); piano e ordine dei passi, annullamento al termine del passo (`runner`); preparazione avviata dalla finestra (`task`); verifica finale (`checkup`, D-39); download a mano (`downloads`, D-82); file in `data\kit\` (`kit_files`); aggiornamento dell'exe in `data\app\update\`, SHA-256, scambio alla chiusura (`self_update`) |
| `install\` | un installatore per famiglia: `core` (git, python, node, uv, gh), `agents`, `web` (php, apache, VC++), `native` (rust, pnpm, r), `lang\` (gli altri linguaggi, un file per famiglia), `editor` (pwsh, editor), `db` (MariaDB, MongoDB, SQLite, Adminer: D-73, D-85), `android` (D-84) |
| `versions\` | versioni salvate, una cartella per versione minore, scelta del kit e dei progetti (D-80); versioni fianco a fianco per Gradle, Maven e .NET (`side_by_side`, D-81); matrice delle relazioni e intervalli di versione di npm, Composer, PEP 440 (`relations\`, `ranges`, D-81) |
| `projects\`, `detect\` | elenco, creazione, clone con il Git del kit, rilevamento del tipo, dei linguaggi e delle azioni, `panel.json`; `dev-settings.json` (`settings`, D-63); MCP e `.vscode\settings.json` (`mcp`, `vscode`, D-78); «Nuovo progetto» da framework (`frameworks\`, D-83); azioni con porte 7100-7999 in `data\app\ports.json`, log, browser, rete locale (`run`, D-50); server di file `Dev-Kit.exe serve` (`serve`, D-51); Apache del progetto (`apache`) |
| `editor\` | server web: porta, token, avvio, salute, riavvio; finestre `editor-<n>` con la barra di VSCodium (`window`, D-69); impostazioni e profili del terminale (`cfg`); estensioni dalla matrice linguaggio-estensione e dai progetti, CodeLLDB (`ext`, D-71; nessuna estensione propria, Q4 = a); tema NEXKORA (`theme`); console del kit in `conhost.exe` (`terminal`, D-57, D-95) |
| `agents\` | stato, lancio in una console del kit, uscita, sorvegliante; allineamento di istruzioni, skill, MCP, regole e cartelle attendibili (`sync\`); limiti d'uso e consumi di Claude Code, Codex e Antigravity (`stats\`, D-61); accesso di Antigravity con Gestione credenziali (`keyring`); rilocazione dopo uno spostamento, regole di `moved.js` (`relocate`) |
| `db\` | server MariaDB e MongoDB del kit, impostazioni in `data\app\services.json`, dati per versione, copie (`services\`, D-73); password e `.env` (`auth`, D-74); contenuto in sola lettura (`browse`, D-79); Adminer (`adminer`, D-85) |
| `github\` | account GitHub del kit, uno per tutto (D-70): `gh auth login --web`, `credential.helper` di Git verso `gh`; repository dell'account, clone, rinomina, eliminazione (`repos`) |
| `commands\` | comandi Tauri, un file per area dell'interfaccia (`app`, `window`, `editing`, `setup`, `versions`, `agents`, `projects`, `frameworks`, `github`, `databases`, `services`, `runs`), errore comune `CmdError` |

Dipendenze (`tauri\Cargo.toml`): `tauri`, `serde`, `serde_json`, `anyhow`, `thiserror`, `windows`, `webview2-com`, `ureq` con `native-tls`, `sha2`, `base64`, `zip`, `tar`, `flate2`, `sevenz-rust2`, `regex`, `junction`. Motivi in D-22.

## 5. Interfaccia

React + TypeScript `strict` + Tailwind, con i controlli di VSCodium (`@vscode-elements/elements`, D-88), Codicons e lo stesso tema NEXKORA dell'editor (D-02, D-27). Chiamate a Rust solo da `ui/src/api/`, un file per area come `commands/`; componenti di base in `ui/src/workbench/`; hook comuni in `ui/src/hooks.ts` ([ARCHITECTURE.md](ARCHITECTURE.md)). Pagine: Home (con la scelta del primo avvio e i progetti recenti), Progetti, Agenti (con limiti d'uso e consumi), Linguaggi e strumenti, Impostazioni, Rimuovi l'unità. Barra del titolo disegnata dall'interfaccia (D-64), barra di stato. Editor, console e agenti si aprono solo in un progetto (D-62). Tutte le funzioni del menu attuale hanno una pagina (tabella di corrispondenza nella revisione 1, invariata). "Proxy e impostazioni" diventa un modulo nella pagina Impostazioni (niente più `proxy.cmd` da modificare a mano).

## 6. Terminale esclusivo del kit

| Componente | Cosa | Perché |
| --- | --- | --- |
| `pwsh` | PowerShell 7 portabile (zip ufficiale, x64 e ARM64, SHA-256), obbligatorio | shell predefinita dell'editor e delle console; PSReadLine incluso; isolabile dal PC |
| (nessuno) | console classica di Windows: `conhost.exe` di System32 avviato direttamente (D-57, D-95); prima Windows Terminal portabile (D-68), e prima ancora xterm.js e `portable-pty` (D-44) | stessa console, senza un programma in più |
| `git` | Git Bash, installato quando serve (con Claude Code, per un clone: D-59) | shell dei comandi di Claude |

Profili nell'editor (riscritti a ogni avvio; profili di sistema nascosti; quelli dell'utente conservati): **Dev-Kit** (pwsh, predefinito, anche per le attività), Dev-Kit - Git Bash, Dev-Kit - cmd, Claude Code / Codex / Antigravity (se installati).

Isolamento: ambiente del kit; `PSModulePath` senza i moduli del PC (D-34); profilo del PC mai caricato (`-NoProfile` + profilo del kit da `data\kit\`); Codex con `allow_login_shell = false`; telemetria e controllo aggiornamenti di PowerShell spenti; cronologia in `data\home\`. Il terminale vede tutti gli strumenti installati in Linguaggi e strumenti, con cache in `data\`.

## 7. Editor integrato

Server web di VSCodium in `data\apps\…\vscodium-server\`, parte dell'app: versione fissata a ogni rilascio (D-58). Avvio su `127.0.0.1`, porta 7000-7099, token in file, `--server-data-dir data\vscodium-server`, telemetria spenta. Finestre `editor` con cartella dati `data\webview2\`, navigazione solo verso il server, nuove finestre intercettate, tasti del browser e menu contestuale disattivati.

**Editor pulito** (Q4 = a, 14/10): nessuna estensione propria nell'editor. Dentro ci sono VSCodium, le estensioni consigliate e i terminali del kit. Progetti, avvio dei server, agenti e componenti stanno nella finestra principale dell'app, scritti in Rust. Le impostazioni che nel Dev-Kit scriveva l'estensione (profili del terminale, `files.dialog.defaultPath`, `git.defaultCloneDirectory`) le scrive `editor/cfg.rs` prima di ogni avvio; `claudeCode.environmentVariables` non serve (D-42). Anche l'estensione `github-repos` del Dev-Kit diventa una pagina dell'app, «GitHub», con l'account GitHub del kit (D-70).

Prototipo all'inizio della fase dell'editor: dove il server web tiene le impostazioni utente, nome dello script del server, `--install-extension` sul server, chiusura con file non salvati.

## 8. Sicurezza

Invariata rispetto al prompt: comandi solo per `main` (`AppManifest::commands` + capability + controllo dell'etichetta); finestre `editor` senza IPC; CSP stretta; server solo su `127.0.0.1` con token; argomenti a vettore; nomi validati; download con checksum o firma; segreti mascherati nei log; manifesto `asInvoker`.

## 9. Importazione da un Dev-Kit

Esclusa dalla prima versione (Q6 = no, 14/10). I formati di `data\app\` restano comunque semplici e documentati, così si potrà aggiungere in seguito.

## 10. Fasi

| Fase | Contenuto | Consegna verificabile |
| --- | --- | --- |
| 0. Piano | questo documento | piano approvato |
| 1. Scheletro | progetto Tauri, `kit.rs`, `env.rs`, `jobs.rs`, istanza singola, log, finestra vuota, `Dev-Kit.exe env`, CI x64 + ARM64 | exe che parte da chiavetta, crea solo `data\`, nulla fuori |
| 2. Motore di installazione | `net`, `verify`, `archive`, `catalog`, `setup_runner`; componenti `pwsh`, `terminal` (tolto e poi ripristinato il 14/10: D-57, D-68; tolto il 16/10: D-95), `git`, `node`, `vscodium-server` | `Dev-Kit.exe prepare --only pwsh,node,git,vscodium-server` installa e verifica (come nella CI) |
| 3. Editor e terminale | `editor*`, `terminal.rs`, finestre `editor`, capability, estensioni consigliate | progetto aperto nell'editor con terminale Dev-Kit esclusivo; nessuna IPC dalle finestre `editor` |
| 4. Tutti i componenti | altri componenti, rimozione, verifica finale; pagina Linguaggi e strumenti e Primo avvio | ogni linguaggio installato e rimosso dall'app |
| 5. Agenti | `agents.rs`, `agents_sync`, `keyring.rs`, `relocate.rs`; pagine Home e Agenti | tre agenti avviati, istruzioni e skill allineate, kit spostato che funziona |
| 6. Progetti e rifinitura | `projects.rs`, `detect`, `run.rs` (avvio dei progetti con porta e anteprima), `sessions.rs`, `self_update.rs`, Impostazioni, Rimuovi l'unità, quattro lingue, accessibilità | checklist manuale completa |
| 7. Rilascio | documentazione in quattro lingue, release `app-v1.0.0` | `Dev-Kit.exe` scaricato da solo in una cartella vuota arriva all'editor senza passaggi manuali |

## 11. Repository

Radice: la radice del repository `nabre/dev-kit2`. Struttura:

```
Cargo.toml  Cargo.lock        workspace
package.json                  workspace npm (ui), comandi dev, build, test, lint
tauri\                        exe (moduli §4), catalogo e testi incorporati
ui\                           interfaccia React + TypeScript + Tailwind (D-02)
runtime\                      file scritti in data\kit\ (loopback.js, profile.ps1, bashrc, bin\agy.cmd, bin\agy)
docs\app\                     piano, decisioni, architettura, catalogo, prove, novità
docs\brand\                   logo, temi, script che li generano
.github\workflows\app.yml     CI e release
Prompt.md                     il prompt iniziale
```

Le cartelle di `tauri\src\` e `ui\src\`: [ARCHITECTURE.md](ARCHITECTURE.md).

Nome del prodotto: **Dev-Kit** (exe `Dev-Kit.exe`), deciso il 14/10. Da quel repository l'exe scarica i propri aggiornamenti.

**Conflitto sul nome del repository** (verificato il 14/10): `github.com/nabre/dev-kit` esiste già ed è il Dev-Kit attuale, pubblico. Due repository dello stesso account non possono avere lo stesso nome. In più, i kit già distribuiti si aggiornano da soli leggendo `nabre/dev-kit` (`Update-Setup.ps1`): se quel nome passasse a un altro progetto, al prossimo avvio scaricherebbero file sbagliati. Rinominare il vecchio repository non basta: GitHub smette di reindirizzare il vecchio nome appena se ne crea uno nuovo uguale. Risolto con Q10.

## 12. Domande aperte

Chiuse il 14/10: Q1 prodotto separato · Q2 `projects\` nella radice · Q3 repository nuovo · Q4 editor pulito · Q5 nessun modulo PowerShell in più · Q6 nessuna importazione · Q7 documenti in italiano · Q9 nome "Dev-Kit".

10. ~~Nome del repository~~: chiusa (14/10), **`nabre/dev-kit2`**, creato su GitHub (`origin`).
11. ~~ConPTY aggiornato~~: chiusa (14/10) da D-57. Le console non usano più una pseudo console: Claude Code gira nella console classica di Windows. Il componente `conpty` non serve.
12. ~~VSCodium desktop nel catalogo~~: chiusa (14/10), tolto dal catalogo (commit `7223793`).
13. ~~Commit della fase 6~~: chiusa (14/10), un solo commit con il terminale integrato (`7187483`).

Nessuna domanda aperta oggi.

## 13. Lavori aperti: server web e database

Restano dallo spec dei server web (16/10/2026); il resto (versioni del progetto in due zone, Apache e Nginx versionati, PHP < 8.0, rilevamento di `.htaccess`, CMS e `nginx.conf`, domini `*.localhost`, alias di percorso, pagina Server con profili) è fatto.

Stato al 17/10/2026 (sciame v3, [CHANGELOG.md](CHANGELOG.md)): punto 1 in parte (all'arresto il sito esce dal pool, pool che si spengono, `keepAlive` e `autostart` fatti, D-101; resta l'aggancio al pool automatico calcolato dalle versioni per un sito senza `server`); punto 2 fatto (D-102); punto 3 ancora da decidere.

1. **Pool all'avvio del progetto.** «Avvia» su un sito Apache o Nginx lancia ancora un `httpd.exe` o `nginx.exe` per progetto (`projects/run.rs`); i pool di `servers/mod.rs` partono solo dalla pagina Server. Da fare:
   - all'avvio il progetto calcola il pool `(motore, versione del server, versione di PHP)` e si aggancia a quello acceso, con un reload; altrimenti lo crea e lo avvia. **Resta**: oggi si aggancia (con ricarica) solo un sito con `server` esplicito in `panel.json`;
   - all'arresto il VirtualHost esce dal pool; un pool creato in automatico senza più progetti si spegne. **Fatto** (17/10/2026, D-101);
   - sui profili, «Mantieni attivo anche senza progetti» e «Avvia con NEXKORA» (`keepAlive`, `autostart` in `servers.json`). **Fatto** (17/10/2026, D-101).
2. **Più versioni dello stesso database accese insieme.** Es. MariaDB 10.6 sulla 3306 e 11.4 sulla 3307 per due progetti che le chiedono; la porta della versione va al progetto in `DB_PORT`. Oggi una porta per database (`db/services/mod.rs`, `port`); i dati sono già separati per versione (`data\db\<id>\<linea>`). **Fatto** (17/10/2026, D-102): porta per versione in `services.json`, versione del progetto in `db.json`, `DB_PORT` nell'ambiente.
3. **Una porta per progetto nel pool** (da decidere). Lo spec prevedeva un `Listen` per VirtualHost; oggi il pool ha una porta e distingue i siti per nome (`<progetto>.localhost`). Va scelto se tenere così o aggiungere la porta per progetto (utile per chi apre `127.0.0.1:<porta>`).

Piano approvato il 14/10/2026. Fasi 1-6 completate: vedi [CHANGELOG.md](CHANGELOG.md); prove in [TESTING.md](TESTING.md).
