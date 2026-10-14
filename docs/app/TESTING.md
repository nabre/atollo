# Prove

Le prove automatiche girano nella CI (`.github/workflows/app.yml`) e in locale:

```
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cd ui && npx tsc -b && npx eslint src && npx prettier --check . && npx vitest run
```

Coprono, fra l'altro:
- radice del kit, ambiente e PATH;
- catalogo e dipendenze;
- download con checksum e firme;
- editor e server web reale (con il token e senza);
- finestre dell'editor che non possono chiamare i comandi dell'app;
- allineamento degli agenti e accesso di Antigravity con il Gestore credenziali reale;
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
- [ ] **Kit nuovo su chiavetta.** Doppio clic, primo avvio, installazione dei predefiniti, editor aperto su un progetto.
- [ ] **Nessuna traccia fuori dal kit.** Dopo una sessione completa (editor, terminale, un agente, un progetto avviato) non c'è nessun file nuovo in `%APPDATA%`, `%LOCALAPPDATA%`, `%TEMP%` e nel registro. Confronto prima/dopo con Process Monitor o con un'istantanea delle cartelle. Eccezione documentata: le credenziali di Antigravity, che restano in Windows solo per la durata di `agy` e con persistenza di sessione (D-41).
- [ ] **Kit spostato.** Su un'altra lettera di unità e in una cartella con spazi tutto funziona, lo stato dell'editor resta e Claude ritrova la cronologia.
- [ ] **PC senza WebView2** (Windows 10 LTSC o macchina virtuale). Il messaggio è chiaro e l'app funziona con il runtime fisso nel kit.
- [ ] **Utente standard** senza diritti di amministratore: nessuna richiesta UAC in nessun passo.
- [ ] **Proxy autenticato e ispezione SSL.** Impostazioni › Rete con proxy e file PEM: installazione, estensioni dell'editor e agenti funzionano.
- [ ] **Senza rete.** L'app e l'editor partono, gli errori di download si leggono, il controllo degli aggiornamenti fallisce senza bloccare nulla.
- [ ] **Chiusura forzata dal Task Manager.** Non resta nessun `node.exe`, server dell'editor, console del kit o server di progetto.
- [ ] **ARM64.** L'exe è nativo, l'editor gira in emulazione e l'avviso si vede.

### Progetti
- [ ] **Nuovo progetto** «prova 1»: la cartella compare, ha il pulsante «Elimina la cartella vuota» e l'eliminazione funziona.
- [ ] **Clona** `nabre/atollo` (o un repository privato con `gh auth login` fatto nel terminale del kit): la cartella ha il nome giusto. Con un indirizzo non valido (`-x/y`, `ssh://…`) l'errore si legge.
- [ ] **Progetto Vite.** «Avvia», poi la porta 7100-7999 compare nell'output. Lo stato passa a «in ascolto» e il pulsante con l'indirizzo apre il browser del PC. Un secondo avvio riusa la stessa porta.
- [ ] **Porta occupata.** Scelta a mano una porta già usata da un altro programma, l'avvio dà un errore chiaro e la porta non cambia da sola.
- [ ] **Sito statico.** Una cartella con solo `index.html`: il server di file del kit lo serve. Un file `.env` non è raggiungibile (403).
- [ ] **«Servi come sito web».** Con la cartella `public` e PHP: `panel.json` contiene l'azione `site` e le altre chiavi del file restano.
- [ ] **«Ferma»** su `npm run dev`: in Gestione attività non resta nessun `node.exe`/`esbuild.exe` del progetto.
- [ ] **Electron o Tauri.** L'app apre la sua finestra e l'output arriva nella pagina.

### Agenti
- [ ] **Claude Code, Codex e Antigravity.** Accesso, avvio dall'app e dall'editor, allineamento di istruzioni e skill fra i tre.

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

### Accessibilità
- [ ] **Solo tastiera.** Tutte le pagine si usano: Tab mostra sempre il contorno di focus, «Vai al contenuto» è il primo elemento, il dialogo di chiusura riceve il focus.
- [ ] **Assistente vocale.** Le pagine hanno titoli e le icone hanno un nome. Errori e stati sono annunciati (`role="alert"` e `role="status"`).
- [ ] **Tema e animazioni.** Il tema chiaro e quello scuro seguono Windows. Con «Effetti di animazione» spenti in Windows, nessuna icona gira.
