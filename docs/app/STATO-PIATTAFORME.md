# Atollo: stato del lavoro multipiattaforma e ripresa

Aggiornato il 17/10/2026, a fine giornata, quando l'utente ha messo in pausa il lavoro. Branch `feature/platforms`, anche su GitHub (il repository ora è pubblico).

Documenti di riferimento:

- [PLAN-PIATTAFORME.md](PLAN-PIATTAFORME.md): il piano, con scelte, disegno, pacchetti di lavoro (WP), prove, ondate di agenti e la sezione «Avanzamento» in testa;
- [MATRICE-PIATTAFORME.html](MATRICE-PIATTAFORME.html): stato di ogni componente sui 6 target, asset esatti, sostituti.

## 1. Dove siamo

| Fase | Contenuto | Stato |
| --- | --- | --- |
| 0 | Fondamenta comuni (percorsi, link simbolici, script, catalogo per target, lock, CI) | **fatta** |
| 1 | L'app gira su macOS e Linux (radice del kit, orfani, istanza unica, chiavetta, ambiente, terminale nell'app, editor, selettori) | **fatta**, WP-1.9 superato con le prove dell'utente su Mac Apple Silicon, Mac Intel e un PC Linux |
| 2 | Componenti da sistemare (Git, Ruby, C/C++, Rust ARM64, MongoDB/SQLite su Linux, piccole correzioni) | **in corso**: ondata 2 fermata a metà (§2) |
| 3 | Stack web e database (Caddy, PHP su Unix, MySQL al posto di MariaDB) | **iniziata**: WP-3.1 Caddy e WP-3.6 `DbEngine` scritti, da rivedere (§2) |
| 4 | Accessi portabili (Antigravity, Claude Code su macOS) | da fare |
| 5 | Pacchetto e distribuzione (`.app`, AppImage, aggiornamento, rilasci) | da fare |
| 6 | Interfaccia e documenti | da fare |

Numeri all'ultimo controllo:

- Windows: 559 test Rust e 125 Vitest superati; clippy pulito su Windows e, in modo incrociato, sui 4 target Unix.
- Linux (WSL2, test veri): 542 superati, 0 falliti, 16 ignorati.
- CI pubblica di GitHub: test superati su Windows x64, Linux x64, Linux ARM64, macOS Apple Silicon e macOS Intel; due test instabili (sincronizzazione su Linux, client lento su macOS) corretti in `c29a5c4`, da confermare con la prossima CI.
- Prima esecuzione reale su Linux: `prepare` installa e verifica pwsh, gh, Git, Node.js, Python, uv; la finestra si apre (WSLg) e l'editor si installa da solo.

## 2. Ondata 2: fermata a metà

Branch locali, **non su GitHub**, ognuno nel suo worktree in `.claude/worktrees/`. Nessuno è stato rivisto, e nessuno è entrato in `feature/platforms`.

| Branch | Worktree | Pacchetti | Stato |
| --- | --- | --- | --- |
| `plat/p5a` | `wf_0fad9d32-ea9-1` | WP-2.1 Git con dugite-native, WP-2.2 Ruby da conda-forge, WP-2.5 MongoDB e SQLite su Linux, Python predefinito (3.15 installato, i documenti dicono 3.13) | lavoro a metà in un commit `wip` (12 file) |
| `plat/p5b` | `wf_0fad9d32-ea9-2` | WP-2.3 `zig cc` su Unix, WP-2.4 Rust su Windows ARM64, WP-2.7 TinyTeX/Bun/.NET, variabili d'ambiente di Git e Ruby per P5a | lavoro a metà in un commit `wip` (11 file) |
| `plat/p6` | `wf_0fad9d32-ea9-3` | WP-3.1 componente Caddy | commit finito (`fe7e0f4`), da rivedere |
| `plat/p7` | `wf_0fad9d32-ea9-4` | WP-3.6 `DbEngine` senza cambiare comportamento | commit finito (`ac87ccf`), da rivedere |
| `plat/x` | `wf_0fad9d32-ea9-5` | aiuto della riga di comando su Unix e con `--target`; `data/home/AppData` creata anche su Linux | commit `9ec4636` più `wip`; il revisore ha trovato un problema medio da chiudere: dentro un AppImage `current_exe()` è il programma interno, quindi l'aiuto mostrerebbe un nome sbagliato |

Lo script dell'ondata (agenti, compiti, file di ciascuno, revisione) è in `C:\Users\Michel\.claude\projects\d--Projects-dev-kit2\562071cd-6be6-4746-b129-b40969f3aff4\workflows\scripts\platforms-wave-2-wf_0fad9d32-ea9.js`. I compiti di ogni agente sono anche nel piano (§5).

## 3. Come riprendere

1. Su questo PC: `git switch feature/platforms`, `git pull`, `git worktree list` (devono esserci i cinque worktree del §2).
2. Chiedere a Claude Code: «riprendi l'ondata 2 da docs/app/STATO-PIATTAFORME.md». Il coordinatore:
   - fa ripartire gli agenti **nei worktree esistenti**, dal commit `wip`, come è stato fatto per l'ondata 1 (stesso script con i percorsi dei worktree, senza worktree nuovi, con l'istruzione di trasformare il `wip` in commit veri);
   - rivede P6 e P7 (già finiti) con il revisore avversario;
   - fonde in un branch d'integrazione `plat/wave2`, esegue i test su Windows e Linux (WSL2), porta tutto in `feature/platforms`, fa push e segue la CI.
3. Prima dei test Linux: `npm -w ui run build` nel worktree (lo script di WSL non deve usare l'`npm` di Windows), poi da PowerShell nel worktree `wsl.exe -d Ubuntu -- bash scripts/test-linux.sh`.

Ambiente già pronto su questo PC: virtualizzazione attiva, WSL2 con Ubuntu 24.04, i pacchetti di Tauri installati dall'utente, Rust per l'utente di WSL. Controllo incrociato del codice Unix da Windows: `bash scripts/cross-check/xcheck.sh <target> clippy --all-targets -- -D warnings`.

## 4. Cosa manca, pacchetto per pacchetto

### Fase 2 (ondata 2 da completare)

- WP-2.1 Git rilocabile su macOS e Linux (dugite-native).
- WP-2.2 Ruby da conda-forge su macOS e Linux.
- WP-2.3 C e C++ con `zig cc` su macOS e Linux, `make` portabile, `CC`/`CXX` nell'ambiente, avviso per Xcode Command Line Tools.
- WP-2.4 Rust su Windows ARM64 con `gnullvm` e llvm-mingw.
- WP-2.5 MongoDB su Linux (conda-forge o tarball per distribuzione), SQLite su Linux ARM64.
- WP-2.7 TinyTeX con i nomi degli asset del 2026, Bun «baseline» senza AVX2, .NET senza libicu.

### Fase 3

- WP-3.1 Caddy: scritto (P6), da rivedere e fondere.
- WP-3.2 PHP su macOS e Linux con static-php-cli (cli + fpm, hash fissati nel catalogo); `php.ini` per sistema; togliere il limite «solo Windows» di PHP, Composer e phpMyAdmin. phpMyAdmin 5.2.3 dichiara PHP < 8.4: provare 8.4 e 8.5.
- WP-3.3 Caddy come motore dei pool e dei siti (`WebServerEngine::Caddy`, Caddyfile, porta admin per istanza, php-cgi su Windows e php-fpm su Unix).
- WP-3.4 Interfaccia dei server con Caddy; Apache solo su Windows.
- WP-3.5 Togliere Nginx, con la migrazione di `servers.json` e `panel.json`.
- WP-3.6 `DbEngine`: scritto (P7), da rivedere e fondere.
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
- `version` stampa «NEXKORA» mentre il prodotto è «Atollo» (rebrand incompleto, lavoro di un'altra sessione).
- `os::volume::startup_problem` non è più usata (al suo posto `disk_notice` in `main.rs`): toglierla o lasciarla.
- Decisione da prendere: su Windows la rimozione di pnpm non comprende corepack, su Unix sì.
- `router::probe::TargetToolStatus` senza il motivo dell'indisponibilità (serve a WP-6.1).
- CI: portare `actions/checkout` e `actions/setup-node` alla v5 (Node 20 deprecato); dal 19/10/2026 `ubuntu-latest` passa a Ubuntu 26: controllare i nomi dei pacchetti di Tauri.
- Con il repository pubblico: controllare in *Settings → Actions → General* che i workflow delle pull request da fork richiedano l'approvazione.

## 5. Scelte già prese (dal §3 del piano)

Caddy su tutti i sistemi (Apache solo su Windows, Nginx da togliere); **MySQL al posto di MariaDB ovunque**, con migrazione automatica; terminale nell'app su macOS e Linux; build rilocabili più conda-forge per Unix; Linux ARM64 come target di primo livello; firma ad hoc su macOS; integrazione su `feature/platforms`, ora anche su GitHub.
