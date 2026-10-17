# Dev-Kit App: piano per macOS, Linux e Windows ARM64

Stato: **proposta** (17/10/2026), da approvare. Le domande del §3 sono aperte: ogni pacchetto che ne dipende lo dice.

Base: branch `fix/v3-swarm`, commit `5c11a7a`. I riferimenti `file:riga` sono relativi a `tauri/src/` e presi dal codice di `6c26f88`: dopo i commit successivi possono essersi spostati di poche righe. Nel working tree c'è un lavoro in corso, D-107 (phpMyAdmin al posto di Adminer): questo piano parla già di phpMyAdmin.

Fonte: [MATRICE-PIATTAFORME.html](MATRICE-PIATTAFORME.html), con lo stato di ogni componente e funzione sui 6 target, gli asset esatti, le fonti upstream e i sostituti. Questo documento ne sviluppa il piano (sezione 7 della matrice) in pacchetti di lavoro eseguibili.

Obiettivo, dall'indicazione dell'utente del 17/10/2026: **rendere Dev-Kit multipiattaforma e portabile al 100%** sui 6 target del router (`kit.rs:187`, `Target::ALL`): `win32-x64`, `win32-arm64`, `darwin-arm64`, `darwin-x64`, `linux-x64`, `linux-arm64`.

«100%» vuol dire:

1. ogni componente del catalogo si installa e funziona su ogni target dove esiste una build (nativa, in emulazione o tramite un sostituto); dove non esiste, l'app lo dice con il motivo;
2. ogni funzione dell'app (terminale, agenti, editor, server dei progetti, database, accessi, rimozione dell'unità, aggiornamento) è disponibile su ogni sistema, o ha un equivalente;
3. fuori dal kit restano solo le tracce dichiarate al §1.4, scritte nel README.

## 1. Stato attuale

### 1.1 In numeri (matrice, 17/10/2026)

| | Caselle | Note |
| --- | --- | --- |
| Componenti × target | 270 (45 righe × 6), 266 escluse le «non serve» | |
| Attive o emulate | **194** (177 native, 17 in emulazione) | |
| Da sistemare | 42 | codice presente con difetti o presupposti di Windows |
| Da collegare | 22 | build upstream disponibile, manca il codice o il catalogo la esclude |
| Solo con un sostituto | 7 | Apache su macOS e Linux, MariaDB su macOS e Linux ARM64, server MariaDB su macOS |
| Impossibili | 1 | Android platform/build-tools su Linux ARM64 |
| Sottosistemi dell'app su macOS e Linux | 58 (29 × 2) | 6 pronti, 31 da sistemare, 15 da scrivere, 4 solo con un sostituto, 2 limiti del sistema |

Per target: `win32-x64` 45/45 pronti; `win32-arm64` 43/45 (17 in emulazione); `darwin-*` 27/44; `linux-x64` 27/44; `linux-arm64` 25/44.

### 1.2 Cosa funziona già

- **Installatori.** Quasi tutti scelgono l'archivio per sistema e architettura (`rust_triple`, `go_os`, `gnu_arch`, `os_tag`, `texlive_bin` in `kit.rs`); `translate_rel` (`setup/catalog.rs:175-207`) traduce i file chiave per Unix; l'estrazione di zip, tar.gz, tar.xz conserva i bit di esecuzione e i link simbolici (`net/archive.rs:154-325`).
- **Router multi-target** (commit `36d2324`): `router/target.rs` (`can_execute_between`, `effective_target`, `TargetProfile`), `router/bins.rs` (nomi dei binari per sistema), `router/probe.rs`, `router/bootstrap.rs` (preparazione da un altro sistema con completamento al primo avvio), `prepare --target`, selettore «Piattaforma» in «Tutti gli strumenti».
- **Strato del sistema operativo (D-105).** Gruppi di processi al posto dei Job Object, `flock` e socket per l'istanza unica, `FsKind::Posix`, elenco dei processi via `/proc` e libproc, `kill`, lock dei temporanei. Compila in CI su ubuntu e macOS (`cargo check`), **non è mai stato avviato**.

### 1.3 Cosa manca, per area

| Area | Lavoro | Pacchetti |
| --- | --- | --- |
| Percorsi e file system | ~25 `Path::join` con backslash, link simbolici persi fra volumi (PORT-V4-01), nessun ripiego su exFAT/FAT | WP-0.1, WP-0.2 |
| Script del kit | `bashrc` non compatibile con bash 3.2 (PORT-V4-03), `bin/agy` chiama `agy.exe`, `profile.ps1` con `;` e `\` | WP-0.3 |
| Catalogo | disponibilità per singolo target, ripieghi silenziosi (SQLite), Rosetta non verificata, `remove` solo per Windows, Node non LTS | WP-0.4 |
| Console e agenti | console esterne e selettori sono stub su Unix: gli agenti non partono fuori dall'editor | WP-1.2, WP-1.4 |
| Editor | profili del terminale solo `.windows`, «Git Bash» tradotto in `git` | WP-1.3 |
| Processi, istanza, volume | orfani dopo un crash, socket su FAT, nessun controllo di `noexec` | WP-1.5…WP-1.7 |
| Componenti | Git e Ruby non rilocabili, C/C++ senza sysroot, Rust su Windows ARM64 senza linker, MongoDB legato a una distribuzione, MariaDB su Linux | Fase 2 |
| Stack web | PHP, server web e phpMyAdmin solo Windows | Fase 3 |
| Database | MariaDB assente su macOS e Linux ARM64 | WP-3.6…WP-3.9 |
| Accessi | keyring di Antigravity solo Windows, Claude Code su macOS nel Portachiavi | Fase 4 |
| Pacchetto | radice del kit dentro `.app`/AppImage, nessun bundle, aggiornamento rotto, CI senza test su Unix | WP-0.6, Fase 5 |
| Interfaccia e documenti | nessuna griglia per target, documenti che dicono «solo Windows» | Fase 6 |

### 1.4 Limiti che il piano non supera

Vanno scritti nel README (WP-6.3) e mostrati dall'app quando servono.

| Limite | Sistema | Cosa fa l'app |
| --- | --- | --- |
| Android platform-tools e build-tools: nessuna build aarch64, nessuna emulazione x64 | linux-arm64 | «non disponibile per linux-arm64» con il motivo (WP-0.4) |
| Strumenti solo x64 | linux-arm64 | come sopra |
| Chiavetta montata `noexec` | Linux | messaggio al primo avvio con la soluzione (rimontare con `exec`) (WP-1.7) |
| FAT32 montato con `showexec` | Linux | messaggio: formattare la chiavetta in exFAT (WP-1.7) |
| Un unico file eseguibile | macOS | il kit è `Dev-Kit.app` più `projects/` e `data/` (WP-5.2) |
| Gatekeeper e App Translocation senza firma Apple | macOS | dipende da Q-2; rilevamento della traslocazione con istruzioni (WP-5.2) |
| Dati della WebView in `~/Library/WebKit` | macOS | traccia dichiarata (WP-1.8) |
| SDK Apple solo con Xcode Command Line Tools (amministratore) | macOS | C/C++ e Rust con framework: rilevamento e messaggio (WP-2.3) |
| Rosetta 2 si installa da amministratore | darwin-arm64 | componenti x64 solo se Rosetta c'è (WP-0.4) |
| Secret Service assente su desktop minimali | Linux | l'accesso di Antigravity resta solo per la sessione, con avviso (WP-4.1) |
| Emulazione su Windows ARM64: editor, CodeLLDB, PHP, Apache, MariaDB, MongoDB, Julia, TinyTeX, Android tools | win32-arm64 | invariato; CodeLLDB debugga solo processi x64 |

## 2. Principi

- **Un solo kit, sei target.** `projects/` e `data/` sono condivisi; i programmi stanno in `data/apps/<target>/`; le impostazioni per sistema in `data/app/versions/<target>/`. Lo stesso kit su chiavetta parte su Windows, macOS e Linux.
- **Rilocabile prima che completo.** Fra due build si sceglie quella che segue il kit quando cambia cartella. Gli ambienti conda (micromamba) solo dove non c'è altro, segnati come non rilocabili (`Installer::relocatable(target)` falso, reinstallazione con `.reinstall`, D-92).
- **Ogni download verificato.** Dove il fornitore non pubblica un hash (MySQL: solo MD5 + GPG; static-php-cli, FrankenPHP, Tectonic: niente) l'hash si fissa nel catalogo per target, come `sha256ByTarget` dell'editor. Mai un'installazione senza verifica.
- **Niente ripieghi silenziosi.** Un componente che non esiste per un target è «non disponibile per `<target>`» con il motivo, mai un binario di un'altra architettura che non parte.
- **Additivo.** I motori nuovi (Caddy, MySQL) si affiancano ai vecchi; nessuna migrazione forzata dei dati dell'utente.
- **Windows invariato.** Nessun pacchetto cambia il comportamento su Windows, salvo dove lo dichiara. I test Windows di oggi restano verdi a ogni merge.
- **Prove reali.** Un pacchetto che tocca un sistema è «fatto» solo dopo un avvio vero su quel sistema (§6), non solo con `cargo check`.
- **Windows resta il sistema principale.** È il sistema degli utenti di oggi e quello su cui si sviluppa (§2.1). I rilasci per Windows non aspettano macOS e Linux: ogni ondata si fonde solo se Windows resta pronto per un rilascio.

### 2.1 Sviluppo da Windows

Il codice si scrive e si prova su un PC Windows 11 x64 (questo, con WSL2 e Ubuntu installati). Ne derivano regole precise.

| Target | Compilare | Test automatici | Avvio vero (§6) |
| --- | --- | --- | --- |
| `win32-x64` | in locale | in locale e in CI (`windows-latest`) | in locale |
| `win32-arm64` | in locale (`--target aarch64-pc-windows-msvc`, come la CI) | CI con un runner `windows-11-arm` (nuovo, WP-0.6) | PC Windows 11 ARM64 (utente) |
| `linux-x64` | **WSL2** (`wsl -d Ubuntu`) | WSL2 in locale e CI (`ubuntu-latest`) | WSL2 con WSLg per la finestra; poi un PC o una VM Linux con desktop |
| `linux-arm64` | CI | CI (`ubuntu-24.04-arm`) | PC o scheda ARM64 (utente), o VM ARM64 in cloud |
| `darwin-arm64`, `darwin-x64` | solo CI (Tauri non compila per macOS da Windows) | CI (`macos-latest`, `macos-13`) | Mac dell'utente, o Mac in cloud; senza Mac WP-1.9 resta aperto |

Regole:

- **Logica di piattaforma come funzioni pure con il target come parametro.** Esempi: `translate_rel`, `root_from(exe, appimage)`, `venv_python(target)`, i generatori di Caddyfile e `my.cnf`, la scelta degli asset. Così i test girano su Windows per tutti e 6 i target, e i `cfg(unix)` restano sottili (chiamate al sistema). È già il modello di `router/target.rs` (`can_execute_between`) e dei test del router.
- **`cargo check` non basta da Windows.** `cargo check --target x86_64-unknown-linux-gnu` fallisce sui crate di sistema (webkit2gtk via pkg-config), e per macOS manca l'SDK. Per Linux si usa WSL2, per macOS solo la CI.
- **WSL2 per Linux in locale.** Script `scripts/test-linux.sh` (WP-0.6), eseguito come `wsl -d Ubuntu -- scripts/test-linux.sh`. Copia il repo nel file system di Linux (`~/dev-kit2`, con `rsync`), perché su `/mnt/d` i permessi e i link simbolici non sono quelli di Linux e la compilazione è lenta. Poi `cargo test` e, per le prove a mano, `cargo run` con la finestra via WSLg.
- **Il file system di Windows nasconde alcuni difetti.** Non distingue le maiuscole, non ha bit di esecuzione, e i link simbolici richiedono la modalità sviluppatore. I test di rilocazione con maiuscole diverse (WP-0.5), dei link simbolici (WP-0.2) e dei permessi 600/755 (`router/fs.rs`) girano in WSL2 e in CI, e su Windows si saltano con il motivo (`cfg_attr(windows, ignore = "…")`).
- **Fine riga e permessi dei file del kit.** `.gitattributes` forza già `eol=lf` (`*.ps1` e `*.cmd` in CRLF). I file sh di `runtime/` sono `100644` nel repo, e va bene così: l'app li scrive con `chmod 755` (`setup/kit_files.rs:44-56`). Per ogni nuovo script sh: scritto da `kit_files.rs` con 755 e `LF`, mai eseguito direttamente dal repo.
- **Preparare da Windows un kit per un altro sistema** è il primo caso reale degli utenti di oggi: chiavetta preparata su Windows, poi usata su un Mac o su Linux. `prepare --target <id>` e `router/bootstrap.rs` esistono già. Ogni pacchetto di componenti lo prova (§6.5): preparazione da Windows su exFAT, poi primo avvio sull'altro sistema, con il completamento dei `.reinstall`.
- **Gli agenti dello sciame lavorano su Windows.** Ognuno esegue in locale i test Windows e, se tocca codice Unix, `scripts/test-linux.sh` in WSL2. Il risultato su macOS lo dà la CI della pull request d'integrazione, e il coordinatore non fonde un'ondata con la CI rossa.

## 3. Domande per l'utente

Ogni domanda ha una proposta. I pacchetti che dipendono da una domanda aperta partono con la proposta solo dopo il sì dell'utente.

| Id | Domanda | Proposta | Scartate | Pacchetti |
| --- | --- | --- | --- | --- |
| Q-1 | Linux ARM64 è un target di primo livello (CI, rilasci)? | Sì, con i buchi dichiarati al §1.4 | «best effort» senza CI | WP-0.6, WP-5.4 |
| Q-2 | Firma Apple Developer ID (99 $ l'anno) e notarizzazione? | Sì per i rilasci; fino ad allora build non firmata con istruzioni (`xattr -dr com.apple.quarantine`) | solo non firmata | WP-5.2 |
| Q-3 | Server web predefinito? | **Caddy** su tutti i target; Apache resta su Windows per i progetti con `.htaccess`; Nginx tolto dopo la migrazione | Nginx su Unix (nessuna build ufficiale); FrankenPHP predefinito | WP-3.1…WP-3.5 |
| Q-4 | Database MySQL-compatibile su macOS e Linux ARM64? | **MySQL** come secondo motore, predefinito per i kit nuovi, unico dove MariaDB non c'è; MariaDB resta per chi ha dati; «Migra a MySQL» via dump | MySQL ovunque con migrazione forzata; nessun database lì | WP-3.6…WP-3.9 |
| Q-5 | Fonte dei binari su Unix? | Build rilocabili dove ci sono (dugite-native, static-php-cli, zig, Caddy, MySQL); conda-forge per Ruby, R, MongoDB su Linux, SQLite su Linux ARM64 | tutto da conda-forge; Homebrew (percorsi fissi) | Fase 2, Fase 3 |
| Q-6 | Terminale e agenti su macOS e Linux? | Terminale nell'app (PTY + xterm.js) su Unix; su Windows resta la console classica (D-57, D-95) | Terminal.app / `x-terminal-emulator` come unico modo (dipende dal desktop, fuori dal controllo del kit) | WP-1.2 |
| Q-7 | Formato su Linux? | AppImage, più un tar.gz con l'eseguibile per chi non ha FUSE | solo tar.gz; Flatpak (non portabile) | WP-5.3 |
| Q-8 | PostgreSQL come terzo database? | Dopo questo piano (§10), sopra `DbEngine` | ora | — |

## 4. Disegno

### 4.1 Catalogo: disponibilità per target

Oggi `Component` (`setup/catalog.rs:43-90`) ha `targets`, `x64Only` (solo `win32-arm64`, `:257-259`) e `noArm64` (solo `win32-arm64`, `:249`). Campi nuovi, tutti facoltativi:

```jsonc
{
  "id": "android",
  // target → motivo: il componente non c'è su quel target, con un testo per l'interfaccia
  "unavailable": { "linux-arm64": "Google non pubblica platform-tools e build-tools per Linux ARM64" },
  // x64 in emulazione anche su darwin-arm64 (Rosetta 2), non solo su win32-arm64
  "x64Only": true,
  // file posseduti solo su Unix (lib/node_modules…), accanto a "remove" che resta per Windows
  "removeUnix": ["@data/npm-global/lib/node_modules/pnpm"],
  // hash fissati per target dove il fornitore non pubblica SHA-256
  "sha256ByTarget": { "linux-x64": "…", "darwin-arm64": "…" }
}
```

- `is_available_on(target)` (`catalog.rs:245`): falso se il target è in `unavailable`, o fuori da `targets`, o `noArm64` su `win32-arm64`. `unavailable_reason(target)` restituisce il testo (tradotto da `i18n.rs`) al posto di «non disponibile per `<os>-<arch>`» (`commands/setup.rs:123`).
- `is_emulated_on(target)`: `x64Only` vale su `win32-arm64` e su `darwin-arm64`. Su `darwin-arm64` serve Rosetta: `router::target::rosetta_present()` (`/usr/bin/arch -x86_64 /usr/bin/true` esce con 0); senza Rosetta il componente è non disponibile con il motivo «richiede Rosetta 2».
- `can_execute_between` (`router/target.rs:29-40`) riceve la stessa verifica di Rosetta.
- `sha256ByTarget` (oggi solo editor, `catalog.rs:263-271`) diventa generico: chi lo usa verifica l'hash fissato e rifiuta un asset con un altro hash (la versione è fissata insieme, campo `version`).

### 4.2 Fonti per Unix e Windows ARM64

| Componente | Target | Fonte nuova | Verifica |
| --- | --- | --- | --- |
| Git | darwin-*, linux-* | `desktop/dugite-native` (GitHub), tar.gz | `.sha256` dell'asset |
| Ruby | darwin-*, linux-* | conda-forge `ruby` via micromamba | micromamba |
| C, C++ | darwin-*, linux-* | `zig` del kit con shim `cc`/`c++`, più `make` | indice di Zig (SHA-256) |
| Rust | win32-arm64 | host `aarch64-pc-windows-gnullvm` con llvm-mingw `ucrt-aarch64` come linker | invariata |
| MongoDB, Database Tools | linux-* | conda-forge `mongodb`; in alternativa il tarball della distribuzione da `/etc/os-release` | micromamba o JSON |
| SQLite | linux-arm64 | conda-forge `sqlite` | micromamba |
| micromamba | win32-arm64 | `micromamba-win-arm64` | `.sha256` |
| PHP | darwin-*, linux-* | static-php-cli `php-<v>-{cli,fpm}-{macos,linux}-{x86_64,aarch64}.tar.gz` (dl.static-php.dev) | SHA-256 fissato nel catalogo |
| Caddy | tutti | `caddy_<v>_{windows,mac,linux}_{amd64,arm64}.{zip,tar.gz}` | `checksums.txt` (firma Sigstore facoltativa) |
| MySQL | tutti (win32-arm64 in emulazione) | `mysql-<v>-winx64.zip`, `mysql-<v>-macos15-{arm64,x86_64}.tar.gz`, `mysql-<v>-linux-glibc2.28-{x86_64-minimal,aarch64}.tar.xz` | SHA-256 fissato nel catalogo (Oracle: MD5 + GPG) |

### 4.3 `DbEngine` (WP-3.6)

Oggi il motore è una stringa e ogni ramo fa `match id { "mongodb" => …, _ => /* MariaDB */ }`; `SERVERS: [&str; 2]` (`db/services/mod.rs:61`). Un terzo motore cadrebbe nel ramo di MariaDB senza errori.

```rust
/// A database server the kit runs.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DbEngine { MariaDb, MySql, MongoDb }

impl DbEngine {
    pub const ALL: [DbEngine; 3];
    pub fn id(self) -> &'static str;               // "mariadb" | "mysql" | "mongodb"
    pub fn component(self) -> &'static str;        // catalog id
    pub fn default_port(self) -> u16;              // 3306 | 3307 | 27017
    pub fn server_bin(self) -> &'static str;       // mariadbd | mysqld | mongod
    pub fn client_bin(self) -> &'static str;       // mariadb | mysql | mongosh
    pub fn dump_bin(self) -> &'static str;         // mariadb-dump | mysqldump | mongodump
    pub fn admin_bin(self) -> Option<&'static str>; // mariadb-admin | mysqladmin | None
    pub fn is_sql(self) -> bool;
    pub fn reserved_names(self) -> &'static [&'static str];
}
```

- `"mariadb"` e `"mongodb"` restano gli id salvati (`db.json`, `secrets.json` `mariadb_root`, `servers.json`): nessuna migrazione per chi non cambia motore.
- MySQL ha la sua porta predefinita (3307) per poter girare accanto a MariaDB.
- **Inizializzazione MySQL**: `mysqld --no-defaults --initialize-insecure --basedir=<line> --datadir=<data> --lower_case_table_names=1`, poi `my.cnf` scritto dal kit (porta, `bind-address=127.0.0.1`, `socket` e `pid-file` dentro `data/db/mysql/<line>/`, `mysqlx=OFF`, `lower_case_table_names=1`). `lower_case_table_names=1` su tutti i sistemi: MySQL 8 non lo lascia cambiare dopo l'inizializzazione, e con lo stesso valore la cartella dati segue il kit da un sistema all'altro.
- **Accesso dal terminale (D-74).** MySQL non legge `%MYSQL_HOME%\my.ini` come MariaDB (da verificare in T3.7.1). Il kit passa `--defaults-extra-file=<data/app/db/mysql.cnf>` tramite i lanciatori `mysql`/`mysqldump` di `data/kit/bin` e non si appoggia a `MYSQL_HOME` per MySQL.
- **Utenti.** `CREATE USER … IDENTIFIED WITH caching_sha2_password BY …` (predefinito di 8.4 e 9.x; `mysql_native_password` non esiste più in 9.x). Gli utenti del progetto si connettono via TCP su 127.0.0.1: si crea `'<u>'@'localhost'` e `'<u>'@'127.0.0.1'`.

### 4.4 `WebServerEngine::Caddy` (WP-3.3)

`WebServerEngine { Apache, Nginx }` (`servers/mod.rs:31-51`) riceve `Caddy`. Un Caddyfile per pool e uno per sito del progetto, generati come oggi per Nginx (`servers/config.rs:270-330`):

```caddyfile
{
	admin 127.0.0.1:{admin_port}   # una porta per istanza: due pool non si scontrano su :2019
	auto_https off
	persist_config off
	storage file_system {kit}/data/app/caddy/{pool}
}
http://127.0.0.1:{port}, http://localhost:{port} {
	root * {project_root}
	php_fastcgi 127.0.0.1:{fcgi_port} {
		env DEVKIT_FCGI_KEY {key}
	}
	file_server
	@hidden path /.git/* /.env /private/*
	respond @hidden 404
}
```

- PHP: `php-cgi -b 127.0.0.1:<p>` su Windows (come Nginx, `projects/nginx.rs:317-363`, con `runtime/fcgi-guard.php`), `php-fpm` su Unix con un pool `fpm.conf` generato (`listen = 127.0.0.1:<p>`, `pm = ondemand`).
- Avvio `caddy run --config <file> --adapter caddyfile`, ricarica `caddy reload --config <file> --address 127.0.0.1:<admin_port>`, arresto tramite il gruppo di processi.
- `.htaccess` non letto: per WordPress, Drupal e Laravel il kit genera la regola `try_files {path} {path}/ /index.php?{query}`, come oggi `vhost_block` di Nginx (`projects/nginx.rs:186-203`). `detect/php.rs:47-112` propone Apache solo su Windows quando il progetto ha un `.htaccess` con regole proprie.

### 4.5 Terminale nell'app su Unix (WP-1.2, Q-6)

- Backend: crate `portable-pty` (openpty), un PTY per sessione con `pwsh` del kit o `bash --rcfile data/kit/bashrc`, ambiente del kit (`env.rs`), cartella del progetto; il processo entra nel gruppo di `os/jobs.rs`.
- Comandi Tauri: `pty_open(project, shell, agent?) -> id`, `pty_write(id, bytes)`, `pty_resize(id, cols, rows)`, `pty_close(id)`; uscita come evento `pty://<id>`.
- Interfaccia: xterm.js in una finestra o scheda del kit; gli agenti (`agents/mod.rs:186`) aprono una sessione con il loro comando.
- Su Windows niente cambia: `spawn_console` (conhost) resta.

### 4.6 Pacchetto e radice del kit (WP-5.1…WP-5.3)

```text
macOS                         Linux
Dev-Kit/                      Dev-Kit/
  Dev-Kit.app/                  Dev-Kit.AppImage   (oppure Dev-Kit, dal tar.gz)
  projects/                     projects/
  data/                         data/
```

- `kit.rs:376-383` oggi usa la cartella dell'eseguibile. Nuova regola: su macOS, se l'eseguibile sta in `*.app/Contents/MacOS/`, la radice è la cartella che contiene `*.app`; su Linux, se c'è `$APPIMAGE`, la radice è `dirname($APPIMAGE)`; altrimenti la cartella dell'eseguibile.
- macOS: niente scritto dentro il bundle (romperebbe la firma). App Translocation: se il percorso dell'app è sotto `/private/var/folders/…/AppTranslocation/`, messaggio con le istruzioni (con firma: spostare con il Finder; senza: `xattr -dr com.apple.quarantine Dev-Kit.app`).

### 4.7 Accessi degli agenti (Fase 4)

- Antigravity: crate `keyring` con `apple-native` (Portachiavi) e `sync-secret-service` (Linux). Stesso modello di Windows (`agents/keyring.rs`): prima di `agy` si scrive l'accesso salvato nel kit, dopo si rilegge e si cancella dall'archivio del sistema. La copia nel kit oggi cifrata con DPAPI: su Unix con una chiave nel Portachiavi o in Secret Service; senza Secret Service resta nel file privato (permessi 600) con un avviso.
- Claude Code su macOS: `claude setup-token` dalla pagina Agenti, token salvato in `data/agents/claude/oauth-token` (600), passato come `CLAUDE_CODE_OAUTH_TOKEN`. «Esci» cancella il file.

## 5. Pacchetti di lavoro

Ogni pacchetto (WP) ha: obiettivo, dipendenze, file, compiti numerati, criteri di accettazione, test, dimensione (S fino a un giorno, M qualche giorno, L una settimana o più). Commenti del codice in inglese come il resto; testi dell'interfaccia in italiano, con le traduzioni in `ui/src/locales` e `tauri/i18n/catalog.json`.

---

### Fase 0: fondamenta comuni

Sblocca macOS e Linux insieme. Nessun cambiamento visibile su Windows.

#### WP-0.1 Percorsi portabili · M

- **Obiettivo.** Nessun `Path::join` con un separatore scritto dentro la stringa.
- **Dipende da.** Nulla.
- **File.** `editor/cfg.rs`, `editor/ext.rs`, `agents/sync/mod.rs`, `agents/sync/mcp.rs`, `agents/sync/project.rs`, `agents/relocate.rs`, `projects/mod.rs`, `projects/apache.rs`, `projects/migrate.rs`, `servers/config.rs`, `github/mod.rs`, `github/repos.rs`, `detect/python.rs`, `versions/relations/checks.rs`, `versions/relations/mod.rs`; test nuovo `test/rust/paths.rs`.
- **Compiti.**
  - T0.1.1 Sostituire con `fsx::join_rel` (o `.join()` concatenati) le chiamate note: `editor/cfg.rs:108` (settings.json della macchina), `:165` (`bin\agy.cmd`); `editor/ext.rs:145` (cartella delle estensioni); `agents/sync/mod.rs:62` (stato `.sync`), `:174-175`, `:386`; `agents/sync/mcp.rs:214`; `agents/sync/project.rs:40-41`; `agents/relocate.rs:403-429`; `projects/mod.rs:237` (`private\backups`); `projects/apache.rs:33,50,378`; `projects/migrate.rs:49`; `servers/config.rs:272`; `github/mod.rs:110` (`gh\hosts.yml`); `github/repos.rs:118` (`.git\config`); `versions/relations/checks.rs:101` e `relations/mod.rs:252` (vcruntime: solo Windows, si chiudono dentro `cfg(windows)` o con un controllo del target).
  - T0.1.2 `detect/python.rs:33-34,175`: venv in `.venv/Scripts/python.exe` su Windows e `.venv/bin/python` su Unix, con una funzione `venv_python(target)`.
  - T0.1.3 Test che legge tutti i file `.rs` di `tauri/src` e fallisce su `join(r"` o `join("` seguiti da un testo con `\`. Le eccezioni (percorsi Windows legittimi dentro `cfg(windows)`) in una lista nel test, con il motivo.
- **Accettazione.** Il test di T0.1.3 è verde; i percorsi generati su Windows sono identici a prima (test esistenti verdi).
- **Test.** `venv_python` per Windows e Unix; i test di `editor`, `agents/sync`, `github` con una radice Unix simulata dove il codice lo permette.

#### WP-0.2 Link simbolici e file system senza link · M

- **Obiettivo.** Le installazioni Unix conservano i link simbolici; su exFAT/FAT si ripiega su una copia.
- **Dipende da.** Nulla.
- **File.** `net/archive.rs`, `versions/side_by_side.rs`.
- **Compiti.**
  - T0.2.1 `copy_tree` (`net/archive.rs:597`, `:622`): un link simbolico si ricrea come link (`std::os::unix::fs::symlink`) con la stessa destinazione relativa, se resta dentro l'albero copiato (stesso controllo dell'estrazione, `:262-325`); un link che esce dall'albero si salta con un avviso.
  - T0.2.2 Estrazione (`:187-190`): se `symlink()` fallisce con `EPERM`, `EOPNOTSUPP` o `ENOTSUP` (exFAT, FAT), si copia il file o la cartella di destinazione al posto del link, dopo aver estratto tutto il resto (secondo passaggio).
  - T0.2.3 `versions/side_by_side.rs:131-155`: stesso ripiego a copia per i collegamenti delle versioni (Maven, Gradle, .NET).
- **Accettazione.** Node e Python su Unix installati da un temp su un altro volume hanno `bin/npm`, `bin/python3` come link funzionanti; su exFAT funzionano come copie.
- **Test.** Archivio tar con link interno, link verso `../fuori`, link a una cartella; copia fra due cartelle con link; ripiego simulato con un errore iniettato.

#### WP-0.3 Script del kit portabili · S

- **Obiettivo.** `bashrc`, `profile.ps1` e i lanciatori di `runtime/bin` funzionano su macOS (bash 3.2, zsh) e Linux.
- **Dipende da.** Nulla.
- **File.** `runtime/bashrc`, `runtime/profile.ps1`, `runtime/bin/agy`, `setup/kit_files.rs`.
- **Compiti.**
  - T0.3.1 `runtime/bashrc:26,42,51`: `${var,,}` sostituito da `$(printf '%s' "$var" | tr '[:upper:]' '[:lower:]')`; nessun'altra sintassi di bash 4 (`declare -A`, `mapfile`, `;&`).
  - T0.3.2 `runtime/bin/agy:5`: `"$CP_ANTIGRAVITY/agy"` su Unix, `agy.exe` solo se esiste (Git Bash su Windows).
  - T0.3.3 `runtime/profile.ps1:10,27-45`: separatore da `[IO.Path]::PathSeparator`, percorsi con `Join-Path`, niente `\` scritto.
  - T0.3.4 `setup/kit_files.rs`: i file sh con `LF` e permessi 755 (già per `bin/agy` e `bin/gradle`, `:44-56`); controllare che `bashrc` sia scritto con `LF`.
- **Accettazione.** `bash --rcfile data/kit/bashrc -i -c 'echo ok'` con bash 3.2 e 5; `pwsh -NoProfile -File data/kit/profile.ps1` su Linux senza errori.
- **Test.** Test Rust che esegue `bash -n` sul bashrc generato (dove c'è bash); `shellcheck` in CI facoltativo.

#### WP-0.4 Disponibilità per target nel catalogo · M

- **Obiettivo.** Il catalogo dice esattamente cosa c'è su ogni target (§4.1); nessun binario dell'architettura sbagliata.
- **Dipende da.** Nulla.
- **File.** `setup/catalog.rs`, `tauri/catalog/components.json`, `router/target.rs`, `commands/setup.rs`, `install/db.rs`, `install/core.rs`, `kit.rs`, `tauri/i18n/catalog.json`.
- **Compiti.**
  - T0.4.1 Campi `unavailable`, `removeUnix`, `sha256ByTarget` generico in `Component`, con `unavailable_reason(target)`.
  - T0.4.2 `x64Only` su `darwin-arm64` con `rosetta_present()`; `can_execute_between` aggiornato.
  - T0.4.3 Android: `"unavailable": { "linux-arm64": … }`.
  - T0.4.4 SQLite: togliere il ripiego su x64 (`install/db.rs:490-491`); `linux-arm64` passa a conda-forge in WP-2.5, fino ad allora `unavailable`.
  - T0.4.5 mongosh: usare `target_of()` (`x64Only`) invece della build x64 scritta fissa (`install/db.rs:320`).
  - T0.4.6 Node: scegliere la prima voce con `lts` diverso da `false` in `index.json` (`install/core.rs:466`), come dice il commento; facoltativo «Current» dalle impostazioni.
  - T0.4.7 micromamba: `micromamba-win-arm64` su `win32-arm64` (`kit.rs:328-336`).
  - T0.4.8 `removeUnix` per pnpm/yarn (`lib/node_modules/pnpm`, `lib/node_modules/corepack`), uv, cmake, r.
  - T0.4.9 Testo `about` di `components.json` e commenti: `data/apps/<target>` al posto di `data\apps\win32-<arch>`.
- **Accettazione.** `Dev-Kit.exe components --target linux-arm64` mostra Android e SQLite non disponibili con il motivo; nessun asset x64 scelto per `linux-arm64`.
- **Test.** `is_available_on`/`unavailable_reason` per ogni campo; Rosetta simulata presente o assente; scelta LTS di Node su un `index.json` di esempio; asset di mongosh per `win32-arm64`.

#### WP-0.5 Lock e rilocazione su Unix · S

- **Obiettivo.** Una preparazione alla volta anche fra processi; rilocazione corretta su file system che distinguono le maiuscole.
- **Dipende da.** Nulla.
- **File.** `setup/runner.rs`, `agents/relocate.rs`.
- **Compiti.**
  - T0.5.1 `setup/runner.rs:140-185`: su Unix `flock` esclusivo su `data/app/prepare.lock` oltre al set in memoria; secondo processo: «Un'altra preparazione è in corso».
  - T0.5.2 `agents/relocate.rs:29-95`: regex `(?i)` solo su Windows; su Unix confronto esatto.
  - T0.5.3 `Replacer::substitute`: verificare il caso `/home`→`/Home` lasciato aperto da PORT-V4-02 e correggerlo con un test.
- **Accettazione.** Due `prepare` contemporanei da riga di comando su Linux: il secondo si ferma con il messaggio.
- **Test.** Lock fra due processi (test con `std::process::Command` sul binario dei test); rilocazione di `/media/u/Kit` → `/media/u/kit` trattata come spostamento.

#### WP-0.6 CI che esegue i test su Unix · M

Anticipato dalla fase 5 della matrice: i pacchetti successivi hanno bisogno dei test su Unix.

- **Obiettivo.** `cargo test` su ubuntu-x64, ubuntu-arm64, macOS arm64 e macOS x64; oggi solo `cargo check` (`.github/workflows/app.yml:54-86`).
- **Dipende da.** Q-1 per il job linux-arm64.
- **File.** `.github/workflows/app.yml`, `scripts/test-linux.sh` (nuovo), `docs/app/TESTING.md`, eventuali `#[cfg(windows)]` sui test che usano solo Windows.
- **Compiti.**
  - T0.6.1 Job `test-unix` con matrice `ubuntu-latest`, `ubuntu-24.04-arm`, `macos-latest` (arm64), `macos-13` (x64): `cargo test` del crate; dipendenze di Tauri per Linux (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`).
  - T0.6.2 Il problema della feature di test di tauri 2.12 su macOS (`app.yml:82-86`): test senza `--all-targets` o feature separata; documentare la scelta.
  - T0.6.3 Segnare con `#[cfg(windows)]` o `#[cfg_attr(not(windows), ignore)]` i test che usano API di Windows, con il motivo.
  - T0.6.4 Vitest e build dell'interfaccia anche su ubuntu (`npm test`, `npm -w ui run build`).
  - T0.6.5 Job `windows-11-arm`: `cargo test` nativo su Windows ARM64 (oggi la build ARM64 è solo compilata in modo incrociato, senza test).
  - T0.6.6 `scripts/test-linux.sh` per WSL2 (§2.1): `rsync` del repo in `~/dev-kit2` (esclusi `target/` e `node_modules/`), dipendenze di Tauri controllate (`dpkg -s`, con la riga `apt` da eseguire se mancano), `cargo test`. Uso documentato in TESTING.md.
  - T0.6.7 CI che si avvia anche sulle pull request verso `feature/platforms`, così il coordinatore vede macOS e Linux prima di ogni merge.
- **Accettazione.** Tutti i job verdi; il numero di test eseguiti per job nel riepilogo; `scripts/test-linux.sh` verde in WSL2 su questo PC.
- **Test.** —

---

### Fase 1: l'app gira su macOS e Linux

Fine fase: primo avvio reale (WP-1.9) su macOS arm64 e Linux x64, con editor, terminale e un agente.

#### WP-1.1 Radice del kit fuori dal bundle · S

- **Obiettivo.** `projects/` e `data/` accanto a `Dev-Kit.app` o all'AppImage (§4.6).
- **Dipende da.** Nulla.
- **File.** `kit.rs`.
- **Compiti.**
  - T1.1.1 `kit.rs:376-383`: radice dalla cartella che contiene `*.app` (se l'eseguibile è in `*.app/Contents/MacOS/`) o da `dirname($APPIMAGE)`, altrimenti cartella dell'eseguibile.
  - T1.1.2 Rilevamento di App Translocation (`/AppTranslocation/` nel percorso): errore all'avvio con le istruzioni (§4.6) invece di creare un kit in una cartella temporanea.
  - T1.1.3 Radice in sola lettura: messaggio chiaro invece del fallimento della creazione di `data/`.
- **Accettazione.** `Dev-Kit.app` avviato da `~/Kit/` crea `~/Kit/data`, niente dentro il bundle.
- **Test.** Funzione pura `root_from(exe, appimage_env)` con i casi `.app`, AppImage, binario semplice, percorso traslocato.

#### WP-1.2 Terminale e agenti su Unix · L

- **Obiettivo.** Terminale del kit e avvio degli agenti su macOS e Linux (§4.5). Dipende da Q-6.
- **Dipende da.** WP-0.3, Q-6.
- **File.** `editor/terminal.rs` (oggi stub `:124-134`), `editor/pty.rs` (nuovo), `commands/terminals.rs`, `agents/mod.rs`, `os/jobs.rs` (solo l'aggancio al gruppo), `Cargo.toml` (`portable-pty`, `cfg(unix)`), `ui/src/terminal/*` (nuovo, xterm.js), `ui/package.json`.
- **Compiti.**
  - T1.2.1 `editor/pty.rs`: sessioni PTY (apertura, lettura in un thread, scrittura, ridimensionamento, chiusura), processo nel gruppo dei figli del kit.
  - T1.2.2 Comandi `pty_open`, `pty_write`, `pty_resize`, `pty_close` ed evento `pty://<id>`; solo `cfg(unix)`, su Windows restano le console di oggi.
  - T1.2.3 Shell: `pwsh` del kit se installato, altrimenti `/bin/bash --rcfile <kit>/data/kit/bashrc -i` (macOS: bash 3.2 di sistema, compatibile dopo WP-0.3).
  - T1.2.4 `agents/mod.rs:186`: su Unix l'agente apre una sessione PTY con il suo comando invece di `spawn_console`.
  - T1.2.5 Interfaccia: componente terminale con xterm.js (tema del kit), schede per sessione, chiusura con conferma se il processo è vivo.
  - T1.2.6 «Rimuovi l'unità» chiude anche le sessioni PTY.
- **Accettazione.** Su macOS e Linux: «Terminale» di un progetto apre una shell nella cartella del progetto con l'ambiente del kit (`echo $CP_DATA`); «Claude Code» parte e risponde; chiudendo l'app i processi si fermano.
- **Test.** PTY che esegue `echo` e restituisce l'output; ridimensionamento; chiusura che termina il gruppo; Vitest del componente con un backend finto.

#### WP-1.3 Editor su Unix · S

- **Obiettivo.** Profili del terminale, impostazioni ed estensioni dell'editor su macOS e Linux.
- **Dipende da.** WP-0.1.
- **File.** `editor/cfg.rs`, `setup/catalog.rs` (`translate_rel`).
- **Compiti.**
  - T1.3.1 `editor/cfg.rs:186-212`: scrivere anche `terminal.integrated.profiles.osx`/`.linux` e `defaultProfile.osx`/`.linux`: «PowerShell» (pwsh del kit, se c'è), «Bash» (`/bin/bash --rcfile …/bashrc`), «zsh» su macOS; niente `cmd.exe` né `agy.cmd` fuori da Windows (`:132-170`).
  - T1.3.2 `setup/catalog.rs:181`: `git/bin/bash.exe` non diventa più `git/bin/git`; la chiave di Git su Unix resta `git/bin/git`, ma il profilo Bash usa la shell di sistema.
  - T1.3.3 Profilo «Antigravity» con `bin/agy` (sh) su Unix.
- **Accettazione.** Il terminale integrato dell'editor su Linux apre bash con l'ambiente del kit; le estensioni installate compaiono nell'editor.
- **Test.** JSON generato per i tre sistemi confrontato con un atteso.

#### WP-1.4 Selettori di file e cartelle · S

- **File.** `editor/terminals.rs:438-445`, `setup/downloads.rs:127-134`, `commands/setup.rs:315`, `commands/terminals.rs:83`, `Cargo.toml` (`tauri-plugin-dialog`), `tauri/capabilities/*`.
- **Compiti.** T1.4.1 `tauri-plugin-dialog` su Unix (NSOpenPanel, GTK, portale XDG) al posto degli stub; su Windows invariato (IFileOpenDialog). T1.4.2 Permessi del plugin nelle capability.
- **Accettazione.** «Scegli cartella» e «Scegli file» funzionano su macOS e Linux.
- **Test.** Manuale (§6).

#### WP-1.5 Processi figli dopo un crash · M

- **Obiettivo.** Nessun processo orfano se l'app viene uccisa (equivalente di `KILL_ON_JOB_CLOSE`).
- **File.** `os/jobs.rs`, `os/sessions.rs`.
- **Compiti.**
  - T1.5.1 Linux: in `pre_exec`, `prctl(PR_SET_PDEATHSIG, SIGKILL)` e controllo che il padre sia ancora vivo (`getppid()`), sui figli diretti; i nipoti restano nel gruppo e li chiude il watchdog di T1.5.2.
  - T1.5.2 Watchdog: processo di guardia minimo (lo stesso `Dev-Kit` con `__guard <pgid>`) che legge una pipe dall'app e alla sua chiusura fa `killpg(pgid, SIGTERM)`, poi `SIGKILL` dopo 5 s. Su macOS è l'unico meccanismo (niente `PR_SET_PDEATHSIG`); in alternativa kqueue `EVFILT_PROC`/`NOTE_EXIT` sul pid dell'app.
  - T1.5.3 `os/sessions.rs:162`: `SIGTERM`, attesa fino a 5 s, poi `SIGKILL` (i database scrivono su disco al `SIGTERM`).
- **Accettazione.** `kill -9 <app>` con MariaDB e un terminale aperti: entro 6 s nessun processo del kit resta (`pgrep -f <kit>`).
- **Test.** Test di integrazione Unix: avvio di un figlio `sleep`, uccisione del padre simulato, verifica che il figlio sparisca.

#### WP-1.6 Istanza unica robusta · S

- **File.** `os/instance.rs`.
- **Compiti.** T1.6.1 Linux: socket astratto `\0Dev-Kit-<id>`. T1.6.2 macOS: socket in `$TMPDIR/devkit-<id>.sock` (percorso corto, dentro i 104 byte). T1.6.3 Ripiego se il socket non si crea: porta TCP casuale su 127.0.0.1 con un token in `data/app/instance.json` (600). `<id>` = hash della radice del kit.
- **Accettazione.** Kit su exFAT e in una cartella di 150 caratteri: il secondo avvio porta in primo piano il primo.
- **Test.** Lunghezza del percorso; ripiego TCP con token sbagliato rifiutato.

#### WP-1.7 Chiavetta e volume · M

- **File.** `os/volume.rs`, `main.rs` (messaggio al primo avvio), `ui/src/settings/*` (riga del disco).
- **Compiti.**
  - T1.7.1 `noexec`: `statvfs` `ST_NOEXEC` (Linux), `statfs` `MNT_NOEXEC` (macOS); se la radice è `noexec`, messaggio bloccante con il comando per rimontare.
  - T1.7.2 Linux: unità rimovibile da `/sys/dev/block/<maj>:<min>/../removable` (oggi sempre `DriveKind::Other`, `:147-151`).
  - T1.7.3 FAT32 su Linux con `showexec`: avviso con il consiglio di usare exFAT; exFAT: avviso che link, socket e permessi non ci sono (le installazioni ripiegano, WP-0.2).
  - T1.7.4 Impostazioni: file system, tipo di disco, `noexec` mostrati come oggi su Windows.
- **Accettazione.** Chiavetta montata `noexec`: l'app non tenta installazioni e spiega cosa fare.
- **Test.** Funzioni pure sulle opzioni di montaggio; lettura di `removable` da un albero `/sys` finto.

#### WP-1.8 Ambiente, WebView, proxy, lingua · S

- **File.** `env.rs`, `os/webview.rs`, `net/mod.rs`, `i18n.rs`.
- **Compiti.**
  - T1.8.1 `env.rs:27-67`: `XAUTHORITY` e `WAYLAND_DISPLAY` fra le variabili di sistema conservate; `/opt/homebrew/bin` e `/usr/local/bin` dopo i percorsi del kit su macOS; `GCM_CREDENTIAL_STORE=dpapi` (`:377`) solo su Windows.
  - T1.8.2 `os/webview.rs:32-40`: su Linux verificare `libwebkit2gtk-4.1.so.0` (dlopen) prima di aprire la finestra; se manca, messaggio con il pacchetto da installare (o AppImage che lo include, WP-5.3).
  - T1.8.3 macOS: dati della WebView in `~/Library/WebKit/io.github.nabre.devkit2` dichiarati nel README (§1.4); nessun tentativo di spostarli.
  - T1.8.4 Proxy di sistema: macOS da `scutil --proxy` (lettura), Linux da `gsettings get org.gnome.system.proxy` se presente; sempre dopo le impostazioni dell'app e le variabili d'ambiente.
  - T1.8.5 `i18n.rs:41-61`: su macOS `defaults read -g AppleLanguages` prima di `LANG`.
- **Test.** Tabella attesa dell'ambiente per Linux e macOS (come quella di Windows); parsing dell'uscita di `scutil --proxy`.

#### WP-1.9 Primo avvio reale · S (verifica)

- **Obiettivo.** Prova su macchine vere (§6.1) prima di aprire la fase 2.
- **Dipende da.** WP-0.*, WP-1.1…WP-1.8.
- **Compiti.**
  - T1.9.1 Linux x64 prima in WSL2 con WSLg, dal PC di sviluppo: avvio, editor, terminale, agente. WSL2 non ha chiavette né `noexec` reali, quindi le voci su chiavetta si provano dopo su un PC o una VM Linux con desktop.
  - T1.9.2 macOS arm64 su un Mac dell'utente o in cloud, con l'artefatto della CI (WP-5.1 basta una build non firmata con firma ad hoc). Senza un Mac disponibile, WP-1.9 resta aperto per macOS e la fase 2 parte solo per Linux.
  - T1.9.3 Checklist §6.1 da chiavetta exFAT preparata su Windows (§6.5) e da disco interno.
  - T1.9.4 Problemi trovati: si riaprono i pacchetti della fase 1. Esito in TESTING.md.
- **Accettazione.** Checklist §6.1 completa su Linux x64 e su macOS arm64.

---

### Fase 2: componenti da sistemare

Fine fase: ogni casella «da sistemare» della matrice diventa «attiva».

#### WP-2.1 Git rilocabile su Unix · M

- **Dipende da.** Q-5.
- **File.** `install/core.rs:28-131`, `env.rs`, `tauri/catalog/components.json`.
- **Compiti.**
  - T2.1.1 Installatore Unix su `desktop/dugite-native` (ultima release, asset `dugite-native-v<v>-<sha>-{macOS-arm64,macOS-x64,ubuntu-x64,ubuntu-arm64}.tar.gz`, `.sha256`); niente più micromamba per Git.
  - T2.1.2 Ambiente: `GIT_EXEC_PATH=<git>/libexec/git-core`, `GIT_TEMPLATE_DIR=<git>/share/git-core/templates`, `PREFIX=<git>` (README di dugite), solo su Unix.
  - T2.1.3 `Installer::relocatable(target)` torna vero per Git; i `.reinstall` di Git esistenti si eseguono una volta e poi spariscono.
  - T2.1.4 Credential helper su Unix: `osxkeychain` su macOS (incluso in dugite) o `cache --timeout=3600`; `.gitconfig` portabile del kit come su Windows.
- **Accettazione.** Kit spostato di cartella su Linux: `git --version` risponde senza reinstallare; `git clone` HTTPS di un repository privato con l'accesso del kit.
- **Test.** Scelta dell'asset per i 4 target; ambiente atteso.

#### WP-2.2 Ruby su Unix · M

- **File.** `install/lang/ruby.rs:69-143`, `install/native.rs:495-627` (riuso).
- **Compiti.** T2.2.1 conda-forge `ruby` (linea scelta) via `micromamba_create`, come R. T2.2.2 `relocatable` falso fuori da Windows, con `.reinstall`. T2.2.3 Elenco delle linee da `conda_versions("ruby")`. T2.2.4 `gem` installa nel kit (`GEM_HOME` in `data/ruby-gems/<linea>`), con i compilatori di WP-2.3 per le gem native.
- **Accettazione.** `ruby -v`, `gem install json` (nativa) su macOS e Linux.
- **Test.** Comando `micromamba create` atteso; linee da una risposta di esempio.

#### WP-2.3 C e C++ su Unix con zig cc · M

- **Dipende da.** Q-5.
- **File.** `install/lang/cc.rs`, `env.rs`, `setup/needs.rs`, `tauri/catalog/components.json`, `tauri/catalog/relations.json`.
- **Compiti.**
  - T2.3.1 Su Unix il componente `cc` dipende da `zig` (`needs`) e installa solo shim in `cc/bin`: `cc`, `c++`, `gcc`, `g++`, `clang`, `clang++` → `zig cc`/`zig c++`; `ar`, `ranlib` → `zig ar`/`zig ranlib`. Shim sh di una riga, `chmod 755`.
  - T2.3.2 `make` portabile: GNU make da conda-forge (`make`) via micromamba, oppure build statica; `cc/bin/make`.
  - T2.3.3 `env.rs`: `CC=<cc>/bin/cc`, `CXX=<cc>/bin/c++`, `AR`, `RANLIB` quando `cc` è installato (oggi nessuna variabile), solo su Unix.
  - T2.3.4 macOS: `xcode-select -p` all'installazione e nella pagina del componente; senza Command Line Tools, avviso: «I programmi che usano i framework Apple richiedono Xcode Command Line Tools».
  - T2.3.5 Rimuovere la logica LLVM per Unix (`cc.rs:144-235`), incluso il ripiego fisso 20.1.7 per macOS Intel (D-104).
  - T2.3.6 Windows invariato (llvm-mingw).
- **Accettazione.** Su Linux senza `build-essential`: `cc hello.c && ./a.out`, `cargo build` di un crate con `cc-rs`, `go build` con cgo. Su macOS senza CLT: `cc hello.c`.
- **Test.** Shim generati; ambiente con e senza `cc`.

#### WP-2.4 Rust su Windows ARM64 · M

- **File.** `install/native.rs:226-272`, `tauri/catalog/components.json` (`rust.needs` su ARM64: `cc`).
- **Compiti.** T2.4.1 Host `aarch64-pc-windows-gnullvm` invece di `aarch64-pc-windows-msvc`. T2.4.2 `cc` (llvm-mingw `ucrt-aarch64`) diventa una dipendenza di Rust su `win32-arm64`; `CARGO_TARGET_AARCH64_PC_WINDOWS_GNULLVM_LINKER=<cc>/bin/clang.exe` nell'ambiente. T2.4.3 Impostazione «Compilatore di Rust» su ARM64: «GNU (LLVM)» predefinito, «MSVC» solo se c'è Visual Studio.
- **Accettazione.** `cargo new x && cargo run` su Windows 11 ARM64 senza Visual Studio.
- **Test.** Argomenti di `rustup-init` per i due host; ambiente atteso.

#### WP-2.5 MongoDB e SQLite su Linux · S–M

- **File.** `install/db.rs:187-445`, `router/bins.rs:37-46`.
- **Compiti.** T2.5.1 MongoDB su Linux: conda-forge `mongodb` (server); se mongosh e Database Tools non sono su conda-forge, restano dai fornitori (mongosh generico `linux-x64`/`linux-arm64` già indipendente dalla distribuzione; Database Tools dalla distribuzione rilevata in `/etc/os-release`). T2.5.2 Layout: binari conda in `bin/`, chiavi del catalogo invariate (`mongodb/bin/mongod`). T2.5.3 SQLite `linux-arm64`: conda-forge `sqlite`; togliere `unavailable` messo in T0.4.4.
- **Accettazione.** `mongod --version` su Debian 13 e Fedora; `sqlite3 --version` su Linux ARM64.
- **Test.** Scelta della distribuzione da file `os-release` di esempio.

#### WP-2.6 MariaDB su Linux x64 · M

- **File.** `db/services/mod.rs:484-492`, `db/services/data.rs:155-187`, `db/services/access.rs`, `db/auth.rs:593-624`.
- **Compiti.** T2.6.1 Inizializzazione Unix: `scripts/mariadb-install-db --no-defaults --basedir=<line> --datadir=<data> --auth-root-authentication-method=normal`. T2.6.2 `my.cnf` scritto dal kit (porta, `bind-address=127.0.0.1`, `socket` e `pid-file` nella cartella dati), avvio con `--defaults-file=<data>/my.cnf`. T2.6.3 `--user=$(id -un)` solo se l'app gira come root. T2.6.4 Controllo di `libaio` all'avvio con un messaggio chiaro se manca.
- **Accettazione.** Su Ubuntu 24.04: avvio, password di root, database di un progetto, copia e ripristino.
- **Test.** Argomenti e `my.cnf` attesi per Unix.

#### WP-2.7 Correzioni piccole · S

- **Compiti.**
  - T2.7.1 TinyTeX: `tlmgr_path` con il prefisso `tinytex/` (`install/lang/typeset.rs:80-81`); nuovi nomi degli asset dal 2026.04 (`TinyTeX-<n>-<os>-<arch>.xz`), con i vecchi come ripiego.
  - T2.7.2 Bun: build `-baseline` se la CPU non ha AVX2 (`is_x86_feature_detected!("avx2")`), solo x64.
  - T2.7.3 pnpm/yarn: `removeUnix` di WP-0.4 usato dalla rimozione.
  - T2.7.4 `.NET` su Linux: `DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1` se `libicu` manca.
  - T2.7.5 Ruby, R, Git (prima di WP-2.1): `relocatable` coerente con il catalogo.
- **Test.** Uno per compito.

---

### Fase 3: stack web PHP e database su tutti i sistemi

Fine fase: un progetto Laravel con database parte su macOS e Linux come su Windows.

#### WP-3.1 Componente Caddy · S

- **Dipende da.** Q-3.
- **File.** `install/web.rs` (o `install/caddy.rs` nuovo), `install/mod.rs`, `tauri/catalog/components.json`, `ui/src/logos.tsx`.
- **Compiti.** T3.1.1 Installatore: GitHub `caddyserver/caddy`, asset `caddy_<v>_{windows,mac,linux}_{amd64,arm64}.{zip,tar.gz}`, SHA-256 da `caddy_<v>_checksums.txt` (o `digest`). T3.1.2 Voce di catalogo `caddy` (gruppo linguaggi e strumenti, tutti i target, chiave `caddy/caddy.exe` → `caddy/caddy`, `check: ["caddy/caddy.exe", "version"]`, `versioned`).
- **Accettazione.** `Dev-Kit.exe prepare --only caddy` sui 6 target (CI per i 4 Unix).
- **Test.** Asset per target; parsing di `checksums.txt`.

#### WP-3.2 PHP su macOS e Linux · L

- **Dipende da.** Q-5.
- **File.** `install/web.rs:318-630`, `tauri/catalog/components.json` (`php`, `composer`, `phpmyadmin`), `setup/catalog.rs` (`translate_rel` di `php/bin/php` già pronto).
- **Compiti.**
  - T3.2.1 Installatore Unix: static-php-cli `php-<v>-cli-<os>-<arch>.tar.gz` e `php-<v>-fpm-<os>-<arch>.tar.gz` da `dl.static-php.dev/static-php-cli/common/`, in `php/<linea>/bin/php` e `php/<linea>/sbin/php-fpm`.
  - T3.2.2 Verifica: SHA-256 per linea e target fissati nel catalogo (`sha256ByTarget`); aggiornati con uno script (`cargo test live_lines -- --ignored` esteso) a ogni rilascio del kit. Un asset con un hash diverso da quello fissato non si installa.
  - T3.2.3 Linee: quelle per cui c'è un hash fissato (8.4, 8.5).
  - T3.2.4 `php.ini` per sistema: su Unix niente `extension=` (estensioni statiche), `curl.cainfo`/`openssl.cafile` con `/`, `date.timezone` invariato.
  - T3.2.5 Composer: stesso phar, lanciatore sh già presente (`web.rs:518-534`).
  - T3.2.6 Catalogo: togliere `targets` da `php`, `composer`, `phpmyadmin`. phpMyAdmin 5.2.3 dichiara `php <8.4` (D-107): provare con 8.4 e 8.5 di static-php-cli; se fallisce, phpMyAdmin usa la linea 8.4.
- **Accettazione.** `php -v`, `php -m` (le estensioni di oggi: curl, fileinfo, gd, intl, mbstring, mysqli, openssl, pdo_mysql, pdo_sqlite, sodium, sqlite3, zip), `composer create-project laravel/laravel` su macOS e Linux.
- **Test.** Asset per target; `php.ini` atteso per Unix; rifiuto di un hash diverso.

#### WP-3.3 Caddy nei pool e nei siti dei progetti · M

- **Dipende da.** WP-3.1, WP-3.2 (per PHP su Unix).
- **File.** `servers/mod.rs`, `servers/config.rs`, `servers/process.rs`, `servers/pools.rs`, `projects/run.rs:506-545`, `projects/caddy.rs` (nuovo), `projects/state.rs`, `detect/mod.rs:79-84`, `detect/overrides.rs`, `env.rs` (`CP_CADDY`).
- **Compiti.**
  - T3.3.1 `WebServerEngine::Caddy` con `as_str` `"caddy"`, `label` «Caddy».
  - T3.3.2 `servers/config.rs`: `caddy_pool_conf` e `caddy_site_conf` (§4.4), con gli alias di `projects/alias.rs` (solo `SiteAlias` risolti, come Apache e Nginx).
  - T3.3.3 `servers/process.rs`: avvio `caddy run`, ricarica `caddy reload` sulla porta admin dell'istanza, porta admin scelta come le altre porte del kit.
  - T3.3.4 PHP: su Windows riuso di `spawn_php_cgi_if_present` (`projects/nginx.rs:317-363`) e `fcgi-guard.php`; su Unix php-fpm con `fpm.conf` generato (`listen = 127.0.0.1:<p>`, `pm = ondemand`, `clear_env = no`), stessa chiave per avvio passata con `env DEVKIT_FCGI_KEY`.
  - T3.3.5 `Action.caddy_dir` serializzato come `"caddy"` in `panel.json`; `SiteKind::Caddy` in `detect/overrides.rs`.
  - T3.3.6 Rilevamento: Caddy proposto per i progetti PHP su tutti i sistemi; Apache proposto solo su Windows e solo se il progetto ha un `.htaccess` con regole oltre quelle note di WordPress/Laravel/Drupal (`detect/php.rs:47-112`).
  - T3.3.7 File nascosti: `.git`, `.env`, `private/` e i file che oggi `hidden_deny` di Apache nega, rifiutati con 404.
- **Accettazione.** Pool con due siti PHP e un alias `@altro/cartella` su Windows, macOS e Linux; `caddy reload` dopo l'aggiunta di un sito senza interrompere l'altro.
- **Test.** Caddyfile atteso per pool e sito (come i test di Nginx); nessun alias non risolto nel testo; `fpm.conf` atteso.

#### WP-3.4 Interfaccia dei server · S

- **File.** `ui/src/servers.tsx`, `ui/src/api/servers.ts` (`caddyLines`), `ui/src/projects/site.tsx`, `ui/src/projects/needs.tsx`, guida.
- **Compiti.** T3.4.1 Caddy fra i motori dei pool e del sito. T3.4.2 Apache nascosto fuori da Windows. T3.4.3 Testi «Caddy non legge .htaccess: le regole di riscrittura si generano dal tipo di progetto».
- **Test.** Vitest delle pagine con Caddy.

#### WP-3.5 Togliere Nginx · M

- **Dipende da.** WP-3.3 in uso su Windows per almeno un rilascio.
- **File.** `servers/pools.rs` (precedente: `migrate_port_ids`, `:197`), `projects/migrate.rs`, `install/web.rs:767-841`, `tauri/catalog/components.json`, `versions/disk.rs` (`RETIRED`).
- **Compiti.** T3.5.1 Migrazione all'avvio: pool e siti `nginx-…` diventano `caddy-…` in `servers.json` e `panel.json` (`"nginx"` → `"caddy"`), con copia `pre-caddy`. T3.5.2 `nginx` in `RETIRED` come Adminer (D-107). T3.5.3 Codice di Nginx tolto (`projects/nginx.rs`, ramo in `servers/*`), tranne il php-cgi già spostato in un modulo comune.
- **Accettazione.** Kit con un pool Nginx: dopo l'aggiornamento il pool parte con Caddy sulla stessa porta.
- **Test.** Migrazione di `servers.json` e `panel.json` di esempio; ripetibile.

#### WP-3.6 `DbEngine` senza cambiare comportamento · M

- **Dipende da.** Q-4.
- **File.** `db/mod.rs` (enum), `db/services/*.rs`, `db/auth.rs`, `db/browse.rs`, `db/phpmyadmin.rs`, `commands/databases.rs`, `router/bins.rs`, `env.rs`, `detect/*`, `projects/frameworks/mod.rs`, `projects/state.rs`, `setup/needs.rs`.
- **Compiti.**
  - T3.6.1 Enum `DbEngine` (§4.3) con `MariaDb` e `MongoDb`; `SERVERS` (`db/services/mod.rs:61`) diventa `DbEngine::ALL`.
  - T3.6.2 Ogni `match id { "mongodb" => …, _ => … }` diventa un `match engine` esaustivo (il compilatore trova tutti i posti).
  - T3.6.3 Nomi dei binari dai metodi dell'enum: `db/browse.rs:150`, `db/services/access.rs:64,198,267`, `db/services/backups.rs:96,149`, `db/services/mod.rs:721`, `db/services/tools.rs:74-115`.
  - T3.6.4 Nomi riservati per motore (`db/auth.rs:637-655`).
- **Accettazione.** Nessuna differenza di comportamento: tutti i test di `db/*` invariati e verdi.
- **Test.** Quelli esistenti; un test che `DbEngine::ALL` copre `SERVERS` di oggi.

#### WP-3.7 Componente MySQL · M

- **Dipende da.** WP-3.6, WP-0.4 (`sha256ByTarget`).
- **File.** `install/db.rs`, `install/mod.rs`, `tauri/catalog/components.json`.
- **Compiti.**
  - T3.7.1 Prima di tutto, verifica sulle build vere: avvio senza root, file di opzioni letti da `mysql` su Windows e Unix (`MYSQL_HOME`, `--defaults-extra-file`), nome di libaio su Ubuntu 24.04. Esito in DECISIONS.md (D-110) prima di T3.7.2.
  - T3.7.2 Installatore: linee 8.4 LTS e 9.7 LTS; asset `mysql-<v>-winx64.zip`, `mysql-<v>-macos15-{arm64,x86_64}.tar.gz`, `mysql-<v>-linux-glibc2.28-x86_64-minimal.tar.xz`, `mysql-<v>-linux-glibc2.28-aarch64.tar.xz`; URL `https://cdn.mysql.com/Downloads/MySQL-<linea>/<file>`.
  - T3.7.3 Verifica: SHA-256 per linea e target fissati nel catalogo, calcolati dal coordinatore dopo aver verificato la firma GPG di Oracle; nessuna installazione se l'hash è diverso.
  - T3.7.4 Prima del passaggio in `data/apps`: via `mysql-test`, `*.pdb`, `lib/plugin/debug` (dimensioni).
  - T3.7.5 Catalogo: `mysql` (gruppo database, tutti i target, `x64Only` per `win32-arm64`, chiave `mysql/bin/mysqld.exe`, `versioned`).
- **Accettazione.** `prepare --only mysql` sui 6 target; download Linux ARM64 dichiarato (~943 MB) nella pagina del componente.
- **Test.** Asset e URL per target e linea; rifiuto di un hash diverso.

#### WP-3.8 Servizio MySQL · L

- **Dipende da.** WP-3.7.
- **File.** `db/services/mod.rs`, `db/services/data.rs`, `db/services/access.rs`, `db/services/backups.rs`, `db/auth.rs`, `env.rs`, `setup/kit_files.rs` (lanciatori), `projects/run.rs:563-577`.
- **Compiti.**
  - T3.8.1 Inizializzazione (§4.3) con `lower_case_table_names=1`, `my.cnf` del kit, avvio `mysqld --defaults-file=<data>/my.cnf`.
  - T3.8.2 Password di root: `ALTER USER 'root'@'localhost' IDENTIFIED BY …` e creazione di `'root'@'127.0.0.1'` (MySQL crea solo `localhost`).
  - T3.8.3 Utenti dei progetti con `caching_sha2_password` su `localhost` e `127.0.0.1`; `GRANT` con `_` escapato come per MariaDB.
  - T3.8.4 Accesso dal terminale e dalle azioni: `data/app/db/mysql.cnf` (`[client]`, 600) e lanciatori `mysql`, `mysqldump` in `data/kit/bin` con `--defaults-extra-file`; `project_mysql_home` (`db/services/mod.rs:259-270`) per progetto allo stesso modo.
  - T3.8.5 Copie: `mysqldump --single-transaction --routines --events`; ripristino come utente del progetto (come MariaDB, `3071169`).
  - T3.8.6 Arresto: `mysqladmin shutdown`, poi `SIGTERM`/`TerminateProcess`.
  - T3.8.7 phpMyAdmin e l'explorer dei database con MySQL (stessa porta scelta per il motore).
- **Accettazione.** Su macOS e Linux ARM64: database di un progetto Laravel, migrazioni, copia e ripristino, phpMyAdmin; stessa cartella dati letta su Windows e Linux con la stessa linea.
- **Test.** Argomenti e `my.cnf` attesi; SQL di utenti e grant; `real_mysql_*` con `DEVKIT_TEST_KIT` (come `real_phpmyadmin_on_running_mariadb`).

#### WP-3.9 MySQL nell'interfaccia, rilevamento, migrazione · M

- **Dipende da.** WP-3.8.
- **File.** `ui/src/databases/page.tsx`, `ui/src/api/databases.ts`, `ui/src/api/projects.ts`, `ui/src/api/frameworks.ts`, `ui/src/frameworks/fields.tsx`, `ui/src/guide/model.ts`, `ui/src/logos.tsx`, locali; `detect/mod.rs:465-514,709-711`, `detect/container.rs:80-82`, `detect/php.rs:9`, `tauri/catalog/frameworks.json`, `tauri/catalog/extensions.json:69-70`, `tauri/catalog/relations.json:62-63`; `db/migrate_engine.rs` (nuovo).
- **Compiti.**
  - T3.9.1 Tipi `"mariadb" | "mysql" | "mongodb"`; controlli `s.id === "mariadb"` diventano «motore SQL».
  - T3.9.2 Motore predefinito: MySQL per i kit nuovi e dove MariaDB non c'è; MariaDB per i kit con dati MariaDB.
  - T3.9.3 Rilevamento: immagine `mysql` di docker compose → MySQL con la sua linea; `mariadb` → MariaDB.
  - T3.9.4 Framework: opzione «MySQL» accanto a «MariaDB» in `frameworks.json` (`:229,269,312,697-705`).
  - T3.9.5 «Migra a MySQL»: dump di ogni database del progetto con `mariadb-dump`, filtro delle parti solo MariaDB (`/*M!`, sequenze, collazioni `uca1400_*` → `utf8mb4_0900_ai_ci`), ripristino in MySQL, utenti e password ricreati da `db.json`; MariaDB non toccato finché l'utente non lo rimuove.
- **Accettazione.** Progetto WordPress su MariaDB migrato a MySQL e funzionante; nessun dato perso nell'originale.
- **Test.** Filtro del dump su esempi; Vitest della pagina con tre motori.

---

### Fase 4: accessi portabili

#### WP-4.1 Keyring di Antigravity su Unix · M

- **File.** `agents/keyring.rs:89-280`, `Cargo.toml` (`keyring` con `apple-native`, `sync-secret-service`, `cfg(unix)`).
- **Compiti.** T4.1.1 `list`, `write`, `delete` su Portachiavi e Secret Service per il servizio di go-keyring usato da `agy` (nome da verificare con `agy` reale). T4.1.2 Copia nel kit cifrata con una chiave tenuta nel Portachiavi o in Secret Service; senza Secret Service, file 600 e avviso. T4.1.3 Su macOS la scrittura prima e la cancellazione dopo ogni avvio di `agy`, anche dopo un crash (all'avvio dell'app si cancella un accesso rimasto).
- **Accettazione.** Accesso fatto su un Mac, kit portato su un altro Mac: `agy` è già collegato; sul primo Mac non resta nulla nel Portachiavi.
- **Test.** Con un archivio finto dietro un trait.

#### WP-4.2 Claude Code su macOS · M

- **File.** `agents/mod.rs:197`, `env.rs`, `ui/src/agents/*`.
- **Compiti.** T4.2.1 Pulsante «Collega con un token» su macOS: esegue `claude setup-token` in un PTY (WP-1.2), salva il token in `data/agents/claude/oauth-token` (600). T4.2.2 `CLAUDE_CODE_OAUTH_TOKEN` nell'ambiente di Claude Code se il file c'è. T4.2.3 «Esci» cancella il file.
- **Accettazione.** Kit su un altro Mac: Claude Code già collegato.
- **Test.** Ambiente con e senza token; il token non compare nei log.

---

### Fase 5: pacchetto e distribuzione

#### WP-5.1 Configurazione dei bundle · S

- **File.** `tauri/tauri.conf.json`, `tauri/icons/` (`.icns`), `tauri/Info.plist`, `tauri/entitlements.plist`.
- **Compiti.** T5.1.1 `bundle.active` per macOS (`app`) e Linux (`appimage`), Windows invariato (exe singolo). T5.1.2 Icona `.icns` da `docs/brand/symbol-512.png`. T5.1.3 `Info.plist`: `LSMinimumSystemVersion` 12.0, nome, identificatore `io.github.nabre.devkit2`. T5.1.4 Entitlements del runtime protetto: `com.apple.security.cs.allow-jit` (WebView), `com.apple.security.cs.disable-library-validation` (programmi scaricati), da verificare uno per uno.

#### WP-5.2 macOS: firma e notarizzazione · L

- **Dipende da.** Q-2, WP-5.1, WP-1.1.
- **File.** `.github/workflows/app.yml`.
- **Compiti.** T5.2.1 Build universale (`--target universal-apple-darwin`) o due build. T5.2.2 Con Q-2 = sì: `codesign --options runtime --timestamp` con Developer ID, `notarytool submit --wait`, `stapler staple`; segreti in GitHub. T5.2.3 Senza firma: firma ad hoc (`codesign -s -`), istruzioni nel README. T5.2.4 Archivio di rilascio: `Dev-Kit-macos.zip` con `Dev-Kit.app` dentro una cartella `Dev-Kit/`.
- **Accettazione.** Su un Mac pulito: scaricare lo zip con il browser, estrarlo su una chiavetta exFAT, aprire `Dev-Kit.app` (con firma: senza avvisi; senza: con le istruzioni).

#### WP-5.3 Linux: AppImage e tar.gz · L

- **Dipende da.** Q-7, WP-1.1.
- **Compiti.** T5.3.1 Build su Ubuntu 22.04 (glibc 2.35) per una glibc minima ragionevole, x64 e arm64. T5.3.2 AppImage con WebKitGTK incluso (linuxdeploy con il plugin GTK). T5.3.3 Senza FUSE: messaggio dell'AppImage con `--appimage-extract-and-run`; in più un `Dev-Kit-linux-<arch>.tar.gz` con l'eseguibile. T5.3.4 Valutare `rustls` con `rustls-platform-verifier` al posto di `native-tls` per non dipendere dalla versione di OpenSSL del sistema.
- **Accettazione.** AppImage avviata su Ubuntu 24.04, Fedora 42, Debian 13 da chiavetta exFAT.

#### WP-5.4 Aggiornamento dell'app su macOS e Linux · M

- **File.** `setup/self_update.rs:69-75,220-244`.
- **Compiti.** T5.4.1 macOS: scaricare il nuovo `.app` zip, verificarlo, estrarlo accanto (`Dev-Kit.app.new`), all'uscita rinominare il vecchio in `Dev-Kit.app.old` e il nuovo in `Dev-Kit.app`; cancellare `.old` al prossimo avvio. T5.4.2 Linux: nuovo AppImage in `Dev-Kit.AppImage.new`, `chmod 755`, scambio all'uscita. T5.4.3 Nomi senza `.exe` fuori da Windows.
- **Accettazione.** Aggiornamento da una versione di prova a un'altra su macOS e Linux.
- **Test.** Piano dello scambio (funzione pura) per i tre sistemi.

#### WP-5.5 Rilasci · S

- **Compiti.** T5.5.1 Job di rilascio per `darwin` (zip) e `linux-x64`/`linux-arm64` (AppImage e tar.gz) accanto a `Dev-Kit-x64.exe` e `Dev-Kit-arm64.exe` (`app.yml:89-134`). T5.5.2 Prova di avvio dell'artefatto in CI (`--version`, `check` con `xvfb-run` su Linux).

---

### Fase 6: interfaccia e documenti

#### WP-6.1 Griglia componente × target in «Strumenti» · M

- **File.** `commands/setup.rs` (`target_profiles`), `ui/src/components/table.tsx:206-236`, `ui/src/components/state.ts`, `ui/src/api/setup.ts`.
- **Compiti.** T6.1.1 Comando `target_profiles` che espone `router::target::all_profiles` e `probe_all_targets` (oggi non usati). T6.1.2 Vista «Matrice» con le 45 righe e 6 colonne, stati come nella matrice, dettaglio al clic. T6.1.3 Nomi leggibili dei sistemi («macOS Apple Silicon»), marcatori «questo PC», «in emulazione», «non eseguibile qui». T6.1.4 Target scelto ricordato fra le sessioni (`settings.json`). T6.1.5 «non disponibile su PC ARM64» solo quando l'host è Windows ARM64.

#### WP-6.2 Matrici per target · M

- **Compiti.** T6.2.1 Relazioni (`ui/src/relations.tsx`, `relations_matrix`) ed estensioni (`ui/src/extensions.tsx`, `extensions_matrix`) filtrate per il target scelto.

#### WP-6.3 Documenti · S

- **Compiti.**
  - T6.3.1 README e traduzioni (`docs/README.*.md`): «Limiti noti» con i sistemi supportati e il §1.4; «Come si usa» per macOS e Linux.
  - T6.3.2 `PLAN.md:35`, `GUIDA.md:61`: tolto «solo Windows».
  - T6.3.3 DECISIONS.md: decisioni nuove (§9) e note **Oggi** su D-57, D-74, D-85, D-92, D-95, D-104, D-105, D-107.
  - T6.3.4 CATALOG.md: tabella delle fonti per target (§4.2).
  - T6.3.5 TESTING.md: checklist del §6.
  - T6.3.6 MATRICE-PIATTAFORME.html aggiornata a fine di ogni fase (stati e piano).

## 6. Prove manuali (in TESTING.md)

### 6.1 Primo avvio (WP-1.9), per macOS arm64, macOS x64, Linux x64, Linux arm64

- [ ] Kit su disco interno e su chiavetta exFAT: avvio, creazione di `projects/` e `data/` accanto all'app, niente dentro il bundle.
- [ ] Secondo avvio: porta in primo piano il primo.
- [ ] Componenti obbligatori installati (pwsh, gh, Git, editor) con verifica; `Dev-Kit check` verde.
- [ ] Nuovo progetto vuoto e clone di un repository privato.
- [ ] Editor: apertura, terminale integrato con l'ambiente del kit, estensioni installate.
- [ ] Terminale del kit e Claude Code avviati da un progetto.
- [ ] Chiusura dell'app con un terminale e un database aperti: nessun processo resta.
- [ ] `kill -9` dell'app: nessun processo resta entro 6 s.
- [ ] Kit spostato in un'altra cartella: tutto riparte (Git senza reinstallare dopo WP-2.1).
- [ ] Chiavetta montata `noexec` (Linux): messaggio, nessuna installazione.
- [ ] «Rimuovi l'unità»: chiude tutto e lascia espellere.

### 6.2 Componenti (fine fase 2)

- [ ] Ogni componente disponibile per il target: installazione, `check`, rimozione senza resti.
- [ ] Android e SQLite su Linux ARM64 mostrati non disponibili con il motivo (prima di WP-2.5 per SQLite).
- [ ] C/C++ su Linux senza `build-essential`, su macOS senza Xcode CLT.
- [ ] Rust su Windows 11 ARM64 senza Visual Studio.

### 6.3 Stack web e database (fine fase 3)

- [ ] Laravel con MySQL su macOS e Linux: `composer create-project`, migrazioni, sito su Caddy, phpMyAdmin.
- [ ] WordPress con Caddy: permalink funzionanti senza `.htaccess`.
- [ ] Pool con due siti e un alias verso un altro progetto.
- [ ] Stessa cartella dati MySQL letta da Windows e da Linux.
- [ ] «Migra a MySQL» di un progetto MariaDB.
- [ ] Kit con un pool Nginx aggiornato: il pool riparte con Caddy (WP-3.5).

### 6.4 Accessi e pacchetto (fasi 4 e 5)

- [ ] Antigravity e Claude Code collegati su un Mac, kit portato su un altro Mac: già collegati; nessuna traccia nel Portachiavi del primo.
- [ ] Download dei rilasci con il browser, apertura su un sistema pulito, aggiornamento dell'app.

### 6.5 Kit preparato su Windows per un altro sistema (a ogni fase)

- [ ] Su Windows: `Dev-Kit.exe prepare --target linux-x64` (poi `darwin-arm64`, `linux-arm64`) su una chiavetta exFAT, con l'app per quel sistema copiata accanto.
- [ ] Il selettore «Piattaforma» mostra per quel target cosa è pre-scaricato, cosa si completa al primo avvio e cosa non è disponibile.
- [ ] Sull'altro sistema: primo avvio dalla chiavetta, installazioni da `.bootstrap` completate (Rust, Git/R con micromamba, Android, pnpm), nessun nuovo download per ciò che era già scaricato.
- [ ] Di nuovo su Windows: il kit si apre come prima; `data/apps/win32-x64` e le impostazioni di Windows sono intatti.
- [ ] Stesso progetto aperto sui due sistemi: impostazioni, porte e database del progetto condivisi (`data/projects/<p>`).

## 7. Ondate e agenti

### 7.1 Regole

Come PLAN-DATA.md §7.1: un solo agente scrive un file in un'ondata; file condivisi (righe `mod`, `main.rs`, `commands/mod.rs`, `Cargo.toml`, `ui/src/locales/*`, `tauri/i18n/catalog.json`, `components.json`) solo dal coordinatore; un worktree per agente, branch `plat/<agente>` dal branch d'integrazione `feature/platforms`; uscita con `cargo fmt`, `clippy -D warnings`, test del modulo verdi; al massimo 5 agenti per ondata. In più:

- **ogni agente prova su Windows che il suo pacchetto non cambia nulla lì** (test esistenti verdi);
- chi tocca codice Unix esegue anche `scripts/test-linux.sh` in WSL2 e lo scrive nel rapporto;
- il coordinatore fonde un'ondata solo con la CI verde su tutti i job (Windows x64 e ARM64, Linux x64 e ARM64, macOS arm64 e x64).

### 7.2 Agenti

| Id | Ruolo | Pacchetti | Scrive | Modello / sforzo |
| --- | --- | --- | --- | --- |
| **C** | Coordinatore | ondate, merge, file condivisi, hash fissati, WP-1.9 con l'utente | file condivisi, `components.json` | Opus, alto |
| **P1** | Percorsi e file system | WP-0.1, WP-0.2 | file di WP-0.1, `net/archive.rs`, `versions/side_by_side.rs` | Sonnet, medio |
| **P2** | Catalogo e router | WP-0.4, WP-0.5, WP-6.1 (backend) | `setup/catalog.rs`, `router/*`, `setup/runner.rs`, `agents/relocate.rs`, `commands/setup.rs` | Opus, alto |
| **P3** | Sistema operativo | WP-0.3, WP-1.1, WP-1.5…WP-1.8 | `runtime/*`, `setup/kit_files.rs`, `kit.rs`, `os/*`, `env.rs`, `i18n.rs`, `net/mod.rs` | Opus, alto |
| **P4** | Terminale ed editor | WP-1.2, WP-1.3, WP-1.4 | `editor/*`, `commands/terminals.rs`, `agents/mod.rs`, `ui/src/terminal/*` | Opus, alto |
| **P5** | Componenti | Fase 2 (tranne WP-2.6) | `install/core.rs`, `install/lang/*`, `install/native.rs` | Sonnet, alto |
| **P6** | Server web | WP-3.1…WP-3.5 | `install/web.rs`, `servers/*`, `projects/run.rs`, `projects/caddy.rs`, `projects/nginx.rs`, `projects/apache.rs`, `detect/*` | Opus, alto |
| **P7** | Database | WP-2.6, WP-3.6…WP-3.9 (backend) | `db/*`, `install/db.rs`, `commands/databases.rs` | Opus, alto |
| **P8** | Accessi | Fase 4 | `agents/keyring.rs`, `ui/src/agents/*` | Opus, alto |
| **P9** | Pacchetto e CI | WP-0.6, Fase 5 | `.github/workflows/*`, `tauri/tauri.conf.json`, `tauri/icons`, `setup/self_update.rs` | Sonnet, medio |
| **U** | Interfaccia | WP-3.4, WP-3.9 (UI), WP-6.1 (UI), WP-6.2 | `ui/src/**` tranne `ui/src/terminal` e `ui/src/agents` | Sonnet, medio |
| **R** | Revisore | ogni merge | nulla | Opus, alto |
| **V** | Verificatore | dopo ogni merge | nulla (solo comandi, anche in CI su Unix) | Sonnet, basso |
| **D** | Documenti | WP-6.3 | `docs/app/*` | Sonnet, medio |

### 7.3 Ondate

```text
Ondata 0  P1 (WP-0.1, WP-0.2) │ P2 (WP-0.4, WP-0.5) │ P3 (WP-0.3) │ P9 (WP-0.6)
          → merge P9, P1, P3, P2; R e V dopo ciascuno
Ondata 1  P3 (WP-1.1, WP-1.5…1.8) │ P4 (WP-1.2…1.4)
          → merge P3, P4; C con l'utente: WP-1.9 (macOS, Linux)
Ondata 2  P5 (WP-2.1…2.5, 2.7) │ P7 (WP-2.6, WP-3.6) │ P6 (WP-3.1)
          → merge P7 (WP-3.6 prima di tutto), P5, P6
Ondata 3  P6 (WP-3.2, WP-3.3) │ P7 (WP-3.7, WP-3.8) │ P8 (WP-4.1, WP-4.2) │ P9 (WP-5.1, 5.3, 5.4)
          → merge P6, P7, P8, P9; C: hash fissati di PHP e MySQL
Ondata 4  U (WP-3.4, WP-3.9 UI, WP-6.1, WP-6.2) │ P7 (WP-3.9 backend) │ P9 (WP-5.2, WP-5.5)
          → merge P7, U, P9
Ondata 5  P6 (WP-3.5, dopo un rilascio con Caddy) │ D (WP-6.3) │ C (§6 completo, /security-review)
```

- WP-3.6 (`DbEngine`) si fonde prima di ogni altro lavoro su `db/*`: è un refactoring che tocca molti file.
- WP-0.6 per primo: tutti i merge successivi passano dai test su Unix.
- WP-1.9 richiede l'utente con un Mac e un PC Linux: il coordinatore prepara la lista e aspetta l'esito prima dell'ondata 2.
- Gli hash fissati (PHP, MySQL) li calcola il coordinatore dopo aver verificato la firma del fornitore; gli agenti non li inventano.

### 7.4 Richiesta all'agente e rapporto

Come PLAN-DATA.md §7.4, con il riferimento a questo documento e una riga in più nel rapporto: «Provato su: <sistemi>».

## 8. Rischi

| Rischio | Effetto | Contromisura |
| --- | --- | --- |
| Codice Unix mai avviato (D-105) | Molti difetti scoperti tutti insieme | WP-0.6 per primo; WP-1.9 prima della fase 2 |
| Sviluppo solo su Windows | Difetti di maiuscole, permessi e link non visti in locale | Logica come funzioni pure provate per i 6 target su Windows; WSL2 (`scripts/test-linux.sh`) e CI per il resto (§2.1) |
| Nessun Mac a disposizione | macOS provato solo in CI, mai avviato | CI con artefatto; Mac in cloud; WP-1.9 macOS resta aperto finché non c'è |
| WSL2 diverso da un Linux vero | Chiavette, `noexec`, desktop non provati | WSL2 solo per avvio, editor e test; chiavetta e desktop su un PC o una VM Linux (T1.9.1) |
| Regressioni su Windows | Utenti di oggi colpiti | Test Windows a ogni merge; principio «Windows invariato»; revisore |
| Hash fissati non aggiornati | PHP o MySQL nuovi non installabili | Script di aggiornamento degli hash a ogni rilascio; si offrono solo le linee con hash |
| Fornitore che cambia i nomi degli asset (TinyTeX 2026.04) | Installazione fallita | Ripiego sui nomi vecchi; `live_lines` in CI settimanale |
| MySQL non legge il file di opzioni come MariaDB | Accesso dal terminale rotto | T3.7.1 prima del codice; lanciatori con `--defaults-extra-file` |
| `lower_case_table_names` diverso fra sistemi | Cartella dati MySQL che non parte su un altro sistema | Sempre 1 all'inizializzazione e in `my.cnf` |
| Download MySQL Linux ARM64 da 943 MB | Chiavetta piena, attesa | Dimensione mostrata prima; controllo dello spazio (D-90) |
| `.htaccess` ignorato da Caddy | Progetti con regole proprie rotti | Apache resta su Windows; rilevamento delle regole non note; messaggio nella pagina del sito |
| phpMyAdmin 5.2.3 dichiara `php <8.4` | phpMyAdmin rotto su PHP 8.5 di static-php-cli | Prova in T3.2.6; linea 8.4 per phpMyAdmin se serve |
| Firma Apple non disponibile (Q-2) | Avvio ostacolato da Gatekeeper | Firma ad hoc, rilevamento della traslocazione, istruzioni |
| WebKitGTK assente o diverso | Finestra che non si apre su Linux | Controllo all'avvio; AppImage con WebKitGTK |
| Secret Service assente | Accesso di Antigravity non salvato nel kit | File 600 con avviso |
| Processi orfani su macOS | Database che restano aperti dopo un crash | Watchdog di T1.5.2 |

## 9. Decisioni nuove da scrivere in DECISIONS.md

Numeri dopo D-107 (phpMyAdmin, in corso):

- **D-108** Disponibilità per target nel catalogo (`unavailable`, `removeUnix`, `sha256ByTarget` generico, Rosetta), niente ripieghi silenziosi. WP-0.4.
- **D-109** Terminale nell'app su macOS e Linux (PTY), console classica su Windows. Q-6, WP-1.2.
- **D-110** MySQL come secondo motore (`DbEngine`), con `lower_case_table_names=1` e l'accesso con `--defaults-extra-file`. Q-4, WP-3.6…WP-3.9.
- **D-111** Caddy come server web predefinito; Apache solo Windows; Nginx tolto. Q-3, WP-3.1…WP-3.5.
- **D-112** Fonti per Unix: dugite-native, static-php-cli, zig cc, conda-forge (Ruby, MongoDB su Linux, SQLite su Linux ARM64). Q-5, Fase 2, WP-3.2.
- **D-113** Pacchetto per macOS e Linux (`Dev-Kit.app`, AppImage, radice del kit, aggiornamento, firma). Q-2, Q-7, Fase 5.
- **D-114** Router multi-target e preparazione da un altro sistema (commit `36d2324`, oggi senza decisione). WP-6.3.

## 10. Dopo questo piano

- PostgreSQL (theseus-rs/postgresql-binaries: 5/6 target nativi, SHA-256 per file) come terzo motore di `DbEngine` (Q-8).
- FrankenPHP come opzione per i progetti Laravel Octane e Symfony Runtime.
- R su Windows ARM64: r-base win-64 in emulazione o R nativo sperimentale di CRAN.
- Verifica delle firme oltre agli hash: Sigstore (Caddy, Codex), GPG (Node, Bun, MySQL), `codesign --verify` su macOS.
- Terminale nell'app anche su Windows, per un'interfaccia sola (rivedrebbe D-57 e D-95).
