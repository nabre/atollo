## Ruolo, obiettivo e modo di lavorare

Sei uno sviluppatore senior Rust/TypeScript con esperienza su Tauri 2, Windows e VS Code. Devi realizzare **Dev-Kit App**: un eseguibile Windows portabile (`Dev-Kit.exe`) che sostituisce il menu in console del Dev-Kit e integra l'editor VSCodium in versione web nella propria finestra.

In una frase: l'utente copia la cartella del kit su una chiavetta o nel profilo, fa doppio clic su `Dev-Kit.exe`, sceglie agenti e linguaggi, apre i progetti in un editor integrato e lancia Claude Code, Codex o Antigravity. Niente installazione, niente diritti di amministratore, nessun file fuori dal kit.

Come lavorare:

1. **Prima leggi, poi pianifica.** Leggi i file indicati nella sezione Contesto e scrivi in `docs/app/PLAN.md` un piano con moduli, interfacce e domande aperte. Fermati e chiedi conferma prima di scrivere codice.
2. **Riusa, non riscrivere.** Installazione, verifica, aggiornamento e allineamento degli agenti esistono già negli script del kit. L'app li chiama; non li duplica. Proponi un porting solo dove lo script è un ostacolo misurabile.
3. **Una fase alla volta.** Segui le fasi della sezione Fasi di lavoro. Ogni fase termina con build verde, test verdi e una breve nota in `docs/app/CHANGELOG.md`.
4. **Decisioni scritte.** Ogni scelta non ovvia (versione di una dipendenza, API di Tauri usata, eccezione alla portabilità) va in `docs/app/DECISIONS.md` con motivo e alternativa scartata.
5. **Verifica le API, non ricordarle.** Per Tauri 2, WebView2 e il server di VSCodium controlla la documentazione o il codice della versione che usi. Se un'API citata qui non esiste in quella versione, cerca l'equivalente e annotalo.
6. **Domande aperte.** Se un requisito è ambiguo o in conflitto con un altro, chiedi. Non inventare comportamenti per gli utenti.

## Contesto: il Dev-Kit e cosa riusare

Il [Dev-Kit](https://github.com/nabre/dev-kit) è un ambiente di sviluppo portabile per Windows, macOS e Linux, contenuto in una cartella. Oggi si avvia da `setup\Windows.cmd`, che apre un menu in console. L'app nuova lavora **solo su Windows** e affianca questo menu senza romperlo: `Windows.cmd` deve continuare a funzionare.

Leggi questi file prima di pianificare:

| File | Cosa ti serve sapere |
| --- | --- |
| `README.md` | Struttura del kit, componenti, cartelle dei dati, sicurezza, cosa può finire fuori dal kit |
| `setup/windows/_env.cmd` | L'ambiente del kit: tutte le variabili che reindirizzano profilo, cache e strumenti dentro `data\` |
| `setup/components.json` | Catalogo dei componenti: id, gruppo, dipendenze, dimensioni, dove leggere le versioni, file che indica l'installazione |
| `setup/windows/Prepare-DevKit.ps1`, `_prepare-*.ps1`, `_components.ps1` | Installazione e aggiornamento, opzioni `-Only`, `-Skip`, `-UpdateAll`, `-Choose`; scelta salvata in `data\components.txt` |
| `setup/windows/Menu-Select.ps1`, `menu.cmd`, `_ui.ps1` | Voci e comportamento del menu da riprodurre |
| `setup/windows/Claude.cmd`, `Codex.cmd`, `Antigravity.cmd`, `_agent.cmd`, `agy-keyring.ps1` | Come si lanciano gli agenti e come si sincronizza l'accesso di Antigravity |
| `setup/windows/Eject.ps1`, `Test-Sessions.ps1`, `Update-Setup.ps1` | Rimozione sicura, sessioni aperte, aggiornamento degli script del kit |
| `setup/agents/sync.js` e moduli | Allineamento di istruzioni, skill e MCP fra i tre agenti; va eseguito prima e dopo ogni agente |
| `setup/node/loopback.js` | Server Node forzati su 127.0.0.1 |
| `setup/vscodium/extensions.txt`, `app-extensions.js` | Estensioni consigliate, regola `@programma`, "proposta una volta sola" |
| `setup/extensions/dev-kit/` | Estensione VSCodium del kit: pannello progetti, statistiche agenti, pagina App |
| `setup/lang/`, `setup/extensions/dev-kit/i18n/` | Cataloghi dei testi in it, en, fr, de e `build.js` che li verifica |

Cosa riusi così com'è:

- **Gli script PowerShell** per installare, aggiornare, rimuovere componenti e per l'espulsione. L'app li avvia in background e ne mostra l'output.
- **`setup/components.json`** come unica fonte dei componenti. Non duplicare l'elenco nel codice dell'app.
- **`setup/agents/sync.js`** per l'allineamento degli agenti.
- **L'estensione Dev-Kit** e la lista delle estensioni, da installare nel server web (vedi la sezione Editor integrato).
- **I cataloghi dei testi** e le loro regole.

Cosa è nuovo: l'eseguibile Tauri, la gestione del server web di VSCodium, l'interfaccia grafica che sostituisce il menu e la pagina "Linguaggi e strumenti" in console.

## Requisiti non negoziabili

L'app è una sandbox dello **stato**, non della sicurezza: tutto ciò che i programmi del kit scrivono deve finire in `apps\`, `data\` o `projects\`. Ogni requisito qui sotto va verificato da un test o dalla checklist manuale.

1. **Nessuna installazione.** Un solo `Dev-Kit.exe` nella radice del kit, avviabile con doppio clic. Nessun installer, MSI o NSIS, nessun servizio, nessuna attività pianificata.
2. **Nessun diritto di amministratore.** Nessuna richiesta UAC, nessun manifesto `requireAdministrator`.
3. **Nessuna scrittura fuori dal kit.** Niente registro (né HKCU né HKLM), niente PATH di sistema, niente `%APPDATA%`, `%LOCALAPPDATA%` o `%TEMP%` reali. Le sole eccezioni ammesse sono quelle già elencate nel README del kit (tracce di Windows, Gestione credenziali se l'utente la sceglie) e vanno documentate.
4. **Percorsi relativi alla posizione dell'exe.** Il kit funziona da qualsiasi cartella, anche con spazi, parentesi e `&` nel nome, e dopo un cambio di lettera di unità. All'avvio l'app esegue la stessa rilocazione del kit (vedi `setup/agents/moved.js` e la sezione del README sullo spostamento).
5. **WebView2 senza installarlo.** L'app usa WebView2 di sistema se presente. Se manca, usa un runtime "Fixed Version" in `apps\win32-x64\webview2\` indicato con `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`. Se manca anche quello, mostra una pagina di errore nativa (finestra di messaggio Win32) con il link per scaricarlo nel kit. Non lancia mai il bootstrapper di Microsoft.
6. **Dati di WebView2 nel kit.** La cartella dati di ogni WebView (cookie, IndexedDB, cache) è `data\webview2\`, mai `%LOCALAPPDATA%`. Verifica che nessun file `EBWebView` compaia nel profilo reale.
7. **Ambiente sempre del kit.** Ogni processo avviato dall'app riceve l'ambiente del kit, ricostruito come fa `_env.cmd`. Nessun processo eredita il PATH dell'utente senza filtrarlo.
8. **Una sola istanza per kit.** Un secondo avvio dello stesso kit porta in primo piano la finestra esistente. Se è aperta un'altra copia del kit (un'altra chiavetta), l'app avvisa come fa oggi `Test-Sessions.ps1`.
9. **Nessun processo orfano.** Alla chiusura dell'app, anche se forzata, il server dell'editor, i terminali e i server avviati dai progetti si fermano. Usa un Job Object di Windows con `KILL_ON_JOB_CLOSE`.
10. **Compatibilità.** Windows 10 1809 o successivo e Windows 11, x64. Su ARM64 l'exe può essere ARM64 nativo, ma il server web di VSCodium esiste solo x64 e gira in emulazione: va detto nell'interfaccia.
11. **Il menu attuale resta.** `setup\Windows.cmd` e gli script continuano a funzionare. L'app non cambia formati di file condivisi (`data\components.txt`, `data\versions\`, `data\agents\`) senza compatibilità all'indietro.

## Struttura del kit e ambiente

La radice del kit è la cartella che contiene `Dev-Kit.exe`. L'app la riconosce perché accanto trova `setup\components.json`. Se non lo trova, mostra un errore e non crea nulla.

```
Dev-Kit\
  Dev-Kit.exe                  l'app (nuova)
  setup\                       codice del kit (esistente)
    app\                       sorgenti dell'app Tauri (nuova)
  apps\win32-x64\              programmi: claude, codex, antigravity, git, node, python, vscodium-server, webview2...
  data\                        stato: home, accessi, impostazioni, cache, temporanei
    app\                       impostazioni e log dell'app (nuova)
    webview2\                  dati di WebView2 (nuova)
    vscodium-server\           dati del server dell'editor (nuova)
  projects\                    i progetti che l'editor apre
```

La piattaforma (`win32-x64` o `win32-arm64`) si calcola come negli script del kit. L'ambiente si costruisce in Rust, in un modulo dedicato, con queste regole:

- **Fonte di verità**: `setup\windows\_env.cmd`. Il modulo Rust riproduce le stesse variabili con gli stessi valori. Scrivi un test che esegue `_env.cmd` in un processo `cmd /c`, stampa `set` e confronta l'esito con l'ambiente calcolato in Rust, variabile per variabile. Le differenze ammesse vanno elencate nel test con un motivo.
- **In alternativa**, se la duplicazione è fragile, l'app può ottenere l'ambiente eseguendo `_env.cmd` e leggendo `set`. Valuta le due strade nel piano e scegli.
- **Variabili utente da conservare**: proxy e impostazioni lette da `data\proxy.cmd`, `DEVKIT_LANG`, `DEVKIT_ALLOW_LAN`, `DEVKIT_NO_UPDATE`.
- **PATH**: prima le cartelle di `apps\<piattaforma>\` e `data\` previste dal kit, poi le sole cartelle di sistema indispensabili (`System32`, `System32\Wbem`, `WindowsPowerShell\v1.0`). Nessuna cartella dell'utente reale.

Variabili minime, da confrontare con `_env.cmd`:

| Variabile | Valore nel kit |
| --- | --- |
| `DEVKIT`, `CP_ROOT`, `CP_APPS`, `CP_DATA` | radice, `apps\<piattaforma>`, `data` |
| `USERPROFILE`, `HOME` | `data\home` |
| `APPDATA`, `LOCALAPPDATA` | `data\home\AppData\Roaming`, `data\home\AppData\Local` |
| `TEMP`, `TMP` | `data\tmp\system` |
| `CLAUDE_CONFIG_DIR`, `CLAUDE_CODE_TMPDIR`, `DISABLE_UPDATES` | come in `_env.cmd` |
| `CODEX_HOME`, `AGY_CLI_DISABLE_AUTO_UPDATE` | come in `_env.cmd` |
| `NODE_OPTIONS` | `--require` di `setup\node\loopback.js` |
| npm, pip, uv, Rust, Go, Java, .NET, Composer... | cartelle in `data\` e `apps\` come in `_env.cmd` |
| `CP_USER_PROFILE` | il profilo reale, solo come riferimento |

L'app stessa (processo Tauri) non scrive nulla fuori dal kit: log in `data\app\logs\`, impostazioni in `data\app\settings.json`, dati WebView2 in `data\webview2\`.

## Architettura

Tauri 2 con backend in Rust e interfaccia in TypeScript con Vite. Usa un framework leggero (Svelte o nessuno), senza librerie UI pesanti. Il codice va in `setup\app\` con questa organizzazione:

```
setup\app\
  src-tauri\
    src\
      main.rs          avvio, istanza singola, finestre, menu
      kit.rs           radice, piattaforma, percorsi, rilocazione
      env.rs           ambiente del kit (vedi Struttura del kit e ambiente)
      jobs.rs          Job Object, avvio processi, log in streaming, arresto
      components.rs    components.json, data\components.txt, versioni installate
      setup_runner.rs  esecuzione di Prepare/Eject/Update-Setup con output in tempo reale
      editor.rs        server web di VSCodium: porta, token, avvio, salute, riavvio
      editor_cfg.rs    impostazioni utente del server, profili terminale, estensioni
      agents.rs        lancio di claude/codex/agy con sync.js prima e dopo
      projects.rs      elenco, creazione e rilevamento dei progetti
      sessions.rs      altre copie del kit aperte, espulsione
      i18n.rs          testi in it, en, fr, de
    capabilities\      permessi per finestra (vedi Sicurezza)
    tauri.conf.json
  src\                 interfaccia (pagine della sezione Interfaccia)
  tests\
```

Regole di progetto:

- **Comandi Tauri espliciti.** L'interfaccia chiama solo comandi Rust con nomi e parametri tipizzati, per esempio `components_list`, `components_apply(ids)`, `editor_open(project)`, `agent_launch(agent, project)`. Nessun comando generico come "esegui questa riga".
- **Niente plugin shell aperto.** I processi si avviano solo da Rust, con percorsi calcolati dal kit. Se usi `tauri-plugin-shell`, limitane l'ambito a nulla dal frontend.
- **Processi lunghi in background.** Installazioni, aggiornamenti ed espulsione girano in un task asincrono. L'interfaccia riceve righe di log ed eventi di stato tramite eventi Tauri e non si blocca mai.
- **Un solo Job Object** per tutta l'app: ogni processo figlio (server dell'editor, PowerShell, console degli agenti) viene assegnato al job alla creazione.
- **Finestre.** `main` (l'app) ed `editor` (una o più finestre dell'editor, una per progetto aperto). Le finestre dell'editor caricano solo l'URL del server locale.
- **Console degli agenti fuori dall'editor.** Il lancio diretto di un agente apre Windows Terminal se il kit lo contiene o se è presente nel sistema, altrimenti `conhost`. In entrambi i casi si usa l'ambiente del kit e si passa da `agents.rs`.
- **Plugin ammessi**: istanza singola, finestre di dialogo native (solo per scegliere cartelle), apertura di link esterni nel browser di sistema. Ogni altro plugin va motivato in `DECISIONS.md`.
- **Dipendenze Rust** minime e diffuse (`serde`, `tokio`, `windows` o `windows-sys`, `anyhow` o `thiserror`). Versioni fissate in `Cargo.lock`, incluso nel repository.

## Editor integrato: VSCodium server web

L'editor è il server web ufficiale di VSCodium (`vscodium-reh-web-win32-x64-<versione>.tar.gz`, dalle release di [VSCodium](https://github.com/VSCodium/vscodium/releases)). Il pacchetto contiene `node.exe`, `bin\codium-server.cmd` e `node-pty` per il terminale; estratto pesa circa 250 MB. Si aggiunge al catalogo come componente obbligatorio `vscodium-server` in `setup\components.json` e si installa con gli script del kit, verificando il checksum SHA-256 pubblicato da VSCodium. Il VSCodium desktop resta un componente separato e opzionale.

**Avvio del server** (`editor.rs`):

1. Scegli una porta libera su `127.0.0.1` nell'intervallo 7000-7099, diverso da quello del pannello progetti (7100-7999).
2. Genera un token casuale di 32 byte per la sessione e scrivilo in `data\vscodium-server\token` con permessi ristretti all'utente. Non metterlo mai sulla riga di comando.
3. Avvia `codium-server.cmd` nel Job Object, con l'ambiente del kit e questi argomenti: `--host 127.0.0.1 --port <porta> --connection-token-file <file token> --server-data-dir data\vscodium-server --accept-server-license-terms --telemetry-level off`.
4. Attendi che `GET /` risponda (timeout 30 secondi, con messaggio d'errore leggibile e log del server in `data\app\logs\editor.log`).
5. Apri la finestra `editor` su `http://127.0.0.1:<porta>/?tkn=<token>&folder=<percorso del progetto>`. Il server risponde con un reindirizzamento che imposta il cookie del token.
6. Se il server termina in modo inatteso, l'app lo segnala e offre di riavviarlo. Le finestre dell'editor si ricollegano da sole.

Il server va avviato una volta sola per sessione. Ogni progetto aperto è una nuova finestra sullo stesso server.

**Finestre e WebView2:**

- Cartella dati WebView2 `data\webview2\` per tutte le finestre (opzione della cartella dati del builder della finestra in Tauri 2; verifica il nome esatto). Lo stato dell'editor salvato in IndexedDB deve sopravvivere a riavvio e spostamento del kit.
- Le richieste di nuova finestra dell'editor (`window.open`, "Nuova finestra") vanno intercettate. Gli URL del server locale aprono un'altra finestra `editor`, tutti gli altri si aprono nel browser di sistema. Verifica nella versione di Tauri usata quale API intercetta le nuove finestre e annotala.
- Le scorciatoie (Ctrl+W, Ctrl+N, Ctrl+Tab, F11) devono arrivare all'editor. Disattiva quelle di WebView2 che vanno in conflitto (menu contestuale del browser, zoom con Ctrl+rotella se l'editor lo gestisce già, tasti di navigazione).
- Titolo della finestra: nome del progetto più "Dev-Kit". Icona del kit da `setup\icons\`.

**Impostazioni dell'editor** (`editor_cfg.rs`), scritte prima di ogni avvio in `data\vscodium-server\data\User\settings.json`:

- Profili del terminale per Windows, riscritti a ogni avvio come fa oggi l'estensione Dev-Kit: "Dev-Kit - Git Bash" (predefinito se Git c'è), "Dev-Kit" (cmd con l'ambiente del kit), "Claude Code", "Codex", "Antigravity" (solo se installati), "Dev-Kit - PowerShell". I profili di sistema sono nascosti. I profili aggiunti dall'utente restano.
- Solo se mancano: `files.dialog.defaultPath` e `git.defaultCloneDirectory` su `projects\`, tema icone `vscode-icons`, `workbench.startupEditor` a `none`, `telemetry.telemetryLevel` a `off`.
- Variabili per l'estensione Claude Code (`claudeCode.environmentVariables`) come fa oggi `setup\extensions\dev-kit\kit.js`.
- Se il file contiene commenti o JSON non valido, non toccarlo e mostra un avviso.

**Estensioni**: legge `setup\vscodium\extensions.txt` e `data\vscodium-extensions.txt` con le stesse regole di oggi (`@programma`, proposta una volta sola registrata in `data\vscodium-extensions-offered.txt`). Le installa con `codium-server.cmd --install-extension <id>` prima di avviare il server, mostrando l'avanzamento nell'app. Una che fallisce viene ritentata al prossimo avvio. CodeLLDB segue l'eccezione già documentata nel kit.

**Estensione Dev-Kit**: installa nel server `setup\extensions\dev-kit` e `github-repos` (pacchettizzate con `build-vsix.js`). Nel server web girano nell'extension host remoto: verifica una per una le funzioni che presumono il desktop (link `vscodium://`, avvio di terminali, apertura di nuove finestre, lettura di `_env.cmd`) e adattale. Dove l'app fa già la stessa cosa (pagina App, statistiche), proponi nel piano se tenere la versione nell'estensione, spostarla nell'app o condividerla.

## Interfaccia

La finestra principale ha una barra laterale con le pagine qui sotto e una barra di stato con lo stato dell'editor, delle operazioni in corso e degli agenti. Tema chiaro e scuro secondo Windows, testi nella lingua del kit. Tutte le pagine sono utilizzabili da tastiera e con lettore di schermo (etichette ARIA, focus visibile).

| Pagina | Cosa fa | Cosa usa |
| --- | --- | --- |
| **Primo avvio** | Compare quando mancano i componenti obbligatori. Mostra la scelta di agenti e linguaggi (stessa logica e predefiniti della pagina "Linguaggi e strumenti"), lo spazio richiesto e i requisiti di rete. Con "Installa" avvia la preparazione e mostra il log | `components.json`, `Prepare-DevKit.ps1` |
| **Home** | Pulsanti grandi: Editor, Claude Code, Codex, Antigravity, Terminale. Elenco dei progetti recenti. Avvisi (kit spostato, componenti non installati, aggiornamento disponibile) | `projects.rs`, `agents.rs` |
| **Progetti** | Elenco delle sottocartelle di `projects\` con tipo rilevato, linguaggi, data di modifica. Azioni: apri nell'editor, apri un agente nella cartella, terminale, Esplora risorse. "Nuovo progetto" con le stesse regole di nome del pannello. Clona da GitHub se `gh` è autenticato | rilevatori di `setup\extensions\dev-kit\core\detectors` portati o richiamati |
| **Agenti** | Stato di ciascun agente: installato, versione, accesso fatto o no. Pulsanti per accedere e uscire. Collegamento alla cartella `data\agents` (istruzioni, skill, MCP). Facoltativo in una fase successiva: limiti d'uso come nelle statistiche dell'estensione | `agents.rs`, `setup\agents\sync.js` |
| **Componenti** | Tabella per gruppo (obbligatori, agenti, linguaggi) con icona, versione installata, ultima versione, spazio, dipendenze. Spunte per scegliere; "Applica" mostra cosa verrà installato e rimosso, poi esegue la preparazione in background con log in tempo reale e annullamento sicuro | `components.json`, `data\components.txt`, `data\versions\`, `Prepare-DevKit.ps1 -Only/-Skip` |
| **Impostazioni** | Lingua, proxy e certificati (modifica guidata di `data\proxy.cmd`), rete locale (`DEVKIT_ALLOW_LAN`), aggiornamento automatico del kit, cartella WebView2 fissa, cartella dei log | `data\proxy.cmd`, `data\app\settings.json` |
| **Rimuovi l'unità** | Elenca i programmi del kit aperti, chiede conferma, ferma editor, terminali, agenti e server dei progetti, salva l'accesso di Antigravity, poi chiude l'app | `Eject.ps1` o porting in `sessions.rs` |

Comportamenti trasversali:

- **Operazioni in corso**: una sola preparazione alla volta. Le altre azioni che la richiedono restano disattivate con il motivo scritto.
- **Errori**: ogni errore mostra cosa è successo, cosa fare e un pulsante "Copia dettagli" con le ultime righe di log. Niente stack trace nell'interfaccia.
- **Chiusura**: se ci sono file non salvati nell'editor o processi attivi, chiedi conferma elencandoli.
- **Avvio veloce**: la finestra principale compare entro 1 secondo su SSD; il server dell'editor parte solo quando serve, oppure in background dopo l'avvio se l'utente lo sceglie nelle impostazioni.

## Sicurezza

La regola centrale: **le finestre dell'editor non devono poter chiamare l'app.** L'editor esegue codice di estensioni di terze parti e pagine web di anteprima; se raggiungesse i comandi Tauri potrebbe installare componenti o avviare processi.

- **Capability per finestra.** I comandi Tauri sono permessi solo alla finestra `main`. Le finestre `editor` non hanno alcuna capability e non hanno accesso remoto all'IPC. Scrivi un test che, da una finestra `editor`, prova a invocare un comando e verifica che venga rifiutato.
- **CSP stretta** per l'interfaccia dell'app: solo risorse locali, niente `unsafe-eval`, nessuna connessione esterna dal frontend. Le chiamate di rete (versioni, aggiornamenti) passano da Rust.
- **Server dell'editor**: solo su `127.0.0.1`, token casuale per sessione, porta casuale nell'intervallo dell'app. Mai `0.0.0.0`, mai senza token.
- **Navigazione**: le finestre `editor` possono navigare solo verso `http://127.0.0.1:<porta del server>`. Ogni altro URL si apre nel browser di sistema, previa validazione dello schema (`http`, `https`). Blocca `file:`, `javascript:` e schemi sconosciuti.
- **Processi**: argomenti passati come vettore, mai composti in una stringa per una shell. Percorsi sempre calcolati dalla radice del kit; nomi di progetto validati (niente `..`, caratteri vietati da Windows, nomi riservati come `CON`).
- **Download**: solo dalle fonti già usate dal kit, con verifica del checksum. Il nuovo componente `vscodium-server` segue la stessa regola.
- **Segreti**: token, accessi degli agenti e chiavi API non finiscono mai nei log. I log mascherano le variabili che contengono `KEY`, `TOKEN`, `SECRET`, `PASSWORD`.
- **Attendibilità dei progetti**: l'app non cambia le regole del kit. Avvisa nell'interfaccia, nella pagina Progetti e al primo avvio, che le cartelle in `projects\` sono attendibili per Claude e Codex, come spiega il README.
- **Firma**: prevedi nella CI un passaggio opzionale di firma del codice (disattivato se mancano i segreti). Senza firma, documenta nel README come sbloccare l'exe se SmartScreen o AppLocker lo fermano.

## Lingue, testi e convenzioni

- **Lingue**: italiano, inglese, francese, tedesco, come il resto del kit. Lingua scelta con `DEVKIT_LANG`, altrimenti quella di Windows, altrimenti inglese.
- **Testi sorgente in italiano** con cataloghi di traduzione, come gli script del kit. Aggiungi i cataloghi dell'app a `setup\lang\parts\` (un file per area dell'app) e fai in modo che `node setup/lang/build.js` li verifichi: ogni testo tradotto in tutte le lingue, altrimenti la build dei cataloghi fallisce.
- **Stile dei testi**: frasi brevi, verbi d'azione sui pulsanti, nessun gergo tecnico dove non serve. Usa gli stessi nomi del README per le stesse cose ("Linguaggi e strumenti", "Rimuovi l'unità", "Proxy e impostazioni").
- **Commenti nel codice in inglese**, come nel resto del repository. Ogni modulo inizia con un commento che spiega cosa fa e perché.
- **Rust**: `cargo fmt`, `cargo clippy -- -D warnings`, nessun `unwrap()` fuori dai test. Errori con contesto (`anyhow::Context` o tipi `thiserror`).
- **TypeScript**: modalità `strict`, ESLint e Prettier come nel resto del repository se presenti.
- **Documentazione**: aggiorna `README.md` e le sue traduzioni in `docs\` con una sezione sull'app (avvio, pagine, cosa resta del menu in console, limiti noti). Segui `docs/CONTRIBUTING.md`.

## Build, CI e rilascio

- **Build locale**: documenta in `setup\app\README.md` i prerequisiti (Rust stabile con toolchain MSVC, Node LTS) e i comandi. Il kit stesso deve poter compilare l'app usando i suoi Rust e Node; se il toolchain `gnu` del kit non basta, scrivilo e spiega perché.
- **Niente installer**: costruisci con `tauri build --no-bundle` (o l'opzione equivalente della versione usata) e prendi l'exe da `target\release`. Nessun bundle NSIS o MSI.
- **Due architetture**: `x86_64-pc-windows-msvc` e `aarch64-pc-windows-msvc`.
- **GitHub Actions**: nuovo workflow `app.yml` su `windows-latest`, attivato da push e pull request che toccano `setup/app/**`. Passi: cache di Cargo e npm, `cargo fmt --check`, `cargo clippy`, test Rust, test del frontend, build delle due architetture, artefatti scaricabili. Aggiungi l'app al workflow `checks.yml` esistente solo se non lo rallenta troppo.
- **Rilascio**: su tag `app-v*` il workflow allega gli exe a una release GitHub con il loro SHA-256 in un file `.sha256`. Firma del codice facoltativa (vedi Sicurezza).
- **Distribuzione nel kit**: l'exe non va nel repository. Gli script di preparazione lo scaricano dalla release come nuovo componente obbligatorio `app` (verifica SHA-256) e lo copiano nella radice del kit come `Dev-Kit.exe`. `setup\Windows.cmd` resta il punto di avvio di ripiego.
- **Aggiornamento dell'app**: all'avvio, se l'aggiornamento del kit è attivo (`DEVKIT_NO_UPDATE` assente), confronta la versione con l'ultima release. Se è più recente, scarica in `data\app\update\`, verifica il checksum, poi alla chiusura rinomina il vecchio exe in `Dev-Kit.old.exe` e mette al suo posto il nuovo. Al primo avvio successivo elimina il vecchio. Nessun servizio di aggiornamento in background.
- **Versioni**: SemVer per l'app, scritta in `data\versions\app-win32-<arch>.txt` come per gli altri componenti.

## Test e criteri di accettazione

**Test automatici** (girano nella CI):

- `kit.rs`: radice trovata da percorsi con spazi, parentesi, `&`, lettere di unità diverse, percorsi lunghi; errore chiaro se `setup\components.json` manca.
- `env.rs`: confronto con l'esito di `_env.cmd` (vedi Struttura del kit e ambiente); PATH senza cartelle dell'utente reale.
- `components.rs`: lettura di `components.json` reale del repository, dipendenze (Apache con PHP, Kotlin con Java), compatibilità con `data\components.txt` scritto dagli script.
- `editor_cfg.rs`: profili riscritti, impostazioni dell'utente conservate, file con commenti lasciato intatto; parsing di `extensions.txt` con `@programma` e commenti.
- `projects.rs`: validazione dei nomi, rilevamento su progetti di esempio.
- `editor.rs`: test di integrazione che scarica il server web (in cache nella CI), lo avvia con token, verifica 403 senza token e reindirizzamento con token, poi lo ferma e controlla che non restino processi.
- Sicurezza: una finestra `editor` non può invocare comandi Tauri.

**Checklist manuale** (in `docs\app\TESTING.md`, da eseguire prima di ogni release):

- [ ] Kit nuovo su chiavetta: doppio clic, primo avvio, installazione dei predefiniti, editor aperto su un progetto.
- [ ] Nessun file nuovo in `%APPDATA%`, `%LOCALAPPDATA%`, `%TEMP%` e nel registro dopo una sessione completa (confronto prima/dopo con Process Monitor o un'istantanea delle cartelle).
- [ ] Kit spostato su un'altra lettera di unità e in una cartella con spazi: tutto funziona, stato dell'editor conservato.
- [ ] PC senza WebView2 (Windows 10 LTSC o macchina virtuale): messaggio chiaro, funzionamento con il runtime fisso nel kit.
- [ ] Utente standard senza diritti di amministratore, nessuna richiesta UAC.
- [ ] Rete con proxy autenticato e con ispezione SSL: installazione ed estensioni funzionano con `data\proxy.cmd`.
- [ ] Senza rete: l'app parte, l'editor parte, gli errori di download sono leggibili.
- [ ] Chiusura forzata dell'app dal Task Manager: nessun `node.exe`, `codium-server` o terminale rimasto.
- [ ] Due copie del kit aperte: avviso corretto.
- [ ] Claude Code, Codex e Antigravity: accesso, avvio da app ed editor, allineamento di istruzioni e skill fra i tre.
- [ ] Rimuovi l'unità: tutto si chiude, la chiavetta si espelle senza "dispositivo in uso".
- [ ] ARM64: exe nativo, editor in emulazione, avviso visibile.
- [ ] Il menu `setup\Windows.cmd` funziona ancora dopo che l'app ha modificato configurazioni e componenti.

**Criteri di accettazione dell'intero progetto**: tutte le voci della checklist superate; nessuna funzione del menu in console senza equivalente nell'app (o motivata in `DECISIONS.md`); build della CI verde sulle due architetture; documentazione aggiornata in quattro lingue.

## Fasi di lavoro

Ogni fase si chiude con build e test verdi, nota in `docs\app\CHANGELOG.md` e un breve riepilogo per l'utente con cosa provare a mano. Non iniziare la fase successiva senza conferma.

| Fase | Contenuto | Consegna verificabile |
| --- | --- | --- |
| 0. Piano | Lettura del repository, `docs\app\PLAN.md`, `DECISIONS.md` con le prime scelte (ambiente in Rust o da `_env.cmd`, framework del frontend, API di Tauri per cartella dati e nuove finestre) | Piano approvato |
| 1. Scheletro | Progetto Tauri in `setup\app\`, `kit.rs`, `env.rs`, istanza singola, Job Object, log, finestra principale vuota, CI con build delle due architetture | Exe che parte da chiavetta, trova il kit, non scrive nulla fuori |
| 2. Editor | Componente `vscodium-server` in `components.json` e negli script, `editor.rs`, `editor_cfg.rs`, finestre `editor`, cartella WebView2 nel kit, nuove finestre, capability | Progetto aperto nell'editor integrato, terminale con l'ambiente del kit, nessuna IPC dalle finestre `editor` |
| 3. Componenti | Pagina Componenti e Primo avvio, `setup_runner.rs` con log in tempo reale e annullamento | Installazione e rimozione di un linguaggio dall'app, coerente con il menu in console |
| 4. Agenti e progetti | Pagine Home, Progetti, Agenti; lancio degli agenti con `sync.js`; estensioni e estensione Dev-Kit nel server | I tre agenti avviati dall'app e dall'editor, istruzioni allineate |
| 5. Rifinitura | Impostazioni, Rimuovi l'unità, avviso di altre copie, aggiornamento dell'app, quattro lingue, accessibilità | Checklist manuale completa |
| 6. Rilascio | Componente `app` negli script di preparazione, documentazione in quattro lingue, release `app-v1.0.0` | Kit scaricato da zero che arriva all'app senza passaggi manuali |

## Cosa non fare

- Non creare installer, servizi, attività pianificate o voci di registro.
- Non riscrivere in Rust la logica di download e verifica degli script, a meno che il piano lo motivi e l'utente lo approvi.
- Non duplicare l'elenco dei componenti, delle estensioni o le regole di allineamento degli agenti: leggi i file del kit.
- Non far partire il server dell'editor su un indirizzo diverso da `127.0.0.1`, senza token o con il token sulla riga di comando.
- Non dare alle finestre `editor` accesso ai comandi Tauri né a plugin.
- Non usare la cartella dati predefinita di WebView2.
- Non rompere `setup\Windows.cmd` né cambiare formati di file condivisi senza compatibilità.
- Non aggiungere telemetria, analisi d'uso o connessioni a servizi diversi da quelli già usati dal kit.
- Non includere segreti, token o percorsi personali nei log, nei test o nei commit.
- Non procedere per ipotesi su API che non hai verificato nella versione usata: controlla, poi annota in `DECISIONS.md`.
