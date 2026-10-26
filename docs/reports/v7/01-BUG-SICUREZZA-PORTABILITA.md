# 01 — Bug, Sicurezza e Portabilità (`v7`)

> **Riferimento codice**: `HEAD 1f84370` (22/10/2026, versione `0.5.1`). Ogni rilievo ha: dove, problema (stato → risultato sbagliato), impatto, correzione. Tutti i rilievi sono stati riletti nel codice da un verificatore avversario; dove la verifica ha cambiato la gravità o ristretto lo scenario, lo dice la riga **Verifica**.
> I rilievi `v6` sono chiusi (vedi [`CHANGELOG.md`](../../app/CHANGELOG.md)) e non sono ripetuti. Due rilievi della dimensione «progetti/db/server» sono stati confutati dalla verifica e non compaiono.

---

## 1. Tabella riassuntiva

I 39 rilievi sono divisi nei quattro gruppi di esecuzione del piano ([05](./05-PIANO-ESECUTIVO-V7.md)): ogni gruppo tocca file diversi e può andare in un worktree a parte.

| ID | Grav. | Gruppo | Area | Titolo |
|---|:---:|:---:|---|---|
| `BUG-V7-01` | MEDIA | A | Modelli locali | La ripresa di un download Hugging Face scarica dal commit nuovo, non da quello del `.part` |
| `BUG-V7-02` | BASSA | A | Modelli locali | Lock e cartella del modello dal nome del repo così come è scritto: due grafie, una cartella, due lock |
| `BUG-V7-03` | BASSA | A | Modelli locali | Il lock tra processi dei download è «controlla poi scrivi», non atomico |
| `BUG-V7-04` | BASSA | A | Modelli locali | «Riprendi» non trova la scelta di un modello con id rinominato |
| `BUG-V7-05` | BASSA | A | Modelli locali | Un file già presente è accettato per la sola dimensione e segnato come controllato |
| `BUG-V7-06` | BASSA | A | Setup | I file aggiunti a mano possono essere potati subito: la copia conserva la vecchia data |
| `BUG-V7-07` | BASSA | A | Modelli locali | Un token Hugging Face illeggibile blocca anche i modelli pubblici del catalogo |
| `BUG-V7-08` | BASSA | A | Modelli locali | Repo «gated» con un token senza accesso: errore HTTP 403 grezzo |
| `BUG-V7-35` | BASSA | A | i18n | Errori dei download dei modelli e del token HF solo in inglese |
| `BUG-V7-09` | MEDIA | B | Server web | `DocumentRoot`/`<Directory>` di Apache scritti senza escape: iniezione di direttive da un nome di cartella (Unix) |
| `BUG-V7-10` | BASSA | B | Progetti | Modalità LAN: ogni `127.0.0.1` nei comandi e nelle variabili diventa `0.0.0.0`, anche negli URL dei client |
| `BUG-V7-11` | BASSA | B | DB | Socket MySQL di riserva in `<TEMP>/Atollo/<id>`, la cartella che `caddy.rs` rifiuta perché creabile da altri |
| `BUG-V7-12` | BASSA | B | i18n | Errori non tradotti con il percorso di menu italiano «Strumenti» |
| `BUG-V7-13` | BASSA | B | Progetti | Il runner della creazione decodifica ogni blocco da 8 KiB da solo: caratteri UTF-8 spezzati |
| `BUG-V7-14` | BASSA | B | DB, server | Processi uccisi per timeout o errore mai raccolti (zombie su macOS/Linux) |
| `BUG-V7-38` | BASSA | B | i18n | La pagina di benvenuto generata da Apache è solo in italiano |
| `BUG-V7-15` | MEDIA | C | Editor | Mutex del server dell'editor tenuto per tutto l'avvio: i comandi sincroni bloccano la finestra |
| `BUG-V7-16` | BASSA | C | Agenti | La sessione «Collega con un token» non manda output e uscita alla sua finestra staccata |
| `BUG-V7-17` | BASSA | C | OS | La cartella temporanea dell'host ignora `TMPDIR` all'avvio, ma l'ambiente del kit lo passa |
| `BUG-V7-18` | BASSA | C | Core | L'id del kit ignora le maiuscole su ogni sistema, mentre su Linux la radice le distingue |
| `BUG-V7-19` | BASSA | C | OS | Il riconoscimento dei processi del kit ignora le regole dei percorsi per sistema e la radice reale |
| `BUG-V7-20` | BASSA | C | OS | Figli Unix mai raccolti (zombie) dopo le aperture staccate e gli arresti dell'editor |
| `BUG-V7-21` | BASSA | C | OS | Il socket d'istanza Unix legge senza timeout: una connessione ferma blocca le richieste successive |
| `BUG-V7-36` | BASSA | C | Prove | La prova i18n Rust salta le chiamate `t(lang, "...")` e quelle con parentesi nella lingua (45 chiamate) |
| `BUG-V7-22` | BASSA | D | Interfaccia | La rilettura dei framework sovrascrive le modifiche fatte durante `frameworks_get` |
| `BUG-V7-23` | MEDIA | D | Interfaccia | L'errore di «Salva e apri» / «Apri in…» su un terminale nuovo si perde |
| `BUG-V7-24` | BASSA | D | Interfaccia | Corsa nella ricerca Hugging Face: una risposta più lenta sostituisce i file del repo corrente |
| `BUG-V7-25` | MEDIA | D | Interfaccia | I pulsanti «Ferma» del pannello Attività inghiottono ogni errore |
| `BUG-V7-26` | BASSA | D | Interfaccia | Chiavi React duplicate nell'elenco dei passi del setup |
| `BUG-V7-27` | BASSA | D | Interfaccia | Pagina dei server: l'errore di una lettura fallita non si cancella più; letture non ordinate |
| `BUG-V7-28` | BASSA | D | Interfaccia | Letture fallite sostituite da valori di riserva senza errore (agenti, GitHub, versioni del progetto) |
| `BUG-V7-29` | BASSA | D | Interfaccia | Il pulsante dei terminali nella barra di stato non fa nulla se tutte le sessioni sono staccate |
| `BUG-V7-30` | BASSA | D | Interfaccia | La rilettura del log di un'azione può perdere righe arrivate durante `runLog` |
| `BUG-V7-31` | BASSA | D | Interfaccia | Pagina dei modelli: una lettura periodica in volo annulla lo stato appena impostato da un'azione |
| `BUG-V7-32` | BASSA | D | Interfaccia | La lettura fallita del catalogo dei componenti resta in cache per tutta la sessione |
| `BUG-V7-33` | BASSA | D | Interfaccia | Il divisore continua a trascinare dopo un puntatore annullato |
| `BUG-V7-34` | BASSA | D | Interfaccia | Commenti superati: «il terminale dell'app non si usa su Windows» |
| `BUG-V7-37` | BASSA | D | i18n | Segnaposto italiano «CHIAVE» mostrato in ogni lingua |
| `BUG-V7-39` | BASSA | D | Sito | Chiave `navDownload` inutilizzata nelle quattro stringhe del sito |

---

## 2. Gruppo A — Modelli locali e setup (`llm/`, `setup/downloads.rs`)

Il codice più nuovo (modelli GGUF, D-151) concentra i rilievi: la ripresa dei download si fida dell'elenco dei file salvato e rilegge il repository da capo, così due commit diversi finiscono nello stesso `.part`.

### `BUG-V7-01` [MEDIA] — La ripresa di un download Hugging Face scarica dal commit nuovo, non da quello del `.part`

- **Dove**: [`tauri/src/commands/llm.rs:234`](../../../tauri/src/commands/llm.rs#L234); [`tauri/src/llm/hf.rs:203`](../../../tauri/src/llm/hf.rs#L203) (`meta.sha`), [`tauri/src/llm/pull.rs:306`](../../../tauri/src/llm/pull.rs#L306) (`entry_for`), [`tauri/src/net/resume.rs:140`](../../../tauri/src/net/resume.rs#L140).
- **Problema**: «Riprendi» ([`ui/src/models/page.tsx:577`](../../../ui/src/models/page.tsx#L577)) richiama `llm_pull`, che rilegge il repository con `hf::inspect` e prende il commit **attuale**. `entry_for` ritrova la voce vecchia per nome di file e la restituisce invariata, con il commit e gli SHA vecchi. `download_resumable` calcola l'hash del `.part` esistente e chiede `Range: bytes=<offset>-` a `resolve/<commit nuovo>/<file>`.
- **Impatto**: Unsloth e bartowski ricaricano spesso i GGUF con lo stesso nome. Se il repository riceve un commit tra «Sospendi» e «Riprendi», i byte vecchi si uniscono a quelli del file nuovo: dopo diversi GB l'utente legge «il checksum non corrisponde, il file è stato cancellato» e riparte da zero. Se il file nuovo ha la stessa dimensione e passa, `atollo.json` dichiara il commit e lo SHA vecchi per un file diverso.
- **Correzione**: se esiste una voce incompleta con gli stessi file e commit o SHA diversi da `plan.commit`, cancellare i `.part` corrispondenti e aggiornare commit e file della voce in `entry_for`. Alternativa più semplice: riprendere dal commit della voce (`resolve/<entry.commit>`), che Hugging Face tiene raggiungibile.
- **Verifica**: confermato; il `.part` viene scartato solo se è più lungo della dimensione nuova (`resume.rs:143`), altrimenti l'errore emerge solo a fine download (`resume.rs:188-196`).

### `BUG-V7-02` [BASSA] — Lock e cartella del modello dal nome del repo così come è scritto

- **Dove**: [`tauri/src/llm/pull.rs:223`](../../../tauri/src/llm/pull.rs#L223); `local::folder_of` ([`tauri/src/llm/local.rs:72`](../../../tauri/src/llm/local.rs#L72)), `RepoInfo.repo` ([`tauri/src/llm/hf.rs:229`](../../../tauri/src/llm/hf.rs#L229)), `busy()` (`pull.rs:256`).
- **Problema**: la chiave di `PULLING` e il nome della cartella vengono dal testo digitato dall'utente, mai dall'`id` canonico che restituisce Hugging Face; `busy()` confronta con il nome della cartella su disco in modo esatto.
- **Impatto**: su un file system che non distingue le maiuscole (NTFS, exFAT, APFS: i dischi tipici del kit) un download dal catalogo di `Qwen/Qwen2.5-Coder-7B-Instruct-GGUF` e uno a mano di `qwen/qwen2.5-coder-7b-instruct-gguf:q4_k_m` passano entrambi `Pulling::take` e scrivono lo stesso `.part`. Una rimozione può cancellare un `.part` ancora in scrittura. Su Linux lo stesso repo finisce in due cartelle.
- **Correzione**: usare come `RepoInfo.repo` l'`id` canonico della risposta `/api/models`; indicizzare `PULLING` con una chiave piegata come `os::processes::path_key` (senza maiuscole su Windows e macOS) e usarla anche in `busy()` e `cancel()`.
- **Verifica**: gravità abbassata da MEDIA a BASSA: serve lo stesso repo in due grafie nello stesso momento.

### `BUG-V7-03` [BASSA] — Il lock tra processi dei download è «controlla poi scrivi», non atomico

- **Dove**: [`tauri/src/llm/pull.rs:228`](../../../tauri/src/llm/pull.rs#L228).
- **Problema**: `locked_elsewhere(dir)` legge `.pulling`, poi `std::fs::write` lo sovrascrive col proprio pid, senza creazione esclusiva né lock del sistema. Il mutex interno non copre la riga di comando (`atollo llm pull`) che gira insieme all'app. Il `Drop` cancella il file senza controllare che contenga ancora il proprio pid.
- **Impatto**: app e CLI che partono quasi insieme sulla stessa cartella scrivono entrambe nello stesso `.part` e finiscono con un checksum sbagliato; il primo che finisce toglie il lock su cui conta l'altro. Gli installatori hanno già un mutex con nome o `flock` (v6, §2); i modelli no.
- **Correzione**: creare `.pulling` con `OpenOptions::create_new(true)`; su `AlreadyExists` leggere il pid e sostituire il file solo se quel processo è morto, poi riprovare. Meglio ancora un lock esclusivo (`LockFileEx`/`flock`) tenuto per tutto il download, come fa il lock di `prepare`. Nel `Drop`, cancellare solo se il file contiene il proprio pid.

### `BUG-V7-04` [BASSA] — «Riprendi» non trova la scelta di un modello con id rinominato

- **Dove**: [`tauri/src/llm/hf.rs:243`](../../../tauri/src/llm/hf.rs#L243) (`pick`); [`tauri/src/llm/pull.rs:316`](../../../tauri/src/llm/pull.rs#L316).
- **Problema**: in caso di conflitto `entry_for` rinomina l'id in `slug("{id}-{owner}")`. L'interfaccia riprende sempre con `choice: m.id` ([`ui/src/models/page.tsx:577`](../../../ui/src/models/page.tsx#L577)); `pick` accetta solo `c.quant` o `c.id` (= `slug(stem)`, senza il suffisso del proprietario).
- **Impatto**: ogni download sospeso con id rinominato fallisce su «Riprendi» con «no `<id>-<owner>`; there are …». L'unica via è cancellarlo e riscaricarlo.
- **Correzione**: riprendere dalla voce del manifest: `llm_pull` cerca la voce incompleta per id e sceglie la variante con gli stessi nomi di file. In alternativa, `pick` accetta anche `slug("{c.id}-{owner}")`.
- **Verifica**: confermato e più ampio del previsto: succede qualunque sia la quantizzazione.

### `BUG-V7-05` [BASSA] — Un file già presente è accettato per la sola dimensione e segnato come controllato

- **Dove**: [`tauri/src/llm/pull.rs:390`](../../../tauri/src/llm/pull.rs#L390).
- **Problema**: `if dest.is_file() && … m.len() == f.bytes { … continue; }`, poi `mark_complete` e la riga di log «downloaded and checked», senza calcolare lo SHA-256.
- **Impatto**: un `.gguf` messo a mano in una cartella `owner--repo` (D-151 li tratta come modelli) o rimasto da un commit precedente salta il controllo d'integrità se ha la dimensione giusta, e il manifest lo registra come completo e verificato.
- **Correzione**: saltare per dimensione solo se la voce riusata era già completa con lo stesso SHA; altrimenti calcolare l'hash di `dest` contro `f.sha256` e riscaricare se non corrisponde.
- **Verifica**: solo integrità, nessun vantaggio per chi non ha già accesso in scrittura alla cartella.

### `BUG-V7-06` [BASSA] — I file aggiunti a mano possono essere potati subito

- **Dove**: [`tauri/src/setup/downloads.rs:234`](../../../tauri/src/setup/downloads.rs#L234) (`add`), `prune` alle righe 350-364.
- **Problema**: `add` copia con `fs::copy`, che su Windows (`CopyFileExW`) e macOS (`copyfile`) conserva la data di modifica della sorgente. `prune`, chiamato all'inizio di ogni `add`, cancella i file più vecchi di 30 giorni (`KEEP`).
- **Impatto**: un installatore rimasto in Download da più di 30 giorni, aggiunto per un componente fallito, viene cancellato al primo `add` successivo, prima che un'installazione lo usi; il componente fallisce di nuovo e chiede lo stesso file.
- **Correzione**: dopo `fs::copy`, aprire il `.part` e chiamare `File::set_modified(SystemTime::now())` prima del `rename`.

### `BUG-V7-07` [BASSA] — Un token Hugging Face illeggibile blocca anche i modelli pubblici del catalogo

- **Dove**: [`tauri/src/commands/llm.rs:228`](../../../tauri/src/commands/llm.rs#L228); lo stesso in [`tauri/src/llm/cli.rs:96`](../../../tauri/src/llm/cli.rs#L96).
- **Problema**: `hf_token(&kit)` è letto e propagato con `?` prima di distinguere catalogo e repository. `read_secret` dà errore d'accesso su un file «solo proprietario» creato da un altro account o PC (`PORT-V6-01`).
- **Impatto**: un kit portato su un altro account Windows non scarica nessun modello del catalogo, anche se sono pubblici e non serve un token; l'errore parla del file del token.
- **Correzione**: leggere il token solo nel ramo dei repository Hugging Face, oppure su errore di lettura scrivere un avviso nel log e continuare con `None` (un repo «gated» fallisce poi con `HfError::Gated`, che spiega il problema).

### `BUG-V7-08` [BASSA] — Repo «gated» con un token senza accesso: errore HTTP 403 grezzo

- **Dove**: [`tauri/src/llm/hf.rs:211`](../../../tauri/src/llm/hf.rs#L211).
- **Problema**: `inspect` rifiuta con `Gated` solo se il token manca. Con un token che non ha accettato la licenza, metadati ed elenco dei file si leggono; il download riceve 403, che `is_permanent` tratta come definitivo. La traduzione in `Gated` esiste solo in `hf::get` (riga 191).
- **Impatto**: l'utente vede «GET https://huggingface.co/…: http status: 403» dopo il controllo del disco, invece del messaggio «serve un token Hugging Face con accesso».
- **Correzione**: in `pull()`, tradurre 401/403 da `download_resumable` su un URL `huggingface.co` in `HfError::Gated(plan.repo)`; in alternativa, per un repo «gated», una richiesta `HEAD` sul primo file in `inspect`.

### `BUG-V7-35` [BASSA] — Errori dei download dei modelli e del token HF solo in inglese

- **Dove**: [`tauri/src/llm/pull.rs:224`](../../../tauri/src/llm/pull.rs#L224) («a model of this repository is already being downloaded»), `pull.rs:315` e `:320` («a model called {id} is already in the kit»), [`tauri/src/commands/llm.rs:366`](../../../tauri/src/commands/llm.rs#L366) («not a Hugging Face token»).
- **Problema**: i tre rifiuti arrivano all'interfaccia con `format!("{e:#}")` (`commands/llm.rs:286-289`). L'interfaccia abilita il download per chiave (`<cartella>/<id>`, [`ui/src/models/page.tsx:208`](../../../ui/src/models/page.tsx#L208)), mentre `Pulling::take` rifiuta per cartella: il primo caso è raggiungibile dall'interfaccia.
- **Impatto**: una seconda quantizzazione dello stesso repository, un id in conflitto o un token con un carattere di troppo mostrano una frase inglese in un'interfaccia italiana, francese o tedesca. Il resto del flusso (`HfError::message`, gli errori di `service.rs`) è tradotto.
- **Correzione**: passare `lang` a `Pulling::take` e alla chiusura del conflitto di id, avvolgere i tre messaggi in `crate::i18n::t` con nuove voci in `tauri/i18n/catalog.json`; in `llm_hf_token_set` usare `tr(&state, "Token di Hugging Face non valido.", &[])`. Facoltativo: disabilitare le altre quantizzazioni di un repository mentre una è in `view.pulling`.
- **Verifica**: gravità abbassata; la regola D-105 (testi tecnici in inglese nella catena `anyhow`) vale per gli errori del sistema, non per questi rifiuti rivolti all'utente. Le righe 348, 367 e 374 sono tecniche e restano in inglese.

---

## 3. Gruppo B — Progetti, database e server (`projects/`, `db/`, `servers/`, `install/web/`)

> Il merge `c256fb6` (copie di sicurezza dei database, D-154) è arrivato dopo la revisione e tocca `db/services/`: prima di correggere `BUG-V7-11` e `BUG-V7-14` rileggere le righe citate.

### `BUG-V7-09` [MEDIA] — `DocumentRoot`/`<Directory>` di Apache scritti senza escape (Unix)

- **Dove**: [`tauri/src/projects/apache.rs:376`](../../../tauri/src/projects/apache.rs#L376); lo stesso in [`tauri/src/servers/config.rs:60`](../../../tauri/src/servers/config.rs#L60) (`apache_vhost_conf`).
- **Problema**: `format!("DocumentRoot \"{root}\"")` e `format!("<Directory \"{root}\">")` con `root` preso dalla cartella `apache` di `panel.json`. `detect::overrides::inside()` rifiuta solo percorsi assoluti, `..`, cartelle mancanti e collegamenti; su Unix un nome con `"` o un a capo passa. Caddy rifiuta gli stessi percorsi (`caddy.rs:91-97`, `quoted_path`), Apache no.
- **Impatto**: su macOS/Linux un repository clonato può contenere una cartella con `"` e un a capo nel nome (git li permette) e indicarla in `panel.json`: l'`httpd.conf` del progetto, o il vhost di un pool condiviso, riceve direttive arbitrarie (`CustomLog "|cmd"`, `LoadModule`, `Include`) eseguite all'avvio di Apache, anche per un sito statico, e in un pool toccano i siti degli altri progetti. Con le sole virgolette, Apache del pool non parte.
- **Correzione**: una controparte Apache di `caddy::quoted_path` che rifiuti `"`, `\` (escape dentro gli argomenti tra virgolette), CR, LF e `${`; usarla in `project_conf_with_site` e `apache_vhost_conf`. Percorso rifiutato: errore per un server dedicato, sito saltato con avviso nel log in un pool (come `config.rs:194-197` per Caddy). Controllare anche le destinazioni scritte da `alias_directives`.
- **Verifica**: confermato; non è un'escalation di privilegi (il sito PHP esegue già il codice del repository), ma raggiunge direttive eseguite all'avvio e può fermare un pool condiviso.

### `BUG-V7-10` [BASSA] — Modalità LAN: ogni `127.0.0.1` diventa `0.0.0.0`, anche negli URL dei client

- **Dove**: [`tauri/src/projects/run.rs:1246`](../../../tauri/src/projects/run.rs#L1246) (`on_network`), applicata al comando (riga 638) e a ogni valore di `action.env` (riga 672).
- **Problema**: con «Permetti ai server dei progetti…» attivo, la sostituzione vale per tutto il testo, anche per le azioni scritte dall'utente (D-136). DECISIONS dice che solo «i comandi trovati dall'app» passano a `0.0.0.0`.
- **Impatto**: `DATABASE_URL=mysql://u:p@127.0.0.1:3306/db`, `REDIS_URL=redis://127.0.0.1:6379` o un `curl http://127.0.0.1:…` diventano `0.0.0.0`; su Windows la connessione a `0.0.0.0` fallisce (`WSAEADDRNOTAVAIL`): l'app perde il database solo in modalità LAN, senza indizi.
- **Correzione**: limitare la sostituzione all'host di ascolto che inseriscono i rilevatori, per esempio con un segnaposto `{listen_host}` risolto da `fill()`; lasciare invariati comandi e variabili scritti a mano. Aggiornare `test/rust/projects/run.rs`.

### `BUG-V7-11` [BASSA] — Socket MySQL di riserva in `<TEMP>/Atollo/<id>`

- **Dove**: [`tauri/src/db/services/data.rs:240`](../../../tauri/src/db/services/data.rs#L240).
- **Problema**: quando il percorso dei dati supera il limite del socket, la cartella di riserva è `kit.system_temp()/mysql`, creata con `create_dir_all` e resa privata solo con `chmod 0700` sulla foglia, senza controllo del proprietario, dei collegamenti o del genitore. `caddy.rs:208-214` evita apposta la stessa cartella perché «un altro utente può averla creata prima».
- **Impatto**: su Linux (`/tmp/Atollo/<id>`), se un altro utente ha creato prima `/tmp/Atollo` o la foglia, `chmod` fallisce e MySQL non parte; con il genitore scrivibile la cartella del socket può essere scambiata dopo il `chmod`. Su macOS il `TMPDIR` per utente è di solito privato.
- **Correzione**: scegliere la cartella come fa `caddy.rs` (`socket_bases`/`socket_parent`/`private_leaf`, con controllo di proprietario e sticky bit) e tenerne la guardia per tutta la vita del server; come minimo verificare proprietario, assenza di collegamenti e permessi del genitore e rifiutare l'avvio se il controllo fallisce.

### `BUG-V7-12` [BASSA] — Errori non tradotti con il percorso di menu italiano

- **Dove**: [`tauri/src/projects/apache.rs:474`](../../../tauri/src/projects/apache.rs#L474) («Apache is not installed (Strumenti > Apache)»), [`tauri/src/projects/caddy.rs:835`](../../../tauri/src/projects/caddy.rs#L835) (lo stesso per Caddy).
- **Problema**: nessuno dei due passa da `crate::i18n::t`, mentre `run.rs` traduce gli altri messaggi del runner (`run.rs:408`, `469`, `726`) e li avvolge in «Avvio non riuscito: {0}».
- **Impatto**: in ogni lingua l'utente legge una frase inglese con un nome di menu italiano, che può non coincidere con l'etichetta della sua interfaccia. Succede quando il componente è stato tolto dopo aver configurato l'azione.
- **Correzione**: passare `lang` a `apache::command` e `caddy::command` (o tradurre l'errore in `run.rs`) con `tr(lang, "Apache non è installato (Strumenti > Apache)", &[])` e le voci in `tauri/i18n/catalog.json`.

### `BUG-V7-13` [BASSA] — Il runner della creazione decodifica ogni blocco da 8 KiB da solo

- **Dove**: [`tauri/src/projects/frameworks/runner.rs:120`](../../../tauri/src/projects/frameworks/runner.rs#L120).
- **Problema**: `String::from_utf8_lossy(&buf[..n])` su ogni lettura: un carattere multibyte diviso tra due letture diventa U+FFFD. `run.rs:870-885` gestisce lo stesso caso con un buffer della coda incompleta.
- **Impatto**: nell'output di composer, npm o `create-*` (forzati in UTF-8) compaiono caratteri di sostituzione su lettere accentate, disegni di riquadri ed emoji; l'ultima riga, usata come messaggio d'errore, può uscire rovinata.
- **Correzione**: tenere i byte pendenti come in `run.rs` (`valid_up_to()` quando `error_len()` è `None`) e mettere la logica in un aiuto comune ai due lettori.

### `BUG-V7-14` [BASSA] — Processi uccisi per timeout o errore mai raccolti (zombie)

- **Dove**: [`tauri/src/db/services/tools.rs:99`](../../../tauri/src/db/services/tools.rs#L99) (`run_tool`), `tools.rs:132-135` (`run_logged`), `db/web_admin.rs:166-171` (`wait_ready`), `servers/process.rs:564-571` (ramo `!ready` di `start_locked`).
- **Problema**: `child.kill()` senza `child.wait()`; nel ramo `!ready` dei server il figlio non viene nemmeno ucciso e resta affidato al `Drop` del Job (`os/jobs.rs:130-145`, `killpg`), che non raccoglie il capo del gruppo. `run.rs` e i pool usano `end()`, che aspetta.
- **Impatto**: ogni `mysqladmin`/`mongosh` andato in timeout, ogni avvio fallito di `php -S` o dell'amministrazione web e ogni pool che non ascolta lascia uno zombie (su Windows un handle) fino all'uscita dell'app; i guasti ripetuti li accumulano.
- **Correzione**: `let _ = child.wait();` dopo `kill()` in `run_tool`, `run_logged` e `wait_ready`; in `start_locked`, su `!ready`, `kill()` e `wait()` prima del `bail!`.

### `BUG-V7-38` [BASSA] — La pagina di benvenuto generata da Apache è solo in italiano

- **Dove**: [`tauri/src/install/web/apache.rs:349`](../../../tauri/src/install/web/apache.rs#L349) e righe 361-362.
- **Problema**: in un `htdocs` vuoto si scrive `<h1>Apache funziona</h1>` e «I file del sito stanno nella cartella … del kit», qualunque sia la lingua dell'interfaccia.
- **Impatto**: chi non usa l'italiano apre il server Apache e trova una pagina italiana; è l'unico testo generato per l'utente che salta il catalogo.
- **Correzione**: una pagina neutra (`<h1>Apache</h1>` e il percorso in `<code>`, senza frase) oppure passare `lang` al passo d'installazione e usare `crate::i18n::t`. La pagina si scrive una volta sola: resterebbe nella lingua attiva al momento dell'installazione.

---

## 4. Gruppo C — Core, editor e sistema (`editor/`, `agents/`, `os/`, `kit.rs`, `main.rs`)

Tema comune: su macOS e Linux le regole dei percorsi e dei processi non seguono quelle già decise (D-123, D-90), e alcuni figli non vengono mai raccolti.

### `BUG-V7-15` [MEDIA] — Mutex del server dell'editor tenuto per tutto l'avvio

- **Dove**: [`tauri/src/editor/mod.rs:144`](../../../tauri/src/editor/mod.rs#L144) (`ensure_running`).
- **Problema**: `ensure_running` tiene il lock di `server` mentre `self.start()` prova fino a 3 porte con `START_TIMEOUT` di 30 s ciascuna. `status()` prende lo stesso mutex ed è chiamato sul thread principale da `editor_status` (comando sincrono, [`tauri/src/commands/editing.rs:7`](../../../tauri/src/commands/editing.rs#L7), interrogato ogni 5 s da `ui/src/App.tsx:119`), da `open_now` ([`tauri/src/commands/window.rs:43`](../../../tauri/src/commands/window.rs#L43)) per `eject_list` e per la chiusura della finestra, anche dal «Esci» del vassoio. Anche `sync_extensions` (riga 168) prende lo stesso lock.
- **Impatto**: durante un avvio lento dell'editor (primo avvio, disco lento, una porta che non risponde) la finestra intera si blocca fino a circa 90 s: l'interrogazione periodica, il pulsante di chiusura e «Esci» dal vassoio aspettano il mutex.
- **Correzione**: serializzare gli avvii con un `starting: Mutex<()>` separato (o uno stato `Starting/Running`) e prendere `server` solo per controllare, svuotare e salvare; in `status()` usare `try_lock` e rispondere `running: false` (o «in avvio») su `WouldBlock`. In alternativa rendere asincroni `editor_status` ed `eject_list` con `spawn_blocking`.

### `BUG-V7-16` [BASSA] — La sessione «Collega con un token» non parla alla sua finestra staccata

- **Dove**: [`tauri/src/agents/claude_token.rs:281`](../../../tauri/src/agents/claude_token.rs#L281) e riga 293.
- **Problema**: `link()` emette output e uscita solo a `main`; ogni altra sessione ([`tauri/src/editor/terminal.rs:86`](../../../tauri/src/editor/terminal.rs#L86)) emette anche a `terminal-<id>`. `pty_detach` accetta qualunque id e il pannello mostra «Sposta in una nuova finestra» per ogni sessione ([`ui/src/terminal/panel.tsx:72`](../../../ui/src/terminal/panel.tsx#L72)).
- **Impatto**: se l'utente sposta la sessione `claude setup-token` in una finestra sua (D-149), quella finestra non riceve più l'output dopo lo storico e non sa che il programma è finito: l'URL d'accesso e la fine del flusso non si vedono.
- **Correzione**: un aiuto comune in `editor::terminal` che emette a `main` e a `terminal::label(id)` per output ed `EXIT_EVENT`, usato anche da `claude_token::link` (oppure nascondere «Sposta» per questa sessione).

### `BUG-V7-17` [BASSA] — La cartella temporanea dell'host ignora `TMPDIR` all'avvio

- **Dove**: [`tauri/src/main.rs:97`](../../../tauri/src/main.rs#L97); [`tauri/src/env.rs:414`](../../../tauri/src/env.rs#L414).
- **Problema**: `main.rs` passa a `with_host` `CP_HOST_TEMP`, `TEMP` e `TMP`, ma non `TMPDIR`; `env.rs` invece include `TMPDIR` quando imposta `CP_HOST_TEMP`.
- **Impatto**: su macOS/Linux avviati da Finder o dal desktop c'è solo `TMPDIR`, quindi `system_temp()` è `None` e D-90 non si applica: download ed estrazioni vanno in `data/tmp` sulla chiavetta (limite di 4 GiB su FAT32/exFAT, lentezza), e all'uscita `stop_children` e `cleanup_at_exit` sono saltati (`main.rs:362`). Avviato da una console del kit, Atollo riceve `CP_HOST_TEMP=$TMPDIR` e si comporta in modo diverso.
- **Correzione**: un solo aiuto per la cartella temporanea dell'host nei due punti. Se si aggiunge `TMPDIR`, creare `<tmp>/Atollo/<id>` in `0700` e controllarne il proprietario (su Linux `/tmp` è condivisa); altrimenti togliere `TMPDIR` da `env.rs`. A parte: non legare `stop_children` a `system_temp`.

### `BUG-V7-18` [BASSA] — L'id del kit ignora le maiuscole su ogni sistema

- **Dove**: [`tauri/src/kit.rs:508`](../../../tauri/src/kit.rs#L508) (`Kit::id`).
- **Problema**: l'id è lo SHA-256 della radice in minuscolo su ogni sistema, mentre `same_root` (`kit.rs:589-601`) tratta come diverse su Linux `/media/u/Kit` e `/media/u/kit`. L'id dà il nome al socket astratto Linux `Atollo-{uid}-{id}` ([`tauri/src/os/instance.rs:152`](../../../tauri/src/os/instance.rs#L152)) e a `<TEMP>/Atollo/<id>`.
- **Impatto**: due kit su Linux con radici diverse solo per le maiuscole condividono l'id: il secondo non apre il socket e ripiega su TCP, un secondo avvio del kit B porta in primo piano il kit A; con la cartella temporanea comune `TempWork::at` cancella il lavoro in corso dell'altro kit (`os/temp.rs:53`).
- **Correzione**: ignorare le maiuscole in `id()` solo su Windows (o con la semantica di `path_key`), tenendo i valori già salvati (l'id del sigillo è fissato da `SEAL_ID_FILE`).

### `BUG-V7-19` [BASSA] — Il riconoscimento dei processi del kit ignora le regole dei percorsi per sistema

- **Dove**: [`tauri/src/os/sessions.rs:47`](../../../tauri/src/os/sessions.rs#L47) (`kit_processes`), righe 329-335 (`other_kits`), riga 152 (`stop_children`).
- **Problema**: `kit_processes` e `other_kits` confrontano con `to_lowercase()` invece di `processes::path_key`, che D-123 vuole esatto su Linux; `stop_children` filtra con `p.path.starts_with(kit.root())` senza `kit_roots`.
- **Impatto**: su Linux «Rimuovi l'unità» può fermare un programma di un altro kit la cui radice differisce solo per le maiuscole. Su macOS, con il kit raggiunto da un collegamento (`/var` → `/private/var`), i figli hanno un percorso reale fuori da `kit.root()` e `stop_children` non ne ferma nessuno.
- **Correzione**: costruire i prefissi con `processes::prefix_of`/`path_key` in `kit_processes` e `other_kits`; in `stop_children` confrontare con `kit_roots(kit)` (ogni radice e la cartella della webview passate da `path_key`).
- **Verifica**: confermato in parte; `stop_children` gira solo con `system_temp` impostato (vedi `BUG-V7-17`).

### `BUG-V7-20` [BASSA] — Figli Unix mai raccolti dopo le aperture staccate e gli arresti dell'editor

- **Dove**: [`tauri/src/os/jobs.rs:266`](../../../tauri/src/os/jobs.rs#L266) (`spawn_detached`), riga 284 (`open_with_explorer`); [`tauri/src/editor/mod.rs:201`](../../../tauri/src/editor/mod.rs#L201) e riga 312.
- **Problema**: il `Child` restituito da `spawn_detached` viene scartato senza `wait()`; nessuno ignora `SIGCHLD` (l'unico `SIG_IGN` è nella guardia, `guard.rs:289`). L'arresto dell'editor e il timeout di `start_on` chiamano `kill()` senza `wait()`. Anche la voce in `OWNERS` resta.
- **Impatto**: su macOS/Linux ogni link o cartella aperta (`open`/`xdg-open` escono subito) e ogni riavvio o timeout dell'editor lascia uno zombie fino all'uscita dell'app; una sessione lunga li accumula.
- **Correzione**: `wait()` dopo `kill()` (per l'editor, in un thread se serve); in `spawn_detached` un thread che aspetta il figlio e restituisce solo il pid, oppure un doppio `fork` così che `init` raccolga il processo.

### `BUG-V7-21` [BASSA] — Il socket d'istanza Unix legge senza timeout

- **Dove**: [`tauri/src/os/instance.rs:456`](../../../tauri/src/os/instance.rs#L456).
- **Problema**: `listen()` legge con `read_to_end` sul suo unico thread senza `set_read_timeout`, a differenza del ripiego TCP (riga 311, 2 s).
- **Impatto**: una connessione dello stesso utente che resta aperta senza scrivere (un secondo avvio fermato con `SIGSTOP`, uno strumento locale) blocca per sempre il thread: gli avvii successivi scrivono «focus», escono con 0 e la finestra non viene mai portata davanti.
- **Correzione**: `stream.set_read_timeout(Some(Duration::from_secs(2)))` dopo `same_user` e prima della lettura, come nel percorso TCP.

### `BUG-V7-36` [BASSA] — La prova i18n Rust salta 45 chiamate

- **Dove**: [`test/rust/i18n.rs:141`](../../../test/rust/i18n.rs#L141).
- **Problema**: l'espressione regolare vuole `tr(` o `i18n::t(` e un argomento lingua senza parentesi: non vede `t(lang, "predefinita", &[])` (`cli.rs:284` e altre 23 chiamate tra `cli.rs`, `llm/cli.rs`, `os/volume.rs`, `os/webview.rs`, `ports.rs`, `router/availability.rs`, `setup/task.rs`) né `crate::i18n::t(state.lang(), …)` (`commands/setup.rs:483`, `commands/terminals.rs:89`).
- **Impatto**: oggi i 45 testi sono tutti nel catalogo, ma un testo nuovo o modificato nella CLI, nella finestra «webview mancante», nei messaggi di volumi e porte o nei selettori di file resterebbe in italiano senza che `sources_are_in_the_catalog` fallisca.
- **Correzione**: allargare l'espressione a `\b(?:tr|t|i18n::t)\(\s*(?:"[^"]*"|[^,"]+?),\s*"((?:[^"\\]|\\.)*)"`, controllando che il `t(` nudo non prenda funzioni estranee a due argomenti; in alternativa contare un minimo di chiamate per `cli.rs`, `llm/cli.rs` e `os/webview.rs`.

---

## 5. Gruppo D — Interfaccia (`ui/src/`, `site/strings/`)

Tema comune: errori inghiottiti e letture concorrenti senza numero di sequenza. Le pagine che usano `useLoad`/`useAction` sono corrette; i rilievi sono nelle pagine che leggono a mano.

### `BUG-V7-22` [BASSA] — La rilettura dei framework sovrascrive le modifiche in corso

- **Dove**: [`ui/src/frameworks/model.ts:211`](../../../ui/src/frameworks/model.ts#L211) (`load`).
- **Problema**: `load()` registra `at = edits.current` ma applica comunque `setPieces`, `latest`, `savedEdits` e `dirty=false` anche se nel frattempo `edits.current` è andato avanti. `save()` chiama `load()` dopo ogni salvataggio.
- **Impatto**: un tasto premuto durante il giro di `frameworks_get` sparisce dallo schermo e, con `dirty=false`, non viene mai salvato; nessun avviso.
- **Correzione**: nel ramo di successo, `if (edits.current !== at) { setView(v); return; }`.
- **Verifica**: gravità BASSA, la finestra è un IPC locale breve, ma la perdita è silenziosa.

### `BUG-V7-23` [MEDIA] — L'errore di «Salva e apri» / «Apri in…» su un terminale nuovo si perde

- **Dove**: [`ui/src/terminals.tsx:585`](../../../ui/src/terminals.tsx#L585); `persist()` alle righe 163-178.
- **Problema**: `open()` fa `await persist(); await terminalLaunch(id, dir);`, ma `persist()` chiama `onSaved(id)`, che fa `setDraft(null); setChosen(id); reload();`. La chiave dell'editor passa da `new:<nome>` a `current.id`; il nuovo id non è ancora nell'elenco, `current` ripiega su `terminals[0]` e il `TerminalEditor` che esegue l'azione viene smontato prima che `terminalLaunch` finisca. `useAction` scarta i risultati dopo lo smontaggio (`hooks.ts:156-165`).
- **Impatto**: un avvio fallito non mostra nulla e l'indicatore di attesa sparisce; l'editor del terminale predefinito lampeggia finché arriva la rilettura.
- **Correzione**: `persist()` restituisce l'id senza chiamare `onSaved`; `open()` chiama `onSaved(id)` in un `finally` dopo `terminalLaunch`, e l'errore si mostra in `TerminalsPage`; oppure tenere la bozza salvata come `current` finché l'id non compare nell'elenco.

### `BUG-V7-24` [BASSA] — Corsa nella ricerca Hugging Face

- **Dove**: [`ui/src/models/page.tsx:594`](../../../ui/src/models/page.tsx#L594).
- **Problema**: `run("hf", () => llmHfInspect(repo.trim()).then(setFound))` senza controllo di sequenza.
- **Impatto**: se arriva per ultima la risposta di una ricerca precedente, l'elenco mostra le varianti GGUF del repo A sotto il nome B, e «Scarica» scarica da A.
- **Correzione**: un contatore in un `ref`: `const id = ++hfSeq.current; … if (id === hfSeq.current) setFound(f);`.
- **Verifica**: scenario ristretto; mentre `busy === "hf"` il pulsante e l'invio sono disabilitati, quindi serve un'altra azione della pagina (download, rimozione) che cambi `busy` durante la ricerca.

### `BUG-V7-25` [MEDIA] — I pulsanti «Ferma» del pannello Attività inghiottono ogni errore

- **Dove**: [`ui/src/activity.tsx:352`](../../../ui/src/activity.tsx#L352) (`void onStop(…)`), righe 388 (`serviceStop`), 413 (`llmStop`), 435 (`serverStop`), 198-208 (`stopAll`).
- **Problema**: `() => undefined` sui rifiuti, `void` su `runStop` (da `App.tsx:289`, senza `catch`), `Promise.allSettled` che ignora i `rejected`. Il pannello non usa `useAction` né `ErrorLine`.
- **Impatto**: un arresto fallito (processo in uso, permessi, porta occupata) non mostra nulla: la riga resta «in esecuzione» e «Ferma tutto» non dice niente; i rifiuti di `runStop` diventano promise rejection non gestite.
- **Correzione**: `useAction` nel pannello con un `ErrorLine`; in `stopAll` raccogliere i motivi dei `rejected` come errore; alla riga 388 chiamare anche `reloadServices()` dopo `setServices`.

### `BUG-V7-26` [BASSA] — Chiavi React duplicate nell'elenco dei passi del setup

- **Dove**: [`ui/src/setup.tsx:198`](../../../ui/src/setup.tsx#L198).
- **Problema**: `key={step.id}`, ma `setup/runner.rs` emette più passi con lo stesso id componente (rimozione e installazione, rimozioni per altri target, patch per riga come MySQL 8.4 e 9.7).
- **Impatto**: React avvisa e può riusare o perdere l'`<li>` sbagliato mentre i passi si aggiornano: avanzamento ed esito possono comparire sul passo sbagliato.
- **Correzione**: `key={`${String(i)}:${step.id}`}` con l'indice della mappa, dato che i passi si aggiungono soltanto.

### `BUG-V7-27` [BASSA] — Pagina dei server: errore permanente e letture non ordinate

- **Dove**: [`ui/src/servers.tsx:307`](../../../ui/src/servers.tsx#L307).
- **Problema**: il ramo di successo fa solo `setView` e non `setError(null)`; letture ogni 3 s e `refreshFresh` non hanno sequenza; lo stato `error` è condiviso con le azioni.
- **Impatto**: un guasto passeggero lascia il riquadro d'errore finché l'utente non esegue un'azione (le pagine con `useLoad` lo tolgono alla lettura successiva riuscita); una lettura vecchia che arriva dopo può riportare uno stato superato dei pool.
- **Correzione**: passare a `useLoad((…) => serversList(fresh), null, { onError: "show", every: 3000 })` con la sua sequenza e tenere l'errore delle azioni in uno stato separato.

### `BUG-V7-28` [BASSA] — Letture fallite sostituite da valori di riserva senza errore

- **Dove**: [`ui/src/agents/page.tsx:47`](../../../ui/src/agents/page.tsx#L47); [`ui/src/github/account.tsx:25`](../../../ui/src/github/account.tsx#L25); [`ui/src/versions/project.tsx:35`](../../../ui/src/versions/project.tsx#L35).
- **Problema**: `onError: { fallback: [] }` con `every: 5000` per gli agenti; `fallback: null` per lo stato GitHub (che poi mostra «Caricamento…») e per le versioni del progetto, senza mostrare l'errore.
- **Impatto**: una lettura fallita degli agenti svuota l'elenco: la pagina passa al benvenuto vuoto e i pulsanti degli agenti nel progetto spariscono per 5 s. Un `githubStatus` fallito mostra «Caricamento…» per sempre, senza riprovare. Le versioni del progetto appaiono vuote, come se non ci fosse niente da configurare.
- **Correzione**: agenti `onError: "keep"`; stato GitHub e versioni del progetto `onError: "show"` con l'errore e un pulsante «Riprova».

### `BUG-V7-29` [BASSA] — Il pulsante dei terminali nella barra di stato non fa nulla con sessioni solo staccate

- **Dove**: [`ui/src/App.tsx:585`](../../../ui/src/App.tsx#L585); `terminal/state.ts:109`.
- **Problema**: il conteggio usa `terminals.sessions` (staccate incluse), mentre `state.ts` restituisce `shown && docked.length > 0`.
- **Impatto**: con sole sessioni staccate il pulsante mostra un numero e «Mostra i terminali», ma il clic non cambia nulla e `aria-pressed` resta `false`: un controllo morto, anche per i lettori di schermo.
- **Correzione**: conteggio e stato premuto da `terminals.docked`; se `docked` è vuoto ma ci sono sessioni, nascondere il pulsante o fargli portare davanti (o riagganciare) le finestre staccate.

### `BUG-V7-30` [BASSA] — La rilettura del log di un'azione può perdere righe

- **Dove**: [`ui/src/projects/runs.ts:32`](../../../ui/src/projects/runs.ts#L32).
- **Problema**: `loadLog` sostituisce `logs[key]` con la fotografia di `runLog`, mentre gli eventi `run` aggiungono righe; risposte e eventi viaggiano su canali diversi.
- **Impatto**: le righe emesse dopo la fotografia ma arrivate prima della risposta vengono aggiunte e poi cancellate dalla fotografia più vecchia; mancano dal log di Attività fino alla rilettura successiva. `loadLog` gira a ogni cambio di selezione del pannello.
- **Correzione**: tenere da parte gli eventi di `key` arrivati mentre `runLog` è in volo e impostare fotografia + righe tenute; oppure numerare le righe e unire per numero.

### `BUG-V7-31` [BASSA] — Pagina dei modelli: una lettura in volo annulla lo stato appena impostato

- **Dove**: [`ui/src/models/page.tsx:169`](../../../ui/src/models/page.tsx#L169) (`onInfo`).
- **Problema**: `setInfo(i)` non invalida la lettura di `llmStatus` (ogni 3000 ms) già partita; la pagina dei database chiama `refresh()` dopo `setList` proprio per questo.
- **Impatto**: dopo «Avvia», una lettura partita prima risponde «fermo»: intestazione e pulsante tornano su «Avvia» per fino a 3 s e invitano a un secondo clic.
- **Correzione**: dopo `setInfo(i)` chiamare `reloadInfo()` (la sequenza scarta la lettura vecchia); lo stesso in `activity.tsx:388` con `reloadServices()`.

### `BUG-V7-32` [BASSA] — La lettura fallita del catalogo dei componenti resta in cache

- **Dove**: [`ui/src/frameworks/model.ts:284`](../../../ui/src/frameworks/model.ts#L284).
- **Problema**: `catalog ??= componentsList().catch(() => [])` a livello di modulo conserva il vuoto per tutta la sessione.
- **Impatto**: se la prima `components_list` fallisce (primo avvio, guasto passeggero), le etichette dei framework perdono nomi e loghi fino al riavvio dell'app.
- **Correzione**: `catalog ??= componentsList().catch(() => { catalog = null; return [] as ComponentInfo[]; });`.

### `BUG-V7-33` [BASSA] — Il divisore continua a trascinare dopo un puntatore annullato

- **Dove**: [`ui/src/workbench/layout.tsx:137`](../../../ui/src/workbench/layout.tsx#L137) (`Sash`, righe 120-140).
- **Problema**: solo `onPointerUp` azzera `start.current`; mancano `onPointerCancel` e `onLostPointerCapture`. Il separatore è focalizzabile ma non ha `aria-valuenow/min/max`.
- **Impatto**: se il trascinamento finisce senza `pointerup` (Alt+Tab a metà, tocco annullato) il pannello si ridimensiona al passaggio successivo del mouse senza tasti premuti; i lettori di schermo non annunciano la dimensione.
- **Correzione**: `onPointerCancel` e `onLostPointerCapture` che azzerano `start.current`; passare dimensione e limiti (già in `useStoredSize`) come `aria-valuenow`, `aria-valuemin`, `aria-valuemax`.

### `BUG-V7-34` [BASSA] — Commenti superati sul terminale dell'app

- **Dove**: [`ui/src/App.tsx:60`](../../../ui/src/App.tsx#L60) e riga 582.
- **Problema**: «read when a session first opens (macOS and Linux), never on Windows» e «none on Windows», mentre `terminal/state.ts`, `api.ts`, `index.ts`, `panel.tsx` dicono «every system, D-148» e `view.tsx:137-145` gestisce ConPTY.
- **Impatto**: confonde chi mantiene il codice su dove si caricano xterm.js e il pannello.
- **Correzione**: «il pannello e xterm.js si caricano all'apertura della prima sessione, su ogni sistema (D-148, ConPTY del kit su Windows)».

### `BUG-V7-37` [BASSA] — Segnaposto italiano «CHIAVE» in ogni lingua

- **Dove**: [`ui/src/projects/wizard.tsx:80`](../../../ui/src/projects/wizard.tsx#L80); citato anche [`ui/src/frameworks/fields.tsx:94`](../../../ui/src/frameworks/fields.tsx#L94).
- **Problema**: `placeholder="CHIAVE"` per il nome della variabile `.env` non passa da `t()`; i segnaposto vicini usano inglese neutro (`APP_NAME={name}`).
- **Impatto**: chi usa inglese, francese o tedesco vede «CHIAVE» nella procedura del nuovo progetto.
- **Correzione**: segnaposto neutro `KEY` (come `APP_NAME` in `fields.tsx:199`) o `t("CHIAVE")` con le voci en/fr/de.
- **Verifica**: confutato in parte: «Avvia: npm run dev» in `fields.tsx:94` resta, perché «Avvia» è un dato del catalogo delle ricette, italiano anche nell'app reale (seguito di D-135). Resta solo `CHIAVE`.

### `BUG-V7-39` [BASSA] — Chiave `navDownload` inutilizzata nelle stringhe del sito

- **Dove**: [`site/strings/en.json:9`](../../../site/strings/en.json#L9) e i tre gemelli.
- **Problema**: `navDownload` («Download», «Scarica», «Télécharger», «Herunterladen») non è usata da `site/template.html` (la navigazione ha solo `navFeatures`/`navHow`/`navTools`/`navPlatforms`/`navTodo`), da `build.mjs` né da `matrix.mjs`.
- **Impatto**: un testo morto che invecchierà. Per il resto le quattro lingue hanno 118 chiavi uguali, nessun valore vuoto e gli stessi segnaposto.
- **Correzione**: toglierla dai quattro file, oppure aggiungere il link di download alla navigazione se era voluto. Da fare insieme all'ondata 3, che divide il modello del sito in parti.

---

## 6. Osservazioni sulle prove

- **`TEST-V7-01`** (osservazione, non un rilievo): il primo giro di `npm -w ui test`, con la macchina molto carica, ha dato 15 errori «Failed to start forks worker» / «Timeout waiting for worker» di Vitest; il secondo giro ha dato **42 file, 272 superati**. Sono timeout del pool di Vitest sotto carico, non prove fallite. Vitest stesso suggerisce `pool: "vmThreads"` o `isolate: false` per creare jsdom una volta per worker (27 creazioni, 41% del tempo): da valutare se il problema si ripete in CI.
- La prova i18n Rust ha un buco di copertura: `BUG-V7-36`.

---

## 7. Punti controllati senza rilievi

- **Download riprendibili**: gestione di `Range`/206/416 e continuità dell'hash in `resume.rs`; `Authorization` non inoltrata sui redirect (comportamento predefinito di ureq 3.4.2); nomi di file delle versioni remote filtrati (`is_plain_file_name`).
- **Porte**: `assign_port` tiene la prenotazione pendente e la mappa salvata; le porte dei pool vengono da `servers.json` sotto `store_lock`; la corsa sulla porta FastCGI è coperta da `php_cgi_listening` e `wait_with_php`.
- **Caddy**: URL degli alias, domini e percorsi validati o tra virgolette; socket di amministrazione e php-fpm privati.
- **Segreti**: passano da stdin, file privati o ambiente, mai dagli argomenti; la chiave FCGI solo nell'ambiente; il token GitHub con `GIT_CONFIG_*` limitato a `https://github.com/`.
- **MySQL**: nomi SQL limitati a `[a-z0-9_]`, password di 32 cifre esadecimali; ripristino come utente del progetto con `--commands=OFF`.
- **Server statico**: controllo dell'header `Host`, file nascosti, nomi corti 8.3 e collegamenti.
- **ConPTY e PTY**: il programma nasce sospeso, entra nel Job e poi riparte; i percorsi d'errore uccidono e raccolgono il figlio (`BUG-V6-25`); ogni comando del terminale passa da `ensure_pty`/`ensure_main`.
- **Lock**: sempre `unwrap_or_else(PoisonError::into_inner)`.
- **Interfaccia**: pulizia degli effetti del terminale (ascoltatori, xterm, `ResizeObserver`), guardie di `useSubscription`, `useLoad`, `useAction`, trappola del fuoco del dialogo modale.
- **Marchio**: nessun residuo dei nomi di prima nelle cartelle esaminate.

Non riportati perché minori o scelte di progetto: token HF e chiave Gemini scritti su FAT/exFAT senza il controllo «solo proprietario» di `claude_token`; ripiego del terminale su `/bin/bash` fisso; `links.rs` e `agents::running` che ignorano le maiuscole su Linux; minima finestra di riuso del pid in `hang_up`; la pipe con nome di Windows che un altro processo può creare prima.

---

## 8. Rilievi emersi durante l'ondata 1

Due rilievi nati dal lavoro dell'ondata 1, fuori dalla tabella del § 1; corretti nel ramo `fix/v7-followups`.

### `SEC-V7-01` [MEDIA] — Il token di GitHub nell'ambiente dell'agente, letto dai server MCP di un repository clonato

- **Dove**: [`tauri/src/agents/presets.rs`](../../../tauri/src/agents/presets.rs) `launch_vars`, [`tauri/src/commands/agents.rs`](../../../tauri/src/commands/agents.rs) `agent_launch` (D-156).
- **Problema**: con il server github pronto in `mcp.json` del kit, `launch_vars` metteva `GITHUB_PERSONAL_ACCESS_TOKEN` (il token di `gh auth token`) nell'ambiente dell'agente. I server MCP che sono programmi li avvia l'agente e ereditano il suo ambiente: un server stdio del `.mcp.json` di un repository clonato leggeva il token, e un suo indirizzo http lo riceveva espanso nelle intestazioni. `wanted_vars` escludeva solo la richiesta esplicita del progetto, non l'eredità.
- **Impatto**: il token dell'account GitHub del kit consegnato a codice del repository, appena l'utente ne approva i server.
- **Correzione**: il server github è ora `Atollo mcp github` ([`agents/github_mcp.rs`](../../../tauri/src/agents/github_mcp.rs)), avviato dall'agente con il percorso completo dell'app (mai un nome da cercare: su Windows la cartella del progetto, dove gira l'agente, è cercata prima del `PATH`, e un `atollo-mcp.cmd` o `cmd.exe` di un repository clonato partirebbe al suo posto; `agents::relocate` riscrive il percorso quando il kit si sposta): un ponte fra stdio e lo Streamable HTTP di GitHub che legge il token da sé e lo manda solo a `api.githubcopilot.com`. `launch_vars` non dà più il token a nessuno; nessun segreto nei file, come prima. Resta il rischio, scritto in D-156, che un programma eseguito a nome dell'utente chieda il token al `gh` del kit. Prove: `test/rust/agents/presets.rs` `the_token_never_in_the_agents_environment` (fallisce sul codice di prima), `test/rust/agents/github_mcp.rs`.

### `TEST-V7-02` [BASSA] — `serves_over_http` fallisce sotto carico con una connessione chiusa (reset)

- **Dove**: [`test/rust/projects/serve.rs`](../../../test/rust/projects/serve.rs) `start` e `serves_over_http`.
- **Problema**: `start` cercava una porta libera, la lasciava e poi `run` la riprendeva in un altro thread. Nel frattempo il server di un'altra prova del file, in parallelo, poteva prendersi la stessa porta: `run` allora non partiva, e la richiesta arrivava al server di `slowloris_does_not_block`, pieno di connessioni mute, che chiude subito le nuove (reset). `read_to_string(...).unwrap()` cadeva su quel reset.
- **Impatto**: una prova rossa a caso con la macchina carica, nessun difetto del server statico.
- **Correzione**: `run` è diviso in `run` (la porta e il messaggio) e `serve` (il ciclo delle connessioni); le prove danno a `serve` un listener già legato alla porta scelta, che non resta mai libera. In più la richiesta di `serves_over_http` riprova fino a 5 volte, con un'attesa breve, solo sugli errori di connessione (rifiutata, reset, interrotta): una risposta sbagliata fallisce come prima, e se il server non risponde mai la prova fallisce con gli errori di ogni tentativo. `sources_are_in_the_catalog` (i18n) non ha stato condiviso: legge i sorgenti di `tauri/src` mentre gira, con il catalogo compilato dentro la prova, quindi fallisce solo se un sorgente cambia durante il giro (un'altra sessione che lavora nella stessa copia); lasciata com'è.
