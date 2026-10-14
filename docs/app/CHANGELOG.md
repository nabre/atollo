# Novità

## Fase 4: tutti i componenti, Linguaggi e strumenti, Primo avvio (14/10/2026)

**Fatto**
- Installatori di tutti i 31 componenti del catalogo (specifica in CATALOG.md): agenti (Claude Code con firma Anthropic, Codex, Antigravity), Git, Python (pacchetti dell'utente conservati), Node.js, uv, gh, PHP con Composer, `cacert.pem` e runtime VC++ estratto da `vc_redist` (firma Microsoft), Apache con `kit.conf`, Rust (rustup, voce di registro ripristinata), Go, Java con Maven e Gradle, Kotlin, .NET, Dart, Zig, Julia, Bun, Deno, Typst, TinyTeX, pnpm e yarn, CMake con Ninja, C/C++ (llvm-mingw), Ruby, R (micromamba), PowerShell 7, Windows Terminal, editor, VSCodium desktop (facoltativo).
- Motore: parti multiple per componente, sostituzione di singoli file anche se in uso, firma Authenticode (`signature.rs`), installatori fatti di processi, rimozione di tutto ciò che un componente possiede (cartelle e file in `tools\` o `npm-global`).
- Verifica finale (`checkup.rs`): `Atollo.exe check`, e in fondo a `prepare` per ciò che è stato installato.
- Pagina **Linguaggi e strumenti**: componenti per gruppo, spunte con dipendenze a cascata, dimensioni, versioni, riepilogo di cosa verrà installato e rimosso, "Aggiorna tutto", avanzamento con annullamento.
- **Primo avvio**: senza una scelta salvata la Home mostra la scelta di agenti e linguaggi con l'avviso sull'attendibilità di `projects\`.
- Estensioni consigliate dell'editor (60, regola `@programma`, proposta una volta sola, CodeLLDB da GitHub) installate in background dopo l'avvio dell'editor (D-36).
- Impostazioni avanzate in `settings.json`: versione di Python, ramo di PHP, toolchain di Rust.

**Verificato su questo PC** (kit nuovo in `kit3 (prova) & co`)
- `cargo fmt`, `clippy -D warnings`: puliti; 83 test Rust, 8 test dell'interfaccia.
- Tutti i 31 componenti installati da zero in circa 10 minuti; `Atollo.exe check`: tutti rispondono (Claude Code 2.1.299, Python 3.13.16, PHP 8.5.11, Apache 2.4.69, Rust 1.99.0, Java 25 LTS, .NET 10, R 4.6.1…).
- Rimozione di Java (Kotlin segue), Zig, Typst, pnpm, VSCodium desktop: cartelle, file in `tools\` e versioni tolti; il resto "già aggiornato" senza riscaricare.
- Voce "Rustup" in HKCU: invariata (era del rustup dell'utente).
- Installazione di estensioni con la riga di comando del server verificata (Open VSX).
- Pagina Linguaggi e strumenti nella finestra reale.

**Corretto durante la fase**
- pnpm/yarn non si installavano con `&` nel percorso del kit (`cmd /c` toglie le virgolette): ora npm parte da `node.exe` senza `cmd`.
- La verifica di R falliva (`0xc0000135`): ora usa il lanciatore `r\bin\Rscript.cmd`, che aggiunge le DLL di R.
- pnpm 12 crea anche gli alias `pn` e `pnx`: aggiunti a ciò che si rimuove.

**Limiti noti**
- Rust su un kit in FAT32/exFAT non si installa (rustup vuole i collegamenti fisici); il ripiego del Atollo (toolchain sul disco del PC) non è ancora portato.
- WebView2 "Fixed Version" non si scarica dall'app (D-37).
- Estensioni di un programma tolto restano installate (D-36).

**Da provare a mano**
1. Kit nuovo: al primo avvio compare la scelta; "Installa" scarica gli obbligatori e i linguaggi scelti, con l'avanzamento.
2. Linguaggi e strumenti: togliere un linguaggio installato, "Applica" → riepilogo → "Conferma": viene rimosso.
3. Da un prompt: `Atollo.exe check`.

## Fase 3: editor e terminale (14/10/2026)

**Fatto**
- `editor.rs`: server web di VSCodium su `127.0.0.1`, porta libera 7000-7099, token casuale di 32 byte per sessione in `data\vscodium-server\token` (solo il proprietario, dove il disco ha i permessi), mai sulla riga di comando; avviato come `node.exe out\server-main.js` senza `cmd.exe`, sospeso e messo in un job suo (fermarlo chiude anche extension host e terminali); controllo di salute entro 30 s; log in `data\app\logs\editor.log` con il token mascherato.
- `editor_window.rs`: una finestra per progetto (`editor-<n>`, nessuna capability), cartella dati `data\webview2`, navigazione solo verso il server, `window.open` sul server → nuova finestra dell'editor, link `http(s)` esterni → browser di sistema fuori dal job, altri schemi bloccati; tasti acceleratori e menu contestuale del browser spenti, zoom gestito dall'editor.
- `editor_cfg.rs`: impostazioni del server in `data\vscodium-server\data\Machine\settings.json`: profili del terminale del kit riscritti a ogni avvio ("Atollo" con PowerShell 7, "Atollo - Git Bash", "Atollo - cmd", agenti installati), profili del PC nascosti, profili dell'utente conservati; valori predefiniti solo se mancano; file con commenti lasciato com'è.
- `terminal.rs`: Windows Terminal portabile con il profilo PowerShell del kit e gli schemi "VSCodium Dark/Light Modern"; senza Windows Terminal, PowerShell del kit in una console classica; mai la shell del PC.
- `kit-files/profile.ps1` e `kit-files/bashrc`: prompt "Atollo", strumenti disponibili, moduli e cronologia nel kit.
- Componenti obbligatori mancanti installati da soli prima di aprire editor o console (`SetupTask`); avanzamento in Home e nella barra di stato.
- Interfaccia: Home con "Editor" e "Terminale", pagina Progetti (apri nell'editor, console nella cartella), stato dell'editor e dell'installazione nella barra di stato, avviso per i percorsi del kit che bloccano il controllo dei file (D-33).
- CI: test del server reale (403 senza token, reindirizzamento con cookie, token mascherato, nessun processo rimasto).

**Verificato su questo PC** (kit in `kit2 (prova) & co`)
- `cargo fmt`, `clippy -D warnings`: puliti; 63 test Rust (più il test del server reale), 7 test dell'interfaccia.
- Progetto `demo app (1)` aperto nell'editor integrato (tema Dark Modern); "Terminal: Create New Terminal" apre il profilo "Atollo": PowerShell 7.6.7 del kit, `node` del kit, `USERPROFILE` = `data\home`.
- Pulsante Terminale: Windows Terminal del kit, profilo Atollo, colori VSCodium, `TEMP` nel kit; WT e pwsh figli dell'app.
- Chiusura forzata dell'app: nessun processo rimasto (server, extension host, terminali, Windows Terminal, WebView2).
- Nessuna scrittura di PowerShell nel profilo reale dopo la correzione (`StartupProfileData`, cronologia).

**Corretto durante la fase**
- Pagina bianca nell'editor: `freezePrototype` di Tauri era iniettato anche nelle finestre dell'editor (D-32).
- Pulsante Terminale che apriva `cmd.exe` del PC quando il kit non aveva ancora PowerShell 7 (per esempio con `npm run dev`, dove il kit è `target\debug`): ora installa prima i componenti obbligatori e non ripiega mai su una shell del PC (D-35).
- PowerShell scriveva la cache di avvio di .NET e la cronologia nel profilo reale (D-34).
- Windows Terminal sposta `compatibility.reloadEnvironmentVariables` nei valori predefiniti dei profili: ora l'app la scrive lì.

**Limiti noti**
- Cartelle del kit che iniziano con una cifra, "x" o "u" (es. `D:\2026\kit`): l'editor non vede i file cambiati fuori (difetto di VSCodium/`@parcel/watcher`, D-33); la Home lo segnala.
- Chiudendo una finestra dell'editor con file non salvati non compare la domanda: le modifiche restano nei backup dell'editor e ricompaiono alla riapertura. La domanda alla chiusura dell'app arriva nella fase 6.
- Estensioni consigliate e CodeLLDB: fase 4.
- Resta solo la cartella vuota `%LOCALAPPDATA%\Microsoft\PowerShell`, se non esiste già (D-34).

**Da provare a mano**
1. Home › Editor: si apre VSCodium nella finestra di Atollo; se mancano componenti obbligatori, prima compare l'installazione.
2. Nell'editor: Terminale › Nuovo terminale. Il profilo è "Atollo", `$env:USERPROFILE` punta a `data\home`.
3. Progetti › Terminale su un progetto: Windows Terminal del kit in quella cartella.
4. Chiudere Atollo da Gestione attività (Termina attività): non resta nessun `node.exe`, `pwsh.exe` o `WindowsTerminal.exe` del kit.

## Fase 2: motore di installazione (14/10/2026)

**Fatto**
- `catalog.rs` + `catalog/components.json`: i 32 componenti (agenti e linguaggi presi da nabre/atollo, più PowerShell 7, Windows Terminal, editor, WebView2) in ordine di installazione; scelta in `data\app\components.txt` nel formato del Atollo (`id=1|0`, CRLF); dipendenze a cascata (Apache con PHP, Kotlin con Java); versioni in `data\app\versions\`.
- `net.rs`: HTTPS con il TLS di Windows e i certificati del suo archivio (funziona con l'ispezione SSL aziendale), proxy dalle impostazioni, dall'ambiente o di Windows; download in `.part` con 3 tentativi; API di GitHub.
- `verify.rs`: SHA-256, SHA-512, integrità npm; un file che non corrisponde viene cancellato.
- `archive.rs`: zip, tar.gz, 7z (anche dentro un `.7z.exe`), con rifiuto dei percorsi che escono dalla cartella; scambio delle cartelle con ripristino, "in uso" se un file è aperto.
- `setup_runner.rs`: ordine dei passi, una preparazione alla volta per kit, rimozione dei componenti tolti, annullamento; un componente che fallisce non ferma gli altri.
- Installatori: **PowerShell 7**, **Windows Terminal** (modalità portabile), **Git** (con `post-install` e configurazione: `credential.helper=manager`, `.gitconfig` del kit), **Node.js LTS**, **editor** (server web di VSCodium, SHA-256 di VSCodium confrontato con quello di GitHub).
- Riga di comando: `Atollo.exe prepare [--only a,b] [--skip a,b] [--update-all] [--force]` e `Atollo.exe components`, con i marcatori del Atollo (`── N. Nome ──`, `√`, `x`, `!`).
- CI: installazione reale dei cinque componenti e controllo degli aggiornamenti nel job x64.

**Verificato su questo PC**
- `cargo fmt`, `clippy -D warnings`: puliti; `cargo test`: 49 test verdi.
- `prepare --only pwsh,terminal,node` in 21 s e `--only git,vscodium-server` in 34 s, da una cartella `kit2 (prova) & co`: Node v24.21.0, PowerShell 7.6.7, Windows Terminal 1.25.2733.0 (con `.portable`), Git 2.56.0.2 (Bash funziona, post-install eseguito), editor 1.140.06880 (`bin\codium-server.cmd` presente). Spazio occupato: circa 1,1 GB.
- `--update-all`: "già aggiornato" senza riscaricare.
- PowerShell aperto durante `--force`: "in uso", cartella e versione intatte, nessun residuo.
- Tre preparazioni consecutive: nessun file nuovo in `%LOCALAPPDATA%`, `%APPDATA%` o `%TEMP%` reali dovuto a Atollo (i file trovati erano di Chrome, Office, Notion, OneDrive, Teams, aperti sul PC).

**Corretto durante la fase**
- Git: dopo l'estrazione diretta del 7z mancava il `post-install.bat` che l'SFX esegue da solo; aggiunto. Lo script esce sempre con 1 (si cancella da solo): il controllo ora guarda se lo script è sparito.
- `--skip` non esclude più i componenti obbligatori (regola degli script del kit).

**Non ancora fatto**
- Gli altri 27 componenti risultano "non ancora disponibile in questa versione" (fase 4).
- Firma Authenticode (serve a Claude Code, VC++ e al ripiego di Git senza API di GitHub): fase 4.
- Rimozione dei componenti senza cartella propria (uv, gh in `tools\`, pnpm): fase 4.
- Proxy con configurazione automatica (PAC): non supportato, va impostato a mano.

**Da provare a mano**
1. In una cartella vuota con `Atollo.exe`: `Atollo.exe prepare --only pwsh,terminal,node,git,vscodium-server` da un prompt.
2. `Atollo.exe components`: i cinque risultano installati con la versione.
3. Aprire `data\apps\win32-x64\pwsh\pwsh.exe`, lasciarlo aperto e lanciare `Atollo.exe prepare --only pwsh --force`: deve dire "in uso" e non toccare nulla.
4. Dietro un proxy aziendale: impostarlo in `data\app\settings.json` (`"network": {"httpsProxy": "http://host:porta"}`) e ripetere il punto 1.

## Fase 1: scheletro (14/10/2026)

**Fatto**
- Progetto Tauri 2.12.0 (Rust) con interfaccia React 19 + TypeScript + Tailwind 4 + react-icons, in tema VSCodium (Dark Modern / Light Modern secondo Windows, icone Codicons).
- `kit.rs`: radice = cartella di `Atollo.exe`; creazione di `data\` e `projects\` soltanto; piattaforma `win32-x64`/`win32-arm64`; rilevamento dello spostamento (`data\app\root.txt`, `old-root.txt`); regole dei nomi dei progetti.
- `env.rs`: porting di `_env.cmd` (≈90 variabili) sulla nuova struttura; ambiente costruito da un elenco di variabili di sistema, PATH = kit + `System32`, `Wbem`, `WindowsPowerShell\v1.0`; il processo dell'app punta profilo, AppData, temp e dati WebView2 in `data\`.
- `settings.rs`: `data\app\settings.json` (lingua, proxy, certificati, rete locale, chiave Gemini, aggiornamenti) al posto di `proxy.cmd`; un file illeggibile non viene sovrascritto.
- `jobs.rs`: Job Object con `KILL_ON_JOB_CLOSE`, l'app ci entra all'avvio.
- `instance.rs`: una sola istanza per kit (mutex + pipe), il secondo avvio porta in primo piano la finestra.
- `webview.rs`: WebView2 di sistema, altrimenti "Fixed Version" in `data\apps\win32-<arch>\webview2\`, altrimenti messaggio con il link; mai il bootstrapper.
- `cli.rs`: `Atollo.exe env | version | help`.
- `i18n.rs`: catalogo it/en/fr/de dei messaggi Rust, con test di completezza.
- Sicurezza: comandi limitati con `AppManifest::commands`, capability solo per `main`, controllo dell'etichetta; test che verifica il rifiuto da una finestra `editor`.
- Manifesto dell'exe con `asInvoker` (nessuna richiesta di amministratore).
- CI `.github/workflows/app.yml`: lint, formattazione, test, build x64 e ARM64 senza installer, controllo del manifesto, prova di fumo.

**Verificato**
- `cargo fmt --check`, `cargo clippy --all-targets -D warnings`: puliti. `cargo test`: 27 test verdi. Frontend: ESLint, Prettier, `tsc`, 4 test Vitest verdi.
- Exe di release (3,3 MB) avviato da una cartella `kit test (1) & co`: crea solo `data\` e `projects\`; `Atollo.exe env` corretto; secondo avvio esce in circa 1 s portando in primo piano il primo; dopo la chiusura forzata nessun `msedgewebview2.exe` rimasto; nessuna cartella nuova in `%LOCALAPPDATA%` reale.

**Corretto durante la fase**
- Il test IPC creava `%LOCALAPPDATA%\io.github.nabre.atollo` (finestre di prova senza cartella dati): ora usano una cartella temporanea. La cartella vuota creata dai test è stata rimossa.

**Non ancora fatto (fasi successive)**
- Testi dell'interfaccia solo in italiano: i cataloghi dell'interfaccia arrivano nella fase 6.
- La build ARM64 è solo nella CI (il target non è installato su questo PC): da verificare al primo avvio della CI.
- Compilazione con il Rust `gnu` di un kit: non provata (questo PC usa MSVC).

**Da provare a mano**
1. Copiare `target\release\Atollo.exe` in una cartella vuota (anche su chiavetta, con spazi o `&` nel nome) e avviarlo con doppio clic: compare la finestra in stile VSCodium con la cartella del kit.
2. Avviarlo una seconda volta: non si apre una seconda finestra, torna in primo piano la prima.
3. Spostare la cartella (o cambiare lettera all'unità) e riavviare: la Home avvisa che il kit è stato spostato.
4. Da un prompt: `Atollo.exe env` mostra l'ambiente del kit.
5. Cambiare il tema di Windows (chiaro/scuro): la finestra segue.
