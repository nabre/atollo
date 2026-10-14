# Atollo App: decisioni

Ogni voce: scelta, motivo, alternativa scartata. Stato **proposta** finché il piano (revisione 3) non è approvato.
API verificate il 13-14/10/2026 su docs.rs (`tauri 2.12.0`), v2.tauri.app e le release GitHub citate.

## D-00 Struttura: `Atollo.exe` + `data\` + `projects\`

- **Scelta.** Tutta la logica, i testi, il catalogo dei componenti e le estensioni del kit sono nell'exe. Tutto ciò che si scarica o si genera sta in `data\` accanto all'exe: programmi (`data\apps\win32-<arch>\`), stato, impostazioni, cache, temporanei. I progetti stanno in `projects\` nella radice, accanto all'exe (indicazione dell'utente, 14/10/2026).
- **Motivo.** Indicazione dell'utente (14/10/2026). Un solo file da distribuire e da aggiornare; una sola cartella da salvare.
- **Scartata.** Struttura del Atollo attuale (`setup\`, `apps\`, `projects\`, `data\` nella radice) con script richiamati dall'app.
- **Conseguenze.** Il prompt chiedeva di riusare gli script e di non rompere `setup\Windows.cmd`: superato (PLAN, Q1). `setup\` non si aggiorna più da `main`: si aggiorna solo l'exe.

## D-01 Ambiente: calcolato in Rust

- **Scelta.** `env.rs` produce le stesse variabili di `setup\windows\_env.cmd` (commit `a61a354`), con `CP_APPS = data\apps\win32-<arch>` e le stesse condizioni (per esempio `JAVA_HOME` solo se c'è `java\bin\java.exe`). PATH = cartelle del kit nell'ordine di `_env.cmd` + `System32`, `System32\Wbem`, `System32\WindowsPowerShell\v1.0`. Mai il PATH del PC.
- **Impostazioni di rete** (proxy, `NODE_EXTRA_CA_CERTS`, `GEMINI_API_KEY`, `ATOLLO_LANG`, `ATOLLO_ALLOW_LAN`) in `data\app\settings.json`, modificate dalla pagina Impostazioni; nessun `proxy.cmd` eseguito.
- **Test.** Tabella attesa per ogni variabile; con un Atollo attuale disponibile in CI, confronto con l'esito di `_env.cmd` (differenze ammesse elencate: radice di `apps`, PATH filtrato, `CP_MOVED_FROM`, `PROMPT`).
- **Scartata.** Eseguire `_env.cmd`: nel nuovo kit non c'è.

## D-02 Frontend: React + TypeScript + Tailwind + react-icons

- **Scelta dell'utente** (14/10/2026), al posto di Svelte proposto nella revisione 1.
- **Versioni** (pubblicate da almeno due settimane, fissate in `ui\package.json` e `package-lock.json`): React 19.3.0, Vite 8.3.1, `@vitejs/plugin-react` 6.1.1, Tailwind 4.3.3 con `@tailwindcss/vite`, react-icons 5.7.0, Vitest 5.0.2 + jsdom + Testing Library, ESLint 10 + typescript-eslint 8.71 (`strictTypeChecked`), Prettier 3.9.
- **TypeScript 6.0.3, non 7.** TypeScript 7 (luglio 2026) non è ancora supportato da typescript-eslint (`>=4.8.4 <6.1.0`).
- **Icone**: `react-icons/vsc`, cioè le Codicons di VS Code, per lo stesso linguaggio visivo dell'editor (D-27).

## D-27 Tema VSCodium ovunque

- **Scelta dell'utente** (14/10/2026): il tema di VSCodium per tutto.
- **Interfaccia dell'app**: colori dei temi predefiniti "Dark Modern" e "Light Modern" come variabili CSS (`ui\src\theme.css`), scelti da `prefers-color-scheme` (tema di Windows); struttura a barra delle attività, area di contenuto e barra di stato come VSCodium; carattere Segoe WPC/Segoe UI 13 px; focus visibile con il colore di VSCodium.
- **Editor**: `window.autoDetectColorScheme` attivo, con Dark Modern / Light Modern come temi preferiti (fase 3).
- **Terminali**: schema di colori di Windows Terminal portabile e colori di PSReadLine ricavati da `terminal.ansi*` degli stessi temi (fase 3).
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

- Mutex `Local\Atollo-App-<sha256(radice minuscola)[0:16]>` e pipe omonima per "porta in primo piano".
- Scartato `tauri-plugin-single-instance`: identificatore fisso, due chiavette diverse si bloccherebbero a vicenda.

## D-09 Altre copie del kit aperte

- `CreateToolhelp32Snapshot` + `QueryFullProcessImageNameW`: processo di un altro kit se il percorso sta sotto `<radice>\data\apps\win32-*\` o è un altro `Atollo.exe`, con radice diversa. Regola equivalente a `Test-Sessions.ps1:59-73`.

## D-10 Scorciatoie e menu del browser nell'editor

- Tauri non li espone. `with_webview` → `ICoreWebView2Settings3::SetAreBrowserAcceleratorKeysEnabled(false)`, `SetAreDefaultContextMenusEnabled(false)`; `zoom_hotkeys_enabled(false)`. Solo sulle finestre `editor`.

## D-11 Motore di installazione in Rust

- Stessi passi e controlli degli script (`Prepare-Atollo.ps1` e helper): verifica prima di sostituire, componente in uso lasciato intatto, un componente fallito non ferma gli altri, 3 tentativi per download. Una preparazione alla volta (mutex per kit). Annullamento al termine del passo in corso; "Ferma subito" mai durante uno scambio di cartelle.
- La rimozione copre **tutti** i componenti (gli script dimenticano Typst e TinyTeX).

## D-12 Rilevatori dei progetti in Rust

- Porting di `core/detectors` (Node, PHP, Python, Rust, Go, statico; priorità desktop > web > comando; `panel.json`). Usati solo dall'app (editor pulito, Q4 = a): test su progetti di esempio presi dai casi del Atollo.

## D-13 Console fuori dall'editor

- Windows Terminal portabile del kit, avviato da noi (quindi nel job): `WindowsTerminal.exe --window new -d <cartella> -- <comando>`. Ripiego `conhost` con `CREATE_NEW_CONSOLE`.

## D-14 Comandi solo per la finestra `main`

- Verificato: in Tauri 2 i comandi registrati sono permessi a **tutte** le finestre salvo `AppManifest::commands` in `build.rs`; le origini remote non hanno accesso salvo `remote.urls`.
- (1) `AppManifest::commands`; (2) una capability `windows: ["main"]`, nessun `remote`; (3) controllo `label == "main"` in ogni comando; (4) test con `tauri::test`: invocazione da una WebView `editor` su `http://127.0.0.1` rifiutata.

## D-15 Plugin

- `tauri-plugin-dialog` (scelta cartelle) e `tauri-plugin-opener` usati solo da Rust. Nessun `tauri-plugin-shell`, nessun `single-instance`.

## D-16 Catalogo dei componenti incorporato

- Il catalogo è un file JSON nel codice sorgente, incorporato con `include_str!` e validato da un test (id unici, `needs` esistenti, gruppi validi). Id liberi (il vincolo `^[a-z]+$` era degli script): `vscodium-server`, `pwsh`, `terminal`, `webview2`.
- Scelta in `data\app\components.txt` (formato `id=1|0` del Atollo: semplice e leggibile a mano); versioni in `data\app\versions\<id>.txt`.

## D-17 `cmd.exe` solo dove serve

- Il motore non usa `cmd`. Restano solo il profilo "Atollo - cmd" del terminale e gli strumenti installati come `.cmd`/`.bat` (per esempio `mvn.cmd`, `gradle.bat`): avvio con `raw_arg` da una sola funzione testata con `a b`, `(x)`, `a&b`, `50%`, `a!b`.

## D-18 Testi

- Sorgente in italiano, cataloghi it/en/fr/de nel codice, incorporati. Test di build: ogni testo tradotto in tutte le lingue, stessi segnaposto `{n}`, nessuna traduzione vuota (stesse regole di `setup/lang/build.js`).

## D-19 Toolchain

- CI: `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`. Compilazione con il Rust `gnu` del Atollo: da verificare in fase 1.

## D-20 Aggiornamento dell'exe

- `data\app\update\`, SHA-256 dal file `.sha256` della release; alla chiusura `Atollo.exe` → `Atollo.old.exe` (Windows permette di rinominare un exe in esecuzione), nuovo al suo posto; eliminazione al primo avvio successivo. Disattivabile nelle impostazioni. Fonte delle release: Q3.

## D-21 Riga di comando nello stesso exe

- `Atollo.exe env | prepare | sync | agent <id> [cartella] | eject | sessions`. Exe GUI che per la riga di comando si aggancia alla console del genitore (`AttachConsole(ATTACH_PARENT_PROCESS)`).
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

Versioni pubblicate da almeno due settimane, fissate in `Cargo.lock`.

## D-29 Regole del motore di installazione (fase 2)

- **Proxy**: impostazioni dell'app → `HTTPS_PROXY`/`HTTP_PROXY` → proxy di Windows (chiave `Internet Settings` di HKCU, sola lettura). I PAC (configurazione automatica) non sono supportati: chi li usa imposta il proxy nella pagina Impostazioni.
- **Token GitHub**: solo se è nell'ambiente (`GITHUB_TOKEN`/`GH_TOKEN`, cioè nella CI), solo verso `api.github.com`. L'app non lo chiede e non lo salva.
- **Estrazione**: voci con percorsi assoluti, `..` o unità refuse; collegamenti simbolici ignorati; date dei file non ripristinate (exFAT).
- **Scambio**: la cartella vecchia viene rinominata prima di tutto; se Windows rifiuta (un file in uso) non cambia nulla e il componente risulta "in uso". Verificato con PowerShell aperto.
- **Git**: `post-install.bat` di Git Portable eseguito nella sua cartella con l'ambiente del kit; riuscito se lo script si è cancellato (il suo codice di uscita è sempre 1, perché cmd non trova più il file che stava eseguendo).
- **`-Skip` sugli obbligatori**: ignorato, come negli script del kit.
- **Id con trattino**: ammessi (`vscodium-server`), perché il catalogo è dell'app e non viene letto dagli script del Atollo.
- **Annullamento**: immediato durante un download (il file parziale viene cancellato); altrimenti alla fine del passo in corso.

## D-23 Terminale esclusivo del kit

- **Scelta.** Shell predefinita nell'editor: PowerShell 7 portabile, `-NoProfile` + profilo del kit in `data\kit\`; `PSModulePath` ristretto al kit; `POWERSHELL_TELEMETRY_OPTOUT=1`, `POWERSHELL_UPDATECHECK=Off`. Git Bash e cmd come profili aggiuntivi. Console esterne: Windows Terminal portabile (file `.portable` → impostazioni nella sua cartella).
- **Scartate.** Windows PowerShell 5.1 di sistema: carica moduli e profili del PC. Nushell: estranea agli agenti, che si aspettano PowerShell o Bash. Windows Terminal di sistema: pacchetto Store, fuori dal job, impostazioni in `%LOCALAPPDATA%`.
- **Verificato il 14/10/2026.** PowerShell 7.6.7: `PowerShell-7.6.7-win-x64.zip` / `-win-arm64.zip`, SHA-256 in `hashes.sha256` e nel `digest` delle asset. Windows Terminal 1.25.2733.0: `Microsoft.WindowsTerminal_1.25.2733.0_x64.zip` / `_arm64.zip`, `digest` SHA-256. Requisito minimo di Windows Terminal (Windows 10 2004) da confermare in fase 2.

## D-24 Nessuna richiesta di amministratore

- Manifesto incorporato con `requestedExecutionLevel level="asInvoker" uiAccess="false"` (senza, Windows applica il rilevamento degli installer, e un motore di installazione lo rischierebbe). Test in CI che legge il manifesto dall'exe compilato. Nessuna scrittura nel registro, nessun servizio, nessuna attività pianificata.

## D-25 File che i programmi vogliono su disco

- `loopback.js` (per `NODE_OPTIONS=--require`), profilo PowerShell, `bashrc`: incorporati nell'exe, scritti in `data\kit\` al primo avvio di ogni versione (confronto di un hash), mai modificati dall'utente.

## D-26 Server web dell'editor

- Release VSCodium `vscodium-reh-web-win32-x64-<versione>.tar.gz` + `.sha256` (verificato: 1.140.06880). Nessun pacchetto ARM64: su ARM64 si usa x64 in emulazione, con avviso nell'interfaccia.

## D-27 Tema VSCodium ovunque

- Finestra dell'app con i colori di VSCodium "Default Dark Modern" / "Default Light Modern" (variabili CSS in `ui/src/theme.css`, scelte da `prefers-color-scheme`) e icone Codicons (`react-icons/vsc`). Editor: `window.autoDetectColorScheme` con gli stessi due temi. Windows Terminal: schemi "VSCodium Dark Modern" / "VSCodium Light Modern" con i colori del terminale di VSCodium, `theme: system`.

## D-28 Interfaccia: React 19 + TypeScript + Tailwind 4 + react-icons

- Indicazione dell'utente (14/10/2026); sostituisce Svelte di D-02. Versioni pubblicate da almeno due settimane, fissate in `package-lock.json`.

## D-30 Server dell'editor avviato senza `cmd.exe`

- `bin\codium-server.cmd` esegue solo `node.exe out\server-main.js <argomenti>`: l'app avvia direttamente quel comando, con gli argomenti a vettore. Così i percorsi con `&`, `%`, `!` o parentesi non passano mai da `cmd`.
- Il server parte sospeso, entra in un Job Object suo (annidato in quello dell'app) e poi riprende: anche l'extension host e i terminali stanno nel job, e fermare il server chiude tutto l'albero (`jobs::spawn_in`). Verificato da `editor::tests::real_server_token_redirect_and_stop`.

## D-31 Impostazioni dell'editor in `data\vscodium-server\data\Machine\settings.json`

- Sono le impostazioni "Remote" del server, che il client web applica a ogni finestra; quelle dell'utente restano sue (nel browser). Verificato: con i profili scritti lì, "Terminal: Create New Terminal" apre il profilo "Atollo" (PowerShell 7 del kit, `USERPROFILE` in `data\home`).
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

- Elenco del Atollo (`src-tauri/catalog/extensions.txt`, 60 estensioni) più `data\app\extensions.txt` dell'utente; `@programma` = solo se quel componente è nel kit; proposta una volta sola (`data\app\extensions-offered.txt`); una che fallisce viene ritentata all'avvio successivo; CodeLLDB dalla release GitHub (SHA-256).
- Installate con la riga di comando del server (`--install-extension`, Open VSX) **dopo** l'avvio dell'editor, non prima come chiedeva il prompt: misurate circa 35 s per estensione al primo giro, l'editor non può aspettare minuti. Le estensioni compaiono ricaricando la finestra; lo stato è in `editor_status.extensions`.
- Scartato: disattivare le estensioni di un programma tolto (il Atollo lo faceva scrivendo nel database SQLite di VSCodium). Restano installate: non fanno danni e si riattivano da sole se il programma torna.

## D-37 WebView2 "Fixed Version" non è un componente

- Microsoft la pubblica solo da una pagina web, con indirizzi che cambiano e senza checksum: non c'è una fonte da cui scaricarla e verificarla come gli altri componenti. Resta il supporto all'avvio (D-05): se manca il WebView2 di sistema, la finestra di messaggio spiega dove scaricarla e in quale cartella del kit estrarla.

## D-38 Motore di installazione: parti, firme, processi

- `Release` = una o più parti (Java + Maven + Gradle, CMake + Ninja, PHP + Composer), ognuna con destinazione: cartella propria, altra cartella, file in `tools\`, file singolo sostituito (Claude, Antigravity: il vecchio exe rinominato `.old`, così si aggiorna anche se l'agente è aperto). La versione salvata unisce le versioni delle parti.
- Firma Authenticode (`signature.rs`, `WinVerifyTrust` + soggetto del firmatario): Claude Code (`Anthropic`, revoca non verificabile = avviso), runtime VC++ (`O=Microsoft Corporation`, obbligatoria sull'exe e sulla DLL).
- Installatori fatti di processi (`custom`): Rust (`rustup-init`, poi `rustup update` sul posto per non perdere crate e `cargo install`; voce "Rustup" in HKCU ripristinata com'era: verificato con il rustup dell'utente già presente), pnpm/yarn (`node npm-cli.js`, senza `cmd.exe`), R (micromamba con profilo nell'area di lavoro).
- File `.bat`/`.cmd` (verifica di Kotlin): `jobs::batch_command`, `cmd /d /s /c ""percorso" argomenti"` con `raw_arg`; trovato in prova che `cmd /c` con un percorso tra virgolette e `&` si rompe.
- Limite: Rust su un kit FAT32/exFAT non si installa (rustup richiede i collegamenti fisici); il Atollo installava la toolchain sul disco del PC e copiava solo `rustup\`. Da portare (vedi CHANGELOG).

## D-39 Verifica finale

- `checkup.rs` (`Test-Installation` del kit): ogni componente installato risponde al suo comando di versione (campo `check` del catalogo), con l'ambiente del kit e 60 s di limite. `Atollo.exe check`; dopo `prepare` per ciò che è stato installato.

## D-40 Allineamento degli agenti in Rust

- Porting completo di `setup/agents/*.js` (`agents_sync/`): istruzioni (copia a tre vie con stato e conflitti), skill (giunzioni NTFS con il crate `junction`, copie su FAT32/exFAT), MCP (Claude ↔ `mcp.json`, Codex `config.toml` a blocchi di testo, Antigravity solo i server del kit), regola del kit, cartelle attendibili, skill di progetto. Stessi file di stato e stessi marcatori del Atollo.
- Miglioria: due passaggi di collegamento delle skill, così una skill nata in un agente arriva a tutti nello stesso allineamento (nel Atollo serviva il giro successivo).
- Messaggi in inglese nel log (`data\app\logs\app.log`); l'interfaccia mostra lo stato, non i dettagli.
- Quando: all'avvio dell'app, prima di ogni agente lanciato dall'app e quando un agente del kit termina, ovunque sia stato aperto (sorvegliante ogni 3 s). Scartato: il wrapper `claude.cmd`/`codex.cmd` nel PATH, che avrebbe aggiunto la domanda "Terminare il processo batch?" di cmd a ogni Ctrl+C.

## D-41 Accesso di Antigravity: `agy.cmd` del kit e sorvegliante dell'app

- `data\kit\bin\agy.cmd` (primo nel PATH, anche per Git Bash con `bin\agy`): `Atollo.exe keyring restore`, `agy.exe`, `Atollo.exe keyring save`. In un file batch `cmd` aspetta la fine anche di un programma grafico. `agy.exe` non è nel PATH: non si scavalca il wrapper.
- Il processo "watch" nascosto del Atollo è sostituito dal sorvegliante dell'app; inoltre il `save` a fine esecuzione toglie il segnale "in avvio" quando nessun agy del kit gira (trovato in prova: con un agy brevissimo il login restava in Windows fino al sorvegliante).
- Verificato con il Gestore credenziali reale: persistenza "solo sessione" al restore; dopo il save la credenziale del kit è sparita da Windows e il token aggiornato è nel kit; la credenziale `gemini:antigravity` dell'Antigravity installato sul PC non viene toccata.

## D-42 `claudeCode.environmentVariables` non serve nel server web

- Nel Atollo l'estensione Atollo le scriveva perché VSCodium desktop poteva partire senza l'ambiente del kit. Nel server web l'extension host eredita l'ambiente del server, che è quello del kit: l'estensione Claude Code vede già `CLAUDE_CONFIG_DIR`, `DISABLE_UPDATES`, `NODE_OPTIONS`, `CLAUDE_CODE_GIT_BASH_PATH`.

## D-43 Rilocazione

- Porting di `moved.js` (`relocate.rs`) sulla nuova struttura: radice precedente da `data\app\old-root.txt` (scritto da `kit.rs`), consumato a lavoro fatto; tra i file aggiornati anche impostazioni ed estensioni del server dell'editor. Verificato rinominando un kit con tutti i componenti: chiavi di `.claude.json` unite sotto il nuovo percorso, `config.toml` senza il vecchio percorso, giunzioni delle skill rifatte.
