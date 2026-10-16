# Prove

Le prove automatiche girano nella CI (`.github/workflows/app.yml`) e in locale, dalla radice del repository:

```
npm run lint
npm test
npm -w ui run build
```

`npm run lint` controlla `cargo fmt`, `cargo clippy -D warnings`, ESLint e Prettier (anche sui test, con i tipi di `test\ui\tsconfig.json`); `npm test` esegue `cargo test` e i test dell'interfaccia (Vitest); `npm -w ui run build` controlla i tipi (`tsc -b`) e compila l'interfaccia.

Tutte le prove stanno in `test\`, con la struttura dei sorgenti:
- `test\rust\<percorso>.rs`: le prove di `tauri\src\<percorso>.rs`. Restano un modulo figlio di quel file, che le dichiara in fondo con `#[cfg(test)] #[path = "…/test/rust/<percorso>.rs"] mod tests;`: vedono le sue funzioni private (`use super::*`), come prima. Una prova nuova va nel file di `test\rust` del modulo; un modulo che non ne ha ancora aggiunge le tre righe.
- `test\ui\<percorso>.test.ts(x)`: le prove di `ui\src\<percorso>`, che importano da `../../ui/src/…`; `test\ui\setup.ts` prepara jsdom (Vitest, `ui\vite.config.ts`).

Le prove dal vivo sulle fonti delle versioni (rete, ignorate di solito): `cargo test live_lines -- --ignored --nocapture` (con `GITHUB_TOKEN`), vedi CATALOG.md §2.

Coprono, fra l'altro:
- radice del kit, ambiente e PATH;
- file del kit (`fsx.rs`): BOM, byte non UTF-8, file mancante, vuoto o danneggiato, scrittura atomica senza file temporanei rimasti;
- errore dei comandi (`commands\error.rs`): Tauri manda per `CmdError` lo stesso valore di una stringa, `or_fail` manda il testo breve e `or_fail_chain` tutta la catena;
- catalogo, dipendenze, parti (`keys`), limiti ARM64 e componenti che servono a comandi e agenti (`setup\needs.rs`);
- righe di versione lette dalle fonti di ogni linguaggio (Go, Java, .NET, Zig: prove in `test\rust\install\lang\`);
- editor alla versione fissata del rilascio;
- download con checksum e firme;
- editor e server web reale (con il token e senza);
- finestre dell'editor che non possono chiamare i comandi dell'app;
- una console reale in `conhost.exe`, in una cartella con spazi;
- allineamento degli agenti e accesso di Antigravity con Gestione credenziali reale;
- rilevamento dei progetti su esempi (Node, Next, Tauri, Electron, PHP, Laravel, Django, FastAPI, Flask, Streamlit, Rust, statico, progetti composti, `panel.json` e `pannello.json`);
- porte ricordate e bloccate, avvio e arresto di un'azione attraverso il suo job;
- server di file: host, file nascosti, `..`, collegamenti;
- scambio dell'exe in aggiornamento;
- DPAPI di `keyring-before.json`;
- cataloghi delle quattro lingue (Rust e interfaccia), completi e con gli stessi segnaposto;
- `useLoad` (`test\ui\hooks.test.ts`): prima lettura, le tre politiche d'errore, nuova lettura quando cambiano `load` o `key` ma non la politica, `enabled`, lettura periodica fermata allo smontaggio, `setData`;
- controlli dell'interfaccia su `vscode-elements` (`test\ui\workbench\controls.test.tsx`, `elements.test.tsx`): eventi che scattano davvero, valori controllati, Invio nei moduli, nomi accessibili, codicon; ogni variabile `--vscode-*` letta dai componenti definita nei due temi (`test\ui\theme.test.ts`).

## Prove sull'interfaccia reale

Per le prove senza mani sull'exe di release (fase 6) si usa un kit di prova fuori dal repository, con tutti i componenti, in una cartella con spazi:
- **Avvio fuori dalla sessione dello strumento:** l'app si avvia con WMI (`Win32_Process.Create`). Lo strumento chiude i processi che lancia direttamente, quindi l'app non sopravviverebbe al comando.
- **Clic:** con UI Automation, sui nomi accessibili dei pulsanti. Gli stessi nomi servono anche all'Assistente vocale.
- **Schermate:** con `PrintWindow` sulla finestra «Dev-Kit».
- **Controllo dei processi:** dopo «Ferma», «Chiudi tutto» e «Rimuovi l'unità», nessun processo deve avere l'eseguibile nel kit di prova (`Win32_Process.ExecutablePath`).
- **Arresto delle prove:** i processi di prova si fermano **per PID**, mai per nome. Un `taskkill /IM Dev-Kit.exe` chiude anche le altre copie dell'app aperte sul PC (successo durante la fase 6).

## Checklist manuale

Rispetto alla checklist del prompt:
- la voce sulla rete usa la pagina Impostazioni al posto di `data\proxy.cmd` (D-54);
- la voce sul menu `setup\Windows.cmd` non si applica, perché questo è un prodotto separato, senza gli script del kit (Q1 del piano).

Da eseguire prima di ogni release, con l'exe di release (`npm run build`), su un PC che non ha mai visto il kit, quando la voce lo richiede.

### Installazione e portabilità
- [ ] **Kit nuovo su chiavetta.** Doppio clic, primo avvio: PowerShell 7, GitHub CLI (con Git) e l'editor si installano da soli; poi installazione dei predefiniti, editor aperto su un progetto.
- [ ] **Editor alla versione del rilascio.** Con un kit preparato da un'altra versione dell'app (o con un numero diverso in `data\app\versions\vscodium-server.txt`), all'avvio l'editor viene installato di nuovo alla versione fissata, anche se era rimasto aperto, senza «Aggiorna tutto». Se un programma del PC tiene aperta la cartella dell'editor, l'aggiornamento finisce «in uso», ma Editor, Terminale e gli agenti si aprono lo stesso con l'editor già presente. In Linguaggi e strumenti l'editor è tra gli obbligatori, spuntato e non modificabile.
- [ ] **Nessuna traccia fuori dal kit.** Dopo una sessione completa (editor, terminale, un agente, un progetto avviato) non c'è nessun file nuovo in `%APPDATA%`, `%LOCALAPPDATA%`, `%TEMP%` e nel registro. Confronto prima/dopo con Process Monitor o con un'istantanea delle cartelle. Eccezioni documentate: le credenziali di Antigravity, che restano in Windows solo per la durata di `agy` e con persistenza di sessione (D-41); i file di passaggio in `%TEMP%\Dev-Kit\<id>\`, che a fine sessione sono spariti (D-90).
- [ ] **Kit spostato.** Su un'altra lettera di unità e in una cartella con spazi tutto funziona, lo stato dell'editor resta e Claude ritrova la cronologia.
- [ ] **PC senza WebView2** (Windows 10 LTSC o macchina virtuale). Il messaggio è chiaro e l'app funziona con il runtime fisso nel kit.
- [ ] **Utente standard** senza diritti di amministratore: nessuna richiesta UAC in nessun passo.
- [ ] **Proxy autenticato e ispezione SSL.** Impostazioni › Rete con proxy e file PEM: installazione, estensioni dell'editor e agenti funzionano.
- [ ] **Senza rete.** L'app e l'editor partono, gli errori di download si leggono, il controllo degli aggiornamenti fallisce senza bloccare nulla.
- [ ] **Chiusura forzata dal Task Manager.** Non resta nessun `node.exe`, server dell'editor, console del kit (`conhost.exe`, `pwsh.exe`) o server di progetto.
- [ ] **ARM64.** L'exe è nativo, l'editor gira in emulazione e l'avviso si vede.

### Progetti
- [ ] **Nuovo progetto** «prova 1»: la cartella compare, ha il pulsante «Elimina la cartella vuota» e l'eliminazione funziona.
- [ ] **Clona** `nabre/dev-kit` (o un repository privato con `gh auth login` fatto nel terminale del kit): la cartella ha il nome giusto. Su un kit senza Git, Git si installa prima del clone. Con un indirizzo non valido (`-x/y`, `ssh://…`) l'errore si legge.
- [ ] **Progetto Vite.** «Avvia», poi la porta 7100-7999 compare nell'output. Lo stato passa a «in ascolto» e il pulsante con l'indirizzo apre il browser del PC. Un secondo avvio riusa la stessa porta. Su un kit senza Node.js, «Avvia» installa prima Node.js (avanzamento in Home e nella barra di stato) e poi parte; Node.js risulta scelto in Linguaggi e strumenti.
- [ ] **Rete locale.** Con «Permetti ai server dei progetti di farsi raggiungere dagli altri PC della rete», un altro PC apre il progetto avviato con l'indirizzo IP di questo PC; senza, no.
- [ ] **Porta occupata.** Scelta a mano una porta già usata da un altro programma, l'avvio dà un errore chiaro e la porta non cambia da sola.
- [ ] **Sito statico.** Una cartella con solo `index.html`: il server di file del kit lo serve. Un file `.env` non è raggiungibile (403).
- [ ] **«Servi come sito web».** Con la cartella `public` e PHP: `panel.json` contiene l'azione `site` e le altre chiavi del file restano.
- [ ] **«Ferma»** su `npm run dev`: in Gestione attività non resta nessun `node.exe`/`esbuild.exe` del progetto.
- [ ] **Electron o Tauri.** L'app apre la sua finestra e l'output arriva nella pagina.

### Dati per progetto e file di passaggio (D-89, D-90)
- [ ] **Migrazione da un kit della versione 1.** Con la copia di un kit reale (dati in `data\app\projects`, `ports.json`, `secrets.json`, `data\db\backup`): al primo avvio compaiono `data\projects\<progetto>\`, `data\app\layout.txt` vale `2`, `data\app\pre-v2\` ha le due copie. Progetti, porte (anche quelle bloccate), password e copie dei database intatti; `.env` dei progetti ancora validi; un secondo avvio non cambia nulla.
- [ ] **Chiavetta exFAT.** Installazione di Node, Python e Java: l'avanzamento non dà errori di spazio o di rename fra dischi; «Temporanei dei programmi: Automatico» usa il sistema; dopo la chiusura dell'app non resta nessun file in `data\tmp` e `%TEMP%\Dev-Kit\<id>\system` non esiste più.
- [ ] **Chiavetta FAT32.** Un componente con un file oltre 4 GiB è rifiutato con un errore che dice quale file, la versione già installata resta intatta e non resta `.devkit-new`.
- [ ] **Rust** (`695c177`).
  - Kit NTFS senza Rust: installazione, `cargo --version` da `rust\cargo\bin`.
  - Kit NTFS con Rust: aggiornamento sul posto («rustup update» nel registro), uno strumento di `cargo install` resta.
  - Kit FAT32/exFAT senza Rust: nessun errore sui collegamenti fisici, `rustup-init` nel temp e copia sulla chiavetta; `cargo`, `rustc` e `rustup show` funzionano dalle copie dei proxy.
  - Kit FAT32/exFAT con Rust: aggiornamento («rustup update (copy in the temp folder)»), strumenti di `cargo install` presenti, nessun `rust.devkit-new` rimasto.
  - `TEMP` su un disco FAT/exFAT: errore chiaro sul temp senza collegamenti fisici, kit intatto.
- [ ] **Eliminazione di un progetto con MariaDB e MongoDB.** Server fermo: la conferma elenca cartella, motori e copie; dopo «Elimina» i server si avviano, database e utente sono tolti (`SHOW DATABASES`, `SHOW GRANTS`) e `data\projects\<progetto>\` non esiste. Con un server che non parte non si cancella nulla; «Elimina senza i database» cancella le cartelle e il log dice quali database restano.
- [ ] **Ripristino di una copia fatta altrove.** Una copia `.sql` ottenuta con `mysqldump --databases altro` è rifiutata dal server (accesso negato) e il database `altro` resta intatto; una copia del database del progetto si ripristina.
- [ ] **Nomi simili.** Progetto `my-app` (database `my_app`) e progetto `my2app`: l'utente `my_app` non vede `my2app` (`SHOW GRANTS`). Un progetto chiamato `mysql` o `admin` non può cancellare né cambiare i database dei server.
- [ ] **Progetto rinominato in Esplora file.** Compare in «Dati senza progetto» nella pagina Progetti; «Collega a…» lo riporta con password e copie, e la `.env` continua a funzionare. Un orfano con database si elimina con la stessa conferma.
- [ ] **C: quasi pieno.** Le installazioni ripiegano su `data\tmp` e il log ha l'avviso.
- [ ] **Alias dei siti.** Un sito con `/media` → `storage/uploads` e `/shared` → `@common/assets`, con Apache, con Nginx e in un pool: i file si aprono nel browser.
- [ ] **Alias e giunzione.** `storage` sostituito da una giunzione verso `C:\Users`: l'alias non è servito e la pagina del sito lo dice.
- [ ] **Impostazioni › Temporanei dei programmi.** La pagina mostra file system e tipo di disco del kit; Automatico, Nel kit e Nel sistema cambiano `$env:TEMP` in una console nuova del kit (`data\tmp\system` o `%TEMP%\Dev-Kit\<id>\system`).
- [ ] **File scaricati a mano** (D-82). Un file messo a mano nel riquadro finisce in `%TEMP%\Dev-Kit\<id>\downloads`; uno lasciato in `data\downloads` si usa lo stesso e si sposta.

### GitHub
- [ ] **Accesso.** GitHub › «Accedi a GitHub»: si apre github.com/login/device nel browser del PC e la pagina mostra il codice. Dopo l'autorizzazione compare «Accesso fatto come @…» senza ricaricare, `data\gh\hosts.yml` contiene l'account e nella Gestione credenziali di Windows non c'è nessuna voce `gh:github.com`. «Annulla» durante l'attesa non lascia nessun `gh.exe`.
- [ ] **Un solo accesso.** Dopo l'accesso, `git config --global --get-all credential.https://github.com.helper` in una console del kit dà una riga vuota e `!gh auth git-credential`. Push di un repository privato dall'editor (Controllo del codice sorgente) e da una console senza nessuna finestra di Git Credential Manager. Claude Code usa `gh` con lo stesso account. Dopo «Esci» il helper sparisce. Dopo aver spostato il kit su un'altra lettera di unità, il push funziona ancora.
- [ ] **Repository.** Elenco con i repository propri e delle organizzazioni (gruppi separati), ricerca, «Clona» su un repository privato: la cartella è in `projects\` e `.git\config` non contiene il token. Con il nome già preso la cartella diventa `<proprietario>-<nome>`. Sul repository clonato c'è «Editor».
- [ ] **Rinomina ed elimina** su un repository di prova: dopo «Rinomina» il remote del clone punta al nuovo nome. «Elimina da GitHub» chiede conferma; la prima volta offre «Concedi il permesso», poi con un nuovo accesso l'eliminazione riesce e il clone resta in `projects\`.

### Agenti
- [ ] **Claude Code, Codex e Antigravity.** Accesso, avvio dall'app e dall'editor, allineamento di istruzioni e skill fra i tre.
- [ ] **Console di un agente.** Home o Progetti › Agente con Claude Code: si apre la console classica di Windows, anche se il terminale predefinito del PC è Windows Terminal o un altro. Claude Code riceve Invio, frecce e Ctrl+C. Chiudendo la console non resta nessun `claude.exe`.
- [ ] **Git con GitHub CLI.** Su un kit senza Git, all'avvio Git si installa con i componenti di sistema; in «Linguaggi e strumenti» è spuntato, non modificabile, con l'etichetta «per GitHub CLI».
- [ ] **Editor aggiornato con una cartella aperta.** Con Esplora file aperto su `data\apps\win32-x64\vscodium-server\extensions`, l'editor di un'altra versione viene comunque sostituito all'avvio (D-60).

- [ ] **Limiti d'uso.** Con l'accesso fatto in Claude Code e Codex, Agenti mostra gli stessi valori di `/usage` e `/status`, con l'ora in cui si azzerano. Senza accesso compare «accesso non fatto». Senza rete l'errore si legge e la pagina resta usabile. «Aggiorna i consumi» premuto più volte di fila non fa più di una richiesta ogni 20 s.
- [ ] **Solo nel progetto.** In Home e Progetti, Editor, Terminale e agente si aprono nella cartella del progetto; non c'è nessun pulsante per `projects\`. Nell'editor, File › Apri cartella su un'altra cartella, «Nuova finestra» e «Chiudi cartella» non escono dal progetto.
- [ ] **Fiducia nell'editor.** Alla prima apertura di un progetto l'editor chiede se fidarsi. Con «No» compare la barra «Restricted Mode» e le attività non partono; con «Sì» non chiede più alla riapertura, anche dopo aver riavviato l'app (D-67).
- [ ] **dev-settings.json.** Dopo aver aperto Codex in un progetto, `data\projects\<progetto>\dev-settings.json` ha `"agent": "codex"` e la scelta dell'agente propone Codex. Eliminando dall'app un progetto vuoto la sua cartella in `data\projects\` sparisce (D-89).

### Linguaggi e strumenti
- [ ] **Loghi.** Ogni componente ha il suo logo, leggibile nel tema chiaro e in quello scuro; i progetti mostrano le etichette dei linguaggi.
- [ ] **Dipendenze.** Apache mostra la dipendenza «PHP»; con Claude Code scelto, Git mostra «per Claude Code». Togliendo PHP si toglie anche Apache.
- [ ] **PATH dopo un'installazione.** Dopo l'installazione di un linguaggio da questa pagina, Home › Terminale apre la console del kit con PowerShell 7 del kit e il programma nuovo è nel PATH, senza riavviare l'app.

### Impostazioni
- [ ] **Lingua.** Passando a English, Français e Deutsch, tutta l'interfaccia cambia subito. Con «Lingua di Windows» vale quella del sistema.
- [ ] **Chiave di Gemini.** Salvata, il campo mostra «salvata». Non compare mai in chiaro (né nell'interfaccia, né in `data\app\logs`). «Togli la chiave» la rimuove.
- [ ] **Valore non valido** (proxy `proxy:8080`, Python `3.x`): l'errore si legge e il file non cambia.
- [ ] **Editor in esecuzione.** Dopo il salvataggio l'app chiede di riavviare l'editor, e dopo il riavvio i terminali dell'editor vedono le variabili nuove.
- [ ] **Aggiornamento.** Con una release `app-v*` più recente pubblicata, «Cerca aggiornamenti» scarica l'exe. Alla chiusura `Dev-Kit.exe` è quello nuovo e `Dev-Kit.old.exe` sparisce al riavvio. Con `DEVKIT_NO_UPDATE=1` all'avvio non parte nessun controllo.

### Chiusura, altre copie, rimozione dell'unità
- [ ] **Chiusura con qualcosa aperto.** Con editor, console o un progetto avviato, chiudere la finestra chiede conferma ed elenca cosa si chiude. «Annulla» (anche con Esc) lascia tutto aperto.
- [ ] **Due copie del kit aperte.** La Home della seconda mostra l'avviso con la cartella dell'altra.
- [ ] **Rimuovi l'unità.** Tutto si chiude e l'accesso di Antigravity torna nel kit. Dopo «Chiudi Dev-Kit» la chiavetta si espelle senza «dispositivo in uso».

### Barra del titolo
- [ ] **Finestra principale senza cornice di Windows.** Trascinamento dalla barra, doppio clic per ingrandire, Riduci, Ingrandisci/Ripristina, Chiudi (con la conferma se qualcosa è aperto); ridimensionamento dai bordi; Win+frecce funzionano. I pulsanti si raggiungono con Tab e hanno un nome per l'Assistente vocale.
- [ ] **Database** (D-73). Pagina Database: «Avvia» su MariaDB non installato lo installa e lo avvia; `mariadb -uroot -h127.0.0.1` e `mongosh` rispondono; porta occupata: messaggio e cambio porta; «Avvia con Dev-Kit» lo fa partire al riavvio dell'app; chiusura dell'app: la conferma conta i database e i log finiscono con «Shutdown complete» / «Now exiting». Un progetto Laravel con `DB_CONNECTION=mysql` riceve SQLTools.
- [ ] **Contenuto dei database** (D-79). Pagina Database con MariaDB avviato: «Contenuto» mostra il database di un progetto e le sue tabelle; una tabella con più di 100 righe va avanti e indietro; colonne BLOB come «n byte binari». MongoDB: collezioni e documenti con `ObjectId(…)`. Server fermo: «Avvia il server per vederne il contenuto.». SQLite non è nella lista.
- [ ] **Password dei database** (D-74). Pagina Database: password di root/admin nascosta, «Mostra» e «Copia». `mariadb --no-defaults -uroot -h127.0.0.1` rifiutato; `mariadb` nel terminale del kit entra. Pagina del progetto › Database: «Crea» e poi «Scrivi in .env» mostra prima le righe che cambiano.
- [ ] **Impostazioni del progetto** (D-78). Avanzate: «Nuova azione» con una variabile `NOME=valore` compare in tabella e in `panel.json`; «Predefinita» la mette sul pulsante Avvia; «Togli» la toglie. Editor: formatta al salvataggio e rientro scritti in `.vscode\settings.json`, aperti nell'editor valgono. MCP: un server aggiunto compare in `.mcp.json` e in `.codex\config.toml`; `codex mcp list` nel progetto lo elenca. Risorse › database: «Esporta» crea la copia, «Ripristina…» chiede conferma e riporta i dati.
- [ ] **Struttura a sei pagine** (D-76). Progetti è la prima pagina; «Nuovo progetto» con le tre strade (il clone dai repository chiede l'accesso a GitHub nelle Impostazioni); pagina del progetto con Riepilogo, Risorse, Avanzate; pannello Attività: avviare un progetto, aprire il pannello, vedere l'output, Apri nel browser, Ferma; installazione in corso nel pannello; database con errore (porta occupata) nel pannello.
- [ ] **Pagina del progetto** (D-75). Dal nome in Progetti e nella Home; le sei schede si scelgono anche con le frecce; Avvio avvia e ferma come prima; Database crea il database e scrive il `.env` dopo l'anteprima; Git mostra ramo e remoto di un progetto clonato da GitHub.
- [ ] **Branding** (D-72). App: logo nella barra del titolo e nella Home, accento verde NEXKORA in tema scuro e chiaro, testo dei pulsanti leggibile. Editor: titolo «<progetto> - Dev-Kit», logo nella barra, benvenuto con logo e progetto nell'editor vuoto, anche in tema chiaro.
- [ ] **Estensioni mentre si lavora** (D-71). Editor aperto su un progetto senza Ruby: si crea `app.rb`; entro circa 20 s (più il download) compare l'avviso con Ruby LSP e Ruby Debugger; «Riavvia le estensioni» ricarica l'editor sul progetto, «Più tardi» chiude l'avviso.
- [ ] **Finestra dell'editor con la barra di VSCodium** (D-69). Menu File…Help nella barra; trascinamento dalle zone vuote, doppio clic per ingrandire, Aero Snap; Riduci, Ingrandisci/Ripristina (l'icona cambia anche con Win+frecce), Chiudi; ridimensionamento dai bordi; tema chiaro e scuro. Con il server dell'editor fermo compare la barra di ripiego, che sposta e chiude la finestra.

### Accessibilità
- [ ] **Solo tastiera.** Tutte le pagine si usano: Tab mostra sempre il contorno di focus, «Vai al contenuto» è il primo elemento, il dialogo di chiusura riceve il focus.
- [ ] **Assistente vocale.** Le pagine hanno titoli e le icone hanno un nome. Errori e stati sono annunciati (`role="alert"` e `role="status"`).
- [ ] **Stesso aspetto dell'editor** (D-88). App ed editor affiancati, in tema scuro e in chiaro: stessi colori di sfondo, testo, focus, pulsanti, campi e menu a tendina; select, campi e caselle si usano con mouse e tastiera; le icone codicon si vedono (nessun quadratino).
- [ ] **Tema e animazioni.** Il tema chiaro e quello scuro seguono Windows. Con «Effetti di animazione» spenti in Windows, nessuna icona gira.
