# 03 — Bug, Sicurezza e Funzionalità Mancanti (`v4`)

> **Riferimento Codice**: `HEAD 18698f9` · **Totale Rilievi `v4`**: **25** (`4` Sicurezza `SEC-V4-01..04`, `13` Bug Logici/Concorrenza `BUG-V4-01..13`, `8` Portabilità/Multi-OS `PORT-V4-01..08` approfonditi anche nel Doc 04).

---

## 1. Riepilogo per Severità

| Categoria | Critica | Alta | Media | Bassa | Totale |
|---|:---:|:---:|:---:|:---:|:---:|
| **Sicurezza Locale (`SEC-V4-*`)** | 0 | 1 | 2 | 1 | **4** |
| **Logica Backend, Pool, DB & Detection (`BUG-V4-*`)** | 0 | 5 | 7 | 1 | **13** |
| **Portabilità 100% & Multi-OS (`PORT-V4-*`)** | 4 | 2 | 2 | 0 | **8** |
| **TOTALE RILIEVI `v4`** | **4** | **8** | **11** | **2** | **25** |

---

## 2. Vulnerabilità e Gap di Sicurezza (`SEC-V4-01..04`)

### `SEC-V4-01` [ALTA] — Esecuzione di codice PHP (RCE) tramite Alias di Upload in Apache e Intercettazione Regex in Nginx
- **File e Righe**:
  - [`tauri/src/projects/apache.rs:197-205`](../../../tauri/src/projects/apache.rs#L197-L205) (`alias_directives`)
  - [`tauri/src/projects/nginx.rs:95-114, 152-160`](../../../tauri/src/projects/nginx.rs#L95-L114) (`alias_locations`)
  - [`tauri/src/servers/mod.rs:416-435, 1273-1284`](../../../tauri/src/servers/mod.rs#L416-L435) (pool condivisi)
- **Analisi Tecnica**:
  1. **Apache**: nel blocco globale di `httpd.conf` è definito `<FilesMatch "\.(?:php|phtml|phar)$"> SetHandler application/x-httpd-php </FilesMatch>`. Le sezioni `<Directory "{target}">` generate da `alias_directives` per gli alias di progetto (usati tipicamente per montare cartelle di upload/media come `/storage` o `/uploads` fuori dalla document root) **ereditano** il `SetHandler` globale e abilitano inoltre `AllowOverride FileInfo`. Se un'applicazione web consente l'upload di un file con estensione `.phtml`, `.phar` o `.php` (o di un `.htaccess` con `SetHandler`), Apache lo esegue come codice PHP.
  2. **Nginx**: `alias_locations` genera `location /media/ { alias "..."; }` **senza** il modificatore prefisso `^~`. In Nginx le location regex (`location ~* \.(?:php|phtml|phar)$`) hanno priorità sulle location prefisso ordinarie: una richiesta a `/media/shell.php` viene quindi catturata dal blocco FastCGI principale (con `$document_root` della root del progetto anziché dell'alias) o, se si usa `$request_filename`, passata a `php-cgi`.
- **Patch Proposta**:
  - In `apache.rs` (`alias_directives`), rimuovere `FileInfo` da `AllowOverride` (`AllowOverride Indexes Limit`) e aggiungere dentro `<Directory "{target}">`:
    ```apache
    <FilesMatch "\.(?:php|phtml|phar|inc|php[34578]|phps)$">
        SetHandler None
        Require all denied
    </FilesMatch>
    ```
  - In `nginx.rs` (`alias_locations`), usare il modificatore `^~` (`location ^~ {url} { ... }`) così che nessuna location regex PHP venga valutata per gli URL sotto l'alias, e bloccare esplicitamente `\.(?:php|phtml|phar|inc|php[34578]|phps)$` con `return 403;`.

---

### `SEC-V4-02` [MEDIA] — `runtime/php-router.php` blocca la modalità LAN (`ATOLLO_ALLOW_LAN=1`), ammette `Host` vuoto e non avvisa quando un `router.php` di progetto lo sostituisce
- **File e Righe**:
  - [`runtime/php-router.php:17-27`](../../../runtime/php-router.php#L17-L27)
  - [`tauri/src/detect/php.rs:64-137`](../../../tauri/src/detect/php.rs#L64-L137)
  - [`tauri/src/projects/run.rs:195-215`](../../../tauri/src/projects/run.rs#L195-L215)
- **Analisi Tecnica**:
  1. Quando l'utente attiva *"Condividi in LAN"* (`allow_lan: true`), `Runner` fa il bind su `0.0.0.0:<port>`, ma [`runtime/php-router.php:19`](../../../runtime/php-router.php#L19) controlla solo `getenv('ATOLLO_ALLOW_LAN') !== '1'` senza che `projects/run.rs` imposti mai `ATOLLO_ALLOW_LAN=1` nell'ambiente del processo! Di conseguenza qualsiasi richiesta da un dispositivo sulla LAN con `Host: 192.168.1.x:<port>` viene rifiutata con `421 Misdirected Request`.
  2. La guardia `if ($host !== '' && getenv('ATOLLO_ALLOW_LAN') !== '1')` lascia passare una richiesta HTTP/1.0 priva di header `Host` (`$host === ''`).
  3. Se il progetto possiede un proprio `router.php`, `detect/php.rs` usa quello del progetto al posto di `php-router.php`, disattivando silenziosamente la protezione dotfile e DNS rebinding.
- **Patch Proposta**:
  - In `projects/run.rs`, esportare `cmd.env("ATOLLO_ALLOW_LAN", if allow_lan { "1" } else { "0" })`.
  - In `runtime/php-router.php:19`, cambiare la condizione in `if (getenv('ATOLLO_ALLOW_LAN') !== '1') { if ($host === '' || !preg_match(...)) { http_response_code(421); exit; } }`.

---

### `SEC-V4-03` [MEDIA] — `projects/serve.rs` rifiuta i sottodomini `*.localhost` con `403 Forbidden`
- **File e Righe**: [`tauri/src/projects/serve.rs:56-67`](../../../tauri/src/projects/serve.rs#L56-L67).
- **Analisi Tecnica**: `is_local_host_header` accetta solo `matches!(host, "localhost" | "127.0.0.1" | "[::1]")`. A differenza di Apache, Nginx e `php-router.php` (che accettano `<progetto>.localhost`), una richiesta a `http://mio-sito.localhost:<port>` verso il server statico integrato viene bloccata con `403 Forbidden`.
- **Patch Proposta**: estendere `is_local_host_header` con `|| host.ends_with(".localhost")`.

---

### `SEC-V4-04` [BASSA] — `php-cgi` senza `FCGI_WEB_SERVER_ADDRS=127.0.0.1` e vHost di default dei Pool Condivisi
- **File e Righe**: [`tauri/src/projects/nginx.rs:212-235`](../../../tauri/src/projects/nginx.rs#L212-L235), [`tauri/src/servers/mod.rs:640-675`](../../../tauri/src/servers/mod.rs#L640-L675).
- **Analisi Tecnica**: `spawn_php_cgi` avvia `php-cgi -b 127.0.0.1:<port>` senza impostare `FCGI_WEB_SERVER_ADDRS="127.0.0.1"`. Impostare tale variabile d'ambiente attiva il controllo nativo in `php-cgi` sui peer autorizzati e documenta il perimetro loopback. Inoltre, nei pool condivisi il primo vHost di fallback serve una pagina neutra `200 OK` anziché `421 Misdirected Request` quando riceve un `Host` estraneo (a differenza dei server dedicati in `D-98`).

---

## 3. Bug Logici, di Concorrenza e di Stato nel Backend (`BUG-V4-01..13`)

### `BUG-V4-01` [ALTA] — Disallineamento tra `site_mode` (`projects/run.rs`) e `WebServers::scan` (`servers/mod.rs`) per `server: None | "auto"` e profili eliminati
- **File e Righe**:
  - [`tauri/src/projects/run.rs:115-128`](../../../tauri/src/projects/run.rs#L115-L128) (`site_mode`)
  - [`tauri/src/servers/mod.rs:878-918`](../../../tauri/src/servers/mod.rs#L878-L918) (`WebServers::scan` e `pool_of`)
- **Analisi Tecnica**:
  - In `projects/run.rs:115-128`:
    ```rust
    fn site_mode(action: &DetectedAction) -> SiteMode {
        match action.server.as_deref().map(str::trim) {
            None | Some("") | Some("dedicated") | Some("auto") => SiteMode::Dedicated,
            Some(id) => SiteMode::Pool(id.to_string()),
        }
    }
    ```
  - Invece in `servers/mod.rs:878-888`, `WebServers::scan` tratta `None | Some("") | Some("auto")` come siti appartenenti al **pool condiviso automatico** (`auto_pool_id`).
  - Inoltre, se l'utente assegna un sito a un profilo personalizzato `my-pool` e poi elimina `my-pool` in `servers.tsx`, `action.server` nel progetto rimane `Some("my-pool")`: `site_mode` restituisce `SiteMode::Pool("my-pool")` e `pool_of` (non trovando `"my-pool"` nei profili) fa fallback al pool automatico!
- **Patch Proposta**:
  - Allineare `WebServers::scan` e `site_mode`: se il default per i progetti è `Dedicated` quando `action.server` è `None`, allora `WebServers::scan` deve includere nei pool automatici **solo** le azioni con `action.server == Some("auto")` (oppure `site_mode` deve mappare `Some("auto")` a `SiteMode::Pool("auto".into())`), e se `action.server` punta a un profilo eliminato, deve ricadere in `Dedicated` (o pulire il riferimento all'eliminazione del profilo).

---

### `BUG-V4-02` [MEDIA] — I Pool Condivisi ignorano i file `data/projects/<p>/{apache,nginx}/conf.d/*.conf` del singolo progetto
- **File e Righe**: [`tauri/src/servers/mod.rs:408-447, 1273-1284`](../../../tauri/src/servers/mod.rs#L408-L447).
- **Analisi Tecnica**: mentre i server dedicati includono `data/projects/<p>/apache/conf.d/*.conf` ([`apache.rs:185`](../../../tauri/src/projects/apache.rs#L185)) e `data/projects/<p>/nginx/conf.d/*.conf` ([`nginx.rs:163`](../../../tauri/src/projects/nginx.rs#L163)), i vHost generati dai pool condivisi includono solo il `conf.d` globale del pool, ignorando le direttive custom del progetto (`TW1.5` di [`docs/app/PLAN.md`](../../app/PLAN.md#L118)).
- **Patch Proposta**: includere `IncludeOptional "<project_apache_conf_d>/*.conf"` nel `<VirtualHost>` di `render_apache_vhost` e `include "<project_nginx_conf_d>/*.conf";` nel `server {}` di `render_nginx_vhost`.

---

### `BUG-V4-03` [MEDIA] — Assenza di validazione incrociata delle porte tra Pool Web, Progetti, Editor (`7000..=7099`) e Database + `php.ini` condiviso
- **File e Righe**: [`tauri/src/servers/mod.rs:1043-1072`](../../../tauri/src/servers/mod.rs#L1043-L1072), [`tauri/src/projects/run.rs:420-445`](../../../tauri/src/projects/run.rs#L420-L445), [`tauri/src/db/services/mod.rs:739-807`](../../../tauri/src/db/services/mod.rs#L739-L807).
- **Analisi Tecnica**:
  - `WebServers::save_profile` controlla le collisioni solo contro gli altri profili di `servers.json`, permettendo di salvare una porta già assegnata a MySQL (`3306`/`3307`), MongoDB (`27017`), range VSCodium (`7000..=7099`) o a un progetto. Analogamente `Runner::set_port` e `Services::set_ports` non verificano i pool web.
  - Inoltre tutte le linee PHP (`8.2`, `8.3`, `8.4`, `8.5`) condividono un unico `data/php/php.ini` anziché `data/php/<line>/php.ini` (con fallback a `data/php/php.ini`), per cui `extension_dir` o estensioni specifiche di una linea rompono le altre linee nei pool multi-PHP.

---

### `BUG-V4-04` [ALTA] — Race Condition su `services.json` e avvii concorrenti in `Services::start_line`
- **File e Righe**: [`tauri/src/db/services/mod.rs:150-169, 500-564, 739-807`](../../../tauri/src/db/services/mod.rs#L500-L564).
- **Analisi Tecnica**: a differenza di `WebServers` (che usa `op_lock` e `settings_lock`) e di `db.json` (che usa `db_json_lock` introdotto in `0e5e0e8`), la struct `Services` non possiede alcun mutex per serializzare le transizioni di avvio/arresto (`start_line`, `stop_line`) e i cicli read-modify-write su `data/app/db/services.json` (`remember_running`, `forget_running`, `keep_port`, `set_ports`). Se due progetti che richiedono MySQL vengono avviati contemporaneamente, entrambi vedono `running_at == false`, allocano porte diverse o lanciano due processi `mysqld` sulla stessa directory `data/db/mysql/<line>` ed effettuano sovrascritture perse su `services.json`.
- **Patch Proposta**: aggiungere `op_lock: Mutex<()>` e `cfg_lock: Mutex<()>` nella struct `Services`.

---

### `BUG-V4-05` [ALTA] — Scambio silenzioso delle porte MySQL (`Port Swap`) quando cambia la linea predefinita
- **File e Righe**: [`tauri/src/db/services/mod.rs:188-222, 538-547, 788-807`](../../../tauri/src/db/services/mod.rs#L188-L222).
- **Analisi Tecnica**:
  - In `start_line` (riga 539), `keep_port` salva la porta in `services.json` (`cfg.ports`) **solo** quando `line != default_line`:
    ```rust
    if line.as_deref() != default_line.as_deref() {
        Self::keep_port(root, l, port);
    }
    ```
  - E in `line_ports` (riga 199), la linea che coincide con `default_line` riceve sempre `cfg.mysql_port` (`3306`), ignorando `cfg.ports`.
  - **Scenario di corruzione**: il Progetto A usa MySQL `9.7` (default, porta `3306`) e il Progetto B usa MySQL `8.4` (secondaria, porta `3307`). Se l'utente imposta `8.4` come linea predefinita nella pagina *Versioni*, `line_ports` assegna `3306` a `8.4` e sposta `9.7` su `3307` (o `3308`), rompendo tutte le stringhe di connessione salvate nei `.env` dei progetti!
- **Patch Proposta**: salvare sempre in `cfg.ports` la porta effettiva di ogni linea avviata (inclusa la prima linea su `3306`) e in `line_ports` rispettare sempre `cfg.ports.get(&line)` se presente, usando `cfg.mysql_port` (`3306`) come default solo per la prima linea che non ha ancora una porta registrata in `cfg.ports`.

---

### `BUG-V4-06` [MEDIA] — `ProjectDrops::drop_db` elimina il DB del progetto solo sulla linea attualmente selezionata
- **File e Righe**: [`tauri/src/db/services/mod.rs:834-839`](../../../tauri/src/db/services/mod.rs#L834-L839).
- **Analisi Tecnica**: quando un progetto viene eliminato (o quando si cambia linea DB del progetto), `drop_db` interroga solo `project_db_line(root, project, DbKind::Mysql)`. Se il progetto aveva precedentemente creato un DB su un'altra linea MySQL, quel database e il relativo utente SQL restano orfani.

---

### `BUG-V4-07` [MEDIA] — Browser Database (`db/browse.rs`) e Adminer (`db/adminer.rs`) interrogano solo la linea MySQL primaria (`None`)
- **File e Righe**: [`tauri/src/db/browse.rs:148-181`](../../../tauri/src/db/browse.rs#L148-L181), [`tauri/src/db/services/mod.rs:657-670`](../../../tauri/src/db/services/mod.rs#L657-L670).
- **Analisi Tecnica**: `browse::list_databases`, `list_tables`, `query_table` e `Services::adminer_url` passano sempre `None` come linea a `running_port(root, DbKind::Mysql)` / `ensure_running`. Se l'utente ha avviato solo una linea secondaria di MySQL (es. `8.4` sulla porta `3307`) mentre la primaria è ferma, il Database Browser mostra MySQL come spento o avvia forzosamente la linea primaria invece di ispezionare la linea attiva.

---

### `BUG-V4-08` [BASSA] — Collisione del file temporaneo `<exe>-<pid>.cnf` tra thread concorrenti in `db/services/tools.rs`
- **File e Righe**: [`tauri/src/db/services/tools.rs:115, 139`](../../../tauri/src/db/services/tools.rs#L115).
- **Analisi Tecnica**: `write_client_cnf` genera il nome file temporaneo con `format!("{}-{}.cnf", stem, std::process::id())`. Poiché tutti i worker thread del processo Tauri condividono lo stesso PID, due query concorrenti (es. poll di stato + browser DB) sovrascrivono e cancellano lo stesso file `.cnf` mentre il processo figlio `mysql` lo sta leggendo.
- **Patch Proposta**: aggiungere un contatore atomico `static CNF_SEQ: AtomicU64 = AtomicU64::new(0);` nel nome del file: `format!("{}-{}-{}.cnf", stem, std::process::id(), CNF_SEQ.fetch_add(1, Ordering::Relaxed))`.

---

### `BUG-V4-09` [ALTA] — `detect::requires(dir)` aggiunge incondizionatamente `"apache"` su ogni progetto con `.htaccess` (incluso Laravel)
- **File e Righe**:
  - [`tauri/src/detect/mod.rs:447, 548-562, 677-687`](../../../tauri/src/detect/mod.rs#L548-L562)
  - [`tauri/src/commands/mod.rs:237-255`](../../../tauri/src/commands/mod.rs#L237-L255) (`settle_project`)
- **Analisi Tecnica**:
  - Con `D-103`, [`detect::project`](../../../tauri/src/detect/mod.rs#L677-L687) rimuove `"apache"` e `"nginx"` da `project.requires` alla riga 677 se nessuna azione rilevata usa effettivamente Apache o Nginx.
  - Tuttavia, quella pulizia avviene in `detect::project` *dopo* la chiamata a `detect::requires(dir)`! Chiunque chiami direttamente [`detect::requires(dir)`](../../../tauri/src/detect/mod.rs#L447) — incluso [`settle_project`](../../../tauri/src/commands/mod.rs#L237) eseguito dopo ogni `frameworks_create`, `project_clone` e `github_clone` — riceve la lista non filtrata delle righe 548–562.
  - Poiché ogni progetto Laravel standard include `public/.htaccess` (pur usando `php artisan serve` come azione predefinita), `settle_project` vede `"apache"` tra i componenti richiesti e **scarica e installa automaticamente Apache** alla creazione di qualsiasi progetto Laravel!
- **Patch Proposta**: spostare il filtro delle righe 677–687 dentro la fine di `detect::requires(dir)` (o derivare `detect::requires(dir)` da `detect::project(dir).requires`).

---

### `BUG-V4-10` [ALTA] — `db::auth::env_apply` tronca `.env` a 6 righe quando parte da `.env.example` e viene sovrascritto da `env_set` nel Wizard Framework
- **File e Righe**:
  - [`tauri/src/db/auth.rs:516-598`](../../../tauri/src/db/auth.rs#L516-L598) (`env_preview` e `env_apply`)
  - [`tauri/src/commands/frameworks.rs:319-343`](../../../tauri/src/commands/frameworks.rs#L319-L343) (`frameworks_create`)
- **Analisi Tecnica**:
  1. In [`db::auth::env_apply`](../../../tauri/src/db/auth.rs#L581-L598), il codice controlla `if env_path.is_file()` (cioè se `.env` esiste già): se `.env` non esiste ancora ma esiste `.env.example`, `env_preview` legge il formato da `.env.example`, ma `env_apply` entra nel ramo `else` e chiama `render_fresh_env(&preview.entries)`, che scrive un file `.env` contenente **solo** le 6 righe `DB_*` e scarta tutte le altre variabili di `.env.example` (`APP_NAME`, `APP_ENV`, `APP_KEY`, `APP_URL`, `LOG_CHANNEL`, ecc.)!
  2. In [`commands/frameworks.rs:319-343`](../../../tauri/src/commands/frameworks.rs#L319-L343), `frameworks_create` chiama prima `db::auth::env_apply` (riga 323, che scrive la password reale del DB generata dal provisioning) e **dopo** (riga 341) chiama `projects::frameworks::env_set(&dir, &vars)` passando le variabili della ricetta del framework (che per Laravel/Symfony contengono `DB_USERNAME=root`, `DB_PASSWORD=` vuota!), sovrascrivendo le credenziali reali appena scritte da `env_apply`!
- **Patch Proposta**:
  - In `db/auth.rs:581`, se `.env` non esiste ma esiste il file template sorgente (`.env.example` / `.env.dist`), leggere il contenuto del template sorgente e applicare `rewrite_env_content(&raw, &preview.entries)` invece di `render_fresh_env`.
  - In `commands/frameworks.rs:319-343`, eseguire `projects::frameworks::env_set(&dir, &vars)` **prima** di `db::auth::env_apply`.

---

### `BUG-V4-11` [MEDIA] — Cache `WebServers` non invalidata dopo `frameworks_create`, `project_clone`, `github_clone` e `project_delete`
- **File e Righe**: [`tauri/src/commands/frameworks.rs:345`](../../../tauri/src/commands/frameworks.rs#L345), [`tauri/src/commands/projects.rs:145, 210`](../../../tauri/src/commands/projects.rs#L145), [`tauri/src/commands/github.rs:195`](../../../tauri/src/commands/github.rs#L195).
- **Analisi Tecnica**: questi comandi aggiungono o rimuovono progetti su disco ma non chiamano `state.servers.invalidate()`, per cui la pagina *Server web* continua a mostrare l'elenco precedente fino al riavvio o a un'azione manuale sui pool.

---

### `BUG-V4-12` [MEDIA] — `detect/container.rs` ignora il prefisso registry con porta (`localhost:5000/org/php:8.2`) quando estrae il tag immagine
- **File e Righe**: [`tauri/src/detect/container.rs:142-168`](../../../tauri/src/detect/container.rs#L142-L168).
- **Analisi Tecnica**: il parser dell'immagine Docker divide su `:` o prima di `/`. Con un registry privato dotato di porta (es. `registry.local:5000/library/postgres:16-alpine`), `split_once(':')` cattura `5000/library/postgres:16-alpine` invece del tag finale dopo l'ultimo `/`.
- **Patch Proposta**: isolare prima l'ultimo segmento dopo `/` (`let last = image.rsplit('/').next().unwrap_or(image);`) e cercare `:` solo su `last`.

---

### `BUG-V4-13` [MEDIA] — `terminals.rs` non convalida che le linee salvate in un profilo terminale siano ancora installate
- **File e Righe**: [`tauri/src/terminals.rs:280-335`](../../../tauri/src/terminals.rs#L280-L335).
- **Analisi Tecnica**: se un profilo terminale ha salvato `"php": "8.2"` in `terminals.json` e l'utente disinstalla PHP `8.2` lasciando solo PHP `8.4`, `terminal_env` cerca `data/apps/<platform>/php/8.2`, non la trova e lascia il terminale **senza PHP nel `PATH`** invece di fare fallback alla linea predefinita installata (`8.4`) e segnalare il fallback.

---

## 4. Rilievi Critici di Portabilità 100% e Multi-OS (`PORT-V4-01..08` — Sintesi)

I dettagli architetturali completi e le patch per i rilievi `PORT-V4-01..08` sono trattati nel documento dedicato [`04-PORTABILITA-100-E-MULTIPIATTAFORMA.md`](../../../reports/v4/04-PORTABILITA-100-E-MULTIPIATTAFORMA.md):

| ID | Severità | File Coinvolti | Descrizione Tecnica Sintetica |
|---|:---:|---|---|
| **`PORT-V4-01`** | **CRITICA** | [`tauri/src/net/archive.rs:597-599`](../../../tauri/src/net/archive.rs#L597-L599) | `copy_tree` salta tutti i symlink (`if kind.is_symlink() { continue; }`) quando `same_volume` è `false` (sempre vero su Linux `/tmp` → disco e su USB), distruggendo i binari di `python`, `node`, `cc`, `tinytex`, `mysql`. |
| **`PORT-V4-02`** | **CRITICA** | [`tauri/src/agents/relocate.rs:123-140, 389-435, 461`](../../../tauri/src/agents/relocate.rs#L461) | `relocate.rs` cerca `"old-root.txt"` anziché `"old-root-{darwin,linux}.txt"`, corrompe il case del primo carattere dei path Unix (`/home` → `/Home`) e usa backslash letterali in `targets()`. |
| **`PORT-V4-03`** | **CRITICA** | [`tauri/src/setup/kit_files.rs:21-35`](../../../tauri/src/setup/kit_files.rs#L21-L35), [`runtime/bin/agy:5`](../../../runtime/bin/agy#L5), [`runtime/bashrc:26,42,51`](../../../runtime/bashrc#L26) | `kit_files.rs` scrive un file letterale `"bin\\agy"` senza `chmod 0o755` su Unix; `runtime/bin/agy` invoca `agy.exe`; `runtime/bashrc` usa `${var,,}` che crasha su `/bin/bash` 3.2 di macOS. |
| **`PORT-V4-04`** | **CRITICA** | [`tauri/src/editor/mod.rs:258-261`](../../../tauri/src/editor/mod.rs#L258-L261), [`editor/ext.rs:140, 504`](../../../tauri/src/editor/ext.rs#L140), [`editor/cfg.rs:108, 127`](../../../tauri/src/editor/cfg.rs#L108) | VSCodium Server hardcoda `node.exe`, `r"out\server-main.js"`, `r"vscodium-server\..."` e popola solo `terminal.integrated.profiles.windows`. |
| **`PORT-V4-05`** | **ALTA** | [`tauri/src/db/services/*`](../../../tauri/src/db/services/mod.rs), [`db/adminer.rs:119`](../../../tauri/src/db/adminer.rs#L119) | `db/*` hardcoda `.exe` (`mysqld.exe`, `mysql.exe`, `mongod.exe`, `mongosh.exe`), `r"mongosh\bin"` e chiama `kit_path(root, "php/php.exe")` con `/` che non matcha `translate_rel` (`r"php\php.exe"`). |
| **`PORT-V4-06`** | **ALTA** | [`agents/mod.rs`](../../../tauri/src/agents/mod.rs#L70), [`agents/sync/mod.rs`](../../../tauri/src/agents/sync/mod.rs#L62), [`projects/apache.rs`](../../../tauri/src/projects/apache.rs#L33), [`detect/python.rs`](../../../tauri/src/detect/python.rs#L33) | Backslash letterali residui nei join di path per Claude/Codex/Antigravity, Apache, runner framework e venv Python. |
| **`PORT-V4-07`** | **MEDIA** | [`catalog/components.json:23-24`](../../../tauri/catalog/components.json#L23-L24), [`install/lang/julia.rs`](../../../tauri/src/install/lang/julia.rs#L18), [`install/lang/js.rs`](../../../tauri/src/install/lang/js.rs#L16), [`setup/runner.rs:138`](../../../tauri/src/setup/runner.rs#L138) | `Julia` e `Bun` falliscono su `win32-arm64` perché privi di `x64Only: true`; `RunLock` su Unix è solo in-process (serve `flock`); `vcruntime` segnalato mancante su Unix; pin `tauri = "=2.12.0"` vs `2.12.1`. |
| **`PORT-V4-08`** | **MEDIA** | [`install/cli.rs`](../../../tauri/src/install/cli.rs), [`install/lang/r.rs`](../../../tauri/src/install/lang/r.rs), [`catalog/components.json`](../../../tauri/catalog/components.json) | Rilocazione offline Conda per `git` e `r` su Unix (sostituendo `.reinstall`), sblocco di `php`, `apache`, `nginx`, `adminer` su macOS/Linux via `conda-forge` / `static-php-cli`. |
