# Dev-Kit App: decisioni

Ogni voce: scelta, motivo, alternativa scartata. Stato **approvato** con il piano (revisione 3, 14/10/2026). Una voce superata lo dice nel titolo e rimanda a quella nuova; dove il codice ha preso un'altra strada, una nota **Oggi** dice com'è.
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
- **Versioni** (pubblicate da almeno due settimane, fissate in `ui\package.json` e `package-lock.json`): React 19.3.0, Vite 8.3.1, `@vitejs/plugin-react` 6.1.1, Tailwind 4.3.3 con `@tailwindcss/vite`, react-icons 5.7.0, Vitest 5.0.2 + jsdom + Testing Library, ESLint 10 + typescript-eslint 8.71 (`strictTypeChecked`), Prettier 3.9. xterm.js 6.0.0 e `@xterm/addon-fit` 0.11.0 (console, D-44) sono stati tolti con D-57.
- **TypeScript 6.0.3, non 7.** TypeScript 7 (luglio 2026) non è ancora supportato da typescript-eslint (`>=4.8.4 <6.1.0`).
- **Icone**: `react-icons/vsc`, cioè le Codicons di VS Code, per lo stesso linguaggio visivo dell'editor (D-27).

## D-27 Tema VSCodium ovunque

- **Scelta dell'utente** (14/10/2026): il tema di VSCodium per tutto.
- **Interfaccia dell'app**: colori dei temi predefiniti "Dark Modern" e "Light Modern" come variabili CSS (`ui\src\theme.css`), scelti da `prefers-color-scheme` (tema di Windows); struttura a barra delle attività, area di contenuto e barra di stato come VSCodium; carattere Segoe WPC/Segoe UI 13 px; focus visibile con il colore di VSCodium.
- **Editor**: `window.autoDetectColorScheme` attivo, con Dark Modern / Light Modern come temi preferiti (fase 3).
- **Icone**: Codicons (`react-icons/vsc`).
- **Terminali**: i colori di PSReadLine ricavati da `terminal.ansi*` previsti per la fase 3 non si fanno (scelta dell'utente del 14/10/2026): bastano gli schemi del terminale. Le console dell'app usavano i colori del terminale di VSCodium in xterm.js (D-44, D-46); oggi Windows Terminal del kit usa gli schemi «VSCodium Dark Modern» / «VSCodium Light Modern» scritti dall'app (D-68), la console classica (D-57) i colori della console di Windows.
- **Da verificare**: i valori in `theme.css` vanno confrontati con i file `dark_modern.json` / `light_modern.json` del server di VSCodium appena installato (fase 3); eventuali differenze si correggono lì.

## D-03 Tauri 2.12.0 fissato

- Ultima 2.x pubblicata da più di due settimane (26/09/2026); 3.0 è alpha. `tauri build --no-bundle`, exe da `target\<triple>\release\`.

## D-04 Il processo dell'app non scrive fuori da `data\`

- All'avvio, prima di qualsiasi thread: `APPDATA`, `LOCALAPPDATA`, `TEMP`, `TMP`, `USERPROFILE` del processo puntati in `data\`; `WEBVIEW2_USER_DATA_FOLDER=data\webview2`; ogni `WebviewWindowBuilder` con `.data_directory(data\webview2)` (API verificata). Nessun uso di `app_data_dir` e simili di Tauri.
- Vincolo dalla documentazione: le WebView con la stessa cartella dati devono avere gli stessi `additional_browser_args` e `scroll_bar_style`: un'unica configurazione per tutte.

## D-05 WebView2 senza installarlo

- `GetAvailableCoreWebView2BrowserVersionString` (crate `webview2-com`, già usato da `wry`). Se manca il runtime di sistema e c'è `data\apps\win32-<arch>\webview2\msedgewebview2.exe` → `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`. Altrimenti `TaskDialogIndirect` con il link per scaricarlo e la cartella dove metterlo, poi uscita. Mai il bootstrapper.
- Facoltativo (Q nel piano di fase 2): pulsante che scarica la "Fixed Version" in `data\apps\…\webview2\` dalla finestra di dialogo, con verifica della firma Microsoft.
- **Oggi**: una finestra di messaggio semplice (`MessageBoxW`) con l'indirizzo e la cartella; nessun pulsante di download (D-37).

## D-06 Nuove finestre e navigazione dell'editor

- API verificate: `on_new_window(Fn(Url, NewWindowFeatures) -> NewWindowResponse)`, `on_navigation(Fn(&Url) -> bool)`.
- `on_navigation`: solo `http://127.0.0.1:<porta>` dentro la cartella del progetto (D-62) e `about:`. `on_new_window`: stessa origine → rifiuto e nuova finestra `editor` creata dal nostro costruttore; altri `http`/`https` → browser di sistema; altri schemi → bloccati.

## D-07 Job Object: l'app entra nel job da sé

- `KILL_ON_JOB_CLOSE | BREAKAWAY_OK`, handle non ereditabile, `AssignProcessToJobObject(GetCurrentProcess())`: ogni figlio nasce nel job. Sotto-job annidati per fermare un albero di processi (server dell'editor, azioni dei progetti).
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

- Stessi passi e controlli degli script (`Prepare-DevKit.ps1` e helper): verifica prima di sostituire, componente in uso lasciato intatto, un componente fallito non ferma gli altri, 3 tentativi per download. Una preparazione alla volta (mutex per kit). «Annulla» ferma subito un download, altrimenti al termine del passo in corso; mai durante un'estrazione o uno scambio di cartelle, che lascerebbero un componente a metà (scelta dell'utente del 14/10/2026: niente «Ferma subito»).
- La rimozione copre **tutti** i componenti (gli script dimenticano Typst e TinyTeX).

## D-12 Rilevatori dei progetti in Rust

- Porting di `core/detectors` (Node, PHP, Python, Rust, Go, statico; priorità desktop > web > comando; `panel.json`). Usati solo dall'app (editor pulito, Q4 = a): test su progetti di esempio presi dai casi del Dev-Kit.

## D-13 Console fuori dall'editor

- Windows Terminal portabile del kit (D-68), predefinito; la console classica di Windows (D-57) come alternativa nelle Impostazioni e come ripiego. Sempre con il solo ambiente del kit e nel job; mai il terminale predefinito del PC.
- Prima (fino al 14/10/2026): Windows Terminal portabile del kit, `WindowsTerminal.exe --window new -d <cartella> -- <comando>`, sostituito perché esiste solo per Windows (D-44); poi finestre dell'app con xterm.js e pseudo console (D-44…D-46), sostituite da D-57; poi di nuovo Windows Terminal, predefinito (D-68).

## D-14 Comandi solo per la finestra `main`

- Verificato: in Tauri 2 i comandi registrati sono permessi a **tutte** le finestre salvo `AppManifest::commands` in `build.rs`; le origini remote non hanno accesso salvo `remote.urls`.
- (1) `AppManifest::commands`; (2) una capability `windows: ["main"]`, nessun `remote`; (3) controllo `label == "main"` in ogni comando; (4) test con `tauri::test`: invocazione da una WebView `editor` su `http://127.0.0.1` rifiutata.

## D-15 Plugin

- `tauri-plugin-dialog` (scelta cartelle) e `tauri-plugin-opener` usati solo da Rust. Nessun `tauri-plugin-shell`, nessun `single-instance`.
- **Oggi**: nessun plugin di Tauri in `Cargo.toml`. Esplora risorse e i link della finestra principale si aprono con `explorer.exe`, i link dell'editor con `rundll32 url.dll,FileProtocolHandler`, sempre fuori dal job (`jobs::spawn_detached`, D-07); l'app non ha una finestra di scelta delle cartelle.

## D-16 Catalogo dei componenti incorporato

- Il catalogo è un file JSON nel codice sorgente, incorporato con `include_str!` e validato da un test (id unici, `needs` esistenti, gruppi validi). Id liberi (il vincolo `^[a-z]+$` era degli script): `vscodium-server`, `pwsh` (`terminal`, Windows Terminal: tolto con D-44, tornato con D-68; WebView2 non è un componente: D-37).
- Campi oltre a quelli del Dev-Kit: `keys` (le altre parti: Maven, Ninja, Composer…; il componente è installato solo se ci sono tutte), `noArm64` (nessuna build per ARM64, nemmeno emulata: R), `remove` condiviso (`tools\vcruntime` di PHP e Apache se ne va con l'ultimo dei due), `version` e `sha256` per il gruppo `app` (D-58). Gruppi: `required`, `agents`, `languages`, `app`.
- Scelta in `data\app\components.txt` (formato `id=1|0` del Dev-Kit: semplice e leggibile a mano); versioni in `data\app\versions\<id>.txt`.

## D-17 `cmd.exe` solo dove serve

- Il motore non usa `cmd`. Restano solo il profilo "Dev-Kit - cmd" del terminale e gli strumenti installati come `.cmd`/`.bat` (per esempio `mvn.cmd`, `gradle.bat`): avvio con `raw_arg` da una sola funzione (`jobs::batch_command`), che lancia `.\<nome>` dalla cartella dello script: il percorso non passa da `cmd`, che espanderebbe `%VAR%` anche fra virgolette (trovato il 14/10/2026: con una cartella `%SystemRoot%` lo script non partiva). Test `batch_files_in_odd_folders` con `a b (x) & y`, `50%`, `a!b`, `%SystemRoot%`, `%OS% & !OS!`.
- **Oggi** `cmd.exe` gira anche: per le azioni dei progetti (`cmd.exe /d /s /c`, come nel Dev-Kit, D-50); per `post-install.bat` di Git (D-29); per i `.cmd`/`.bat` della verifica finale (`jobs::batch_command`, D-38); per `data\kit\bin\agy.cmd` (D-41), che la console avvia dalla PowerShell del kit e che il profilo «Antigravity» dell'editor usa. Questi ultimi passano il percorso completo a `cmd`: un percorso del kit con `%VAR%` di una variabile esistente non funziona (limite noto, come D-33).

## D-18 Testi

- Sorgente in italiano, cataloghi it/en/fr/de nel codice, incorporati. Test di build: ogni testo tradotto in tutte le lingue, stessi segnaposto `{n}`, nessuna traduzione vuota (stesse regole di `setup/lang/build.js`).

## D-19 Toolchain

- CI: `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`. Compilazione con il Rust `gnu` del Dev-Kit: da verificare in fase 1.

## D-20 Aggiornamento dell'exe

- `data\app\update\`, SHA-256 dal digest di GitHub o dal file `.sha256` della release; alla chiusura `Dev-Kit.exe` → `Dev-Kit.old.exe` (Windows permette di rinominare un exe in esecuzione), nuovo al suo posto; eliminazione al primo avvio successivo. Disattivabile nelle impostazioni. Fonte delle release: `app-vX.Y.Z` di `nabre/dev-kit2` (D-52).

## D-21 Riga di comando nello stesso exe

- `Dev-Kit.exe env | components | check | prepare | sync | keyring | serve | version | help` (`cli.rs`). Exe GUI che per la riga di comando scrive sull'uscita rediretta, se c'è, o si aggancia alla console del genitore (`AttachConsole(ATTACH_PARENT_PROCESS)`).
  - `prepare [--only a,b] [--skip a,b] [--update-all] [--force]`: 0 tutto bene, 1 un componente non riuscito, 3 un'altra preparazione in corso;
  - `check`: 1 se un programma non risponde;
  - `sync [--project <cartella>] [--verbose]`, `keyring restore|save`, `serve <cartella> <porta>`.
- Usi: `data\kit\bin\agy.cmd` (`keyring`), le azioni dei siti statici (`serve`), la CI (`env`, `prepare`), le prove a mano. I profili degli agenti nell'editor lanciano l'agente direttamente: l'allineamento lo fa il sorvegliante quando l'agente termina (D-40).
- Scartati: `agent`, `eject`, `sessions` della prima proposta; agenti e «Rimuovi l'unità» stanno nella finestra.

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

**Oggi** (`src-tauri\Cargo.toml`):
- `toml_edit` non c'è: i blocchi del kit in `config.toml` di Codex si scrivono come blocchi di testo con marcatori, lasciando com'è il resto del file (D-40).
- `portable-pty` è stato tolto con D-57: le console sono Windows Terminal del kit (D-68) o `conhost.exe` avviato con `CreateProcessW`.
- In più: `junction` 2.1 (giunzioni NTFS delle skill, D-40), `webview2-com` 0.39 (D-05), `native-tls` 0.2, `anyhow`, `thiserror`, `serde`, `serde_json`.

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

- **Scelta.** Shell predefinita nell'editor: PowerShell 7 portabile, `-NoProfile` + profilo del kit in `data\kit\`; `PSModulePath` senza i moduli del PC (D-34); `POWERSHELL_TELEMETRY_OPTOUT=1`, `POWERSHELL_UPDATECHECK=Off`. Git Bash e cmd come profili aggiuntivi. Console esterne: Windows Terminal portabile del kit (D-68), console classica di Windows come alternativa e ripiego (D-57); prima D-44.
- **Scartate.** Windows PowerShell 5.1 di sistema: carica moduli e profili del PC. Nushell: estranea agli agenti, che si aspettano PowerShell o Bash. Windows Terminal di sistema: pacchetto Store, fuori dal job, impostazioni in `%LOCALAPPDATA%`.
- **Verificato il 14/10/2026.** PowerShell 7.6.7: `PowerShell-7.6.7-win-x64.zip` / `-win-arm64.zip`, SHA-256 in `hashes.sha256` e nel `digest` delle asset.

## D-24 Nessuna richiesta di amministratore

- Manifesto incorporato con `requestedExecutionLevel level="asInvoker" uiAccess="false"` (senza, Windows applica il rilevamento degli installer, e un motore di installazione lo rischierebbe). Test in CI che legge il manifesto dall'exe compilato. Nessuna scrittura nel registro (eccezione: la voce `Uninstall\Rustup` che scrive rustup viene riportata com'era, D-38), nessun servizio, nessuna attività pianificata.

## D-25 File che i programmi vogliono su disco

- `loopback.js` (per `NODE_OPTIONS=--require`), `profile.ps1`, `console.ps1`, `bashrc`, `bin\agy.cmd`, `bin\agy`: incorporati nell'exe, riscritti in `data\kit\` a ogni avvio se mancano o sono diversi (confronto del contenuto); una modifica a mano viene sostituita.

## D-26 Server web dell'editor

- Release VSCodium `vscodium-reh-web-win32-x64-<versione>.tar.gz` + `.sha256` (verificato: 1.140.06880). Nessun pacchetto ARM64: su ARM64 si usa x64 in emulazione, con avviso nell'interfaccia.
- **Oggi** la versione non è più l'ultima: è fissata nel catalogo per ogni rilascio, con il suo SHA-256 (D-58); oggi 1.135.06055.

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

- VSCodium passa a `@parcel/watcher` esclusioni con percorsi Windows assoluti (`<kit>\data\vscodium-server\extensions/**`), che il watcher compila come espressioni regolari: dopo `\`, una cifra è un riferimento all'indietro e `x`/`u` iniziano escape esadecimali. Con il kit in `D:\2026\kit` il controllo dei file della cartella aperta fallisce (`regex_error`) e l'editor non vede i cambiamenti fatti fuori (per esempio dagli agenti). Parentesi, spazi e `&` non danno problemi (verificato con il watcher incluso in VSCodium 1.140 e di nuovo il 14/10/2026 con la 1.135.06055 installata dal kit, `@parcel/watcher` 2.5.6: cifra → `error_backref`, `x`/`u` → `error_escape`; vale per ogni cartella del percorso, non solo l'ultima).
- Passare `--extensions-dir` con le barre normali non serve: VSCodium lo normalizza. L'app riconosce questi percorsi (`kit::watcher_unsafe_folder`) e lo dice nella Home, con il suggerimento di rinominare la cartella.

## D-34 PowerShell del kit isolato dal profilo del PC

- .NET chiede le cartelle a Windows e ignora `APPDATA`/`LOCALAPPDATA`: pwsh scriverebbe `StartupProfileData-*` in `%LOCALAPPDATA%\Microsoft\PowerShell` e la cronologia in `%APPDATA%`. Rimedi verificati: `DOTNET_MultiCoreJitNoProfileGather=1` nell'ambiente (nessun file di profilo JIT) e `Set-PSReadLineOption -HistorySavePath` nel profilo del kit (`kit-files/profile.ps1`).
- Resta fuori dal kit solo la cartella vuota `%LOCALAPPDATA%\Microsoft\PowerShell` se non esiste già: pwsh la crea sempre all'avvio (codice di PowerShell, nessuna opzione per evitarlo). Eccezione documentata.
- `PSModulePath` limitato ai moduli di pwsh e a `data\pwsh\Modules` (profilo del kit): i moduli in Documenti e Program Files del PC non vengono caricati.
- Anche l'ambiente del kit ha `PSModulePath` = `data\pwsh\Modules` + `System32\WindowsPowerShell\v1.0\Modules`, per le PowerShell avviate dagli agenti senza il profilo del kit. Codex ha `allow_login_shell = false` nel blocco del kit di `config.toml`: i suoi comandi non caricano i profili PowerShell del PC.

## D-35 Mai una shell del PC

- Prima di aprire editor o console, l'app installa (con la stessa preparazione della pagina "Linguaggi e strumenti", `SetupTask`) i componenti obbligatori e l'editor, se mancano, e attende la fine (un editor di un'altra versione non blocca: D-58); l'avanzamento compare in Home e nella barra di stato.
- Se PowerShell 7 del kit manca comunque, la console non si apre e l'errore lo dice: nessun ripiego su `cmd.exe` o Windows PowerShell.

## D-36 Estensioni consigliate in background dopo l'avvio dell'editor

- Elenco del Dev-Kit (`src-tauri/catalog/extensions.txt`, 60 estensioni) più `data\app\extensions.txt` dell'utente; `@programma` = solo se quel componente è nel kit; proposta una volta sola (`data\app\extensions-offered.txt`); una che fallisce viene ritentata all'avvio successivo; CodeLLDB dalla release GitHub (SHA-256).
- Installate con la riga di comando del server (`--install-extension`, Open VSX) **dopo** l'avvio dell'editor, non prima come chiedeva il prompt: misurate circa 35 s per estensione al primo giro, l'editor non può aspettare minuti. Le estensioni compaiono ricaricando la finestra; lo stato è in `editor_status.extensions`.
- **Si fermano con il server** (correzione del 14/10/2026, trovata dalla prova reale della CI su un kit nuovo): l'installazione usa `node.exe` della cartella del server e gira in un job suo, che `Editor::stop` chiude. Prima restava accesa dopo lo stop, e un aggiornamento dell'editor o «Riavvia editor» la trovavano ancora nella cartella del server. Le estensioni interrotte non risultano offerte e si riprovano alla partenza successiva.
- Scartato: disattivare le estensioni di un programma tolto (il Dev-Kit lo faceva scrivendo nel database SQLite di VSCodium). Restano installate: non fanno danni e si riattivano da sole se il programma torna.

## D-37 WebView2 "Fixed Version" non è un componente

- Microsoft la pubblica solo da una pagina web, con indirizzi che cambiano e senza checksum: non c'è una fonte da cui scaricarla e verificarla come gli altri componenti. Resta il supporto all'avvio (D-05): se manca il WebView2 di sistema, la finestra di messaggio spiega dove scaricarla e in quale cartella del kit estrarla.

## D-38 Motore di installazione: parti, firme, processi

- `Release` = una o più parti (Java + Maven + Gradle, CMake + Ninja, PHP + Composer), ognuna con destinazione: cartella propria, altra cartella, file in `tools\`, file singolo sostituito (Claude, Antigravity: il vecchio exe rinominato `.old`, così si aggiorna anche se l'agente è aperto). La versione salvata unisce le versioni delle parti.
- Firma Authenticode (`signature.rs`, `WinVerifyTrust` + soggetto del firmatario): Claude Code (`Anthropic`, revoca non verificabile = avviso), runtime VC++ (`O=Microsoft Corporation`, obbligatoria sull'exe e sulla DLL).
- Installatori fatti di processi (`custom`): Rust (`rustup-init`, poi `rustup update` sul posto per non perdere crate e `cargo install`; voce "Rustup" in HKCU ripristinata com'era: verificato con il rustup dell'utente già presente), pnpm/yarn (`node npm-cli.js`, senza `cmd.exe`), R (micromamba con profilo nell'area di lavoro).
- File `.bat`/`.cmd` (verifica di Kotlin): `jobs::batch_command`, `cmd /d /s /c ""percorso" argomenti"` con `raw_arg`; trovato in prova che `cmd /c` con un percorso tra virgolette e `&` si rompe.
- Limite: Rust su un kit FAT32/exFAT non si installa (rustup richiede i collegamenti fisici): l'installazione si ferma con un errore. Il Dev-Kit installava la toolchain sul disco del PC e copiava solo `rustup\`. Da portare (vedi CHANGELOG).

## D-39 Verifica finale

- `checkup.rs` (`Test-Installation` del kit): ogni componente installato risponde al suo comando di versione (campo `check` del catalogo, obbligatorio per tutti: test `every_check_names_an_installed_file`), con l'ambiente del kit e 60 s di limite. Un programma a finestra non si avvia: `"check": ["<programma>", "<file-version>"]` legge la versione dal file (`GetFileVersionInfoW`); oggi solo Windows Terminal. `Dev-Kit.exe check`; dopo `prepare` per ciò che è stato installato.

## D-40 Allineamento degli agenti in Rust

- Porting completo di `setup/agents/*.js` (`agents_sync/`): istruzioni (copia a tre vie con stato e conflitti), skill (giunzioni NTFS con il crate `junction`, copie su FAT32/exFAT), MCP (Claude ↔ `mcp.json`, Codex `config.toml` a blocchi di testo, Antigravity solo i server del kit), regola del kit, cartelle attendibili, skill di progetto. Stessi file di stato e stessi marcatori del Dev-Kit.
- Miglioria: due passaggi di collegamento delle skill, così una skill nata in un agente arriva a tutti nello stesso allineamento (nel Dev-Kit serviva il giro successivo).
- Messaggi in inglese nel log (`data\app\logs\app.log`); l'interfaccia mostra lo stato, non i dettagli.
- Quando: all'avvio dell'app, prima di ogni agente lanciato dall'app e quando un agente del kit termina, ovunque sia stato aperto (sorvegliante ogni 3 s). Scartato: il wrapper `claude.cmd`/`codex.cmd` nel PATH, che avrebbe aggiunto la domanda "Terminare il processo batch?" di cmd a ogni Ctrl+C.

## D-41 Accesso di Antigravity: `agy.cmd` del kit e sorvegliante dell'app

- `data\kit\bin\agy.cmd` (primo nel PATH, anche per Git Bash con `bin\agy`): `Dev-Kit.exe keyring restore`, `agy.exe`, `Dev-Kit.exe keyring save`. In un file batch `cmd` aspetta la fine anche di un programma grafico. `agy.exe` non è nel PATH: non si scavalca il wrapper.
- Il processo "watch" nascosto del Dev-Kit è sostituito dal sorvegliante dell'app; inoltre il `save` a fine esecuzione toglie il segnale "in avvio" quando nessun agy del kit gira (trovato in prova: con un agy brevissimo il login restava in Windows fino al sorvegliante).
- Verificato con Gestione credenziali reale: persistenza "solo sessione" al restore; dopo il save la credenziale del kit è sparita da Windows e il token aggiornato è nel kit; la credenziale `gemini:antigravity` dell'Antigravity installato sul PC non viene toccata.

## D-42 `claudeCode.environmentVariables` non serve nel server web

- Nel Dev-Kit l'estensione Dev-Kit le scriveva perché VSCodium desktop poteva partire senza l'ambiente del kit. Nel server web l'extension host eredita l'ambiente del server, che è quello del kit: l'estensione Claude Code vede già `CLAUDE_CONFIG_DIR`, `DISABLE_UPDATES`, `NODE_OPTIONS`, `CLAUDE_CODE_GIT_BASH_PATH`.

## D-43 Rilocazione

- Porting di `moved.js` (`relocate.rs`) sulla nuova struttura: radice precedente da `data\app\old-root.txt` (scritto da `kit.rs`), consumato a lavoro fatto; tra i file aggiornati anche impostazioni ed estensioni del server dell'editor. Verificato rinominando un kit con tutti i componenti: chiavi di `.claude.json` unite sotto il nuovo percorso, `config.toml` senza il vecchio percorso, giunzioni delle skill rifatte.

## D-44 Terminale integrato nell'app (sostituita da D-57)

Superata il 14/10/2026 da D-57 (console classica di Windows), oggi alternativa a Windows Terminal del kit (D-68). Resta per la storia delle scelte.

- **Scelta dell'utente** (14/10/2026): un terminale disponibile per tutti i sistemi operativi; tra un terminale esterno multipiattaforma e uno dentro l'app, l'utente ha scelto **dentro l'app**. Sostituisce Windows Terminal (D-13, D-23), che esiste solo per Windows ed era un eseguibile in più da scaricare.
- **Come.** `terminal.rs`: il processo (PowerShell del kit, o un agente) parte su una pseudo console di `portable-pty` 0.9.0 (ConPTY su Windows, pty su macOS e Linux; D-22) con il solo ambiente del kit; una finestra `terminal-<n>` dell'app carica `terminal.html` (xterm.js 6.0.0 + `@xterm/addon-fit` 0.11.0, D-28) e la mostra. Stesso motore del terminale di VSCodium (xterm.js + PTY).
- **Scartate.** WezTerm (multipiattaforma, portabile, configurazione Lua): resta un eseguibile esterno da scaricare e configurare, con le sue regole di ambiente. Alacritty: niente schede. Kitty: niente Windows. Ghostty: su Windows non maturo. Windows Terminal: solo Windows.
- **Portata.** Il codice del terminale (`portable-pty`, xterm.js, finestre Tauri) è multipiattaforma; il resto dell'app usa ancora API di Windows (job, `windows`, WebView2, percorsi `win32-*`): il porting su macOS e Linux resta da fare, ma il terminale non lo ostacola.
- **Il terminale di VSCodium resta il suo.** L'editor ha già un terminale integrato (xterm.js nella pagina, PTY nel server) sulle shell del kit (D-23); la console dell'app non lo sostituisce e non vi si può innestare, perché il PTY dell'editor è interno al suo server. Stesse shell, stesso profilo, stessi colori: i due terminali si comportano allo stesso modo.
- **Kit esistenti.** La cartella `data\apps\<piattaforma>\terminal\` è di nuovo usata da Windows Terminal del kit (D-68).

## D-45 Console: processo e pseudo console (sostituita da D-57)

Superata il 14/10/2026 con la pseudo console e `portable-pty`. Restano valide l'idea dei `.cmd` lanciati dalla PowerShell del kit e il ripristino di Ctrl+C (`SetConsoleCtrlHandler`), che `terminal.rs` fa ancora.

- **Avvio.** `CommandBuilder` con `env_clear` + ambiente del kit, cartella del progetto. Un `.cmd` (Antigravity, `agy.cmd`) passa dalla PowerShell del kit (`-Command "& '<percorso>'; exit $LASTEXITCODE"`, apici raddoppiati): `cmd` leggerebbe `&` e `%` nel percorso, come in D-17. Senza la PowerShell del kit, nessuna console (mai una shell del PC).
- **Job.** Il processo nasce nel job dell'app (l'app ci sta dentro, D-07); se l'app non è nel job, `jobs::assign_pid` lo assegna subito dopo l'avvio (i suoi primi figli potrebbero sfuggire: limite di quel caso raro).
- **ConPTY chiede la posizione del cursore** (`ESC[6n`) all'avvio, per l'opzione `INHERIT_CURSOR` di `portable-pty`, e non fa partire il programma finché non riceve risposta: la prima risposta (`ESC[1;1R`) la dà l'app, così il programma parte anche prima che la finestra si colleghi. Le domande di posizione non entrano nell'output conservato, altrimenti una finestra ricaricata risponderebbe a domande vecchie scrivendo testo nel programma.
- **Ctrl+C.** Il flag "ignora Ctrl+C" passa dal processo ai figli: l'app avviata in un nuovo gruppo di processi (da uno script, da un altro programma) lo darebbe alla shell, e Ctrl+C non fermerebbe più i comandi (verificato: `ping -t` non si fermava). `SetConsoleCtrlHandler(None, false)` prima di ogni console.
- **Senza ConPTY** (Windows 10 prima della 1809) `portable-pty` va in panic invece di restituire un errore: `catch_unwind`, poi ripiego sulla console classica (`CREATE_NEW_CONSOLE`) con lo stesso programma.
- **Output.** Un thread legge la pseudo console e manda i byte alla finestra con un `Channel` di Tauri (`InvokeResponseBody::Raw`). Tauri consegna i messaggi sotto 1 KB con `eval` e quelli più grandi con il comando interno `plugin:__TAURI_CHANNEL__|fetch` (permesso anche alle finestre console; verificato), in ordine. Gli ultimi 512 KB restano in memoria per una finestra che si collega tardi o si ricarica.
- **Input.** La pagina manda i tasti uno alla volta in ordine (una chiamata `terminal_input` per volta, i tasti battuti nel frattempo insieme): chiamate IPC parallele potrebbero arrivare in disordine.
- **Fine.** Quando il processo termina, l'app chiude la pseudo console (così la lettura finisce), attende l'ultimo output, manda il codice di uscita; la finestra scrive "Processo terminato (codice N)" e si chiude al primo tasto. Chiudere la finestra termina il processo (`Destroyed` → `Terminals::close`). Per gli agenti, alla fine si riallineano (D-40).

## D-46 Finestre console: sicurezza e tasti (sostituita da D-57)

Superata il 14/10/2026: niente più finestre `terminal-*`, capability `terminal` né `terminal.html`.

- **Capability `terminal`** per le finestre `terminal-*` (D-14): solo `terminal_attach`, `terminal_input`, `terminal_resize`, più `core:window:allow-set-title` e `allow-close` della propria finestra. Ogni comando trova la sessione dall'etichetta della finestra che chiama: una finestra raggiunge solo il suo processo. La finestra principale e quelle dell'editor non possono usare questi comandi; le console non possono usare quelli della finestra principale. Test `console_windows_reach_only_their_session`; verificato anche nell'app (`app_info` e `terminal_open` rifiutati da una console).
- **Pagina separata** (`ui/terminal.html`, `ui/src/terminal.ts`, seconda voce di `build.rolldownOptions.input`): la console non carica l'interfaccia principale.
- **`Object.prototype` non congelato** nella console: xterm.js assegna `toString` ai suoi oggetti e con `freeze.ts` non si carica (verificato: "Cannot assign to read only property 'toString'"). Rischio basso: la pagina non legge dati esterni, mostra solo i byte del proprio processo come testo.
- **WebView2** come le finestre dell'editor (`editor_window::quiet_webview`): tasti del browser (F5, Ctrl+F, Ctrl+P), menu contestuale e barra di stato spenti, così i tasti arrivano alla shell; nuove finestre negate.
- **Tasti come Windows Terminal.** Ctrl+C copia se c'è una selezione, altrimenti va al programma; Ctrl+V e Ctrl+Maiusc+V incollano; Ctrl+Maiusc+C copia; clic destro copia la selezione o incolla. Titolo della finestra dal programma (OSC 0/2), con "- Dev-Kit".
- **Colori** del terminale di VSCodium "Dark Modern" / "Light Modern" (gli stessi degli schemi che si scrivevano per Windows Terminal), scelti da `prefers-color-scheme` e aggiornati quando cambia il tema di Windows (D-27). Carattere Cascadia Mono / Consolas, 14 px.

## D-47 ConPTY aggiornato (chiusa da D-57)

Chiusa il 14/10/2026: senza pseudo console il ConPTY aggiornato non serve e il componente `conpty` non è stato aggiunto. Da provare a mano che Claude Code riceva i tasti nella console classica (TESTING.md).

- **Problema.** Con il ConPTY di Windows, Claude Code si apre e si disegna ma non riceve i tasti (né Invio né frecce; Ctrl+C arriva solo come segnale). PowerShell e Node in modalità raw li ricevono. All'avvio Claude interroga il terminale: XTVERSION (`ESC[>0q`), protocollo tastiera kitty (`ESC[?u`), colore di sfondo (OSC 11); la conhost di sistema chiede anche `ESC[?9001h` (win32-input-mode). Ipotesi: la conhost di sistema non fa arrivare a Claude le risposte, e Claude resta in attesa.
- **Rimedio previsto.** Il ConPTY aggiornato di Microsoft, come fanno Windows Terminal, VS Code e WezTerm: `conpty.dll` + `OpenConsole.exe` dal pacchetto NuGet ufficiale `Microsoft.Windows.Console.ConPTY` (firmati Microsoft, verificato con Authenticode). `portable-pty` carica `conpty.dll` con `LoadLibrary` se lo trova, e `conpty.dll` avvia `OpenConsole.exe` dalla propria cartella.
- **Versione.** 1.24.260710001 (13/07/2026, x64 e ARM64): la stabile più recente, 1.25.260930003, è del 02/10/2026, meno di due settimane (regola delle versioni di D-22 e D-28).
- **Come nel kit.** Il kit non mette file accanto all'exe (`Dev-Kit.exe` + `data\`): componente obbligatorio `conpty` in `data\apps\<piattaforma>\conpty\` (circa 1,2 MB), SHA-256 dal catalogo NuGet e firma Microsoft; prima della prima console, `SetDllDirectoryW` su quella cartella, così `LoadLibrary("conpty.dll")` la trova. Senza il componente, ConPTY di Windows (comportamento attuale).
- **Stato** (14/10/2026): pacchetto scaricato e firme verificate; la prova con l'app è ferma perché il sistema dei permessi di Claude Code ha bloccato l'esecuzione di binari scaricati: serve il permesso dell'utente (PLAN.md §12).

## D-48 Prove dell'app con il debug remoto di WebView2

- Per provare le finestre senza mani: `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9333` all'avvio dell'app di debug (con Vite su 5173), poi uno script Node che usa il protocollo DevTools (`/json/list`, `Runtime.evaluate`) per chiamare i comandi come la pagina (`window.__TAURI_INTERNALS__.invoke`) e leggere il DOM delle finestre. Le finestre si riconoscono dall'etichetta (`__TAURI_INTERNALS__.metadata.currentWindow.label`): `main` ed `editor-<n>`; quelle dell'editor anche dall'URL (`folder=`).
- Solo per lo sviluppo: la variabile non è mai impostata dall'app. Le prove passano dalle stesse capability della pagina, quindi verificano anche i permessi.
- Più sessioni di lavoro sullo stesso PC: i processi di prova si chiudono per PID, mai per nome (`taskkill /IM Dev-Kit.exe` chiude anche le app degli altri).

## D-49 Ambiente dei figli: elenco di variabili consentite

(Era la prima delle due D-28: rinumerata il 14/10/2026.)

- **Scelta.** I programmi del kit ricevono le variabili del kit più un elenco fisso di variabili di sistema (`SystemRoot`, `ComSpec`, `PATHEXT`, `PROCESSOR_*`, `ProgramFiles*`, `USERNAME`…) e i proxy del PC; mai il resto dell'ambiente dell'utente (per esempio `OneDrive`, `CARGO_HOME`, `JAVA_HOME` del PC). `Command::env_clear()` prima di ogni avvio (`jobs::kit_command`).
- **Motivo.** Terminale e strumenti esclusivi del kit (indicazione del 14/10): una variabile del PC come `CARGO_HOME` o `PYTHONPATH` farebbe usare programmi o pacchetti fuori dal kit.
- **Scartata.** Ereditare tutto e sovrascrivere solo le variabili del kit, come fa `_env.cmd`.
- **Dopo ogni installazione** l'ambiente del kit viene ricostruito (il PATH elenca solo i programmi presenti) e passato all'app, all'editor e alla preparazione: un programma appena installato è visto da console, agenti e progetti senza riavviare l'app. Il server dell'editor già avviato tiene il suo ambiente fino al riavvio (D-54).

## D-50 Progetti: rilevamento e avvio in Rust

- Porting di `core/detectors`, `core/processes.js`, `core/panel/ports.js` e `site.js` del Dev-Kit (`detect/`, `run.rs`, `projects.rs`). Restano le stesse regole: priorità desktop > web > comando, progetti composti (`frontend\`, `backend\`), `panel.json` e il vecchio `pannello.json` con le chiavi italiane e `{porta}`, argomenti di porta per Vite/Next/Nuxt/Angular/Astro e gli altri.
- Le azioni girano con `cmd.exe /d /s /c` e l'ambiente del kit, come nel Dev-Kit (`shell: true`). PowerShell è stato scartato: `npm.ps1` consuma il `--` di `npm run dev -- --port`. `cmd.exe` serve solo da interprete delle righe di comando dei progetti: il terminale dell'utente resta la PowerShell del kit.
- Ogni azione gira in un job annidato in quello dell'app. «Ferma» chiude l'handle del job (`KILL_ON_JOB_CLOSE`): finisce tutto l'albero (npm → node → esbuild) senza `taskkill`, e chiudendo l'app si chiude tutto.
- Porte 7100-7999 ricordate per azione in `data\app\ports.json`, così segnalibri, cookie e localStorage restano validi. La porta scelta dall'utente è bloccata: se è occupata l'avvio fallisce, non si sposta. Una porta è libera se nessuno risponde e se ci si può mettere in ascolto su `127.0.0.1` e su `::1`. Le porte 7000-7099 sono dell'editor: una porta scelta lì viene rifiutata.
- **Rete locale.** Con «Permetti ai server dei progetti di farsi raggiungere dagli altri PC della rete» (Impostazioni, `DEVKIT_ALLOW_LAN=1`) i comandi trovati dall'app ascoltano su `0.0.0.0` al posto di `127.0.0.1`, per tutti i tipi di progetto, non solo Node.
- **Componenti al volo.** Prima di un'azione l'app installa il componente del primo programma di ogni comando (`npm` → Node.js, `php` → PHP…), come in D-59.
- I comandi di `panel.json` sono comandi arbitrari del progetto. Partono solo con un clic su «Avvia», come nel Dev-Kit; la pagina ricorda che le cartelle di `projects` sono attendibili (D-53).
- Clonazione: con il Git del kit (`git clone -- <url> <cartella>`, `GIT_TERMINAL_PROMPT=0`), accettando solo `owner/repo` di GitHub o indirizzi `https://` senza caratteri speciali. Niente `ssh://`, `file://` né argomenti che iniziano con `-`. I repository privati usano l'accesso di `gh` del kit.

## D-51 Server di file nell'exe

- `Dev-Kit.exe serve <cartella> <porta>` sostituisce `static-server.js`: niente Node per un sito statico. Ascolta solo su `127.0.0.1` e risponde solo a un `Host` locale (contro il DNS rebinding). Con la rete locale permessa (`DEVKIT_ALLOW_LAN=1`, D-50) ascolta su tutte le interfacce e accetta in più un `Host` che sia un indirizzo IP, mai un nome. Nega i file e le cartelle nascosti (eccetto `.well-known`), `..` e i collegamenti che portano fuori dalla cartella. Una cartella senza `/` finale viene reindirizzata e ogni risposta ha `Cache-Control: no-store`.

## D-52 Aggiornamento dell'exe

- Implementa D-20 (`self_update.rs`). La fonte sono le release `app-vX.Y.Z` di `nabre/dev-kit2` (non bozze, non pre-release) con l'asset `Dev-Kit-x64.exe` o `Dev-Kit-arm64.exe`. Lo SHA-256 viene dal digest dell'API di GitHub o dal file `.sha256` della release; il workflow della fase 7 deve pubblicarli con questi nomi.
- Il controllo parte 20 s dopo l'avvio, se «Aggiornamenti automatici» è attivo e `DEVKIT_NO_UPDATE` è assente; altrimenti si avvia da Impostazioni. Il file scaricato va in `data\app\update\` e solo un file completo e verificato viene registrato in `update.json`.
- Lo scambio avviene su `RunEvent::Exit` (in Tauri `run()` può non tornare): l'hash viene ricontrollato, poi `Dev-Kit.exe` diventa `Dev-Kit.old.exe` e se lo spostamento fallisce il vecchio torna al suo posto. `Dev-Kit.old.exe` viene eliminato all'avvio successivo; se è ancora in uso da un `sync` o `keyring`, si riprova la volta dopo.
- La versione dell'app è scritta in `data\app\versions\app.txt`, come quella degli altri componenti.

## D-53 Cartelle attendibili e sicurezza dei progetti (revisione di sicurezza)

- Le sottocartelle di `projects` sono attendibili per Claude e Codex (comportamento del Dev-Kit, `trust.js`), così gli agenti non chiedono conferma a ogni progetto. È un rischio accettato: un progetto clonato da fonte sconosciuta ha subito i permessi di un progetto fidato. Lo ricordano la pagina Progetti e Linguaggi e strumenti al primo avvio. Scartato: l'attendibilità progetto per progetto, che il Dev-Kit aveva tolto perché gli agenti la chiedevano di continuo.
- `keyring-before.json` contiene per pochi istanti le credenziali di Antigravity già presenti nel PC, per rimetterle com'erano dopo `agy`. Ora i loro segreti sono cifrati con DPAPI per l'utente di Windows (`CryptProtectData`, senza diritti di amministratore): una chiavetta persa o un file copiato non li rivela. Il file viene comunque cancellato al salvataggio successivo.
- Windows Terminal interpretava `;` negli argomenti come separatore di comandi (una cartella chiamata `a; new-tab cmd` avrebbe aperto un altro programma). Con Windows Terminal del kit (D-68) la cartella è un argomento: ogni `;` viene protetto (`\;`, `wterm::wt_arg`, test `semicolons_are_escaped`). Nella console classica (D-57) la cartella è la cartella di lavoro del processo e un argomento con le virgolette viene rifiutato.

## D-54 Impostazioni

- La pagina Impostazioni scrive `data\app\settings.json` dopo aver controllato i valori: lingua, proxy `http(s)://`, file PEM esistente, Python `3.x`, PHP `8.x`, toolchain `gnu|msvc`, terminale `classic` (console classica, D-68). Un file illeggibile non viene sovrascritto finché l'utente non salva.
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

Dal 14/10/2026 è l'alternativa a Windows Terminal del kit, che è il predefinito (D-68): resta valida per «Console classica di Windows» nelle Impostazioni e come ripiego.

- **Scelta dell'utente** (14/10/2026): via la libreria del terminale «web» (xterm.js) dalle console dell'app; tutto ciò che il kit fa usa solo strumenti del kit o componenti di Windows, mai programmi del PC. Sostituisce D-44, D-45 e D-46; chiude D-47.
- **Come.** `terminal.rs`: la PowerShell del kit, o un agente, parte in `conhost.exe` di System32 avviato direttamente, con il solo ambiente del kit e dentro il job dell'app. Avviata così, la console resta nella finestra classica e Windows non la passa al terminale predefinito del PC (Windows Terminal o altri): verificato su Windows 11 (`ConsoleWindowClass`, nessun `WindowsTerminal.exe`).
- **Senza maniglie standard.** conhost che riceve stdin/stdout fa da pseudo console e scrive la console come testo VT su quelle maniglie, senza finestra. `std::process::Command` le passa sempre (`STARTF_USESTDHANDLES`), quindi `jobs::spawn_console` usa `CreateProcessW` senza maniglie, sospeso, assegnato al job e poi ripreso.
- **Tolti.** `portable-pty`, `@xterm/xterm`, `@xterm/addon-fit`, `terminal.html`, la capability `terminal` e i comandi `terminal_attach`, `terminal_input`, `terminal_resize`. Le finestre dell'app sono di nuovo solo `main` ed `editor-*`.
- **Conseguenze.** Claude Code gira in una console normale di Windows, senza la pseudo console di D-47: i tasti dovrebbero arrivare, da provare a mano con l'app. «Chiudi tutto», la conferma alla chiusura e «Rimuovi l'unità» contano e chiudono le console dai processi `conhost.exe` avviati dall'app. Il titolo della finestra lo decide il programma. Il porting su macOS e Linux dovrà scegliere il terminale di quei sistemi.

## D-58 L'editor fa parte dell'app, con la versione fissata a ogni rilascio

- **Scelta dell'utente** (14/10/2026): l'editor integrato è una parte di Dev-Kit, non un componente da scegliere; versione e aggiornamenti seguono i rilasci dell'app.
- **Come.** Nel catalogo il gruppo nuovo `app` (oggi solo `vscodium-server`) ha `version` e `sha256` fissati per il rilascio: l'installatore scarica sempre quel file da GitHub di VSCodium e non cerca più l'ultima versione. Il componente non si toglie e non si salta; in «Linguaggi e strumenti» compare con gli obbligatori, spuntato e non modificabile (fino al commit `04dd24c` non compariva). Se la versione installata è diversa da quella del rilascio (l'app è stata aggiornata, o il kit viene da un'altra versione), all'avvio l'editor viene installato di nuovo alla versione fissata, senza «Aggiorna tutto»; non diventa mai più nuovo di quella. Prima dello scambio si fermano i processi del vecchio editor.
- **Non blocca.** Solo un editor che manca blocca Editor, Terminale e gli agenti. Un editor di un'altra versione viene aggiornato una volta, all'avvio, e intanto quello installato continua a funzionare: se la sua cartella è in uso (un programma del PC che la tiene aperta), l'editor e le console si aprono lo stesso.
- **Versione** 1.135.06055 (09/09/2026): la 1.140.06880 è del 13/10/2026, meno di due settimane (regola delle versioni di D-22 e D-28). SHA-256 pubblicato da VSCodium e digest di GitHub uguali.
- **Il processo resta.** Le estensioni, i terminali, la ricerca, Git e il debug dell'editor girano nella parte Node.js di VSCodium: senza quel processo resterebbe un editor «web» senza terminale né estensioni normali. Rifarlo in Rust, usare solo Monaco o collegarsi con una named pipe (il workbench usa WebSocket, che i protocolli personalizzati di WebView2 non gestiscono) sono stati scartati. Il processo parte e si ferma con l'app (job), ascolta solo su `127.0.0.1` con il token, e nell'interfaccia si chiama solo «editor».
- **Non nell'exe.** Circa 130 MB compressi: incorporarli renderebbe enorme ogni aggiornamento dell'exe; si scaricano e si verificano al primo avvio, come prima.
- **A ogni rilascio** si aggiornano `version` e `sha256` di `vscodium-server` in `components.json` (una versione di almeno due settimane, checksum di VSCodium uguale al digest di GitHub).

## D-59 Obbligatori solo PowerShell 7 e GitHub CLI; il resto quando serve

- **Scelta dell'utente** (14/10/2026): in «Linguaggi e strumenti» sono obbligatori solo GitHub CLI e il terminale (PowerShell 7 del kit); Git, Node.js e uv diventano dipendenze, installate quando servono. L'editor resta installato sempre come parte dell'app (D-58).
- **Quando si installano** (`commands::ensure`, `needs.rs`):
  - avvio di un agente: l'agente stesso, le sue dipendenze del catalogo (Claude Code → Git) e i programmi dei server MCP di `data\agents\mcp.json` (`npx`/`node` → Node.js, `uvx`/`uv` → uv, `python` → Python);
  - clone di un progetto: Git;
  - azione di un progetto: il componente del primo programma di ogni comando (`npm` → Node.js, `uv` → uv, `php`/`composer` → PHP, `cargo` → Rust...); `Dev-Kit.exe serve` non chiede niente;
  - editor e console: solo gli obbligatori e l'editor.
- **La scelta resta.** Quello che si installa così viene segnato come scelto in `components.txt`, così «Linguaggi e strumenti» non lo toglie al giro dopo. Le scelte predefinite portano con sé le loro dipendenze: Claude Code è predefinito, quindi Git viene installato con lui.
- **ARM64.** Un componente `noArm64` richiesto da un comando dà un errore chiaro invece di un'installazione che fallisce.
- **Kit esistenti.** I file `components.txt` scritti quando Git, Node.js e uv erano obbligatori li hanno a `1`: restano installati finché l'utente non li toglie.
- **Dipendenze obbligatorie dei componenti di sistema** (scelta dell'utente, stesso giorno). GitHub CLI, PowerShell 7 e l'editor sono i componenti di sistema; anche quello che serve a loro è sempre installato (`Catalog::locked`): GitHub CLI richiede Git (`gh repo clone`, `gh pr checkout`, `gh auth setup-git`), l'editor richiede PowerShell 7 (terminali e attività). Git quindi torna sempre presente; nella pagina compare spuntato e non modificabile, con l'etichetta «per GitHub CLI». Node.js e uv restano su richiesta.

## D-60 Scambio di una cartella tenuta aperta da un programma del PC

- **Problema** (14/10/2026). L'aggiornamento dell'editor falliva sempre con «in uso»: lo scambio rinomina la cartella vecchia, e Windows lo rifiuta se un qualsiasi programma tiene aperta anche una sola sua sottocartella. Qui era l'indicizzatore di Cloud Code (`cloudcode_cli.exe`) su `vscodium-server\extensions\terminal-suggest`, trovato enumerando gli handle; un antivirus, l'indicizzatore di Windows o Esplora file aperto sulla cartella fanno lo stesso.
- **Come** (`archive::replace_dir`). Se la rinomina viene rifiutata e nessun programma gira da quella cartella, la cartella vecchia si sposta voce per voce: una sottocartella che non si sposta viene svuotata e resta vuota al suo posto, e la versione nuova la riempie.
- **Limite voluto.** Windows lascia rinominare un exe in esecuzione: senza il controllo dei processi, un programma del componente in esecuzione verrebbe spostato e poi caricherebbe i file nuovi insieme ai vecchi. Con un programma in esecuzione dalla cartella, o un file bloccato, resta `InUse` e tutto torna com'era.

## D-61 Limiti d'uso e consumi degli agenti (`stats/`)

- **Scelta.** Porting di `core/stats/*.js` del Dev-Kit in `src-tauri/src/stats/` (un file per agente, `util.rs` per periodo, token e file di sessione). La pagina Agenti mostra per ogni agente installato i limiti del piano come «% usato» con l'ora in cui si azzerano (soglie 70 e 90 %), i crediti extra di Claude a parte, e i consumi di oggi e degli ultimi 7 giorni locali (token nuovi, dalla cache, in uscita; sessioni; ultima attività; grafico per giorno). Comando `agents_stats` (con `force` per «Aggiorna i consumi»).
- **Fonti, tutte nel kit.** Claude: `api.anthropic.com/api/oauth/usage` con l'accesso di `data\home\.claude\.credentials.json`; Codex: `chatgpt.com/backend-api/wham/usage` con `.codex\auth.json`, altrimenti l'ultimo evento `token_count` delle sessioni; Antigravity: `cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels` con l'accesso salvato in `data\antigravity\keyring.json`. Interfacce non documentate: possono smettere di funzionare dopo un aggiornamento di un agente, e allora la pagina mostra l'errore senza bloccare nulla.
- **Accessi solo letti**, mai rinnovati (un rinnovo sostituirebbe il token che l'agente tiene); ogni token va solo al server del suo fornitore, mai nel log. Proxy e certificati come i download (`net::Net::api_json`, un solo tentativo, 15 s).
- **Server interrogati con parsimonia:** al massimo ogni 2 minuti, ogni 20 s con «Aggiorna»; dopo un 429 nessuna richiesta prima di `Retry-After` (o di un'attesa che raddoppia da 2 a 30 minuti), e restano i limiti letti per ultimi. Il Dev-Kit condivideva questa cache fra le finestre di VSCodium con un file; nell'app c'è un solo processo, quindi resta in memoria.
- **Scartato: il language server locale di Antigravity.** Il Dev-Kit chiedeva le quote anche ad Antigravity in esecuzione, trovandone il processo dalla riga di comando (PowerShell `Get-CimInstance`) e le porte (`netstat`), compreso un Antigravity installato sul PC con un altro account. Nell'app servirebbero letture della riga di comando di processi altrui e un TLS senza verifica verso 127.0.0.1, per dati che il server di Google dà già con l'accesso del kit. Limite: mentre `agy` è aperto il suo accesso sta in Windows e il token del kit può essere scaduto; le quote tornano dopo il salvataggio dell'accesso.
- **Periodo.** I giorni sono quelli locali di Windows (`SystemTimeToTzSpecificLocalTime`), giusti anche al cambio dell'ora legale; niente crate per le date.

## D-62 Editor, terminale e agenti solo nella cartella di un progetto

- **Scelta dell'utente** (14/10/2026): terminale ed editor saldamente collegati alla cartella del progetto; nessun altro luogo offerto all'utente, nemmeno `projects\`.
- **Come.** `editor_open`, `terminal_open` e `agent_launch` vogliono un progetto (prima `null` voleva dire `projects\`). La Home mostra i progetti recenti, ognuno con Editor, Terminale e la scelta dell'agente; la pagina Agenti non ha più «Apri». La finestra dell'editor accetta solo le pagine del server con `folder=` uguale alla cartella del suo progetto (`editor_window::in_project`): un'altra cartella, un workspace o la finestra vuota (`ew=`) vengono rifiutati, anche come nuova finestra. La redirezione del token del server conserva `folder=`.
- **Limiti.** In una shell l'utente può sempre fare `cd` altrove, e nell'editor «Apri file» può ancora mostrare un file fuori dal progetto: la cartella di lavoro e quella aperta restano però quelle del progetto. «Apri la cartella» in Esplora file resta anche per `projects\` e `data\agents` (gestione dei file, non lavoro sul codice).

## D-63 Impostazioni del kit per progetto in `data\app\projects\<progetto>\dev-settings.json`

- **Scelta dell'utente** (14/10/2026): un luogo in `data` per le impostazioni del kit di ogni progetto, con il nome `dev-settings.json`.
- **Perché in `data` e non nel progetto.** Il progetto resta com'è nel suo repository (nessun file nuovo da ignorare in Git), e un backup di `data\` le porta con sé.
- **Contenuto oggi** (`project_settings.rs`): `agent` (l'ultimo agente aperto nel progetto, predefinito nella scelta dell'agente) e `lastOpened` (ordine dei progetti recenti). Le chiavi scritte da una versione più nuova restano. Se il progetto viene eliminato dall'app, la sua cartella in `data\app\projects\` se ne va con lui; le porte restano in `data\app\ports.json` (D-50).

## D-64 Barra del titolo dell'app disegnata dall'interfaccia

- **Scelta dell'utente** (14/10/2026): niente barra di Windows sulla finestra principale, al suo posto quella dell'interfaccia. Solo sulla finestra principale: le finestre dell'editor non hanno IPC (D-14) e tengono la cornice di Windows (cambiato da D-69: usano la barra dell'editor).
- **Come.** `decorations(false)` con l'ombra di Windows; `ui/src/titlebar.tsx` con area di trascinamento (`data-tauri-drag-region`, doppio clic per ingrandire) e i tre pulsanti di Windows. Permessi aggiunti alla sola capability `main`: `core:window:allow-start-dragging`, `allow-internal-toggle-maximize`, `allow-minimize`, `allow-toggle-maximize`, `allow-is-maximized`, `allow-close`. «Chiudi» passa dalla stessa richiesta di chiusura del pulsante di Windows, quindi la conferma con qualcosa aperto resta.

## D-65 Loghi ufficiali di componenti e linguaggi

- **Scelta dell'utente** (14/10/2026): ogni strumento con il suo logo; ogni progetto con le etichette del codice che contiene.
- **Come.** Simple Icons di `react-icons/si` (già una dipendenza), con il colore di ogni marchio; i marchi neri o bianchi prendono il colore del testo, così si leggono in entrambi i temi. Dove Simple Icons non ha il logo: PowerShell (icona Codicon di PowerShell), Windows Terminal (icona Codicon del terminale, D-68), Codex (logo OpenAI di Remix Icon), Antigravity (un'icona neutra: nessun logo ufficiale nei pacchetti, e il sito non ne pubblica uno scaricabile). I linguaggi dei progetti (rilevati da `detect::languages`, i più usati prima) compaiono come etichette con il logo nella riga del progetto e nei progetti recenti della Home.

## D-66 Progetti attendibili anche nell'editor (sostituita da D-67)

Superata il 14/10/2026: l'editor chiede alla prima apertura di ogni progetto (D-67). Resta per la storia delle scelte.


- **Scelta dell'utente** (14/10/2026): niente «Restricted Mode» quando l'editor apre un progetto.
- **Come.** Il server dell'editor parte con `--disable-workspace-trust` (`editor.rs`): il workbench web riceve `enableWorkspaceTrust: false`. Basta spegnere l'attendibilità invece di elencare le cartelle fidate, perché l'editor apre solo cartelle di progetto (D-62). Scartata l'impostazione `security.workspace.trust.enabled`: ha ambito «application» e le impostazioni Machine del server la ignorano (provato: il «Restricted Mode» restava).
- **Rischio accettato**, lo stesso di D-53: le attività, il debug e le estensioni partono subito anche in un progetto clonato da una fonte sconosciuta. La pagina Progetti già avvisa di aprire solo codice di cui ci si fida.

## D-67 L'editor chiede se fidarsi alla prima apertura di un progetto

- **Scelta dell'utente** (14/10/2026), dopo la revisione di sicurezza che segnalava D-66: niente fiducia automatica nell'editor, una conferma alla prima apertura di ogni progetto.
- **Come.** Il server dell'editor parte senza `--disable-workspace-trust` (`editor.rs`): il workbench riceve `enableWorkspaceTrust: true`, chiede con la sua finestra «Ti fidi degli autori?» e apre in «Restricted Mode» i progetti non attendibili (niente attività, debug né estensioni del progetto finché non ci si fida). La risposta resta nella memoria del workbench, nel profilo WebView2 del kit (`data\webview2`); si cambia con «Gestisci attendibilità dell'area di lavoro». Il test del server reale controlla `enableWorkspaceTrust: true` nella pagina.
- **Scartato: la risposta in `dev-settings.json`.** Con un solo server la fiducia è accesa o spenta per tutte le cartelle: una domanda del kit avrebbe chiesto due server (uno per modalità) oppure una doppia domanda, quella del kit e poi quella dell'editor.
- **Limite.** La memoria del workbench dipende dall'origine `127.0.0.1:<porta>`: se la porta 7000 è occupata e il server ne prende un'altra, l'editor chiede di nuovo. Chiede, non si fida da solo.
- Claude Code e Codex restano come in D-53: le cartelle di `projects` sono attendibili per loro.

## D-68 Windows Terminal portabile del kit, predefinito per le console

- **Scelta dell'utente** (14/10/2026): un terminale portabile incluso nel kit, Windows Terminal, come predefinito; la console classica di Windows (D-57) resta come alternativa nelle Impostazioni. Riprende la scelta di D-13 tolta da D-44: Windows Terminal resta solo per Windows (il porting su macOS e Linux sceglierà il terminale di quei sistemi).
- **Componente** `terminal` (gruppo `languages`, predefinito, dipende da `pwsh`): lo zip ufficiale `Microsoft.WindowsTerminal_<v>_(x64|arm64).zip` di `microsoft/terminal` con `.portable` accanto all'exe, così le impostazioni restano in `terminal\settings\` e non in `%LOCALAPPDATA%`. Prima di ogni console l'app riscrive i profili «Dev-Kit» e «Dev-Kit - Git Bash» e gli schemi di colori di VSCodium; il resto del file è dell'utente, e un file che non è JSON semplice o che non si legge (per esempio UTF-16) resta com'è: si crea solo se manca (`wterm.rs`, test `settings_of_the_user_are_never_lost`).
- **Un solo processo, un altro ambiente.** Windows Terminal tiene tutte le finestre in un processo: un secondo avvio passa la richiesta a quello già aperto ed esce subito, e la nuova scheda riceve l'ambiente del terminale, non quello di chi la chiede (verificato nello schema delle impostazioni: non esiste un modo «una finestra per processo»). Quindi l'app non può aspettare il processo che avvia né passargli l'ambiente.
- **Come.** Ogni console è la PowerShell del kit con `data\kit\console.ps1` e il nome di una named pipe nuova (`Dev-Kit-Console-<64 cifre esadecimali casuali>`, solo locale, accesso al solo proprietario: la sicurezza predefinita delle pipe dà la lettura a Everyone, e sul disco senza permessi chiunque può avviare il `pwsh.exe` del kit). L'app aspetta la connessione (30 s), controlla con `GetNamedPipeClientProcessId` che dall'altra parte ci sia proprio `pwsh.exe` del kit, apre quel processo e gli manda l'ambiente del kit in JSON. Lo script tiene solo le variabili del kit, poi avvia l'agente (`& <percorso>`) o il profilo del kit. Quel processo è la console: contato fino alla fine, chiuso da «Chiudi tutto» e da «Rimuovi l'unità», e alla sua fine gli agenti si riallineano (D-40). Se l'ambiente non arriva, la scheda non avvia niente.
- **Ripiego** sulla console classica quando le Impostazioni la chiedono, quando Windows Terminal non è installato, su Windows più vecchio di 10 2004 (build 19041, il minimo di Windows Terminal) o se la console di Windows Terminal non parte.
- **Limiti.** Le schede aperte dall'utente con «+» non passano dall'app: hanno l'ambiente con cui Windows Terminal è partito (quello del kit, ma senza i cambiamenti successivi, per esempio il PATH dopo un'installazione) e non sono contate. Una scheda di un programma che esce con un errore resta aperta su «processo terminato» finché l'utente non la chiude, come fa Windows Terminal.
- **Prove.** `wterm.rs`: unione delle impostazioni, `;` nei percorsi (separa i comandi di Windows Terminal), scambio sulla pipe, DACL della pipe con il solo proprietario; con `DEVKIT_TEST_KIT`, due finestre vere (la seconda passa al terminale già aperto) in una cartella con spazio e `;`: l'ambiente del kit arriva e l'app vede la fine.

## D-69 La barra del titolo dell'editor è quella di VSCodium

- **Scelta dell'utente** (14/10/2026): nelle finestre dell'editor niente barra di Windows; si usa la barra dell'editor stesso, con i suoi menu, coerente con VSCodium desktop.
- **Come, senza IPC (D-14 resta).** `decorations(false)` con l'ombra di Windows (`editor_window.rs`).
  - **Trascinamento**: la barra di VSCodium web ha già `-webkit-app-region: drag` (menu, Command Center e pulsanti `no-drag`). wry accende le regioni non client di WebView2 (`IsNonClientRegionSupportEnabled`), quindi quelle zone si comportano come la barra di Windows: trascinamento, doppio clic per ingrandire, Aero Snap. Nessun codice del kit.
  - **Pulsanti**: VSCodium web crea a destra un `window-controls-container` vuoto (largo 0 senza Window Controls Overlay). Uno script iniettato (`initialization_script`, solo frame principale) lo porta a 138 px come in VSCodium desktop e ci mette tre elementi con le classi dell'editor (`window-icon`, `window-minimize`/`window-max-restore`/`window-close`, `codicon-chrome-*`): icone, colori del tema, rosso al passaggio su Chiudi, stato inattivo e schermo intero vengono dal CSS di VSCodium. Un clic apre `http://127.0.0.1:<porta>/__devkit/window/<azione>` come nuova finestra: `on_new_window` la rifiuta ed esegue l'azione sulla propria finestra. Una pagina può solo ridurre, ingrandire o chiudere la finestra in cui sta, come i pulsanti di Windows. «Chiudi» passa da CloseRequested come la X. L'icona Ingrandisci/Ripristina segue `Resized` (`sync_maximized`, da Rust con `eval`).
  - **Menu nella barra**: nel web VSCodium mette i menu nell'hamburger della barra delle attività (`window.menuBarVisibility` = `compact`). L'impostazione ha ambito «application» e `Machine\settings.json` la ignora (come in D-66); lo script la aggiunge a `configurationDefaults` della configurazione della pagina (`vscode-workbench-web-configuration`) quando il documento diventa «interactive», prima che parta `workbench.js` (script di modulo). Valore `classic`, come VSCodium desktop; resta un predefinito, le impostazioni dell'utente vincono.
- **Ripiego.** Se per circa 1 s (tre controlli ogni 0,5 s) non c'è una barra dell'editor visibile (pagina d'errore di WebView2 con il server fermo, barra spenta dalle impostazioni) e la pagina non è a schermo intero, lo script mostra una barra semplice di 30 px con il titolo trascinabile e gli stessi tre pulsanti (glifi di Segoe Fluent Icons). Sparisce quando la barra dell'editor torna. Senza ripiego una finestra senza cornice non si potrebbe spostare né chiudere.
- **Limiti.** Niente menu dei layout di aggancio al passaggio su Ingrandisci (Windows lo offre solo ai pulsanti veri); Win+Z e Win+frecce funzionano. I pulsanti, come in VSCodium desktop, non si raggiungono con Tab.
- **Prove.** Test `title_bar_buttons` (percorsi accettati, altre porte e percorsi rifiutati, testi tradotti, predefinito dei menu). Nell'app, con WebView2 in debug remoto: tre pulsanti nel contenitore largo 138 px, regione `drag`, menu File…Help nella barra e File che si apre, Ingrandisci e Ripristina con l'icona che cambia, Riduci (`IsIconic`), Chiudi (CloseRequested, poi Destroyed), ripiego che compare con la barra nascosta e sparisce al suo ritorno.

## D-70 Account GitHub del kit, uno per tutto, e pagina GitHub

- **Scelta dell'utente** (14/10/2026): l'estensione `setup/extensions/github-repos` del Dev-Kit (elenco dei repository dell'account, clone, rinomina, eliminazione) diventa una pagina nativa dell'app, non un'estensione dell'editor (coerente con Q4). L'accesso deve essere «un tutt'uno» con quello dell'editor: un solo account GitHub per l'app, Git (editor, console, agenti) e GitHub CLI.
- **L'account è quello di GitHub CLI** (`gh`, componente di sistema sempre installato) in `data\gh` con `--insecure-storage`: viaggia con il kit e non entra nella Gestione credenziali. Gli agenti usavano già `gh`. Scartato l'account del menu Account di VSCodium: nell'editor web a `127.0.0.1` l'estensione `github-authentication` gira nel web worker, dove resta solo il flusso con reindirizzamento verso i client riconosciuti (vscode.dev, github.dev, app desktop), e i suoi segreti sono nei dati di WebView2. Scartato anche Git Credential Manager: il suo token non ha `read:org` e l'app non lo legge senza aprirne la finestra.
- **Accesso.** `gh auth login --hostname github.com --git-protocol https --web --insecure-storage --scopes workflow` senza console (stdin nullo, stderr in pipe, nel job dell'app). Senza terminale `gh` non chiede «Premi Invio»: scrive il codice monouso (`! First copy your one-time code: XXXX-XXXX`) e aspetta l'autorizzazione (flusso «device» di GitHub). L'app mostra il codice, apre `https://github.com/login/device` nel browser del PC (indirizzo fisso, non letto dall'output) e alla fine manda l'evento `github`. «Annulla» ferma `gh`. `workflow` si aggiunge a `repo`, `read:org`, `gist` di `gh`, come chiede `gh auth setup-git`, per poter fare push dei workflow di Actions.
- **Git usa lo stesso account.** Nel `.gitconfig` del kit (`data\home`), per `https://github.com` e `https://gist.github.com`: `helper =` (azzera Git Credential Manager ereditato dalla configurazione di sistema, solo per questi host) e `helper = !gh auth git-credential`. Si trova `gh` nel PATH del kit, senza il suo percorso, quindi l'impostazione resta valida dopo uno spostamento del kit. Non si usa `gh auth setup-git`, che scrive il percorso assoluto. Dopo l'accesso il helper viene scritto. Dopo l'uscita, o all'avvio senza accesso, viene tolto e Git torna com'era. Un helper scritto dall'utente per questi host resta com'è. All'avvio dell'app il helper segue lo stato di `gh`, anche dopo `gh auth login` o `logout` in una console.
- **Pagina GitHub.** Elenco dei repository di `GET /user/repos` (`affiliation=owner,collaborator,organization_member`, 100 per pagina, `Link` per la successiva), raggruppati per proprietario (prima l'account) e in ordine naturale (app2 prima di app10), con ricerca. Clone in `projects\<nome>`, o `<proprietario>-<nome>` se il nome è preso, con il Git del kit: il token passa come intestazione HTTP nell'ambiente (`GIT_CONFIG_*`), solo verso github.com, mai sulla riga di comando né in `.git\config`. Lo stesso vale per «Clona» della pagina Progetti con `owner/repo`. Un repository è «clonato» quando un progetto ha un remote che punta a lui. «Editor» apre quel progetto. Per i repository amministrati ci sono «Rinomina» (`PATCH`: i remote del clone seguono il nuovo nome; la cartella no, perché è l'id del progetto con impostazioni, porte e agente) ed «Elimina da GitHub» (`DELETE`, con conferma; il clone resta). Il token va solo a `https://api.github.com/` (`Net::github_call`), in una sola richiesta senza ripetizioni.
- **Permesso di eliminare.** `delete_repo` non si chiede al primo accesso. Se `X-OAuth-Scopes` dice che manca, o GitHub risponde 403/404, la pagina offre «Concedi il permesso». `gh auth refresh` senza terminale rifiuta, quindi l'app rifà l'accesso con `--scopes workflow,delete_repo`.
- **Non portato** dall'estensione: token personale salvato a parte (sarebbe un secondo accesso), cartella dei clone a scelta (i progetti sono sempre in `projects\`), filtro `githubRepos.affiliation`, cestino per il clone di un repository eliminato (l'app elimina solo cartelle vuote).
- **Limiti.** Il menu Account di VSCodium e le funzioni dell'editor che lo usano («Pubblica su GitHub», «Clona da GitHub» nell'editor) non vedono l'account del kit. Push e pull dell'editor lo usano attraverso Git. Il comportamento di `gh auth login --web` senza console viene dal sorgente di `gh` 2.x: va provato con un accesso vero (checklist in TESTING.md).
- **Prove.** `github.rs`: codice monouso, `hosts.yml`, helper di `gh` riconosciuti, remote https e ssh, pagine `Link`, nomi validi, repository dall'API, clone trovati in `projects\`. Interfaccia: accesso con il codice, elenco ordinato e raggruppato, clone, permesso di eliminare chiesto prima di chiamare GitHub.
