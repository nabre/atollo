# Atollo: stato del lavoro multipiattaforma e ripresa

Aggiornato il 18/10/2026, dopo la fusione dell'ondata 2. Branch `feature/platforms`, anche su GitHub (il repository ora è pubblico).

Documenti di riferimento:

- [PLAN-PIATTAFORME.md](PLAN-PIATTAFORME.md): il piano, con scelte, disegno, pacchetti di lavoro (WP), prove, ondate di agenti e la sezione «Avanzamento» in testa;
- [MATRICE-PIATTAFORME.html](MATRICE-PIATTAFORME.html): stato di ogni componente sui 6 target, asset esatti, sostituti.

## 1. Dove siamo

| Fase | Contenuto | Stato |
| --- | --- | --- |
| 0 | Fondamenta comuni (percorsi, link simbolici, script, catalogo per target, lock, CI) | **fatta** |
| 1 | L'app gira su macOS e Linux (radice del kit, orfani, istanza unica, chiavetta, ambiente, terminale nell'app, editor, selettori) | **fatta**, WP-1.9 superato con le prove dell'utente su Mac Apple Silicon, Mac Intel e un PC Linux |
| 2 | Componenti da sistemare (Git, Ruby, C/C++, Rust ARM64, MongoDB/SQLite su Linux, piccole correzioni) | **fatta** con l'ondata 2 (§2), restano le dipendenze per target (§4) |
| 3 | Stack web e database (Caddy, PHP su Unix, MySQL al posto di MariaDB) | **iniziata**: WP-3.1 Caddy e WP-3.6 `DbEngine` fusi |
| 4 | Accessi portabili (Antigravity, Claude Code su macOS) | da fare |
| 5 | Pacchetto e distribuzione (`.app`, AppImage, aggiornamento, rilasci) | da fare |
| 6 | Interfaccia e documenti | da fare |

Numeri all'ultimo controllo (`plat/wave2`, poi `feature/platforms` a `09b8b01`):

- Windows: 613 test Rust superati, 0 falliti, 6 ignorati; clippy pulito su Windows e, in modo incrociato, sui 4 target Unix.
- Linux (WSL2, test veri): 599 superati, 0 falliti, 16 ignorati.
- CI pubblica di GitHub su `b6d508a`: verdi Linux x64, Linux ARM64, macOS Apple Silicon e Intel (i due test instabili di `c29a5c4` ora passano). Rosso solo `Test win32-arm64`, mai passato (§4, arretrato). CI su `09b8b01` (con l'ondata 2): lint, test e build verdi su Windows x64, Linux x64 e ARM64, macOS Apple Silicon e Intel; `Test win32-arm64` rosso con gli stessi due test (611 superati, 2 falliti).
- Installazioni reali in WSL (ondata 2): Git 2.53.0 (clone https funzionante), Python 3.15.0, Ruby 4.0.7, MongoDB 8.0.23, SQLite 3.54.1; Git preparato da Windows con `--target linux-x64` e completato su Linux.

## 2. Ondata 2: fatta

Ripresa il 17-18/10/2026 nei worktree esistenti (run `wf_d069a823-1da`, script `platforms-wave-2-resume-wf_d069a823-1da.js` nella cartella dei workflow della sessione `1dd8b20e-…`). Tutti e cinque i branch approvati dal revisore avversario, fusi in `plat/wave2` e portati in `feature/platforms`.

| Branch | Pacchetti | Revisione |
| --- | --- | --- |
| `plat/p5a` | WP-2.1 Git da dugite-native (preparato a parte quando l'host non può eseguire il target), WP-2.2 Ruby da conda-forge (solo linee 3.2+), WP-2.5 MongoDB e SQLite ARM64 da conda-forge, Database Tools per distribuzione | 2 giri: un problema alto (Git non preparato da Windows) corretto |
| `plat/p5b` | WP-2.3 `zig cc` su Unix, WP-2.4 Rust `gnullvm` su Windows ARM64, WP-2.7 TinyTeX/Bun/.NET, variabili di Git, Ruby (`GEM_HOME` per linea, `GEMRC`) | 2 giri: tre medi corretti; uno rimandato (dipendenze per target, §4) |
| `plat/p6` | WP-3.1 componente Caddy | approvato al primo giro |
| `plat/p7` | WP-3.6 `DbEngine` | approvato al primo giro |
| `plat/x` | aiuto della riga di comando (nome del programma, AppImage, `--target`) | approvato al primo giro |

Il coordinatore ha aggiunto `09b8b01`: `data/home/AppData` solo su Windows, `GEM_HOME` che segue la linea di Ruby del progetto, traduzione orfana di SQLite tolta. I worktree `wf_0fad9d32-ea9-*` e i branch `plat/p5a…x` non servono più e si possono togliere.

## 3. Come riprendere

1. Su questo PC: `git switch feature/platforms`, `git pull`.
2. Prossimo passo: ondata 3 (WP-3.2 PHP su Unix, WP-3.3-3.5 Caddy come motore e via Nginx, WP-3.7-3.9 MySQL), più le richieste dell'ondata 2 ancora aperte (§4). Lo schema degli script è quello dell'ondata 2.
3. Prima dei test Linux: `npm -w ui run build` nel worktree (lo script di WSL non deve usare l'`npm` di Windows), poi da PowerShell nel worktree `wsl.exe -d Ubuntu -- bash scripts/test-linux.sh`.

Ambiente già pronto su questo PC: virtualizzazione attiva, WSL2 con Ubuntu 24.04, i pacchetti di Tauri installati dall'utente, Rust per l'utente di WSL. Controllo incrociato del codice Unix da Windows: `bash scripts/cross-check/xcheck.sh <target> clippy --all-targets -- -D warnings`.

## 4. Cosa manca, pacchetto per pacchetto

### Fase 2 (resti dell'ondata 2)

- T2.3.1 e T2.4.2: dipendenze per target nel catalogo (`needsByTarget`: `cc` richiede `zig` su Unix, `rust` richiede `cc` su win32-arm64), da usare in `Catalog::locked`, `Choice::repair`/`set`, `runner.rs` e nell'interfaccia. Oggi `cc` senza Zig resta fuori dal PATH, senza danni.
- T2.3.4: avviso Xcode Command Line Tools nella pagina del componente `cc` su macOS (ora solo durante l'installazione, in inglese).
- Testi del catalogo: descrizione di `cc` (dice ancora LLVM su Unix) e di Ruby («RubyInstaller»), con le traduzioni.
- Facoltativo: per gli ambienti conda (Ruby, MongoDB, R) mettere nel PATH solo i programmi del componente, non tutto `bin` (oggi copre `openssl`, `tput` e altri).
- Database Tools di MongoDB scelti sulla distribuzione del PC che prepara il kit; Debian 9 non trova la build. MongoDB di conda-forge è indietro rispetto al fornitore.

### Fase 3

- WP-3.1 Caddy: fatto (il componente; il motore dei siti è WP-3.3).
- WP-3.2 PHP su macOS e Linux con static-php-cli (cli + fpm, hash fissati nel catalogo); `php.ini` per sistema; togliere il limite «solo Windows» di PHP, Composer e phpMyAdmin. phpMyAdmin 5.2.3 dichiara PHP < 8.4: provare 8.4 e 8.5.
- WP-3.3 Caddy come motore dei pool e dei siti (`WebServerEngine::Caddy`, Caddyfile, porta admin per istanza, php-cgi su Windows e php-fpm su Unix).
- WP-3.4 Interfaccia dei server con Caddy; Apache solo su Windows.
- WP-3.5 Togliere Nginx, con la migrazione di `servers.json` e `panel.json`.
- WP-3.6 `DbEngine`: fatto. Da usare poi anche in `detect/`, `install/mod.rs`, `setup/needs.rs` ed `env.rs`; con MySQL, `of_backup` non potrà più riconoscere il motore dalla sola estensione `.sql`.
- WP-3.7 Componente MySQL 8.4 / 9.7 LTS sui 6 target, con gli SHA-256 fissati dal coordinatore dopo la verifica GPG di Oracle; **prima** verificare come MySQL legge i file di opzioni (`MYSQL_HOME` non vale come per MariaDB).
- WP-3.8 Servizio MySQL: inizializzazione con `lower_case_table_names=1`, `my.cnf`, accesso con `--defaults-extra-file`, utenti `caching_sha2_password`, porte ereditate da MariaDB.
- WP-3.9 Interfaccia, rilevamento, framework e **migrazione automatica da MariaDB a MySQL** via dump (dati MariaDB conservati finché l'utente non li elimina).

### Fase 4

- WP-4.1 Accesso di Antigravity su macOS (Portachiavi) e Linux (Secret Service).
- WP-4.2 Claude Code su macOS: token OAuth salvato nel kit.

### Fase 5

- WP-5.1 Configurazione dei bundle (`.icns`, `Info.plist`, entitlement).
- WP-5.2 macOS: firma ad hoc e istruzioni (scelta Q-2: niente Developer ID per ora).
- WP-5.3 AppImage per Linux x64 e ARM64, più un tar.gz.
- WP-5.4 Aggiornamento dell'app su macOS e Linux.
- WP-5.5 Rilasci per tutti i target.

### Fase 6

- WP-6.1 Griglia componente × target nella pagina «Strumenti».
- WP-6.2 Matrici delle relazioni e delle estensioni per target.
- WP-6.3 Documenti: README (limiti, sistemi supportati, tracce fuori dal kit su macOS), PLAN e GUIDA («solo Windows» da togliere), DECISIONS (D-108…D-114 e le note proposte dagli agenti), CATALOG (campi nuovi del catalogo, fonti per target), CHANGELOG, TESTING (checklist §6 del piano). Le righe proposte dagli agenti delle ondate 0-1 sono riassunte nella sezione «Avanzamento» del piano.

### Arretrato fuori dai pacchetti

- 16 test ignorati su Linux: `agents::tests::sign_in_detection`, `editor::terminals` (2), `env` (4), `os::temp` (2), `os::sessions`, `install::editor::processes_of_the_old_editor`. Quest'ultimo nasconde un difetto vero: `os::processes` confronta i percorsi senza distinzione di maiuscole anche su Linux.
- CI `Test win32-arm64`, mai passato: falliscono sempre `editor::terminal::tests::real_console_runs_and_ends` e `setup::checkup::tests::check_program_at_the_kit_root`. Ipotesi da provare: i due test avviano `cmd.exe` con l'ambiente vuoto (`KitEnv::default()`), senza `SystemRoot`.
- Documenti da allineare: CATALOG.md dice Python 3.13 predefinito, ma il kit prende l'ultima linea stabile (D-81), oggi 3.15. Le righe di CHANGELOG, DECISIONS e TESTING proposte dagli agenti dell'ondata 2 sono nel risultato del run `wf_d069a823-1da`.
- `version` stampa «NEXKORA» mentre il prodotto è «Atollo» (`main.rs`, comando `version`; anche la descrizione dell'editor nel catalogo). Rebrand incompleto, lavoro di un'altra sessione.
- `os::volume::startup_problem` non è più usata (al suo posto `disk_notice` in `main.rs`): toglierla o lasciarla.
- Decisione da prendere: su Windows la rimozione di pnpm non comprende corepack, su Unix sì.
- `router::probe::TargetToolStatus` senza il motivo dell'indisponibilità (serve a WP-6.1).
- CI: portare `actions/checkout` e `actions/setup-node` alla v5 (Node 20 deprecato); dal 19/10/2026 `ubuntu-latest` passa a Ubuntu 26: controllare i nomi dei pacchetti di Tauri.
- Con il repository pubblico: controllare in *Settings → Actions → General* che i workflow delle pull request da fork richiedano l'approvazione.

## 5. Scelte già prese (dal §3 del piano)

Caddy su tutti i sistemi (Apache solo su Windows, Nginx da togliere); **MySQL al posto di MariaDB ovunque**, con migrazione automatica; terminale nell'app su macOS e Linux; build rilocabili più conda-forge per Unix; Linux ARM64 come target di primo livello; firma ad hoc su macOS; integrazione su `feature/platforms`, ora anche su GitHub.
