# Atollo App: piano per la cartella `data`

Stato: **approvato** (15/10/2026), domande del §3 chiuse dall'utente. Revisione 2: piano sviluppato in pacchetti di lavoro (§5) e struttura di uno sciame di agenti per eseguirlo (§7). Revisione 3: correzione degli alias dei siti (§1.3, WP-S), dalla revisione di sicurezza automatica del 15/10/2026, eseguita nell'ondata 0.
Base: branch `refactor/code-structure`, commit `3df7ca3`.

Due obiettivi, dall'indicazione dell'utente del 15/10/2026:

1. **Dati dei progetti**: tutto ciò che il kit tiene per un progetto sta in file dedicati, in una sottocartella per progetto di `data\projects\`.
2. **Temporanei**: i file scaricati e il lavoro di installazione passano per la **cartella temporanea del sistema**, così un kit su FAT32/exFAT non ha più i limiti di quei dischi durante download ed estrazione.

Decisioni nuove da scrivere in [DECISIONS.md](DECISIONS.md): **D-89** (dati dei progetti, §4.1) e **D-90** (temporanei, §4.2). Voci da aggiornare con una nota **Oggi**: D-00, D-04, D-63, D-74, D-78 (copie dei database), D-82 (scaricati a mano), D-83 (cartella di lavoro dei framework).

## 1. Stato attuale

### 1.1 Dati dei progetti: cinque posti

| Dato | Dove oggi | Codice | Chiave del progetto |
| --- | --- | --- | --- |
| Agente, `lastOpened`, estensioni, versioni dei linguaggi | `data\app\projects\<p>\dev-settings.json` | `projects/settings.rs`, `Kit::project_states` | cartella |
| Configurazione di Apache del progetto | `data\app\projects\<p>\apache\` | `projects/apache.rs` | cartella |
| Porte delle azioni | `data\app\ports.json` (`Ports { ports, locked }`) | `projects/run.rs` (`ports_file`, `key_of`) | `"<p>\|<azione>"` |
| Password dei database del progetto | `data\app\db\secrets.json`, campo `projects` | `db/auth.rs` | chiave della mappa |
| Copie dei database | `data\db\backup\<p>\` | `db/services/backups.rs` | cartella |

Restano fuori, perché sono del kit e non del progetto: `data\app\servers.json` (profili dei pool di Apache/Nginx e loro porte) e `data\servers\<pool>\` (configurazioni generate dei siti, riscritte a ogni avvio del pool da `servers/mod.rs`). Quando WP-5 elimina un progetto, il suo sito sparisce dal pool alla rigenerazione successiva.

`versions/choice.rs` e `versions/mod.rs` scorrono `Kit::project_states()` per sapere quali linee dei linguaggi usano i progetti.

Problemi:

- **Eliminare un progetto lascia orfani.** `projects::delete_empty` chiama solo `settings::remove`: porte, password, copie, database e utente nei server restano.
- **Nessuna vista unica.** Per sapere cosa il kit tiene di un progetto bisogna leggere tre file globali e due cartelle.
- **File globali scritti da più parti.** `ports.json` e `secrets.json` si riscrivono interi a ogni modifica di un solo progetto.
- **Copie dei database senza protezione propria**: `data\db\backup\` non ha un DACL, a differenza di `data\app\db`.

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

- **FAT32: file al massimo di 4 GiB - 1.** Un archivio grande non si salva nemmeno come `.part`.
- **Nessun collegamento fisico.** rustup non si installa (`install/native.rs`: «rustup needs a disk with hard links (NTFS)»); voce aperta in CHANGELOG.
- **Nessun permesso.** I file privati non si proteggono.
- **Velocità.** Su una chiavetta, scaricare, estrarre e spostare scrive tutto due volte sullo stesso disco lento; decine di migliaia di file piccoli (Node, Python, Android) sono il caso peggiore.
- **Robustezza.** Una chiavetta tolta a metà di un'estrazione lascia una FAT incoerente.

### 1.3 Alias dei siti (revisione di sicurezza)

Un sito servito da Apache o Nginx può avere alias: `"/media": "storage/uploads"` nel `panel.json` del progetto, cioè in un file del repository, quindi non fidato (stesso principio di D-77). Gli alias finiscono nella configurazione del server in tre punti, ognuno con il suo codice di risoluzione:

| Dove | Funzione | Forma accettata per un altro progetto |
| --- | --- | --- |
| Lettura e salvataggio di `panel.json` | `detect/overrides.rs`: `valid_alias_url`, `valid_alias_target` | `@altro/cartella` (`..`, `:` e `/` iniziale rifiutati) |
| Sito Apache di un progetto | `projects/apache.rs`: `alias_target_expr`, `alias_directives` | `../altro/cartella` |
| Sito Nginx di un progetto | `projects/nginx.rs`: `resolve_alias_path`, `alias_locations` | `../altro/cartella` |
| Sito in un pool | `servers/mod.rs:214-238`, copia in linea di `alias_target_expr` | `../altro/cartella` |

Esito della verifica:

- **Oggi non sfruttabile da `panel.json`**: `valid_alias_target` ammette solo lettere, cifre, `-_.`, `/` e un `@` iniziale, senza `..`; `valid_alias_url` è altrettanto stretto. Niente virgolette, a capo o `${…}` arrivano alla configurazione.
- **Difesa solo all'ingresso.** I tre generatori si fidano della stringa e risolvono `../` verso `${CP_PROJECTS_FWD}`, poi scrivono `Alias`/`<Directory>` con `Require all granted` o `alias` di Nginx. Una nuova fonte di alias che salti il controllo (un altro file, un comando, `servers.json`) servirebbe qualsiasi cartella del disco o aggiungerebbe direttive.
- **Errore di funzionamento.** Il controllo accetta `@altro/cartella`, i generatori capiscono `../altro/cartella`: un alias verso un altro progetto diventa `${CP_PROJECT_FWD}/@altro/cartella`, che non esiste. I test di `nginx.rs` e `servers/mod.rs` usano `../common/assets`, una forma che `panel.json` non può contenere.
- **Collegamenti.** La destinazione di un alias non viene controllata contro collegamenti e giunzioni (D-77): `storage` nel repository come collegamento a `C:\Users` verrebbe servito, con `Options FollowSymLinks`.
- **Ascolto.** I pool e i siti ascoltano su `127.0.0.1`, su tutte le interfacce solo con `ATOLLO_ALLOW_LAN=1`: con la LAN aperta il danno di un alias sbagliato si estende alla rete.

La correzione è WP-S (§5).

## 2. Principi

- **Stato nel kit, transito nel sistema.** Ciò che serve alla prossima apertura (impostazioni, programmi, dati, copie) resta in `data\`. Ciò che serve solo durante un'operazione usa la cartella temporanea del sistema e si cancella alla fine.
- **Un progetto, una cartella.** Ogni dato del kit legato a un progetto è sotto `data\projects\<p>\`; eliminare, rinominare o salvare un progetto è un'operazione su una cartella.
- **Il progetto resta pulito** (D-63): niente di nuovo dentro `projects\<p>\`. Ciò che descrive il progetto continua ad andare nei suoi file (D-78).
- **Migrazione automatica e ripetibile** all'avvio, senza perdere dati; un'interruzione a metà si riprende all'avvio dopo.
- **Ritorno al kit se il sistema non basta**: temp non scrivibile o senza spazio → `data\tmp` come oggi, con un avviso nel log.
- **Mai un segreto nel log**, mai un file privato scritto prima che la sua cartella sia privata.

## 3. Scelte dell'utente (15/10/2026)

| | Domanda | Scelta |
| --- | --- | --- |
| Q-1 | Dove vanno le copie dei database | Nella cartella del progetto: `data\projects\<p>\private\backups\` |
| Q-2 | Eliminando un progetto, togliere database e utente dai server | Sì, con conferma |
| Q-3 | File scaricati a mano | Temp di sistema |
| Q-4 | `TEMP` dei programmi del kit e `CLAUDE_CODE_TMPDIR` | «Automatico» secondo il disco del kit, con impostazione «Automatico / Kit / Sistema» (§4.2.5) |
| Q-5 | Nome della cartella | `data\projects\`, specchio di `projects\` nella radice. Scartate: `data\project-settings\` (contiene anche copie, non solo impostazioni), `data\project-state\` (meno immediato), `data\workspaces\` (si confonde con i workspace di VSCodium), `data\kit-projects\` (sembra un elenco di progetti) |

## 4. Disegno

### 4.1 `data\projects\<progetto>\` (D-89)

```text
data\projects\<p>\
  dev-settings.json      agente, lastOpened, estensioni, versioni (formato di oggi, D-63)
  ports.json             { "ports": { "<azione>": porta }, "locked": ["<azione>"] }
  apache\                httpd.conf e file del sito
  private\               DACL protetto ed ereditabile (solo il proprietario)
    db.json              { "mariadb": "<password>", "mongodb": "<password>" }
    backups\             copie dei database (.sql, .archive.gz)
```

- `dev-settings.json` mantiene nome e formato: cambia solo la cartella.
- Ogni file nuovo conserva le chiavi scritte da una versione più nuova (`#[serde(flatten)] other`, come `ProjectSettings`).
- `private\` riceve il DACL con `editor::restrict_dir_to_owner` **prima** di qualsiasi scrittura, come `db::auth::write_private`. Su FAT32/exFAT nessun permesso, come oggi, con un avviso nel log.
- Restano globali in `data\app\`: `settings.json`, `services.json`, `components.txt`, `versions\`, `frameworks.json`, `extensions*.txt`, `root.txt`, `logs\`, `db\secrets.json` senza il campo `projects` (solo `root` e `admin`), `db\my.ini`.
- La chiave delle porte nel file del progetto è solo `<azione>`: il nome del progetto è la cartella. In memoria `Runner` continua a usare `key_of(project, action)`.

### 4.2 Temporanei nella cartella del sistema (D-90)

#### 4.2.1 Dove

- `Kit::system_temp()` = `<TEMP dell'host>\Atollo\<kit.id()>\`. `kit.id()` esiste già: due kit sullo stesso PC non si toccano.
- Il valore viene da `Host::capture()` in `main.rs`, che legge l'ambiente **prima** che D-04 sposti `TEMP`/`TMP` del processo in `data\`. In mancanza, `GetTempPath2W`. Mai `std::env::temp_dir()` dopo l'avvio: restituirebbe la cartella del kit.
- Sottocartelle: `setup\<id>\`, `frameworks\<stamp>\`, `downloads\`, `adminer\`, `extensions\`, `theme\`, `system\` (`TEMP` dei programmi, quando scelto).
- Ogni operazione tiene aperto un file `.lock` nella sua cartella (`TempWork`, §5 WP-8); la pulizia all'avvio cancella solo cartelle senza lock e più vecchie di 24 ore.

#### 4.2.2 Cosa cambia per ogni uso

| Uso | Prima | Dopo |
| --- | --- | --- |
| Download, verifica, estrazione | `data\tmp\setup\<id>\` | `system_temp\setup\<id>\` |
| Passaggio in `data\apps\` | `rename` sullo stesso disco | `stage_into`: copia in `<dest>.atollo-new\` accanto alla destinazione, poi `replace_dir` come oggi |
| Estensioni, tema, vcruntime | `data\tmp\setup\…` | `system_temp\…` |
| Framework | `data\tmp\frameworks\` | `system_temp\frameworks\`, poi copia nel progetto (già una copia oggi) |
| Scaricati a mano (D-82) | `data\downloads\` | `system_temp\downloads\` (Q-3) |
| Caricamenti di Adminer | `data\app\adminer` | `system_temp\adminer\`; le sessioni restano in `data\app\adminer` |

`fs::rename` fra due dischi fallisce (`ERROR_NOT_SAME_DEVICE`): per questo `stage_into`. Lo scambio resta atomico; una copia interrotta lascia solo `.atollo-new`, che `archive::clean_leftovers` già rimuove.

#### 4.2.3 Controlli prima di cominciare

- **Disco del kit**: `GetVolumeInformationW` (`FAT32`, `exFAT`, `NTFS`, `ReFS`) e `GetDriveTypeW` (fisso, rimovibile, rete), letti una volta all'avvio e mostrati nelle Impostazioni.
- **File oltre 4 GiB su FAT32**: dopo l'estrazione e prima della copia; un file troppo grande ferma l'installazione con un errore chiaro, senza toccare la versione installata.
- **Spazio**: `GetDiskFreeSpaceExW` sul temp (archivio × 3) e sul kit (albero estratto). Temp insufficiente → `data\tmp` con avviso; kit insufficiente → errore.

#### 4.2.4 Rust su FAT32/exFAT

1. `RUSTUP_HOME`/`CARGO_HOME` in `system_temp\setup\rust\`, `rustup-init` lì.
2. Copia dell'albero in `data\apps\…\rust` con `stage_into` (i collegamenti fisici diventano copie: più spazio, stesso funzionamento).
3. Aggiornamento: copia del `rust` del kit nel temp, `rustup update`, copia di ritorno. Solo su FAT/exFAT; su NTFS resta l'aggiornamento sul posto.

#### 4.2.5 `TEMP` dei programmi del kit: «Automatico» (Q-4)

| | Nel kit (`data\tmp`) | Nel sistema (`%TEMP%\Atollo\<id>`) |
| --- | --- | --- |
| **Sicurezza** | Su FAT32/exFAT nessun permesso: chi ha la chiavetta legge i resti (codice, output degli agenti, a volte token nei log di npm/pip). Su NTFS valgono i permessi della cartella del kit, spesso larghi | Cartella dell'utente di Windows: solo l'utente, SYSTEM e gli amministratori. Resta sul PC ospite fino alla pulizia |
| **Portabilità** | Nessuna traccia sul PC (D-04) | File di passaggio sul PC, cancellati alla chiusura e dalla pulizia all'avvio |
| **Velocità** | Lenta su chiavetta con migliaia di file piccoli; su disco interno NTFS uguale al sistema | Disco interno: veloce sempre, niente limite di 4 GiB, collegamenti fisici |

Regola «Automatico»:

- kit su disco **fisso NTFS/ReFS** → `data\tmp`, con DACL protetto su `data\tmp`;
- kit su disco **rimovibile**, **di rete** o **FAT32/exFAT** → `system_temp\system\`.

Impostazione «Temporanei dei programmi: Automatico / Kit / Sistema» (predefinita Automatico). Stessa regola per `CLAUDE_CODE_TMPDIR`. Alla chiusura dell'app (il job ha già chiuso i programmi) `system_temp\system\` si cancella.

#### 4.2.6 Portabilità

D-90 è un'eccezione limitata a D-00 e D-04: solo file di passaggio, cancellati a fine operazione, alla chiusura e dalla pulizia all'avvio; mai stato, impostazioni, segreti o file che servono alla prossima apertura; senza la cartella del sistema il kit funziona come oggi.

## 5. Pacchetti di lavoro

Ogni pacchetto (WP) ha: obiettivo, dipendenze, file, compiti numerati, criteri di accettazione, test. I commenti nel codice in inglese come il resto del codice; i testi dell'interfaccia in italiano con le traduzioni in `ui/src/locales` e `i18n.rs`.

### WP-S Alias dei siti: un tipo, un controllo, nessuna uscita dai progetti

**Stato: fatto il 15/10/2026, tranne TS.6.** `cargo clippy -D warnings` pulito, `cargo test` 279 superati. Differenze dal disegno qui sotto:

- TS.2: `Action.aliases` resta `BTreeMap<String, String>` (nessun cambiamento ai file e all'interfaccia); sono i generatori ad accettare solo `&[SiteAlias]`, che si ottiene solo da `alias::resolve`. Stesso effetto: un alias non controllato non compila.
- TS.3: niente `fs::canonicalize`, che segue le giunzioni (anche verso `\\server`, D-77). Il risolutore usa `safe_path::lexical` e `safe_path::is_dir` dalla cartella `projects\`, come `detect::overrides::inside`.
- TS.6 (messaggio nella pagina del sito) **da fare**: oggi un alias saltato lascia solo un avviso nel log. Passa ad A5 con l'ondata 3.

- **Obiettivo.** Rendere impossibile, per costruzione, scrivere nella configurazione di Apache o Nginx un alias che esca da `projects\` o che aggiunga direttive; far funzionare gli alias verso un altro progetto (§1.3).
- **Dipende da.** Nulla. Indipendente dal resto del piano: va nell'ondata 0, così WP-1 trova `projects/apache.rs` già corretto.
- **File.** `projects/alias.rs` (nuovo), `detect/overrides.rs`, `detect/mod.rs` (tipo del campo `aliases`), `projects/apache.rs`, `projects/nginx.rs`, `servers/mod.rs`; riga `mod alias;` dal coordinatore.
- **Compiti.**
  - TS.1 `projects/alias.rs`: tipi che si costruiscono solo da un testo valido.

    ```rust
    /// `/media`: `/`, then ASCII letters, digits, `-_./`; no `..`, no trailing `/`.
    pub struct AliasUrl(String);
    /// Where an alias points: inside the project or inside another project.
    pub enum AliasTarget {
        Project(RelPath),                       // `storage/uploads`, `.`
        Other { project: String, path: RelPath }, // `@common/assets`
    }
    impl AliasUrl { pub fn parse(s: &str) -> Option<AliasUrl>; }
    impl AliasTarget { pub fn parse(s: &str) -> Option<AliasTarget>; }
    ```

    Regole di `valid_alias_url` e `valid_alias_target` di oggi, più: nome dell'altro progetto con `validate_project_name`; ogni parte di `RelPath` non vuota, non `..`, senza caratteri fuori da `[A-Za-z0-9._-]`. `@` resta l'unica forma per un altro progetto; `../` sempre rifiutato.
  - TS.2 `valid_alias_url`/`valid_alias_target` diventano `parse(..).is_some()`; `Action.aliases` diventa `BTreeMap<AliasUrl, AliasTarget>` (serializzato come oggi, testo ↔ testo), così un alias non controllato non compila.
  - TS.3 Un solo risolutore, `AliasTarget::resolve(project_root, projects_root) -> Option<PathBuf>`: percorso canonico (`fs::canonicalize`, poi `\\?\` tolto), esistente, dentro `projects_root` canonico, senza punti di analisi (collegamenti, giunzioni) lungo il percorso, con lo stesso controllo di `detect::overrides::inside` / `os::safe_path`. Altrimenti `None` e avviso nel log con progetto e alias.
  - TS.4 Generatori: `apache::alias_directives`, `nginx::alias_locations` e il sito del pool in `servers/mod.rs` usano `resolve`; la copia in linea di `servers/mod.rs:214-238` sparisce a favore di `apache::alias_directives`. Il testo scritto nella configurazione è `${CP_PROJECTS_FWD}/<progetto>/<percorso>` (Apache, portabile) o il percorso risolto (Nginx), costruito solo da parti già controllate.
  - TS.5 Domini: `servers/mod.rs:192-204` (`ServerAlias`) e `apache.rs` filtrano di nuovo con `valid_domain` al momento della scrittura (oggi solo alla lettura).
  - TS.6 Messaggio nella pagina del sito per un alias saltato («L'alias /media punta fuori dai progetti o a un collegamento: non servito»), attraverso il risultato del comando che avvia il sito.
- **Accettazione.** Nessuna stringa di alias arriva ai generatori senza essere passata da `parse` e `resolve`; un alias `@altro/cartella` funziona con Apache, Nginx e i pool; i test di oggi passano con `@common/assets` al posto di `../common/assets`.
- **Test.** Rifiutati da `parse`: `../x`, `../../Windows`, `C:/x`, `\\srv\x`, `/x`, `@../x`, `@altro/../x`, `@con/x`, `a"b`, `a\nb`, `${X}`, `a;b`, `a{b`. Rifiutati da `resolve`: cartella inesistente; giunzione nel progetto verso una cartella fuori (crate `junction`, già usato in `agents/sync`); altro progetto inesistente. Accettati: `storage/uploads`, `.`, `@common/assets`. Configurazioni generate di Apache, Nginx e pool senza `..` né caratteri fuori dall'insieme ammesso (controllo sul testo completo).

### WP-0 Contratti comuni

- **Obiettivo.** Fissare le API che gli altri pacchetti usano, così possono lavorare in parallelo.
- **Dipende da.** Nulla.
- **File.** `kit.rs`, `projects/state.rs` (nuovo), `os/volume.rs` (nuovo), `projects/mod.rs` e `os/mod.rs` (solo le righe `mod`), `Cargo.toml` (feature `Win32_Storage_FileSystem` se manca).
- **Compiti.**
  - T0.1 `Kit::project_data()` = `data\projects`; `Kit::project_states()` resta come alias deprecato fino a WP-6 (serve alla migrazione).
  - T0.2 `projects/state.rs`: `ProjectState { dir }` con `of`, `settings_file`, `ports_file`, `apache_dir`, `private_dir` (DACL prima di tornare), `backups_dir`, `remove`, `rename`; `all(kit)`. Nomi validati con `validate_project_name`.
  - T0.3 `os/volume.rs`: `VolumeInfo { fs: FsKind, drive: DriveKind, free: u64 }`, `fn info(path) -> io::Result<VolumeInfo>`, `fn same_volume(a, b) -> bool`, `FsKind::max_file_size()`.
  - T0.4 `Kit` riceve `system_temp: Option<PathBuf>` e `volume: Option<VolumeInfo>` (costruttore `with_host`); `Kit::system_temp()` restituisce `Option<PathBuf>`.
- **Accettazione.** Compila; nessun comportamento cambiato; firme documentate.
- **Test.** `ProjectState::of` rifiuta `..`, `a/b`, `con`; `private_dir` crea la cartella; `same_volume` vero per due cartelle dello stesso disco; `info` su `C:\` restituisce NTFS.

### WP-1 Impostazioni e Apache in `data\projects`

- **Dipende da.** WP-0.
- **File.** `projects/settings.rs`, `projects/apache.rs`, `versions/choice.rs`, `versions/mod.rs`.
- **Compiti.**
  - T1.1 `settings::dir/load/update/remove` su `ProjectState`.
  - T1.2 `apache.rs`: cartella da `ProjectState::apache_dir`; riscrittura della `httpd.conf` esistente con il nuovo percorso alla prossima scrittura.
  - T1.3 `versions/choice.rs` e `versions/mod.rs`: scorrono `state::all(kit)`.
- **Accettazione.** Il test `saved_in_data_and_kept` passa con `data\projects\app\dev-settings.json`.
- **Test.** Esistenti aggiornati; versioni scelte lette da due progetti.

### WP-2 Porte per progetto

- **Dipende da.** WP-0.
- **File.** `projects/run.rs`.
- **Compiti.**
  - T2.1 `ProjectPorts { ports: BTreeMap<azione, u16>, locked: Vec<azione>, other }` in `data\projects\<p>\ports.json`.
  - T2.2 Caricamento all'avvio di tutti i file in `Ports` in memoria con `key_of`; `save_ports` scrive solo i progetti cambiati (confronto con l'ultima istantanea salvata).
  - T2.3 Riga del comando di `httpd.exe` (`run.rs:381`) con il percorso di `ProjectState`.
- **Accettazione.** Due progetti non ricevono mai la stessa porta; una porta bloccata resta tale dopo il riavvio.
- **Test.** Conflitto fra due progetti; riavvio simulato (salva, ricarica); progetto senza file.

### WP-3 Password per progetto

- **Dipende da.** WP-0.
- **File.** `db/auth.rs`.
- **Compiti.**
  - T3.1 `Secrets.projects` non più serializzato nel file globale; `project_password(kit, p, engine)` e `set_project_password` leggono e scrivono `private\db.json` con `write_private`.
  - T3.2 `drop_project(kit, p)` restituisce le voci tolte (per WP-5).
- **Accettazione.** `secrets.json` contiene solo `root` e `admin`; nessuna password nel log.
- **Test.** Scrittura e lettura; DACL applicato prima del file (verifica con `GetFileSecurityW` su NTFS); su errore DACL solo avviso.

### WP-4 Copie dei database nel progetto

- **Dipende da.** WP-0.
- **File.** `db/services/backups.rs`.
- **Compiti.**
  - T4.1 `backup_dir(project)` = `ProjectState::backups_dir()` (sotto `private\`).
  - T4.2 «Ripristina…» da un file esterno: copia in `backups_dir` come oggi.
- **Accettazione.** Esporta e ripristina funzionano; i file nascono privati.
- **Test.** Elenco, esporta (mock del comando), `safe_path` rifiuta nomi fuori cartella.

### WP-5 Eliminazione del progetto con i suoi database (Q-2)

- **Dipende da.** WP-3, WP-4.
- **File.** `projects/mod.rs` (`delete_empty`), `db/mod.rs` o `db/services/mod.rs` (drop), `commands/projects.rs`, `ui/src/api/projects.ts`, `ui/src/projects/page.tsx`.
- **Compiti.**
  - T5.1 `delete_plan(kit, p)`: cosa sparisce (cartella, motori con password, numero e dimensione delle copie).
  - T5.2 Drop per motore: avvio del server se fermo; MariaDB `DROP DATABASE IF EXISTS` + `DROP USER IF EXISTS` con il login di `my.ini`; MongoDB `dropDatabase()` + `dropUser()` con `admin`. Motore non installato: saltato.
  - T5.3 Ordine: drop → cartella del progetto → `ProjectState::remove`. Un drop fallito ferma tutto; `force_without_db` cancella le cartelle e scrive nel log quali database restano.
  - T5.4 Interfaccia: finestra di conferma con l'elenco di T5.1, «Apri la cartella delle copie», «Elimina», e dopo un errore «Elimina senza i database».
- **Accettazione.** Dopo l'eliminazione nessun file in `data\projects\<p>\`, nessun database né utente del progetto nei server.
- **Test.** Server fermo (si avvia), server che non parte (nulla cancellato), motore non installato, progetto senza database; test Vitest della finestra.

### WP-6 Migrazione 1 → 2

- **Dipende da.** WP-1, WP-2, WP-3, WP-4 (formati finali).
- **File.** `projects/migrate.rs` (nuovo); chiamata in `main.rs` dopo `Kit::prepare`.
- **Compiti.**
  - T6.1 Se `data\app\layout.txt` vale `2`: niente.
  - T6.2 Copia di sicurezza `data\app\pre-v2\` di `ports.json` e `secrets.json`.
  - T6.3 `data\app\projects\<p>\` → `data\projects\<p>\` con `rename`; destinazione esistente: voce per voce senza sovrascrivere.
  - T6.4 `ports.json` diviso per progetto (anche `locked`).
  - T6.5 `secrets.json`: ogni voce di `projects` in `private\db.json`; il file globale riscritto senza `projects` solo dopo tutti i file per progetto.
  - T6.6 `data\db\backup\<p>\` → `private\backups\`: prima `private_dir()` (DACL), poi spostamento voce per voce perché i file ereditino i permessi.
  - T6.7 `layout.txt = 2`, poi cancellazione dei vecchi percorsi.
  - T6.8 Errore a metà: log, nuovo tentativo all'avvio dopo; nel frattempo le letture cercano prima il nuovo percorso e poi il vecchio (fallback in `ProjectState`, tolto quando `layout.txt = 2`).
- **Accettazione.** Nessuna chiave persa; migrazione idempotente.
- **Test.** Kit di prova con tutti i file vecchi; interruzione simulata dopo T6.4 e ripresa; secondo avvio senza effetti; destinazione già presente.

### WP-7 Progetti orfani

- **Dipende da.** WP-0 (backend), WP-5 (eliminazione).
- **File.** `projects/state.rs` (`orphans`, `adopt`), `commands/orphans.rs` (nuovo, registrato dal coordinatore), `ui/src/projects/page.tsx`.
- **Compiti.**
  - T7.1 `orphans(kit)`: cartelle di `data\projects\` senza `projects\<p>\`.
  - T7.2 `adopt(kit, orphan, project)`: sposta i file su un progetto senza dati del kit; le password restano valide (stesso utente del database, nome nuovo solo nei file del kit).
  - T7.3 Sezione chiusa «Dati senza progetto» nella pagina Progetti con «Collega a…» ed «Elimina» (stessa conferma di WP-5).
  - T7.4 Mai cancellazione automatica.
- **Test.** Rinomina fuori dall'app, poi «Collega a…»; elimina orfano con database.

### WP-8 Temp di sistema, disco del kit, pulizia

- **Dipende da.** WP-0.
- **File.** `os/temp.rs` (nuovo), `main.rs` (cattura e pulizia, a cura del coordinatore), `ui/src/settings/sections.tsx` (riga informativa).
- **Compiti.**
  - T8.1 `TempWork::new(kit, "setup/<id>")`: cartella in `system_temp` (o `data\tmp` di ripiego), `.lock` aperto finché vive, cancellazione al `Drop`.
  - T8.2 Ripiego: `system_temp` assente, non scrivibile o senza spazio → `data\tmp\…` con avviso nel log.
  - T8.3 `sweep(kit)` all'avvio: cartelle senza lock più vecchie di 24 ore.
  - T8.4 Alla chiusura: `system_temp\system\` cancellata.
  - T8.5 Impostazioni: file system e tipo di disco del kit.
- **Test.** Lock rispettato da `sweep`; due kit con id diversi; ripiego con `system_temp` in una cartella in sola lettura.

### WP-9 `stage_into` e controlli

- **Dipende da.** WP-0.
- **File.** `net/archive.rs`.
- **Compiti.**
  - T9.1 `stage_into(tree, dest)`: stesso volume → `replace_dir` come oggi; altro volume → copia in `<dest>.atollo-new` (`retry_denied` sulla copia), poi `replace_dir`.
  - T9.2 `check_fits(tree, dest_volume)`: file più grande di `FsKind::max_file_size()` → errore con nome e dimensione.
  - T9.3 `check_space(tree, dest_volume)`.
- **Test.** Copia fra due cartelle simulate su volumi diversi (parametro iniettato a `same_volume`); albero con un file oltre il limite (file sparso); copia interrotta → solo `.atollo-new`, ripulito da `clean_leftovers`.

### WP-10 Installazioni e download nel temp

- **Dipende da.** WP-8, WP-9.
- **File.** `install/mod.rs`, `install/web.rs`, `editor/ext.rs`, `editor/theme.rs`, `commands/frameworks.rs`, `setup/downloads.rs`, `db/adminer.rs`.
- **Compiti.**
  - T10.1 `install::work_dir` → `TempWork`; il passaggio in `data\apps` con `stage_into`.
  - T10.2 Estensioni, tema, vcruntime su `TempWork`.
  - T10.3 Framework in `system_temp\frameworks\<stamp>`.
  - T10.4 `setup/downloads.rs`: `dir(kit)` in `system_temp\downloads`; i file di `data\downloads\` esistenti spostati lì al primo uso.
  - T10.5 Adminer: `upload_tmp_dir` in `system_temp\adminer`.
- **Accettazione.** Nessuna scrittura in `data\tmp` quando `system_temp` è disponibile, salvo `TEMP` dei programmi in modalità Kit.
- **Test.** Installazione di un componente piccolo (mock di rete esistente) con `system_temp` su un'altra cartella; file aggiunto a mano trovato.

### WP-11 `TEMP` dei programmi «Automatico»

- **Dipende da.** WP-0, WP-8.
- **File.** `env.rs`, `settings.rs`, `ui/src/settings/sections.tsx`.
- **Compiti.**
  - T11.1 `Settings.program_temp: Auto | Kit | System` (predefinito `Auto`).
  - T11.2 `env.rs`: `TEMP`, `TMP`, `CLAUDE_CODE_TMPDIR` secondo la regola di §4.2.5.
  - T11.3 Modalità Kit su NTFS: DACL protetto su `data\tmp`.
  - T11.4 Impostazione nella pagina con la spiegazione breve di §4.2.5.
- **Test.** Tabella attesa di `env.rs` per NTFS fisso, exFAT rimovibile, NTFS rimovibile, rete, e per le tre scelte.

### WP-12 Rust su FAT32/exFAT

- **Dipende da.** WP-8, WP-9.
- **File.** `install/native.rs`.
- **Compiti.**
  - T12.1 Kit senza collegamenti fisici (`can_hard_link` falso): installazione in `TempWork("setup/rust")` e `stage_into`.
  - T12.2 Aggiornamento: copia nel temp, `rustup update`, `stage_into` di ritorno.
  - T12.3 Temp di sistema anch'esso senza collegamenti fisici: errore di oggi.
- **Test.** Manuale su chiavetta exFAT (TESTING.md): installazione, aggiornamento, `cargo build` di un progetto di prova.

### WP-13 Documenti

- **Dipende da.** Tutti.
- **File.** `docs/app/DECISIONS.md`, `ARCHITECTURE.md`, `CHANGELOG.md`, `TESTING.md`, questo piano (stato «completato»).
- **Compiti.** D-89 e D-90; note **Oggi** su D-00, D-04, D-63, D-74, D-78, D-82, D-83; nota **Oggi** su D-77 per gli alias dei siti (WP-S: forma `@altro/cartella`, nessun collegamento seguito, controllo anche alla scrittura della configurazione); voce Rust su FAT/exFAT chiusa nel CHANGELOG; nota sul ritorno a una versione precedente (`data\app\pre-v2\`); prove manuali in TESTING.md (§6).

### WP-14 Verifica finale

- **Dipende da.** Tutti.
- **Compiti.** `npm run lint`, `npm test`, `npm -w ui run build`; prove manuali di §6; revisione di sicurezza del diff completo (`/security-review`).

## 6. Prove manuali (in TESTING.md)

- [ ] Kit con i dati della versione 1 (copia di un kit reale): avvio, migrazione, progetti, porte, password e copie intatti.
- [ ] Kit su chiavetta **exFAT**: installazione di Node, Python, Java; `TEMP` dei programmi nel sistema; nessun file in `data\tmp` dopo la chiusura.
- [ ] Kit su chiavetta **FAT32**: componente con un file oltre 4 GiB rifiutato con errore chiaro, versione installata intatta.
- [ ] Rust su exFAT: installazione, aggiornamento, `cargo build`.
- [ ] Eliminazione di un progetto con MariaDB e MongoDB: server fermo, database e utente tolti.
- [ ] Progetto rinominato in Esplora file: compare fra gli orfani, «Collega a…» lo riporta.
- [ ] C: quasi pieno: le installazioni ripiegano su `data\tmp` con avviso.
- [ ] Sito con alias `/media` → `storage/uploads` e `/shared` → `@common/assets`, con Apache, con Nginx e in un pool: i file si aprono nel browser.
- [ ] `storage` sostituito da una giunzione verso `C:\Users`: l'alias non è servito e la pagina del sito lo dice.

## 7. Sciame di agenti

### 7.1 Regole

- **Proprietà esclusiva dei file.** Ogni file ha un solo agente che lo scrive in un'ondata; gli altri lo leggono soltanto. I file condivisi (righe `mod`, `main.rs`, `commands/mod.rs`, `Cargo.toml`, `ui/src/locales/*`, `i18n.rs`) li scrive solo il coordinatore, su richiesta nel rapporto degli agenti.
- **Un worktree per agente** (`isolation: "worktree"`), branch `data/<agente>`, partito dal branch d'integrazione `feature/data-layout` all'inizio dell'ondata.
- **Contratti prima del codice.** WP-0 si chiude prima di ogni altro pacchetto; un agente che ha bisogno di cambiare un contratto si ferma e lo chiede al coordinatore.
- **Integrazione in ordine fisso**, un merge alla volta, con verifica dopo ciascuno.
- **Uscita di ogni agente**: `cargo fmt`, `cargo clippy --all-targets -- -D warnings` e i test del proprio modulo verdi nel suo worktree; rapporto nel formato di §7.4.
- **Al massimo 5 agenti per ondata.**

### 7.2 Agenti

| Id | Ruolo | Pacchetti | Scrive | Legge | Modello / sforzo | Note |
| --- | --- | --- | --- | --- | --- | --- |
| **C** | Coordinatore (sessione principale) | ondate, merge, WP-14 | file condivisi, `main.rs`, `feature/data-layout` | tutto | Opus, alto | Unico che fa merge; decide sui conflitti di contratto; avvia gli agenti |
| **AS** | Sicurezza dei siti | WP-S | `projects/alias.rs`, `detect/overrides.rs`, `detect/mod.rs`, `projects/apache.rs`, `projects/nginx.rs`, `servers/mod.rs` | `os/safe_path.rs`, `agents/sync/tree.rs` (giunzioni), D-77 | Opus, alto | Validazione per costruzione con tipi; un solo risolutore; nessun collegamento seguito; test d'attacco prima della correzione |
| **A1** | Architetto dei contratti | WP-0 | `kit.rs`, `projects/state.rs`, `os/volume.rs` | `db/auth.rs`, `editor/mod.rs` (DACL), `projects/*` | Opus, alto | API piccole e documentate; nessun cambiamento di comportamento |
| **A2** | Stato dei progetti | WP-1, WP-2 | `projects/settings.rs`, `projects/apache.rs`, `projects/run.rs`, `versions/choice.rs`, `versions/mod.rs` | `projects/state.rs` | Sonnet, medio | Lavoro meccanico sui percorsi; attenzione al formato di `ports.json` |
| **A3** | Database | WP-3, WP-4, WP-5 (backend) | `db/auth.rs`, `db/services/backups.rs`, `db/services/mod.rs`, `projects/mod.rs`, `commands/projects.rs` | `db/*`, `projects/state.rs` | Opus, alto | Segreti e DACL; drop dei database con conferma; mai password in log o riga di comando |
| **A4** | Migrazione e orfani | WP-6, WP-7 (backend) | `projects/migrate.rs`, `projects/state.rs` (solo `orphans`, `adopt`, fallback), `commands/orphans.rs` | formati di A2 e A3 | Opus, alto | Idempotenza e ripresa; prove con interruzioni simulate |
| **A5** | Interfaccia | WP-5 (UI), WP-7 (UI), WP-8 (T8.5), WP-11 (T11.4), WP-S (TS.6) | `ui/src/projects/page.tsx`, `ui/src/api/projects.ts`, `ui/src/settings/*` | comandi di A3, A4, A6 | Sonnet, medio | Controlli di VSCodium (D-88); testi italiani consegnati al coordinatore per le traduzioni; test Vitest |
| **A6** | Temp e ambiente | WP-8 (backend), WP-11 (backend) | `os/temp.rs`, `env.rs`, `settings.rs` | `kit.rs`, `os/volume.rs` | Sonnet, alto | Regola «Automatico»; tabella attesa di `env.rs` |
| **A7** | Installazioni | WP-9, WP-10 | `net/archive.rs`, `install/mod.rs`, `install/web.rs`, `editor/ext.rs`, `editor/theme.rs`, `commands/frameworks.rs`, `setup/downloads.rs`, `db/adminer.rs` | `os/temp.rs` (contratto di WP-0/WP-8) | Opus, medio | `stage_into` atomico; nessuna regressione su NTFS |
| **A8** | Rust | WP-12 | `install/native.rs` | `net/archive.rs`, `os/temp.rs` | Sonnet, medio | Solo il ramo FAT/exFAT; NTFS invariato |
| **R** | Revisore | ogni merge | nulla | diff dell'agente | Opus, alto | Correttezza e sicurezza: ordine DACL → scrittura, segreti, percorsi (`safe_path`, collegamenti, D-77), copie fra volumi; risultato «approvato» o elenco di problemi con file e riga |
| **V** | Verificatore | dopo ogni merge, WP-14 | nulla (solo comandi) | branch d'integrazione | Sonnet, basso | `npm run lint`, `npm test`, `npm -w ui run build`; riporta l'output esatto dei fallimenti |
| **D** | Documentazione | WP-13 | `docs/app/*.md` | rapporti degli agenti, diff finale | Sonnet, medio | Stile di DECISIONS.md (scelta, motivo, scartata); italiano |

Il revisore e il verificatore non scrivono mai codice: un problema torna all'agente proprietario del file.

### 7.3 Ondate

```text
Ondata 0   AS (WP-S, fatto) │ A1 (WP-0)                → merge AS, poi A1; R e V dopo ciascuno
Ondata 1   A2 (WP-1, WP-2) │ A3 (WP-3, WP-4) │ A6 (WP-8, WP-11) │ A7 (WP-9 → WP-10)
                                                        → merge in ordine A2, A3, A6, A7; R e V dopo ciascuno
Ondata 2   A3 (WP-5) │ A4 (WP-6, WP-7) │ A8 (WP-12)    → merge A3, A4, A8; R e V
Ondata 3   A5 (tutta l'interfaccia)                     → merge, R, V
Ondata 4   D (WP-13) │ C (WP-14: verifica completa, /security-review, prove manuali con l'utente)
```

- In ondata 0 `AS` e `A1` non hanno file in comune; `AS` si fonde per primo perché `projects/apache.rs` passa poi ad `A2` in ondata 1. Il revisore controlla `AS` con i test d'attacco di WP-S.
- In ondata 1 `A7` usa `TempWork` dalla firma fissata in WP-0: se `A6` non l'ha ancora fusa, compila contro lo stub di WP-0.
- In ondata 2 `projects/state.rs` appartiene solo ad `A4`; `A3` scrive `projects/mod.rs`, `db/*` e `commands/projects.rs`, e chiede ad `A4` (tramite il coordinatore) ciò che gli serve in `state.rs`.
- L'interfaccia viene per ultima perché dipende dai comandi di tre agenti; i testi passano al coordinatore per `ui/src/locales`.
- Le prove manuali su chiavetta FAT32/exFAT richiedono l'utente: il coordinatore le prepara come lista e aspetta l'esito.

### 7.4 Richiesta all'agente e rapporto

Richiesta (scritta dal coordinatore per ogni agente):

```text
Obiettivo: <WP e compiti, copiati da PLAN-DATA.md §5>
Branch: data/<agente> da feature/data-layout@<commit>
Scrivi solo: <file>
Leggi: <file>
Contratti: <firme di WP-0 da non cambiare>
Vincoli: commenti in inglese come il codice; nessun segreto nel log; DACL prima delle scritture
         private; nessun file nuovo nei progetti; stile del codice vicino.
Fatto quando: cargo fmt, clippy -D warnings e i test del modulo verdi; criteri di accettazione del WP.
Fermati e chiedi se: serve cambiare un contratto o un file di un altro agente.
```

Rapporto (dall'agente al coordinatore):

```text
Stato: fatto | bloccato (<motivo>)
Commit: <hash>
File cambiati: <elenco>
Test aggiunti: <nomi>
Righe per il coordinatore: mod/registrazioni/Cargo.toml da aggiungere
Testi nuovi dell'interfaccia: <chiave italiana> (per le traduzioni)
Rischi o dubbi: <elenco>
```

### 7.5 Esecuzione

Lo sciame si avvia con la richiesta esplicita dell'utente («usa un workflow» o simile): ogni ondata è un gruppo di agenti in parallelo, seguito da revisione e verifica. Senza quella richiesta il coordinatore esegue le ondate una alla volta con agenti singoli o direttamente.

## 8. Rischi

| Rischio | Effetto | Contromisura |
| --- | --- | --- |
| Antivirus sul temp di sistema | Spostamento rifiutato per qualche secondo | `retry_denied` anche sulla copia di `stage_into` |
| C: quasi pieno | Installazione fallita | Controllo di spazio e ripiego su `data\tmp` (T8.2) |
| Installazione interrotta su un PC | Resti nel temp di quel PC | `sweep` all'avvio successivo lì; Sensore memoria di Windows |
| Password durante la migrazione | Perdita o fuga | `secrets.json` riscritto per ultimo; copia `pre-v2\`; nessuna password nel log (revisore) |
| Ritorno a una versione precedente dell'app | Non trova `data\projects\` | Nota nel CHANGELOG; `pre-v2\` per tornare a mano |
| Drop di un database per errore | Dati persi | Conferma con elenco; copie mostrate prima; drop solo del database del progetto (`db_name`) |
| Conflitti di merge fra agenti | Lavoro perso | Proprietà esclusiva dei file; file condivisi solo al coordinatore |
| Alias dei siti verso fuori da `projects\` | File del disco serviti dal server del kit (anche in LAN con `ATOLLO_ALLOW_LAN`) | WP-S: tipi validati, risolutore unico con percorso canonico e senza collegamenti |
| `panel.json` esistenti con alias non più accettati | Alias saltati dopo l'aggiornamento | Nessuno possibile oggi (`valid_alias_target` già rifiuta `../`); avviso nel log e nella pagina del sito |
