# Atollo App: piano per la cartella `data`

Stato: **approvato** (15/10/2026), domande del §7 chiuse dall'utente. Base: branch `refactor/code-structure`, commit `3df7ca3`.
Due obiettivi, dall'indicazione dell'utente del 15/10/2026:

1. **Impostazioni dei progetti**: tutto ciò che il kit tiene per un progetto sta in file dedicati, in una sottocartella per progetto di `data\projects\`.
2. **Temporanei**: i file scaricati e il lavoro di installazione passano per la **cartella temporanea del sistema**, così un kit su FAT32/exFAT non ha più i limiti di quei dischi durante download ed estrazione.

Decisioni nuove da scrivere in [DECISIONS.md](DECISIONS.md): D-89 (§3) e D-90 (§4). Voci da aggiornare con una nota **Oggi**: D-00, D-04, D-63, D-74, D-78 (copie dei database), D-83 (cartella di lavoro dei framework).

## 1. Stato attuale

### 1.1 Dati dei progetti: cinque posti

| Dato | Dove oggi | Codice | Chiave del progetto |
| --- | --- | --- | --- |
| Agente, `lastOpened`, estensioni, versioni dei linguaggi | `data\app\projects\<p>\dev-settings.json` | `projects/settings.rs`, `Kit::project_states` | cartella |
| Configurazione di Apache del progetto | `data\app\projects\<p>\apache\` | `projects/apache.rs` | cartella |
| Porte delle azioni | `data\app\ports.json`, mappa unica | `projects/run.rs` (`ports_file`, `key_of`) | `"<p>\|<azione>"` |
| Password dei database del progetto | `data\app\db\secrets.json`, campo `projects` | `db/auth.rs` | chiave della mappa |
| Copie dei database | `data\db\backup\<p>\` | `db/services/backups.rs` | cartella |

`versions/choice.rs` e `versions/mod.rs` scorrono `Kit::project_states()` per sapere quali linee dei linguaggi usano i progetti.

Problemi:

- **Eliminare un progetto lascia orfani.** `projects::delete_empty` chiama solo `settings::remove`: porte, password e copie restano.
- **Nessuna vista unica.** Per sapere cosa il kit tiene di un progetto bisogna leggere tre file globali e due cartelle.
- **File globali scritti da più parti.** `ports.json` e `secrets.json` si riscrivono interi a ogni modifica di un solo progetto.
- `data\app\projects\` sta dentro lo stato dell'app, mescolato a impostazioni e log del kit.

### 1.2 Temporanei e download: tutti nel kit

| Uso | Dove oggi | Codice |
| --- | --- | --- |
| Lavoro di installazione (download `.part`, archivio, estrazione `x\`) | `data\tmp\setup\<id>\` | `install::work_dir`, `install::fetch`, `net::download`, `net/archive.rs` |
| Estensioni e tema dell'editor | `data\tmp\setup\extensions`, `…\theme` | `editor/ext.rs`, `editor/theme.rs` |
| Redistribuibile di Visual C++ | `data\tmp\setup\vcruntime` | `install/web.rs` |
| Nuovo progetto da framework | `data\tmp\frameworks\<stamp>\` | `commands/frameworks.rs` |
| File scaricati a mano (D-82) | `data\downloads\`, tenuti 30 giorni | `setup/downloads.rs` |
| Caricamenti di Adminer | `data\app\adminer` (`upload_tmp_dir`) | `db/adminer.rs` |
| `TEMP`/`TMP` dei programmi del kit | `data\tmp\system` | `env.rs` |
| `CLAUDE_CODE_TMPDIR` | `data\tmp` | `env.rs` |
| `TEMP`/`TMP` del processo dell'app | in `data\` (D-04) | `main.rs` |

Limiti di FAT32/exFAT che oggi colpiscono questi passaggi:

- **FAT32: file al massimo di 4 GiB - 1.** Un archivio grande (SDK di Android, TeX Live, immagini di sistema) non si salva nemmeno come `.part`.
- **Nessun collegamento fisico.** rustup non si installa (`install/native.rs`: «rustup needs a disk with hard links (NTFS)»); la voce è aperta in CHANGELOG.
- **Nessun permesso NTFS.** I file privati (`data\app\db`) non si proteggono; non è un tema di questo piano, ma il §3 non deve peggiorarlo.
- **Velocità.** Su una chiavetta, scaricare, estrarre e poi spostare scrive tutto due volte sullo stesso disco lento; un'estrazione di decine di migliaia di file piccoli (Node, Python, Android) è il caso peggiore.
- **Robustezza.** Una chiavetta tolta a metà di un'estrazione lascia una FAT incoerente; meglio che le scritture parziali avvengano su un disco interno.

## 2. Principi

- **Stato nel kit, transito nel sistema.** Ciò che serve alla prossima apertura (impostazioni, programmi, dati, copie) resta in `data\`. Ciò che serve solo durante un'operazione (download, estrazione, preparazione) usa la cartella temporanea del sistema e si cancella alla fine.
- **Un progetto, una cartella.** Ogni dato del kit legato a un progetto è sotto `data\projects\<p>\`; eliminare, rinominare o salvare un progetto è un'operazione su una cartella.
- **Il progetto resta pulito** (D-63): niente di nuovo dentro `projects\<p>\`. Ciò che descrive il progetto continua ad andare nei suoi file (D-78).
- **Migrazione automatica e ripetibile** all'avvio, senza perdere dati; un'interruzione a metà si riprende all'avvio dopo.
- **Ritorno al kit se il sistema non basta**: temp non scrivibile o senza spazio → `data\tmp` come oggi, con un avviso nel log.

## 3. `data\projects\<progetto>\` (D-89)

### 3.1 Struttura

```text
data\projects\<p>\
  dev-settings.json      agente, lastOpened, estensioni, versioni (come oggi, D-63)
  ports.json             { "<azione>": porta } e le porte bloccate dall'utente
  apache\                httpd.conf e file del sito (da data\app\projects\<p>\apache)
  private\               DACL protetto ed ereditabile, come data\app\db
    db.json              { "mariadb": "<password>", "mongodb": "<password>" }
    backups\             copie dei database (da data\db\backup\<p>, Q-1)
```

- `dev-settings.json` mantiene nome e formato (scelta dell'utente, D-63): cambia solo la cartella.
- Ogni file nuovo segue la regola già in uso: le chiavi scritte da una versione più nuova restano (`#[serde(flatten)] other`).
- `private\` riceve il DACL **prima** di scriverci (stesso ordine della correzione di sicurezza di D-74). Su FAT32/exFAT nessun permesso, come oggi.
- I file globali che restano in `data\app\`: `settings.json`, `services.json`, `components.txt`, `versions\`, `frameworks.json`, `extensions*.txt`, `root.txt`, `logs\`, `db\secrets.json` senza il campo `projects` (solo `root` e `admin`).

Copie dei database (Q-1, scelta dell'utente del 15/10/2026): nella cartella del progetto, così eliminare o spostare un progetto porta con sé tutto. Stanno in `private\` perché contengono i dati dei database: oggi in `data\db\backup\` non hanno un DACL proprio.

### 3.2 Codice

- `Kit::project_states()` → `Kit::project_data()` = `data\projects`. Il nome `projects()` resta per la cartella dei progetti nella radice.
- Nuovo modulo `projects/state.rs` con un tipo unico:

  ```rust
  /// The kit's files of one project: `data\projects\<p>\`.
  pub struct ProjectState { dir: PathBuf }

  impl ProjectState {
      pub fn of(kit: &Kit, project: &str) -> Option<ProjectState>; // validate_project_name
      pub fn settings_file(&self) -> PathBuf;
      pub fn ports_file(&self) -> PathBuf;
      pub fn apache_dir(&self) -> PathBuf;
      pub fn private_dir(&self) -> io::Result<PathBuf>; // DACL prima di tornare
      pub fn remove(self) -> io::Result<()>;
      pub fn rename(self, to: &str) -> io::Result<ProjectState>;
  }
  pub fn all(kit: &Kit) -> Vec<String>; // cartelle valide sotto data\projects
  ```

- `projects/settings.rs` usa `ProjectState::settings_file`; `remove` diventa `ProjectState::remove`.
- `projects/run.rs`: `Ports` caricato da tutti i `ports.json` dei progetti all'avvio (il controllo "porta già usata da un altro progetto" resta in memoria come oggi); `save_ports` scrive solo il file del progetto cambiato.
- `db/auth.rs`: `Secrets.projects` si legge e si scrive per progetto in `private\db.json`; `root`/`admin` restano in `data\app\db\secrets.json`.
- `projects/apache.rs`, `projects/run.rs` (riga del comando di `httpd.exe`), `versions/choice.rs`, `versions/mod.rs`: percorso da `ProjectState`.
- `db/services/backups.rs`: `backup_dir(project)` = `ProjectState::private_dir()\backups`.
- `projects::delete_empty` (Q-2, scelta dell'utente del 15/10/2026): elimina il progetto **e** i suoi database nei server del kit.
  1. La conferma elenca cosa sparisce: cartella del progetto, database e utente MariaDB/MongoDB, numero e dimensione delle copie in `private\backups\`, che vanno via con la cartella. Un pulsante «Esporta le copie…» apre la cartella prima di confermare.
  2. Per ogni motore con una password in `db.json`: avvia il server se è fermo (come fa Adminer), poi `DROP DATABASE IF EXISTS` + `DROP USER IF EXISTS` (MariaDB) o `dropDatabase()` + `dropUser()` (MongoDB), con il login di amministrazione da `secrets.json`.
  3. Solo se il passo 2 riesce si cancellano `projects\<p>\` e `data\projects\<p>\`. Se un server non parte o il comando fallisce: errore, nulla è cancellato, e la conferma offre «Elimina senza i database», che lascia database e utente nel server e scrive nel log quali.
  4. Un motore non installato: niente da togliere, si salta.

### 3.3 Progetti senza cartella

Un progetto rinominato o cancellato fuori dall'app lascia `data\projects\<vecchio>\`. Proposta:

- all'avvio e all'aggiornamento della lista, le cartelle di `data\projects\` senza `projects\<p>\` sono **orfane**;
- la pagina Progetti le mostra in una sezione chiusa con «Collega a…» (sposta i file su un progetto esistente senza dati del kit) e «Elimina»;
- mai cancellate da sole: contengono password di database ancora valide.

### 3.4 Migrazione (versione dello schema 1 → 2)

All'avvio, dopo `Kit::prepare` e prima di qualsiasi lettura dei file dei progetti:

1. Se `data\app\layout.txt` vale già `2`, niente da fare.
2. Per ogni `data\app\projects\<p>\`: `rename` in `data\projects\<p>\` (stesso disco); se la destinazione esiste, sposta voce per voce senza sovrascrivere.
3. `data\app\ports.json`: le chiavi `"<p>|<azione>"` vanno in `data\projects\<p>\ports.json`; le porte bloccate seguono la stessa divisione.
4. `data\app\db\secrets.json`: ogni voce di `projects` va in `data\projects\<p>\private\db.json`; il file globale si riscrive senza `projects` **solo dopo** che tutti i file per progetto sono scritti.
5. `data\db\backup\<p>\` → `data\projects\<p>\private\backups\` con `rename` (stesso disco, nessuna copia anche se pesanti); prima si crea `private\` con il DACL, poi si sposta il contenuto voce per voce perché erediti i permessi.
6. Scritture atomiche (`fsx::write_json`), poi `layout.txt = 2`; i file vecchi si cancellano per ultimi.
7. Un errore a metà: log, si riprova all'avvio dopo; nel frattempo la lettura cerca prima il percorso nuovo e poi il vecchio.

Ritorno a una versione precedente dell'app: non legge `data\projects\`. Va scritto nel CHANGELOG; una copia di sicurezza `data\app\pre-v2\` (i soli `ports.json` e `secrets.json`, piccoli) permette di tornare indietro a mano.

## 4. Temporanei nella cartella del sistema (D-90)

### 4.1 Dove

- `Kit::system_temp()` = `<TEMP del sistema>\Atollo\<kit.id()>\`. `kit.id()` esiste già (SHA-256 della radice): due kit sullo stesso PC non si toccano.
- **Il valore va letto prima che D-04 sposti `TEMP`/`TMP` del processo in `data\`**: `main.rs` lo prende con `GetTempPath2W` (o dalle variabili dell'host) e lo passa a `Kit`. `std::env::temp_dir()` dopo l'avvio restituirebbe la cartella del kit.
- Sottocartelle: `setup\<id>\`, `frameworks\<stamp>\`, `downloads\`, `adminer\`, `extensions\`, `theme\`.
- Ogni operazione tiene un file `.lock` aperto nella sua cartella: la pulizia all'avvio cancella solo le cartelle senza lock e più vecchie di 24 ore.

### 4.2 Cosa cambia per ogni uso

| Uso | Prima | Dopo |
| --- | --- | --- |
| Download, verifica, estrazione | `data\tmp\setup\<id>\` | `system_temp\setup\<id>\` |
| Passaggio in `data\apps\` | `rename` dallo stesso disco | copia in `data\apps\<id>.atollo-new\`, poi `replace_dir` come oggi |
| Estensioni, tema, vcruntime | `data\tmp\setup\…` | `system_temp\…` |
| Framework | `data\tmp\frameworks\` | `system_temp\frameworks\`, poi copia nel progetto (già una copia oggi) |
| Scaricati a mano (D-82) | `data\downloads\` | `system_temp\downloads\` (Q-3: su un altro PC si riscaricano) |
| Caricamenti di Adminer | `data\app\adminer` | `system_temp\adminer\`; le sessioni restano in `data\app\adminer` |

Il passaggio in `data\apps\` è il punto delicato: `fs::rename` fra due dischi fallisce (`ERROR_NOT_SAME_DEVICE`). Si aggiunge `archive::stage_into(tree, dest)`: se la sorgente è su un altro volume copia l'albero in `<dest>.atollo-new` accanto alla destinazione e poi chiama `replace_dir`; sullo stesso volume fa `rename` come oggi. Lo scambio resta atomico, l'eventuale copia interrotta lascia solo `.atollo-new`, che `clean_leftovers` già rimuove.

### 4.3 Controlli prima di cominciare

- **File system del kit**: `GetVolumeInformationW` sulla radice (`FAT32`, `exFAT`, `NTFS`, `ReFS`), in `Kit` e mostrato nella pagina Impostazioni.
- **File oltre 4 GiB su FAT32**: dopo l'estrazione e prima della copia si scorre l'albero; un file troppo grande ferma l'installazione con un errore chiaro, senza toccare la versione installata.
- **Spazio**: `GetDiskFreeSpaceExW` su temp (dimensione dell'archivio × 3, stima di download + estrazione) e sul kit (dimensione dell'albero estratto). Spazio insufficiente nel temp → ritorno a `data\tmp` con avviso; spazio insufficiente nel kit → errore.

### 4.4 Rust su FAT32/exFAT

Con il temp di sistema su NTFS si può portare il comportamento del vecchio Atollo (voce aperta in CHANGELOG):

1. `RUSTUP_HOME`/`CARGO_HOME` in `system_temp\setup\rust\`, `rustup-init` e `rustup update` lì;
2. copia dell'albero in `data\apps\…\rust` con `stage_into` (i collegamenti fisici diventano copie: più spazio, stesso funzionamento);
3. aggiornamento: copia del `rust` del kit nel temp, `rustup update`, copia di ritorno. Lento, ma solo su FAT/exFAT; su NTFS resta l'aggiornamento sul posto.

### 4.5 `TEMP` dei programmi del kit

`TEMP`/`TMP` dei programmi lanciati dal kit (`data\tmp\system`) e `CLAUDE_CODE_TMPDIR` (`data\tmp`) scrivono molto: npm, pip, compilatori, build, output dei comandi degli agenti.

| | Nel kit (`data\tmp`) | Nel sistema (`%TEMP%\Atollo\<id>`) |
| --- | --- | --- |
| **Sicurezza** | Su FAT32/exFAT nessun permesso: chiunque abbia la chiavetta legge i resti (pezzi di codice, output degli agenti, a volte token nei log di npm/pip). Una chiavetta persa li porta con sé. Su NTFS valgono i permessi della cartella del kit, spesso larghi (`D:\` dà modifica agli Utenti autenticati) | Cartella dell'utente di Windows: solo l'utente, SYSTEM e gli amministratori del PC. Resta sul PC ospite fino alla pulizia: un amministratore di un PC in prestito potrebbe leggerla |
| **Portabilità** | Nessuna traccia sul PC (D-04) | File di passaggio sul PC, cancellati alla chiusura e dalla pulizia all'avvio; nessuna differenza per i programmi, che non tengono stato nel `TEMP` |
| **Velocità** | Su chiavetta lenta: `npm install`, `pip install` e le build creano migliaia di file piccoli, il caso peggiore per FAT/exFAT su USB. Su un disco interno NTFS: uguale al sistema | Disco interno (di solito SSD): veloce sempre. Niente limite di 4 GiB, collegamenti fisici disponibili |

Nessuna delle due vince ovunque, ma il confronto dipende solo dal disco del kit. **Scelta: «Automatico»** (Q-4), deciso all'avvio da `GetVolumeInformationW` e `GetDriveTypeW`:

- kit su disco **fisso NTFS/ReFS** → `data\tmp` come oggi, con un DACL protetto (solo l'utente) su `data\tmp`: stessa velocità del sistema, stessa protezione, nessuna traccia sul PC;
- kit su disco **rimovibile** o **FAT32/exFAT** → temp di sistema: più veloce, protetto dai permessi di Windows, e nulla di sensibile sulla chiavetta.

Più un'impostazione «Temporanei dei programmi: Automatico / Kit / Sistema» per chi vuole zero tracce sul PC (Kit) o la velocità sempre (Sistema). Alla chiusura dell'app (il job dell'app ha già chiuso i programmi) la cartella `system\` del temp di sistema si cancella; i resti di una chiusura brusca li toglie la pulizia all'avvio (§4.1). La stessa regola vale per `CLAUDE_CODE_TMPDIR`.

### 4.6 Portabilità

D-00 e D-04 dicono che l'app non scrive fuori da `data\`. D-90 è un'eccezione limitata e da scrivere:

- solo file di passaggio, cancellati a fine operazione e dalla pulizia all'avvio;
- mai stato, impostazioni, segreti o file che servono alla prossima apertura;
- con la cartella del sistema non disponibile, il kit funziona come oggi.

## 5. Fasi

| Fase | Contenuto | File principali | Verifica |
| --- | --- | --- | --- |
| A | `Kit::project_data`, `ProjectState`, `settings.rs` e `apache.rs` sul nuovo percorso | `kit.rs`, `projects/state.rs`, `projects/settings.rs`, `projects/apache.rs`, `versions/choice.rs`, `versions/mod.rs` | test esistenti di `settings.rs` sul percorso nuovo; nomi non validi rifiutati |
| B | Porte, password e copie per progetto; eliminazione con i database | `projects/run.rs`, `db/auth.rs`, `db/services/backups.rs`, `projects/mod.rs`, `ui/src/projects/` | due progetti con porte diverse, conflitto rilevato; DACL di `private\` prima della scrittura; eliminazione con server fermo, server che non parte, motore non installato |
| C | Migrazione 1 → 2 e orfani | `projects/state.rs` (`migrate`), `main.rs`, `ui/src/projects/page.tsx` | kit di prova con i file vecchi; migrazione interrotta dopo il passo 3 e ripresa; nessuna chiave persa |
| D | `Kit::system_temp`, letto prima di D-04; pulizia all'avvio e alla chiusura; tipo di disco; `TEMP` dei programmi «Automatico» (§4.5) | `main.rs`, `kit.rs`, `env.rs`, `settings.rs`, `ui/src/settings*` | temp letto prima dell'override; due kit con id diversi; lock rispettato; scelta giusta per NTFS fisso, chiavetta exFAT, NTFS rimovibile |
| E | Installazioni nel temp, `stage_into`, controlli di §4.3 | `install/mod.rs`, `net/archive.rs`, `install/web.rs`, `editor/ext.rs`, `editor/theme.rs`, `commands/frameworks.rs`, `setup/downloads.rs`, `db/adminer.rs` | kit su chiavetta exFAT e FAT32 (Node, Python, Java, SDK di Android); file > 4 GiB rifiutato; ritorno a `data\tmp` senza spazio |
| F | Rust su FAT/exFAT | `install/native.rs` | installazione e aggiornamento su exFAT; `cargo build` di un progetto di prova |
| G | Documenti | `DECISIONS.md` (D-89, D-90, note **Oggi**), `ARCHITECTURE.md`, `CHANGELOG.md`, `TESTING.md` | |

A-C e D-F sono indipendenti: si possono fare in due rami.

## 6. Rischi

- **Antivirus sul temp di sistema**: esamina l'estrazione come oggi in `data\tmp`; `retry_denied` resta sulla copia finale.
- **Temp su un disco piccolo** (C: quasi pieno): coperto dal controllo di spazio e dal ritorno a `data\tmp`.
- **Doppio PC**: un'installazione interrotta su un PC lascia file nel temp di quel PC; la pulizia avviene alla prossima apertura del kit lì, o li cancella Windows (Sensore memoria).
- **Password nella migrazione**: `secrets.json` riscritto solo dopo i file per progetto; nessuna password nel log.

## 7. Domande aperte

Chiuse il 15/10/2026 (scelte dell'utente):

- **Q-1** Copie dei database → nella cartella del progetto, `data\projects\<p>\private\backups\` (§3.1).
- **Q-2** Eliminando un progetto si tolgono anche il suo database e il suo utente dai server, con conferma (§3.2).
- **Q-3** File scaricati a mano → temp di sistema (§4.2).
- **Q-4** `TEMP` dei programmi del kit e `CLAUDE_CODE_TMPDIR` → «Automatico» secondo il disco del kit, con l'impostazione «Automatico / Kit / Sistema» (§4.5).
- **Q-5** Nome della cartella → `data\projects\`, specchio di `projects\` nella radice (`projects\shop` ↔ `data\projects\shop`). Con le copie dei database dentro (Q-1) non contiene solo impostazioni, quindi `data\project-settings\` sarebbe inesatto. Scartate: `data\project-state\` (meno immediato), `data\workspaces\` (si confonde con i workspace di VSCodium), `data\kit-projects\` (sembra un elenco di progetti).

Nessuna domanda aperta.
