# Novità

## Adminer per MariaDB, come phpMyAdmin (15/10/2026)

**Fatto** (D-85)
- **«Apri Adminer»** nella pagina di MariaDB: il database nel browser, come phpMyAdmin di EasyPHP e XAMPP (tabelle, righe da modificare, query SQL, importazione ed esportazione), già entrato come `root`. Al primo clic installa Adminer (1 MB, nuovo componente nel gruppo database) e avvia MariaDB se è fermo.
- La pagina di MariaDB non ha più la scheda «Contenuto»: c'è Adminer. Resta per MongoDB.
- Lo serve il PHP del kit solo su 127.0.0.1, con un token nuovo a ogni avvio: senza token nessuno entra, nemmeno gli altri utenti del PC. La password di `root` non passa dal browser. Si ferma con MariaDB o con l'app.

**Verificato**: 264 test Rust, clippy e fmt puliti; tipi, ESLint e 27 test dell'interfaccia. Installazione reale da GitHub (SHA-256 e controllo `php -l`: 6.1.2) e test reale con MariaDB 12.3 del kit: 403 senza token o con quello sbagliato, ingresso con il token, elenco dei database, server riusato, fermato con MariaDB. Da provare nell'app: il pulsante e il browser che si apre.

## SDK di Android nel kit e Capacitor (15/10/2026)

**Fatto** (D-84)
- **Android SDK** in Strumenti: strumenti a riga di comando ufficiali, `adb`, ultima piattaforma e build-tools, tutto in `datapps`; `ANDROID_HOME` impostato per Gradle. Serve Java. Sceglierlo accetta i termini dell'SDK di Google.
- Nel wizard le basi **Android** e **NativePHP Mobile** usano l'SDK del kit; nuova aggiunta **Android**: con Vite è Capacitor (la stessa app web come app Android), con Tauri è Tauri mobile (con l'NDK, scaricato solo per questa combinazione).

**Verificato**: 261 test Rust (nuovi: zip e SHA-256 dalla pagina di download, scelta dei pacchetti stabili, Capacitor nelle combinazioni) e lettura reale della pagina di Google; clippy, fmt, tipi, ESLint e 27 test dell'interfaccia. Da provare nell'app: l'installazione reale (circa 600 MB) e una build con un telefono collegato.

## Nuovo progetto da framework (15/10/2026)

**Fatto** (D-83)
- **Wizard «Nuovo progetto»** (icona sopra Progetti): una base (Laravel, Symfony, Vite, Next.js, Electron, Tauri, Android) e le aggiunte che vanno con lei: interfaccia (React, Vue, Svelte, Livewire), stili (Tailwind CSS, Bootstrap, Bulma), database (SQLite, MySQL/MariaDB, MongoDB), app desktop e Android con NativePHP, prove, formattazione, Vite per Symfony. Il kit installa gli strumenti che servono, esegue i comandi ufficiali mostrandone l'output, crea il database e scrive il `.env`, salva le azioni del progetto.
- **Progetti esistenti**: si può riempire un progetto vuoto o uno con file ma senza Git (i file che ci sono restano).
- **Impostazioni come Progetti**: sezioni e framework nella barra laterale, ogni voce in una scheda, ricerca; le istruzioni di ogni framework si modificano e si ripristinano.

**Verificato**: 259 test Rust (nuovi: catalogo valido, combinazioni, salvataggio delle sole modifiche, progetti esistenti, unione dei file, `.env`), clippy e fmt puliti; tipi, ESLint e 27 test dell'interfaccia. Da provare nell'app: la creazione reale di ciascuna base (scarica da npm, Packagist, GitHub).

## Rilasci di GitHub senza limite e download a mano (15/10/2026)

**Fatto** (D-82)
- **Limite dell'API di GitHub**: quando le 60 chiamate l'ora sono finite, l'ultima versione e lo SHA-256 dei file si leggono dalle pagine di github.com. GitHub CLI e PowerShell 7 si installano anche senza accesso a GitHub.
- **Download a mano**: se un download non riesce, sotto «non riuscito» compare il link al file; scaricato dal browser, si trascina nel riquadro o si sceglie con «sfoglia…». Il kit lo conserva in `data\downloads`, riprova da solo e lo usa solo se il suo SHA-256 è quello pubblicato.

**Verificato**: 247 test Rust (nuovi: asset dalla pagina dei rilasci, limite riconosciuto, file aggiunti a mano), clippy e fmt puliti; tipi, ESLint e 23 test dell'interfaccia; lettura reale della pagina di PowerShell. Da provare nell'app: trascinare un file nel riquadro e «sfoglia…».

## Versioni affiancate per Java, .NET, Ruby e altri (15/10/2026)

**Fatto** (D-81)
- **Java su PC ARM64**: si installa la JDK LTS che esiste per ARM64 (oggi la 21); Maven resta alla 3.x.
- **Gradle con ogni Java**: il comando `gradle` del kit parte con la JDK più recente; il progetto compila con la sua tramite le toolchain, che trovano le JDK del kit (anche Maven, con `toolchains.xml`) e mai quelle del PC.
- **.NET**: ogni versione vede SDK e runtime delle altre, così un'app net8 parte anche sotto l'SDK 10.
- **CMake** usa Ninja e configura anche i progetti vecchi; **`make`** c'è con C/C++.
- **uv** usa solo i suoi Python, dentro il kit; mai quelli del PC.
- **Ruby**: le gem stanno con la loro versione di Ruby (e passano agli aggiornamenti di patch); versioni di Ruby da scaricare.
- **TinyTeX** rimette i pacchetti aggiunti dopo ogni aggiornamento.
- Sui PC ARM64 con Windows 10 i programmi solo x64 risultano «richiede Windows 11 su ARM64»; Julia è solo x64.
- **Git** non è più obbligatorio: arriva quando serve (Claude Code, un clone, un progetto).

**Verificato**: 241 test Rust (nuovi: JDK per Gradle e Maven, .NET affiancato con junction, rimozione che non segue le junction), clippy e fmt puliti; tipi, ESLint e 23 test dell'interfaccia; `gradle.cmd` provato senza Gradle. Da provare nell'app: un progetto Gradle su Java 11, un'app net8 con .NET 10 predefinito, `make` in un progetto C, l'aggiornamento di TinyTeX.

## Matrice delle relazioni (15/10/2026)

**Fatto**
- **Relazioni fra strumenti** (D-81): una funzione dell'app genera la matrice di ciò che ogni strumento richiede e con quali versioni (Composer → PHP ≥ 7.2.5, Gradle → Java 17-27, Apache → PHP 8, pnpm → Node.js ≥ 18.12, CMake → compilatore C…), dei runtime che il PC deve fornire e delle versioni chieste dai progetti (`package.json`, `.nvmrc`, `.python-version`, `pyproject.toml`, `composer.json`, `go.mod`, `global.json`, `pom.xml`, `pubspec.yaml`, `Project.toml`, `CMakeLists.txt`…). Salvata in `data\app\relations.json`, si ricalcola solo quando cambia qualcosa.
- **Nel punto giusto**: Visual C++ x64 e WebView2 (e l'emulazione x64 su ARM64) in «Obbligatori»; un legame non soddisfatto nella riga dello strumento; «non per Gradle» accanto alle versioni su cui un altro strumento non gira; la matrice completa nella sezione «Relazioni», con «Usa» o «Scarica e usa» per le versioni chieste dai progetti.
- **Il kit usa la versione più recente**, senza scelta. **Un progetto nuovo la fissa** alla creazione, così non cambia quando ne arrivano di nuove; **un progetto clonato** installa i linguaggi che usa e prende (o scarica) la versione compatibile con quelle che dichiara.
- «latest» sulla versione più recente offerta; colonna della versione vuota per i linguaggi con più versioni.
- Apache carica il PHP in uso (modulo ed estensioni della stessa versione); PHP solo dalla 8.
- **Typst di nuovo con più versioni** (le 0.x cambiano la sintassi; la matrice legge `typst.toml`), con l'elenco delle versioni da scaricare come Kotlin.
- **pnpm e yarn alla versione del progetto**: pnpm 12 nativo usa da solo la versione di `packageManager`; yarn passa da corepack (Yarn 4 dove serve, la classica altrove). Corretto: con il vecchio installatore `pnpm.cmd` di pnpm 12 non partiva.
- **R con più versioni**, ognuna con la sua libreria di pacchetti; l'ambiente nasce nella sua cartella finale; la matrice legge `renv.lock` e `DESCRIPTION`.
- **MariaDB e MongoDB con più versioni**: nella pagina Database la versione del server (a server fermo); ogni versione ha i suoi dati, copiati in avanti dalla precedente al primo avvio (MongoDB una maggiore alla volta, con la FCV aggiornata); i dati esistenti passano nella cartella della loro versione. La matrice legge le immagini di `compose.yaml`.
- **«locked»**: una versione usata da progetti ha il lucchetto con l'elenco dei progetti e non si rimuove; le altre hanno il cestino.

**Verificato**: 239 test Rust (nuovi: intervalli di versione, matrice, versioni dei progetti, dati dei server per versione), clippy e fmt puliti; tipi, ESLint e 23 test dell'interfaccia; installazione di pnpm e yarn provata in una cartella di prova; anteprima nel browser. Da provare nell'app: la matrice su un kit reale con progetti, il primo avvio con i dati di MariaDB già presenti, R scaricato in due versioni.

## Una cartella per versione, Linguaggi e Strumenti separati (15/10/2026)

**Fatto**
- **Una cartella per versione** (D-80): ogni versione di un linguaggio sta in `data\apps\<piattaforma>\<id>\<major.minor>`, predefinita compresa; niente più `versions\`. Un aggiornamento a una minore nuova si installa accanto e la vecchia resta, senza spostare cartelle in uso; una patch sostituisce la cartella della sua linea. Al primo avvio il kit sposta da solo le cartelle della struttura precedente (una cartella in uso viene spostata all'avvio dopo) e corregge il PHP in `data\apache\kit.conf` (`${CP_PHP_FWD}`).
- **«Aggiorna tutto» aggiorna ogni versione** alla sua ultima patch, non solo la predefinita.
- **Linguaggi e Strumenti** nella pagina Strumenti: i linguaggi con più versioni (Python, Node.js, PHP, Go, Java, Kotlin, .NET, Dart, Zig, Julia, Bun, Deno, Ruby) separati dagli strumenti sempre all'ultima versione. Typst, CMake/Ninja e C/C++ passano fra questi: le loro versioni salvate si tolgono.
- **Versioni nella riga del linguaggio**: la versione del kit, quelle scaricate (predefinita, del kit, progetti che le usano), il download di un'altra; «N versioni» nella colonna della versione. Il gruppo «Versioni» non c'è più.
- **La casella vale «almeno una versione»**: ogni versione si toglie anche da sola, predefinita compresa (la più recente rimasta prende il suo posto); l'ultima se ne va togliendo la spunta.
- **Avanzamento nella lista**: niente più riquadro in cima alla pagina Strumenti. Ogni riga mostra il suo stato («in coda», «download N%» con la barra, «non riuscito» con il motivo), anche le versioni in download nella lista delle versioni; in basso il passo in corso e «Annulla».
- **Composer con ogni versione di PHP**: ogni cartella di PHP ha il suo Composer, compatibile con quella versione, e nel terminale `composer` usa quello del PHP in uso. Nei kit esistenti quello di `tools\` viene copiato in ogni versione di PHP all'avvio.
- **Riga «Composer»** sotto PHP, con la sua versione e l'elenco «con PHP 8.4 → 2.8.12»: si installa, si aggiorna e si toglie con PHP.
- **«+» per aggiungere una versione**: sopra la lista delle versioni, apre l'elenco delle versioni offerte e non ancora scaricate, per versione maggiore; al posto del menu con «Scarica».
- **La lista è la scelta**: niente più menu a tendina per la versione del kit; ogni versione della lista ha la sua scelta, sempre visibile anche con una versione sola.
- **Versioni di Kotlin da scaricare** (release stabili di JetBrains/kotlin su GitHub).
- **Meno «http status: 403» da GitHub**: installazioni ed elenchi delle versioni usano anche l'account GitHub del kit (GitHub CLI), non solo `GITHUB_TOKEN` del PC; senza accesso GitHub concede 60 richieste l'ora, e «Aggiorna tutto» ne fa molte.

**Verificato**: 227 test Rust (nuovi: migrazione anche interrotta, ripristino della predefinita, rimozione della predefinita, ambiente dalla cartella della versione, Composer per versione di PHP e dai kit precedenti), clippy e fmt puliti; tipi, ESLint e 23 test dell'interfaccia; elenco degli strumenti di `profile.ps1` provato in PowerShell; anteprima nel browser della pagina Strumenti, anche con un task in corso. Da provare nell'app: avvio su un kit con la struttura precedente (Node.js e PHP con Composer installati, Apache del kit), «Aggiorna tutto» con una versione salvata, `composer --version` nel terminale di un progetto con un altro PHP.

## Menu delle versioni propositivo (15/10/2026)

**Fatto**
- **Versioni da scaricare nei menu** (D-80): il menu della versione, nel kit e in ogni progetto, propone oltre alle versioni scaricate anche le minori offerte dalla fonte, raggruppate per versione maggiore («da scaricare»). Sceglierne una la scarica subito e la imposta; fino alla fine del download si usa la predefinita, e sotto il menu c'è lo stato (in corso con percentuale, non riuscito con «Riprova», «Scarica»).
- **Elenco delle versioni conservato** in `data\app\version-lines.json`: i menu funzionano anche offline; si aggiorna da solo se ha più di un giorno, o col pulsante accanto al menu.
- **Chi usa cosa**: in Strumenti › Versioni ogni versione scaricata mostra predefinita, quella del kit e i progetti che l'hanno scelta; rimuoverne una avvisa chi la usa.
- La pagina del progetto mostra tutti i linguaggi installati che hanno più versioni, non solo quelli con versioni già conservate.

**Verificato**: test Rust delle versioni (nuovo `offered_lines_and_downloads`), clippy e fmt puliti; tipi, ESLint e 23 test dell'interfaccia; anteprima nel browser. Da provare nell'app: scelta di una versione da scaricare da un progetto.

## Versioni dei linguaggi per il kit e per ogni progetto (15/10/2026)

**Fatto**
- **Versioni conservate** (D-80): per Python, Node.js, PHP, Go, Java, .NET e gli altri linguaggi, un aggiornamento che cambia la versione minore conserva la vecchia in `data\apps\<piattaforma>\versions\<id>\<major.minor>`; una patch sostituisce come prima.
- **Scaricare altre versioni** dalla pagina Strumenti › Versioni (Node.js, Python, Go, Java, .NET, Zig, PHP), rimuoverle e scegliere quella che usa tutto il kit.
- **Versione per progetto**: pagina del progetto › Componenti › Versioni. Vale per terminale, azioni e agenti del progetto e per i terminali PowerShell dell'editor (`versions.json` letto da `profile.ps1`).
- `php.ini` trova le estensioni con `${CP_PHP}`, la cartella del PHP in uso.

**Verificato**: 224 test Rust, clippy e fmt puliti; ESLint, tipi e 23 test dell'interfaccia; profilo provato in PowerShell; anteprima nel browser. Da provare nell'app: download reale di una linea e terminale dell'editor in un progetto con versione scelta.

## Contenuto dei database e pagine più chiare (15/10/2026)

**Fatto**
- **Contenuto dei server** (D-79): nella pagina Database ogni server avviato ha la scheda «Contenuto»: database, tabelle o collezioni con il numero di righe, e le righe 100 alla volta, in sola lettura. Connessione, impostazioni e file nella scheda «Dettagli».
- **Click della pagina Database pronti**: elenco dei server, stato degli agenti e impostazioni dei server si leggono fuori dal thread della finestra (prima, ogni 3-5 secondi, la finestra restava ferma finché non rispondevano). L'esploratore non si chiude più se un controllo dello stato manca una volta.
- SQLite non compare più fra i server: non ne ha uno. Stato dei server con il pallino colorato.
- **Pagina Agenti**: limiti del piano in riquadri con percentuale e azzeramento, consumi di oggi e della settimana in riquadri, nuova sezione «Accesso» con «Esci»; nella lista il limite più usato di ogni agente.

**Verificato**: test Rust di `db_browse` e test reale su MariaDB e MongoDB del kit di sviluppo (albero, righe, nome malevolo rifiutato); 23 test dell'interfaccia; anteprima nel browser in tema chiaro e scuro.

## Editor per progetto e niente avvisi del Firewall (14/10/2026)

**Fatto**
- **Ogni finestra dell'editor mostra solo le estensioni del suo progetto** (D-71): quelle dei linguaggi e degli strumenti di altri progetti, le facoltative e quelle aggiunte a mano ad altri progetti risultano «disattivate in quest'area di lavoro». Restano attive le generali, quelle di `extensions.txt` e quelle installate a mano nell'editor; le disattivazioni fatte a mano dall'utente restano. Un linguaggio nuovo o un cambio nel pannello del progetto valgono dal prossimo ricaricamento della finestra.
- **Niente richiesta del Firewall di Windows per Node.js dal terminale dell'editor**: VSCodium toglie `NODE_OPTIONS` ai suoi terminali, quindi `loopback.js` non si caricava; ora il kit la rimette con `terminal.integrated.env.windows`.
- Pagina del progetto: via la sezione «Agenti» e la riga «Agente» nel riquadro del progetto; l'agente si sceglie in testa alla pagina.

**Verificato**: test Rust di `editor_ext`, `editor_window`, `editor_cfg`, `env`; 23 test dell'interfaccia; clippy, ESLint, Prettier puliti. Scrittura dello stato nella IndexedDB provata in Chrome simulando il workbench (prima lettura, scelta dell'utente conservata, estensione riattivata). Da provare nell'app: progetto Python aperto, Rust Analyzer «disattivata (area di lavoro)».

## Impostazioni del progetto nei suoi file (14/10/2026)

**Fatto**
- **Azioni modificabili** dalla pagina del progetto › Avanzate (D-78): nuova azione, modifica, predefinita, togli; con cartella, indirizzo e **variabili d'ambiente**. Salvate in `panel.json` nel progetto.
- **Impostazioni dell'editor del progetto**: formatta al salvataggio, rientro, cartelle nascoste, in `.vscode\settings.json` (le altre impostazioni del file restano).
- **Server MCP del progetto** per Claude Code (`.mcp.json`) e Codex (`.codex\config.toml`).
- **Copie dei database**: Esporta e Ripristina per MariaDB e MongoDB nella pagina del progetto › Risorse; copie in `data\db\backup\<progetto>`. MongoDB porta con sé i Database Tools.

**Verificato**: 211 test Rust (con esportazione, cancellazione e ripristino reali su MariaDB e MongoDB), 23 test dell'interfaccia; prova nell'app: azione con variabile, impostazioni dell'editor e server MCP scritti nei file del progetto.

## Struttura più semplice (14/10/2026)

**Fatto**
- **Sei pagine** (D-76): Progetti, Agenti, Database, Strumenti, Impostazioni, Rimuovi l'unità. La Home e la pagina GitHub non ci sono più: i progetti sono la prima pagina, l'account GitHub è nelle Impostazioni, i repository in «Nuovo progetto».
- **Nuovo progetto** in un posto: vuoto, dai tuoi repository GitHub o da un indirizzo Git.
- **Pagina del progetto a tre schede**: Riepilogo (avvio e output in alto), Risorse (componenti, database, estensioni), Avanzate.
- **Pannello Attività** in basso su ogni pagina: cosa è avviato o in installazione, output ed errori in un solo posto.

**Verificato**: 23 test dell'interfaccia, 204 test Rust; prova nell'app.

## Pagina del progetto (14/10/2026)

**Fatto**
- **Ogni progetto ha la sua pagina** (D-75), dal nome del progetto in Progetti o nella Home: Panoramica, Avvio, Database, Editor, Agenti, Git e GitHub.
- **Database del progetto**: «Crea database e utente» (avvia il server se serve) e «Scrivi in .env» con l'elenco delle righe che cambiano.
- **Stato Git letto dai file**, senza eseguire Git nel progetto; percorsi verso cartelle di rete e collegamenti rifiutati (tre correzioni dalla revisione di sicurezza).

**Verificato**: 204 test Rust, 22 test dell'interfaccia; prova nell'app su un progetto Laravel di prova.

## Database del kit (14/10/2026)

**Fatto**
- **Password per i database** (D-74, dopo la revisione di sicurezza): `root` di MariaDB e `admin` di MongoDB con una password casuale, anche sui dati già creati; ogni progetto con il suo database e il suo utente; scrittura della connessione nel `.env` con anteprima delle righe che cambiano. Nel terminale del kit `mariadb` entra già come root.
- **MariaDB** (compatibile con MySQL), **MongoDB** con mongosh e **SQLite** fra i componenti, nel nuovo gruppo «Database», verificati con SHA-256 o SHA3-256 (D-73).
- **Pagina «Database»**: avvio e arresto dei server, stringa di connessione da copiare, porta, cartella dei dati, log, «Avvia con Atollo». Solo 127.0.0.1; dati in `data\db`. «Avvia» installa il database se manca; chiusura dell'app e «Rimuovi l'unità» li fermano in modo pulito.
- I progetti che usano un database (`.env`, driver npm/pip/Composer, file `.sqlite`) lo segnalano e ricevono le estensioni dell'editor: SQLTools, MongoDB for VS Code, SQLite Viewer; nuovo linguaggio SQL.

**Verificato**: 200 test Rust, 22 test dell'interfaccia; installazione reale dei tre componenti; avvio, query e arresto di MariaDB 12.3.3 e MongoDB 9.0.4 dall'app.

## Branding di app ed editor (14/10/2026)

**Fatto**
- **Logo di Atollo** nella barra del titolo e nella Home dell'app; **accento blu del marchio** (`#3B6CF0`) per pulsanti, fuoco, selezioni e link, temi scuro e chiaro (D-72).
- **Editor con il nome e il logo di Atollo**: titolo «<progetto> - Atollo», logo nella barra del titolo, benvenuto con logo, nome del progetto e sottotitolo nell'editor vuoto.
- **Estensioni mentre lavori** (D-71): vedi sotto.

## Estensioni dell'editor per linguaggio (14/10/2026)

**Fatto**
- **Matrice linguaggio-estensione del kit** (`catalog/extensions.json`, D-71) al posto dell'elenco con `@programma`: 64 estensioni, 26 linguaggi e 3 strumenti (Deno, Bun, CMake). In «Linguaggi e strumenti» la sezione «Estensioni dell'editor» la mostra in sola lettura, con in evidenza le colonne dei linguaggi dei progetti e il segno delle estensioni installate.
- **Estensioni del progetto**: ogni progetto riceve quelle dei suoi linguaggi; dal suo pannello se ne tolgono o se ne aggiungono altre (anche fuori dalla matrice, per `editore.nome`). Salvate in `dev-settings.json`, mai nel progetto.
- Si installano le generali e quelle dei progetti: all'avvio dell'editor, all'apertura di un progetto e quando cambia l'elenco con l'editor acceso.
- **L'editor si aggiorna mentre lavori**: ogni 15 s Atollo rilegge i progetti aperti; un linguaggio nuovo (il primo `.rb`, `.py`…) porta le sue estensioni senza riaprire il progetto, e nell'editor compare l'avviso «Atollo ha installato nuove estensioni» con «Riavvia le estensioni» (D-71).
- Rilevamento dei linguaggi in una tabella unica, con LaTeX, Typst, PowerShell e Shell; un progetto con `.tex` o `.typ` chiede TinyTeX o Typst. Nuova estensione per Astro.

**Verificato**: 194 test Rust, 22 test dell'interfaccia; clippy, ESLint, Prettier e `cargo fmt` puliti. Matrice e pannello del progetto controllati nell'anteprima nel browser. Da provare nell'app: un progetto nuovo con un linguaggio senza estensioni installate le riceve all'apertura nell'editor.

## Pagina GitHub e account GitHub del kit (14/10/2026)

**Fatto**
- **Pagina GitHub** (`github.rs`, `github.tsx`, D-70), porting dell'estensione `github-repos` del Atollo: i repository dell'account e delle sue organizzazioni con ricerca, clone in `projects\` con un clic, «Editor» sui repository già clonati, apertura su github.com, copia dell'indirizzo, rinomina (i remote del clone seguono) ed eliminazione su GitHub con conferma.
- **Un solo account GitHub per tutto il kit**: l'accesso dalla pagina (codice monouso su github.com/login/device) è quello di GitHub CLI in `data\gh`. Git del kit lo usa per github.com, quindi push e pull nell'editor, nelle console e negli agenti usano lo stesso account; «Esci» lo toglie a tutti. Anche «Clona» della pagina Progetti usa l'account per i repository privati.
- Il permesso di eliminare i repository si chiede solo quando serve («Concedi il permesso»).

**Verificato**: 190 test Rust, 22 test dell'interfaccia; `clippy -D warnings`, ESLint, Prettier, `cargo fmt` puliti. Da provare con un account vero: la checklist «GitHub» di TESTING.md (accesso senza console di `gh`, push dall'editor).

## Documentazione allineata al codice (14/10/2026)

**Corretto**
- **Impostazioni di Windows Terminal mai perse** (D-68): un `settings.json` che non si legge (per esempio salvato in UTF-16) veniva sovrascritto; ora si crea solo se manca.
- **Script `.cmd` in cartelle con `%VAR%`** (D-17): la verifica finale passava il percorso a `cmd`, che espandeva le variabili anche fra virgolette (con una cartella `%SystemRoot%` lo script non partiva). Ora lo script parte dalla sua cartella come `.\<nome>`.
- **Verifica finale di Windows Terminal** (D-39): l'unico componente senza controllo; ora legge la versione da `WindowsTerminal.exe` senza aprirlo. Tutti i componenti hanno un controllo (test).
- **CI**: il kit di prova installa anche Windows Terminal ed esegue la prova reale delle sue console.
- Documentazione allineata al codice: README nelle quattro lingue, PLAN, TESTING, CATALOG e 25 decisioni (Windows Terminal di nuovo predefinito, console xterm non più esistenti, file del kit, release, registro, navigazione dell'editor). Decisioni dell'utente: niente «Ferma subito» (D-11) né colori PSReadLine (D-27); limite del watcher riverificato sulla 1.135.06055 (D-33).
- Tolti gli `allow(dead_code)` rimasti dalle fasi precedenti.
- **Installazione delle estensioni fermata con l'editor** (D-36): girava fuori dal job del server e sopravviveva a «Riavvia editor» e all'aggiornamento dell'editor. Trovato dalla prova reale della CI, mai eseguita prima; riprodotto in locale (fallisce con il codice di prima, passa con quello nuovo).
- **CI rossa dal primo build**: lo smoke test leggeva `Atollo.exe env` senza pipe. `Atollo.exe` è un programma a finestra e PowerShell non lo aspettava, quindi «env failed» con `$LASTEXITCODE` vuoto. Ora ogni chiamata passa da una pipe (`| Write-Output`, `| Out-Host`); i README lo spiegano per chi usa la riga di comando da PowerShell. Gli errori di avvio vanno anche su stderr.

**Verificato**: 183 test Rust, 20 test dell'interfaccia; `clippy -D warnings`, ESLint, Prettier puliti; `Atollo.exe check` sul kit di sviluppo (Windows Terminal 1.25.2609.30003).

## Limiti d'uso, progetti e interfaccia (14/10/2026)

**Fatto**
- **Limiti d'uso e consumi degli agenti** (`stats/`, D-61): nella pagina Agenti, per ogni agente installato, i limiti del piano in «% usato» con l'ora in cui si azzerano, i crediti extra di Claude, i token e le sessioni di oggi e degli ultimi 7 giorni con il grafico per giorno. Server di Anthropic, OpenAI e Google con l'accesso del kit, letti al massimo ogni 2 minuti e mai prima di quanto chiede un 429. Pulsante «Aggiorna i consumi».
- **Editor, terminale e agenti solo in un progetto** (D-62): la Home mostra i progetti recenti con Editor, Terminale e agente; la finestra dell'editor rifiuta un'altra cartella, un workspace o la finestra vuota.
- **Impostazioni del kit per progetto** in `data\app\projects\<progetto>\dev-settings.json` (D-63): l'ultimo agente usato (predefinito nella scelta) e l'ultima apertura (ordine dei progetti).
- **Barra del titolo dell'interfaccia** sulla finestra principale, senza la barra di Windows (D-64). **Le finestre dell'editor usano la barra di VSCodium** con i suoi menu (File, Modifica…), il Command Center e i pulsanti di Windows nel suo stile; una barra semplice fa da ripiego se l'editor non risponde (D-69).
- **Loghi ufficiali** di componenti e agenti; **etichette dei linguaggi** con il logo nella riga di ogni progetto (D-65).
- **Solo gli agenti installati** nella pagina Agenti e nella scelta dell'agente.
- **Windows Terminal portabile del kit** (componente `terminal`, D-68), predefinito per Terminale e agenti; la console classica di Windows resta in Impostazioni › Generale › Terminale e fa da ripiego. Ogni scheda riceve l'ambiente del kit dall'app attraverso una named pipe.
- **L'editor chiede se fidarsi** alla prima apertura di ogni progetto; quelli non attendibili si aprono in «Restricted Mode» (D-67, dopo la revisione di sicurezza che segnalava D-66).
- **README in quattro lingue** (`README.md`, `docs/README.en.md`, `.fr.md`, `.de.md`): uso, pagine, cosa resta del Atollo, limiti noti.
- **CI: firma facoltativa dell'exe** con i segreti `SIGN_PFX_BASE64` e `SIGN_PFX_PASSWORD`; senza, l'exe resta non firmato e il passo si salta.

**Corretto durante le prove sull'exe di release**
- Sulle pagine lunghe scorreva anche la finestra intera, barra del titolo compresa: gli elementi solo per lettori di schermo uscivano dal contenitore. Ora scorre solo il contenuto.
- La barra «Applica» di Linguaggi e strumenti lasciava vedere sotto di sé una striscia della lista.

**Verificato**
- 181 test Rust (fra cui periodo locale, date ISO, `Retry-After`, finestre e crediti di Claude, conteggi delle sessioni di Codex senza doppioni, quote di Antigravity, cache con attesa dopo 429, editor che non esce dal progetto, `dev-settings.json`), 20 test dell'interfaccia; `clippy -D warnings`, ESLint, Prettier puliti.

**Da provare a mano** (checklist in [TESTING.md](TESTING.md))
1. Agenti con accesso fatto: i limiti coincidono con `/usage` di Claude Code e `/status` di Codex.
2. Editor su un progetto: File › Apri cartella su un'altra cartella non cambia la finestra.
3. Barra del titolo: trascinamento, doppio clic, riduci, ingrandisci, chiudi con la conferma se qualcosa è aperto; ridimensionamento dai bordi.

## Correzioni e modifiche dopo la fase 6 (14/10/2026)

Tredici commit dopo `7187483`, da `a5aa09a` a `f5eadb4`.

**Fatto**
- **Console classica di Windows** (D-57): la PowerShell del kit e gli agenti girano in `conhost.exe` di System32, avviato direttamente, nel job dell'app e con il solo ambiente del kit; Windows non passa la console al terminale predefinito del PC. Tolti xterm.js, `portable-pty`, `terminal.html`, la capability `terminal` e i suoi tre comandi. D-44, D-45 e D-46 sono superate, D-47 è chiusa.
- **Editor parte dell'app** (D-58): gruppo nuovo `app` nel catalogo, con `version` e `sha256` fissati per il rilascio (1.135.06055). All'avvio un editor di un'altra versione viene installato di nuovo alla versione fissata, dopo aver fermato i processi del vecchio; finché non è aggiornato, quello installato continua a funzionare. In Linguaggi e strumenti compare con gli obbligatori, spuntato e non modificabile.
- **Obbligatori solo PowerShell 7 e GitHub CLI** (D-59, `needs.rs`): Git, Node.js e uv si installano quando servono. Avvio di un agente: l'agente, le sue dipendenze (Claude Code → Git) e i programmi dei server MCP. Clone: Git. Azione di un progetto: il componente del primo programma di ogni comando (`npm` → Node.js, `php` → PHP…). Quello che si installa così resta scelto in `components.txt`.
- **Componenti di sistema con le loro dipendenze obbligatorie** (D-59): GitHub CLI, PowerShell 7 e l'editor, più quello che serve a loro: Git per GitHub CLI, PowerShell 7 per l'editor. Sempre installati, spuntati e non modificabili nella pagina.
- **Scambio di una cartella tenuta aperta da un programma del PC** (D-60): l'aggiornamento dell'editor falliva sempre perché l'indicizzatore di Cloud Code teneva aperta una sua sottocartella. Ora, se nessun programma gira da quella cartella, la vecchia si sposta voce per voce; con un programma in esecuzione resta «in uso».
- **Ambiente ricostruito dopo ogni installazione**: un programma appena installato è visto da editor, console, agenti e progetti senza riavviare l'app (D-49).
- **Catalogo**: `keys` per i componenti fatti di più parti (Java con Maven e Gradle, CMake con Ninja, PHP con Composer, pnpm con yarn, l'editor con `node.exe`); `noArm64` per R; `tools\vcruntime` condiviso da PHP e Apache, tolto con l'ultimo dei due; Composer da `getcomposer.org/versions`; CodeLLDB x64 anche su ARM64, come l'editor; `--only` aggiunge ciò che serve e manca. VSCodium desktop tolto dal catalogo.
- **Rete locale** per tutti i server dei progetti, non solo Node: i comandi trovati dall'app ascoltano su `0.0.0.0` e `Atollo.exe serve` su tutte le interfacce, con un `Host` che sia un indirizzo IP (D-50, D-51). Le porte 7000-7099 dell'editor non si possono scegliere per un progetto.
- **Isolamento di PowerShell**: `PSModulePath` nell'ambiente del kit senza i moduli del PC; Codex con `allow_login_shell = false` (D-34).
- **Impostazioni**: dopo il salvataggio con l'editor aperto, pulsante «Riavvia l'editor».
- **Traduzioni**: le descrizioni dei componenti e gli errori che arrivano all'interfaccia passano dai cataloghi delle quattro lingue.
- **Dipendenze in Linguaggi e strumenti**: ogni componente mostra ciò che richiede in piccoli riquadri («Richiede PHP: si installa insieme.»), e un componente richiesto da uno scelto mostra «per Claude Code».
- **CI**: niente più componente `terminal` nella prova reale; job di release sui tag `app-v*` (tag uguale alla versione di `Cargo.toml`, asset `Atollo-x64.exe` e `Atollo-arm64.exe` con i file `.sha256`); `rust-version` 1.93.

**Verificato**
- 155 test Rust, 19 test dell'interfaccia; lo scambio con una sottocartella aperta senza condivisione di cancellazione è provato da un test.
- Console: avviata così resta nella finestra classica (`ConsoleWindowClass`), nessun `WindowsTerminal.exe`, su Windows 11 (D-57).
- Editor 1.135.06055: lo SHA-256 pubblicato da VSCodium e il digest di GitHub sono uguali (D-58).

**Da provare a mano** (checklist completa in [TESTING.md](TESTING.md))
1. Agenti › Apri su Claude Code: si apre una console classica di Windows e Claude Code riceve i tasti (Invio, frecce, Ctrl+C).
2. Avviare l'app su un kit con l'editor di un'altra versione: all'avvio viene installato di nuovo alla versione fissata; Editor e Terminale si aprono anche se l'aggiornamento non riesce.
3. Su un kit senza Git: all'avvio Git si installa con GitHub CLI; in Linguaggi e strumenti è spuntato e non modificabile («per GitHub CLI»).
4. Su un kit senza Node.js: «Avvia» su un'azione `npm` installa prima Node.js e poi parte.
5. Linguaggi e strumenti: Apache mostra la dipendenza «PHP»; con Claude Code scelto, Git mostra «per Claude Code».

## Fase 6: progetti e rifinitura (14/10/2026)

**Fatto**
- **Progetti** (`projects.rs`, `detect/`, `run.rs`): rilevamento del tipo, dei linguaggi e delle azioni (porting dei rilevatori del Atollo, `panel.json`/`pannello.json`). Creazione, clone da GitHub o da un indirizzo Git, eliminazione delle cartelle vuote. Avvio e arresto delle azioni, ognuna nel suo job, con porte 7100-7999 ricordate o scelte, output in diretta, apertura nel browser e «Servi come sito web» (D-50).
- **Server di file nell'exe**: `Atollo.exe serve <cartella> <porta>`, al posto di `static-server.js` (D-51).
- **Aggiornamento dell'app** (`self_update.rs`) dalle release `app-v*`: SHA-256, scambio dell'exe alla chiusura, `ATOLLO_NO_UPDATE`, versione in `data\app\versions\app.txt` (D-52).
- **Impostazioni**: lingua, rete (proxy, certificati, rete locale), chiave di Gemini, versioni di Python, PHP e Rust, aggiornamenti. Valori controllati; l'ambiente del kit viene ricostruito al salvataggio (D-54).
- **Rimuovi l'unità** e conferma alla chiusura quando sono aperti editor, console, progetti o un'installazione. Nella Home c'è un avviso se è aperta un'altra copia del kit (`sessions.rs`, D-55).
- **Interfaccia in italiano, inglese, francese e tedesco**, con la lingua che cambia senza riavvio. Accessibilità: collegamento «Vai al contenuto», ruoli ARIA, dialogo accessibile, «riduci animazioni», contrasto del rosso nel tema chiaro (D-56).
- **Revisione di sicurezza**: le credenziali del PC in `keyring-before.json` sono cifrate con DPAPI. Le cartelle attendibili di `projects` sono documentate come rischio accettato (D-53).
- **Checklist manuale** in [TESTING.md](TESTING.md).

**Verificato su questo PC**
- `cargo fmt`, `clippy -D warnings` puliti, 133 test Rust. `tsc`, `eslint`, `prettier` puliti, 19 test dell'interfaccia; i test dei cataloghi controllano 194 testi dell'interfaccia e 66 di Rust.
- Exe di release su un kit con tutti i componenti, in una cartella con spazi:
  - progetti riconosciuti («api node» Node.js con express, «sito prova» sito statico);
  - «Avvia» su `npm run start` → porta 7103, stato «in ascolto», risposta del server, output in diretta;
  - «Ferma» → nessun `node.exe` rimasto, porta chiusa;
  - chiusura della finestra con un progetto avviato → dialogo di conferma; «Chiudi tutto» → nessun processo del kit rimasto;
  - Impostazioni › Lingua → English: tutta l'interfaccia cambia subito e `settings.json` viene salvato;
  - Rimuovi l'unità con un progetto avviato → tutto chiuso, poi «Close Atollo» chiude l'app.
- `Atollo.exe serve` reale: 200 sulla pagina, 403 per `.env`, `Host` estraneo e `..`, 301 per le cartelle.

**Corretto durante la fase**
- Nel dialogo di chiusura i processi di un progetto avviato comparivano due volte: l'elenco mostra ora solo i programmi del kit non avviati dall'app.
- `sessions.rs` nascondeva i programmi avviati direttamente dall'app (l'app stessa contava come «padre nel kit»).
- Il segnaposto del file PEM mostrava le barre rovesciate raddoppiate.

**Limiti noti**
- «Rimuovi l'unità» non vede i programmi del PC che tengono aperto solo un file del kit (D-55).
- L'aggiornamento dell'app non è ancora stato provato con una release vera: il repository e il workflow di release arrivano nella fase 7.
- Fase committata in `7187483`, insieme al terminale integrato (PLAN, domanda 13).

**Da provare a mano** (checklist completa in [TESTING.md](TESTING.md))
1. Progetti › Clona `nabre/atollo`: la cartella compare con il nome del repository.
2. Un progetto Vite: «Avvia» usa una porta tra 7100 e 7999, e un secondo avvio riusa la stessa.
3. Impostazioni con un proxy e un file PEM: un'installazione da Linguaggi e strumenti passa dal proxy.
4. Due copie del kit aperte: la Home della seconda mostra l'avviso.
5. Rimuovi l'unità su una chiavetta vera: l'espulsione riesce senza «dispositivo in uso».

## Terminale integrato (14/10/2026)

**Fatto**
- Le console del kit (pulsante Terminale, Progetti › Terminale, Agenti › Apri) sono finestre dell'app con un terminale integrato: xterm.js nella finestra, pseudo console di `portable-pty` (ConPTY su Windows, pty su macOS e Linux) in Rust. Stessi colori di VSCodium, PowerShell del kit, solo l'ambiente del kit, processo nel job dell'app (D-44, D-45).
- Windows Terminal tolto dal catalogo (componente `terminal`, installatore, impostazioni `settings.json`, test).
- Capability `terminal` per le finestre `terminal-*`: tre comandi, ognuno limitato alla sessione della finestra che chiama (D-46).
- `editor_window::quiet_webview` condiviso da finestre dell'editor e console; `jobs::assign_pid`.

**Corretto durante le prove**
- Console bloccata all'avvio: ConPTY aspettava la risposta alla posizione del cursore; ora la dà l'app (D-45).
- Pagina vuota: xterm.js non si carica con `Object.prototype` congelato; la console non usa `freeze.ts` (D-46).
- Ctrl+C non fermava i comandi quando l'app era avviata in un nuovo gruppo di processi (D-45).
- Su Windows senza ConPTY `portable-pty` andava in panic: ora ripiega sulla console classica (D-45).

**Verificato su questo PC**
- `cargo test`: tutti i test, compresi una console reale (output e codice di uscita) e il rifiuto dei comandi delle console da altre finestre.
- Pagina `terminal.html` in anteprima nel browser: xterm.js disegna con il tema chiaro.
- App di debug guidata con il debug remoto di WebView2 (D-48): Terminale apre la finestra con il prompt del kit; `USERPROFILE` = `data\home`, `node` del kit, PowerShell 7.6.7, larghezza della console uguale a quella della finestra; Ctrl+C interrompe `ping -t`; `exit 7` mostra "Processo terminato (codice 7)"; chiudere la finestra termina `pwsh.exe` e `PING.EXE`; Node in modalità raw riceve tasti e frecce; una console ricaricata ritrova il suo output; la finestra console non può chiamare `app_info` né `terminal_open`.

**Problema aperto**
- Claude Code si apre e si disegna, ma non riceve i tasti. Causa probabile e rimedio (ConPTY aggiornato di Microsoft) in D-47; la prova aspetta il permesso dell'utente (PLAN.md §12, domanda 11). Chiuso dopo la fase 6: le console sono ora la console classica di Windows (D-57, sezione in alto).

**Da provare a mano**
1. Home › Terminale: si apre una finestra "projects - Atollo" con PowerShell 7 del kit; `$env:USERPROFILE` punta a `data\home`.
2. Ctrl+C senza selezione interrompe `ping -t 127.0.0.1`; con una selezione la copia; Ctrl+V incolla; clic destro copia o incolla.
3. Agenti › Apri su Claude Code: finestra "Claude Code - Atollo"; all'uscita compare "Processo terminato" e un tasto chiude la finestra (dopo D-47).
4. Chiudere la finestra di una console con un `ping -t` in corso: nessun `pwsh.exe` o `PING.EXE` resta in Gestione attività.
5. Cambiare il tema di Windows con una console aperta: i colori seguono.

## Fase 5: agenti (14/10/2026)

**Fatto**
- Allineamento di Claude Code, Codex e Antigravity in Rust (`agents_sync/`, porting di `sync.js`): istruzioni globali, skill come giunzioni verso `data\agents\skills`, server MCP (Claude, `mcp.json`, Codex `config.toml`, Antigravity), regola del kit, cartelle attendibili, skill di progetto. `Atollo.exe sync [--project <cartella>] [--verbose]`.
- Accesso di Antigravity tenuto nel kit (`keyring.rs`, porting di `agy-keyring.ps1`): `data\kit\bin\agy.cmd` lo porta in Windows (solo sessione) prima di agy e lo riporta nel kit dopo; `Atollo.exe keyring restore|save`.
- Rilocazione (`relocate.rs`, porting di `moved.js`) quando il kit cambia cartella o lettera.
- `agents.rs`: stato (installato, versione, accesso fatto, in esecuzione), lancio in Windows Terminal con allineamento prima e dopo, uscita; sorvegliante che riallinea e salva l'accesso di Antigravity quando un agente termina.
- Interfaccia: pulsanti degli agenti in Home, pagina **Agenti** (stato, Apri, Esci con conferma, cartella condivisa), scelta dell'agente per progetto nella pagina Progetti.

**Verificato su questo PC**
- `cargo fmt`, `clippy -D warnings`: puliti; 105 test Rust (compreso quello con Gestione credenziali reale), 9 test dell'interfaccia.
- Kit con tutti i componenti: skill creata in Codex spostata in `data\agents\skills` e collegata a Claude e Antigravity nello stesso allineamento; istruzione scritta in Claude arrivata ad Antigravity; `codex mcp list` del kit non vede nulla del Codex del PC; `agy --version` passa dal wrapper del kit e non lascia segnali in sospeso.
- Kit rinominato: percorsi di Claude e Codex aggiornati, giunzioni rifatte.

**Corretto durante la fase**
- Con un agy brevissimo l'accesso restava in Windows fino al sorvegliante: il salvataggio a fine esecuzione ora toglie il segnale "in avvio" (D-41).
- Skill nate in un agente arrivavano agli altri solo all'allineamento successivo: ora due passaggi (D-40).

**Limiti noti**
- Statistiche d'uso degli agenti (limiti del piano): non ancora nell'app (facoltative per il prompt).
- "Esci" toglie l'accesso salvato nel kit; le sessioni sul sito del fornitore vanno revocate dalle impostazioni dell'account.

**Da provare a mano**
1. Agenti › Apri su Claude Code: si apre Windows Terminal del kit in `projects`; dopo l'accesso, la pagina mostra "accesso fatto".
2. Creare una skill in Codex (`~/.codex/skills/<nome>/SKILL.md` del kit), chiudere Codex: compare anche in Claude e Antigravity.
3. Nell'editor, terminale "Antigravity": a sessione chiusa, nessuna credenziale `antigravity` nuova in Gestione credenziali.
4. Spostare la cartella del kit e riavviare: Claude ritrova la cronologia dei progetti.

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
