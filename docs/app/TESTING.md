# Prove

Le prove automatiche girano nella CI (`.github/workflows/app.yml`) e in locale, dalla radice del repository:

```
npm run lint
npm test
npm -w ui run build
```

`npm run lint` controlla `cargo fmt`, `cargo clippy -D warnings`, ESLint e Prettier; `npm test` esegue `cargo test` e i test dell'interfaccia (Vitest); `npm -w ui run build` controlla i tipi (`tsc -b`) e compila l'interfaccia.

Coprono, fra l'altro:
- radice del kit, ambiente e PATH;
- catalogo, dipendenze, parti (`keys`), limiti ARM64 e componenti che servono a comandi e agenti (`needs.rs`);
- editor alla versione fissata del rilascio;
- download con checksum e firme;
- editor e server web reale (con il token e senza);
- finestre dell'editor che non possono chiamare i comandi dell'app;
- una console reale in `conhost.exe`, in una cartella con spazi;
- due finestre reali di Windows Terminal del kit (la seconda passa al terminale già aperto) in una cartella con spazio e `;`: l'ambiente del kit arriva e l'app vede la fine (con `ATOLLO_TEST_KIT`);
- allineamento degli agenti e accesso di Antigravity con Gestione credenziali reale;
- rilevamento dei progetti su esempi (Node, Next, Tauri, Electron, PHP, Laravel, Django, FastAPI, Flask, Streamlit, Rust, statico, progetti composti, `panel.json` e `pannello.json`);
- porte ricordate e bloccate, avvio e arresto di un'azione attraverso il suo job;
- server di file: host, file nascosti, `..`, collegamenti;
- scambio dell'exe in aggiornamento;
- DPAPI di `keyring-before.json`;
- cataloghi delle quattro lingue (Rust e interfaccia), completi e con gli stessi segnaposto.

## Prove sull'interfaccia reale

Per le prove senza mani sull'exe di release (fase 6) si usa un kit di prova fuori dal repository, con tutti i componenti, in una cartella con spazi:
- **Avvio fuori dalla sessione dello strumento:** l'app si avvia con WMI (`Win32_Process.Create`). Lo strumento chiude i processi che lancia direttamente, quindi l'app non sopravviverebbe al comando.
- **Clic:** con UI Automation, sui nomi accessibili dei pulsanti. Gli stessi nomi servono anche all'Assistente vocale.
- **Schermate:** con `PrintWindow` sulla finestra «Atollo».
- **Controllo dei processi:** dopo «Ferma», «Chiudi tutto» e «Rimuovi l'unità», nessun processo deve avere l'eseguibile nel kit di prova (`Win32_Process.ExecutablePath`).
- **Arresto delle prove:** i processi di prova si fermano **per PID**, mai per nome. Un `taskkill /IM Atollo.exe` chiude anche le altre copie dell'app aperte sul PC (successo durante la fase 6).

## Checklist manuale

Rispetto alla checklist del prompt:
- la voce sulla rete usa la pagina Impostazioni al posto di `data\proxy.cmd` (D-54);
- la voce sul menu `setup\Windows.cmd` non si applica, perché questo è un prodotto separato, senza gli script del kit (Q1 del piano).

Da eseguire prima di ogni release, con l'exe di release (`npm run build`), su un PC che non ha mai visto il kit, quando la voce lo richiede.

### Installazione e portabilità
- [ ] **Kit nuovo su chiavetta.** Doppio clic, primo avvio: PowerShell 7, GitHub CLI e l'editor si installano da soli; poi installazione dei predefiniti (con Git per Claude Code), editor aperto su un progetto.
- [ ] **Editor alla versione del rilascio.** Con un kit preparato da un'altra versione dell'app (o con un numero diverso in `data\app\versions\vscodium-server.txt`), all'avvio l'editor viene installato di nuovo alla versione fissata, anche se era rimasto aperto, senza «Aggiorna tutto». Se un programma del PC tiene aperta la cartella dell'editor, l'aggiornamento finisce «in uso», ma Editor, Terminale e gli agenti si aprono lo stesso con l'editor già presente. In Linguaggi e strumenti l'editor è tra gli obbligatori, spuntato e non modificabile.
- [ ] **Nessuna traccia fuori dal kit.** Dopo una sessione completa (editor, terminale, un agente, un progetto avviato) non c'è nessun file nuovo in `%APPDATA%`, `%LOCALAPPDATA%`, `%TEMP%` e nel registro. Confronto prima/dopo con Process Monitor o con un'istantanea delle cartelle. Eccezione documentata: le credenziali di Antigravity, che restano in Windows solo per la durata di `agy` e con persistenza di sessione (D-41).
- [ ] **Kit spostato.** Su un'altra lettera di unità e in una cartella con spazi tutto funziona, lo stato dell'editor resta e Claude ritrova la cronologia.
- [ ] **PC senza WebView2** (Windows 10 LTSC o macchina virtuale). Il messaggio è chiaro e l'app funziona con il runtime fisso nel kit.
- [ ] **Utente standard** senza diritti di amministratore: nessuna richiesta UAC in nessun passo.
- [ ] **Proxy autenticato e ispezione SSL.** Impostazioni › Rete con proxy e file PEM: installazione, estensioni dell'editor e agenti funzionano.
- [ ] **Senza rete.** L'app e l'editor partono, gli errori di download si leggono, il controllo degli aggiornamenti fallisce senza bloccare nulla.
- [ ] **Chiusura forzata dal Task Manager.** Non resta nessun `node.exe`, server dell'editor, console del kit (`WindowsTerminal.exe`, `conhost.exe`, `pwsh.exe`) o server di progetto.
- [ ] **ARM64.** L'exe è nativo, l'editor gira in emulazione e l'avviso si vede.

### Progetti
- [ ] **Nuovo progetto** «prova 1»: la cartella compare, ha il pulsante «Elimina la cartella vuota» e l'eliminazione funziona.
- [ ] **Clona** `nabre/atollo` (o un repository privato con `gh auth login` fatto nel terminale del kit): la cartella ha il nome giusto. Su un kit senza Git, Git si installa prima del clone. Con un indirizzo non valido (`-x/y`, `ssh://…`) l'errore si legge.
- [ ] **Progetto Vite.** «Avvia», poi la porta 7100-7999 compare nell'output. Lo stato passa a «in ascolto» e il pulsante con l'indirizzo apre il browser del PC. Un secondo avvio riusa la stessa porta. Su un kit senza Node.js, «Avvia» installa prima Node.js (avanzamento in Home e nella barra di stato) e poi parte; Node.js risulta scelto in Linguaggi e strumenti.
- [ ] **Rete locale.** Con «Permetti ai server dei progetti di farsi raggiungere dagli altri PC della rete», un altro PC apre il progetto avviato con l'indirizzo IP di questo PC; senza, no.
- [ ] **Porta occupata.** Scelta a mano una porta già usata da un altro programma, l'avvio dà un errore chiaro e la porta non cambia da sola.
- [ ] **Sito statico.** Una cartella con solo `index.html`: il server di file del kit lo serve. Un file `.env` non è raggiungibile (403).
- [ ] **«Servi come sito web».** Con la cartella `public` e PHP: `panel.json` contiene l'azione `site` e le altre chiavi del file restano.
- [ ] **«Ferma»** su `npm run dev`: in Gestione attività non resta nessun `node.exe`/`esbuild.exe` del progetto.
- [ ] **Electron o Tauri.** L'app apre la sua finestra e l'output arriva nella pagina.

### Agenti
- [ ] **Claude Code, Codex e Antigravity.** Accesso, avvio dall'app e dall'editor, allineamento di istruzioni e skill fra i tre.
- [ ] **Console di un agente.** Agenti › Apri su Claude Code: si apre Windows Terminal del kit (non quello del PC); con «Console classica di Windows» nelle Impostazioni, la console classica (non Windows Terminal, anche se è il terminale predefinito del PC). Claude Code riceve Invio, frecce e Ctrl+C. Chiudendo la console non resta nessun `claude.exe`.
- [ ] **Windows Terminal del kit.** Con Windows Terminal già aperto da un'altra console del kit, Home › Terminale apre una nuova finestra con il profilo «Atollo» e `$env:CP_ROOT` del kit; «Chiudi tutto» la chiude. Senza il componente `terminal`, o con «Console classica di Windows», si apre la console classica.
- [ ] **Git con GitHub CLI.** Su un kit senza Git, all'avvio Git si installa con i componenti di sistema; in «Linguaggi e strumenti» è spuntato, non modificabile, con l'etichetta «per GitHub CLI».
- [ ] **Editor aggiornato con una cartella aperta.** Con Esplora file aperto su `data\apps\win32-x64\vscodium-server\extensions`, l'editor di un'altra versione viene comunque sostituito all'avvio (D-60).

- [ ] **Limiti d'uso.** Con l'accesso fatto in Claude Code e Codex, Agenti mostra gli stessi valori di `/usage` e `/status`, con l'ora in cui si azzerano. Senza accesso compare «accesso non fatto». Senza rete l'errore si legge e la pagina resta usabile. «Aggiorna i consumi» premuto più volte di fila non fa più di una richiesta ogni 20 s.
- [ ] **Solo nel progetto.** In Home e Progetti, Editor, Terminale e agente si aprono nella cartella del progetto; non c'è nessun pulsante per `projects\`. Nell'editor, File › Apri cartella su un'altra cartella, «Nuova finestra» e «Chiudi cartella» non escono dal progetto.
- [ ] **Fiducia nell'editor.** Alla prima apertura di un progetto l'editor chiede se fidarsi. Con «No» compare la barra «Restricted Mode» e le attività non partono; con «Sì» non chiede più alla riapertura, anche dopo aver riavviato l'app (D-67).
- [ ] **dev-settings.json.** Dopo aver aperto Codex in un progetto, `data\app\projects\<progetto>\dev-settings.json` ha `"agent": "codex"` e la scelta dell'agente propone Codex. Eliminando dall'app un progetto vuoto la sua cartella in `data\app\projects\` sparisce.

### Linguaggi e strumenti
- [ ] **Loghi.** Ogni componente ha il suo logo, leggibile nel tema chiaro e in quello scuro; i progetti mostrano le etichette dei linguaggi.
- [ ] **Dipendenze.** Apache mostra la dipendenza «PHP»; con Claude Code scelto, Git mostra «per Claude Code». Togliendo PHP si toglie anche Apache.
- [ ] **PATH dopo un'installazione.** Dopo l'installazione di un linguaggio da questa pagina, Home › Terminale apre Windows Terminal del kit con PowerShell 7 del kit e il programma nuovo è nel PATH, senza riavviare l'app.

### Impostazioni
- [ ] **Lingua.** Passando a English, Français e Deutsch, tutta l'interfaccia cambia subito. Con «Lingua di Windows» vale quella del sistema.
- [ ] **Chiave di Gemini.** Salvata, il campo mostra «salvata». Non compare mai in chiaro (né nell'interfaccia, né in `data\app\logs`). «Togli la chiave» la rimuove.
- [ ] **Valore non valido** (proxy `proxy:8080`, Python `3.x`): l'errore si legge e il file non cambia.
- [ ] **Editor in esecuzione.** Dopo il salvataggio l'app chiede di riavviare l'editor, e dopo il riavvio i terminali dell'editor vedono le variabili nuove.
- [ ] **Aggiornamento.** Con una release `app-v*` più recente pubblicata, «Cerca aggiornamenti» scarica l'exe. Alla chiusura `Atollo.exe` è quello nuovo e `Atollo.old.exe` sparisce al riavvio. Con `ATOLLO_NO_UPDATE=1` all'avvio non parte nessun controllo.

### Chiusura, altre copie, rimozione dell'unità
- [ ] **Chiusura con qualcosa aperto.** Con editor, console o un progetto avviato, chiudere la finestra chiede conferma ed elenca cosa si chiude. «Annulla» (anche con Esc) lascia tutto aperto.
- [ ] **Due copie del kit aperte.** La Home della seconda mostra l'avviso con la cartella dell'altra.
- [ ] **Rimuovi l'unità.** Tutto si chiude e l'accesso di Antigravity torna nel kit. Dopo «Chiudi Atollo» la chiavetta si espelle senza «dispositivo in uso».

### Barra del titolo
- [ ] **Finestra principale senza cornice di Windows.** Trascinamento dalla barra, doppio clic per ingrandire, Riduci, Ingrandisci/Ripristina, Chiudi (con la conferma se qualcosa è aperto); ridimensionamento dai bordi; Win+frecce funzionano. I pulsanti si raggiungono con Tab e hanno un nome per l'Assistente vocale.

### Accessibilità
- [ ] **Solo tastiera.** Tutte le pagine si usano: Tab mostra sempre il contorno di focus, «Vai al contenuto» è il primo elemento, il dialogo di chiusura riceve il focus.
- [ ] **Assistente vocale.** Le pagine hanno titoli e le icone hanno un nome. Errori e stati sono annunciati (`role="alert"` e `role="status"`).
- [ ] **Tema e animazioni.** Il tema chiaro e quello scuro seguono Windows. Con «Effetti di animazione» spenti in Windows, nessuna icona gira.
