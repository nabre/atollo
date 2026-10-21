# 01 — Bug, Sicurezza e Portabilità (`v6`)

> **Riferimento codice**: `HEAD 6ecd492` (21/10/2026). Ogni rilievo ha: dove, scenario (stato → risultato sbagliato), evidenza, correzione. **Confermato** = percorso del codice univoco alla lettura; **PLAUSIBILE** = il fattore scatenante è raro o dipende da un comportamento esterno da verificare.
> I rilievi `v5` sono chiusi (vedi [`CHANGELOG.md`](../../app/CHANGELOG.md), voce «Rilievi del report v5») e non sono ripetuti.

---

## 1. Tabella riassuntiva

| ID | Grav. | Stato | Area | Titolo |
|---|:---:|:---:|---|---|
| `BUG-V6-01` | ALTA | Confermato | DB | Ripristino MySQL interrotto da `DEFINER=root` |
| `BUG-V6-02` | ALTA | Confermato | Installatori | Aggiornamento conda: l'ambiente vecchio cancellato prima del nuovo |
| `PORT-V6-01` | ALTA | Plausibile | OS | DACL «solo proprietario» illeggibile su un altro PC/account |
| `BUG-V6-04` | MEDIA | Confermato | DB | Dump senza `--hex-blob`, ripristino con `read_to_string` |
| `BUG-V6-05` | MEDIA | Confermato | DB | `run_with_input` ignora il timeout durante la scrittura di stdin |
| `BUG-V6-06` | MEDIA | Confermato | DB | Archivio MongoDB parziale elencato come valido, poi `--drop` |
| `BUG-V6-07` | MEDIA | Confermato | DB | Utente MySQL oltre 32 caratteri: il database non nasce |
| `BUG-V6-08` | MEDIA | Confermato | Server web | Caddy dedicato: nome del progetto grezzo nell'indirizzo del sito |
| `BUG-V6-09` | MEDIA | Confermato | DB | Copia dei dati iniziali MySQL sul posto |
| `BUG-V6-10` | MEDIA | Plausibile | Progetti | `Runner::finish` per chiave d'azione: una corsa sostituita uccide la nuova |
| `BUG-V6-11` | MEDIA | Plausibile | Server web | `caddy reload` fallito → stop e start di tutto il pool |
| `BUG-V6-13` | MEDIA | Plausibile | Rete | Download fermo: nessun timeout sul corpo, «Annulla» inefficace |
| `BUG-V6-14` | MEDIA | Confermato | Setup | Un download fatto a mano viene consumato anche se l'installazione fallisce |
| `BUG-V6-15` | MEDIA | Confermato | Agenti | Dopo «Rimuovi unità» le sincronizzazioni in background scrivono ancora |
| `BUG-V6-17` | MEDIA | Confermato | Agenti | Salvataggio del portachiavi riprovato ogni 3 s per sempre; log senza rotazione |
| `SEC-V6-01` | MEDIA | Plausibile | DB, server | File di segreti/stato illeggibile → riscritto vuoto |
| `SEC-V6-02` | MEDIA | Scelta da rivedere | Agenti | Token a lunga durata in ogni processo del kit, cartelle `projects/` già fidate |
| `BUG-V6-22` | BASSA | Confermato | Router | `bootstrap::take` cancella un file legacy di un altro componente |
| `BUG-V6-23` | BASSA | Plausibile | Rete, update | `.old` ancora in uso blocca la sostituzione |
| `BUG-V6-25` | BASSA | Confermato | OS | Processo avviato e lasciato fuori dal Job se `Job::new`/`assign` falliscono |
| `BUG-V6-26` | BASSA | Confermato | DB | Amministratore MongoDB creato prima di salvarne la password |
| `BUG-V6-27` | BASSA | Confermato | Core | `write_atomic` senza `fsync` |
| `BUG-V6-28` | BASSA | Confermato | Progetti | Server statico: `//cartella` dà un redirect relativo al protocollo |
| `BUG-V6-29` | BASSA | Confermato | Agenti | Uscita da Antigravity annullata se agy è aperto; comandi sul thread principale |
| `BUG-V6-31` | BASSA | Confermato | Editor | Virgolette tipografiche nel percorso rompono il comando PowerShell |
| `BUG-V6-32` | BASSA | Confermato | Agenti | Accesso «api key» di Claude/Codex irraggiungibile |
| `BUG-V6-33` | BASSA | Confermato | Vari | Minori: `sync --project` vuoto, `components.txt` senza lock, token scritto poi protetto |
| `SEC-V6-03` | BASSA | Confermato | Rete | Editore Authenticode confrontato come sottostringa |
| `SEC-V6-04` | BASSA | Plausibile | Installatori | Metadati remoti usati nei nomi dei file senza filtro |
| `PORT-V6-05` | BASSA | Plausibile | Rete | `:` nei nomi d'archivio rifiutato anche su Unix |
| `PORT-V6-06` | BASSA | Plausibile | Agenti | Chiave di sigillo persa dopo crash e spostamento (macOS/Linux) |
| `TEST-V6-01` | BASSA | Confermato | Prove | `ports_are_kept_and_locked` instabile in parallelo |

---

## 2. ALTA

### `BUG-V6-01` [ALTA] — Il ripristino MySQL si interrompe su viste, trigger, routine ed eventi

- **Dove**: [`db/services/backups.rs:23-29`](../../../tauri/src/db/services/backups.rs#L23-L29) (`DUMP_ARGS`), [`:151-172`](../../../tauri/src/db/services/backups.rs#L151-L172) (ripristino).
- **Scenario**: l'esportazione gira come `root` con `--routines --events --triggers`; il dump contiene ``DEFINER=`root`@`localhost` ``. Il ripristino gira, per scelta (D-78), come utente del progetto, che ha solo `GRANT ALL ON db.*`. Creare un oggetto con un `DEFINER` altrui richiede `SET_USER_ID`/`SUPER`: MySQL risponde `ERROR 1227` e il client si ferma al primo errore. Il dump è per tabella `DROP; CREATE; INSERT`: alcune tabelle sono ripristinate, le altre restano com'erano. Il database è incoerente.
- **Evidenza**: `"--routines", "--events", "--triggers"` nel dump; `// As the project's user, never root` nel ripristino. Nel repository nessuna gestione di `DEFINER`.
- **Correzione**: togliere `DEFINER=...` mentre si passa l'SQL al client (filtro in streaming su `/*!50013 DEFINER=... */` e `DEFINER=`…`@`…``), così gli oggetti nascono con l'utente del progetto. Prova: dump con vista e trigger → ripristino completo.

### `BUG-V6-02` [ALTA] — Aggiornare un componente conda cancella quello funzionante prima di avere il nuovo

- **Dove**: [`install/native.rs:723-726`](../../../tauri/src/install/native.rs#L723-L726) (`micromamba_create`); stesso schema in [`install/lang/cc.rs:346-348`](../../../tauri/src/install/lang/cc.rs#L346-L348). Passano di qui R, Ruby, C/C++, MongoDB su Linux, SQLite su Linux ARM64.
- **Scenario**: R 4.5.1 installato; «Aggiorna tutto» con la rete che cade, conda-forge irraggiungibile, solver che fallisce o disco pieno. `remove_dir(prefix)` è già avvenuto: l'ambiente non c'è più, il file di versione dice ancora 4.5.1, il componente risulta non installato e va riscaricato. Offline (chiavetta su un PC senza rete) non c'è rimedio.
- **Evidenza**: `// The same line again (a patch): the old folder goes first.` `if prefix.exists() { remove_dir(prefix)?; } ... run(ctx, &mm, &args, ...)?;`
- **Correzione**: spostare il vecchio prefisso da parte (`<prefix>.old-<ms>`), creare il nuovo, togliere il vecchio solo a successo; in caso d'errore rimetterlo al suo posto (lo schema `set_aside` di `conda::restore` esiste già). Conda incide il prefisso nei file: va creato nel percorso definitivo, quindi «sposta da parte e ripristina», non «crea altrove e scambia».

### `PORT-V6-01` [ALTA, da confermare] — La DACL «solo proprietario» rende i file del kit illeggibili su un altro PC

- **Dove**: [`router/fs.rs:26-31`](../../../tauri/src/router/fs.rs#L26-L31) (`D:P(A;;FA;;;OW)`, protetta, Owner Rights), applicata al token dell'editor ([`editor/mod.rs:229-234`](../../../tauri/src/editor/mod.rs#L229-L234)), alla chiave Gemini ([`settings.rs:232`](../../../tauri/src/settings.rs#L232)), allo stato privato dei progetti ([`projects/state.rs:61`](../../../tauri/src/projects/state.rs#L61)), ai segreti dei DB e a `data\tmp`.
- **Scenario**: su un disco **NTFS** la DACL viaggia con il file e nomina il proprietario del PC A. Sul PC B un altro account locale ha un altro SID e Owner Rights non lo riguarda. `fs::write(&token_file, ...)` su un file esistente riceve `ACCESS_DENIED` e l'editor non parte; [`read_secret`](../../../tauri/src/settings.rs#L272-L277) inghiotte l'errore e la chiave Gemini appare «non impostata»; le password dei DB diventano illeggibili. Nessun ripiego (`takeown`, reset della DACL) nel codice. Su exFAT/FAT non succede (niente ACL): il rischio è per SSD esterni NTFS, il caso «chiavetta seria».
- **Correzione**: (1) i segreti di sessione (token dell'editor) vanno ricreati: cancellare e scrivere, non sovrascrivere; meglio in `%TEMP%`. (2) Per i file che viaggiano col kit, su accesso negato: tentare `SetNamedSecurityInfo` come proprietario attuale, altrimenti segnalare con un messaggio che spiega cosa fare. (3) `read_secret` deve distinguere «assente» da «illeggibile». Verifica manuale: due account Windows sullo stesso SSD NTFS.

---

## 3. MEDIA

### `BUG-V6-04` — Dump senza `--hex-blob`, ripristino con `read_to_string`
- **Dove**: [`backups.rs:23-29`](../../../tauri/src/db/services/backups.rs#L23-L29), [`:158`](../../../tauri/src/db/services/backups.rs#L158).
- **Scenario**: una tabella con BLOB o binari non UTF-8 (immagini, hash) → il backup del kit non si ripristina («stream did not contain valid UTF-8»); il file intero va in memoria.
- **Correzione**: `--hex-blob` in `DUMP_ARGS`; ripristino in streaming (`Stdio::from(File)`), con il filtro di `BUG-V6-01` su un lettore a righe di byte.

### `BUG-V6-05` — `run_with_input` scrive stdin prima del ciclo di timeout
- **Dove**: [`db/services/tools.rs:24-27`](../../../tauri/src/db/services/tools.rs#L24-L27).
- **Scenario**: il client esce su un errore SQL → `write_all` dà `BrokenPipe` e l'utente vede «Broken pipe» al posto dello stderr di MySQL; il client si blocca su un lock → la pipe si riempie, `write_all` resta bloccata e i 600 s non scadono mai.
- **Correzione**: scrivere stdin in un thread; a errore di scrittura raccogliere comunque stderr ed esito e riportare quelli.

### `BUG-V6-06` — Archivio MongoDB parziale tra le copie, poi `--drop`
- **Dove**: [`backups.rs:92-100`](../../../tauri/src/db/services/backups.rs#L92-L100); nome con risoluzione al secondo in [`agents/sync/mod.rs:135`](../../../tauri/src/agents/sync/mod.rs#L135) (`stamp()`).
- **Scenario**: `mongodump` scade o fallisce, l'archivio troncato resta ed è elencato; il ripristino fa `mongorestore --drop` e fallisce sul flusso troncato dopo aver tolto le collezioni. Due esportazioni nello stesso secondo condividono il nome.
- **Correzione**: scrivere su `*.part`, rinominare a successo, cancellare su errore (come già fa il ramo MySQL); millisecondi o contatore nel nome.

### `BUG-V6-07` — Nome utente MySQL oltre 32 caratteri
- **Dove**: [`db/auth.rs:425-431`](../../../tauri/src/db/auth.rs#L425-L431) (`is_valid_name` ammette 64), [`:667-678`](../../../tauri/src/db/auth.rs#L667-L678), [`:698-710`](../../../tauri/src/db/auth.rs#L698-L710).
- **Scenario**: progetto `acme-customer-portal-staging-environment` → `CREATE USER` con 40 caratteri → errore 1470 (limite 32 in MySQL 8.4/9.x); il database del progetto non nasce.
- **Correzione**: utente ≤ 32 caratteri (tronca + 6 cifre di hash); il nome già salvato vince per i progetti esistenti.

### `BUG-V6-08` — Caddy dedicato: nome del progetto grezzo nell'indirizzo
- **Dove**: [`projects/caddy.rs:578-580`](../../../tauri/src/projects/caddy.rs#L578-L580) (`format!("{p}.localhost")`); i pool usano già `host_label` ([`servers/pools.rs:67`](../../../tauri/src/servers/pools.rs#L67)).
- **Scenario**: [`validate_project_name`](../../../tauri/src/kit.rs#L779-L803) ammette spazi, `{`, `}`, `#`, unicode. Progetto «My App» → `http://My App.localhost:7100, ... {`: Caddy separa sullo spazio, `http://My` diventa un sito sulla porta 80 (fallisce o apre un ascolto non voluto); `{x}` è un segnaposto.
- **Correzione**: `host_label(project)` anche qui e in `ServerAlias` di Apache ([`apache.rs:348`](../../../tauri/src/projects/apache.rs#L348)); prove con spazi e unicode.

### `BUG-V6-09` — Dati iniziali di MySQL copiati sul posto
- **Dove**: [`db/services/data.rs:100-104`](../../../tauri/src/db/services/data.rs#L100-L104), [`:43-47`](../../../tauri/src/db/services/data.rs#L43-L47) (`has_data` guarda solo `mysql.ibd`).
- **Scenario**: copia interrotta (chiavetta piena o staccata) → `mysql.ibd` c'è già, l'avvio successivo salta la copia e `mysqld` apre una cartella incompleta.
- **Correzione**: copiare in `<linea>.partial`, rinominare a fine copia, pulire i `.partial` all'avvio.

### `BUG-V6-10` [Plausibile] — Una corsa sostituita chiude anche la nuova
- **Dove**: [`projects/run.rs:485-491`](../../../tauri/src/projects/run.rs#L485-L491), [`:710-721`](../../../tauri/src/projects/run.rs#L710-L721), [`:1011-1028`](../../../tauri/src/projects/run.rs#L1011-L1028).
- **Scenario**: due avvii della stessa azione (doppio clic durante `assign_port`/`ensure`) passano entrambi `alive()`; il secondo `insert` scarta il Job del primo (che muore); il thread d'attesa del primo chiama `finish(key)`, trova la seconda voce e la chiude con stato Errore. Muoiono entrambi.
- **Correzione**: un identificativo di corsa nel thread d'attesa e in `finish` (ignorare completamenti vecchi); `start` serializzato per chiave.

### `BUG-V6-11` [Plausibile] — `caddy reload` rifiutato ferma tutto il pool
- **Dove**: [`servers/process.rs:681-724`](../../../tauri/src/servers/process.rs#L681-L724).
- **Scenario**: uno snippet `conf.d/*.caddy` sbagliato → reload rifiutato → il codice ferma il pool e lo riavvia → l'avvio fallisce sulla stessa configurazione. Tutti i siti del pool sono giù.
- **Correzione**: `caddy validate` prima; riavviare solo se l'admin non risponde, non se la configurazione è rifiutata; nel secondo caso riportare l'errore e lasciare in piedi l'istanza.

### `BUG-V6-13` [Plausibile] — Download fermo: nessun timeout sul corpo
- **Dove**: [`net/mod.rs:155-164`](../../../tauri/src/net/mod.rs#L155-L164), [`:550-567`](../../../tauri/src/net/mod.rs#L550-L567).
- **Scenario**: un proxy smette di mandare byte dopo le intestazioni; `body.read` resta bloccato fino al timeout TCP del sistema; «Annulla» è controllato solo tra una lettura e l'altra; il lock dell'installazione resta preso.
- **Correzione**: timeout d'inattività sulla lettura (es. 60 s) con `ureq`; verificare i default della versione in uso.

### `BUG-V6-14` — Un download fatto a mano si perde se l'installazione fallisce
- **Dove**: [`setup/downloads.rs:256-259`](../../../tauri/src/setup/downloads.rs#L256-L259), [`router/bootstrap.rs:104-110`](../../../tauri/src/router/bootstrap.rs#L104-L110).
- **Scenario**: l'utente scarica il JDK dal browser perché il proxy blocca il download; il file viene spostato nella cartella di lavoro; l'estrazione fallisce (antivirus, `InUse`); la cartella di lavoro viene cancellata col file. Al nuovo tentativo va riscaricato a mano.
- **Correzione**: copiare, e cancellare l'originale solo dopo un'installazione riuscita.

### `BUG-V6-15` — «Rimuovi unità» non ferma le sincronizzazioni in background
- **Dove**: [`commands/window.rs:56-112`](../../../tauri/src/commands/window.rs#L56-L112), [`agents/mod.rs:199-201`](../../../tauri/src/agents/mod.rs#L199-L201), [`:246-266`](../../../tauri/src/agents/mod.rs#L246-L266), [`editor/terminal.rs:151-159`](../../../tauri/src/editor/terminal.rs#L151-L159).
- **Scenario**: chiudere agenti e console fa scattare `on_exit` → `sync::run`; il watcher (3 s) vede l'agente sparito e sincronizza, e ritenta `keyring::save`. Le scritture (`CLAUDE.md`, `config.toml`, giunzioni) possono essere in corso quando l'interfaccia dice che l'unità si può staccare.
- **Correzione**: un flag «in chiusura» in `AppState`, impostato prima di fermare qualcosa; watcher e callback d'uscita non sincronizzano se è alzato; l'espulsione prende il lock di sync e aspetta.

### `BUG-V6-17` — Portachiavi riprovato ogni 3 s, log che cresce
- **Dove**: [`agents/mod.rs:259-263`](../../../tauri/src/agents/mod.rs#L259-L263), [`log.rs:16-24`](../../../tauri/src/log.rs#L16-L24).
- **Scenario**: `keyring-before*.json` presente e salvataggio impossibile (Portachiavi bloccato, Secret Service assente) → tentativo ogni 3 s con una riga WARN; su macOS può riaprire la richiesta del Portachiavi; `app.log` ruota solo all'avvio e cresce senza limite.
- **Correzione**: attesa esponenziale e stop dopo N errori; rotazione del log per dimensione anche durante la sessione.

### `SEC-V6-01` [Plausibile, effetto grave] — Un file di stato illeggibile diventa vuoto e viene riscritto
- **Dove**: [`db/auth.rs:53-55`](../../../tauri/src/db/auth.rs#L53-L55) (`secrets.json`), [`db/services/data.rs:193-200`](../../../tauri/src/db/services/data.rs#L193-L200), [`db/services/access.rs:36-70`](../../../tauri/src/db/services/access.rs#L36-L70), [`servers/pools.rs:38-40`](../../../tauri/src/servers/pools.rs#L38-L40), [`db/services/mod.rs:381-383`](../../../tauri/src/db/services/mod.rs#L381-L383).
- **Scenario**: `fsx::read_json` dà `None` su un file danneggiato o bloccato un attimo dall'antivirus; `unwrap_or_default()` lo tratta come vuoto; l'avvio successivo scrive un file nuovo: profili dei server persi, oppure nuova password di root MySQL / amministratore MongoDB e dati chiusi a chiave. Inoltre `secrets.json` è letto-modificato-scritto senza lock e i lock per motore sono distinti: due primi avvii (MySQL e MongoDB) possono cancellarsi la password a vicenda.
- **Correzione**: per segreti e stato non ricostruibile usare `try_read_json` e **rifiutare di scrivere** se il file esiste ma non si legge; un lock unico attorno a `secrets.json` (come `db_json_lock`).

### `SEC-V6-02` [Scelta da rivedere] — Token in ogni processo, progetti fidati prima di essere visti
- **Dove**: [`env.rs:858`](../../../tauri/src/env.rs#L858) (`GEMINI_API_KEY`), [`env.rs:886`](../../../tauri/src/env.rs#L886) + [`claude_token.rs:111-115`](../../../tauri/src/agents/claude_token.rs#L111-L115) (`CLAUDE_CODE_OAUTH_TOKEN`, un anno), [`agents/sync/mod.rs:188-264`](../../../tauri/src/agents/sync/mod.rs#L188-L264), [`agents/sync/codex.rs:290-305`](../../../tauri/src/agents/sync/codex.rs#L290-L305).
- **Scenario**: l'ambiente del kit passa ad azioni dei progetti, script `npm`, terminali ed editor: uno script di un repository clonato può leggere ed esfiltrare il token. Le cartelle di `projects/` sono già fidate per Claude e Codex: hook e server MCP del repository si caricano senza domanda. Il README lo dice («open only projects you trust»), ma la fiducia è concessa prima che l'utente abbia visto il codice.
- **Correzione**: token e chiave solo nell'ambiente di lancio dell'agente, non in `KitEnv`/`project_env`; fiducia concessa al primo avvio esplicito di un agente nel progetto (o un interruttore «Fidati di questo progetto» nella pagina del progetto). Va registrata come decisione `D-1xx`.

---

## 4. BASSA

| ID | Dove | Scenario | Correzione |
|---|---|---|---|
| `BUG-V6-22` | [`router/bootstrap.rs:79-83`](../../../tauri/src/router/bootstrap.rs#L79-L83) | nel vecchio `.bootstrap/` piatto, `verify::check` cancella un file di un altro componente con altro hash | `verify::matches` (non distruttivo) |
| `BUG-V6-23` | [`net/archive.rs:1053-1060`](../../../tauri/src/net/archive.rs#L1053-L1060), [`setup/self_update.rs:618-620`](../../../tauri/src/setup/self_update.rs#L618-L620) | `Atollo.old.exe` ancora in esecuzione → `rename` fallisce → aggiornamento rimandato senza motivo | nome unico `*.old-<ms>` se il vecchio non si toglie |
| `BUG-V6-25` | [`db/services/mod.rs:703-705`](../../../tauri/src/db/services/mod.rs#L703-L705), [`editor/pty.rs:702-711`](../../../tauri/src/editor/pty.rs#L702-L711), [`editor/ext.rs:569-576`](../../../tauri/src/editor/ext.rs#L569-L576) | `spawn` poi `Job::new()?`/`assign()?`: a errore il figlio resta vivo fuori dal Job, con porta e cartella dati | `jobs::spawn_in` (già usato dai pool) o kill+wait su ogni errore |
| `BUG-V6-26` | [`db/services/access.rs:44-70`](../../../tauri/src/db/services/access.rs#L44-L70) | `createUser` poi salvataggio della password: se il salvataggio fallisce l'amministratore esiste con password ignota | salvare prima, creare dopo (come MySQL) |
| `BUG-V6-27` | [`fsx.rs:84`](../../../tauri/src/fsx.rs#L84) | senza `sync_all` prima del `rename`, su exFAT/FAT uno stacco brusco lascia file a lunghezza zero; con `SEC-V6-01` si perde lo stato | `File::create` + `write_all` + `sync_all` + `rename` |
| `BUG-V6-28` | [`projects/serve.rs:128-136`](../../../tauri/src/projects/serve.rs#L128-L136) | `GET //assets` → `Location: //assets/`, letto come `http://assets/` | comprimere le barre iniziali |
| `BUG-V6-29` | [`agents/mod.rs:227-230`](../../../tauri/src/agents/mod.rs#L227-L230), [`commands/agents.rs:112-124`](../../../tauri/src/commands/agents.rs#L112-L124) | uscita da Antigravity con agy aperto: alla chiusura l'accesso torna in `keyring.json`; `agent_sign_out` è sincrono (thread principale), `codex logout` senza timeout | rifiutare l'uscita con l'agente aperto; comando `async` + `spawn_blocking`; timeout |
| `BUG-V6-31` | [`editor/terminal.rs:119-133`](../../../tauri/src/editor/terminal.rs#L119-L133) | PowerShell tratta U+2018..U+201B come apici: kit in `D:\O’Brien\Atollo` → l'agente non parte | raddoppiare anche quei quattro caratteri, o passare il percorso come argomento |
| `BUG-V6-32` | [`env.rs:27-72`](../../../tauri/src/env.rs#L27-L72), [`agents/mod.rs:87`](../../../tauri/src/agents/mod.rs#L87), [`:105`](../../../tauri/src/agents/mod.rs#L105) | `ANTHROPIC_API_KEY`/`OPENAI_API_KEY` non sono nella lista ammessa: i rami «api key» non scattano mai | campo in Impostazioni come per Gemini, o togliere i rami morti |
| `BUG-V6-33` | [`cli.rs:131`](../../../tauri/src/cli.rs#L131); [`commands/mod.rs:177-187`](../../../tauri/src/commands/mod.rs#L177-L187); [`editor/mod.rs:231-232`](../../../tauri/src/editor/mod.rs#L231-L232) | `sync --project` senza valore diventa `None` invece di errore; `components.txt` letto-modificato-scritto senza lock; token scritto e solo dopo protetto | errore d'uso; lock; `write_private` |
| `SEC-V6-03` | [`net/signature.rs:88`](../../../tauri/src/net/signature.rs#L88), [`install/web.rs:110`](../../../tauri/src/install/web.rs#L110), [`install/agents.rs:65`](../../../tauri/src/install/agents.rs#L65) | l'editore Authenticode è una sottostringa del soggetto: un certificato valido con `O=Microsoft Corporation` in un OU passa; per `vc_redist` è l'unico controllo | confrontare l'attributo `O=` analizzato, esatto; eventualmente l'impronta |
| `SEC-V6-04` | [`install/agents.rs:54`](../../../tauri/src/install/agents.rs#L54), [`:163-168`](../../../tauri/src/install/agents.rs#L163-L168), [`install/lang/mod.rs:57`](../../../tauri/src/install/lang/mod.rs#L57), [`install/db.rs:584-587`](../../../tauri/src/install/db.rs#L584-L587) | un manifesto compromesso con `version = "../../x"` fa scrivere fuori dalla cartella di lavoro | in `fetch` ammettere solo `[A-Za-z0-9._+-]` |
| `PORT-V6-05` | [`net/archive.rs:59`](../../../tauri/src/net/archive.rs#L59) | un tar per macOS/Linux con `foo:bar` (valido su Unix) fa fallire l'installazione | rifiutare `:` solo su Windows |
| `PORT-V6-06` | [`agents/keyring.rs:164-168`](../../../tauri/src/agents/keyring.rs#L164-L168) | dopo crash + spostamento del kit la voce della chiave di sigillo ha il vecchio `kit.id()`: `restore` fallisce a ogni avvio | nome della voce da `machine_tag`, o scartare il file irrecuperabile con un avviso |
| `TEST-V6-01` | [`test/rust/projects/run.rs:40-49`](../../../test/rust/projects/run.rs#L40-L49) | la prova usa porte reali 7100+; in parallelo un'altra prova occupa 7100 e `assign_port` ne dà un'altra (7102 ≠ 7100). Passa da sola 3/3 | intervallo di porte iniettabile nel `Runner` per le prove, o porta effimera |

---

## 5. Punti controllati senza rilievi

- **Estrazione degli archivi**: `safe_join`, `no_link_on_the_way`, ordine dei collegamenti, sostituzioni con rollback (`replace_dir`, `stage_with`, `clean_leftovers`). Nessuna zip-slip.
- **Checksum**: niente si installa senza SHA fisso, hash del produttore, integrità npm o SHA-512/SHA3; l'unica eccezione è `vc_redist` (`SEC-V6-03`). Pacchetti conda conservati ricontrollati. Aggiornamento automatico: digest GitHub verificato al download e allo scambio.
- **Lock delle installazioni**: mutex con nome su Windows, `flock` su Unix; cartelle di lavoro per id del kit.
- **Intervalli di versione**: caret, tilde, `~=`, trattino, jolly e `||` corretti secondo npm e PEP 440.
- **Porte**: registro comune coerente (pool 8081-8999, progetti 7100-7999, editor 7000-7099, DB).
- **Comandi IPC**: tutti i 123 chiamano `ensure_main`; argomenti dalla UI validati; nessun segreto nei log.
- **Runtime PHP**: `php-router.php`, `fcgi-guard.php`, phpMyAdmin; ascolti su 127.0.0.1 o 127.x casuale salvo LAN permessa.
