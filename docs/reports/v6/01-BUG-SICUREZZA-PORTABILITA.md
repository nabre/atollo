# 01 — Bug, Sicurezza e Portabilità (`v6`)

> **Riferimento codice**: `HEAD 6ecd492` (21/10/2026). Ogni rilievo ha: dove, scenario (stato → risultato sbagliato), evidenza, correzione. **Confermato** = percorso del codice univoco alla lettura; **PLAUSIBILE** = il fattore scatenante è raro o dipende da un comportamento esterno da verificare.
> I rilievi `v5` sono chiusi (vedi [`CHANGELOG.md`](../../app/CHANGELOG.md), voce «Rilievi del report v5») e non sono ripetuti.

---

## 1. Tabella riassuntiva

| ID | Grav. | Stato | Area | Titolo |
|---|:---:|:---:|---|---|
| `BUG-V6-01` | ALTA | Confermato | DB | Ripristino MySQL interrotto da `DEFINER=root` |
| `PORT-V6-01` | ALTA | Plausibile | OS | DACL «solo proprietario» illeggibile su un altro PC/account |
| `BUG-V6-04` | MEDIA | Confermato | DB | Dump senza `--hex-blob`, ripristino con `read_to_string` |
| `BUG-V6-05` | MEDIA | Confermato | DB | `run_with_input` ignora il timeout durante la scrittura di stdin |
| `BUG-V6-06` | MEDIA | Confermato | DB | Archivio MongoDB parziale elencato come valido, poi `--drop` |
| `BUG-V6-07` | MEDIA | Confermato | DB | Utente MySQL oltre 32 caratteri: il database non nasce |
| `BUG-V6-09` | MEDIA | Confermato | DB | Copia dei dati iniziali MySQL sul posto |
| `SEC-V6-01` | MEDIA | Plausibile | DB, server | File di segreti/stato illeggibile → riscritto vuoto |
| `BUG-V6-25` | BASSA | Confermato | OS | Processo avviato e lasciato fuori dal Job se `Job::new`/`assign` falliscono |
| `BUG-V6-26` | BASSA | Confermato | DB | Amministratore MongoDB creato prima di salvarne la password |
| `BUG-V6-27` | BASSA | Confermato | Core | `write_atomic` senza `fsync` |
| `BUG-V6-31` | BASSA | Confermato | Editor | Virgolette tipografiche nel percorso rompono il comando PowerShell |
| `BUG-V6-33` | BASSA | Confermato | Vari | Minori: `sync --project` vuoto, `components.txt` senza lock, token scritto poi protetto |

---

## 2. ALTA

### `BUG-V6-01` [ALTA] — Il ripristino MySQL si interrompe su viste, trigger, routine ed eventi

- **Dove**: [`db/services/backups.rs:23-29`](../../../tauri/src/db/services/backups.rs#L23-L29) (`DUMP_ARGS`), [`:151-172`](../../../tauri/src/db/services/backups.rs#L151-L172) (ripristino).
- **Scenario**: l'esportazione gira come `root` con `--routines --events --triggers`; il dump contiene ``DEFINER=`root`@`localhost` ``. Il ripristino gira, per scelta (D-78), come utente del progetto, che ha solo `GRANT ALL ON db.*`. Creare un oggetto con un `DEFINER` altrui richiede `SET_USER_ID`/`SUPER`: MySQL risponde `ERROR 1227` e il client si ferma al primo errore. Il dump è per tabella `DROP; CREATE; INSERT`: alcune tabelle sono ripristinate, le altre restano com'erano. Il database è incoerente.
- **Evidenza**: `"--routines", "--events", "--triggers"` nel dump; `// As the project's user, never root` nel ripristino. Nel repository nessuna gestione di `DEFINER`.
- **Correzione**: togliere `DEFINER=...` mentre si passa l'SQL al client (filtro in streaming su `/*!50013 DEFINER=... */` e `DEFINER=`…`@`…``), così gli oggetti nascono con l'utente del progetto. Prova: dump con vista e trigger → ripristino completo.

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

### `BUG-V6-09` — Dati iniziali di MySQL copiati sul posto
- **Dove**: [`db/services/data.rs:100-104`](../../../tauri/src/db/services/data.rs#L100-L104), [`:43-47`](../../../tauri/src/db/services/data.rs#L43-L47) (`has_data` guarda solo `mysql.ibd`).
- **Scenario**: copia interrotta (chiavetta piena o staccata) → `mysql.ibd` c'è già, l'avvio successivo salta la copia e `mysqld` apre una cartella incompleta.
- **Correzione**: copiare in `<linea>.partial`, rinominare a fine copia, pulire i `.partial` all'avvio.

### `SEC-V6-01` [Plausibile, effetto grave] — Un file di stato illeggibile diventa vuoto e viene riscritto
- **Dove**: [`db/auth.rs:53-55`](../../../tauri/src/db/auth.rs#L53-L55) (`secrets.json`), [`db/services/data.rs:193-200`](../../../tauri/src/db/services/data.rs#L193-L200), [`db/services/access.rs:36-70`](../../../tauri/src/db/services/access.rs#L36-L70), [`servers/pools.rs:38-40`](../../../tauri/src/servers/pools.rs#L38-L40), [`db/services/mod.rs:381-383`](../../../tauri/src/db/services/mod.rs#L381-L383).
- **Scenario**: `fsx::read_json` dà `None` su un file danneggiato o bloccato un attimo dall'antivirus; `unwrap_or_default()` lo tratta come vuoto; l'avvio successivo scrive un file nuovo: profili dei server persi, oppure nuova password di root MySQL / amministratore MongoDB e dati chiusi a chiave. Inoltre `secrets.json` è letto-modificato-scritto senza lock e i lock per motore sono distinti: due primi avvii (MySQL e MongoDB) possono cancellarsi la password a vicenda.
- **Correzione**: per segreti e stato non ricostruibile usare `try_read_json` e **rifiutare di scrivere** se il file esiste ma non si legge; un lock unico attorno a `secrets.json` (come `db_json_lock`).

---

## 4. BASSA

| ID | Dove | Scenario | Correzione |
|---|---|---|---|
| `BUG-V6-25` | [`db/services/mod.rs:703-705`](../../../tauri/src/db/services/mod.rs#L703-L705), [`editor/pty.rs:702-711`](../../../tauri/src/editor/pty.rs#L702-L711), [`editor/ext.rs:569-576`](../../../tauri/src/editor/ext.rs#L569-L576) | `spawn` poi `Job::new()?`/`assign()?`: a errore il figlio resta vivo fuori dal Job, con porta e cartella dati | `jobs::spawn_in` (già usato dai pool) o kill+wait su ogni errore |
| `BUG-V6-26` | [`db/services/access.rs:44-70`](../../../tauri/src/db/services/access.rs#L44-L70) | `createUser` poi salvataggio della password: se il salvataggio fallisce l'amministratore esiste con password ignota | salvare prima, creare dopo (come MySQL) |
| `BUG-V6-27` | [`fsx.rs:84`](../../../tauri/src/fsx.rs#L84) | senza `sync_all` prima del `rename`, su exFAT/FAT uno stacco brusco lascia file a lunghezza zero; con `SEC-V6-01` si perde lo stato | `File::create` + `write_all` + `sync_all` + `rename` |
| `BUG-V6-31` | [`editor/terminal.rs:119-133`](../../../tauri/src/editor/terminal.rs#L119-L133) | PowerShell tratta U+2018..U+201B come apici: kit in `D:\O’Brien\Atollo` → l'agente non parte | raddoppiare anche quei quattro caratteri, o passare il percorso come argomento |
| `BUG-V6-33` | [`cli.rs:131`](../../../tauri/src/cli.rs#L131); [`commands/mod.rs:177-187`](../../../tauri/src/commands/mod.rs#L177-L187); [`editor/mod.rs:231-232`](../../../tauri/src/editor/mod.rs#L231-L232) | `sync --project` senza valore diventa `None` invece di errore; `components.txt` letto-modificato-scritto senza lock; token scritto e solo dopo protetto | errore d'uso; lock; `write_private` |

---

## 5. Punti controllati senza rilievi

- **Estrazione degli archivi**: `safe_join`, `no_link_on_the_way`, ordine dei collegamenti, sostituzioni con rollback (`replace_dir`, `stage_with`, `clean_leftovers`). Nessuna zip-slip.
- **Checksum**: niente si installa senza SHA fisso, hash del produttore, integrità npm o SHA-512/SHA3; l'unica eccezione è `vc_redist` (`SEC-V6-03`). Pacchetti conda conservati ricontrollati. Aggiornamento automatico: digest GitHub verificato al download e allo scambio.
- **Lock delle installazioni**: mutex con nome su Windows, `flock` su Unix; cartelle di lavoro per id del kit.
- **Intervalli di versione**: caret, tilde, `~=`, trattino, jolly e `||` corretti secondo npm e PEP 440.
- **Porte**: registro comune coerente (pool 8081-8999, progetti 7100-7999, editor 7000-7099, DB).
- **Comandi IPC**: tutti i 123 chiamano `ensure_main`; argomenti dalla UI validati; nessun segreto nei log.
- **Runtime PHP**: `php-router.php`, `fcgi-guard.php`, phpMyAdmin; ascolti su 127.0.0.1 o 127.x casuale salvo LAN permessa.
