# Novità

## Fase 2: motore di installazione (14/10/2026)

**Fatto**
- `catalog.rs` + `catalog/components.json`: i 32 componenti (agenti e linguaggi presi da nabre/dev-kit, più PowerShell 7, Windows Terminal, editor, WebView2) in ordine di installazione; scelta in `data\app\components.txt` nel formato del Dev-Kit (`id=1|0`, CRLF); dipendenze a cascata (Apache con PHP, Kotlin con Java); versioni in `data\app\versions\`.
- `net.rs`: HTTPS con il TLS di Windows e i certificati del suo archivio (funziona con l'ispezione SSL aziendale), proxy dalle impostazioni, dall'ambiente o di Windows; download in `.part` con 3 tentativi; API di GitHub.
- `verify.rs`: SHA-256, SHA-512, integrità npm; un file che non corrisponde viene cancellato.
- `archive.rs`: zip, tar.gz, 7z (anche dentro un `.7z.exe`), con rifiuto dei percorsi che escono dalla cartella; scambio delle cartelle con ripristino, "in uso" se un file è aperto.
- `setup_runner.rs`: ordine dei passi, una preparazione alla volta per kit, rimozione dei componenti tolti, annullamento; un componente che fallisce non ferma gli altri.
- Installatori: **PowerShell 7**, **Windows Terminal** (modalità portabile), **Git** (con `post-install` e configurazione: `credential.helper=manager`, `.gitconfig` del kit), **Node.js LTS**, **editor** (server web di VSCodium, SHA-256 di VSCodium confrontato con quello di GitHub).
- Riga di comando: `Dev-Kit.exe prepare [--only a,b] [--skip a,b] [--update-all] [--force]` e `Dev-Kit.exe components`, con i marcatori del Dev-Kit (`── N. Nome ──`, `√`, `x`, `!`).
- CI: installazione reale dei cinque componenti e controllo degli aggiornamenti nel job x64.

**Verificato su questo PC**
- `cargo fmt`, `clippy -D warnings`: puliti; `cargo test`: 49 test verdi.
- `prepare --only pwsh,terminal,node` in 21 s e `--only git,vscodium-server` in 34 s, da una cartella `kit2 (prova) & co`: Node v24.21.0, PowerShell 7.6.7, Windows Terminal 1.25.2733.0 (con `.portable`), Git 2.56.0.2 (Bash funziona, post-install eseguito), editor 1.140.06880 (`bin\codium-server.cmd` presente). Spazio occupato: circa 1,1 GB.
- `--update-all`: "già aggiornato" senza riscaricare.
- PowerShell aperto durante `--force`: "in uso", cartella e versione intatte, nessun residuo.
- Tre preparazioni consecutive: nessun file nuovo in `%LOCALAPPDATA%`, `%APPDATA%` o `%TEMP%` reali dovuto a Dev-Kit (i file trovati erano di Chrome, Office, Notion, OneDrive, Teams, aperti sul PC).

**Corretto durante la fase**
- Git: dopo l'estrazione diretta del 7z mancava il `post-install.bat` che l'SFX esegue da solo; aggiunto. Lo script esce sempre con 1 (si cancella da solo): il controllo ora guarda se lo script è sparito.
- `--skip` non esclude più i componenti obbligatori (regola degli script del kit).

**Non ancora fatto**
- Gli altri 27 componenti risultano "non ancora disponibile in questa versione" (fase 4).
- Firma Authenticode (serve a Claude Code, VC++ e al ripiego di Git senza API di GitHub): fase 4.
- Rimozione dei componenti senza cartella propria (uv, gh in `tools\`, pnpm): fase 4.
- Proxy con configurazione automatica (PAC): non supportato, va impostato a mano.

**Da provare a mano**
1. In una cartella vuota con `Dev-Kit.exe`: `Dev-Kit.exe prepare --only pwsh,terminal,node,git,vscodium-server` da un prompt.
2. `Dev-Kit.exe components`: i cinque risultano installati con la versione.
3. Aprire `data\apps\win32-x64\pwsh\pwsh.exe`, lasciarlo aperto e lanciare `Dev-Kit.exe prepare --only pwsh --force`: deve dire "in uso" e non toccare nulla.
4. Dietro un proxy aziendale: impostarlo in `data\app\settings.json` (`"network": {"httpsProxy": "http://host:porta"}`) e ripetere il punto 1.

## Fase 1: scheletro (14/10/2026)

**Fatto**
- Progetto Tauri 2.12.0 (Rust) con interfaccia React 19 + TypeScript + Tailwind 4 + react-icons, in tema VSCodium (Dark Modern / Light Modern secondo Windows, icone Codicons).
- `kit.rs`: radice = cartella di `Dev-Kit.exe`; creazione di `data\` e `projects\` soltanto; piattaforma `win32-x64`/`win32-arm64`; rilevamento dello spostamento (`data\app\root.txt`, `old-root.txt`); regole dei nomi dei progetti.
- `env.rs`: porting di `_env.cmd` (≈90 variabili) sulla nuova struttura; ambiente costruito da un elenco di variabili di sistema, PATH = kit + `System32`, `Wbem`, `WindowsPowerShell\v1.0`; il processo dell'app punta profilo, AppData, temp e dati WebView2 in `data\`.
- `settings.rs`: `data\app\settings.json` (lingua, proxy, certificati, rete locale, chiave Gemini, aggiornamenti) al posto di `proxy.cmd`; un file illeggibile non viene sovrascritto.
- `jobs.rs`: Job Object con `KILL_ON_JOB_CLOSE`, l'app ci entra all'avvio.
- `instance.rs`: una sola istanza per kit (mutex + pipe), il secondo avvio porta in primo piano la finestra.
- `webview.rs`: WebView2 di sistema, altrimenti "Fixed Version" in `data\apps\win32-<arch>\webview2\`, altrimenti messaggio con il link; mai il bootstrapper.
- `cli.rs`: `Dev-Kit.exe env | version | help`.
- `i18n.rs`: catalogo it/en/fr/de dei messaggi Rust, con test di completezza.
- Sicurezza: comandi limitati con `AppManifest::commands`, capability solo per `main`, controllo dell'etichetta; test che verifica il rifiuto da una finestra `editor`.
- Manifesto dell'exe con `asInvoker` (nessuna richiesta di amministratore).
- CI `.github/workflows/app.yml`: lint, formattazione, test, build x64 e ARM64 senza installer, controllo del manifesto, prova di fumo.

**Verificato**
- `cargo fmt --check`, `cargo clippy --all-targets -D warnings`: puliti. `cargo test`: 27 test verdi. Frontend: ESLint, Prettier, `tsc`, 4 test Vitest verdi.
- Exe di release (3,3 MB) avviato da una cartella `kit test (1) & co`: crea solo `data\` e `projects\`; `Dev-Kit.exe env` corretto; secondo avvio esce in circa 1 s portando in primo piano il primo; dopo la chiusura forzata nessun `msedgewebview2.exe` rimasto; nessuna cartella nuova in `%LOCALAPPDATA%` reale.

**Corretto durante la fase**
- Il test IPC creava `%LOCALAPPDATA%\io.github.nabre.devkit2` (finestre di prova senza cartella dati): ora usano una cartella temporanea. La cartella vuota creata dai test è stata rimossa.

**Non ancora fatto (fasi successive)**
- Testi dell'interfaccia solo in italiano: i cataloghi dell'interfaccia arrivano nella fase 6.
- La build ARM64 è solo nella CI (il target non è installato su questo PC): da verificare al primo avvio della CI.
- Compilazione con il Rust `gnu` di un kit: non provata (questo PC usa MSVC).

**Da provare a mano**
1. Copiare `target\release\Dev-Kit.exe` in una cartella vuota (anche su chiavetta, con spazi o `&` nel nome) e avviarlo con doppio clic: compare la finestra in stile VSCodium con la cartella del kit.
2. Avviarlo una seconda volta: non si apre una seconda finestra, torna in primo piano la prima.
3. Spostare la cartella (o cambiare lettera all'unità) e riavviare: la Home avvisa che il kit è stato spostato.
4. Da un prompt: `Dev-Kit.exe env` mostra l'ambiente del kit.
5. Cambiare il tema di Windows (chiaro/scuro): la finestra segue.
