# 01 — Bug, Sicurezza e Portabilità (`v6`)

> **Riferimento codice**: `HEAD 6ecd492` (21/10/2026). Ogni rilievo ha: dove, scenario (stato → risultato sbagliato), evidenza, correzione. **Confermato** = percorso del codice univoco alla lettura; **PLAUSIBILE** = il fattore scatenante è raro o dipende da un comportamento esterno da verificare.
> I rilievi `v5` sono chiusi (vedi [`CHANGELOG.md`](../../app/CHANGELOG.md), voce «Rilievi del report v5») e non sono ripetuti.

---

## 1. Tabella riassuntiva

| ID | Grav. | Stato | Area | Titolo |
|---|:---:|:---:|---|---|
| `PORT-V6-01` | ALTA | Plausibile | OS | DACL «solo proprietario» illeggibile su un altro PC/account |
| `SEC-V6-01` | MEDIA | Plausibile | DB, server | File di segreti/stato illeggibile → riscritto vuoto |
| `BUG-V6-25` | BASSA | Confermato | OS | Processo avviato e lasciato fuori dal Job se `Job::new`/`assign` falliscono |
| `BUG-V6-31` | BASSA | Confermato | Editor | Virgolette tipografiche nel percorso rompono il comando PowerShell |
| `BUG-V6-33` | BASSA | Confermato | Vari | Minori: `sync --project` vuoto, `components.txt` senza lock, token scritto poi protetto |

---

## 2. ALTA

### `PORT-V6-01` [ALTA, da confermare] — La DACL «solo proprietario» rende i file del kit illeggibili su un altro PC

- **Dove**: [`router/fs.rs:26-31`](../../../tauri/src/router/fs.rs#L26-L31) (`D:P(A;;FA;;;OW)`, protetta, Owner Rights), applicata al token dell'editor ([`editor/mod.rs:229-234`](../../../tauri/src/editor/mod.rs#L229-L234)), alla chiave Gemini ([`settings.rs:232`](../../../tauri/src/settings.rs#L232)), allo stato privato dei progetti ([`projects/state.rs:61`](../../../tauri/src/projects/state.rs#L61)), ai segreti dei DB e a `data\tmp`.
- **Scenario**: su un disco **NTFS** la DACL viaggia con il file e nomina il proprietario del PC A. Sul PC B un altro account locale ha un altro SID e Owner Rights non lo riguarda. `fs::write(&token_file, ...)` su un file esistente riceve `ACCESS_DENIED` e l'editor non parte; [`read_secret`](../../../tauri/src/settings.rs#L272-L277) inghiotte l'errore e la chiave Gemini appare «non impostata»; le password dei DB diventano illeggibili. Nessun ripiego (`takeown`, reset della DACL) nel codice. Su exFAT/FAT non succede (niente ACL): il rischio è per SSD esterni NTFS, il caso «chiavetta seria».
- **Correzione**: (1) i segreti di sessione (token dell'editor) vanno ricreati: cancellare e scrivere, non sovrascrivere; meglio in `%TEMP%`. (2) Per i file che viaggiano col kit, su accesso negato: tentare `SetNamedSecurityInfo` come proprietario attuale, altrimenti segnalare con un messaggio che spiega cosa fare. (3) `read_secret` deve distinguere «assente» da «illeggibile». Verifica manuale: due account Windows sullo stesso SSD NTFS.

---

## 3. MEDIA

### `SEC-V6-01` [Plausibile, effetto grave] — Un file di stato illeggibile diventa vuoto e viene riscritto
- **Dove**: [`db/auth.rs:53-55`](../../../tauri/src/db/auth.rs#L53-L55) (`secrets.json`), [`db/services/data.rs:193-200`](../../../tauri/src/db/services/data.rs#L193-L200), [`db/services/access.rs:36-70`](../../../tauri/src/db/services/access.rs#L36-L70), [`servers/pools.rs:38-40`](../../../tauri/src/servers/pools.rs#L38-L40), [`db/services/mod.rs:381-383`](../../../tauri/src/db/services/mod.rs#L381-L383).
- **Scenario**: `fsx::read_json` dà `None` su un file danneggiato o bloccato un attimo dall'antivirus; `unwrap_or_default()` lo tratta come vuoto; l'avvio successivo scrive un file nuovo: profili dei server persi, oppure nuova password di root MySQL / amministratore MongoDB e dati chiusi a chiave. Inoltre `secrets.json` è letto-modificato-scritto senza lock e i lock per motore sono distinti: due primi avvii (MySQL e MongoDB) possono cancellarsi la password a vicenda.
- **Correzione**: per segreti e stato non ricostruibile usare `try_read_json` e **rifiutare di scrivere** se il file esiste ma non si legge; un lock unico attorno a `secrets.json` (come `db_json_lock`).

---

## 4. BASSA

| ID | Dove | Scenario | Correzione |
|---|---|---|---|
| `BUG-V6-25` | [`db/services/mod.rs:703-705`](../../../tauri/src/db/services/mod.rs#L703-L705), [`editor/pty.rs:702-711`](../../../tauri/src/editor/pty.rs#L702-L711), [`editor/ext.rs:569-576`](../../../tauri/src/editor/ext.rs#L569-L576) | `spawn` poi `Job::new()?`/`assign()?`: a errore il figlio resta vivo fuori dal Job, con porta e cartella dati | `jobs::spawn_in` (già usato dai pool) o kill+wait su ogni errore |
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
