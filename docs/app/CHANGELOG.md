# Novità

## Rilascio 0.6.0 alpha (24/10/2026)

- **Versione 0.6.0** in `tauri/Cargo.toml` e `ui/package.json`; il tag `app-v0.6.0` pubblica i file per i sei sistemi.
- **«In arrivo nel prossimo rilascio» tolto** da README e sito, nelle quattro lingue (`DOC-V7-02`): le funzioni qui sotto sono nella 0.6.0; «C'è ancora da fare» parla della 0.6.0.
- **Corretti prima del rilascio**: `docs/app/DECISIONS.md` torna al suo testo (il commit di D-165 aveva ripetuto una riga prima di ogni carattere, 78 MB); in `test/ui/App.test.tsx` la prima apertura della scelta iniziale e della Guida, pagine caricate al bisogno, ha 5 secondi invece di uno, come serve al runner Windows.
- **Cosa porta**, rispetto alla 0.5.1:
  - **Agenti e modelli**: Pi, il quarto agente (D-152); i modelli locali GGUF con il server llama.cpp (D-151); i server MCP pronti per il kit e i progetti (D-156); un pulsante per agente (D-146).
  - **Il kit**: «Esporta diagnostica» e «Segnala un problema» (D-153); «Salute del kit» e `Atollo check --kit` (D-155); «Verifica i file» e `Atollo check --files` (D-159); «Copia del kit», esportazione e importazione (D-163); l'icona nell'area di notifica (D-145).
  - **Progetti e database**: copie di sicurezza prima di un ripristino o di un cambio di versione (D-154); «Copia blocco .env» (D-157); il QR del sito in rete locale (D-158); «Salva come base» e «Le mie basi» (D-162); i nuovi progetti in quattro gruppi; mongo-express (D-147) e MongoDB senza la scheda «Contenuto» (D-150).
  - **Interfaccia**: la pagina AI, con agenti e modelli locali insieme (D-165); il terminale dentro l'app anche su Windows, con ConPTY (D-148), e i terminali staccabili (D-149); la palette dei comandi (D-160); il riepilogo degli aggiornamenti (D-161); un clic sulla versione cerca gli aggiornamenti; un solo interruttore per «Attività».
  - **Guida**: sul sito, una pagina per argomento in quattro lingue (D-164).
  - **Correzioni**: i rilievi del report v7 (voce qui sotto).

## La pagina AI (23/10/2026)

- **Una pagina «AI»** (icona scintilla) al posto delle pagine «Agenti» e «Modelli locali» (D-165). A sinistra due riquadri: «Agenti», con gli agenti installati e per ultima la riga «Condiviso fra gli agenti» (istruzioni, skill e server MCP), e «Modelli locali», con il server llama.cpp e i modelli nel kit. A destra la riga scelta; all'apertura il primo agente installato, o i modelli se non ce n'è nessuno.
- **Ciò che gli agenti condividono ha una riga sua**: AGENTS.md, skill, mcp.json e il riquadro «Server MCP del kit» non si ripetono più sotto ogni agente.
- **La chiave API di Gemini** è in AI › Antigravity › «Accesso», accanto agli altri accessi: le Impostazioni non hanno più la sezione «Agenti» e passano a sei sezioni.
- Le azioni dei riquadri («Aggiorna i consumi», «Apri la cartella condivisa», «Aggiorna i modelli», «Apri la cartella dei modelli») compaiono sul titolo del riquadro. I link della guida ad Agenti e a Modelli locali aprono la pagina AI sul riquadro giusto; il capitolo 4 della guida è «AI: agenti e modelli locali».

## Rilievi del report v7 (23/10/2026)

Il report v7 (`docs/reports/v7/`) chiuso in quattro ondate, un commit per rilievo; la verifica della chiusura è in `docs/reports/v8/00-VERIFICA-CHIUSURA-V7.md`.

- **Ondata 1, `fix/v7-bugs`** (con `fix/v7-llm-setup`, `fix/v7-projects-db`, `fix/v7-core` e `fix/v7-ui`, uno per gruppo, in worktree paralleli): i 39 rilievi del codice `BUG-V7-01..39`. Un download di un modello ripreso dopo un cambio di commit riparte dai file giusti e un file della dimensione giusta si accetta solo con lo SHA giusto; un lock per cartella del modello, preso in modo atomico; il nome di una cartella con `"` o un a capo non entra più in `httpd.conf`; l'editor che parte piano non blocca la finestra; «Salva e apri» e i «Ferma» del pannello Attività dicono perché non sono riusciti; processi raccolti su macOS e Linux, `TMPDIR` letto all'avvio, percorsi confrontati con le regole del sistema; testi nella lingua dell'utente e la prova i18n che li vede tutti.
- **`fix/v7-followups`**: il token di GitHub non è più nell'ambiente dell'agente, dove i server MCP di un repository clonato lo leggevano; il server github del kit è `Atollo mcp github`, un ponte che lo manda solo al server MCP di GitHub (`SEC-V7-01`); una prova del file server che cadeva sotto carico (`TEST-V7-02`); metà delle CPU come worker di Vitest (`TEST-V7-01`).
- **Ondata 2, `refactor/v7-structure`**: senza cambiare comportamento, `projects/run.rs`, `env.rs`, `projects/caddy.rs`, `install/mod.rs`, `kit.rs` e `main.rs` divisi in parti; riesportazioni al posto delle funzioni che inoltrano, un solo orologio, le sonde delle porte in `crate::ports`; in `ui/src` le cartelle `shared/`, `shell/`, `servers/` e gli alias `@/`, `@guide`, `@test`; la matrice degli strumenti in JavaScript; un aiuto comune per gli script delle schermate e per le prove Rust (`STR-V7-01..22`).
- **Ondata 3, `feature/v7-guide-site`**: la Guida in `guide/` alla radice, letta dall'app e dal sito con un contratto di widget, senza più `GUIDE.md`; la Guida sul sito in quattro lingue, la CI che costruisce e controlla il sito a ogni modifica (`GUI-V7-01..12`, D-164).
- **Ondata 4, `docs/v7-consistency`**: il numero degli agenti del sito dal catalogo; Pi, modelli locali e diagnostica «in arrivo nel prossimo rilascio» e la voce «Non ancora rilasciato» qui sopra; Git che arriva con Claude Code e Pi; «Modelli locali» negli elenchi delle pagine; schermate rifatte con Pi e la pagina dei modelli; otto tipi in Strumenti, ConPTY e i MB del primo avvio su Windows; `diagnostics.rs`, `health.rs`, `backup.rs` e `tray.rs` nell'architettura; la chiave dei modelli locali nel `models.json` di Pi; «Il kit» con le sue azioni; quattro agenti in MARKETING (`DOC-V7-01..13`); l'informativa con il server MCP di GitHub e le pagine della Guida.

## La Guida sul sito (23/10/2026)

- **La Guida si legge sul sito**, senza installare Atollo: una pagina per argomento e per lingua su https://nabre.github.io/atollo/guide/ (inglese) e `it/guide/`, `fr/guide/`, `de/guide/`, con i capitoli a lato, la ricerca nei titoli e nelle parole chiave, le schermate da ingrandire e il link «Guida» nell'intestazione del sito (D-164).
- **Niente più `GUIDE.md` e `GUIDE.it.md`**: i README rimandano al sito, che ha la Guida in quattro lingue invece di due. Nell'app la Guida è la stessa di prima.
- Il testo della Guida sta in `guide/` alla radice, letto dall'app e dal sito; la CI costruisce e controlla il sito a ogni modifica, prima del merge.

## Copia del kit (22/10/2026)

- **«Copia del kit»** in Impostazioni › «Il kit»: «Esporta in una cartella…» o «Esporta in uno zip…» copia i progetti e i dati (impostazioni, database, agenti) in `Atollo-<data-ora>`, senza i programmi, le cache e i modelli locali, che si riscaricano. È la copia di sicurezza del README, fatta da Atollo (D-163).
- **I database si fermano** per la copia, così i loro file sono coerenti; dopo ripartono quelli con «Avvia con Atollo». Le azioni dei progetti in corso restano accese.
- **«Importa da una cartella…» o «Importa da uno zip…»** controlla la copia e la prepara; chiudi Atollo con «Chiudi Atollo per importarla» e riaprilo: all'avvio la copia prende il posto dei progetti e dei dati. Quelli di prima vanno in `.atollo-before-import-<data-ora>` nella cartella del kit, mai cancellati; «Annulla l'importazione» ci ripensa.
- **Su un altro PC o un altro disco**: i programmi già scaricati nel kit restano, quelli che mancano si riscaricano da soli, e il kit si sistema come quando lo sposti. L'accesso di Antigravity va rifatto su un altro PC o account.
- **La copia contiene le password del kit** in chiaro: tienila al sicuro. Atollo non la manda da nessuna parte.

## Le mie basi (22/10/2026)

- **«Salva come base»** nella testata della pagina del progetto, accanto ad «Apri la cartella»: i file del progetto diventano una base di «Nuovo progetto», da cui crearne altri uguali, per esempio un Laravel con i pacchetti di sempre (D-162).
- **Scegli che cosa copiare**: la finestra mostra ogni cartella e file della radice con il numero dei file e la dimensione, e ognuno si può togliere; più il nome e la descrizione della base.
- **Solo i file che servono**: in un repository, quelli che Git terrebbe (`.gitignore` rispettato); senza Git, tutto tranne `node_modules`, `vendor`, `.venv` e le altre cartelle senza sorgenti. **Mai `.env`** né le sue copie, con le password; `.env.example` resta.
- **«Le mie basi»**, il primo gruppo (con la stella) della procedura, dell'albero di «Nuovo progetto» e del «Pannello rapido»: creare un progetto da una tua base copia i suoi file, poi `.env`, passi e azioni come per le altre. Salvarla di nuovo con lo stesso nome la sostituisce; toglierla nelle istruzioni dei framework in Impostazioni toglie anche i suoi file.
- Il riquadro «Nuovo progetto» della barra laterale mostra una base appena salvata dopo aver riaperto la pagina Progetti. I nomi dentro i file (`package.json`, `composer.json`) restano quelli del progetto originale.

## Riepilogo degli aggiornamenti (22/10/2026)

- **Il riquadro «Aggiornamenti»** nella barra laterale di Strumenti elenca i componenti installati che hanno una versione nuova, con la versione installata e quella nuova (2.100.0 → 2.103.0): invece di «Aggiorna tutto» alla cieca, vedi che cosa è indietro e scegli (D-161).
- **Il numero** compare sul titolo del riquadro e sull'icona di Strumenti nella barra delle attività («Versioni nuove: 2»).
- **«Cosa cambia»** apre nel browser le note di rilascio della versione nuova (la pagina dei cambiamenti o delle release del componente), o il suo sito quando non ce ne sono; **«Aggiorna»** aggiorna solo quel componente.
- **Una volta al giorno**, dopo l'avvio, solo con «Aggiornamento automatico» attivo nelle Impostazioni: Atollo chiede a ogni fonte l'ultima versione dei componenti installati, come farebbe un aggiornamento. «Controlla ora» chiede subito, anche con l'aggiornamento automatico spento. Le versioni si confrontano per numeri: 2.10 viene dopo 2.9.
- **Corretto**: «Sito ufficiale» nella pagina di uno strumento non apriva niente; ora apre il sito nel browser.

## Palette dei comandi (22/10/2026)

- **Ctrl+Shift+P** (⌘⇧P su macOS) apre «Comandi» da qualsiasi pagina, anche con il cursore in un terminale: scrivi cosa vuoi fare e premi Invio, come in VSCodium (D-160).
- **Per ogni progetto**, dal più recente: «Avvia» o «Ferma» (la sua azione predefinita), «Apri» (la sua pagina), «Apri … nell'editor» e ogni agente installato nel progetto («Claude Code in shop»). Poi «Avvia» o «Ferma» per MySQL e MongoDB, «Vai a» per le altre pagine e, quando qualcosa è in corso, «Ferma tutto», che qui non chiede conferma: sceglierlo è già la conferma.
- **Più parole, in qualsiasi ordine**: «avvia shop» e «shop avvia» trovano la stessa voce. ↑ e ↓ scelgono, Esc o un clic fuori chiudono; se un comando non riesce, l'errore compare sotto la barra del titolo.
- **Ctrl+P** apre ancora «Apri un progetto» come prima; in un terminale Ctrl+P resta della shell.
- In inglese e in tedesco il pulsante «Ferma tutto» del pannello «Attività» ora si chiama «Stop everything» e «Alles stoppen».

## Verifica dei file dei componenti (22/10/2026)

- **«Verifica i file»** nella pagina Strumenti, accanto a «Verifica»: confronta i file di ogni programma installato con quelli della sua installazione e trova quelli mancanti o cambiati, per esempio su una chiavetta danneggiata. «Verifica» dice solo se un programma risponde; questa trova la libreria rotta che lo fa chiudere più tardi (D-159).
- **«Reinstalla»** su ogni cartella danneggiata, e «Reinstalla tutti» quando sono più di una: rimette i file del programma alla stessa versione; progetti e database non si toccano. Se va tutto bene: «Nessun file danneggiato» con il numero di cartelle verificate.
- **Non sono un danno** i file aggiunti dopo l'installazione (pacchetti pip, gem, workload di .NET) e quelli che Atollo o i programmi cambiano apposta (`httpd.conf`, `config.inc.php`, i log). Rust, Android e TinyTeX non si verificano: la loro cartella la cambiano loro.
- **Dalla prossima installazione**: Atollo scrive l'elenco dei file (`.atollo-manifest`, con dimensione e SHA-256) alla fine di ogni installazione. I componenti installati prima compaiono come «senza manifesto» e si verificano dal loro prossimo aggiornamento.
- **`Atollo check --files`** fa la stessa verifica dalla riga di comando e finisce con 1 quando qualcosa è danneggiato.

## QR del sito in rete locale (22/10/2026)

- **«Dal telefono»** nel dettaglio di un'azione in corso, nella pagina del progetto, quando «Rete locale» è accesa nelle Impostazioni: l'indirizzo del sito con l'IP di questo PC sulla rete, con «Copia» e «Mostra il QR». Inquadri il QR con la fotocamera del telefono, collegato alla stessa rete, invece di ricopiare l'IP (D-158).
- **Il QR lo crea Atollo**, sul PC: nessun servizio su internet vede l'indirizzo. Scuro su bianco, si legge anche con il tema scuro.
- **Solo per gli indirizzi di questo PC** (`localhost`, `127.0.0.1`): un sito di un server condiviso con un nome come `shop.localhost` non ce l'ha, perché il telefono non può mandare quel nome. Il sito va avviato dopo aver acceso «Rete locale»; al primo avvio il firewall di Windows può chiedere il permesso: consentilo per le reti private.

## Copia blocco .env (22/10/2026)

- **«Copia blocco .env»** nella sezione «Database» della pagina del progetto, accanto a «Scrivi in .env»: copia la connessione come righe di `.env`, da incollare dove vuoi (un altro `.env`, un file di configurazione, un terminale); compare «Copiato» (D-157).
- **Le righe giuste per il progetto**: `DB_CONNECTION`, `DB_HOST`, `DB_PORT`, `DB_DATABASE`, `DB_USERNAME` e `DB_PASSWORD` per un progetto Laravel (il suo `.env` ha `DB_CONNECTION=`), altrimenti `DATABASE_URL` (MySQL) o `MONGODB_URI` (MongoDB). Il blocco contiene la password: Atollo lo legge solo quando premi il pulsante.

## Server MCP pronti (22/10/2026)

- **«Server MCP del kit»** nella pagina Agenti: l'elenco dei server di `data\agents\mcp.json`, con «Togli», «Aggiungi un server» e i **server pronti** da aggiungere con un clic: **github** (issue, pull request e repository con l'account GitHub del kit, il server di GitHub stesso), **playwright** (un browser che l'agente apre e usa) e **fetch** (una pagina web come testo). Arrivano subito a Claude Code, Codex, Antigravity e Pi (D-156).
- **Server pronti del progetto** in «Editor» › «Server MCP del progetto»: **filesystem** (i file, solo nella cartella del progetto) e, quando il database del progetto è stato creato, **mysql** (DBHub) o **mongodb**.
- **Nessun segreto nei file**: i server leggono il token di GitHub e il database del progetto da una variabile (`${DSN}`, …) che Atollo dà all'agente quando lo avvia. Il token di GitHub va solo ai server del kit, mai a quelli che il `.mcp.json` di un repository clonato chiede.
- **I programmi dei server del progetto** (Node per `npx`, uv per `uvx`) si installano da soli all'avvio dell'agente, come quelli del kit.

## Salute del kit (22/10/2026)

- **«Salute del kit»** in Impostazioni › «Il kit» (e `Atollo check --kit` dalla riga di comando) dice subito che cosa del PC e del disco può fermare il kit, ognuno con che cosa fare; «Controlla di nuovo» rifà i controlli dopo una correzione (D-155).
- **Il disco**: FAT32 (file fino a 4 GB, niente collegamenti), exFAT, e su macOS e Linux un disco montato `noexec` o `showexec`, con il comando per rimontarlo. **Lo spazio libero**: errore sotto 1 GB, avviso sotto 5 GB.
- **Il percorso del kit**: un avviso se contiene `&`, `%`, `!`, `^`, apici o lettere accentate, che cmd, PowerShell, Maven e Gradle gestiscono male.
- **File resi privati da un altro account**, magari su un altro PC: quanti sono e il comando da eseguire da amministratore per riprenderli (`takeown` e `icacls` su Windows, `chown` su macOS e Linux).
- **Su Linux, le librerie del sistema** che servono al kit: libaio e libnuma (MySQL), libicu (.NET), libvulkan1 (la scheda video dei modelli locali), con il comando `apt` per installarle.
- `Atollo check --kit` finisce con 1 quando un controllo è un errore: si può usare in uno script.

## Copie di sicurezza dei database (22/10/2026)

- **Prima di un ripristino** il kit copia ciò che il database del progetto contiene ora, in `<database>-prima-del-ripristino-<data e ora>` nella cartella delle copie: se hai ripristinato il file sbagliato, ripristini quella copia e torni a prima. Se la copia non riesce il ripristino non si fa. La conferma lo dice: «Ripristinare …? Prima il kit copia ciò che il database contiene ora.» (D-154)
- **Prima di cambiare versione** del server di un database (MySQL o MongoDB), il kit copia il database sulla versione che lascia, in `<database>-prima-del-cambio-di-versione-<data e ora>`; se la copia non riesce la versione non cambia.
- **Solo le ultime 3** copie di sicurezza di ogni tipo restano, le più vecchie si eliminano; le copie fatte con «Esporta» non si eliminano mai.
- **Ripristino MySQL più robusto**: viste, trigger, routine ed eventi creati da root non fermano più il ripristino su MySQL 8.0.16 e successivi (tornano con l'utente del progetto), e una vista rotta non ferma la copia di sicurezza.

## Segnalare un problema (22/10/2026)

- **«Esporta diagnostica»** in Impostazioni › «Il kit» (e `Atollo diagnostics` dalla riga di comando) crea uno zip in `data\app\diagnostics` con quello che serve per capire un problema: versione e sistema, disco del kit, RAM, ambiente, verifica dei programmi, componenti, porte e la parte finale dei log. «Apri la cartella» lo mostra (D-153).
- **Niente segreti nello zip**: chiavi, token e password che il kit conserva (database, agenti, GitHub, Hugging Face, il server dei modelli locali) diventano `***` ovunque compaiano, la cartella del tuo profilo diventa `~`, il tuo nome utente e il nome del PC `***`. Lo zip resta sul PC: lo guardi e lo alleghi tu.
- **«Segnala un problema»** apre nel browser una segnalazione nuova su GitHub, con versione di Atollo e sistema già scritti: racconti che cosa è successo, i passi per rifarlo, e trascini lo zip. L'app non manda niente; la segnalazione arriva a GitHub solo quando la crei.

## Modelli locali e Pi, il quarto agente (22/10/2026)

- **Modelli linguistici sul PC**, senza rete e senza account: la nuova pagina «Modelli locali» scarica modelli GGUF dal catalogo di Atollo (Qwen2.5-Coder 7B, Qwen3.5 9B, gpt-oss 20B, Devstral Small 2 24B, Qwen3-Coder 30B-A3B) o da qualunque repository di Hugging Face, e dice se entrano nella RAM o nella memoria video del PC. I modelli stanno in `data\models`, una copia per Windows, macOS e Linux (D-151).
- **Download che riprendono**: un modello di molti GB si può sospendere e riprende da dove si è fermato, anche dopo aver chiuso Atollo; ogni file è controllato col suo SHA-256. Su un disco FAT32 i file oltre i 4 GB sono rifiutati prima di iniziare, con cosa fare.
- **Il server dei modelli** (llama.cpp, su CPU o GPU con Vulkan o Metal) ascolta solo su questo PC, con una chiave; carica il modello quando serve e lo libera dopo dieci minuti. Compare in Attività, si ferma con «Ferma tutto» e «Rimuovi l'unità»; «Apri la chat» apre la sua chat nel browser.
- **Pi**, l'agente da terminale di Earendil, si aggiunge da Strumenti e parte dai progetti come gli altri: usa da solo un modello locale che sa usare gli strumenti, oppure un provider in cloud con `/login`. Istruzioni, skill e server MCP sono condivisi con Claude Code, Codex e Antigravity (D-152).
- **Riga di comando**: `Atollo models`, `models pull`, `models rm`, `llm start|stop|status`.

## Terminali staccabili (22/10/2026)

- **«Sposta in una nuova finestra»** nel pannello dei terminali: la scheda scelta continua in una finestra di Atollo tutta sua, da spostare su un altro schermo o ingrandire, con la stessa sessione e la sua uscita. «Riporta nel pannello» la rimette tra le schede; chiudere la finestra fa lo stesso e non ferma il programma. «Chiudi il terminale» lo ferma, dalla finestra o dal pannello (D-149).
- **Il carattere giusto da subito**: il terminale si apre quando Geist Mono è pronto, anche con le cornici delle TUI e il grassetto, così le colonne non si storcono in una finestra nuova; se il carattere arriva tardi, il terminale si rimisura.

## La versione nella barra di stato cerca gli aggiornamenti (22/10/2026)

- **Un clic sulla versione** (l'ultima voce della barra di stato) cerca gli aggiornamenti di Atollo, come «Cerca aggiornamenti» in Impostazioni › Aggiornamenti: mentre cerca gira la rotellina, poi accanto compaiono un segno di spunta (è la più recente), la versione nuova (pronta per la prossima chiusura, o da scaricare) o un avviso (controllo non riuscito); passandoci sopra si legge l'esito per intero.

## Un solo interruttore per «Attività» nella barra di stato (21/10/2026)

- **Barra di stato più pulita**: il pannello «Attività» si apre e si chiude solo dal contatore ▶ · ⨯, come «Problemi» in VSCodium. Tolta l'icona del pannello in fondo a destra, che faceva la stessa cosa; la rotellina di un'installazione in corso resta visibile ma non è più un pulsante.

## MongoDB senza la scheda «Contenuto» (21/10/2026)

- **Una pagina sola**: la pagina di MongoDB mostra solo i dettagli, come quella di MySQL; il contenuto si esplora e si modifica in mongo-express («Apri mongo-express»). Tolto l'esploratore in sola lettura dell'app, che faceva meno di mongo-express (D-150).

## Il terminale dentro Atollo anche su Windows (21/10/2026)

- **Un'interfaccia sola**: su Windows «Terminale» del progetto, «Apri», «Salva e apri» e «Apri in…» della pagina Terminali e i pulsanti degli agenti aprono una scheda del pannello dei terminali dentro l'app, come su macOS e Linux. Niente più finestra della console classica, mai Windows Terminal né il terminale del PC (D-148).
- **Claude Code riceve i tasti**: Invio, frecce, Esc e Ctrl+C arrivano a Claude Code e a Codex. Il terminale usa il ConPTY aggiornato di Microsoft (`conpty.dll` e `OpenConsole.exe`, la console di Windows Terminal e VS Code), una parte dell'app scaricata da NuGet con i componenti obbligatori, verificata con lo SHA-256 e la firma di Microsoft. Se manca, l'app lo scarica prima di aprire il terminale; se non ci riesce lo dice nell'app.
- **Copia e incolla** come in Windows Terminal: Ctrl+C copia il testo selezionato (senza selezione arriva al programma), Ctrl+Maiusc+C copia, Ctrl+V e Ctrl+Maiusc+V incollano. Ctrl+P nel terminale va alla shell, non apre «Apri un progetto».
- **Caratteri larghi**: emoji e ideogrammi occupano le colonne giuste anche quando si torna indietro con le frecce.
- **Chiusura**: chiudere la scheda, «Chiudi tutto» o «Rimuovi l'unità» fermano il programma e tutto ciò che ha avviato, compreso il server in background di Codex; la conferma alla chiusura conta i terminali aperti.
- **Il pannello dei terminali si apre davvero**: la protezione del prototipo dell'interfaccia impediva a xterm.js di caricarsi (anche su macOS e Linux); ora la protezione resta e il pannello si apre.

## L'intestazione del terminale (21/10/2026)

- **«ATOLLO» in grande** all'apertura del terminale del kit (PowerShell e Bash, su ogni sistema): lettere a blocchi nel colore del terminale, l'ultima O è l'atollo del wordmark (smeraldo, ambra e corallo con l'isolotto, la laguna e le onde). Sotto, il nome del terminale e gli strumenti trovati. `ATOLLO_QUIET` la toglie, come prima.

## Nuovi progetti in quattro gruppi (21/10/2026)

- **Pannello rapido**: sotto «Nuovo progetto», «Progetto vuoto» e «Da un indirizzo Git» diventano due collegamenti accanto al titolo; lo spazio va a un riquadro per gruppo, «Web», «CMS», «App» e «Altro», con le basi come pulsanti (logo e nome) sotto le loro categorie. Un clic su una base apre la procedura già su quella base; i riquadri stanno affiancati quando c'è spazio, uno sotto l'altro quando è stretto.
- **Albero della barra laterale**: al posto di «Da framework o CMS» i quattro rami Web, CMS, App e Altro, ciascuno con il numero delle basi; sotto Web, App e Altro le categorie, sotto CMS direttamente le basi. Si apre il ramo della base mostrata; un clic su un gruppo apre la procedura sulla sua prima base.
- **Gruppi**: Web raccoglie PHP, JavaScript e TypeScript, Python, gli altri linguaggi e i siti statici e la documentazione; App le app desktop e mobile; Altro i dati e documenti, la riga di comando e le basi fatte in Impostazioni senza categoria. La scheda e l'intestazione della procedura dicono il gruppo della base scelta; l'icona della barra laterale diventa «Nuovo progetto da una base».
- **Riquadri della barra laterale**: con «Nuovo progetto» o «Dai tuoi repository GitHub» aperti, «Progetti» prende solo l'altezza delle sue righe e scorre solo se non ci stanno; il resto va agli altri riquadri, come «Editor aperti» in VSCodium. Da solo, «Progetti» occupa tutta l'altezza come prima.

## mongo-express: MongoDB nel browser (21/10/2026)

- **«Apri mongo-express»** nella pagina di MongoDB in Database, al posto e con l'aspetto di «Apri phpMyAdmin» per MySQL: la prima volta installa mongo-express (e Node.js se manca) mostrando l'avanzamento, accende MongoDB se è spento e apre nel browser database, collezioni, documenti, indici, importazione ed esportazione, già collegato come admin. Per un'altra versione c'è «mongo-express su …» (D-147).
- **Una build di Atollo**: il pacchetto npm di mongo-express non parte (manca la parte compilata dell'interfaccia), quindi un workflow lo compila dal suo codice a un commit fissato, lo prova su un MongoDB vero e lo pubblica nelle release di Atollo; il kit lo scarica da lì e lo controlla col suo SHA-256.
- **Stessa protezione di phpMyAdmin**: indirizzo di loopback suo, porta che nessun'altra parte di Atollo usa, gettone nuovo a ogni avvio, 403 senza; la password di admin solo nell'ambiente del processo. Si ferma con la sua versione di MongoDB, con «Ferma tutto», «Rimuovi l'unità» e la chiusura dell'app.
- **Strumenti**: «Database» elenca solo i server (MySQL, MongoDB, SQLite); subito dopo la nuova sezione «Visualizzazione database» con phpMyAdmin e mongo-express, nella barra laterale, nella tabella «Tutti gli strumenti», nella matrice del sito e nella tabella delle fonti della Guida.

## Il terminale dell'editor resta quello del kit (21/10/2026)

- **Profili lasciati da un kit precedente**: con un nome che il kit non usa più, puntavano a una cartella di PowerShell o Git con la versione; dopo un aggiornamento il terminale predefinito dell'editor mancava e VSCodium apriva la shell del PC. Ora quei profili si tolgono e il predefinito torna a «Atollo».
- **Nessun nome di prima**: tolti dal codice e dai documenti anche i riferimenti al progetto da cui Atollo è nato.

## Un clic sul logo apre l'agente (21/10/2026)

- **Un pulsante per agente** nella pagina del progetto, nella sua riga della barra laterale e nel «Pannello rapido»: il logo di Claude Code, Codex o Antigravity apre subito quell'agente nel progetto. Non si sceglie più l'agente prima di premere «Agente» (D-146).
- **Solo gli agenti installati** hanno il loro pulsante; l'errore di un avvio compare sul pulsante di quell'agente.

## «Apri nel browser» sui siti dei pool (21/10/2026)

- **«Apri nel browser»** apre anche i siti serviti da un pool (`http://<progetto>.localhost:<porta>/`): prima l'app accettava solo `localhost` e `127.0.0.1`, e il pulsante non faceva nulla.

## Codice più ordinato e interfaccia più leggera (21/10/2026)

- **Interfaccia più leggera**: il file principale passa da 1,2 MB a 258 kB; Impostazioni, Strumenti e la procedura guidata si caricano quando si aprono, e solo i componenti di `vscode-elements` usati vengono registrati.
- **Letture**: le informazioni del kit, l'elenco di «Rimuovi l'unità» e le impostazioni ignorano una lettura vecchia che risponde tardi; Impostazioni e primo avvio hanno «Riprova».
- **Controlli**: un campo con suggerimenti (estensioni del progetto), pulsanti a scelta con testo (procedura guidata) e misuratori con valore (limiti degli agenti) nello stile di VSCodium.
- **Codice**: gli installatori, la rete, i database e il portachiavi degli agenti divisi in cartelle, nessun file oltre le 450 righe; le prove dell'interfaccia partono dai dati dell'anteprima; `tool-versions.json` fuori dall'impronta della matrice.

## L'icona nell'area di notifica (21/10/2026)

- **Icona di Atollo** nell'area di notifica di Windows, nella barra dei menu di macOS e nell'area di stato di Linux (AppIndicator), con il menu nella lingua dell'app: «Mostra Atollo», «Progetti» (i più recenti, il clic apre la pagina del progetto) e «Chiudi Atollo», che chiede conferma come la chiusura della finestra se qualcosa è aperto (D-145).
- **Clic sinistro** sull'icona (Windows, macOS): la finestra torna in primo piano. Su Linux senza la libreria AppIndicator l'app parte senza icona.

## Rilascio 0.5.1 alpha (21/10/2026)

- **Versione 0.5.1** in `tauri/Cargo.toml` e `ui/package.json`; il tag `app-v0.5.1` pubblica i file per i sei sistemi.
- **Cosa porta**: tutti i rilievi del report v6 chiusi (database, installazioni, kit su un altro disco o PC, agenti, progetti e server web, interfaccia), i token degli agenti solo agli agenti e un progetto fidato quando ci avvii un agente (D-144), la documentazione allineata all'app nelle quattro lingue.
- **«C'è ancora da fare»** (README e sito, nelle quattro lingue) parla della 0.5.1.

## Rilievi del report v6 (21/10/2026)

- **Database**: un backup MySQL con viste, trigger, routine, eventi o colonne binarie si ripristina intero; una copia compare solo quando è completa; un progetto con un nome lungo ha il suo database; un `secrets.json`, `services.json` o `servers.json` illeggibile non viene mai riscritto vuoto; i dati iniziali di MySQL copiati a metà non vengono aperti.
- **Installazioni**: un aggiornamento conda che fallisce lascia funzionante l'ambiente di prima; un download fermo finisce dopo un minuto e «Annulla» lo ferma subito; un file scaricato a mano resta finché l'installazione riesce; firma Authenticode con l'editore esatto; nomi di file dai metadati remoti controllati.
- **Kit su un altro disco o PC**: un kit alla radice di un'unità (`E:\`) si sposta senza rompere percorsi e JSON; i file che lo spostamento non ha potuto aggiornare si riprovano a ogni avvio; i file privati creati da un altro account vengono sostituiti o nominati nell'errore; Maven, Gradle e PowerShell accettano `&`, lettere accentate e `’` nel percorso.
- **Agenti**: token e chiave Gemini solo agli agenti, un progetto è fidato quando ci avvii un agente (D-144); le istruzioni comuni non si perdono se un file manca un attimo; dopo «Rimuovi unità» nulla scrive più sul disco; uscita da Antigravity rifiutata con agy aperto; salvataggi dell'accesso non riusciti ritentati con attese crescenti; log che ruota anche durante la sessione.
- **Progetti e server web**: un nome con spazi o graffe dà un indirizzo valido; un doppio clic su Avvia avvia una volta sola; uno snippet Caddy sbagliato non spegne il pool.
- **Interfaccia**: la chiave Gemini e le modifiche ai framework non si perdono; `npx vitest` dalla radice esegue solo le prove dell'interfaccia.
- **Documentazione**: README, aiuto della riga di comando, Guida, informativa e sito allineati all'app (`DOC-V6-01..16`).

## Rilascio 0.5.0 alpha (21/10/2026)

- **Versione 0.5.0** in `tauri/Cargo.toml` e `ui/package.json`; il tag `app-v0.5.0` pubblica i file per i sei sistemi.
- **Cosa porta**: i rilievi del report v5 chiusi (lock dei database e porte che restano, D-140; registro comune delle porte e `conf.d` del progetto anche nei pool e nel Caddy del progetto, D-141), l'aspetto del workbench di VSCodium 1.135 con le sue icone (D-142), le schermate del sito e dei README nella lingua della pagina (D-143).
- **Anche**: la porta automatica di un'azione evita quelle dei pool e dei database; Esc chiude anche la domanda di eliminazione di un repository GitHub; gli stessi termini per le stesse cose in italiano, inglese, francese e tedesco; la Guida, i README, il sito, l'informativa e i documenti tecnici descrivono la 0.5.0; la matrice degli strumenti (`docs/app/TOOLS-MATRIX.html`) è tornata.
- **«C'è ancora da fare»** (README e sito, nelle quattro lingue) parla della 0.5.0.

## Documentazione coerente in quattro lingue (21/10/2026)

- **README, sito e informativa**: le stesse sezioni e gli stessi fatti in inglese, italiano, francese e tedesco; le pagine dell'app come sono oggi.
- **Guida dell'app**: descrive Attività e «Ferma tutto», le porte rifiutate con il loro proprietario, i `conf.d` dei progetti, la rete locale, le versioni dei database; rigenerati `GUIDE.md` e `GUIDE.it.md`.
- **Documenti tecnici**: architettura (lock, registro delle porte, token, icone, script delle schermate), checklist dei test e catalogo seguono il codice, in inglese e italiano.
- **Testi dell'app**: termini uniformi nelle quattro lingue («Matrice», «Übernehmen», «vous» in francese, «USB stick»).
- **Anteprima nel browser**: mostra la versione vera dell'app; tutte le schermate rifatte.

## L'aspetto di VSCodium 1.135 (20/10/2026)

- **Cornice**: barra del titolo, barra delle attività, barra di stato e schede del pannello con le misure e i colori di VSCodium 1.135, la versione dell'editor del kit (D-142).
- **Liste e pannelli**: righe, intestazioni dei pannelli laterali e barre di scorrimento sottili come nel workbench.
- **Controlli**: pulsanti, campi, menu a tendina e finestre di dialogo delle stesse misure di VSCodium.
- **Editor**: schede con la chiusura che compare al passaggio, pagine degli strumenti con l'icona grande e la barra delle sezioni, impostazioni con indice e barra del valore modificato come in VSCodium.
- **Icone**: quelle di VSCodium, dal font codicon.
- **Guida**: schermate rifatte nelle quattro lingue.

## Rilievi del report v5, priorità 1-4 (20/10/2026)

- **Database**: due avvii insieme della stessa versione di MySQL fanno partire un solo server; cambiando la versione del server nessuna porta si sposta, e le stringhe di connessione dei progetti restano giuste (D-140). Due backup avviati insieme non si rubano più il file di login.
- **Porte**: un pool, una versione di un database o un'azione di un progetto non si salvano su una porta già usata da un altro, né in quelle dell'editor; il messaggio dice di chi è (D-141).
- **Pool di server web**: un sito nel pool usa il `conf.d` del suo progetto, come con il server dedicato (D-141).
- **Rete locale**: con i server dei progetti in rete permessi, `php -S` risponde ai dispositivi della rete che lo chiamano per indirizzo IP; ogni altro nome resta rifiutato.
- **Sito statico** raggiungibile anche come `<progetto>.localhost`.
- **Attività e barra di stato** contano anche le versioni secondarie dei database e i pool accesi, e li elencano; «Ferma tutto» ferma ogni cosa dopo una conferma.
- **Conferme**: «Elimina profilo» di un pool chiede prima; Esc chiude la domanda di rimozione di un terminale.
- **Terminali**: se la versione salvata di un terminale non c'è più, dice quale usa al suo posto (per esempio «PHP 8.2 → 8.4»).
- **Wizard**: la password del database generata vince su una vuota della base.
- **Server web**: un progetto creato, clonato o cancellato compare o sparisce subito.
- **Sviluppo**: il server dell'interfaccia si ferma anche quando Tauri chiude la pipe dello stdin.

## Rilascio 0.4.0 alpha (20/10/2026)

- **Versione 0.4.0** in `tauri/Cargo.toml` e `ui/package.json`; il tag `app-v0.4.0` pubblica i file per i sei sistemi.
- **Cosa porta**: impostazioni salvate da sole e scelte al primo avvio prima di ogni download (D-137), informativa sulla privacy, tema chiaro/scuro, chi firma i commit, Git limitato alla cartella del kit, proxy e certificati anche per i download dell'app, strumenti divisi per tipo come nel sito (D-139), il logo di MySQL, il server di sviluppo che si chiude con l'app (D-138).
- **Report v5** in `docs/reports/v5`: valutazione `95,8/100`, 958 test, rilievi aperti e piano per la 0.5.0 alpha.
- **«C'è ancora da fare»** (README e sito, nelle quattro lingue) parla della 0.4.0.

## Strumenti divisi per tipo (20/10/2026)

- **«Tutti gli strumenti»** è diviso in Sistema ed editor, Agenti AI, Linguaggi, Strumenti di sviluppo, Server web e Database, come nel sito: ogni blocco ha titolo, numero, descrizione e una casella per sceglierli tutti, e si chiude con un clic (D-139).
- Gli stessi tipi nell'albero di Strumenti, nella Matrice e al primo avvio.

## Sviluppo: il server dell'interfaccia si chiude con l'app (20/10/2026)

- **`npm run dev`**: chiudendo l'app si ferma anche Vite, così la porta 5173 resta libera per la volta dopo; se è occupata, dice da chi e come fermarlo (D-138).

## Impostazioni senza «Salva», primo avvio, privacy (20/10/2026)

- **Impostazioni**: ogni modifica si salva e si applica da sola, senza «Salva»; anche le istruzioni dei framework. La chiave di Gemini si salva quando lasci il campo (D-137).
- **Primo avvio**: lingua, aggiornamenti e rete (proxy, certificati) si scelgono nella pagina di benvenuto, prima di scaricare qualunque cosa.
- **Privacy**: nuova sezione nelle Impostazioni e informativa `docs/PRIVACY.md` in inglese, italiano, francese e tedesco, collegata da README e sito.
- **Più robuste**: le chiavi di una versione più nuova restano nel file, due modifiche contemporanee non si perdono più, l'ambiente del kit non torna indietro dopo un salvataggio.
- **Aree vuote** con lo sfondo del marchio.
- **macOS e Linux**: «Lingua del sistema», esempi e percorsi con la barra giusta, il compilatore di Rust solo dove serve.
- **Chiave di Gemini** in un file proprio, leggibile solo dal tuo utente, non più in chiaro in `settings.json`.
- **Proxy e certificati** valgono anche per i download di Atollo: «Proxy HTTP», «Senza proxy per» e il file PEM, che ora raggiunge anche Python, pip, curl, Git e Cargo.
- **Atollo avviato da una console del kit** non riprende più lingua e proxy del kit come se fossero del PC.
- **Kit spostato**: il file PEM e le cartelle dei terminali dentro il kit lo seguono.
- **Tema** chiaro, scuro o come il sistema, per Atollo, l'editor e i terminali, anche al primo avvio.
- **Chi firma i commit**: nome ed e-mail di Git in Impostazioni › Account GitHub; con l'accesso a GitHub si compilano da soli, con l'indirizzo privato «noreply».
- **Git più prudente**: si fida solo dei repository dentro la cartella del kit, non più di ogni repository del PC.
- **Aggiornamento automatico** vale subito: acceso, cerca la nuova versione senza aspettare il prossimo avvio.
- **Niente più sezione «Versioni»**: le linee di Python e PHP si scelgono in Strumenti e per progetto; il compilatore di Rust (Windows) è nella pagina di Rust in Strumenti.

## Rilascio 0.3.0 alpha (20/10/2026)

- **Versione 0.3.0** in `tauri/Cargo.toml` e `ui/package.json`; il tag `app-v0.3.0` pubblica i file per i sei sistemi.
- **PHP del kit**: su macOS e Linux le linee 8.4 e 8.5 vengono dalle release `php-v8.4.26-1` e `php-v8.5.11-1` (D-133), non più da static-php-cli.
- **«C'è ancora da fare»** (README e sito, nelle quattro lingue) parla della 0.3.0.

## Social card, Search Console (20/10/2026)

- **Social card** `docs/brand/social.png` (1280×640): l'immagine del sito quando si condivide un link e l'anteprima del repository su GitHub (D-135).
- **Ricerca**: il sito pubblica il file che verifica la proprietà in Google Search Console (`site/root/`).

## Schermate della guida nella lingua giusta (19/10/2026)

- **Guida**: le 13 schermate in inglese, francese e tedesco non mostrano più testi italiani dei dati di esempio (ricette, terminale predefinito, sito statico, limiti degli agenti); rifatte tutte e 52 da `node scripts/guide-shots/make.mjs` (D-136).

## Azioni dall'app, dati senza progetto, clone da GitHub (19/10/2026)

- **Azioni**: in Progetti › «Avvio» si aggiungono, modificano e tolgono le azioni del progetto con un modulo, senza scrivere `panel.json` a mano (D-136).
- **«Collega a…»** dei dati senza progetto funziona anche con un progetto già aperto, se ha solo le sue impostazioni.
- **«Clona»** dall'elenco dei repository GitHub installa i componenti e sceglie le versioni, come «Da un indirizzo Git».
- **«pre-installato»** è l'unica etichetta per i componenti già nel kit che si completano al primo avvio.
- **Guida nei documenti**: `node scripts/guide-docs.cjs` rigenera `GUIDE.md` e `GUIDE.it.md`.
- **CI**: `node scripts/guide-docs.cjs --check` fallisce se `GUIDE.md` o `GUIDE.it.md` non corrispondono alla Guida dell'app.

## Guida nei documenti, schermate per lingua, tedesco con il «du» (19/10/2026)

- **GUIDE.md e GUIDE.it.md** (D-136) riscritte dalla Guida dell'app: nove capitoli, 36 argomenti, le stesse schermate.
- **Schermate** nella lingua dell'app (italiano, inglese, francese, tedesco); tolte le vecchie immagini di `docs/app/images`.
- **Tedesco**: la Guida e i testi dell'app danno del «du», come il sito e il README.

## La Guida ripensata (19/10/2026)

- **Nove capitoli** (D-136), dal più semplice al più tecnico: Primi passi, Progetti, Database siti e terminali, Agenti IA, Strumenti e linguaggi, Account e impostazioni, Portare il kit con sé, Come funziona dentro, Aiuto. 36 argomenti, ognuno con «In breve», i passi con le etichette dell'interfaccia, consigli, la parte tecnica ripiegata e gli argomenti collegati.
- **Nuovi argomenti**: un primo progetto passo per passo, la finestra spiegata parte per parte, la pagina Terminali, problemi comuni con il loro messaggio, domande frequenti, glossario.
- **Corretta** su ogni punto rispetto all'app di oggi: niente più funzioni che non esistono, percorsi e variabili verificati nel codice; screenshot nuovi dall'anteprima, senza il vecchio marchio.
- La pagina **Terminali** ha il suo pulsante della guida.

## Documentazione in inglese prima di tutto (19/10/2026)

- **README** (D-135): `README.md` è in inglese, l'italiano è in `docs/README.it.md`; nelle quattro lingue wordmark, badge in inglese, screenshot, «Features» e tabella dei documenti, con l'inglese primo nella barra delle lingue. Il tedesco dà del «du» ovunque.
- **Guida e documenti tecnici in inglese**: `GUIDE.md` (prima `GUIDA.md`), `ARCHITECTURE.md`, `CATALOG.md`, `TESTING.md`; l'italiano resta in `GUIDE.it.md`, `ARCHITECTURE.it.md`, `CATALOG.it.md`, `TESTING.it.md`. Schermate con nomi inglesi.
- **Matrice degli strumenti** in inglese: `docs/app/TOOLS-MATRIX.html` (prima `MATRICE-STRUMENTI.html`).
- **Sito**: l'inglese alla radice, l'italiano in `it/`, `en/` rimanda alla radice; `sitemap.xml` con le alternative, JSON-LD `SoftwareApplication`, Open Graph e Twitter card completi, parole chiave in inglese.
- **Metadati**: `keywords`, `homepage` e `repository` in `package.json` e `tauri/Cargo.toml`; descrizione e topic del repository su GitHub.

## Rimuovere uno strumento toglie anche i file scaricati in anticipo (19/10/2026)

- **Rimozione** (D-114): togliendo uno strumento dalla scelta, su ogni sistema del kit vanno anche i file scaricati in anticipo per il primo avvio su un altro sistema (`.bootstrap`, ora una cartella per strumento), gli elenchi e gli archivi dei suoi pacchetti conda conservati e i file aggiunti a mano per lui; un file che serve ancora a un altro strumento resta. Si toglie anche uno strumento di cui c'era solo quanto scaricato in anticipo.
- **Versioni**: rimuovere una linea dalla pagina Strumenti toglie anche i suoi file scaricati in anticipo e il suo elenco conda.

## Testi al passo con il rilascio 0.2.0 (19/10/2026)

- **«C'è ancora da fare»** (README e sito, nelle quattro lingue): la 0.2.0 alpha è su GitHub per i sei sistemi e va provata su macchine pulite; su Windows esce senza firma, SignPath è ancora da attivare.
- **Sistemi**: la hero e la statistica del sito nominano Windows, macOS e Linux, ognuno su x64 e ARM64; «Limiti noti» non parla più del «prossimo tag».
- **GUIDE.it.md** e **CATALOG.it.md**: su macOS e Linux c'è anche Apache, build del kit con `php-fpm` (D-116), fissata su `2.4.69-1`; per Linux anche il `tar.gz`.
- **Sito**: il titolo è «Sviluppo, tutto in una cartella, su qualsiasi computer.», il sottotitolo nomina Windows, macOS e Linux; le statistiche fanno parte della hero e restano sullo schermo con il titolo anche su finestre basse; tolta la sezione «Perché «Atollo»».
- **Scarica**: il sito prende l'ultimo rilascio `app-v*`, mai quelli di Apache o PHP dello stesso repository; senza rilascio dell'app il pulsante porta all'elenco dei rilasci.

## Il sito al passo con l'app (19/10/2026)

- **«Dentro l'app»**: le tre schermate stanno in una finestra vestita come il sistema di chi visita, la stessa della cartella nella hero (Windows 11, macOS, Linux con GNOME); senza JavaScript restano i puntini del brand. Barra e controlli sono un componente solo del sito, `.win`.
- **Testi**: «Come si usa» nomina le oltre 60 basi dei progetti, «Tutto da una finestra» il pannello rapido e Ctrl+P, i server web dicono «su ogni sistema». In «C'è ancora da fare» Apache fuori da Windows non è più in arrivo (D-116): resta da provare su macchine vere, con il PHP del kit con `pdo_sqlite` che arriva con il suo primo rilascio (D-133).
- **README**: la riga Progetti descrive pannello rapido, ▶ nella lista, Ctrl+P e la scheda Nuovo progetto con le 63 basi in gruppi; la voce da fare su Apache come nel sito. **CATALOG.it.md** e **TESTING.it.md** non dicono più R assente su Windows ARM64 (D-132) né Apache solo su Windows.

## PHP del kit per macOS e Linux, con pdo_sqlite (19/10/2026)

- **Build propria di PHP** (D-133, issue #4): `scripts/build-php` e il workflow `php.yml` compilano PHP 8.4.26 e 8.5.11 con static-php-cli sui quattro sistemi, con le estensioni del set «bulk» e in più `pdo_sqlite` e `pdo_pgsql` veri: `ext-pdo_sqlite` nei `composer.json` e `extension_loaded('pdo_sqlite')` funzionano.
- **Installazione**: una linea con la build del kit fissata nel catalogo viene dalle release del repository; le altre restano su static-php-cli fino alla prima release.

## R su Windows ARM64 (19/10/2026)

- **R** c'è anche sui PC Windows ARM64 (D-132): la stessa build `win-64` di conda-forge di Windows x64, con il compilatore per i pacchetti, in emulazione x64 come PHP, Apache e MySQL. conda-forge non ha R per `win-arm64` e l'R nativo ARM64 di CRAN è ancora sperimentale. Ora ogni componente del catalogo c'è su Windows ARM64.

## Apache su ogni sistema (19/10/2026)

- **Apache** c'è su Windows, macOS e Linux, x64 e ARM64 (D-116): su macOS e Linux arriva dal primo rilascio della build del kit, `httpd-v2.4.69-1`, con gli SHA-256 dei quattro archivi fissati nel catalogo; PHP passa da `php-fpm`. Su Windows niente cambia: Apache Lounge con il modulo PHP, x64 in emulazione su ARM64.

## 52 nuove basi per i progetti, in gruppi (19/10/2026)

- **Gruppi** (D-131): il wizard e gli alberi dei progetti e delle Impostazioni dividono le basi in Web con PHP, Web con JavaScript e TypeScript, Web con Python, Web con altri linguaggi, CMS, Siti statici e documentazione, App desktop, App mobile, Dati e documenti, Riga di comando e librerie.
- **Basi nuove**: Nuxt, SvelteKit, Astro, Angular, React Router, NestJS, Hono, Fresh, Elysia; Django, FastAPI, Flask; ASP.NET Core, Blazor, Spring Boot, Ktor, Gin, Axum, Dart; Craft CMS, Kirby, Grav, Joomla, Strapi, Payload; Docusaurus, Starlight, VitePress, Eleventy, MkDocs Material; Avalonia, Wails, Neutralinojs; Streamlit, JupyterLab, marimo, Shiny, Pluto, Typst, LaTeX; programmi in Rust, Go, Zig, C++, .NET, Python, Deno, Dart, Kotlin, Java e Julia.
- **Aggiunte** su più basi: Tailwind CSS, Bootstrap e Bulma, Vitest, Prettier, i componenti React, Vue e Svelte in Astro, MySQL per Django.
- Ogni base ha il suo logo.

## Arretrato delle piattaforme (19/10/2026, issue #8)

- **pnpm su Windows**: rimuovendo il componente «pnpm, yarn» va via anche il corepack che aveva installato, come già su macOS e Linux.
- **exFAT e FAT su macOS e Linux**: il README dice che lì i link diventano copie (per esempio i 142 link interni di Git) e consiglia APFS o ext4 per una chiavetta usata solo con un Mac o con Linux.
- **Pool automatici** (D-101): la decisione ora dice che i siti senza server scelto entrano già nel pool automatico delle loro linee.

## Pannello rapido dei progetti (19/10/2026)

- **Pannello rapido** (D-130): la prima scheda di Progetti mostra cosa è in esecuzione (con indirizzo, output e «Ferma»), i progetti preferiti e recenti con Avvia, Editor, Terminale e agente, cosa c'è da sistemare (avvii falliti, componenti mancanti) e i modi di creare un progetto.
- **Preferiti**: la stella su una scheda del pannello porta il progetto tra i preferiti; resta nelle impostazioni del progetto nel kit.
- **▶ nella lista dei progetti**: ogni riga avvia e ferma l'azione predefinita senza aprire la pagina.
- **Ctrl+P** apre un progetto per nome da qualsiasi pagina; Maiusc+Invio lo avvia.
## La cartella della home del sito come la vede il tuo sistema (19/10/2026)

- **La finestra della cartella** nella hero del sito prende la barra del sistema di chi visita: Windows 11 (titolo a sinistra, riduci, ingrandisci e chiudi a destra), macOS (i tre semafori, titolo al centro), Linux con GNOME (titolo al centro, chiusura tonda).
- **Dentro, i file veri**: `Atollo-<versione>-win32-x64.exe`, `Atollo.app` o `Atollo-<versione>-linux-<arch>.AppImage`, con la versione dell'ultimo rilascio, i separatori del sistema e una chiavetta come radice (`E:\Atollo\`, `/Volumes/KIT/Atollo/`, `/media/kit/Atollo/`). Senza JavaScript resta la cartella generica.
- **`?os=<target>`** nell'indirizzo mostra la pagina come la vede un altro sistema (per esempio `?os=linux-arm64`).

## Caratteri Geist e Geist Mono (19/10/2026)

- **App, sito ed editor** usano Geist per testi e titoli e Geist Mono per codice, percorsi, terminale ed editor, al posto di Segoe UI, Inter e Consolas: lo stesso aspetto su Windows, macOS e Linux.
- **Sempre in locale**: i caratteri viaggiano con l'app e con il sito, nessun servizio esterno (l'app resta portabile e senza rete).
- **Editor integrato**: Geist Mono nel testo e nel terminale, Geist nell'interfaccia. Chi ha già scelto un carattere nelle impostazioni dell'editor lo tiene.
- **Titoli del sito** un poco più leggeri e stretti, perché Geist è più largo di Segoe UI.

## Nuovo progetto dentro Progetti (19/10/2026)

- **Il wizard è una scheda di Progetti** (D-83): «Nuovo progetto» non è più una voce della barra delle attività, si apre nell'editor accanto ai progetti e tiene scelte e avanzamento quando passi a un'altra scheda.
- **Un albero per i modi di creare un progetto**: nella barra laterale, il riquadro «Nuovo progetto» elenca «Progetto vuoto», «Da un indirizzo Git» e «Da framework o CMS» con le sue basi (Laravel, Vite, Electron…); ogni voce apre la sua procedura, una base apre il wizard già su quella base.
- **Inizia** e la barra del titolo di Progetti portano alle stesse procedure.

## La versione nel nome del file (19/10/2026)

- **I file dei rilasci dicono la versione** (D-129): `Atollo-0.2.0-win32-x64.exe`, `Atollo-0.2.0-darwin-arm64.app.zip`, `Atollo-0.2.0-linux-x64.AppImage` e così per gli altri sistemi; anche Windows usa il target intero (`win32-x64`, prima `Atollo-x64.exe`).
- **Il nome segue gli aggiornamenti**: quando l'app si aggiorna da sola, l'exe e l'AppImage prendono il nome della versione nuova; la copia vecchia si cancella all'avvio dopo. Un file rinominato a mano tiene il suo nome.
- **Sito, README, note del rilascio e prove della CI** usano i nomi nuovi; l'aiuto della riga di comando mostra il nome vero del file anche su Windows.

## Il simbolo è un atollo (19/10/2026)

- **Icona dell'app, favicon e simbolo** mostrano un atollo visto dall'alto al posto della «A»: barriera ad anello, tre isolotti (smeraldo, smeraldo che passa ad ambra e arancio, un isolotto arancio), laguna turchese e due onde stilizzate. Restano il fondo verde foresta, i colori e la scritta «ATOLLO».
- **Taglie piccole** (fino a 48 px: barra delle applicazioni, favicon, icone piccole di `icon.ico`) con un disegno apposta, `symbol-small.svg`, che resta leggibile.
- **Sfondi** della Home, delle pagine vuote e del sito: l'atollo e le sue onde al posto delle bande diagonali della «A».
- **Tutte le icone** di `tauri/icons/` (anche `icon.icns` e i PNG per macOS e Linux) e `ui/public/favicon.png` escono da `docs/brand/make.py`; prima alcune erano rimaste fuori e avevano un disegno vecchio.

## Strumenti descritti come li installa il codice (19/10/2026)

- **Windows ARM64**: README, sito e catalogo elencano tutti i componenti che lì usano la build x64 in emulazione (editor, PHP, Apache, MySQL, MongoDB, Android SDK, Julia, TinyTeX); prima mancavano MongoDB, Android SDK e Julia. L'etichetta «x64» della pagina Strumenti dice «Su Windows ARM64 gira in emulazione x64», non «sui PC ARM64»: su un Mac o un Linux ARM64 quei componenti sono nativi. L'etichetta compare solo nel kit Windows ARM64 (prima su ogni sistema), e l'editor non si dice più «in emulazione x64» su macOS e Linux ARM64, dove è nativo.
- **SQLite su Windows ARM64** prende sempre lo zip `win-arm64` di sqlite.org: se mancasse, l'installazione lo dice invece di prendere in silenzio quello x64 (D-108).
- **R** non c'è solo su Windows ARM64: su macOS e Linux, ARM64 compreso, viene da conda-forge (il catalogo diceva «non disponibile sui kit ARM64»). **Apache** oggi il catalogo lo offre solo su Windows: su macOS e Linux arriva quando la build del kit è fissata (D-116); il sito diceva «Caddy e Apache per i siti» senza distinguere.
- **Git** non arriva più con GitHub CLI (D-81): arriva con Claude Code, un clone o un progetto. README e catalogo dicevano ancora «Git per GitHub CLI». La sua descrizione dice che Git Bash c'è solo su Windows.
- **Rust su Windows ARM64** usa l'host `gnullvm` collegato da C/C++ (D-112), non sempre MSVC come dicevano guida e catalogo; preparato da un altro sistema si completa al primo avvio, non dà errore.
- **Guida nell'app**: la tabella delle fonti conta le sue righe (35 componenti e 5 parti, prima «36»), dice per Git, PHP, Apache, C/C++, Ruby, R, MongoDB, SQLite, Rust, TinyTeX, Antigravity e l'editor da dove li prendono macOS e Linux, e segna i componenti non disponibili sul sistema. La presentazione nomina macOS e Linux accanto a Windows.
- **Matrice degli strumenti** ([TOOLS-MATRIX.html](TOOLS-MATRIX.html)): note, dimensioni e componenti tolti vengono dal catalogo e dal codice; R su Windows non è più segnato «conda» (segue il kit, D-104); la pagina non cambia se il catalogo non cambia (niente data né commit). Le fonti stanno in `scripts/matrix/sources.json`, che `cargo test` confronta con gli installatori.
- **Altro**: ordine dei componenti, pnpm (corepack e yarn), SQLite x64 su Windows ARM64 quando manca lo zip arm64, phpMyAdmin al posto di Adminer nel testo di presentazione.

**Verificato**: `cargo test`, fmt e clippy, ESLint, `tsc -b` e i test dell'interfaccia (uno nuovo per la tabella delle fonti); `gen_matrix.py` rigenera la matrice senza differenze; build del sito.

## Sito di Atollo (19/10/2026)

- **Pagina di presentazione** su GitHub Pages, `https://nabre.github.io/atollo/` (D-126): in italiano, inglese, francese e tedesco come i README, con i testi di [MARKETING.md](../brand/MARKETING.md), i colori dell'app e tre schermate della guida. Il pulsante «Scarica» riconosce il sistema del visitatore (Windows, macOS, Linux, x64 o ARM64) e punta al file giusto dell'ultimo rilascio; l'elenco dei sei sistemi ha il link a ogni file. Prima del primo rilascio, o senza JavaScript, ogni link apre la pagina dei rilasci.
- **«Come funziona»** al posto del confronto con altri prodotti: quattro aspetti di Atollo (isolato con i programmi nativi, tutto da una finestra, la cartella che viaggia, ogni componente verificato). In alto solo la scritta «ATOLLO», senza il simbolo accanto. [MARKETING.md](../brand/MARKETING.md) ora vieta i confronti e fissa la regola del logo.
- **«Strumenti per sistema»**: tutti i componenti del catalogo divisi per genere (sistema ed editor, agenti AI, linguaggi, strumenti di sviluppo, server web, database), con una colonna per ognuno dei sei sistemi: nativo, x64 in emulazione o non disponibile, con il motivo passando sopra la casella. Si genera a ogni pubblicazione da `tauri/catalog/components.json` (stessa logica di `is_available_on`), con descrizioni e motivi tradotti da `tauri/i18n/catalog.json`.
- **Landing rifatta**: hero a tutta pagina con `hero-dark.svg` / `hero-light.svg` del brand (secondo il tema) e un ingresso scaglionato, la cartella del kit com'è davvero accanto al titolo con il sistema del visitatore evidenziato, una fascia di numeri presi dal catalogo, funzioni in una griglia, schermate in cornice, una chiusura con l'hero specchiato e il download. Le animazioni rispettano «riduci movimento» e senza JavaScript la pagina è tutta visibile. Le schermate perdono la barra del titolo e quella di stato, che hanno ancora il vecchio nome, finché non si rifanno.
- **Sorgente in `site/`**, compilata da `node site/build.mjs` in `_site/` (ignorata da Git) e pubblicata dal workflow `site.yml` a ogni push su `main` che tocca `site/`, `docs/brand/` o `docs/app/images/`.

**Verificato**: build locale (quattro pagine, nessun segnaposto rimasto, chiavi uguali nelle quattro lingue), pagina aperta in Chrome in tema chiaro con Windows x64 riconosciuto e il messaggio «nessun rilascio pubblicato», workflow letto con PyYAML. Non provati: il deploy vero su Pages e il tema scuro a schermo.

## Atollo anche negli identificatori (19/10/2026)

- **Un solo nome, anche dentro** (D-127): le variabili d'ambiente del kit sono `ATOLLO_*` (`ATOLLO_TEST_KIT`, `ATOLLO_LANG`, `ATOLLO_ALLOW_LAN`, `ATOLLO_FCGI_KEY`, `ATOLLO_DB_*`…), i file di lavoro `.atollo-version`, `.atollo-new`, `.atollo-move`, i lock `Atollo-*`; i profili del terminale dell'editor si chiamano «Atollo», «Atollo - Git Bash», «Atollo - Bash»; i blocchi del kit nei file degli agenti e la regola `rules/atollo.md` portano il nome Atollo; l'user agent dei download è `Atollo/<versione>` con il link a `github.com/nabre/atollo`, come ogni link dei documenti.
- **Nessuna compatibilità con i nomi di prima**: un kit fatto con una versione precedente riscrive accanto ai vecchi i blocchi di Codex, i profili del terminale e la regola degli agenti; le variabili vecchie impostate a mano non valgono più; **l'accesso ad Antigravity va rifatto una volta**, perché le credenziali salvate prima non si leggono più. Tolti gli id dei temi dell'editor del nome di prima.
- **Documenti**: decisioni, novità e rapporti usano solo Atollo; i link dei rapporti a una cartella locale ora sono relativi al repository.

**Verificato**: 745 test Rust (6 ignorati), 147 test dell'interfaccia; fmt, clippy, ESLint, Prettier e `tsc -b` puliti su Windows; workflow letti con PyYAML.

## Rilasci per macOS e Linux (19/10/2026)

- **Ogni rilascio porta macOS e Linux** (D-119, [#5](https://github.com/nabre/atollo/issues/5)): accanto agli exe di Windows, `Atollo-darwin-arm64.app.zip` (Apple Silicon) e `Atollo-darwin-x64.app.zip` (Intel), `Atollo-linux-x64` e `Atollo-linux-arm64` in AppImage e `tar.gz`, ognuno con il suo `.sha256`. Sono i nomi che l'aggiornamento dell'app già cerca: un kit con il `.app` o l'AppImage si aggiornerà da solo dal prossimo rilascio.
- **Prova prima di pubblicare**: un job nuovo scarica i file che il rilascio pubblicherà e li avvia di nuovo come farebbe un utente, su macOS dopo aver tolto la quarantena con `xattr`, su Linux l'AppImage senza FUSE e il `tar.gz` con il solo WebKitGTK del sistema. Se uno non parte, il rilascio non esce.
- **README e note del rilascio** spiegano come aprire l'app su macOS (`xattr -dr com.apple.quarantine`, l'app aperta dalla cartella del kit, non da Download, per evitare App Translocation) e su Linux (`chmod +x`, `--appimage-extract-and-run` senza FUSE, oppure il `tar.gz` con WebKitGTK 4.1).

**Verificato**: workflow letto con PyYAML e `actionlint` 1.7.7 (nessun errore nuovo), script dei passi controllati con `bash -n` e il parser di PowerShell, il passo dei file del rilascio provato su Windows con file finti (con e senza `linux-arm64`). Non provati: la CI vera e le prove su macchine pulite (Mac, Ubuntu 24.04, Fedora 42, Debian 13, aggiornamento da una versione all'altra).

## Accessi degli agenti su macOS e Linux (19/10/2026)

- **Claude Code collegato con un token** («Collega con un token», macOS) è collegato in ogni programma del kit: nei terminali e nell'editor, non più solo avviato dalla pagina Agenti. Dopo «Esci» i terminali aperti da lì in poi non hanno più il token (D-115, [#6](https://github.com/nabre/atollo/issues/6)).
- **`Atollo env` non mostra più i segreti**: token, chiavi e password (`CLAUDE_CODE_OAUTH_TOKEN`, `GEMINI_API_KEY`, ogni `*_TOKEN`…) e la password di un proxy compaiono come `***`.
- **Antigravity su Linux** scrive l'accesso nella collezione `login` del Secret Service, l'unica che `agy` legge: ora funziona anche dove la collezione predefinita è un'altra (alcune configurazioni di KWallet). Dove `login` non c'è (KeePassXC) l'accesso non si sposta e un avviso lo dice, invece di finire dove `agy` non lo trova.

**Verificato**: su Windows 741 test Rust, fmt e clippy puliti; in WSL2 (Ubuntu 24.04) 733 test, fmt e clippy puliti, e la prova reale `real_secret_service_login_collection` con gnome-keyring: con `login` la voce va lì e si rilegge, senza `login` non si scrive nulla. Da provare su un Mac vero.

## Caddy al posto di Nginx (19/10/2026)

- **Nginx non c'è più** (D-120, [#4](https://github.com/nabre/atollo/issues/4)): Caddy è il server web del kit su ogni sistema, Apache resta per i progetti con regole proprie nel `.htaccess`. All'avvio i profili e i siti di Nginx diventano di Caddy, nella pagina Server web e nel `panel.json` dei progetti; prima di cambiarli il kit ne fa una copia in `data/app/pre-caddy/`. Un pool automatico di Nginx riparte con Caddy sulla stessa porta appena Caddy è installato. La cartella di Nginx in `data/apps` si cancella.
- **Caddy nell'interfaccia**: nella pagina Server web è il motore proposto per un profilo nuovo, accanto ad Apache; nella scheda Sito web del progetto è la prima scelta, con la nota «Caddy non legge .htaccess: le regole di riscrittura si generano dal tipo di progetto.».
- **PHP si aggiorna tutto o niente**: le parti di una versione (PHP, `php-fpm` su macOS e Linux, Composer) si scaricano e si verificano tutte prima di toccare la cartella installata. Prima un errore di rete a metà poteva lasciare un PHP senza `php-fpm` o senza Composer.
- **Rilevamento**: in un sito con moltissimi file la ricerca dei `.htaccess` delle sottocartelle si ferma a 5000 voci; ora lo dice con un avviso, perché un `.htaccess` di una cartella di caricamenti potrebbe sfuggire.

**Verificato**: su Windows 730 test Rust, fmt e clippy puliti; interfaccia: lint, tsc e 139 test. Nuovi: il passaggio di pool, siti e porte da Nginx a Caddy (ripetibile, con la copia `pre-caddy`), un download di PHP che fallisce a metà lascia la versione installata com'era, l'avviso sulla cartella troppo grande. Non eseguite le prove reali dei server web (`ATOLLO_TEST_KIT`).

## Griglia dei sistemi in «Strumenti» (19/10/2026)

- **«Matrice»**, nella Panoramica di «Strumenti» (D-121, [#7](https://github.com/nabre/atollo/issues/7)): una riga per componente e una colonna per ognuno dei sei sistemi (Windows x64 e ARM64, macOS Apple Silicon e Intel, Linux x64 e ARM64). Ogni casella dice se il componente lì è disponibile, in emulazione, pre-scaricato, se si completa al primo avvio su quel sistema o se non è disponibile, con il motivo; un punto verde se è installato. Un clic apre il dettaglio, con la versione e «Mostra questo sistema».
- **«Piattaforma»** ha i nomi dei sistemi al posto degli id, con «questo PC», «in emulazione» o «non eseguibile qui», e il sistema scelto resta al prossimo avvio (`settings.json`). Il sistema di questo PC non si salva: lo stesso kit aperto su un altro PC mostra il suo.
- **Relazioni ed estensioni dell'editor** seguono il sistema scelto: niente collegamenti né colonne per i componenti che quel sistema non ha.
- «non disponibile sui PC ARM64» solo su un PC Windows ARM64: da un altro PC un kit Windows ARM64 dice «non disponibile per win32-arm64».
- `Atollo components` segna come scelto Zig accanto a C e C++ su macOS e Linux, come la pagina.
- Su un Mac la pagina non lancia più `xcode-select` a ogni apertura: la risposta resta valida (gli strumenti installati intanto si vedono entro un minuto).
- Zig cancellato a mano su macOS o Linux toglie dalla scelta anche C e C++, su ogni sistema del kit: ora è una decisione scritta (D-122).

**Verificato**: su Windows 743 test Rust e 141 test dell'interfaccia, lint, clippy e formattazione puliti; clippy incrociato per macOS Apple Silicon e Linux x64 pulito. La griglia vista da un Mac o da un PC Linux veri non è provata.

## Arretrato di macOS e Linux (19/10/2026)

- **Linux distingue le maiuscole nei percorsi** (D-123): un aggiornamento dell'editor o la rimozione dell'unità fermano solo i programmi della cartella giusta, non quelli di una cartella che differisce solo nelle maiuscole; lo stesso per le voci del PATH tolte da un profilo del terminale. Windows e macOS restano senza distinzione, come i loro file system.
- **Salvataggi contemporanei su Windows**: due salvataggi dello stesso file nello stesso momento (per esempio le impostazioni da due finestre) potevano fallire con «accesso negato», perché Windows rifiuta una delle due sostituzioni; ora quella rifiutata riprova per al massimo un secondo e vince l'ultima, come su macOS e Linux. Era la causa del test instabile `write_atomic_concurrent_threads_do_not_collide_on_tmp`.
- **Il nome è Atollo ovunque** (D-124): `version`, l'aiuto, i titoli della finestra e dell'editor, la pagina degli aggiornamenti, «Avvia con Atollo», i messaggi d'errore e la descrizione dell'editor dicono «Atollo».
- **Ruby, R e MongoDB su macOS e Linux mettono nel PATH solo i loro programmi** (D-125): prima arrivava tutta la cartella `bin` del loro ambiente conda, con i programmi delle dipendenze (`openssl`, `tput`, `kinit`, `envsubst`...) che nei terminali del kit prendevano il posto di quelli del sistema. Ora nel PATH ci sono `ruby`, `gem`, `bundle`, `rake` e gli altri di Ruby, `R` e `Rscript`, `mongod` e `mongos`, attraverso dei lanciatori in `kit-bin`; i componenti già installati li ricevono al prossimo avvio.
- **MongoDB Database Tools su Debian 9**: si sceglie la build `debian92`, come per RHEL a parità di versione maggiore; prima Debian 9 non la trovava e riceveva quella di Ubuntu 20.04, che lì non parte.

**Verificato**: su Windows 742 test Rust (6 ignorati) e 133 test dell'interfaccia, fmt, clippy, ESLint, Prettier e `tsc` puliti; in WSL2 (Ubuntu 24.04) 746 test (5 ignorati, contro i 16 di prima) e clippy pulito; i lanciatori di Ruby, R e MongoDB provati su ambienti conda-forge veri in una cartella con uno spazio.

## Spostare il kit senza rete su macOS e Linux (18/10/2026)

- **R, Ruby, MongoDB (Linux), SQLite (Linux ARM64) e il `make` di C e C++** si reinstallano dopo che il kit cambia cartella, perché vengono da conda-forge e hanno il percorso scritto dentro (D-92). Ora i loro pacchetti restano nel kit, in `data/app/conda/<sistema>/`, e la reinstallazione li usa senza scaricare nulla, alla stessa versione: in WSL 43 s per R, Ruby, MongoDB e make, contro 86 s dalla rete con una connessione veloce, e funziona anche senza rete (D-118).
- Ogni pacchetto si ricontrolla con il suo SHA-256 prima dell'uso: se qualcosa manca o è diverso, il kit li riscarica come prima e lo dice nel pannello Attività.
- **Impostazioni → Generale**: «Pacchetti per reinstallare senza rete» (attivo di serie) con lo spazio che occupano, circa 380 MB per R, Ruby e MongoDB, e «Libera». I pacchetti che nessuna versione installata usa più si cancellano da soli.
- I componenti installati prima di questa versione hanno la copia dalla loro prossima installazione o aggiornamento.
- **R su macOS e Linux parte anche da una cartella con uno spazio** (per esempio `/Volumes/My Kit`): lo script `R` di conda-forge si rompeva sul percorso del suo `sed`; ora usa quello del sistema. Vale dalla prossima installazione o reinstallazione di R.

**Verificato**: su Windows 759 test Rust e i test dell'interfaccia, lint e clippy puliti, clippy incrociato per macOS Apple Silicon pulito; in WSL2 (Ubuntu 24.04) 751 test e la prova reale `real_offline_reinstall_after_a_move` senza rete (`unshare -rn`): R 4.6.1, Ruby 4.0.7, MongoDB 8.0.23 e make 4.4.1 ricreati in una cartella nuova e di nuovo in quella di prima, gem e dati di MongoDB intatti. Non provati dal vivo: macOS e SQLite su Linux ARM64.

## Apache su macOS e Linux, alias più sicuri (18/10/2026)

- **Apache anche su macOS e Linux** (D-116), da una build del kit: httpd 2.4.69 con OpenSSL 3.5, compilato in CI dai sorgenti con firma verificata (`scripts/build-httpd`), una cartella che si sposta con il kit. PHP passa da `php-fpm`, che parte con il pool o con il sito del progetto, come per Caddy. Come su Windows, Apache si propone solo per i progetti con regole proprie nel `.htaccess` (Drupal compreso); Caddy resta il server predefinito. Su Windows niente cambia: Apache Lounge con il modulo PHP, x64 anche su ARM64. Apache compare su macOS e Linux con il primo rilascio della build (`httpd-v2.4.69-1`).
- **Alias solo per file statici** (SEC-V4-01), con Apache, Nginx e Caddy: uno script finito in una cartella di alias (caricamenti, file condivisi) non viene più eseguito, e un `.htaccess` caricato lì non può dargli un gestore né riaprirlo. Un alias protetto da password con `.htaccess` non è più possibile.
- `php_value` e `php_flag` in un `.htaccess` funzionano solo con il modulo di Windows: con `php-fpm` danno errore; al loro posto `.user.ini`.

**Verificato**: 725 test Rust su Windows e 717 in WSL; prove reali su Windows (Apache Lounge, PHP 8.5.11) e in WSL Ubuntu 24.04 (build del kit, `php-fpm` 8.5.8): due siti, un alias, file nascosti e host estranei rifiutati, script e `.htaccess` ostili in un alias neutralizzati, cartella dei file di Drupal senza PHP, ricarica; Caddy reale in WSL; la build Linux x64 provata in `ubuntu:22.04` (glibc richiesta 2.34). macOS e Linux ARM64: solo nella CI della build.

## Solo MySQL (18/10/2026)

MySQL è l'unico server SQL di Atollo, su ogni sistema (D-117, [#3](https://github.com/nabre/atollo/issues/3)): tutto ciò che chiede un database SQL usa MySQL.

- **phpMyAdmin** apre MySQL e c'è sui sei sistemi, macOS e Linux ARM64 compresi.
- **Nuovo progetto**: WordPress, TYPO3, Drupal e l'aggiunta «MySQL» di Laravel creano il database su MySQL.
- **Rilevamento**: le immagini Docker di un server SQL, i driver MySQL e i CMS chiedono MySQL; un `.env` di Laravel con un driver che MySQL non regge del tutto riceve un avviso (meglio `mysql`).
- Su Linux MySQL vuole `libaio1t64` e `libnuma1` (Ubuntu 24.04): il README lo dice.

**Verificato**: su Windows 737 test Rust e 134 test dell'interfaccia, lint e clippy puliti, clippy incrociato per macOS Apple Silicon pulito; in WSL2 (Ubuntu 24.04) 729 test superati.

## macOS e Linux (18/10/2026)

L'app gira su macOS (Apple Silicon e Intel), su Linux (x64 e ARM64) e, con le prove, anche su Windows ARM64. Lo stesso kit su una chiavetta si apre sui tre sistemi: `projects/` e `data/` sono condivisi, i programmi stanno in `data/apps/<sistema>-<architettura>/`. Provato dall'utente su un Mac Apple Silicon, un Mac Intel e un PC Linux, in WSL2 e nella CI sui sei sistemi.

**L'app**
- **Pacchetti**: `Atollo.app` per macOS (firma ad hoc, D-113), AppImage e `tar.gz` per Linux, costruiti con glibc 2.35 come minimo. `projects/` e `data/` stanno accanto a `.app` o all'AppImage, mai dentro. Un `.app` messo in quarantena da macOS (App Translocation) ferma l'avvio con il comando da eseguire.
- **Terminale dentro l'app** su macOS e Linux (D-109): il terminale del kit, gli agenti e i profili del terminale dell'editor si aprono in una scheda della finestra, con PowerShell del kit o Bash. Su Windows resta la console classica.
- **Processi**: un guardiano chiude i programmi del kit anche se l'app viene uccisa; seconda apertura che porta in primo piano la prima; un disco montato `noexec` ferma l'avvio con il comando per rimontarlo; proxy e lingua letti dal sistema (D-105).
- **Aggiornamento**: su macOS il nuovo `.app`, su Linux la nuova AppImage, scambiati alla chiusura. Il `tar.gz` dice che c'è una versione nuova da scaricare.
- **Preparare per un altro sistema**: `prepare --target <id>` e `components --target <id>` da riga di comando, anche nell'aiuto (D-114). Un kit preparato su Windows completa al primo avvio sul Mac o su Linux ciò che serve.

**Componenti** (D-108, D-112)
- Il catalogo dice per ogni sistema cosa c'è e perché manca quello che manca (Android su Linux ARM64), senza ripiegare su un'architettura che non parte.
- **Git** su macOS e Linux da dugite-native: segue il kit quando si sposta, senza reinstallare.
- **PHP** 8.4 e 8.5 su macOS e Linux da static-php-cli, con Composer e phpMyAdmin; un `php.ini` per sistema.
- **C e C++** su macOS e Linux con `zig cc` del kit e GNU make, senza `build-essential` né Xcode (i framework Apple vogliono le Command Line Tools); scegliendo C e C++ arriva Zig. **Rust** su Windows ARM64 senza Visual Studio (`gnullvm`, con il C/C++ del kit).
- **Ruby** su macOS e Linux da conda-forge (linee 3.2 e successive), con le gem per linea; **MongoDB** su Linux da conda-forge; **SQLite** anche su Linux ARM64; **Antigravity** su macOS e Linux.
- **Caddy**, nuovo componente su tutti i sistemi.

**Server web e database**
- **Caddy** è il server web dei pool e dei siti dei progetti su ogni sistema (D-111), con php-cgi su Windows e php-fpm su macOS e Linux. Apache resta su Windows per i progetti con regole proprie nel `.htaccess` (Drupal compreso).
- **MySQL** 8.4 e 9.7 LTS sui sei sistemi (D-110), con le password dei progetti e le copie.

**Accessi** (D-115)
- **Antigravity** su macOS (Portachiavi) e Linux (Secret Service): l'accesso resta nel kit, come su Windows.
- **Claude Code su macOS**: «Collega con un token» nella pagina Agenti; il token resta nel kit e «Esci» lo cancella.

**Sicurezza** (revisione dei commit)
- L'admin di Caddy non è più su una porta TCP locale, da cui chiunque sul PC poteva leggere la chiave di PHP e caricare una configurazione: socket del solo utente in una cartella privata, `admin off` per i siti dei progetti, chiave di PHP solo nell'ambiente di Caddy.
- Ripristino di una copia SQL con i comandi del client spenti (`\!`, `system`).
- MySQL: `root` ha la password dalla prima inizializzazione; il socket non sta mai in `/tmp`.
- Accesso di Antigravity: se il Portachiavi o il Secret Service non rispondono non si sposta nulla; una copia in chiaro solo su Linux senza Secret Service e solo su un disco che tiene i permessi 600.
- Token di Claude Code rifiutato su un disco senza permessi (exFAT, FAT).

**Resta da fare**: i bisogni SQL verso MySQL ([#3](https://github.com/nabre/atollo/issues/3)); interfaccia di Caddy, Nginx da togliere, PHP su Unix ([#4](https://github.com/nabre/atollo/issues/4)); rilasci per macOS e Linux ([#5](https://github.com/nabre/atollo/issues/5)); accessi da provare su un Mac ([#6](https://github.com/nabre/atollo/issues/6)); griglia componente × sistema in «Strumenti» ([#7](https://github.com/nabre/atollo/issues/7)); arretrato ([#8](https://github.com/nabre/atollo/issues/8)).

**Verificato**: su Windows 716 test Rust e 131 test dell'interfaccia, lint pulito, clippy pulito anche in modo incrociato sui quattro target Unix; in WSL2 708 test superati; CI verde su Windows x64 e ARM64, Linux x64 e ARM64, macOS Apple Silicon e Intel, con `.app` e AppImage costruiti e avviati (`ae42f65`). Installazioni reali in WSL: Git, Python, Ruby, MongoDB, SQLite, PHP, MySQL, Caddy con un pool di due siti.

## phpMyAdmin al posto di Adminer (17/10/2026)

- **«Apri phpMyAdmin»** nella pagina di MySQL sostituisce «Apri Adminer» (D-107): phpMyAdmin 5.2.3, già entrato come `root`, nella lingua dell'app. Al primo clic installa phpMyAdmin (circa 60 MB) e, se mancano, PHP e MySQL; avvia MySQL se è fermo. Funziona anche sulle altre versioni di MySQL («phpMyAdmin su 8.4»).
- Stessa protezione di Adminer: token monouso nell'indirizzo, che viene subito tolto, cookie `HttpOnly`, password di `root` solo nell'ambiente del server. Il router serve solo le pagine e i file pubblici di phpMyAdmin.
- Importazione di file SQL fino a 512 MB.
- **Adminer viene tolto**: all'avvio l'app cancella la sua cartella, il router, le sessioni e il log.

**Correzione**
- **Browser che non si apriva**: «Apri Adminer» avviava il server di PHP, ma la pagina non si apriva mai. Su Windows 11 `explorer.exe` ignora un indirizzo con `?…`. Ora gli indirizzi web passano da `rundll32 url.dll,FileProtocolHandler`, come già faceva l'editor; vale anche per gli indirizzi dei progetti, per GitHub e per i download a mano.

**Verificato**: `cargo fmt --check`, clippy con `-D warnings`, 455 test Rust, ESLint, Prettier, `tsc`, 117 test dell'interfaccia; installazione reale e test reale sul server SQL del kit con PHP 8.5.

## Riordino di cartelle e file (17/10/2026)

Struttura riallineata alle regole di [ARCHITECTURE.it.md](ARCHITECTURE.it.md), senza cambiare il comportamento (salvo la correzione qui sotto).

**Correzione**
- **Spostamento del kit su macOS e Linux**: `kit.rs` salvava la radice vecchia in `old-root-darwin.txt` / `old-root-linux.txt`, ma `agents::relocate` leggeva solo `old-root.txt`: fuori da Windows i percorsi degli agenti non venivano riscritti dopo uno spostamento e il file restava. Ora tutti e due usano `Kit::old_root_file`.

**Motore Rust**
- `servers\mod.rs` (1911 righe) diviso in `mod.rs` (tipi ed elenco), `pools.rs` (servers.json, pool automatici, scansione), `config.rs` (configurazione generata), `process.rs` (avvio, arresto, ricarica, siti collegati); codice copiato invariato.
- `terminals.rs` passa da `src\` a `editor\terminals.rs`, accanto a `terminal.rs`: in cima a `src\` resta solo l'infrastruttura.
- Percorsi in un solo posto: `Kit::versions_root()` (`data\app\versions`, prima scritto a mano in tre moduli), `ext::user_path` (`extensions.txt`, due volte), `auth::secrets_path` usato anche dalla migrazione.
- Stato del kit letto con `fsx::read_text` / `fsx::read_json` (BOM e byte non UTF-8 tollerati): `components.txt`, file delle versioni, `layout.txt`, `root.txt`, la radice vecchia, `extensions.txt`, `secrets.json`.

**Interfaccia**
- Nuove cartelle `databases\` (`page`, `browser`, `backups`), `agents\` (`page`, `stats`), `theme\` (i tre CSS; `docs\brand\themes.py` scrive lì); `projectmcp.tsx` ed `editorsettings.tsx` passano in `projects\` (`mcp.tsx`, `editorsettings.tsx`). I test in `test\ui` seguono la stessa struttura. Il CSS compilato è identico.

**Repository**
- `Prompt.md` (il prompt iniziale) → `docs\app\`, `brand.png` → `docs\brand\`, `reports\` → `docs\reports\`: nella radice restano configurazione e README.

**Verificato**: `cargo fmt --check`, clippy con `-D warnings`, 452 test Rust (uno nuovo), ESLint, Prettier, `tsc`, 117 test dell'interfaccia, build dell'interfaccia.

## Correzioni e sviluppi della v3 (17/10/2026)

Bug, falle e lavori aperti raccolti da [`reports/v3/03`](../reports/v3/03-BUG-SICUREZZA-E-FUNZIONALITA-MANCANTI.md), i lavori aperti dei server web e la verifica della loro spec, chiusi da uno sciame di sette agenti (sicurezza web, pool, installazioni, database, rilevamento, interfaccia, multi-OS) più un giro di revisione con 17 rilievi, tutti corretti. Stato voce per voce in [reports/v3/03 §4](../reports/v3/03-BUG-SICUREZZA-E-FUNZIONALITA-MANCANTI.md).

**Sicurezza**
- **Estensioni PHP** (D-96, SEC-V3-01): con PHP, Apache e Nginx eseguono `.php`, `.phtml` e `.phar`; senza PHP li rifiutano (Apache 403, Nginx 503). `.inc` e `.php3`…`.php8` sono sempre rifiutati (403), nei server dedicati, nei pool e nel sito del kit. Un `kit.conf` esistente riceve i blocchi che mancano all'avvio, una volta sola.
- **File nascosti nel sito del kit** (SEC-V3-02): `.env`, `.git` e simili rifiutati anche in `data\apache\kit.conf` (`.well-known` resta servito).
- **`php -S` attraverso un router del kit** (D-97, SEC-V3-02): `data\kit\php-router.php` risponde 403 a file nascosti, `.inc`, `.php3`…`.php8`, `.phtml`, `.phar`, nomi come `index.php.` o `index.PHP`, nomi brevi 8.3 (`ENV~1`), `//.env`, `::$DATA` e `\` usato come separatore, e 421 a un `Host` che non è locale. Il percorso si legge da `REQUEST_URI` senza `parse_url`. Un `router.php` del progetto prende il suo posto. Su macOS e Linux la cartella del sito passa a `/bin/sh` tra apici singoli: `$(…)` nel nome di una cartella resta testo.
- **DNS rebinding** (D-98, SEC-V3-03): fuori dalla modalità rete locale il server dedicato di un progetto risponde solo a `localhost`, `127.0.0.1`, `[::1]`, `*.localhost` e ai domini del progetto; un altro `Host` riceve 403 (Apache) o 421 (Nginx).
- **Alias senza collegamenti** (D-100, WEB-8): nelle cartelle degli alias Apache non segue collegamenti e un `.htaccess` non può riattivarli; Nginx scrive `disable_symlinks on` fuori da Windows.

**Server Web**
- **I pool seguono i progetti** (D-101): «Ferma» toglie il sito dal pool e lo ricarica; un pool avviato dai progetti si spegne con l'ultimo, salvo «Mantieni attivo». Nuove opzioni dei profili «Mantieni attivo anche senza progetti» e «Avvia con Atollo» (`keepAlive`, `autostart` in `servers.json`), nel modulo di creazione e nella sezione «Avvio e arresto».
- **Un nome per sito** (BUG-V3-02): nel pool il sito principale di un progetto è `<progetto>.localhost`, le altre azioni `<azione>.<progetto>.localhost`, ognuna con il suo file `vhosts\<nome>.conf`. Nomi di progetto con spazi o maiuscole danno ora una configurazione valida.
- **php-cgi sorvegliato anche nel Nginx dedicato** (D-99, BUG-V3-01): se php-cgi si chiude, Nginx si ferma e l'esecuzione va in errore con una riga nel log. Nei pool php-cgi si aspetta sulla sua porta e si riprova su un'altra fino a tre volte; senza php-cgi il pool non parte.
- Difetti dei pool (D-101): una linea non installata ferma l'avvio con un errore che la nomina (4); un pool acceso il cui id è cambiato resta in elenco per poterlo fermare (5, parziale); una porta per pool, anche nel ripiego e in un `servers.json` vecchio (6); un progetto entra nei pool una volta per sito (7); assegnare un progetto ricarica solo i suoi pool (8).

**Database**
- **Più versioni di MySQL insieme** (D-102): la versione che il server usa tiene la sua porta (3306), le altre ne ricevono una propria, salvata al primo avvio. La pagina Database ha «Altre versioni» (stato, porta, avvia/ferma); la pagina del progetto «Versione del server», che crea database e utente sulla versione scelta. I dati non si spostano da soli.
- **`DB_PORT`** nell'ambiente dei programmi del progetto, se il progetto ha un database MySQL del kit e il suo `.env` non indica un'altra porta.
- Le scritture di `db.json` di un progetto sono serializzate; la versione principale si cambia solo con nessuna versione accesa; un cambio di versione fallito lascia quella di prima.

**Rilevamento dei progetti** (D-103)
- CMS da archivio: PrestaShop, Drupal, Joomla e TYPO3 classico, con Apache e MySQL.
- Laravel e Statamic con `.htaccess` nella cartella pubblica ricevono l'azione Apache; PHP semplice propone Apache accanto a `php -S`; `nginx.conf` o `.nginx` danno l'azione Nginx anche ai siti statici.
- Compose (`compose.yaml`, `docker-compose.yml`…) e `FROM` di `Dockerfile`/`Containerfile` letti per nome esatto dell'immagine, con la linea dal tag (`php:7.4-apache` → PHP 7.4 e Apache).
- Apache e Nginx contano fra i requisiti solo se sono l'azione predefinita o scritti in `panel.json`.

**Installazione e piattaforme** (D-104, D-105)
- **C/C++ su macOS e Linux** usa l'ultimo rilascio di LLVM con l'archivio del sistema: macOS Intel resta su LLVM 20.1.7. La descrizione dice cosa serve sul PC (Xcode CLT; header della libc e `make`) (BUG-V3-05).
- **Editor** con SHA-256 fissato per ogni sistema (BUG-V3-09).
- **R su Windows**: `Makeconf` segue la cartella del kit a ogni avvio, così i pacchetti da sorgente si compilano dopo un cambio di lettera d'unità (BUG-V3-06, parte Windows).
- **Il codice dell'app compila per macOS e Linux**: dipendenze Windows solo su Windows, icona PNG, strato `os` (istanza, temp, dischi, processi, job, sessioni) per Unix, programmi di shell e dei server web per sistema, altri kit aperti riconosciuti ovunque (B7, N-A1, BUG-V3-03/07/08). Nuovo job di CI `check-unix` su Ubuntu e macOS. Accesso di Antigravity, scelta di file e cartelle e console fuori dall'editor danno un errore chiaro fuori da Windows.

**Interfaccia** (D-106)
- Un'unica trappola del focus per i dialoghi (conferma e ingrandimento delle schermate della Guida).
- Le quattro conferme sì/no nella pagina (uscita da un agente e da GitHub, ripristino di un database, «Applica» in Linguaggi e strumenti) sono `alertdialog` con il focus su «Annulla»; alla chiusura il focus torna al pulsante che le ha aperte.
- Versioni del progetto: uno strumento mancante resta nella zona del progetto con il rimando a «Installa ciò che manca»; il testo nomina anche il server web.

**Verificato**: 426 test Rust (3 ignorati), 107 test dell'interfaccia; fmt, clippy, ESLint, Prettier e build puliti su Windows. Il codice Unix è solo compilato (controllo incrociato per macOS e Linux). Da provare a mano: due versioni di MySQL insieme, pool che si spengono e «Avvia con Atollo», `httpd -t`/`nginx -t` delle configurazioni nuove con `Host` estraneo, prove con `ATOLLO_TEST_KIT`, primo avvio reale su macOS e Linux ([TESTING.it.md](TESTING.it.md)).

## Niente Windows Terminal (16/10/2026)

- **Console solo nella console classica di Windows** (D-95): Terminale, agenti e terminali del kit si aprono in `conhost.exe` con PowerShell 7 del kit, come succedeva già quando Windows Terminal mancava. Tolti il componente Windows Terminal, l'impostazione «Terminale» e il gruppo «Legati a un sistema operativo» della pagina «Linguaggi e strumenti».
- **Kit esistenti**: all'avvio l'app toglie la cartella di Windows Terminal (impostazioni comprese), i suoi file di versione e `data\kit\console.ps1`.
- Proposta del primo avvio: circa 1,8 GB (45 MB in meno).

## CMS nel wizard dei progetti (16/10/2026)

- **Nuove basi** nel wizard «Nuovo progetto»: WordPress (Bedrock), TYPO3, Drupal e Statamic, con database MySQL del progetto, installazione completa e amministratore pronto (utente e password nel `.env`, D-93).
- **Database chiesto dalla base**: una base può avere il suo server di database, come le aggiunte; il campo è anche nell'editor delle Impostazioni.
- I comandi della creazione ricevono la connessione al database e le variabili del `.env` nell'ambiente, senza password nelle righe mostrate; `{password}` nel `.env` diventa una password nuova.
- Un `.env` che Git prenderebbe si aggiunge al `.gitignore`.
- La voce «Inizia» della pagina Progetti apre anche il wizard.
- I progetti TYPO3, Drupal, Statamic e WordPress con Bedrock sono riconosciuti come tali.

## Terminali del kit (16/10/2026)

- **Pagina «Terminali»** (D-94): console del kit che si aprono in qualsiasi cartella del PC. «Tutti gli strumenti» ha ogni strumento installato alla versione più recente; i terminali propri includono solo gli strumenti scelti (agenti, linguaggi, database), ognuno alla versione più recente o a una salvata. Cartella di partenza salvata per terminale, «Apri in…» per sceglierne un'altra una volta, «Duplica» per partire dal predefinito.
- Gli strumenti esclusi escono dal PATH con le variabili che puntano nelle loro cartelle; PowerShell 7, GitHub CLI e i comandi del kit restano sempre.
- In una cartella di progetto il terminale tiene le sue versioni, non quelle del progetto (`ATOLLO_TERMINAL` in `profile.ps1` e `bashrc`).

## Correzioni dopo la verifica della v2 (16/10/2026)

27 difetti confermati rileggendo il codice ([`reports/v2/07`](../reports/v2/07-PIANO-RISOLUZIONE-BUG-SCIAME.md)), chiusi da uno sciame di agenti in quattro onde; CI di nuovo verde (`cargo fmt`, clippy, ESLint, Prettier) e passo dell'interfaccia in CI che si ferma al primo comando fallito.

**Sicurezza**
- **Pool dei server web**: un sito la cui cartella manca, sta fuori dal progetto o è una giunzione si salta (avviso nel log), invece di servire la radice del progetto con `.env`, `.git` e `wp-config.php` (N-S1). I pool Apache hanno un VirtualHost di default `localhost` prima dei progetti: un `Host` sconosciuto, o l'IP dalla LAN, non arriva più al primo progetto (N-S5).
- **`.php` mai serviti come testo**: Nginx senza FastCGI risponde 503 (N-S2); Apache (progetti, pool e sito del kit `data\apache\kit.conf`) rifiuta i `.php` con 403 se nessun modulo PHP è caricato, e riconosce `.php` in qualsiasi maiuscola (`INDEX.PHP`). Un `kit.conf` già esistente resta com'è.
- **Server statico**: rifiutati i nomi brevi 8.3 (`GIT~1`, `ENV~1`) che aggiravano il blocco dei file nascosti (N-S3); test sulle connessioni silenziose che non bloccano il server.
- **Archivi `.tar.xz`** estratti in Rust con lo stesso percorso sicuro dei `.tar.gz`: nomi controllati, link solo se sicuri, niente `tar` di sistema (BUG-22, N-A1).

**Server Web**
- `servers.json` non perde più profili: ogni lettura-modifica-scrittura è serializzata (N-S4a); «Salva» e «Rimuovi» non bloccano l'interfaccia (N-S4b); le porte dei pool spariti si tolgono (N-S4c).
- La lista dei pool rilegge i progetti al più ogni 15 s (o col pulsante «Aggiorna»), non a ogni aggiornamento di 3 s (BUG-07).
- L'output di httpd/nginx va in `data\servers\<pool>\stderr.log` e un errore di sintassi della configurazione arriva all'utente (BUG-03). Il riavvio aspetta fino a 5 s che la porta si liberi, senza il falso «porta già in uso» (BUG-04).
- **php-cgi dei pool sorvegliato**: se si chiude, il pool risulta fermo (da riavviare) invece di «avviato» con un 502 permanente; parte solo se un sito del pool usa PHP (BUG-05r).
- **«Avvia» del progetto rispetta il server scelto**: un sito assegnato a un profilo manuale avvia quel pool e la sua esecuzione ne usa l'indirizzo; «Ferma» la segna solo ferma, perché il pool è condiviso e si ferma dalla pagina Server Web (GAP-RUN).

**Installazione e piattaforme**
- **Git su macOS e Linux via micromamba** (D-92): il pacchetto `git` di conda-forge, con le sue librerie, in un ambiente nella cartella della sua linea, come R (BUG-23a). Un ambiente conda ha il percorso scritto dentro: quando il kit cambia cartella, Git e R fuori da Windows si segnano da reinstallare e l'installazione automatica all'avvio li rifà.
- **PHP, Apache, Nginx e Adminer solo su Windows, per ora**: le fonti sono build Windows e `php.ini`/`httpd.conf` sono scritti per Windows; su macOS e Linux Nginx scaricava il sorgente (BUG-23b, BUG-24a). I progetti propongono solo componenti disponibili sul sistema del kit; [CATALOG.it.md](CATALOG.it.md) dice cosa c'è dove.
- Python conserva i pacchetti e trova `pip` usando la linea davvero installata, non quella delle impostazioni (BUG-24b).
- Un componente non disponibile dice perché: «non disponibile sui PC ARM64» solo su Windows ARM64, altrimenti il sistema (BUG-24c).
- Il file da scaricare a mano nomina la cartella giusta (nel temp del sistema) in tutte e quattro le lingue (BUG-17r).

**Interfaccia**
- **Database**: gli errori delle azioni restano visibili anche dopo l'aggiornamento della lista, e una lettura vecchia non riporta la lista di prima (N-U1).
- **«Riprova»** quando non si leggono lo stato del kit, le impostazioni dell'editor, le estensioni o i server MCP del progetto; il riquadro MCP mostra l'errore invece di sparire (BUG-18r, N2).
- **Tastiera**: menu delle versioni con frecce, Home/End, Esc e focus restituito; il dialogo di conferma tiene Tab tra i controlli raggiungibili e rende il focus alla chiusura (BUG-20r).
- I divisori salvano la misura a fine trascinamento, non a ogni pixel (BUG-11r). Badge «più recente»/«bloccata» tradotti, etichette dei giorni leggibili, Guida allineata ai nomi dell'app («Rimuovi l'unità») (UI-POLISH). La Guida si carica a parte, fuori dal pacchetto principale (ARCH-LAZY).

**Verificato**: `cargo check`, clippy e fmt per Windows; 85 test dell'interfaccia, ESLint, Prettier e build puliti. I test Rust girano solo nella CI Windows. Da provare a mano: Git via micromamba su macOS e Linux reali, avvio di un progetto in un pool, morte di php-cgi ([reports/v2/07 §8](../reports/v2/07-PIANO-RISOLUZIONE-BUG-SCIAME.md)).

## Una cartella per versione minore per tutti gli strumenti (16/10/2026)

Ogni strumento sta in una cartella col nome della sua versione minore, e una correzione (patch) prende il posto di quella cartella (D-91).

**Fatto**
- **Strumenti sempre all'ultima versione** (Claude Code, Codex, Antigravity, Git, GitHub CLI, uv, PowerShell 7, Windows Terminal, CMake/Ninja, Adminer, SQLite): `git\2.56`, `pwsh\7.6`, `claude\2.1`… Una patch sostituisce la cartella; una minore nuova si installa accanto e la vecchia si toglie (in uso: al prossimo avvio). PowerShell e Windows Terminal si aggiornano a una minore nuova anche aperti.
- **GitHub CLI e uv** escono da `tools\`: `gh\<linea>\bin\gh.exe`, `uv\<linea>\uv.exe`.
- **Windows Terminal** porta le sue impostazioni (`settings\`) in ogni aggiornamento.
- Linguaggi e database come prima: più minori affiancate, scelte per il kit o per progetto.
- Restano senza cartella di versione Rust, Android SDK, TinyTeX e pnpm (i pacchetti aggiunti stanno dentro), C/C++ (versioni a data) e l'editor.

**Cambia**
- **Apache** torna a una cartella per minore, `apache\2.4`: al primo avvio la cartella della versione installata (`apache\2.4.69`) prende quel nome e le altre build salvate si tolgono. La scelta di una patch in un progetto non vale più; l'elenco delle versioni si chiede di nuovo.
- Al primo avvio le cartelle esistenti si spostano da sole (`git\` → `git\2.56`, `tools\gh.exe` → `gh\2.102\bin\gh.exe`); una cartella in uso si sposta all'avvio dopo.

## Guida operativa e pagina «Guida» nell'app (16/10/2026)

Manuale completo del kit in `GUIDE.it.md` (con 16 schermate commentate in `images\`) e pagina interattiva **Guida** (`VscBook`) in basso a sinistra nella barra delle attività.

**Fatto**
- **5 capitoli e 16 sezioni navigabili** nello stile Workbench di VSCodium (`ui/src/guide/`): architettura portabile e layout su disco, 36 fonti ufficiali con tabella ricercabile e stato live dei componenti installati, account GitHub unificato (D-70), panoramica visiva di tutte le viste e 12 ricette di configurazione passo-passo.
- **Galleria visiva WebP con Lightbox** (`ui/public/guide/*.webp`) e collegamenti contestuali *«Guida»* nell'intestazione di tutte le pagine principali dell'app per aprire direttamente il capitolo associato.

## Versioni di Apache (16/10/2026)

Apache ha una sola linea (2.4) e Apache Lounge cancella una build quando esce la successiva: l'elenco delle versioni aveva sempre una voce sola.

**Fatto**
- **Una versione per patch** (`patchLines` nel catalogo, D-80): Apache si sceglie e si scarica come `2.4.69`, `2.4.66`…, ognuna nella sua cartella `apache\2.4.66`. Un aggiornamento si installa accanto alla versione precedente, che resta salvata.
- **Archivio di Apache Lounge**: oltre alla build corrente, l'elenco propone l'ultima build di ogni compilatore ritirato (VS17 2.4.66, VS16 2.4.57, VC15 2.4.54, VC14 2.4.41), ognuna con lo SHA-256 pubblicato. VC11 (2.4.38) è escluso: richiede il runtime di Visual C++ 2012.
- Tabella degli strumenti: «Installa» è un'icona.

**Cambia**
- Al primo avvio la cartella `apache\2.4` prende il nome della versione che contiene (`apache\2.4.69`). La scelta «2.4» di un progetto non vale più: il progetto segue il kit finché non sceglie una versione. L'elenco delle versioni di Apache già salvato si chiede di nuovo.

## Dati per progetto e file di passaggio nel temp del sistema (16/10/2026)

Ciò che il kit tiene per un progetto sta in una cartella sola, e download ed estrazioni non scrivono più sulla chiavetta (D-89, D-90).

**Fatto**
- **`data\projects\<progetto>\`** al posto di cinque posti: impostazioni, porte, configurazioni di Apache e Nginx, e in `private\` (solo il proprietario, protetta prima di scrivere) le password e le copie dei database. La migrazione parte da sola al primo avvio dell'app, si può ripetere e conserva `ports.json` e `secrets.json` in `data\app\pre-v2\`.
- **Eliminare un progetto** toglie anche i suoi database e l'utente dai server, dopo una conferma che elenca cosa sparisce; «Elimina senza i database» se un server non risponde.
- **Dati senza progetto**: una cartella di `data\projects\` il cui progetto è stato rinominato o tolto fuori dall'app compare nella pagina Progetti, con «Collega a…» ed «Elimina». Mai cancellata da sola.
- **Database più sicuri.** Il nome del database si salva (prima si ricavava dal nome del progetto, e due progetti potevano contendersi lo stesso: eliminarne uno cancellava l'altro); nomi uguali rifiutati, nomi dei server riservati (`mysql`, `root`, `admin`…). Una copia `.sql` si ripristina come utente del progetto, non come `root`. Il `_` nel `GRANT` di MySQL è escapato.
- **Alias dei siti** (`/media` → `storage/uploads`, `/shared` → `@common/assets`) validati e risolti in un punto solo: niente `..`, nessun collegamento seguito, controllo anche al momento di scrivere la configurazione; con `@altro/cartella` funzionano per Apache, Nginx e i pool. Un alias saltato è detto nella pagina del sito.
- **File di passaggio nel temp del sistema** (`<TEMP>\Atollo\<id>\`): download, estrazioni, framework in creazione, file scaricati a mano, caricamenti di Adminer. Un kit su FAT32/exFAT non ha più il limite di 4 GiB né la lentezza della chiavetta durante questi passi; il risultato si copia in `data\apps` accanto alla destinazione e si scambia in un colpo solo. Con meno di 4 GiB liberi nel temp o senza temp l'installazione ripiega su `data\tmp`, con un avviso nel log. Cartelle dimenticate da un'app chiusa male si tolgono all'avvio dopo 24 ore.
- **Impostazioni › «Temporanei dei programmi»**: Automatico, Nel kit, Nel sistema (`TEMP` dei programmi e `CLAUDE_CODE_TMPDIR`). Automatico sceglie secondo il disco: fisso NTFS/ReFS nel kit (con permessi); rimovibile, di rete, FAT32 o exFAT nel sistema. La pagina mostra file system e tipo del disco del kit.
- **Rust su FAT32/exFAT** si installa e si aggiorna (voce aperta chiusa): rustup lavora nel temp e l'albero si copia nel kit, con le copie al posto dei collegamenti fisici. Un file oltre 4 GiB su FAT32 ferma l'installazione con un errore che dice quale, senza toccare la versione installata.

**Cambia**
- **Una versione precedente dell'app non legge `data\projects`**: tornare indietro non è automatico. Per farlo a mano si usano le copie in `data\app\pre-v2\` (`ports.json`, `secrets.json`) e si rimettono i dati in `data\app\projects\` e `data\db\backup\`.
- **Da riga di comando su un kit vecchio** (`Atollo.exe` senza finestra, prima del primo avvio dell'app) i dati dei progetti non si vedono: la migrazione gira solo come unica copia dell'app del kit, al primo avvio.
- I file aggiunti a mano ora sono nel temp del sistema; `data\downloads` si accetta ancora e il suo contenuto si sposta al primo uso.

**Verificato**: 322 test Rust e 66 test dell'interfaccia; clippy, fmt e lint puliti. Da provare a mano: migrazione di un kit reale, chiavetta exFAT e FAT32, eliminazione con database veri, alias con un collegamento ([TESTING.it.md](TESTING.it.md)).

## Interfaccia con i componenti di VSCodium (15/10/2026)

L'app usa gli stessi controlli di VSCodium e gli stessi colori dell'editor, chiave per chiave, con il brand Atollo (D-88, [ARCHITECTURE.it.md](ARCHITECTURE.it.md)).

**Fatto**
- **Controlli di VSCodium** (`@vscode-elements/elements`): pulsanti, campi, aree di testo, select, badge, barre di avanzamento, divisori, icone codicon, voci delle impostazioni. Checkbox, radio, pulsanti a icona, tabelle, riquadri e schede restano elementi propri disegnati come i componenti, perché quelli dei componenti nasconderebbero etichette e ruoli all'Assistente vocale.
- **Un solo tema**: `themes.py` genera dal tema Atollo dell'editor tutte le variabili `--vscode-*` (circa 800 per tema) che i componenti leggono; i colori dell'app ne derivano. Cambiare un colore in un posto lo cambia nell'editor e nell'app.
- **Pagine**: i controlli scritti a mano sono passati ai componenti comuni; restano nativi, con il motivo scritto, pochi casi (pulsanti della finestra, barra delle attività e di stato, scelta dell'agente, misuratore dei limiti, campo con suggerimenti delle estensioni).
- **Contrasto** corretto dopo la verifica visiva nei due temi: barra di stato (anche nel tema chiaro dell'editor), errori e avvisi, righe arancioni, procedura guidata. Freccia di VSCodium nelle sezioni comprimibili.

**Verificato**: 274 test Rust; tipi, ESLint e build puliti; 56 test dell'interfaccia (nuovi: controlli, registrazione dei componenti, copertura delle variabili). Pagine controllate nel browser in scuro e in chiaro. Da provare nell'app: l'aspetto nella finestra di Tauri accanto all'editor.

## Struttura del codice, seconda parte (15/10/2026)

Nessun cambiamento per chi usa l'app: stessi testi, comandi, eventi, log e file su disco (D-87, [ARCHITECTURE.it.md](ARCHITECTURE.it.md)).

**Fatto**
- **Moduli Rust in cartelle per area**: `setup\`, `versions\`, `projects\`, `editor\`, `agents\` (con `sync\` e `stats\`), `db\`, `github\`, `net\`, `os\`, accanto a `commands\`, `install\` e `detect\`. In cima a `tauri\src` restano `main`, `cli`, `kit`, `env`, `fsx`, `log`, `i18n` e `settings`. Spostamenti con `git mv`; i prefissi spariscono (`editor_cfg.rs` → `editor\cfg.rs`, `db_auth.rs` → `db\auth.rs`, `setup_task.rs` → `setup\task.rs`).
- **File grandi divisi**: `db\services` (1303 righe: impostazioni e avvio, dati per versione, accessi, copie, strumenti), `install\lang` (1162: un file per linguaggio o famiglia), `versions` (1022: scelta, righe offerte, disco), `projects\frameworks` (1022: controlli, piano, destinazione, esecuzione), `versions\relations` (933: conoscenza, controlli, progetti), `github` (840: account e repository). Nessun file supera più le 900 righe.
- **Un errore per i comandi**: `CmdError` e `CmdResult` in `commands\error.rs`; `or_fail` e `or_fail_chain` al posto dei `map_err(|e| fail(…))`, `?` al posto dei `map_err(|e| format!("{e:#}"))`. L'interfaccia riceve la stessa stringa di prima.
- **Interfaccia, pagine grandi in cartelle**: `projects\`, `frameworks\`, `versions\`, `github\`, `settings\`, `components\`. Sparisce il ciclo di import fra le due pagine dei progetti; `pages.tsx` diventa `common.tsx` e `useAction` va in `hooks.ts`.
- **`useLoad`** in `hooks.ts` (dato, errore, ricarica, lettura periodica) per 16 letture in 14 file, con la politica d'errore scritta a ogni lettura: tenere il valore, mostrare l'errore o un valore di ripiego.
- **Componenti di `workbench\`**: `Select` al posto di 13 `<select>`, `DataTable` di 5 `<table>`, `ConfirmDialog` e `InlineConfirm` per la chiusura dell'app e l'eliminazione di un repository GitHub; `useCopy` per le 3 copie negli appunti.
- **Commenti**: i percorsi dei file Rust citati nell'interfaccia e nella CI puntano alle cartelle nuove; tolti i caratteri di controllo lasciati da escape sbagliati in tre commenti.

**Non fatto** (elenco in ARCHITECTURE.it.md): 12 letture restano senza `useLoad` perché cambierebbe ciò che l'utente vede (un errore che oggi resta dopo una lettura riuscita); quattro conferme sì/no nella pagina restano senza il ruolo `alertdialog` (fatte il 17/10/2026, D-106).

**Verificato**: 274 test Rust (3 nuovi per `CmdError`, 3 in più dalla prova delle righe di versione divisa per linguaggio), 2 ignorati; clippy e fmt puliti; tipi, ESLint, Prettier e 36 test dell'interfaccia (9 nuovi per `useLoad`); nessun ciclo di import in `ui\src`. La build dell'interfaccia riesce, con un avviso: il file principale supera 500 kB (761 kB).

## Struttura del codice (15/10/2026)

Nessun cambiamento per chi usa l'app: stesso comportamento, codice più facile da leggere e da cambiare (D-86, [ARCHITECTURE.it.md](ARCHITECTURE.it.md)).

**Fatto**
- **Comandi divisi per area**: `commands.rs` (2655 righe, 87 comandi) diventa la cartella `commands\` con un file per pagina o area (`app`, `window`, `editing`, `setup`, `versions`, `agents`, `projects`, `frameworks`, `github`, `databases`, `services`, `runs`); in `mod.rs` restano lo stato condiviso, i controlli comuni e l'elenco per Tauri.
- **File del kit in un posto solo** (`fsx.rs`): lettura senza BOM e con i byte non UTF-8 sostituiti, JSON mancante, vuoto o danneggiato, scrittura atomica con la cartella creata. Prende il posto di 17 letture e una decina di scritture ripetute nei moduli (impostazioni, impostazioni dei progetti, servizi, porte, versioni offerte, framework, matrice delle relazioni, aggiornamento). `write_atomic` passa da `kit.rs` a `fsx.rs`.
- **Browser ed Esplora risorse** da una sola funzione, `jobs::open_with_explorer` (prima cinque copie), con `jobs::system_root`.
- **Percorsi**: `Kit::project_states()` per `data\app\projects`, `versions::offered_file` usato anche dalla matrice delle relazioni.
- **Interfaccia**: `api.ts` (1877 righe) diventa la cartella `api\`, un file per area con gli stessi nomi di `commands\`; i 92 `browserPreview ? … : invoke(…)` diventano `query`, `act` e `subscribe` di `api\core.ts`. Gli import `./api` delle pagine restano gli stessi.
- **Hook comuni** in `hooks.ts`: `useSubscription` (eventi di Rust, prima sette copie dello stesso codice di registrazione e rimozione) e `useEvery` (letture periodiche, prima quattro copie).

**Cambia un poco**
- Un file JSON vuoto del kit vale come mancante: le Impostazioni non mostrano più un errore per un `settings.json` vuoto.
- `services.json` scritto a mano con il BOM ora si legge (prima si ignorava).

**Verificato**: 268 test Rust (4 nuovi per `fsx`), clippy e fmt puliti; tipi, ESLint, Prettier e 27 test dell'interfaccia.

## Adminer per MySQL, come phpMyAdmin (15/10/2026)

**Fatto** (D-85)
- **«Apri Adminer»** nella pagina di MySQL: il database nel browser, come phpMyAdmin di EasyPHP e XAMPP (tabelle, righe da modificare, query SQL, importazione ed esportazione), già entrato come `root`. Al primo clic installa Adminer (1 MB, nuovo componente nel gruppo database) e avvia MySQL se è fermo.
- La pagina di MySQL non ha più la scheda «Contenuto»: c'è Adminer. Resta per MongoDB.
- Lo serve il PHP del kit solo su 127.0.0.1, con un token nuovo a ogni avvio: senza token nessuno entra, nemmeno gli altri utenti del PC. La password di `root` non passa dal browser. Si ferma con MySQL o con l'app.

**Verificato**: 264 test Rust, clippy e fmt puliti; tipi, ESLint e 27 test dell'interfaccia. Installazione reale da GitHub (SHA-256 e controllo `php -l`: 6.1.2) e test reale con il server SQL del kit: 403 senza token o con quello sbagliato, ingresso con il token, elenco dei database, server riusato, fermato con il server SQL. Da provare nell'app: il pulsante e il browser che si apre.

## SDK di Android nel kit e Capacitor (15/10/2026)

**Fatto** (D-84)
- **Android SDK** in Strumenti: strumenti a riga di comando ufficiali, `adb`, ultima piattaforma e build-tools, tutto in `data\apps`; `ANDROID_HOME` impostato per Gradle. Serve Java. Sceglierlo accetta i termini dell'SDK di Google.
- Nel wizard le basi **Android** e **NativePHP Mobile** usano l'SDK del kit; nuova aggiunta **Android**: con Vite è Capacitor (la stessa app web come app Android), con Tauri è Tauri mobile (con l'NDK, scaricato solo per questa combinazione).

**Verificato**: 261 test Rust (nuovi: zip e SHA-256 dalla pagina di download, scelta dei pacchetti stabili, Capacitor nelle combinazioni) e lettura reale della pagina di Google; clippy, fmt, tipi, ESLint e 27 test dell'interfaccia. Da provare nell'app: l'installazione reale (circa 600 MB) e una build con un telefono collegato.

## Nuovo progetto da framework (15/10/2026)

**Fatto** (D-83)
- **Wizard «Nuovo progetto»** (icona sopra Progetti): una base (Laravel, Symfony, Vite, Next.js, Electron, Tauri, Android) e le aggiunte che vanno con lei: interfaccia (React, Vue, Svelte, Livewire), stili (Tailwind CSS, Bootstrap, Bulma), database (SQLite, MySQL, MongoDB), app desktop e Android con NativePHP, prove, formattazione, Vite per Symfony. Il kit installa gli strumenti che servono, esegue i comandi ufficiali mostrandone l'output, crea il database e scrive il `.env`, salva le azioni del progetto.
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
- **MySQL e MongoDB con più versioni**: nella pagina Database la versione del server (a server fermo); ogni versione ha i suoi dati, copiati in avanti dalla precedente al primo avvio (MongoDB una maggiore alla volta, con la FCV aggiornata); i dati esistenti passano nella cartella della loro versione. La matrice legge le immagini di `compose.yaml`.
- **«locked»**: una versione usata da progetti ha il lucchetto con l'elenco dei progetti e non si rimuove; le altre hanno il cestino.

**Verificato**: 239 test Rust (nuovi: intervalli di versione, matrice, versioni dei progetti, dati dei server per versione), clippy e fmt puliti; tipi, ESLint e 23 test dell'interfaccia; installazione di pnpm e yarn provata in una cartella di prova; anteprima nel browser. Da provare nell'app: la matrice su un kit reale con progetti, il primo avvio con i dati di MySQL già presenti, R scaricato in due versioni.

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

**Verificato**: test Rust di `db_browse` e test reale su MySQL e MongoDB del kit di sviluppo (albero, righe, nome malevolo rifiutato); 23 test dell'interfaccia; anteprima nel browser in tema chiaro e scuro.

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
- **Copie dei database**: Esporta e Ripristina per MySQL e MongoDB nella pagina del progetto › Risorse; copie in `data\db\backup\<progetto>`. MongoDB porta con sé i Database Tools.

**Verificato**: 211 test Rust (con esportazione, cancellazione e ripristino reali su MySQL e MongoDB), 23 test dell'interfaccia; prova nell'app: azione con variabile, impostazioni dell'editor e server MCP scritti nei file del progetto.

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
- **Password per i database** (D-74, dopo la revisione di sicurezza): `root` di MySQL e `admin` di MongoDB con una password casuale, anche sui dati già creati; ogni progetto con il suo database e il suo utente; scrittura della connessione nel `.env` con anteprima delle righe che cambiano. Nel terminale del kit `mysql` entra già come root.
- **MySQL**, **MongoDB** con mongosh e **SQLite** fra i componenti, nel nuovo gruppo «Database», verificati con SHA-256 o SHA3-256 (D-73).
- **Pagina «Database»**: avvio e arresto dei server, stringa di connessione da copiare, porta, cartella dei dati, log, «Avvia con Atollo». Solo 127.0.0.1; dati in `data\db`. «Avvia» installa il database se manca; chiusura dell'app e «Rimuovi l'unità» li fermano in modo pulito.
- I progetti che usano un database (`.env`, driver npm/pip/Composer, file `.sqlite`) lo segnalano e ricevono le estensioni dell'editor: SQLTools, MongoDB for VS Code, SQLite Viewer; nuovo linguaggio SQL.

**Verificato**: 200 test Rust, 22 test dell'interfaccia; installazione reale dei tre componenti; avvio, query e arresto di MySQL e MongoDB 9.0.4 dall'app.

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
- **Pagina GitHub** (`github.rs`, `github.tsx`, D-70), porting dell'estensione `github-repos` degli script PowerShell da cui l'app è nata: i repository dell'account e delle sue organizzazioni con ricerca, clone in `projects\` con un clic, «Editor» sui repository già clonati, apertura su github.com, copia dell'indirizzo, rinomina (i remote del clone seguono) ed eliminazione su GitHub con conferma.
- **Un solo account GitHub per tutto il kit**: l'accesso dalla pagina (codice monouso su github.com/login/device) è quello di GitHub CLI in `data\gh`. Git del kit lo usa per github.com, quindi push e pull nell'editor, nelle console e negli agenti usano lo stesso account; «Esci» lo toglie a tutti. Anche «Clona» della pagina Progetti usa l'account per i repository privati.
- Il permesso di eliminare i repository si chiede solo quando serve («Concedi il permesso»).

**Verificato**: 190 test Rust, 22 test dell'interfaccia; `clippy -D warnings`, ESLint, Prettier, `cargo fmt` puliti. Da provare con un account vero: la checklist «GitHub» di TESTING.it.md (accesso senza console di `gh`, push dall'editor).

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
- **README in quattro lingue** (`README.md`, `docs/README.en.md`, `.fr.md`, `.de.md`): uso, pagine, cosa resta di Atollo, limiti noti.
- **CI: firma facoltativa dell'exe** con i segreti `SIGN_PFX_BASE64` e `SIGN_PFX_PASSWORD`; senza, l'exe resta non firmato e il passo si salta.

**Corretto durante le prove sull'exe di release**
- Sulle pagine lunghe scorreva anche la finestra intera, barra del titolo compresa: gli elementi solo per lettori di schermo uscivano dal contenitore. Ora scorre solo il contenuto.
- La barra «Applica» di Linguaggi e strumenti lasciava vedere sotto di sé una striscia della lista.

**Verificato**
- 181 test Rust (fra cui periodo locale, date ISO, `Retry-After`, finestre e crediti di Claude, conteggi delle sessioni di Codex senza doppioni, quote di Antigravity, cache con attesa dopo 429, editor che non esce dal progetto, `dev-settings.json`), 20 test dell'interfaccia; `clippy -D warnings`, ESLint, Prettier puliti.

**Da provare a mano** (checklist in [TESTING.it.md](TESTING.it.md))
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

**Da provare a mano** (checklist completa in [TESTING.it.md](TESTING.it.md))
1. Agenti › Apri su Claude Code: si apre una console classica di Windows e Claude Code riceve i tasti (Invio, frecce, Ctrl+C).
2. Avviare l'app su un kit con l'editor di un'altra versione: all'avvio viene installato di nuovo alla versione fissata; Editor e Terminale si aprono anche se l'aggiornamento non riesce.
3. Su un kit senza Git: all'avvio Git si installa con GitHub CLI; in Linguaggi e strumenti è spuntato e non modificabile («per GitHub CLI»).
4. Su un kit senza Node.js: «Avvia» su un'azione `npm` installa prima Node.js e poi parte.
5. Linguaggi e strumenti: Apache mostra la dipendenza «PHP»; con Claude Code scelto, Git mostra «per Claude Code».

## Fase 6: progetti e rifinitura (14/10/2026)

**Fatto**
- **Progetti** (`projects.rs`, `detect/`, `run.rs`): rilevamento del tipo, dei linguaggi e delle azioni (porting dei rilevatori di Atollo, `panel.json`/`pannello.json`). Creazione, clone da GitHub o da un indirizzo Git, eliminazione delle cartelle vuote. Avvio e arresto delle azioni, ognuna nel suo job, con porte 7100-7999 ricordate o scelte, output in diretta, apertura nel browser e «Servi come sito web» (D-50).
- **Server di file nell'exe**: `Atollo.exe serve <cartella> <porta>`, al posto di `static-server.js` (D-51).
- **Aggiornamento dell'app** (`self_update.rs`) dalle release `app-v*`: SHA-256, scambio dell'exe alla chiusura, `ATOLLO_NO_UPDATE`, versione in `data\app\versions\app.txt` (D-52).
- **Impostazioni**: lingua, rete (proxy, certificati, rete locale), chiave di Gemini, versioni di Python, PHP e Rust, aggiornamenti. Valori controllati; l'ambiente del kit viene ricostruito al salvataggio (D-54).
- **Rimuovi l'unità** e conferma alla chiusura quando sono aperti editor, console, progetti o un'installazione. Nella Home c'è un avviso se è aperta un'altra copia del kit (`sessions.rs`, D-55).
- **Interfaccia in italiano, inglese, francese e tedesco**, con la lingua che cambia senza riavvio. Accessibilità: collegamento «Vai al contenuto», ruoli ARIA, dialogo accessibile, «riduci animazioni», contrasto del rosso nel tema chiaro (D-56).
- **Revisione di sicurezza**: le credenziali del PC in `keyring-before.json` sono cifrate con DPAPI. Le cartelle attendibili di `projects` sono documentate come rischio accettato (D-53).
- **Checklist manuale** in [TESTING.it.md](TESTING.it.md).

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

**Da provare a mano** (checklist completa in [TESTING.it.md](TESTING.it.md))
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
- Claude Code si apre e si disegna, ma non riceve i tasti. Causa probabile e rimedio (ConPTY aggiornato di Microsoft) in D-47; la prova aspetta il permesso dell'utente (domanda 11 del piano iniziale). Chiuso dopo la fase 6: le console sono ora la console classica di Windows (D-57, sezione in alto).

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
- Installatori di tutti i 31 componenti del catalogo (specifica in CATALOG.it.md): agenti (Claude Code con firma Anthropic, Codex, Antigravity), Git, Python (pacchetti dell'utente conservati), Node.js, uv, gh, PHP con Composer, `cacert.pem` e runtime VC++ estratto da `vc_redist` (firma Microsoft), Apache con `kit.conf`, Rust (rustup, voce di registro ripristinata), Go, Java con Maven e Gradle, Kotlin, .NET, Dart, Zig, Julia, Bun, Deno, Typst, TinyTeX, pnpm e yarn, CMake con Ninja, C/C++ (llvm-mingw), Ruby, R (micromamba), PowerShell 7, Windows Terminal, editor, VSCodium desktop (facoltativo).
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
- Rust su un kit in FAT32/exFAT non si installa (rustup vuole i collegamenti fisici); il ripiego di Atollo (toolchain sul disco del PC) non è ancora portato. **Chiuso il 16/10/2026** (D-90): rustup lavora nel temp del sistema e l'albero si copia nel kit.
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
- `runtime/profile.ps1` e `runtime/bashrc`: prompt "Atollo", strumenti disponibili, moduli e cronologia nel kit.
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
- `catalog.rs` + `catalog/components.json`: i 32 componenti (agenti e linguaggi, più PowerShell 7, Windows Terminal, editor, WebView2) in ordine di installazione; scelta in `data\app\components.txt` nel formato di Atollo (`id=1|0`, CRLF); dipendenze a cascata (Apache con PHP, Kotlin con Java); versioni in `data\app\versions\`.
- `net.rs`: HTTPS con il TLS di Windows e i certificati del suo archivio (funziona con l'ispezione SSL aziendale), proxy dalle impostazioni, dall'ambiente o di Windows; download in `.part` con 3 tentativi; API di GitHub.
- `verify.rs`: SHA-256, SHA-512, integrità npm; un file che non corrisponde viene cancellato.
- `archive.rs`: zip, tar.gz, 7z (anche dentro un `.7z.exe`), con rifiuto dei percorsi che escono dalla cartella; scambio delle cartelle con ripristino, "in uso" se un file è aperto.
- `setup_runner.rs`: ordine dei passi, una preparazione alla volta per kit, rimozione dei componenti tolti, annullamento; un componente che fallisce non ferma gli altri.
- Installatori: **PowerShell 7**, **Windows Terminal** (modalità portabile), **Git** (con `post-install` e configurazione: `credential.helper=manager`, `.gitconfig` del kit), **Node.js LTS**, **editor** (server web di VSCodium, SHA-256 di VSCodium confrontato con quello di GitHub).
- Riga di comando: `Atollo.exe prepare [--only a,b] [--skip a,b] [--update-all] [--force]` e `Atollo.exe components`, con i marcatori di Atollo (`── N. Nome ──`, `√`, `x`, `!`).
- CI: installazione reale dei cinque componenti e controllo degli aggiornamenti nel job x64.

**Verificato su questo PC**
- `cargo fmt`, `clippy -D warnings`: puliti; `cargo test`: 49 test verdi.
- `prepare --only pwsh,terminal,node` in 21 s e `--only git,vscodium-server` in 34 s, da una cartella `kit2 (prova) & co`: Node v24.21.0, PowerShell 7.6.7, Windows Terminal 1.25.2733.0 (con `.portable`), Git 2.56.0.2 (Bash funziona, post-install eseguito), editor 1.140.06880 (`bin\codium-server.cmd` presente). Spazio occupato: circa 1,1 GB.
- `--update-all`: "già aggiornato" senza riscaricare.
- PowerShell aperto durante `--force`: "in uso", cartella e versione intatte, nessun residuo.
- Tre preparazioni consecutive: nessun file nuovo in `%LOCALAPPDATA%`, `%APPDATA%` o `%TEMP%` reali dovuto ad Atollo (i file trovati erano di Chrome, Office, Notion, OneDrive, Teams, aperti sul PC).

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
