# Prove

Le prove automatiche girano nella CI (`.github/workflows/app.yml`) e in locale, dalla radice del repository:

```
npm run lint
npm test
npm -w ui run build
```

`npm run lint` controlla `cargo fmt`, `cargo clippy -D warnings`, ESLint e Prettier (anche sui test, con i tipi di `test\ui\tsconfig.json`); `npm test` esegue `cargo test` e i test dell'interfaccia (Vitest); `npm -w ui run build` controlla i tipi (`tsc -b`) e compila l'interfaccia.

Tutte le prove stanno in `test\`, con la struttura dei sorgenti:
- `test\rust\<percorso>.rs`: le prove di `tauri\src\<percorso>.rs`. Restano un modulo figlio di quel file, che le dichiara in fondo con `#[cfg(test)] #[path = "…/test/rust/<percorso>.rs"] mod tests;`: vedono le sue funzioni private (`use super::*`), come prima. Una prova nuova va nel file di `test\rust` del modulo; un modulo che non ne ha ancora aggiunge le tre righe.
- `test\ui\<percorso>.test.ts(x)`: le prove di `ui\src\<percorso>`, che importano da `../../ui/src/…`; `test\ui\setup.ts` prepara jsdom (Vitest, `ui\vite.config.ts`).

Le prove dal vivo sulle fonti delle versioni (rete, ignorate di solito): `cargo test live_lines -- --ignored --nocapture` (con `GITHUB_TOKEN`), vedi CATALOG.md §2.

Coprono, fra l'altro:
- radice del kit, ambiente e PATH;
- file del kit (`fsx.rs`): BOM, byte non UTF-8, file mancante, vuoto o danneggiato, scrittura atomica senza file temporanei rimasti;
- errore dei comandi (`commands\error.rs`): Tauri manda per `CmdError` lo stesso valore di una stringa, `or_fail` manda il testo breve e `or_fail_chain` tutta la catena;
- catalogo, dipendenze, parti (`keys`), limiti ARM64 e componenti che servono a comandi e agenti (`setup\needs.rs`);
- righe di versione lette dalle fonti di ogni linguaggio (Go, Java, .NET, Zig: prove in `test\rust\install\lang\`);
- editor alla versione fissata del rilascio;
- download con checksum e firme;
- editor e server web reale (con il token e senza);
- finestre dell'editor che non possono chiamare i comandi dell'app;
- una console reale in `conhost.exe`, in una cartella con spazi;
- allineamento degli agenti e accesso di Antigravity con Gestione credenziali reale;
- rilevamento dei progetti su esempi (Node, Next, Tauri, Electron, PHP, Laravel, Django, FastAPI, Flask, Streamlit, Rust, statico, progetti composti, `panel.json` e `pannello.json`);
- porte ricordate e bloccate, avvio e arresto di un'azione attraverso il suo job;
- server di file: host, file nascosti, `..`, collegamenti;
- scambio dell'exe in aggiornamento;
- DPAPI di `keyring-before.json`;
- cataloghi delle quattro lingue (Rust e interfaccia), completi e con gli stessi segnaposto;
- `useLoad` (`test\ui\hooks.test.ts`): prima lettura, le tre politiche d'errore, nuova lettura quando cambiano `load` o `key` ma non la politica, `enabled`, lettura periodica fermata allo smontaggio, `setData`;
- controlli dell'interfaccia su `vscode-elements` (`test\ui\workbench\controls.test.tsx`, `elements.test.tsx`): eventi che scattano davvero, valori controllati, Invio nei moduli, nomi accessibili, codicon; ogni variabile `--vscode-*` letta dai componenti definita nei due temi (`test\ui\theme.test.ts`).

## Linux, macOS e Windows ARM64

**CI** (`.github/workflows/app.yml`, WP-0.6). Su ogni push di `main` e `feature/platforms` e su ogni pull request:

| Job | Runner | Cosa |
| --- | --- | --- |
| `Lint and test` | `windows-latest` | lint, Vitest, build dell'interfaccia, `cargo fmt`, `clippy`, `cargo test` |
| `Test linux-x64` | `ubuntu-latest` | librerie di Tauri, build e Vitest dell'interfaccia, `clippy --all-targets -D warnings`, `cargo test` |
| `Test linux-arm64` | `ubuntu-24.04-arm` | come sopra, senza Vitest |
| `Test darwin-arm64` | `macos-latest` | build dell'interfaccia, `clippy`, `cargo test` |
| `Test darwin-x64` | `macos-15-intel` | come sopra (`macos-13` non c'è più) |
| `Test win32-arm64` | `windows-11-arm` | build dell'interfaccia, `cargo test` nativo |
| `Build …` | `windows-latest` | exe x64 e arm64, prove reali (x64) |
| `Bundle darwin-*` | `macos-latest`, `macos-15-intel` | `Atollo.app` con firma ad hoc verificata, `Info.plist`, zip, avvio di prova (D-113) |
| `Bundle linux-*` | `ubuntu-24.04`, `ubuntu-24.04-arm`, contenitore `ubuntu:22.04` | AppImage e `tar.gz`, minimo di glibc con `objdump`, `--version` e `check` sotto `xvfb-run` |

**Build di Apache** (`.github/workflows/httpd.yml`, D-116), solo sulle pull request che toccano `scripts/build-httpd/**`, a mano (`gh workflow run httpd.yml --ref <branch>`) e sul tag `httpd-v<versione>-<build>`, che pubblica il rilascio: `httpd <target>` per i quattro target Unix (Linux nel contenitore `ubuntu:22.04`), `build.sh` e poi `test.sh`, che avvia Apache da una cartella con spazio e parentesi e dopo averla spostata, controlla i moduli che il kit carica, il minimo di glibc e le librerie (Linux), il minimo macOS 12 e le firme (macOS). In locale per Linux x64: `docker run ubuntu:22.04` con i pacchetti del job, `WORK=/w bash scripts/build-httpd/build.sh linux-x64 0 /out`, poi `test.sh /out/httpd-*.tar.gz`.

Ogni job di test scrive nel riepilogo della run le prove passate, fallite e ignorate, con i nomi di quelle fallite. Su macOS gira tutto `cargo test`: con tauri 2.12.1 il feature `test` (il runtime finto di `test\rust\main.rs`) compila anche lì, quindi non serve un feature a parte (con 2.12.0 mancava `transparent`). In un repository senza runner ARM64 (privato) la variabile del repository `ARM64_RUNNERS=off` toglie i job `linux-arm64` e `win32-arm64`, che altrimenti resterebbero in coda: `linux-arm64` esce dalla lista dei target della matrice (il runner si ricava dal target, senza `include`, che GitHub applica dopo `exclude` e rimetterebbe il target tolto). Il job `win32-arm64` installa la toolchain `stable-aarch64-pc-windows-msvc` e fallisce se `rustc -vV` non riporta `host: aarch64-pc-windows-msvc`: le prove non girano mai in emulazione x64 senza dirlo.

Una prova che vale solo su Windows (percorsi con `\`, `cmd.exe`, `ping.exe`, junction con `mklink /J`, layout `.exe` del kit) porta `#[cfg_attr(not(windows), ignore = "<motivo>")]`: su Windows gira, altrove conta fra le ignorate con il motivo. Una prova nuova usa invece, quando può, funzioni pure con il target come parametro (`Kit::at_target`, `fsx::join_rel` con `/`), così vale per tutti e sei i target.

**WSL2 in locale.** `scripts/test-linux.sh` copia il repository nel file system di Linux (una copia per checkout o worktree in `~/atollo-copies/<cartella>-<hash>`, o `ATOLLO_LINUX_DIR`; due esecuzioni sulla stessa copia si danno il turno con `flock`; con `rsync`, senza `target*`, `node_modules`, `.claude/worktrees`, `.git` e le cartelle del kit `data` e `projects`), controlla Rust e le librerie di Tauri (lo stato `install ok installed` di `dpkg-query`), poi esegue `cargo test` e stampa il riepilogo:

```
wsl -d Ubuntu -- bash scripts/test-linux.sh            # da Windows, nella radice del repository
bash scripts/test-linux.sh                             # dentro WSL o su Linux
bash scripts/test-linux.sh --clippy -- servers::       # anche clippy; filtro e opzioni dopo --
```

Se manca qualcosa stampa il comando da eseguire (`sudo apt-get install -y …`, o `rustup` per l'utente) ed esce con 2; con prove fallite esce con 1. `target/` resta nella copia, quindi le esecuzioni dopo la prima compilano solo ciò che è cambiato. La copia usa `rsync --delete`: lo script copia solo in una cartella vuota o in una copia fatta da lui (file `.atollo-linux-copy`); altrimenti esce con 2 senza toccare nulla. Anche un clone del repository senza quel file (con il suo `.git` e lavoro non salvato) viene rifiutato: scegliere un'altra cartella con `ATOLLO_LINUX_DIR`. `bash scripts/test-linux.sh --check-dest` dice soltanto se la cartella va bene (0 sì, 2 no) e gira ovunque, anche in Git Bash. `cargo` gira dentro la copia, così valgono i suoi `.cargo/config.toml` e `rust-toolchain.toml`, non quelli sopra la cartella di Windows. L'interfaccia compilata (`dist\`) viene dalla copia di Windows (`npm -w ui run build`); senza, lo script la compila in Linux con Node.js. WSL2 richiede la virtualizzazione attiva nel BIOS e la «Piattaforma macchina virtuale» di Windows (`wsl.exe --install --no-distribution`, da amministratore). Prima dello script, in un worktree, `npm -w ui run build` da Windows: dentro WSL l'`npm` trovato può essere quello di Windows.

**Controllo incrociato da Windows.** `bash scripts/cross-check/xcheck.sh <target> clippy --all-targets -- -D warnings` (target `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin`) compila il codice Unix con strumenti finti, senza linker: trova gli errori di tipo e di `cfg`, non quelli di link (un `_EMBED_INFO_PLIST` doppio su macOS l'ha visto solo la CI).

**CI su un altro branch.** I push di `main` e `feature/platforms` avviano la CI da soli; per un altro branch: `git push -u origin <branch>`, poi `gh workflow run app.yml --ref <branch>`.

**Prove con programmi veri** (`ATOLLO_TEST_KIT=<kit con i componenti>`, ignorate senza):
- `real_databases_start_and_stop`: MySQL e MongoDB del kit su porte libere: avvio, database e utente di un progetto (accesso TCP come quell'utente), copia e ripristino, eliminazione, arresto senza resti (D-110); su Windows e in WSL;
- `caddy_pool_two_php_sites_alias_and_reload`: un pool Caddy con due siti PHP e un alias, `caddy reload` dopo l'aggiunta di un sito senza fermare l'altro (D-111);
- `apache_pool_two_php_sites_alias_and_reload` (con `start_reports_syntax_error` e `apache_pool_reload_twice`): un pool Apache con due siti PHP (modulo su Windows, `php-fpm` su macOS e Linux), un alias, `.env` e host estranei 403, uno script e un `.htaccess` ostile nella cartella dell'alias neutralizzati, la cartella `files` di Drupal senza PHP, un terzo sito dopo la ricarica (D-116, SEC-V4-01); su Windows e in WSL. In WSL, finché la build non è fissata nel catalogo, Apache si mette a mano: l'archivio in `data/apps/linux-x64/apache/2.4/`, `httpd.conf` riscritto come fa `httpd_conf`, `data/app/versions/linux-x64/apache.txt`;
- `a_new_kit_needs_mysql_for_every_sql_need` (`db::services::data`, senza kit): ogni bisogno SQL del catalogo e dei framework è MySQL; `a_new_kit_installs_mysql_for_phpmyadmin_wordpress_and_laravel` (`commands::frameworks`): phpMyAdmin, WordPress e Laravel installano MySQL, sui sei target.
- `real_offline_reinstall_after_a_move` (`install::conda`, D-118, macOS e Linux): con gli ambienti conda installati in `ATOLLO_TEST_KIT` da questa versione dell'app (`Atollo prepare --only r,ruby,mongodb,sqlite,cc`, che conserva archivi ed elenchi), la cartella del kit si rinomina accanto (`<nome>-moved-d118`; senza spazi: il `bin/R` di conda-forge scrive `SED=` senza virgolette e si rompe in ogni cartella con uno spazio), `Kit::prepare` segna i componenti, una preparazione con un proxy che non esiste li ricrea dagli archivi conservati; ogni programma risponde dalla cartella nuova (`R RHOME` sotto di essa), gem e dati di MongoDB invariati; poi lo stesso all'indietro. Per provarla davvero senza rete si esegue il binario dei test sotto `unshare -rn` (in WSL funziona senza root):

  ```
  cargo test --no-run   # stampa il percorso del binario dei test
  ATOLLO_TEST_KIT=~/d118/kit unshare -rn target/debug/deps/Atollo-<hash> real_offline_reinstall_after_a_move --nocapture
  ```

**Secret Service in WSL senza root** (D-115). Per provare Antigravity con un Secret Service vero:

```
mkdir -p ~/gk && cd ~/gk
apt-get download gnome-keyring libgcr-base-3-1 libgck-1-0 libsecret-tools
for f in *.deb; do dpkg -x "$f" root; done
export LD_LIBRARY_PATH=~/gk/root/usr/lib/x86_64-linux-gnu PATH=~/gk/root/usr/bin:$PATH
dbus-run-session -- sh -c 'echo | gnome-keyring-daemon --unlock --components=secrets; cargo test keyring'
```

Con `keyring.json` che contiene `gemini:antigravity` e un nome di Windows (`jetski:other`), dopo `restore` e `save` il Secret Service è vuoto e `keyring.json` ha tutte e due le voci. Senza Secret Service la copia in chiaro si scrive solo sul disco di Linux (600), mai su `/mnt/<lettera>`.

**Kit condiviso fra Windows e WSL.** PHP 8.5 installato da Windows e da WSL nella stessa cartella del kit, nei due ordini: `data/php/8.5/php.ini` con le 12 righe `extension=` e `data/php/linux-x64/8.5/php.ini` senza nessuna; `php -v` su Linux senza «Unable to load». phpMyAdmin 5.2.3 sul PHP 8.5.8 di static-php-cli: `index.php` 200, nessun avviso con `E_ALL`.

**Preparazione da Windows** (D-114). `Atollo.exe prepare --target linux-x64 --only git` dice `(bootstrap linux-x64)`; sul target il primo avvio, o `prepare --only git`, scrive «bootstrap pre-downloaded» e `git clone https://…` funziona. In WSL: `prepare --only cc` senza Zig non mette `CC`/`CXX` né `cc/bin` nel PATH; `prepare --only zig,cc` dà `CC=…/cc/bin/cc` e un hello in C si compila con `make` ed esegue.

**Bundle Linux in locale.** `docker run ubuntu:22.04` con i pacchetti del job `Bundle linux-*`, `tauri build --bundles appimage`, minimo di glibc con `objdump -T`, poi `--version` e `check` sotto `xvfb-run` per l'AppImage (`--appimage-extract-and-run`) e per il `tar.gz`.

## Prove sull'interfaccia reale

Per le prove senza mani sull'exe di release (fase 6) si usa un kit di prova fuori dal repository, con tutti i componenti, in una cartella con spazi:
- **Avvio fuori dalla sessione dello strumento:** l'app si avvia con WMI (`Win32_Process.Create`). Lo strumento chiude i processi che lancia direttamente, quindi l'app non sopravviverebbe al comando.
- **Clic:** con UI Automation, sui nomi accessibili dei pulsanti. Gli stessi nomi servono anche all'Assistente vocale.
- **Schermate:** con `PrintWindow` sulla finestra «Atollo».
- **Controllo dei processi:** dopo «Ferma», «Chiudi tutto» e «Rimuovi l'unità», nessun processo deve avere l'eseguibile nel kit di prova (`Win32_Process.ExecutablePath`).
- **Arresto delle prove:** i processi di prova si fermano **per PID**, mai per nome. Un `taskkill /IM Atollo.exe` chiude anche le altre copie dell'app aperte sul PC (successo durante la fase 6).

## Checklist manuale

Rispetto alla checklist del prompt iniziale (i requisiti sono in ARCHITECTURE.md, «Requisiti»):
- la voce sulla rete usa la pagina Impostazioni al posto di `data\proxy.cmd` (D-54);
- la voce sul menu `setup\Windows.cmd` non si applica, perché questo è un prodotto separato, senza gli script del kit (Q1 del piano).

Da eseguire prima di ogni release, con l'exe di release (`npm run build`), su un PC che non ha mai visto il kit, quando la voce lo richiede.

### Installazione e portabilità
- [ ] **Kit nuovo su chiavetta.** Doppio clic, primo avvio: PowerShell 7, GitHub CLI (con Git) e l'editor si installano da soli; poi installazione dei predefiniti, editor aperto su un progetto.
- [ ] **Editor alla versione del rilascio.** Con un kit preparato da un'altra versione dell'app (o con un numero diverso in `data\app\versions\vscodium-server.txt`), all'avvio l'editor viene installato di nuovo alla versione fissata, anche se era rimasto aperto, senza «Aggiorna tutto». Se un programma del PC tiene aperta la cartella dell'editor, l'aggiornamento finisce «in uso», ma Editor, Terminale e gli agenti si aprono lo stesso con l'editor già presente. In Linguaggi e strumenti l'editor è tra gli obbligatori, spuntato e non modificabile.
- [ ] **Nessuna traccia fuori dal kit.** Dopo una sessione completa (editor, terminale, un agente, un progetto avviato) non c'è nessun file nuovo in `%APPDATA%`, `%LOCALAPPDATA%`, `%TEMP%` e nel registro. Confronto prima/dopo con Process Monitor o con un'istantanea delle cartelle. Eccezioni documentate: le credenziali di Antigravity, che restano in Windows solo per la durata di `agy` e con persistenza di sessione (D-41); i file di passaggio in `%TEMP%\Atollo\<id>\`, che a fine sessione sono spariti (D-90).
- [ ] **Kit spostato.** Su un'altra lettera di unità e in una cartella con spazi tutto funziona, lo stato dell'editor resta e Claude ritrova la cronologia.
- [ ] **PC senza WebView2** (Windows 10 LTSC o macchina virtuale). Il messaggio è chiaro e l'app funziona con il runtime fisso nel kit.
- [ ] **Utente standard** senza diritti di amministratore: nessuna richiesta UAC in nessun passo.
- [ ] **Proxy autenticato e ispezione SSL.** Impostazioni › Rete con proxy e file PEM: installazione, estensioni dell'editor e agenti funzionano.
- [ ] **Senza rete.** L'app e l'editor partono, gli errori di download si leggono, il controllo degli aggiornamenti fallisce senza bloccare nulla.
- [ ] **Chiusura forzata dal Task Manager.** Non resta nessun `node.exe`, server dell'editor, console del kit (`conhost.exe`, `pwsh.exe`) o server di progetto.
- [ ] **ARM64.** L'exe è nativo, l'editor gira in emulazione e l'avviso si vede.
- [ ] **R spostato** (D-104). Kit con R su `D:`, poi su `E:`: dopo l'avvio `lib\R\etc\x64\Makeconf` ha `LOCAL_SOFT` sulla cartella nuova e `install.packages(..., type="source")` compila.
- [ ] **macOS e Linux**: la sezione «macOS, Linux e Windows ARM64» qui sotto.

### Progetti
- [ ] **Nuovo progetto** «prova 1»: la cartella compare, ha il pulsante «Elimina la cartella vuota» e l'eliminazione funziona.
- [ ] **Clona** `nabre/atollo` (o un repository privato con `gh auth login` fatto nel terminale del kit): la cartella ha il nome giusto. Su un kit senza Git, Git si installa prima del clone. Con un indirizzo non valido (`-x/y`, `ssh://…`) l'errore si legge.
- [ ] **Progetto Vite.** «Avvia», poi la porta 7100-7999 compare nell'output. Lo stato passa a «in ascolto» e il pulsante con l'indirizzo apre il browser del PC. Un secondo avvio riusa la stessa porta. Su un kit senza Node.js, «Avvia» installa prima Node.js (avanzamento in Home e nella barra di stato) e poi parte; Node.js risulta scelto in Linguaggi e strumenti.
- [ ] **Rete locale.** Con «Permetti ai server dei progetti di farsi raggiungere dagli altri PC della rete», un altro PC apre il progetto avviato con l'indirizzo IP di questo PC; senza, no.
- [ ] **Porta occupata.** Scelta a mano una porta già usata da un altro programma, l'avvio dà un errore chiaro e la porta non cambia da sola.
- [ ] **Sito statico.** Una cartella con solo `index.html`: il server di file del kit lo serve. Un file `.env` non è raggiungibile (403).
- [ ] **«Servi come sito web».** Con la cartella `public` e PHP: `panel.json` contiene l'azione `site` e le altre chiavi del file restano.
- [ ] **«Ferma»** su `npm run dev`: in Gestione attività non resta nessun `node.exe`/`esbuild.exe` del progetto.
- [ ] **Electron o Tauri.** L'app apre la sua finestra e l'output arriva nella pagina.

### Server web e database (sciame v3, D-96…D-102)
- [ ] **Configurazioni valide.** Con un progetto PHP avviato su Apache e su Nginx dedicati, in un pool e con il sito del kit: `httpd -t -f <conf>` e `nginx -t -c <conf>` sulle configurazioni scritte dal kit non danno errori. Un `kit.conf` di un kit precedente, dopo l'avvio, ha i blocchi nuovi una volta sola.
- [ ] **Estensioni PHP** (D-96). Con PHP: `a.phtml` si esegue; `a.inc`, `a.php5` e `.env` danno 403. Senza PHP (Apache senza modulo, Nginx senza php-cgi) `index.php` dà 403/503, mai il sorgente.
- [ ] **`php -S`** (D-97). Progetto PHP semplice avviato con «Avvia»: `/.env`, `/a.inc`, `/index.php.` danno 403; `curl -H "Host: evil.com"` dà 421; `/` e un file statico 200. Con un `router.php` nel progetto vale il suo.
- [ ] **Host estraneo sul server dedicato** (D-98). Senza «Rete locale»: `curl -H "Host: evil.com" http://127.0.0.1:<porta>/` dà 403 (Apache) o 421 (Nginx); `localhost`, `127.0.0.1`, `<progetto>.localhost` e i domini del progetto rispondono. Con «Rete locale» l'IP del PC da un altro PC apre il sito.
- [ ] **php-cgi che muore** (D-99). Progetto PHP su Nginx dedicato avviato: chiudere `php-cgi.exe` dal Task Manager ferma anche `nginx.exe`; l'esecuzione va in errore e il log ha «php-cgi terminato».
- [ ] **Pool che seguono i progetti** (D-101). Due progetti assegnati allo stesso profilo: «Avvia» sul primo avvia il pool, sul secondo lo ricarica; «Ferma» sul primo lo toglie (il suo indirizzo non risponde più) e il secondo resta servito. Con «Mantieni attivo» spento, fermare anche il secondo spegne il pool; acceso, il pool resta. Un pool avviato dalla pagina Server Web resta acceso comunque.
- [ ] **Avvia con Atollo** (D-101). Profilo con «Avvia con Atollo»: chiusa e riaperta l'app, il pool è acceso senza nessun progetto avviato. Un `servers.json` di prima (senza i due campi) si legge con «Mantieni attivo» spuntato.
- [ ] **Due azioni web di un progetto nello stesso pool** (BUG-V3-02). `http://<progetto>.localhost:<porta>` e `http://<azione>.<progetto>.localhost:<porta>` servono le due cartelle.
- [ ] **Due versioni di MySQL insieme** (D-102). Con 8.4 e 9.7 installati: Database › «Altre versioni» avvia 8.4 su una porta sua mentre 9.7 resta sulla 3306; `mysql -uroot -h127.0.0.1 -P<porta> -e "select version()"` dà le due versioni. Progetto con «Versione del server» 8.4: «Crea» crea database e utente su 8.4; un programma del progetto vede `DB_PORT` di 8.4; con un `.env` che ha `DB_PORT=3310` vale quello. Cambiare la versione principale mentre 8.4 gira è rifiutato; chiusa l'app, nessun `mysqld` resta.
- [ ] **Prove con kit reale.** `ATOLLO_TEST_KIT=<kit con Apache e PHP> cargo test -- start_reports_syntax_error apache_pool_reload_twice apache_pool_two_php_sites_alias_and_reload --test-threads=1` passa.

### macOS, Linux e Windows ARM64 (D-105, D-108…D-115)
Su macOS Apple Silicon e Intel, Linux x64 e ARM64 (un PC o una VM con desktop: WSL2 non ha chiavette né `noexec` veri), con il bundle della CI o `tauri build --debug --no-bundle`.

Primo avvio:
- [ ] **Kit su disco interno e su chiavetta exFAT.** Avvio, `projects/` e `data/` accanto a `Atollo.app` o all'AppImage, niente dentro il bundle. Un `.app` scaricato con il browser e non spostato (App Translocation) ferma l'avvio con il comando `xattr`.
- [ ] **Seconda apertura** porta in primo piano la prima.
- [ ] **Componenti obbligatori** (PowerShell 7, GitHub CLI, Git, editor) installati e verificati; `Atollo check` verde.
- [ ] **Progetti.** Nuovo progetto vuoto e clone di un repository privato.
- [ ] **Editor**: apertura, terminale integrato con l'ambiente del kit (`echo $CP_DATA`), estensioni installate.
- [ ] **Terminale del kit e Claude Code** da un progetto: si aprono in una scheda dell'app, rispondono ai tasti, chiudendo la scheda non resta nessun processo.
- [ ] **Chiusura dell'app** con un terminale e un database aperti: nessun processo resta. `kill -9` dell'app: nessun processo del kit entro 6 s (`pgrep -f <kit>`).
- [ ] **Kit spostato** in un'altra cartella: tutto riparte, Git senza reinstallare.
- [ ] **Kit spostato senza rete** (D-118), con R, Ruby e (su Linux) MongoDB installati: al riavvio offline si ricreano dai pacchetti conservati («created again from the kept packages» nel pannello Attività), `R --version`, `ruby -v`, `mongod --version` rispondono; Impostazioni → Generale mostra «Copia per reinstallare senza rete: N MB»; dopo «Libera» lo spostamento successivo chiede la rete.
- [ ] **Chiavetta montata `noexec`** (Linux): messaggio con il comando per rimontare, nessuna installazione.
- [ ] **«Rimuovi l'unità»**: chiude tutto e lascia espellere.

Componenti:
- [ ] Ogni componente disponibile per il sistema: installazione, `check`, rimozione senza resti. Android su Linux ARM64 si mostra non disponibile con il motivo.
- [ ] **C/C++** su Linux senza `build-essential` e su macOS senza Xcode CLT: `cc hello.c && ./a.out`, `cargo build` di un crate con `cc-rs`. Su macOS senza CLT la pagina del componente avvisa per i framework Apple.
- [ ] **Rust su Windows 11 ARM64** senza Visual Studio: `cargo new x && cargo run`.
- [ ] **Ruby**: `ruby -v`, `gem install json` (nativa); dopo uno spostamento del kit i comandi delle gem partono.

Stack web e database:
- [ ] **Laravel con MySQL** su macOS e Linux: `composer create-project`, migrazioni, sito su Caddy, phpMyAdmin.
- [ ] **WordPress con Caddy**: permalink funzionanti senza `.htaccess`. Drupal propone Apache dove il catalogo lo offre (Windows; macOS e Linux dopo il primo rilascio della build, D-116).
- [ ] **Apache su macOS e Linux** (D-116). Un progetto Laravel con un `.htaccess` proprio, sito Apache: `php-fpm` parte con Apache (`data/projects/<p>/apache/fpm.conf`), le rotte rispondono, `httpd` gira con `-DFOREGROUND`; fermando il sito si fermano entrambi.
- [ ] **Pool Caddy** con due siti e un alias verso un altro progetto; aggiungere un sito ricarica il pool senza fermare l'altro.
- [ ] **Stessa cartella dati MySQL** letta da Windows e da Linux con la stessa linea.
- [ ] **MySQL su Ubuntu 24.04** senza `libaio1` (solo `libaio1t64`): il server parte; senza `libnuma1` l'errore dice `sudo apt install libaio1t64 libnuma1`.

Accessi e pacchetto:
- [ ] **Antigravity e Claude Code** collegati su un Mac (Claude con «Collega con un token»), kit portato su un altro Mac: già collegati; nel Portachiavi del primo non resta nulla.
- [ ] **Antigravity su Linux** con un Secret Service bloccato: nessuna voce spostata, avviso; senza Secret Service e kit su exFAT: rifiutato.
- [ ] **Rilasci**: download con il browser, apertura su un sistema pulito, aggiornamento dell'app (il `tar.gz` dice che la versione nuova va scaricata a mano). Senza FUSE: `./Atollo.AppImage --appimage-extract-and-run`.

Kit preparato su Windows per un altro sistema (a ogni rilascio):
- [ ] Su Windows `Atollo.exe prepare --target linux-x64` (poi `darwin-arm64`, `linux-arm64`) su una chiavetta exFAT, con l'app per quel sistema copiata accanto.
- [ ] Il selettore «Piattaforma» mostra per quel target cosa è pre-scaricato, cosa si completa al primo avvio e cosa non è disponibile.
- [ ] Sull'altro sistema: primo avvio dalla chiavetta, installazioni da `.bootstrap` completate (Rust, Git di dugite-native, R e i componenti conda, Android, pnpm), nessun nuovo download per ciò che era già scaricato.
- [ ] Di nuovo su Windows: il kit si apre come prima; `data/apps/win32-x64` e le impostazioni di Windows sono intatti, la scelta dei componenti non è cambiata.
- [ ] Stesso progetto aperto sui due sistemi: impostazioni, porte e database del progetto condivisi (`data/projects/<p>`).

### Dati per progetto e file di passaggio (D-89, D-90)
- [ ] **Migrazione da un kit della versione 1.** Con la copia di un kit reale (dati in `data\app\projects`, `ports.json`, `secrets.json`, `data\db\backup`): al primo avvio compaiono `data\projects\<progetto>\`, `data\app\layout.txt` vale `2`, `data\app\pre-v2\` ha le due copie. Progetti, porte (anche quelle bloccate), password e copie dei database intatti; `.env` dei progetti ancora validi; un secondo avvio non cambia nulla.
- [ ] **Chiavetta exFAT.** Installazione di Node, Python e Java: l'avanzamento non dà errori di spazio o di rename fra dischi; «Temporanei dei programmi: Automatico» usa il sistema; dopo la chiusura dell'app non resta nessun file in `data\tmp` e `%TEMP%\Atollo\<id>\system` non esiste più.
- [ ] **Chiavetta FAT32.** Un componente con un file oltre 4 GiB è rifiutato con un errore che dice quale file, la versione già installata resta intatta e non resta `.atollo-new`.
- [ ] **Rust** (`695c177`).
  - Kit NTFS senza Rust: installazione, `cargo --version` da `rust\cargo\bin`.
  - Kit NTFS con Rust: aggiornamento sul posto («rustup update» nel registro), uno strumento di `cargo install` resta.
  - Kit FAT32/exFAT senza Rust: nessun errore sui collegamenti fisici, `rustup-init` nel temp e copia sulla chiavetta; `cargo`, `rustc` e `rustup show` funzionano dalle copie dei proxy.
  - Kit FAT32/exFAT con Rust: aggiornamento («rustup update (copy in the temp folder)»), strumenti di `cargo install` presenti, nessun `rust.atollo-new` rimasto.
  - `TEMP` su un disco FAT/exFAT: errore chiaro sul temp senza collegamenti fisici, kit intatto.
- [ ] **Eliminazione di un progetto con MySQL e MongoDB.** Server fermo: la conferma elenca cartella, motori e copie; dopo «Elimina» i server si avviano, database e utente sono tolti (`SHOW DATABASES`, `SHOW GRANTS`) e `data\projects\<progetto>\` non esiste. Con un server che non parte non si cancella nulla; «Elimina senza i database» cancella le cartelle e il log dice quali database restano.
- [ ] **Ripristino di una copia fatta altrove.** Una copia `.sql` ottenuta con `mysqldump --databases altro` è rifiutata dal server (accesso negato) e il database `altro` resta intatto; una copia del database del progetto si ripristina.
- [ ] **Nomi simili.** Progetto `my-app` (database `my_app`) e progetto `my2app`: l'utente `my_app` non vede `my2app` (`SHOW GRANTS`). Un progetto chiamato `mysql` o `admin` non può cancellare né cambiare i database dei server.
- [ ] **Progetto rinominato in Esplora file.** Compare in «Dati senza progetto» nella pagina Progetti; «Collega a…» lo riporta con password e copie, e la `.env` continua a funzionare. Un orfano con database si elimina con la stessa conferma.
- [ ] **C: quasi pieno.** Le installazioni ripiegano su `data\tmp` e il log ha l'avviso.
- [ ] **Alias dei siti.** Un sito con `/media` → `storage/uploads` e `/shared` → `@common/assets`, con Apache, con Nginx e in un pool: i file si aprono nel browser.
- [ ] **Alias e giunzione.** `storage` sostituito da una giunzione verso `C:\Users`: l'alias non è servito e la pagina del sito lo dice.
- [ ] **Impostazioni › Temporanei dei programmi.** La pagina mostra file system e tipo di disco del kit; Automatico, Nel kit e Nel sistema cambiano `$env:TEMP` in una console nuova del kit (`data\tmp\system` o `%TEMP%\Atollo\<id>\system`).
- [ ] **Alias e `.htaccess`** (D-100, SEC-V4-01). In una cartella di alias servita da Apache, un collegamento simbolico non si apre; un `.htaccess` con `Options +FollowSymLinks`, `RewriteRule`, `AddHandler` o `Require all granted` non cambia nulla (avviso nel log di Apache, la cartella resta servita); un `.php` o `.phtml` lì dà 403 con Apache e Nginx, 404 con Caddy.
- [ ] **File scaricati a mano** (D-82). Un file messo a mano nel riquadro finisce in `%TEMP%\Atollo\<id>\downloads`; uno lasciato in `data\downloads` si usa lo stesso e si sposta.

### GitHub
- [ ] **Accesso.** GitHub › «Accedi a GitHub»: si apre github.com/login/device nel browser del PC e la pagina mostra il codice. Dopo l'autorizzazione compare «Accesso fatto come @…» senza ricaricare, `data\gh\hosts.yml` contiene l'account e nella Gestione credenziali di Windows non c'è nessuna voce `gh:github.com`. «Annulla» durante l'attesa non lascia nessun `gh.exe`.
- [ ] **Un solo accesso.** Dopo l'accesso, `git config --global --get-all credential.https://github.com.helper` in una console del kit dà una riga vuota e `!gh auth git-credential`. Push di un repository privato dall'editor (Controllo del codice sorgente) e da una console senza nessuna finestra di Git Credential Manager. Claude Code usa `gh` con lo stesso account. Dopo «Esci» il helper sparisce. Dopo aver spostato il kit su un'altra lettera di unità, il push funziona ancora.
- [ ] **Repository.** Elenco con i repository propri e delle organizzazioni (gruppi separati), ricerca, «Clona» su un repository privato: la cartella è in `projects\` e `.git\config` non contiene il token. Con il nome già preso la cartella diventa `<proprietario>-<nome>`. Sul repository clonato c'è «Editor».
- [ ] **Rinomina ed elimina** su un repository di prova: dopo «Rinomina» il remote del clone punta al nuovo nome. «Elimina da GitHub» chiede conferma; la prima volta offre «Concedi il permesso», poi con un nuovo accesso l'eliminazione riesce e il clone resta in `projects\`.

### Agenti
- [ ] **Claude Code, Codex e Antigravity.** Accesso, avvio dall'app e dall'editor, allineamento di istruzioni e skill fra i tre.
- [ ] **Console di un agente.** Home o Progetti › Agente con Claude Code: si apre la console classica di Windows, anche se il terminale predefinito del PC è Windows Terminal o un altro. Claude Code riceve Invio, frecce e Ctrl+C. Chiudendo la console non resta nessun `claude.exe`.
- [ ] **Git con GitHub CLI.** Su un kit senza Git, all'avvio Git si installa con i componenti di sistema; in «Linguaggi e strumenti» è spuntato, non modificabile, con l'etichetta «per GitHub CLI».
- [ ] **Editor aggiornato con una cartella aperta.** Con Esplora file aperto su `data\apps\win32-x64\vscodium-server\extensions`, l'editor di un'altra versione viene comunque sostituito all'avvio (D-60).

- [ ] **Limiti d'uso.** Con l'accesso fatto in Claude Code e Codex, Agenti mostra gli stessi valori di `/usage` e `/status`, con l'ora in cui si azzerano. Senza accesso compare «accesso non fatto». Senza rete l'errore si legge e la pagina resta usabile. «Aggiorna i consumi» premuto più volte di fila non fa più di una richiesta ogni 20 s.
- [ ] **Solo nel progetto.** In Home e Progetti, Editor, Terminale e agente si aprono nella cartella del progetto; non c'è nessun pulsante per `projects\`. Nell'editor, File › Apri cartella su un'altra cartella, «Nuova finestra» e «Chiudi cartella» non escono dal progetto.
- [ ] **Fiducia nell'editor.** Alla prima apertura di un progetto l'editor chiede se fidarsi. Con «No» compare la barra «Restricted Mode» e le attività non partono; con «Sì» non chiede più alla riapertura, anche dopo aver riavviato l'app (D-67).
- [ ] **dev-settings.json.** Dopo aver aperto Codex in un progetto, `data\projects\<progetto>\dev-settings.json` ha `"agent": "codex"` e la scelta dell'agente propone Codex. Eliminando dall'app un progetto vuoto la sua cartella in `data\projects\` sparisce (D-89).

### Linguaggi e strumenti
- [ ] **Loghi.** Ogni componente ha il suo logo, leggibile nel tema chiaro e in quello scuro; i progetti mostrano le etichette dei linguaggi.
- [ ] **Dipendenze.** Apache mostra la dipendenza «PHP»; con Claude Code scelto, Git mostra «per Claude Code». Togliendo PHP si toglie anche Apache.
- [ ] **PATH dopo un'installazione.** Dopo l'installazione di un linguaggio da questa pagina, Home › Terminale apre la console del kit con PowerShell 7 del kit e il programma nuovo è nel PATH, senza riavviare l'app.

### Impostazioni
- [ ] **Lingua.** Passando a English, Français e Deutsch, tutta l'interfaccia cambia subito. Con «Lingua di Windows» vale quella del sistema.
- [ ] **Chiave di Gemini.** Salvata, il campo mostra «salvata». Non compare mai in chiaro (né nell'interfaccia, né in `data\app\logs`). «Togli la chiave» la rimuove.
- [ ] **Valore non valido** (proxy `proxy:8080`, Python `3.x`): l'errore si legge e il file non cambia.
- [ ] **Editor in esecuzione.** Dopo il salvataggio l'app chiede di riavviare l'editor, e dopo il riavvio i terminali dell'editor vedono le variabili nuove.
- [ ] **Aggiornamento.** Con una release `app-v*` più recente pubblicata, «Cerca aggiornamenti» scarica l'exe. Alla chiusura `Atollo.exe` è quello nuovo e `Atollo.old.exe` sparisce al riavvio. Con `ATOLLO_NO_UPDATE=1` all'avvio non parte nessun controllo.

### Chiusura, altre copie, rimozione dell'unità
- [ ] **Chiusura con qualcosa aperto.** Con editor, console o un progetto avviato, chiudere la finestra chiede conferma ed elenca cosa si chiude. «Annulla» (anche con Esc) lascia tutto aperto.
- [ ] **Due copie del kit aperte.** La Home della seconda mostra l'avviso con la cartella dell'altra.
- [ ] **Rimuovi l'unità.** Tutto si chiude e l'accesso di Antigravity torna nel kit. Dopo «Chiudi Atollo» la chiavetta si espelle senza «dispositivo in uso».

### Barra del titolo
- [ ] **Finestra principale senza cornice di Windows.** Trascinamento dalla barra, doppio clic per ingrandire, Riduci, Ingrandisci/Ripristina, Chiudi (con la conferma se qualcosa è aperto); ridimensionamento dai bordi; Win+frecce funzionano. I pulsanti si raggiungono con Tab e hanno un nome per l'Assistente vocale.
- [ ] **Database** (D-73). Pagina Database: «Avvia» su MySQL non installato lo installa e lo avvia; `mysql -uroot -h127.0.0.1` e `mongosh` rispondono; porta occupata: messaggio e cambio porta; «Avvia con Atollo» lo fa partire al riavvio dell'app; chiusura dell'app: la conferma conta i database e i log finiscono con «Shutdown complete» / «Now exiting». Un progetto Laravel con `DB_CONNECTION=mysql` riceve SQLTools.
- [ ] **Contenuto dei database** (D-79). Pagina Database con MongoDB avviato (MySQL si vede in phpMyAdmin, D-85): «Contenuto» mostra il database di un progetto e le sue tabelle; una tabella con più di 100 righe va avanti e indietro; colonne BLOB come «n byte binari». MongoDB: collezioni e documenti con `ObjectId(…)`. Server fermo: «Avvia il server per vederne il contenuto.». SQLite non è nella lista.
- [ ] **Password dei database** (D-74). Pagina Database: password di root/admin nascosta, «Mostra» e «Copia». `mysql --no-defaults -uroot -h127.0.0.1` rifiutato; `mysql` nel terminale del kit entra. Pagina del progetto › Database: «Crea» e poi «Scrivi in .env» mostra prima le righe che cambiano.
- [ ] **Impostazioni del progetto** (D-78). Avanzate: «Nuova azione» con una variabile `NOME=valore` compare in tabella e in `panel.json`; «Predefinita» la mette sul pulsante Avvia; «Togli» la toglie. Editor: formatta al salvataggio e rientro scritti in `.vscode\settings.json`, aperti nell'editor valgono. MCP: un server aggiunto compare in `.mcp.json` e in `.codex\config.toml`; `codex mcp list` nel progetto lo elenca. Risorse › database: «Esporta» crea la copia, «Ripristina…» chiede conferma e riporta i dati.
- [ ] **Struttura a sei pagine** (D-76). Progetti è la prima pagina; «Nuovo progetto» con le tre strade (il clone dai repository chiede l'accesso a GitHub nelle Impostazioni); pagina del progetto con Riepilogo, Risorse, Avanzate; pannello Attività: avviare un progetto, aprire il pannello, vedere l'output, Apri nel browser, Ferma; installazione in corso nel pannello; database con errore (porta occupata) nel pannello.
- [ ] **Pagina del progetto** (D-75). Dal nome in Progetti e nella Home; le sei schede si scelgono anche con le frecce; Avvio avvia e ferma come prima; Database crea il database e scrive il `.env` dopo l'anteprima; Git mostra ramo e remoto di un progetto clonato da GitHub.
- [ ] **Branding** (D-72). App: logo nella barra del titolo e nella Home, accento verde Atollo in tema scuro e chiaro, testo dei pulsanti leggibile. Editor: titolo «<progetto> - Atollo», logo nella barra, benvenuto con logo e progetto nell'editor vuoto, anche in tema chiaro.
- [ ] **Estensioni mentre si lavora** (D-71). Editor aperto su un progetto senza Ruby: si crea `app.rb`; entro circa 20 s (più il download) compare l'avviso con Ruby LSP e Ruby Debugger; «Riavvia le estensioni» ricarica l'editor sul progetto, «Più tardi» chiude l'avviso.
- [ ] **Finestra dell'editor con la barra di VSCodium** (D-69). Menu File…Help nella barra; trascinamento dalle zone vuote, doppio clic per ingrandire, Aero Snap; Riduci, Ingrandisci/Ripristina (l'icona cambia anche con Win+frecce), Chiudi; ridimensionamento dai bordi; tema chiaro e scuro. Con il server dell'editor fermo compare la barra di ripiego, che sposta e chiude la finestra.

### Accessibilità
- [ ] **Solo tastiera.** Tutte le pagine si usano: Tab mostra sempre il contorno di focus, «Vai al contenuto» è il primo elemento, il dialogo di chiusura riceve il focus.
- [ ] **Conferme nella pagina** (D-106). Uscita da un agente, da GitHub, ripristino di un database e «Applica» in Linguaggi e strumenti: l'Assistente vocale legge la domanda, il focus è su «Annulla», Esc chiude e il focus torna al pulsante di partenza. Lo zoom di una schermata della Guida tiene Tab dentro e si chiude con Esc.
- [ ] **Assistente vocale.** Le pagine hanno titoli e le icone hanno un nome. Errori e stati sono annunciati (`role="alert"` e `role="status"`).
- [ ] **Stesso aspetto dell'editor** (D-88). App ed editor affiancati, in tema scuro e in chiaro: stessi colori di sfondo, testo, focus, pulsanti, campi e menu a tendina; select, campi e caselle si usano con mouse e tastiera; le icone codicon si vedono (nessun quadratino).
- [ ] **Tema e animazioni.** Il tema chiaro e quello scuro seguono Windows. Con «Effetti di animazione» spenti in Windows, nessuna icona gira.
