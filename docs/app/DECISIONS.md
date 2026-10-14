# Dev-Kit App: decisioni

Ogni voce: scelta, motivo, alternativa scartata. Stato **proposta** finché il piano (revisione 3) non è approvato.
API verificate il 13-14/10/2026 su docs.rs (`tauri 2.12.0`), v2.tauri.app e le release GitHub citate.

## D-00 Struttura: `Dev-Kit.exe` + `data\` + `projects\`

- **Scelta.** Tutta la logica, i testi, il catalogo dei componenti e le estensioni del kit sono nell'exe. Tutto ciò che si scarica o si genera sta in `data\` accanto all'exe: programmi (`data\apps\win32-<arch>\`), stato, impostazioni, cache, temporanei. I progetti stanno in `projects\` nella radice, accanto all'exe (indicazione dell'utente, 14/10/2026).
- **Motivo.** Indicazione dell'utente (14/10/2026). Un solo file da distribuire e da aggiornare; una sola cartella da salvare.
- **Scartata.** Struttura del Dev-Kit attuale (`setup\`, `apps\`, `projects\`, `data\` nella radice) con script richiamati dall'app.
- **Conseguenze.** Il prompt chiedeva di riusare gli script e di non rompere `setup\Windows.cmd`: superato (PLAN, Q1). `setup\` non si aggiorna più da `main`: si aggiorna solo l'exe.

## D-01 Ambiente: calcolato in Rust

- **Scelta.** `env.rs` produce le stesse variabili di `setup\windows\_env.cmd` (commit `a61a354`), con `CP_APPS = data\apps\win32-<arch>` e le stesse condizioni (per esempio `JAVA_HOME` solo se c'è `java\bin\java.exe`). PATH = cartelle del kit nell'ordine di `_env.cmd` + `System32`, `System32\Wbem`, `System32\WindowsPowerShell\v1.0`. Mai il PATH del PC.
- **Impostazioni di rete** (proxy, `NODE_EXTRA_CA_CERTS`, `GEMINI_API_KEY`, `DEVKIT_LANG`, `DEVKIT_ALLOW_LAN`) in `data\app\settings.json`, modificate dalla pagina Impostazioni; nessun `proxy.cmd` eseguito.
- **Test.** Tabella attesa per ogni variabile; con un Dev-Kit attuale disponibile in CI, confronto con l'esito di `_env.cmd` (differenze ammesse elencate: radice di `apps`, PATH filtrato, `CP_MOVED_FROM`, `PROMPT`).
- **Scartata.** Eseguire `_env.cmd`: nel nuovo kit non c'è.

## D-02 Frontend: React + TypeScript + Tailwind + react-icons

- **Scelta dell'utente** (14/10/2026), al posto di Svelte proposto nella revisione 1.
- **Versioni** (pubblicate da almeno due settimane, fissate in `ui\package.json` e `package-lock.json`): React 19.3.0, Vite 8.3.1, `@vitejs/plugin-react` 6.1.1, Tailwind 4.3.3 con `@tailwindcss/vite`, react-icons 5.7.0, Vitest 5.0.2 + jsdom + Testing Library, ESLint 10 + typescript-eslint 8.71 (`strictTypeChecked`), Prettier 3.9, xterm.js 6.0.0 + `@xterm/addon-fit` 0.11.0 (console, D-44).
- **TypeScript 6.0.3, non 7.** TypeScript 7 (luglio 2026) non è ancora supportato da typescript-eslint (`>=4.8.4 <6.1.0`).
- **Icone**: `react-icons/vsc`, cioè le Codicons di VS Code, per lo stesso linguaggio visivo dell'editor (D-27).

## D-27 Tema VSCodium ovunque

- **Scelta dell'utente** (14/10/2026): il tema di VSCodium per tutto.
- **Interfaccia dell'app**: colori dei temi predefiniti "Dark Modern" e "Light Modern" come variabili CSS (`ui\src\theme.css`), scelti da `prefers-color-scheme` (tema di Windows); struttura a barra delle attività, area di contenuto e barra di stato come VSCodium; carattere Segoe WPC/Segoe UI 13 px; focus visibile con il colore di VSCodium.
- **Editor**: `window.autoDetectColorScheme` attivo, con Dark Modern / Light Modern come temi preferiti (fase 3).
- **Terminali**: colori di xterm.js nelle console dell'app (D-44) e colori di PSReadLine ricavati da `terminal.ansi*` degli stessi temi (fase 3).
- **Da verificare**: i valori in `theme.css` vanno confrontati con i file `dark_modern.json` / `light_modern.json` del server di VSCodium appena installato (fase 3); eventuali differenze si correggono lì.

## D-28 Ambiente dei figli: elenco di variabili consentite

- **Scelta.** I programmi del kit ricevono le variabili del kit più un elenco fisso di variabili di sistema (`SystemRoot`, `ComSpec`, `PATHEXT`, `PROCESSOR_*`, `ProgramFiles*`, `USERNAME`…) e i proxy del PC; mai il resto dell'ambiente dell'utente (per esempio `OneDrive`, `CARGO_HOME`, `JAVA_HOME` del PC). `Command::env_clear()` prima di ogni avvio.
- **Motivo.** Terminale e strumenti esclusivi del kit (indicazione del 14/10): una variabile del PC come `CARGO_HOME` o `PYTHONPATH` farebbe usare programmi o pacchetti fuori dal kit.
- **Scartata.** Ereditare tutto e sovrascrivere solo le variabili del kit, come fa `_env.cmd`.

## D-03 Tauri 2.12.0 fissato

- Ultima 2.x pubblicata da più di due settimane (26/09/2026); 3.0 è alpha. `tauri build --no-bundle`, exe da `target\<triple>\release\`.

## D-04 Il processo dell'app non scrive fuori da `data\`

- All'avvio, prima di qualsiasi thread: `APPDATA`, `LOCALAPPDATA`, `TEMP`, `TMP`, `USERPROFILE` del processo puntati in `data\`; `WEBVIEW2_USER_DATA_FOLDER=data\webview2`; ogni `WebviewWindowBuilder` con `.data_directory(data\webview2)` (API verificata). Nessun uso di `app_data_dir` e simili di Tauri.
- Vincolo dalla documentazione: le WebView con la stessa cartella dati devono avere gli stessi `additional_browser_args` e `scroll_bar_style`: un'unica configurazione per tutte.

## D-05 WebView2 senza installarlo

- `GetAvailableCoreWebView2BrowserVersionString` (crate `webview2-com`, già usato da `wry`). Se manca il runtime di sistema e c'è `data\apps\win32-<arch>\webview2\msedgewebview2.exe` → `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`. Altrimenti `TaskDialogIndirect` con il link per scaricarlo e la cartella dove metterlo, poi uscita. Mai il bootstrapper.
- Facoltativo (Q nel piano di fase 2): pulsante che scarica la "Fixed Version" in `data\apps\…\webview2\` dalla finestra di dialogo, con verifica della firma Microsoft.

## D-06 Nuove finestre e navigazione dell'editor

- API verificate: `on_new_window(Fn(Url, NewWindowFeatures) -> NewWindowResponse)`, `on_navigation(Fn(&Url) -> bool)`.
- `on_navigation`: solo `http://127.0.0.1:<porta>`. `on_new_window`: stessa origine → rifiuto e nuova finestra `editor` creata dal nostro costruttore; altri `http`/`https` → browser di sistema; altri schemi → bloccati.

## D-07 Job Object: l'app entra nel job da sé

- `KILL_ON_JOB_CLOSE | BREAKAWAY_OK`, handle non ereditabile, `AssignProcessToJobObject(GetCurrentProcess())`: ogni figlio nasce nel job. Sotto-job annidati per annullare una singola operazione.
- Browser ed Esplora risorse con `CREATE_BREAKAWAY_FROM_JOB`, altrimenti morirebbero con l'app.
- Scartato: avvio sospeso + assegnazione per ogni processo (`std::process::Child` non espone l'handle del thread).

## D-08 Istanza singola per kit, senza plugin

- Mutex `Local\Dev-Kit-App-<sha256(radice minuscola)[0:16]>` e pipe omonima per "porta in primo piano".
- Scartato `tauri-plugin-single-instance`: identificatore fisso, due chiavette diverse si bloccherebbero a vicenda.

## D-09 Altre copie del kit aperte

- `CreateToolhelp32Snapshot` + `QueryFullProcessImageNameW`: processo di un altro kit se il percorso sta sotto `<radice>\data\apps\win32-*\` o è un altro `Dev-Kit.exe`, con radice diversa. Regola equivalente a `Test-Sessions.ps1:59-73`.

## D-10 Scorciatoie e menu del browser nell'editor

- Tauri non li espone. `with_webview` → `ICoreWebView2Settings3::SetAreBrowserAcceleratorKeysEnabled(false)`, `SetAreDefaultContextMenusEnabled(false)`; `zoom_hotkeys_enabled(false)`. Solo sulle finestre `editor`.

## D-11 Motore di installazione in Rust

- Stessi passi e controlli degli script (`Prepare-DevKit.ps1` e helper): verifica prima di sostituire, componente in uso lasciato intatto, un componente fallito non ferma gli altri, 3 tentativi per download. Una preparazione alla volta (mutex per kit). Annullamento al termine del passo in corso; "Ferma subito" mai durante uno scambio di cartelle.
- La rimozione copre **tutti** i componenti (gli script dimenticano Typst e TinyTeX).

## D-12 Rilevatori dei progetti in Rust

- Porting di `core/detectors` (Node, PHP, Python, Rust, Go, statico; priorità desktop > web > comando; `panel.json`). Usati solo dall'app (editor pulito, Q4 = a): test su progetti di esempio presi dai casi del Dev-Kit.

## D-13 Console fuori dall'editor

- Finestre dell'app con un terminale integrato (D-44): pseudo console avviata da noi (quindi nel job), xterm.js nella finestra (D-45, D-46). Ripiego `conhost` con `CREATE_NEW_CONSOLE` dove la pseudo console non parte (Windows 10 prima della 1809).
- Prima (fino al 14/10/2026): Windows Terminal portabile del kit, `WindowsTerminal.exe --window new -d <cartella> -- <comando>`; sostituito perché esiste solo per Windows (D-44).

## D-14 Comandi solo per la finestra `main`

- Verificato: in Tauri 2 i comandi registrati sono permessi a **tutte** le finestre salvo `AppManifest::commands` in `build.rs`; le origini remote non hanno accesso salvo `remote.urls`.
- (1) `AppManifest::commands`; (2) una capability `windows: ["main"]`, nessun `remote`; (3) controllo `label == "main"` in ogni comando; (4) test con `tauri::test`: invocazione da una WebView `editor` su `http://127.0.0.1` rifiutata.

## D-15 Plugin

- `tauri-plugin-dialog` (scelta cartelle) e `tauri-plugin-opener` usati solo da Rust. Nessun `tauri-plugin-shell`, nessun `single-instance`.

## D-16 Catalogo dei componenti incorporato

- Il catalogo è un file JSON nel codice sorgente, incorporato con `include_str!` e validato da un test (id unici, `needs` esistenti, gruppi validi). Id liberi (il vincolo `^[a-z]+$` era degli script): `vscodium-server`, `pwsh`, `webview2` (`terminal`, Windows Terminal, tolto il 14/10/2026: D-44).
- Scelta in `data\app\components.txt` (formato `id=1|0` del Dev-Kit: semplice e leggibile a mano); versioni in `data\app\versions\<id>.txt`.

## D-17 `cmd.exe` solo dove serve

- Il motore non usa `cmd`. Restano solo il profilo "Dev-Kit - cmd" del terminale e gli strumenti installati come `.cmd`/`.bat` (per esempio `mvn.cmd`, `gradle.bat`): avvio con `raw_arg` da una sola funzione testata con `a b`, `(x)`, `a&b`, `50%`, `a!b`.

## D-18 Testi

- Sorgente in italiano, cataloghi it/en/fr/de nel codice, incorporati. Test di build: ogni testo tradotto in tutte le lingue, stessi segnaposto `{n}`, nessuna traduzione vuota (stesse regole di `setup/lang/build.js`).

## D-19 Toolchain

- CI: `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`. Compilazione con il Rust `gnu` del Dev-Kit: da verificare in fase 1.

## D-20 Aggiornamento dell'exe

- `data\app\update\`, SHA-256 dal file `.sha256` della release; alla chiusura `Dev-Kit.exe` → `Dev-Kit.old.exe` (Windows permette di rinominare un exe in esecuzione), nuovo al suo posto; eliminazione al primo avvio successivo. Disattivabile nelle impostazioni. Fonte delle release: Q3.

## D-21 Riga di comando nello stesso exe

- `Dev-Kit.exe env | prepare | sync | agent <id> [cartella] | eject | sessions`. Exe GUI che per la riga di comando si aggancia alla console del genitore (`AttachConsole(ATTACH_PARENT_PROCESS)`).
- Usi: profili del terminale per gli agenti (allineamento prima e dopo, come `_agent.cmd`), test di integrazione, scambio dell'exe in aggiornamento.

## D-22 Dipendenze Rust del motore

| Crate (versione) | Uso | Scartata |
| --- | --- | --- |
| `ureq` 3.4.2 con `native-tls` (SChannel), `json`, `gzip`, `win-system-proxy` | HTTPS con il TLS di Windows e i certificati del suo archivio (`RootCerts::PlatformVerifier`: senza, ureq usa le radici Mozilla e l'ispezione SSL aziendale fallirebbe); proxy di sistema letto dal registro (sola lettura) | `rustls` + `rustls-platform-verifier`: richiede `ring`, cioè clang per compilare su ARM64; `reqwest`: più pesante per download sequenziali |
| `sha2` 0.11, `base64` 0.23 | SHA-256/512, integrità npm | — |
| `zip` 8.6 (solo `deflate`), `tar` 0.4.46, `flate2` 1.1 | estrazione | `tar.exe` di sistema: processo esterno |
| `sevenz-rust2` 0.23 | Git Portable `.7z.exe`: si cerca la firma 7z dopo lo stub dell'eseguibile e si estrae da lì | lanciare l'SFX: apre la finestra di 7-Zip |
| `toml_edit` | (fase 5) blocchi del kit in `config.toml` di Codex senza toccare il resto | riscrittura a testo: perde la formattazione dell'utente |
| `regex` 1.13 | nomi delle asset delle release | — |
| `windows` 0.62 | job, mutex, pipe, credenziali, processi, finestre di messaggio | `windows-sys`: senza wrapper COM, che servono per WebView2 |
| `portable-pty` 0.9.0 | pseudo console delle finestre console (D-44, D-45): ConPTY su Windows (con `conpty.dll` affiancato se c'è, D-47), pty su macOS e Linux | ConPTY scritto a mano con `windows`: solo Windows |

Versioni pubblicate da almeno due settimane, fissate in `Cargo.lock`.

## D-29 Regole del motore di installazione (fase 2)

- **Proxy**: impostazioni dell'app → `HTTPS_PROXY`/`HTTP_PROXY` → proxy di Windows (chiave `Internet Settings` di HKCU, sola lettura). I PAC (configurazione automatica) non sono supportati: chi li usa imposta il proxy nella pagina Impostazioni.
- **Token GitHub**: solo se è nell'ambiente (`GITHUB_TOKEN`/`GH_TOKEN`, cioè nella CI), solo verso `api.github.com`. L'app non lo chiede e non lo salva.
- **Estrazione**: voci con percorsi assoluti, `..` o unità refuse; collegamenti simbolici ignorati; date dei file non ripristinate (exFAT).
- **Scambio**: la cartella vecchia viene rinominata prima di tutto; se Windows rifiuta (un file in uso) non cambia nulla e il componente risulta "in uso". Verificato con PowerShell aperto.
- **Git**: `post-install.bat` di Git Portable eseguito nella sua cartella con l'ambiente del kit; riuscito se lo script si è cancellato (il suo codice di uscita è sempre 1, perché cmd non trova più il file che stava eseguendo).
- **`-Skip` sugli obbligatori**: ignorato, come negli script del kit.
- **Id con trattino**: ammessi (`vscodium-server`), perché il catalogo è dell'app e non viene letto dagli script del Dev-Kit.
- **Annullamento**: immediato durante un download (il file parziale viene cancellato); altrimenti alla fine del passo in corso.

## D-23 Terminale esclusivo del kit

- **Scelta.** Shell predefinita nell'editor: PowerShell 7 portabile, `-NoProfile` + profilo del kit in `data\kit\`; `PSModulePath` ristretto al kit; `POWERSHELL_TELEMETRY_OPTOUT=1`, `POWERSHELL_UPDATECHECK=Off`. Git Bash e cmd come profili aggiuntivi. Console esterne: finestre dell'app con terminale integrato (D-44).
- **Scartate.** Windows PowerShell 5.1 di sistema: carica moduli e profili del PC. Nushell: estranea agli agenti, che si aspettano PowerShell o Bash. Windows Terminal di sistema: pacchetto Store, fuori dal job, impostazioni in `%LOCALAPPDATA%`.
- **Verificato il 14/10/2026.** PowerShell 7.6.7: `PowerShell-7.6.7-win-x64.zip` / `-win-arm64.zip`, SHA-256 in `hashes.sha256` e nel `digest` delle asset.

## D-24 Nessuna richiesta di amministratore

- Manifesto incorporato con `requestedExecutionLevel level="asInvoker" uiAccess="false"` (senza, Windows applica il rilevamento degli installer, e un motore di installazione lo rischierebbe). Test in CI che legge il manifesto dall'exe compilato. Nessuna scrittura nel registro, nessun servizio, nessuna attività pianificata.

## D-25 File che i programmi vogliono su disco

- `loopback.js` (per `NODE_OPTIONS=--require`), profilo PowerShell, `bashrc`: incorporati nell'exe, scritti in `data\kit\` al primo avvio di ogni versione (confronto di un hash), mai modificati dall'utente.

## D-26 Server web dell'editor

- Release VSCodium `vscodium-reh-web-win32-x64-<versione>.tar.gz` + `.sha256` (verificato: 1.140.06880). Nessun pacchetto ARM64: su ARM64 si usa x64 in emulazione, con avviso nell'interfaccia.

## D-27 Tema VSCodium ovunque

- Finestra dell'app con i colori di VSCodium "Default Dark Modern" / "Default Light Modern" (variabili CSS in `ui/src/theme.css`, scelte da `prefers-color-scheme`) e icone Codicons (`react-icons/vsc`). Editor: `window.autoDetectColorScheme` con gli stessi due temi. Console (D-44): i colori del terminale di VSCodium in xterm.js (`ui/src/terminal.ts`), scelti da `prefers-color-scheme`.

## D-28 Interfaccia: React 19 + TypeScript + Tailwind 4 + react-icons

- Indicazione dell'utente (14/10/2026); sostituisce Svelte di D-02. Versioni pubblicate da almeno due settimane, fissate in `package-lock.json`.

## D-30 Server dell'editor avviato senza `cmd.exe`

- `bin\codium-server.cmd` esegue solo `node.exe out\server-main.js <argomenti>`: l'app avvia direttamente quel comando, con gli argomenti a vettore. Così i percorsi con `&`, `%`, `!` o parentesi non passano mai da `cmd`.
- Il server parte sospeso, entra in un Job Object suo (annidato in quello dell'app) e poi riprende: anche l'extension host e i terminali stanno nel job, e fermare il server chiude tutto l'albero (`jobs::spawn_in`). Verificato da `editor::tests::real_server_token_redirect_and_stop`.

## D-31 Impostazioni dell'editor in `data\vscodium-server\data\Machine\settings.json`

- Sono le impostazioni "Remote" del server, che il client web applica a ogni finestra; quelle dell'utente restano sue (nel browser). Verificato: con i profili scritti lì, "Terminal: Create New Terminal" apre il profilo "Dev-Kit" (PowerShell 7 del kit, `USERPROFILE` in `data\home`).
- Profili di sistema (`PowerShell`, `Windows PowerShell`, `Command Prompt`, `Git Bash`, `JavaScript Debug Terminal`) messi a `null`; `automationProfile.windows` = PowerShell del kit, così anche attività e debug usano la shell del kit.

## D-32 `freezePrototype` solo nella finestra dell'app

- Tauri inietta `freezePrototype` in tutte le WebView, finestre dell'editor comprese, e VSCodium con `Object.prototype` congelato resta a pagina bianca. L'opzione è spenta in `tauri.conf.json` e la finestra dell'app congela il prototipo da sé (`ui/src/freeze.ts`, primo modulo importato).

## D-33 Limite noto: cartelle del kit che iniziano con una cifra, `x` o `u`

- VSCodium passa a `@parcel/watcher` esclusioni con percorsi Windows assoluti (`<kit>\data\vscodium-server\extensions/**`), che il watcher compila come espressioni regolari: dopo `\`, una cifra è un riferimento all'indietro e `x`/`u` iniziano escape esadecimali. Con il kit in `D:\2026\kit` il controllo dei file della cartella aperta fallisce (`regex_error`) e l'editor non vede i cambiamenti fatti fuori (per esempio dagli agenti). Parentesi, spazi e `&` non danno problemi (verificato con il watcher incluso in VSCodium 1.140).
- Passare `--extensions-dir` con le barre normali non serve: VSCodium lo normalizza. L'app riconosce questi percorsi (`kit::watcher_unsafe_folder`) e lo dice nella Home, con il suggerimento di rinominare la cartella.

## D-34 PowerShell del kit isolato dal profilo del PC

- .NET chiede le cartelle a Windows e ignora `APPDATA`/`LOCALAPPDATA`: pwsh scriverebbe `StartupProfileData-*` in `%LOCALAPPDATA%\Microsoft\PowerShell` e la cronologia in `%APPDATA%`. Rimedi verificati: `DOTNET_MultiCoreJitNoProfileGather=1` nell'ambiente (nessun file di profilo JIT) e `Set-PSReadLineOption -HistorySavePath` nel profilo del kit (`kit-files/profile.ps1`).
- Resta fuori dal kit solo la cartella vuota `%LOCALAPPDATA%\Microsoft\PowerShell` se non esiste già: pwsh la crea sempre all'avvio (codice di PowerShell, nessuna opzione per evitarlo). Eccezione documentata.
- `PSModulePath` limitato ai moduli di pwsh e a `data\pwsh\Modules`: i moduli in Documenti e Program Files del PC non vengono caricati.

## D-35 Mai una shell del PC

- Prima di aprire editor o console, l'app installa (con la stessa preparazione della pagina "Linguaggi e strumenti", `SetupTask`) i componenti obbligatori mancanti e attende la fine; l'avanzamento compare in Home e nella barra di stato.
- Se PowerShell 7 del kit manca comunque, la console non si apre e l'errore lo dice: nessun ripiego su `cmd.exe` o Windows PowerShell.

## D-36 Estensioni consigliate in background dopo l'avvio dell'editor

- Elenco del Dev-Kit (`src-tauri/catalog/extensions.txt`, 60 estensioni) più `data\app\extensions.txt` dell'utente; `@programma` = solo se quel componente è nel kit; proposta una volta sola (`data\app\extensions-offered.txt`); una che fallisce viene ritentata all'avvio successivo; CodeLLDB dalla release GitHub (SHA-256).
- Installate con la riga di comando del server (`--install-extension`, Open VSX) **dopo** l'avvio dell'editor, non prima come chiedeva il prompt: misurate circa 35 s per estensione al primo giro, l'editor non può aspettare minuti. Le estensioni compaiono ricaricando la finestra; lo stato è in `editor_status.extensions`.
- Scartato: disattivare le estensioni di un programma tolto (il Dev-Kit lo faceva scrivendo nel database SQLite di VSCodium). Restano installate: non fanno danni e si riattivano da sole se il programma torna.

## D-37 WebView2 "Fixed Version" non è un componente

- Microsoft la pubblica solo da una pagina web, con indirizzi che cambiano e senza checksum: non c'è una fonte da cui scaricarla e verificarla come gli altri componenti. Resta il supporto all'avvio (D-05): se manca il WebView2 di sistema, la finestra di messaggio spiega dove scaricarla e in quale cartella del kit estrarla.

## D-38 Motore di installazione: parti, firme, processi

- `Release` = una o più parti (Java + Maven + Gradle, CMake + Ninja, PHP + Composer), ognuna con destinazione: cartella propria, altra cartella, file in `tools\`, file singolo sostituito (Claude, Antigravity: il vecchio exe rinominato `.old`, così si aggiorna anche se l'agente è aperto). La versione salvata unisce le versioni delle parti.
- Firma Authenticode (`signature.rs`, `WinVerifyTrust` + soggetto del firmatario): Claude Code (`Anthropic`, revoca non verificabile = avviso), runtime VC++ (`O=Microsoft Corporation`, obbligatoria sull'exe e sulla DLL).
- Installatori fatti di processi (`custom`): Rust (`rustup-init`, poi `rustup update` sul posto per non perdere crate e `cargo install`; voce "Rustup" in HKCU ripristinata com'era: verificato con il rustup dell'utente già presente), pnpm/yarn (`node npm-cli.js`, senza `cmd.exe`), R (micromamba con profilo nell'area di lavoro).
- File `.bat`/`.cmd` (verifica di Kotlin): `jobs::batch_command`, `cmd /d /s /c ""percorso" argomenti"` con `raw_arg`; trovato in prova che `cmd /c` con un percorso tra virgolette e `&` si rompe.
- Limite: Rust su un kit FAT32/exFAT non si installa (rustup richiede i collegamenti fisici); il Dev-Kit installava la toolchain sul disco del PC e copiava solo `rustup\`. Da portare (vedi CHANGELOG).

## D-39 Verifica finale

- `checkup.rs` (`Test-Installation` del kit): ogni componente installato risponde al suo comando di versione (campo `check` del catalogo), con l'ambiente del kit e 60 s di limite. `Dev-Kit.exe check`; dopo `prepare` per ciò che è stato installato.

## D-40 Allineamento degli agenti in Rust

- Porting completo di `setup/agents/*.js` (`agents_sync/`): istruzioni (copia a tre vie con stato e conflitti), skill (giunzioni NTFS con il crate `junction`, copie su FAT32/exFAT), MCP (Claude ↔ `mcp.json`, Codex `config.toml` a blocchi di testo, Antigravity solo i server del kit), regola del kit, cartelle attendibili, skill di progetto. Stessi file di stato e stessi marcatori del Dev-Kit.
- Miglioria: due passaggi di collegamento delle skill, così una skill nata in un agente arriva a tutti nello stesso allineamento (nel Dev-Kit serviva il giro successivo).
- Messaggi in inglese nel log (`data\app\logs\app.log`); l'interfaccia mostra lo stato, non i dettagli.
- Quando: all'avvio dell'app, prima di ogni agente lanciato dall'app e quando un agente del kit termina, ovunque sia stato aperto (sorvegliante ogni 3 s). Scartato: il wrapper `claude.cmd`/`codex.cmd` nel PATH, che avrebbe aggiunto la domanda "Terminare il processo batch?" di cmd a ogni Ctrl+C.

## D-41 Accesso di Antigravity: `agy.cmd` del kit e sorvegliante dell'app

- `data\kit\bin\agy.cmd` (primo nel PATH, anche per Git Bash con `bin\agy`): `Dev-Kit.exe keyring restore`, `agy.exe`, `Dev-Kit.exe keyring save`. In un file batch `cmd` aspetta la fine anche di un programma grafico. `agy.exe` non è nel PATH: non si scavalca il wrapper.
- Il processo "watch" nascosto del Dev-Kit è sostituito dal sorvegliante dell'app; inoltre il `save` a fine esecuzione toglie il segnale "in avvio" quando nessun agy del kit gira (trovato in prova: con un agy brevissimo il login restava in Windows fino al sorvegliante).
- Verificato con il Gestore credenziali reale: persistenza "solo sessione" al restore; dopo il save la credenziale del kit è sparita da Windows e il token aggiornato è nel kit; la credenziale `gemini:antigravity` dell'Antigravity installato sul PC non viene toccata.

## D-42 `claudeCode.environmentVariables` non serve nel server web

- Nel Dev-Kit l'estensione Dev-Kit le scriveva perché VSCodium desktop poteva partire senza l'ambiente del kit. Nel server web l'extension host eredita l'ambiente del server, che è quello del kit: l'estensione Claude Code vede già `CLAUDE_CONFIG_DIR`, `DISABLE_UPDATES`, `NODE_OPTIONS`, `CLAUDE_CODE_GIT_BASH_PATH`.

## D-43 Rilocazione

- Porting di `moved.js` (`relocate.rs`) sulla nuova struttura: radice precedente da `data\app\old-root.txt` (scritto da `kit.rs`), consumato a lavoro fatto; tra i file aggiornati anche impostazioni ed estensioni del server dell'editor. Verificato rinominando un kit con tutti i componenti: chiavi di `.claude.json` unite sotto il nuovo percorso, `config.toml` senza il vecchio percorso, giunzioni delle skill rifatte.

## D-44 Terminale integrato nell'app

- **Scelta dell'utente** (14/10/2026): un terminale disponibile per tutti i sistemi operativi; tra un terminale esterno multipiattaforma e uno dentro l'app, l'utente ha scelto **dentro l'app**. Sostituisce Windows Terminal (D-13, D-23), che esiste solo per Windows ed era un eseguibile in più da scaricare.
- **Come.** `terminal.rs`: il processo (PowerShell del kit, o un agente) parte su una pseudo console di `portable-pty` 0.9.0 (ConPTY su Windows, pty su macOS e Linux; D-22) con il solo ambiente del kit; una finestra `terminal-<n>` dell'app carica `terminal.html` (xterm.js 6.0.0 + `@xterm/addon-fit` 0.11.0, D-28) e la mostra. Stesso motore del terminale di VSCodium (xterm.js + PTY).
- **Scartate.** WezTerm (multipiattaforma, portabile, configurazione Lua): resta un eseguibile esterno da scaricare e configurare, con le sue regole di ambiente. Alacritty: niente schede. Kitty: niente Windows. Ghostty: su Windows non maturo. Windows Terminal: solo Windows.
- **Portata.** Il codice del terminale (`portable-pty`, xterm.js, finestre Tauri) è multipiattaforma; il resto dell'app usa ancora API di Windows (job, `windows`, WebView2, percorsi `win32-*`): il porting su macOS e Linux resta da fare, ma il terminale non lo ostacola.
- **Il terminale di VSCodium resta il suo.** L'editor ha già un terminale integrato (xterm.js nella pagina, PTY nel server) sulle shell del kit (D-23); la console dell'app non lo sostituisce e non vi si può innestare, perché il PTY dell'editor è interno al suo server. Stesse shell, stesso profilo, stessi colori: i due terminali si comportano allo stesso modo.
- **Kit esistenti.** La cartella `data\apps\<piattaforma>\terminal\` di Windows Terminal non è più usata e si può cancellare.

## D-45 Console: processo e pseudo console

- **Avvio.** `CommandBuilder` con `env_clear` + ambiente del kit, cartella del progetto. Un `.cmd` (Antigravity, `agy.cmd`) passa dalla PowerShell del kit (`-Command "& '<percorso>'; exit $LASTEXITCODE"`, apici raddoppiati): `cmd` leggerebbe `&` e `%` nel percorso, come in D-17. Senza la PowerShell del kit, nessuna console (mai una shell del PC).
- **Job.** Il processo nasce nel job dell'app (l'app ci sta dentro, D-07); se l'app non è nel job, `jobs::assign_pid` lo assegna subito dopo l'avvio (i suoi primi figli potrebbero sfuggire: limite di quel caso raro).
- **ConPTY chiede la posizione del cursore** (`ESC[6n`) all'avvio, per l'opzione `INHERIT_CURSOR` di `portable-pty`, e non fa partire il programma finché non riceve risposta: la prima risposta (`ESC[1;1R`) la dà l'app, così il programma parte anche prima che la finestra si colleghi. Le domande di posizione non entrano nell'output conservato, altrimenti una finestra ricaricata risponderebbe a domande vecchie scrivendo testo nel programma.
- **Ctrl+C.** Il flag "ignora Ctrl+C" passa dal processo ai figli: l'app avviata in un nuovo gruppo di processi (da uno script, da un altro programma) lo darebbe alla shell, e Ctrl+C non fermerebbe più i comandi (verificato: `ping -t` non si fermava). `SetConsoleCtrlHandler(None, false)` prima di ogni console.
- **Senza ConPTY** (Windows 10 prima della 1809) `portable-pty` va in panic invece di restituire un errore: `catch_unwind`, poi ripiego sulla console classica (`CREATE_NEW_CONSOLE`) con lo stesso programma.
- **Output.** Un thread legge la pseudo console e manda i byte alla finestra con un `Channel` di Tauri (`InvokeResponseBody::Raw`). Tauri consegna i messaggi sotto 1 KB con `eval` e quelli più grandi con il comando interno `plugin:__TAURI_CHANNEL__|fetch` (permesso anche alle finestre console; verificato), in ordine. Gli ultimi 512 KB restano in memoria per una finestra che si collega tardi o si ricarica.
- **Input.** La pagina manda i tasti uno alla volta in ordine (una chiamata `terminal_input` per volta, i tasti battuti nel frattempo insieme): chiamate IPC parallele potrebbero arrivare in disordine.
- **Fine.** Quando il processo termina, l'app chiude la pseudo console (così la lettura finisce), attende l'ultimo output, manda il codice di uscita; la finestra scrive "Processo terminato (codice N)" e si chiude al primo tasto. Chiudere la finestra termina il processo (`Destroyed` → `Terminals::close`). Per gli agenti, alla fine si riallineano (D-40).

## D-46 Finestre console: sicurezza e tasti

- **Capability `terminal`** per le finestre `terminal-*` (D-14): solo `terminal_attach`, `terminal_input`, `terminal_resize`, più `core:window:allow-set-title` e `allow-close` della propria finestra. Ogni comando trova la sessione dall'etichetta della finestra che chiama: una finestra raggiunge solo il suo processo. La finestra principale e quelle dell'editor non possono usare questi comandi; le console non possono usare quelli della finestra principale. Test `console_windows_reach_only_their_session`; verificato anche nell'app (`app_info` e `terminal_open` rifiutati da una console).
- **Pagina separata** (`ui/terminal.html`, `ui/src/terminal.ts`, seconda voce di `build.rolldownOptions.input`): la console non carica l'interfaccia principale.
- **`Object.prototype` non congelato** nella console: xterm.js assegna `toString` ai suoi oggetti e con `freeze.ts` non si carica (verificato: "Cannot assign to read only property 'toString'"). Rischio basso: la pagina non legge dati esterni, mostra solo i byte del proprio processo come testo.
- **WebView2** come le finestre dell'editor (`editor_window::quiet_webview`): tasti del browser (F5, Ctrl+F, Ctrl+P), menu contestuale e barra di stato spenti, così i tasti arrivano alla shell; nuove finestre negate.
- **Tasti come Windows Terminal.** Ctrl+C copia se c'è una selezione, altrimenti va al programma; Ctrl+V e Ctrl+Maiusc+V incollano; Ctrl+Maiusc+C copia; clic destro copia la selezione o incolla. Titolo della finestra dal programma (OSC 0/2), con "- Dev-Kit".
- **Colori** del terminale di VSCodium "Dark Modern" / "Light Modern" (gli stessi degli schemi che si scrivevano per Windows Terminal), scelti da `prefers-color-scheme` e aggiornati quando cambia il tema di Windows (D-27). Carattere Cascadia Mono / Consolas, 14 px.

## D-47 ConPTY aggiornato (aperta)

- **Problema.** Con il ConPTY di Windows, Claude Code si apre e si disegna ma non riceve i tasti (né Invio né frecce; Ctrl+C arriva solo come segnale). PowerShell e Node in modalità raw li ricevono. All'avvio Claude interroga il terminale: XTVERSION (`ESC[>0q`), protocollo tastiera kitty (`ESC[?u`), colore di sfondo (OSC 11); la conhost di sistema chiede anche `ESC[?9001h` (win32-input-mode). Ipotesi: la conhost di sistema non fa arrivare a Claude le risposte, e Claude resta in attesa.
- **Rimedio previsto.** Il ConPTY aggiornato di Microsoft, come fanno Windows Terminal, VS Code e WezTerm: `conpty.dll` + `OpenConsole.exe` dal pacchetto NuGet ufficiale `Microsoft.Windows.Console.ConPTY` (firmati Microsoft, verificato con Authenticode). `portable-pty` carica `conpty.dll` con `LoadLibrary` se lo trova, e `conpty.dll` avvia `OpenConsole.exe` dalla propria cartella.
- **Versione.** 1.24.260710001 (13/07/2026, x64 e ARM64): la stabile più recente, 1.25.260930003, è del 02/10/2026, meno di due settimane (regola delle versioni di D-22 e D-28).
- **Come nel kit.** Il kit non mette file accanto all'exe (`Dev-Kit.exe` + `data\`): componente obbligatorio `conpty` in `data\apps\<piattaforma>\conpty\` (circa 1,2 MB), SHA-256 dal catalogo NuGet e firma Microsoft; prima della prima console, `SetDllDirectoryW` su quella cartella, così `LoadLibrary("conpty.dll")` la trova. Senza il componente, ConPTY di Windows (comportamento attuale).
- **Stato** (14/10/2026): pacchetto scaricato e firme verificate; la prova con l'app è ferma perché il sistema dei permessi di Claude Code ha bloccato l'esecuzione di binari scaricati: serve il permesso dell'utente (PLAN.md §12).

## D-48 Prove dell'app con il debug remoto di WebView2

- Per provare le finestre senza mani: `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9333` all'avvio dell'app di debug (con Vite su 5173), poi uno script Node che usa il protocollo DevTools (`/json/list`, `Runtime.evaluate`) per chiamare i comandi come la pagina (`window.__TAURI_INTERNALS__.invoke`) e leggere lo schermo di xterm (`.xterm-rows`). Le finestre si riconoscono dall'etichetta (`__TAURI_INTERNALS__.metadata.currentWindow.label`), non dall'URL: le console hanno tutte lo stesso.
- Solo per lo sviluppo: la variabile non è mai impostata dall'app. Le prove passano dalle stesse capability della pagina, quindi verificano anche i permessi.
- Più sessioni di lavoro sullo stesso PC: i processi di prova si chiudono per PID, mai per nome (`taskkill /IM Dev-Kit.exe` chiude anche le app degli altri).

## D-50 Progetti: rilevamento e avvio in Rust

- Porting di `core/detectors`, `core/processes.js`, `core/panel/ports.js` e `site.js` del Dev-Kit (`detect/`, `run.rs`, `projects.rs`). Restano le stesse regole: priorità desktop > web > comando, progetti composti (`frontend\`, `backend\`), `panel.json` e il vecchio `pannello.json` con le chiavi italiane e `{porta}`, argomenti di porta per Vite/Next/Nuxt/Angular/Astro e gli altri.
- Le azioni girano con `cmd.exe /d /s /c` e l'ambiente del kit, come nel Dev-Kit (`shell: true`). PowerShell è stato scartato: `npm.ps1` consuma il `--` di `npm run dev -- --port`. `cmd.exe` serve solo da interprete delle righe di comando dei progetti: il terminale dell'utente resta la PowerShell del kit.
- Ogni azione gira in un job annidato in quello dell'app. «Ferma» chiude l'handle del job (`KILL_ON_JOB_CLOSE`): finisce tutto l'albero (npm → node → esbuild) senza `taskkill`, e chiudendo l'app si chiude tutto.
- Porte 7100-7999 ricordate per azione in `data\app\ports.json`, così segnalibri, cookie e localStorage restano validi. La porta scelta dall'utente è bloccata: se è occupata l'avvio fallisce, non si sposta. Una porta è libera se nessuno risponde e se ci si può mettere in ascolto su `127.0.0.1` e su `::1`.
- I comandi di `panel.json` sono comandi arbitrari del progetto. Partono solo con un clic su «Avvia», come nel Dev-Kit; la pagina ricorda che le cartelle di `projects` sono attendibili (D-53).
- Clonazione: con il Git del kit (`git clone -- <url> <cartella>`, `GIT_TERMINAL_PROMPT=0`), accettando solo `owner/repo` di GitHub o indirizzi `https://` senza caratteri speciali. Niente `ssh://`, `file://` né argomenti che iniziano con `-`. I repository privati usano l'accesso di `gh` del kit.

## D-51 Server di file nell'exe

- `Dev-Kit.exe serve <cartella> <porta>` sostituisce `static-server.js`: niente Node per un sito statico. Ascolta solo su `127.0.0.1` e risponde solo a un `Host` locale (contro il DNS rebinding). Nega i file e le cartelle nascosti (eccetto `.well-known`), `..` e i collegamenti che portano fuori dalla cartella. Una cartella senza `/` finale viene reindirizzata e ogni risposta ha `Cache-Control: no-store`.

## D-52 Aggiornamento dell'exe

- Implementa D-20 (`self_update.rs`). La fonte sono le release `app-vX.Y.Z` di `nabre/dev-kit2` (non bozze, non pre-release) con l'asset `Dev-Kit-x64.exe` o `Dev-Kit-arm64.exe`. Lo SHA-256 viene dal digest dell'API di GitHub o dal file `.sha256` della release; il workflow della fase 7 deve pubblicarli con questi nomi.
- Il controllo parte 20 s dopo l'avvio, se «Aggiornamenti automatici» è attivo e `DEVKIT_NO_UPDATE` è assente; altrimenti si avvia da Impostazioni. Il file scaricato va in `data\app\update\` e solo un file completo e verificato viene registrato in `update.json`.
- Lo scambio avviene su `RunEvent::Exit` (in Tauri `run()` può non tornare): l'hash viene ricontrollato, poi `Dev-Kit.exe` diventa `Dev-Kit.old.exe` e se lo spostamento fallisce il vecchio torna al suo posto. `Dev-Kit.old.exe` viene eliminato all'avvio successivo; se è ancora in uso da un `sync` o `keyring`, si riprova la volta dopo.
- La versione dell'app è scritta in `data\app\versions\app.txt`, come quella degli altri componenti.

## D-53 Cartelle attendibili e sicurezza dei progetti (revisione di sicurezza)

- Le sottocartelle di `projects` sono attendibili per Claude e Codex (comportamento del Dev-Kit, `trust.js`), così gli agenti non chiedono conferma a ogni progetto. È un rischio accettato: un progetto clonato da fonte sconosciuta ha subito i permessi di un progetto fidato. Lo ricordano la pagina Progetti e Linguaggi e strumenti al primo avvio. Scartato: l'attendibilità progetto per progetto, che il Dev-Kit aveva tolto perché gli agenti la chiedevano di continuo.
- `keyring-before.json` contiene per pochi istanti le credenziali di Antigravity già presenti nel PC, per rimetterle com'erano dopo `agy`. Ora i loro segreti sono cifrati con DPAPI per l'utente di Windows (`CryptProtectData`, senza diritti di amministratore): una chiavetta persa o un file copiato non li rivela. Il file viene comunque cancellato al salvataggio successivo.
- Windows Terminal interpretava `;` negli argomenti come separatore di comandi (una cartella chiamata `a; new-tab cmd` avrebbe aperto un altro programma). Il rischio è sparito con il terminale integrato (D-44).

## D-54 Impostazioni

- La pagina Impostazioni scrive `data\app\settings.json` dopo aver controllato i valori: lingua, proxy `http(s)://`, file PEM esistente, Python `3.x`, PHP `8.x`, toolchain `gnu|msvc`. Un file illeggibile non viene sovrascritto finché l'utente non salva.
- La chiave di Gemini non torna mai all'interfaccia: questa riceve solo «salvata sì/no». `null` la conserva, `""` la toglie.
- Al salvataggio l'ambiente del kit viene ricostruito dall'ambiente reale catturato all'avvio (`Host`): valgono subito per console, agenti, installazioni e progetti. Il server dell'editor già avviato tiene il suo ambiente fino al riavvio, e la pagina lo dice.

## D-55 Chiusura e «Rimuovi l'unità»

- Chiudendo la finestra con editor, console, progetti avviati o un'installazione in corso, l'app chiede conferma (`close-requested`) ed elenca cosa si chiude. Senza nulla aperto si chiude subito.
- «Rimuovi l'unità» (porting di `Eject.ps1`) ferma installazione, progetti, finestre di editor e console, server dell'editor e ogni programma del kit (cime degli alberi di processi con l'eseguibile nel kit). Poi salva l'accesso di Antigravity e chiede di chiudere l'app, che tiene aperto il proprio exe. Non trova i programmi del PC che hanno solo un file del kit aperto: il Dev-Kit li cercava nella riga di comando, che senza WMI non si legge. In quel caso Windows dirà «dispositivo in uso».
- Il dialogo di chiusura e la pagina elencano per categoria ciò che l'app ha avviato (editor, console, progetti, installazione) e, come singoli programmi, solo quelli del kit che **non** discendono dall'app (avviati da altri strumenti o rimasti da una sessione precedente). Quelli dell'app finiscono comunque con il suo job; elencarli anche come processi li mostrava due volte (trovato in prova). L'arresto invece colpisce tutti i programmi del kit.
- Altre copie del kit: processi con l'eseguibile in un `<radice>\data\apps\win32-*` diverso, oppure un altro `Dev-Kit.exe` con `data\app` accanto. Sono segnalate nella Home.

## D-56 Quattro lingue e accessibilità

- L'interfaccia usa `t("testo italiano")` con i cataloghi `ui/src/locales/{en,fr,de}.json`, come `src-tauri/i18n/catalog.json` per i messaggi di Rust. Due test verificano che ogni testo usato sia in tutte le lingue con gli stessi segnaposto e che non restino voci inutilizzate. La lingua viene da Impostazioni, poi da `DEVKIT_LANG`, poi da Windows; cambiandola l'interfaccia si aggiorna senza riavvio.
- Registro: «tu» in italiano, «you» in inglese, «vous» in francese, «Sie» in tedesco, come i messaggi Rust già tradotti e la localizzazione tedesca di VS Code.
- Termini fissati per coerenza:
  - «Linguaggi e strumenti»: Languages and tools, Langages et outils, Sprachen und Werkzeuge;
  - «Esci» (da un agente): Sign out, Se déconnecter, Abmelden;
  - virgolette «» in francese, “” in inglese, „“ in tedesco.
- Frasi intere nei cataloghi, mai composte a pezzi. Una frase con parti in carattere monospaziato passa per `rich()` di `i18n.ts`, che inserisce gli elementi nei segnaposto. I testi che dipendono dalla lingua vengono calcolati al momento del disegno (per esempio i gruppi dei componenti sono una funzione, non una costante).
- Test dei cataloghi:
  - interfaccia: `i18n.test.ts` legge i sorgenti con `import.meta.glob` di Vite, perché `node:fs` richiederebbe `@types/node`;
  - Rust: `i18n::tests::sources_are_in_the_catalog` legge `src\` e controlla anche le tuple `Err(("…", …))` di `settings::validate()`, che vengono tradotte da `settings_set`.
- Accessibilità:
  - focus sempre visibile come in VSCodium e collegamento «Vai al contenuto»;
  - nomi per icone e pulsanti, titoli di pagina e di sezione;
  - errori e stati annunciati (`role="alert"`/`role="status"`);
  - dialogo di chiusura con `role="alertdialog"`, focus su «Annulla» ed Esc;
  - nessuna animazione con «riduci animazioni»;
  - rosso degli errori del tema chiaro portato a un contrasto di almeno 4,5:1.

## D-57 Console classica di Windows al posto di xterm.js

- **Scelta dell'utente** (14/10/2026): via la libreria del terminale «web» (xterm.js) dalle console dell'app; tutto ciò che il kit fa usa solo strumenti del kit o componenti di Windows, mai programmi del PC. Sostituisce D-44, D-45 e D-46; chiude D-47.
- **Come.** `terminal.rs`: la PowerShell del kit, o un agente, parte in `conhost.exe` di System32 avviato direttamente, con il solo ambiente del kit e dentro il job dell'app. Avviata così, la console resta nella finestra classica e Windows non la passa al terminale predefinito del PC (Windows Terminal o altri): verificato su Windows 11 (`ConsoleWindowClass`, nessun `WindowsTerminal.exe`).
- **Senza maniglie standard.** conhost che riceve stdin/stdout fa da pseudo console e scrive la console come testo VT su quelle maniglie, senza finestra. `std::process::Command` le passa sempre (`STARTF_USESTDHANDLES`), quindi `jobs::spawn_console` usa `CreateProcessW` senza maniglie, sospeso, assegnato al job e poi ripreso.
- **Tolti.** `portable-pty`, `@xterm/xterm`, `@xterm/addon-fit`, `terminal.html`, la capability `terminal` e i comandi `terminal_attach`, `terminal_input`, `terminal_resize`. Le finestre dell'app sono di nuovo solo `main` ed `editor-*`.
- **Conseguenze.** Claude Code gira in una console normale di Windows, senza la pseudo console di D-47: i tasti dovrebbero arrivare, da provare a mano con l'app. «Chiudi tutto», la conferma alla chiusura e «Rimuovi l'unità» contano e chiudono le console dai processi `conhost.exe` avviati dall'app. Il titolo della finestra lo decide il programma. Il porting su macOS e Linux dovrà scegliere il terminale di quei sistemi.
