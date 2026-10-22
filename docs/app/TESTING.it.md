# Prove

[English](TESTING.md) · Italiano

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

Filtri: `cargo test ports::` o `cargo test db::services` (solo le prove Rust il cui percorso corrisponde); `npm -w ui test -- activity` (solo i file di prova dell'interfaccia il cui nome corrisponde).

Le prove dal vivo sulle fonti delle versioni (rete, ignorate di solito): `cargo test live_lines -- --ignored --nocapture` (con `GITHUB_TOKEN`), vedi CATALOG.it.md §2.

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
- una sessione reale del terminale sul ConPTY di Windows (`editor/conpty.rs`): solo l'ambiente del kit, una cartella con spazi, il codice di uscita, i tasti battuti e la chiusura che ferma l'albero della sessione;
- allineamento degli agenti e accesso di Antigravity con Gestione credenziali reale;
- rilevamento dei progetti su esempi (Node, Next, Tauri, Electron, PHP, Laravel, Django, FastAPI, Flask, Streamlit, Rust, statico, progetti composti, `panel.json` e `pannello.json`);
- porte ricordate e bloccate, avvio e arresto di un'azione attraverso il suo job;
- il registro delle porte (`test\rust\ports.rs`, D-141): ogni proprietario letto dal suo file; una porta di un pool, di una versione di un database, di un'azione o dell'editor rifiutata agli altri, con il proprietario nominato (`servers::`, `db::services::data`, `projects::run`), e mai data a un pool nuovo;
- server di database (D-140): le operazioni su un server si danno il turno (`operations_on_a_server_take_turns`), `keep_port` non perde voci, le porte restano quando il server cambia versione, i file di login sono diversi per ogni esecuzione;
- il `conf.d` del progetto in un pool Apache e Caddy e nel Caddy proprio del progetto (`apache_pool_includes_the_project_conf_d`, `caddy_pool_imports_the_project_conf_d`, `project_conf_d_imported_in_the_site`);
- server di file: host (`*.localhost`, un IP solo con la rete locale), file nascosti, `..`, collegamenti;
- scambio dell'exe in aggiornamento;
- DPAPI di `keyring-before.json`;
- cataloghi delle quattro lingue (Rust e interfaccia), completi e con gli stessi segnaposto;
- `useLoad` (`test\ui\hooks.test.ts`): prima lettura, le tre politiche d'errore, nuova lettura quando cambiano `load` o `key` ma non la politica, `enabled`, lettura periodica fermata allo smontaggio, `setData`;
- controlli dell'interfaccia su `vscode-elements` (`test\ui\workbench\controls.test.tsx`, `elements.test.tsx`): eventi che scattano davvero, valori controllati, Invio nei moduli, nomi accessibili; i codicon (`codicon.test.tsx`): glifi del font incluso, misure dai token, nessun `react-icons/vsc` rimasto;
- il tema (`test\ui\theme\theme.test.ts`): ogni variabile `--vscode-*` letta dai componenti definita nei due temi, ogni misura dichiarata una volta con la sua classe, le misure di `docs\brand\vscodium-1.135\workbench-ref.css`, nessun token della UI classica, ogni chiave `--vscode-*` letta da Atollo nota a VSCodium 1.135;
- il pannello Attività (`test\ui\activity.test.tsx`): una versione secondaria di un database e un pool elencati e contati, «Ferma tutto» che ferma ogni cosa dopo la conferma; la pagina Terminali (`test\ui\terminals.test.tsx`): Esc chiude la domanda di rimozione;
- la palette dei comandi (`test\ui\commands.test.tsx`, D-160): i comandi di ogni progetto, degli agenti e dei server di database installati e delle altre pagine, «Ferma tutto» solo mentre qualcosa è in corso, un progetto trovato con due parole, ogni parola scritta che deve comparire in qualsiasi ordine, maiuscole o minuscole, un errore mostrato dopo la chiusura; `test\ui\App.test.tsx`: Ctrl+Shift+P, «vai impostazioni», Invio apre le Impostazioni;
- i componenti con una versione nuova (`test\rust\setup\updates.rs`, D-161): versioni confrontate per numeri, l'indirizzo delle note con `{version}` o il sito, elencati solo i componenti installati indietro rispetto alla fonte; `real_check_of_a_kit` (saltata senza `ATOLLO_TEST_KIT`) chiede alle fonti vere del kit che indica; `test\ui\components\updates.test.tsx`: le voci del riquadro, «Cosa cambia», «Aggiorna», «Controlla ora», «Tutto aggiornato» o «Non ancora controllato», l'elenco riletto alla fine di una preparazione;
- «Le mie basi» (`test\rust\projects\frameworks\mine.rs`, D-162): gli id dai nomi, i file senza Git (niente `vendor`, `node_modules`, `.env`, `.env.local`; `.env.example` tenuto), il salvataggio con una cartella tolta, il nuovo salvataggio con lo stesso nome (sostituita), la cartella tolta con la base, un `template` con `..` rifiutato; `real_git_files` (saltata senza `ATOLLO_TEST_KIT`) elenca con il Git vero del kit, `.gitignore` rispettato; `test\ui\projects\savebase.test.tsx`: il totale togliendo una cartella, `project_save_base` con `skip`, il messaggio finale, «Le mie basi» primo gruppo;
- i documenti della Guida: `node scripts/guide-docs.cjs --check` (nel job `Lint and test`) fallisce se `docs\app\GUIDE.md` o `GUIDE.it.md` è rimasto indietro rispetto alla Guida dell'app.

## File generati e schermate

Nessuno di questi gira in `npm test`; ognuno si esegue a mano dopo la modifica che lo richiede, e la sua uscita va nel commit.

| Comando | Quando | Scrive |
| --- | --- | --- |
| `python docs/brand/themes.py` | è cambiato un colore (`BRAND`) o una misura (`LAYOUT`) | `ui\src\theme\theme-vscode.css`, `theme-colors.css`, `theme-sizes.css`, `tauri\editor-theme\` |
| `node scripts/vscodium-css/extract.mjs [archivio o workbench.css]` | il catalogo fissa un altro editor, o l'app legge una chiave `--vscode-*` nuova | `docs\brand\vscodium-1.135\workbench-ref.css` (poi `themes.py`); fallisce su una chiave che VSCodium non conosce |
| `node scripts/guide-docs.cjs` | è cambiato il testo della Guida | `docs\app\GUIDE.md`, `GUIDE.it.md` |
| `python -I scripts/matrix/gen_matrix.py` | è cambiato il catalogo | `docs\app\TOOLS-MATRIX.html` |
| `node site/shots/make.mjs` | sono cambiate le pagine delle schermate | `site\shots\<lingua>\shot-{projects,agents,tools}-{dark,light}.png` per en, it, fr, de |
| `node site/shots/social.mjs` | dopo `make.mjs` | `docs\brand\social.png` (da `en\shot-projects-dark.png`) |
| `node scripts/guide-shots/make.mjs` | sono cambiate le pagine della Guida | `ui\public\guide\<lingua>\*.webp` (52 file; Python con Pillow) |

Gli script delle schermate leggono l'anteprima nel browser: prima `npm -w ui run dev` (`127.0.0.1:5173`), con `playwright-core` installato accanto allo script o in una cartella sua ed Edge o Chrome del sistema (`SHOTS_BROWSER=msedge|chrome`, nessun download). `SHOTS_LANGS=en,it` (sito) e `GUIDE_SHOTS_LANGS=en,it` (Guida) limitano le lingue; `SHOTS_URL=http://127.0.0.1:5199` prende l'anteprima da un altro indirizzo, per tutti e due. Prima del commit si guarda ogni lingua: le etichette vengono da `ui\src\locales`, i testi del motore da `engineText`.

`npm run dev` esegue `scripts/dev-ui.mjs`: chiudendo l'app la porta 5173 deve liberarsi (D-138); con la porta occupata dice chi la tiene e non ferma nulla.

## Linux, macOS e Windows ARM64

**CI** (`.github/workflows/app.yml`, WP-0.6). Su ogni push di `main` e `feature/platforms` e su ogni pull request, solo i job che la modifica richiede (D-134). Il job `Changes` assegna a ogni file cambiato i livelli dei job che possono vederlo rompersi. L'elenco è nel passo `Levels`, e ogni livello comprende quelli sotto:

- **bundle**: il pacchetto e il codice che avvia l'app (`tauri/Cargo.toml`, `build.rs`, `tauri.conf.json`, `Info.plist`, `entitlements.plist`, `windows-app.manifest`, `icons/`; in `tauri/src`: `main.rs`, `cli.rs`, `kit.rs`, `log.rs`, `env.rs`, `settings.rs`, `fsx.rs`, `i18n.rs`, `os/`, `setup/`). Partono `Bundle *` e `Release smoke *`, più tutto quello sotto.
- **kit**: ciò che `prepare` e le prove `real_` usano su un kit vero (`tauri/src/install/`, `net/`, `editor/`, `runtime/`, `tauri/catalog/components.json`). Partono `Build …` con l'exe, l'installazione reale e le prove reali, più tutto quello sotto.
- **rust**: il resto del codice e dei dati dell'app (`tauri/src` negli altri moduli, gli altri cataloghi, `i18n/`, `editor-theme/`, `capabilities/`, `permissions/`, `site/assets/fonts/`) e i test Rust (`test/rust/`, `scripts/matrix/sources.json`, `scripts/test-linux.sh`, `rustfmt.toml`). Partono `Lint and test`, `Test *` e `Test win32-arm64` con `clippy` e `cargo test` su ogni sistema.
- **ui**: l'interfaccia (`ui/`, `test/ui/`, `eslint.config.mjs`, `.prettierrc.json`, anche `components.json`, che i test dell'interfaccia importano, e `docs/app/GUIDE.md`, `GUIDE.it.md` e `scripts/guide-docs.cjs`, che `--check` confronta). Partono lint, Vitest, i documenti della Guida e build in `Lint and test` (più `cargo fmt`) e il solo Vitest in `Test linux-x64`.

`Cargo.toml`, `Cargo.lock`, `package*.json`, `.gitattributes` e `app.yml` avviano tutto. Non avviano altro che `Changes` i file generati (`tauri/gen/`), i documenti, il sito, gli script di Apache e PHP (con i loro workflow) e `scripts/cross-check/`. Un percorso che l'elenco non conosce avvia tutto, e il riepilogo lo nomina: va aggiunto all'elenco. Il giro completo parte sempre anche con un tag `app-v*`, con un avvio a mano (`gh workflow run app.yml`) e quando il confronto non riesce (per esempio dopo un force push). Il riepilogo del job `Changes` mostra i livelli scelti.

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
| `Release smoke *` | `macos-latest`, `macos-15-intel`, `ubuntu-24.04`, `ubuntu-24.04-arm` (senza contenitore) | gli artefatti `bundle-<target>` che il rilascio pubblica: nomi di `self_update.rs` e `.sha256`; macOS: quarantena messa e tolta con `xattr -dr`, firma, `--version`, `check`, kit accanto al `.app`; Linux: AppImage col nome del rilascio con `--appimage-extract-and-run`, `tar.gz` con il solo `libwebkit2gtk-4.1-0` (`ldd` senza «not found»), `--version` e `check` sotto `xvfb-run` (D-119) |
| `Release` | `windows-latest` | solo sui tag `app-v*`, dopo `Build …` e `Release smoke *`: exe, `.app.zip`, AppImage e `tar.gz` con i `.sha256` ricontrollati, note del rilascio |

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
- `real_databases_start_and_stop`: MySQL e MongoDB del kit su porte libere: avvio, database e utente di un progetto (accesso TCP come quell'utente), copia e ripristino (con la copia di sicurezza prima, D-154), eliminazione, arresto senza resti (D-110); su Windows e in WSL;
- `real_mongo_express_on_running_mongodb` (`db::mongo_express`, D-147; anche `ATOLLO_TEST_MONGODB_PORT`, la porta del MongoDB del kit già avviato, con `node` e `mongo-express` nel kit): 403 senza gettone, con uno sbagliato e per un altro Host; il gettone dà un cookie HttpOnly e SameSite=Strict ed esce dall'indirizzo; i database elencati come admin; lo stesso server se riaperto su quella versione; fermato con lei. L'archivio stesso lo prova `scripts/build-mongo-express/test.sh` in CI, su MongoDB 7.0 e 8.0;
- `caddy_pool_two_php_sites_alias_and_reload`: un pool Caddy con due siti PHP e un alias, `caddy reload` dopo l'aggiunta di un sito senza fermare l'altro (D-111);
- `apache_pool_two_php_sites_alias_and_reload` (con `start_reports_syntax_error` e `apache_pool_reload_twice`): un pool Apache con due siti PHP (modulo su Windows, `php-fpm` su macOS e Linux), un alias, `.env` e host estranei 403, uno script e un `.htaccess` ostile nella cartella dell'alias neutralizzati, la cartella `files` di Drupal senza PHP, un terzo sito dopo la ricarica (D-116, SEC-V4-01); su Windows e in WSL. In WSL, finché la build non è fissata nel catalogo, Apache si mette a mano: l'archivio in `data/apps/linux-x64/apache/2.4/`, `httpd.conf` riscritto come fa `httpd_conf`, `data/app/versions/linux-x64/apache.txt`;
- `mysql_is_the_kits_sql_server` (`db::services::data`, senza kit): un kit nuovo mostra MySQL e MongoDB, ogni versione di MySQL installata ha la sua porta e i programmi di un progetto ricevono l'accesso della sua versione; `a_new_kit_installs_mysql_for_phpmyadmin_wordpress_and_laravel` (`commands::frameworks`): phpMyAdmin, WordPress e Laravel installano MySQL, sui sei target.
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

La collezione `login` (D-115, issue #6) con la prova ignorata `real_secret_service_login_collection`, due volte, ognuna con un `XDG_DATA_HOME` vuoto: con `echo | gnome-keyring-daemon --unlock --components=secrets` (c'è `login`) la voce di `agy` va in `/org/freedesktop/secrets/collection/login`, si rilegge e si cancella; con `gnome-keyring-daemon --start --components=secrets` (nessun portachiavi `login`) la scrittura fallisce con `NO_LOGIN_COLLECTION` e non resta nulla.

```
XDG_DATA_HOME=$(mktemp -d) dbus-run-session -- sh -c 'echo | gnome-keyring-daemon --unlock --components=secrets; cargo test real_secret_service_login_collection -- --ignored --nocapture'
```

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

Rispetto alla checklist del prompt iniziale (i requisiti sono in ARCHITECTURE.it.md, «Requisiti»):
- la voce sulla rete usa la pagina Impostazioni al posto di `data\proxy.cmd` (D-54);
- la voce sul menu `setup\Windows.cmd` non si applica, perché questo è un prodotto separato, senza gli script del kit (Q1 del piano).

Da eseguire prima di ogni release, con l'exe di release (`npm run build`), su un PC che non ha mai visto il kit, quando la voce lo richiede.

### Installazione e portabilità
- [ ] **Kit nuovo su chiavetta.** Doppio clic, primo avvio: PowerShell 7, GitHub CLI (con Git) e l'editor si installano da soli; poi installazione dei predefiniti, editor aperto su un progetto.
- [ ] **Editor alla versione del rilascio.** Con un kit preparato da un'altra versione dell'app (o con un numero diverso in `data\app\versions\vscodium-server.txt`), all'avvio l'editor viene installato di nuovo alla versione fissata, anche se era rimasto aperto, senza «Aggiorna tutto». Se un programma del PC tiene aperta la cartella dell'editor, l'aggiornamento finisce «in uso», ma Editor, Terminale e gli agenti si aprono lo stesso con l'editor già presente. In Strumenti l'editor è tra gli obbligatori, spuntato e non modificabile.
- [ ] **Nessuna traccia fuori dal kit.** Dopo una sessione completa (editor, terminale, un agente, un progetto avviato) non c'è nessun file nuovo in `%APPDATA%`, `%LOCALAPPDATA%`, `%TEMP%` e nel registro. Confronto prima/dopo con Process Monitor o con un'istantanea delle cartelle. Eccezioni documentate: le credenziali di Antigravity, che restano in Windows solo per la durata di `agy` e con persistenza di sessione (D-41); i file di passaggio in `%TEMP%\Atollo\<id>\`, che a fine sessione sono spariti (D-90).
- [ ] **Kit spostato.** Su un'altra lettera di unità e in una cartella con spazi tutto funziona, lo stato dell'editor resta e Claude ritrova la cronologia.
- [ ] **PC senza WebView2** (Windows 10 LTSC o macchina virtuale). Il messaggio è chiaro e l'app funziona con il runtime fisso nel kit.
- [ ] **Utente standard** senza diritti di amministratore: nessuna richiesta UAC in nessun passo.
- [ ] **Proxy autenticato e ispezione SSL.** Impostazioni › Rete con proxy e file PEM: installazione, estensioni dell'editor e agenti funzionano.
- [ ] **Senza rete.** L'app e l'editor partono, gli errori di download si leggono, il controllo degli aggiornamenti fallisce senza bloccare nulla.
- [ ] **Chiusura forzata dal Task Manager.** Non resta nessun `node.exe`, server dell'editor, terminale del kit (`OpenConsole.exe`, `pwsh.exe`) o server di progetto.
- [ ] **ARM64.** L'exe è nativo, l'editor gira in emulazione e l'avviso si vede.
- [ ] **R spostato** (D-104). Kit con R su `D:`, poi su `E:`: dopo l'avvio `lib\R\etc\x64\Makeconf` ha `LOCAL_SOFT` sulla cartella nuova e `install.packages(..., type="source")` compila.
- [ ] **macOS e Linux**: la sezione «macOS, Linux e Windows ARM64» qui sotto.

### Progetti
- [ ] **Nuovo progetto** «prova 1»: la cartella compare, ha il pulsante «Elimina la cartella vuota» e l'eliminazione funziona.
- [ ] **Clona** `nabre/atollo` (o un repository privato con `gh auth login` fatto nel terminale del kit): la cartella ha il nome giusto. Su un kit senza Git, Git si installa prima del clone. Con un indirizzo non valido (`-x/y`, `ssh://…`) l'errore si legge.
- [ ] **Progetto Vite.** «Avvia», poi la porta 7100-7999 compare nell'output. Lo stato passa a «in ascolto» e il pulsante con l'indirizzo apre il browser del PC. Un secondo avvio riusa la stessa porta. Su un kit senza Node.js, «Avvia» installa prima Node.js (avanzamento nel pannello Attività e nella barra di stato) e poi parte; Node.js risulta scelto in Strumenti.
- [ ] **Rete locale.** Con «Permetti ai server dei progetti di farsi raggiungere dagli altri PC della rete», un altro PC apre il progetto avviato con l'indirizzo IP di questo PC; senza, no. Lo stesso per un progetto PHP semplice avviato con `php -S`: `http://<IP del PC>:<porta>/` risponde da un altro dispositivo, `curl -H "Host: evil.com"` riceve ancora 421.
- [ ] **QR del sito per il telefono** (D-158). Con «Rete locale» accesa, avvia un progetto Vite (dopo averla accesa): pagina del progetto › il dettaglio dell'azione ha «Dal telefono» sotto «Indirizzo», con l'IP privato di questo PC e la porta (lo stesso che `ipconfig` mostra per la scheda Wi-Fi o Ethernet). «Copia» lo mette negli appunti; «Mostra il QR» mostra un QR nero su bianco (anche con il tema scuro) e il suggerimento; inquadrandolo con la fotocamera di un telefono sullo stesso Wi-Fi si apre il sito. «Nascondi il QR» lo nasconde. Al primo avvio del sito il firewall di Windows chiede: basta consentire le reti private.
- [ ] **Nessuna riga del QR.** Con «Rete locale» spenta la riga non c'è. Un progetto in un pool di server web, con un indirizzo come `http://shop.localhost:<porta>`, non ha la riga nemmeno con «Rete locale» accesa. Un'azione ferma non ha la riga.
- [ ] **Porta occupata.** Scelta a mano una porta già usata da un altro programma, l'avvio dà un errore chiaro e la porta non cambia da sola.
- [ ] **Porta di un'altra parte di Atollo** (D-141). La porta di un'azione scelta a mano che è quella di un pool di server web, di una versione di un database (per esempio 3306) o nella 7000-7099 dell'editor: rifiutata, e il messaggio nomina il proprietario («La porta 3306 è già del database MySQL 9.7: scegline un'altra.»). Lo stesso per la porta di un profilo di pool e per la porta di un server di database o di una sua versione; un pool nuovo non ne riceve mai una.
- [ ] **Sito statico.** Una cartella con solo `index.html`: il server di file del kit lo serve, a `http://localhost:<porta>` e a `http://<progetto>.localhost:<porta>`. Un file `.env` non è raggiungibile (403).
- [ ] **Attività e «Ferma tutto».** Con un'azione, una versione secondaria di MySQL e un pool in esecuzione: la barra di stato, il badge di Progetti e il pannello Attività contano tutti e tre e il pannello li elenca; «Ferma tutto» chiede prima, «Annulla» non cambia nulla, la conferma li ferma tutti e tre.
- [ ] **«Servi come sito web».** Con la cartella `public` e PHP: `panel.json` contiene l'azione `site` e le altre chiavi del file restano.
- [ ] **«Ferma»** su `npm run dev`: in Gestione attività non resta nessun `node.exe`/`esbuild.exe` del progetto.
- [ ] **Electron o Tauri.** L'app apre la sua finestra e l'output arriva nella pagina.

### Le mie basi (D-162)
- [ ] **Laravel con Git.** In un progetto Laravel che è un repository Git, con `composer install` fatto e un `.env`: pagina del progetto › «Salva come base» (accanto ad «Apri la cartella»). «Cosa copiare» elenca cartelle e file della radice con il numero dei file e la dimensione, senza `vendor`, `node_modules` e `.env`, con `.env.example`; il totale è sotto l'elenco. «Salva come base» risponde «Salvata: la trovi in Nuovo progetto › Le mie basi…». `data\frameworks\mine\mine-<nome>` ha i file, niente `.env`, niente `.git`; `data\app\frameworks.json` ha la base con `"category": "mine"` e `"template": "mine-<nome>"`.
- [ ] **.gitignore rispettato.** Aggiungi una cartella a `.gitignore` (per esempio `docs/`) e un file nuovo non escluso: la cartella non è in «Cosa copiare», il file nuovo sì (contato nella sua cartella).
- [ ] **Node senza Git.** Un progetto Node senza `.git`, con `node_modules`, `dist` e `.env.local`: nessuno dei tre nell'elenco né nella cartella salvata; `src`, `package.json` e `.env.example` sì.
- [ ] **Cartella tolta.** Togli la spunta a una cartella: il totale scende; dopo il salvataggio la cartella non è in `data\frameworks\mine\<id>`.
- [ ] **Nuovo progetto da «Le mie basi».** «Nuovo progetto» (procedura, Pannello rapido, l'albero della barra laterale dopo aver riaperto Progetti): «Le mie basi», con la stella, è il primo gruppo. Creare un progetto dalla base salvata copia i suoi file in `projects\<nome>`, poi scrive `.env` ed esegue i passi; `package.json` o `composer.json` tengono il nome del progetto originale. Lo stesso in una cartella esistente con dei file e senza Git: i file che c'erano restano.
- [ ] **Lo stesso nome sostituisce.** Salva di nuovo con lo stesso nome dopo aver tolto un file dal progetto: la base compare una volta sola e il file non è più nella sua cartella.
- [ ] **Tolta nell'editor.** Impostazioni › istruzioni dei framework › la base › «Togli»: dopo «Salvato.», `data\frameworks\mine\<id>` non c'è più e la base non è più nella procedura. Cancellando invece la cartella a mano: creare dalla base dice «La cartella della base non c'è più: salvala di nuovo dal progetto.»

### Server web e database (sciame v3, D-96…D-102)
- [ ] **Configurazioni valide.** Con un progetto PHP avviato su Apache e su Caddy dedicati, in un pool e con il sito del kit: `httpd -t -f <conf>` e `caddy validate --config <Caddyfile>` sulle configurazioni scritte dal kit non danno errori. Un `kit.conf` di un kit precedente, dopo l'avvio, ha i blocchi nuovi una volta sola.
- [ ] **Estensioni PHP** (D-96). Con PHP: `a.phtml` si esegue; `a.inc`, `a.php5` e `.env` danno 403. Senza PHP (Apache senza modulo, Caddy senza php-cgi) `index.php` dà 403/503, mai il sorgente.
- [ ] **`php -S`** (D-97). Progetto PHP semplice avviato con «Avvia»: `/.env`, `/a.inc`, `/index.php.` danno 403; `curl -H "Host: evil.com"` dà 421; `/` e un file statico 200. Con un `router.php` nel progetto vale il suo.
- [ ] **Host estraneo sul server dedicato** (D-98). Senza «Rete locale»: `curl -H "Host: evil.com" http://127.0.0.1:<porta>/` dà 403 (Apache) o 421 (Caddy); `localhost`, `127.0.0.1`, `<progetto>.localhost` e i domini del progetto rispondono. Con «Rete locale» l'IP del PC da un altro PC apre il sito.
- [ ] **php-cgi che muore** (D-99). Progetto PHP su Caddy dedicato avviato (Windows): chiudere `php-cgi.exe` dal Task Manager ferma anche `caddy.exe`; l'esecuzione va in errore e il log ha «php-cgi terminato».
- [ ] **Pool che seguono i progetti** (D-101). Due progetti assegnati allo stesso profilo: «Avvia» sul primo avvia il pool, sul secondo lo ricarica; «Ferma» sul primo lo toglie (il suo indirizzo non risponde più) e il secondo resta servito. Con «Mantieni attivo» spento, fermare anche il secondo spegne il pool; acceso, il pool resta. Un pool avviato dalla pagina Server Web resta acceso comunque.
- [ ] **Avvia con Atollo** (D-101). Profilo con «Avvia con Atollo»: chiusa e riaperta l'app, il pool è acceso senza nessun progetto avviato. Un `servers.json` di prima (senza i due campi) si legge con «Mantieni attivo» spuntato.
- [ ] **Due azioni web di un progetto nello stesso pool** (BUG-V3-02). `http://<progetto>.localhost:<porta>` e `http://<azione>.<progetto>.localhost:<porta>` servono le due cartelle.
- [ ] **Due versioni di MySQL insieme** (D-102). Con 8.4 e 9.7 installati: Database › «Altre versioni» avvia 8.4 su una porta sua mentre 9.7 resta sulla 3306; `mysql -uroot -h127.0.0.1 -P<porta> -e "select version()"` dà le due versioni. Progetto con «Versione del server» 8.4: «Crea» crea database e utente su 8.4; un programma del progetto vede `DB_PORT` di 8.4; con un `.env` che ha `DB_PORT=3310` vale quello. Cambiare la versione principale mentre 8.4 gira è rifiutato; chiusa l'app, nessun `mysqld` resta.
- [ ] **Un avvio alla volta** (D-140). Due progetti che vogliono la stessa versione di MySQL avviati insieme (o «Avvia» due volte di fila nella pagina Database): un solo `mysqld`, e tutti e due ricevono la stessa porta. Cambiando la versione del server con 8.4 e 9.7 installati: ogni versione tiene la sua porta, e le stringhe di connessione dei progetti puntano ancora alla loro versione. Due backup avviati insieme finiscono tutti e due.
- [ ] **Il `conf.d` del progetto** (D-141, BUG-V5-06). Un `.conf` in `data\projects\<p>\apache\conf.d\` e un `.caddy` in `caddy\conf.d\` (per esempio un'intestazione aggiunta): il sito ce l'ha su Apache dedicato, su Caddy dedicato, in un pool Apache e in un pool Caddy. Con `caddy\conf.d\` vuota Caddy parte lo stesso.
- [ ] **«Elimina profilo»** di un pool chiede prima.
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
- [ ] **WordPress con Caddy**: permalink funzionanti senza `.htaccess`. Drupal propone Apache, che il catalogo offre su ogni sistema (su macOS e Linux la build del kit, D-116).
- [ ] **Apache su macOS e Linux** (D-116). Un progetto Laravel con un `.htaccess` proprio, sito Apache: `php-fpm` parte con Apache (`data/projects/<p>/apache/fpm.conf`), le rotte rispondono, `httpd` gira con `-DFOREGROUND`; fermando il sito si fermano entrambi.
- [ ] **Pool Caddy** con due siti e un alias verso un altro progetto; aggiungere un sito ricarica il pool senza fermare l'altro.
- [ ] **Stessa cartella dati MySQL** letta da Windows e da Linux con la stessa linea.
- [ ] **MySQL su Ubuntu 24.04** senza `libaio1` (solo `libaio1t64`): il server parte; senza `libnuma1` l'errore dice `sudo apt install libaio1t64 libnuma1`.

Accessi e pacchetto:
- [ ] **Antigravity e Claude Code** collegati su un Mac (Claude con «Collega con un token»), kit portato su un altro Mac: già collegati; nel Portachiavi del primo non resta nulla. Con il token, Claude Code aperto da Atollo parte collegato; un terminale del kit, l'editor del kit, un'azione di progetto e `Atollo env` non hanno `CLAUDE_CODE_OAUTH_TOKEN`; dopo «Esci» Claude Code aperto da Atollo chiede di nuovo l'accesso.
- [ ] **Antigravity su Linux con KeePassXC o KWallet** senza una collezione `login`: nessuna voce spostata, avviso «no "login" collection»; con GNOME `agy` trova l'accesso.
- [ ] **Antigravity su Linux** con un Secret Service bloccato: nessuna voce spostata, avviso; senza Secret Service e kit su exFAT: rifiutato.
- [ ] **Rilasci** (D-119, #5): su un Mac pulito `Atollo-<versione>-darwin-<arch>.app.zip` scaricato con il browser, estratto su una chiavetta exFAT, aperto seguendo il README (`xattr -dr com.apple.quarantine`); senza quel passo l'avvio si ferma con il comando. AppImage avviata da exFAT su Ubuntu 24.04, Fedora 42 e Debian 13; senza FUSE `./Atollo-<versione>-linux-<arch>.AppImage --appimage-extract-and-run`; il `tar.gz` con WebKitGTK 4.1. Aggiornamento dell'app da una versione di prova a un'altra su macOS e Linux: l'AppImage prende il nome della versione nuova e la vecchia sparisce all'avvio dopo (il `tar.gz` dice che la versione nuova va scaricata a mano).

Kit preparato su Windows per un altro sistema (a ogni rilascio):
- [ ] Su Windows `Atollo.exe prepare --target linux-x64` (poi `darwin-arm64`, `linux-arm64`) su una chiavetta exFAT, con l'app per quel sistema copiata accanto.
- [ ] Il selettore «Piattaforma» mostra per quel target cosa è pre-scaricato, cosa si completa al primo avvio e cosa non è disponibile.
- [ ] «Strumenti» › Panoramica › «Matrice» (D-121): le sei colonne con i nomi leggibili e «questo PC», «in emulazione» o «non eseguibile qui»; sulla chiavetta preparata le caselle del target dicono «pre-scaricato» o «si completa al primo avvio», quelle non disponibili il motivo; un clic apre il dettaglio e «Mostra questo sistema» cambia «Piattaforma». Chiusa e riaperta l'app, «Piattaforma» è ancora quel sistema; scelto quello di questo PC, sull'altro sistema la pagina si apre sul suo.
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
- [ ] **Copia di sicurezza prima di un ripristino** (D-154). «Ripristina…» chiede «Ripristinare {file}? Prima il kit copia ciò che il database contiene ora.»; dopo la conferma, in `data\projects\<progetto>\private\backups` c'è `<db>-prima-del-ripristino-<stamp>.sql` (MySQL) o `.archive.gz` (MongoDB) con i dati di prima del ripristino, e ripristinando quella copia tornano. MySQL con una vista, un trigger, una routine e un evento creati da root nel database del progetto: il ripristino riesce e tornano con l'utente del progetto come definer; una vista su una tabella eliminata non ferma la copia di sicurezza. Se la copia non si può fare (server fermato durante la copia, disco pieno) non si ripristina nulla.
- [ ] **Copia di sicurezza prima di un cambio di versione** (D-154). Con due versioni di MySQL (o MongoDB) installate e il database di un progetto con dei dati, cambiando «Versione del server» nelle copie compare `<db>-prima-del-cambio-di-versione-<stamp>`, fatta sulla versione lasciata. Con la versione lasciata disinstallata il cambio si fa e il log dice che la copia non c'è.
- [ ] **Copia blocco .env** (D-157). Pagina del progetto › Database, con il database creato: «Copia blocco .env» mostra «Copiato». In un progetto Laravel (`DB_CONNECTION=` nel `.env`) negli appunti ci sono le sei righe `DB_CONNECTION` … `DB_PASSWORD`; in un progetto Node senza, `DATABASE_URL=mysql://…` (MongoDB: `MONGODB_URI=mongodb://…`). Incollando in un `.env` vuoto o nel Blocco note si hanno le stesse righe, e il progetto si collega con quelle. Il file `.env` non cambia.
- [ ] **Solo tre copie di sicurezza** (D-154). Dopo quattro ripristini le copie `prima-del-ripristino` sono tre (le più recenti), quelle fatte con «Esporta» ci sono tutte e quelle `prima-del-cambio-di-versione` non sono toccate.
- [ ] **Nomi simili.** Progetto `my-app` (database `my_app`) e progetto `my2app`: l'utente `my_app` non vede `my2app` (`SHOW GRANTS`). Un progetto chiamato `mysql` o `admin` non può cancellare né cambiare i database dei server.
- [ ] **Progetto rinominato in Esplora file.** Compare in «Dati senza progetto» nella pagina Progetti; «Collega a…» lo riporta con password e copie, e la `.env` continua a funzionare. Un orfano con database si elimina con la stessa conferma.
- [ ] **C: quasi pieno.** Le installazioni ripiegano su `data\tmp` e il log ha l'avviso.
- [ ] **Alias dei siti.** Un sito con `/media` → `storage/uploads` e `/shared` → `@common/assets`, con Apache, con Caddy e in un pool: i file si aprono nel browser.
- [ ] **Alias e giunzione.** `storage` sostituito da una giunzione verso `C:\Users`: l'alias non è servito e la pagina del sito lo dice.
- [ ] **Impostazioni › Temporanei dei programmi.** La pagina mostra file system e tipo di disco del kit; Automatico, Nel kit e Nel sistema cambiano `$env:TEMP` in una console nuova del kit (`data\tmp\system` o `%TEMP%\Atollo\<id>\system`).
- [ ] **Alias e `.htaccess`** (D-100, SEC-V4-01). In una cartella di alias servita da Apache, un collegamento simbolico non si apre; un `.htaccess` con `Options +FollowSymLinks`, `RewriteRule`, `AddHandler` o `Require all granted` non cambia nulla (avviso nel log di Apache, la cartella resta servita); un `.php` o `.phtml` lì dà 403 con Apache, 404 con Caddy.
- [ ] **File scaricati a mano** (D-82). Un file messo a mano nel riquadro finisce in `%TEMP%\Atollo\<id>\downloads`; uno lasciato in `data\downloads` si usa lo stesso e si sposta.

### GitHub
- [ ] **Accesso.** GitHub › «Accedi a GitHub»: si apre github.com/login/device nel browser del PC e la pagina mostra il codice. Dopo l'autorizzazione compare «Accesso fatto come @…» senza ricaricare, `data\gh\hosts.yml` contiene l'account e nella Gestione credenziali di Windows non c'è nessuna voce `gh:github.com`. «Annulla» durante l'attesa non lascia nessun `gh.exe`.
- [ ] **Un solo accesso.** Dopo l'accesso, `git config --global --get-all credential.https://github.com.helper` in una console del kit dà una riga vuota e `!gh auth git-credential`. Push di un repository privato dall'editor (Controllo del codice sorgente) e da una console senza nessuna finestra di Git Credential Manager. Claude Code usa `gh` con lo stesso account. Dopo «Esci» il helper sparisce. Dopo aver spostato il kit su un'altra lettera di unità, il push funziona ancora.
- [ ] **Repository.** Elenco con i repository propri e delle organizzazioni (gruppi separati), ricerca, «Clona» su un repository privato: la cartella è in `projects\` e `.git\config` non contiene il token. Con il nome già preso la cartella diventa `<proprietario>-<nome>`. Sul repository clonato c'è «Editor».
- [ ] **Rinomina ed elimina** su un repository di prova: dopo «Rinomina» il remote del clone punta al nuovo nome. «Elimina da GitHub» chiede conferma; la prima volta offre «Concedi il permesso», poi con un nuovo accesso l'eliminazione riesce e il clone resta in `projects\`.

### Agenti
- [ ] **Claude Code, Codex, Antigravity e Pi.** Accesso, avvio dall'app e dall'editor, allineamento di istruzioni e skill fra i quattro.
- [ ] **Terminale di un agente.** Home o Progetti › il pulsante di Claude Code: si apre una scheda del pannello dei terminali dell'app (nessuna finestra di console, niente Windows Terminal, anche se è il terminale predefinito del PC). Claude Code riceve Invio, frecce, Esc e Ctrl+C. Chiudendo la scheda non resta nessun `claude.exe` né `OpenConsole.exe`.
- [ ] **ConPTY del kit.** Nel Task Manager la console della sessione è `OpenConsole.exe` di `data\apps\win32-*\conpty\`, mai `conhost.exe`. Senza il componente e senza rete, «Terminale» dice nell'app che non riesce a installarlo; con la rete lo scarica prima (avanzamento in «Attività») e apre il terminale.
- [ ] **Git quando serve.** Su un kit senza Git, all'avvio si installano solo i componenti di sistema: GitHub CLI funziona senza Git (D-81). Scegliere Claude Code, clonare un progetto o avviare un'azione con `git` installa Git; togliere la spunta a Git toglie anche Claude Code.
- [ ] **Editor aggiornato con una cartella aperta.** Con Esplora file aperto su `data\apps\win32-x64\vscodium-server\extensions`, l'editor di un'altra versione viene comunque sostituito all'avvio (D-60).

- [ ] **Limiti d'uso.** Con l'accesso fatto in Claude Code e Codex, Agenti mostra gli stessi valori di `/usage` e `/status`, con l'ora in cui si azzerano. Senza accesso compare «accesso non fatto». Senza rete l'errore si legge e la pagina resta usabile. «Aggiorna i consumi» premuto più volte di fila non fa più di una richiesta ogni 20 s.
- [ ] **Solo nel progetto.** In Home e Progetti, Editor, Terminale e agente si aprono nella cartella del progetto; non c'è nessun pulsante per `projects\`. Nell'editor, File › Apri cartella su un'altra cartella, «Nuova finestra» e «Chiudi cartella» non escono dal progetto.
- [ ] **Fiducia nell'editor.** Alla prima apertura di un progetto l'editor chiede se fidarsi. Con «No» compare la barra «Restricted Mode» e le attività non partono; con «Sì» non chiede più alla riapertura, anche dopo aver riavviato l'app (D-67).
- [ ] **dev-settings.json.** Dopo aver aperto Codex in un progetto, `data\projects\<progetto>\dev-settings.json` ha `"agent": "codex"` e la scelta dell'agente propone Codex. Eliminando dall'app un progetto vuoto la sua cartella in `data\projects\` sparisce (D-89).

### Modelli locali e Pi (D-151, D-152)
Provato a mano su Windows x64 (con una RTX 2070, Vulkan): installazione di llama.cpp e di Pi 1.4.0 con Git, un download interrotto e ripreso (0,5 GB, e 4,7 GB in due parti), avvio, una risposta con la chiave e 401 senza, arresto insieme ai processi dei modelli, `sync`, Pi in `--print` da un progetto sul modello locale con lo strumento bash. **Non ancora provato a mano**: macOS (Metal) e Linux (Vulkan, e la CPU senza `libvulkan1`), Windows ARM64 e Linux ARM64, un PC senza GPU.
- [ ] **Download interrotto e ripreso.** Modelli locali › un modello del catalogo › «Scarica», poi «Sospendi» (o chiudi Atollo, o stacca la rete): `<file>.part` resta in `data\models\gguf\<proprietario>--<nome>\`. «Riprendi» (o `Atollo.exe models pull <id>` dopo un riavvio) continua da dove si era fermato, non da zero; alla fine lo SHA-256 corrisponde e `atollo.json` ha `complete`. Un `.part` alterato a mano si cancella alla fine, con l'errore.
- [ ] **Modello in più parti.** Qwen2.5-Coder 7B scarica entrambe le parti; «Cancella» le toglie tutte e due con la cartella.
- [ ] **Rifiuto su FAT32.** Con il kit su una chiavetta FAT32, un modello con un file oltre i 4 GiB (Qwen3.5 9B) è rifiutato prima del primo byte, con cosa fare; Qwen2.5-Coder 7B (due parti sotto i 4 GiB) si scarica. Un modello più grande dello spazio libero è rifiutato allo stesso modo.
- [ ] **Hugging Face.** «Cerca i file GGUF» su un repository pubblico elenca le quantizzazioni con la dimensione; un repository ad accesso controllato senza token dice che serve un token; con il token salvato (`data\app\secrets\hf-token`, solo per il proprietario) si scarica.
- [ ] **Server con la chiave.** «Avvia»: `llama-server` ascolta solo su `127.0.0.1`, sulla 9931 o la prossima libera; `curl http://127.0.0.1:<porta>/v1/models` senza chiave risponde 401, con `Authorization: Bearer <chiave>` («Copia la chiave») l'elenco dei modelli. La chiave non è sulla riga di comando di `llama-server` (Gestione attività › Riga di comando). `Atollo.exe llm status` dice se gira senza mandare la chiave.
- [ ] **Chat nel browser.** «Apri la chat»: la pagina di llama.cpp chiede la chiave nelle sue impostazioni; con la chiave copiata risponde.
- [ ] **Arresto.** Il server è in Attività e nel conteggio della barra di stato; «Ferma tutto», «Rimuovi l'unità» e la chiusura di Atollo lo fermano con i processi dei modelli (nessuno resta in Gestione attività). Cancellare un modello mentre il server gira prima lo ferma, poi il file se ne va.
- [ ] **Pi da un progetto.** Con un modello che usa gli strumenti scaricato (Qwen3.5 9B), il pulsante Pi di un progetto apre Pi nella cartella del progetto sul modello locale senza altre impostazioni; se gli si chiede di elencare i file, usa lo strumento bash con il Git Bash del kit. Anche `pi` in un terminale del kit funziona. Con un provider in cloud collegato con `/login`, Pi usa quello e il modello locale resta disponibile.
- [ ] **L'ambiente di Pi.** In `data\home\.pi\agent\`: `models.json` con il provider `atollo` (solo per il proprietario), `settings.json` con il Git Bash del kit come `shellPath`, `APPEND_SYSTEM.md` con il blocco del kit (il testo dell'utente resta), il progetto in `trust.json`. Nessun avviso di aggiornamento da Pi (`PI_SKIP_VERSION_CHECK=1`).

### Server MCP pronti (D-156)
Provato a mano su Windows x64: Claude Code 2.1.306 espande `${VAR}` nelle intestazioni di un server a livello utente (un server d'eco locale ha ricevuto `Bearer <valore>`); DBHub legge `DSN` dall'ambiente. **Non ancora provato a mano**: Antigravity e Pi che espandono `${VAR}`, il server remoto di GitHub con il token della CLI `gh`, mongodb-mcp-server da capo a fondo.
- [ ] **GitHub.** Con l'account GitHub del kit collegato, Agenti › «Server MCP del kit» › «Server pronti» › «Aggiungi» accanto a github: `data\agents\mcp.json` ha `Authorization: Bearer ${GITHUB_PERSONAL_ACCESS_TOKEN}` e nessun token. Claude Code avviato dall'app, a cui chiedi le issue di un tuo repository, le elenca con il server github.
- [ ] **Playwright.** Dopo aver aggiunto playwright, un agente avviato dall'app, a cui chiedi di aprire una pagina, apre un browser (prima si installa Node, se manca).
- [ ] **Filesystem limitato al progetto.** Progetto › Editor › «Server MCP del progetto» › «Aggiungi» accanto a filesystem: `.mcp.json` ha `npx -y @modelcontextprotocol/server-filesystem .`; l'agente legge e scrive con lui i file del progetto, e un percorso fuori dalla cartella del progetto è rifiutato.
- [ ] **MySQL.** mysql non è offerto finché il database del progetto non esiste; dopo «Crea database e utente» con MySQL avviato, «Aggiungi» accanto a mysql scrive `DSN: ${DSN}` in `.mcp.json` (nessuna password); l'agente avviato dall'app elenca le tabelle del database del progetto. Lo stesso per MongoDB con mongodb.
- [ ] **Nessun token per un repository clonato.** Un repository clonato il cui `.mcp.json` ha un server che chiede `${GITHUB_PERSONAL_ACCESS_TOKEN}` (per esempio in `env` o in un'intestazione) non riceve niente: con il server github assente da `mcp.json` del kit, quel server riceve un valore vuoto o non espanso, mai il token.
- [ ] **Codex.** In un terminale del kit `codex mcp list` mostra i server del kit (github con `bearer_token_env_var`, playwright, fetch) e, in un progetto, quelli del progetto (mysql con `env_vars`).
- [ ] **Subito, e a mano.** «Togli» su un server del kit lo toglie da `.claude.json` di Claude Code, `config.toml` di Codex, `mcp_config.json` di Antigravity e `mcp.json` di Pi senza riavviare l'app. Un `data\agents\mcp.json` che non è JSON valido mostra l'avviso e non viene riscritto.

### Palette dei comandi (D-160)
**Non ancora provato a mano**: che WebView2 (Windows), WKWebView (macOS) e WebKitGTK (Linux) lascino Ctrl+Shift+P alla pagina invece di usarlo loro (il gestore chiama `preventDefault`).
- [ ] **Dai Progetti.** Ctrl+Shift+P (⌘⇧P su macOS) nella pagina Progetti apre «Comandi» con la casella «Cosa vuoi fare? Avvia, ferma, apri, un progetto, un agente…»; i progetti sono dal più recente, «Vai a» elenca tutte le pagine tranne Progetti.
- [ ] **Da un'altra pagina.** In Impostazioni (o Strumenti) Ctrl+Shift+P apre la stessa finestra; «Vai a Impostazioni» non c'è, «Vai a Progetti» sì. «vai impostazioni» scritto in un'altra pagina trova «Vai a Impostazioni» (le parole in qualsiasi ordine, maiuscole o minuscole).
- [ ] **Da un terminale.** Con il cursore in un terminale del kit (pagina Terminali o terminale dell'editor nella pagina), Ctrl+Shift+P apre «Comandi» e il terminale non riceve niente; Ctrl+P nel terminale resta della shell (in PowerShell o Bash, la riga precedente della cronologia).
- [ ] **Avviare e fermare un progetto.** «Avvia <progetto>» avvia la sua azione predefinita (compare in Attività); riaprendo la palette, la voce è «Ferma <progetto>» e lo ferma. Un errore dell'avvio (per esempio una porta occupata) compare sotto la barra del titolo, con la X che lo chiude.
- [ ] **Aprire.** «Apri <progetto>» porta alla sua pagina; «Apri <progetto> nell'editor» (solo con l'editor installato) apre l'editor nella sua cartella.
- [ ] **Un agente nel progetto.** «Claude Code in <progetto>» (o un altro agente installato) apre l'agente nella cartella del progetto, come il pulsante della pagina del progetto; un agente non installato non ha voci.
- [ ] **Database.** Con MySQL installato, «Avvia MySQL» lo avvia e la voce diventa «Ferma MySQL»; lo stesso per MongoDB. Un server non installato non ha voci.
- [ ] **Ferma tutto.** Con un'azione, un database e un pool in corso, «Ferma tutto» (presente solo allora) li ferma senza chiedere conferma; il pannello Attività resta vuoto. Senza niente in corso la voce non c'è.
- [ ] **Chiudere.** Esc, o un clic fuori dalla finestra, chiude senza fare niente; ↑/↓ spostano la scelta, Invio esegue quella evidenziata.
- [ ] **Ctrl+P invariato.** Ctrl+P apre ancora «Apri un progetto» come prima: Invio apre la pagina, Maiusc+Invio avvia l'azione predefinita, Esc chiude.

### Strumenti
- [ ] **Tipi** (D-139). «Tutti gli strumenti» ha i blocchi Sistema ed editor, Agenti AI, Modelli locali, Linguaggi, Strumenti di sviluppo, Server web, Database, ognuno con titolo, numero, descrizione e una casella per sceglierli tutti, e si chiude con un clic; gli stessi tipi nell'albero della barra laterale, nella matrice e al primo avvio.
- [ ] **Loghi.** Ogni componente ha il suo logo, leggibile nel tema chiaro e in quello scuro; i progetti mostrano le etichette dei linguaggi.
- [ ] **Dipendenze.** Apache mostra la dipendenza «PHP»; con Claude Code scelto, Git mostra «per Claude Code». Togliendo PHP si toglie anche Apache.
- [ ] **PATH dopo un'installazione.** Dopo l'installazione di un linguaggio da questa pagina, Home › Terminale apre la console del kit con PowerShell 7 del kit e il programma nuovo è nel PATH, senza riavviare l'app.
- [ ] **Verifica i file** (D-159). Dopo l'installazione di un componente (per esempio `gh`) la sua cartella ha `.atollo-manifest` (intestazione `# Atollo manifest 1: sha256 size path`); «Verifica i file» nella barra degli strumenti mostra «Nessun file danneggiato: N cartelle verificate.»
- [ ] **File: danno trovato.** Cambiando un byte di un file nella cartella del componente (per esempio `bin\gh.exe`) e cancellandone un altro, «Verifica i file» elenca il componente con «1 file cambiati, 1 mancanti» e i percorsi; con due componenti danneggiati compare anche «Reinstalla tutti».
- [ ] **File: Reinstalla.** «Reinstalla» sulla riga danneggiata prepara di nuovo il componente alla stessa versione (progetti e database restano come erano); «Verifica i file» torna a posto.
- [ ] **File: senza manifesto.** Un componente installato con una versione precedente a D-159 (o con `.atollo-manifest` cancellato) compare in «Senza manifesto, installati prima che Atollo lo tenesse: …»; dopo il suo aggiornamento si verifica.
- [ ] **File: non è un danno.** `pip install requests` nel Python del kit, una gem in Ruby, una modifica a `httpd.conf` di Apache e i log di un server avviato: «Verifica i file» resta a posto.
- [ ] **File: riga di comando.** `Atollo.exe check --files | Out-Host` stampa `√`, `x` o `-` per cartella, i percorsi danneggiati sotto le righe `x`; `$LASTEXITCODE` è 0 quando è tutto a posto, 1 con una cartella danneggiata.
- [ ] **Aggiornamenti: controllo giornaliero** (D-161). Con Impostazioni › Generale › «Aggiornamento automatico» attivo, un componente installato indietro rispetto alla sua fonte (per esempio `gh` installato a una release più vecchia in un kit di prova) e senza `data\app\updates.json`: circa 10 secondi dopo un avvio il riquadro «Aggiornamenti» di Strumenti si riempie («2.100.0 → 2.103.0»), il suo titolo e l'icona di Strumenti nella barra delle attività mostrano il numero («Versioni nuove: 1» nel suggerimento), `updates.json` ha `checked` e le versioni. Un secondo avvio nello stesso giorno non chiede niente (`checked` non cambia).
- [ ] **Aggiornamenti: spento.** Con «Aggiornamento automatico» spento (o `ATOLLO_NO_UPDATE` impostata) e `updates.json` cancellato, dopo un avvio non si chiede niente: il riquadro dice «Non ancora controllato…», nessun badge, nessun `updates.json`.
- [ ] **Aggiornamenti: Controlla ora.** La freccia circolare «Controlla ora» sul titolo del riquadro chiede subito (anche con «Aggiornamento automatico» spento): l'elenco e «Controllato il <data>» si aggiornano; con tutto all'ultima versione dice «Tutto aggiornato.».
- [ ] **Aggiornamenti: Cosa cambia.** «Cosa cambia» apre le note di rilascio nel browser del PC (per `gh`, `https://github.com/cli/cli/releases/tag/v<versione nuova>`); per un componente senza note nel catalogo, il suo sito.
- [ ] **Aggiornamenti: Aggiorna.** «Aggiorna» nel riquadro aggiorna solo quel componente (come il pulsante della sua riga); finita la preparazione la voce sparisce dal riquadro e il numero del badge scende.
- [ ] **Sito ufficiale.** Nella pagina di uno strumento «Sito ufficiale» ora apre il browser sul sito del componente (prima di D-161 non faceva niente).

### Terminali
- [ ] **Versione sostituita.** Un terminale salvato con PHP 8.2, poi 8.2 tolto: la pagina Terminali dice quale usa al suo posto («PHP 8.2 → 8.4») e il terminale si apre con la 8.4.
- [ ] **Eliminazione.** «Elimina il terminale» chiede prima; Esc chiude la domanda e il terminale resta.

### Impostazioni
- [ ] **Niente «Salva»** (D-137). Una modifica si salva e si applica da sola mezzo secondo dopo l'ultimo tasto: la barra delle schede dice «Salvataggio…», poi «Salvato.»; `data\app\settings.json` ha il valore nuovo.
- [ ] **Lingua.** Passando a English, Français e Deutsch, tutta l'interfaccia cambia subito. Con «Lingua del sistema» vale quella del sistema.
- [ ] **Tema.** «Come il sistema», «Chiaro» e «Scuro» cambiano subito l'app, l'editor e i terminali, senza riavvio.
- [ ] **Chiave di Gemini.** Si salva quando il campo perde il fuoco, poi il campo si svuota e mostra «salvata»; sta in `data\app\secrets\gemini-api-key`, mai in `settings.json`. Non compare mai in chiaro (né nell'interfaccia, né in `data\app\logs`). «Togli la chiave» la rimuove.
- [ ] **Valore non valido** (proxy `proxy:8080`, un file PEM senza certificati): l'errore si legge, il valore resta sullo schermo e il file non cambia.
- [ ] **Editor in esecuzione.** Dopo una modifica di lingua, rete o temporanei dei programmi l'app chiede di riavviare l'editor, e dopo il riavvio i terminali dell'editor vedono le variabili nuove.
- [ ] **Privacy.** La sezione la riassume e «Leggi l'informativa sulla privacy» la apre nella lingua dell'app.
- [ ] **Aggiornamento.** Con una release `app-v*` più recente pubblicata, «Cerca aggiornamenti» scarica l'exe. Alla chiusura `Atollo.exe` è quello nuovo e `Atollo.old.exe` sparisce al riavvio. Con `ATOLLO_NO_UPDATE=1` all'avvio non parte nessun controllo.
- [ ] **Esporta diagnostica** (D-153). Impostazioni › Il kit › «Esporta diagnostica» mostra il percorso di `data\app\diagnostics\atollo-diagnostica-<yyyymmdd-hhmmss>.zip` e «Apri la cartella» apre quella cartella; `Atollo.exe diagnostics | Out-Host` ne crea un altro. Lo zip ha `summary.txt`, `env.txt`, `check.txt`, `components.txt`, `ports.txt` e `logs\*.log`. Cercando in tutti i suoi file: né la chiave di `data\app\secrets\llama-key`, né la chiave di Gemini, né una password dei database, né i token degli agenti; né il nome dell'utente di Windows né quello del PC (`USERNAME=***`, `COMPUTERNAME=***` in `env.txt`); la cartella del profilo compare come `~`.
- [ ] **Segnala un problema** (D-153). Impostazioni › Il kit › «Segnala un problema» apre nel browser del PC `github.com/nabre/atollo/issues/new` con il modulo dei problemi, «Atollo version» e «System» già compilati (sistema del kit, build di Windows, file system e tipo del disco). Nulla viene creato finché non si preme «Create» su GitHub; lo zip della diagnostica si trascina nella segnalazione.
- [ ] **Salute del kit** (D-155). Impostazioni › Il kit › «Salute del kit» mostra una riga per disco, spazio, percorso e file del kit (su Linux anche le librerie), tutte verdi su un kit NTFS in una cartella semplice; `Atollo.exe check --kit | Out-Host` stampa le stesse righe e `$LASTEXITCODE` è 0. «Controlla di nuovo» rifà i controlli.
- [ ] **Salute: chiavetta FAT32.** Con il kit su una chiavetta FAT32 la riga del disco è un avviso (file fino a 4 GB, niente collegamenti) che consiglia exFAT o NTFS; su exFAT la riga è a posto su Windows.
- [ ] **Salute: poco spazio.** Con meno di 5 GB liberi sul disco del kit la riga dello spazio è un avviso; sotto 1 GB è un errore con «Libera spazio…» e `check --kit` finisce con 1.
- [ ] **Salute: percorso a rischio.** Con il kit in una cartella come `D:\R&D\Atollo` o `D:\Città\Atollo` la riga del percorso è un avviso che elenca «&» o «à».
- [ ] **Salute: file di un altro account.** Un file di `data\home` reso leggibile solo da un altro account (Proprietà › Sicurezza, oppure copiato da un altro PC con i permessi) dà un errore con quanti file, il primo e il comando `takeown … && icacls … /reset /t`; eseguito da amministratore, «Controlla di nuovo» torna verde. Su macOS e Linux lo stesso con `sudo chown root` e il comando `sudo chown -R …`.
- [ ] **Salute: Linux senza libaio.** Su un Linux senza `libaio` (o senza `libnuma`, `libicu`, `libvulkan1`) la riga delle librerie è un avviso che le nomina, con `sudo apt install …` dei pacchetti mancanti; installati, «Controlla di nuovo» è a posto. Con il kit su un disco montato `noexec` la riga del disco è un errore con il comando per rimontarlo.

### Copia del kit (D-163)
- [ ] **Esportare in una cartella con MySQL acceso.** Con MySQL avviato a mano e MongoDB segnato «Avvia con Atollo»: Impostazioni › Il kit › «Copia del kit» › «Esporta in una cartella…» e scegli una cartella su un altro disco. MySQL e MongoDB si fermano durante la copia; alla fine MongoDB è di nuovo acceso e MySQL resta fermo. La pagina dice «Copia fatta: … file, …» con il percorso; `<cartella>\Atollo-<aaaammgg-hhmmss>` ha `projects\`, `data\` e `atollo-backup.json`, e non ha `data\apps`, `data\cache`, `data\models`, `data\webview2`, `data\app\logs`. L'azione avviata di un progetto non si è fermata.
- [ ] **Esportare in uno zip.** «Esporta in uno zip…»: `Atollo-<aaaammgg-hhmmss>.zip` si apre in Esplora risorse con lo stesso contenuto; mentre si scrive c'è un `.zip.part`, mai uno zip a metà con il nome finale.
- [ ] **Copia nel kit rifiutata.** Scegliere per l'esportazione la cartella del kit (o una al suo interno) dà un errore e non scrive niente; lo stesso scegliendo per l'importazione una copia dentro il kit.
- [ ] **Uno zip non di Atollo rifiutato.** «Importa da uno zip…» con un altro zip qualsiasi (senza `atollo-backup.json`, o con un percorso fuori da `projects/` e `data/`) dà un errore; nel kit non resta una cartella `.atollo-import`.
- [ ] **Annullare.** Importa una copia valida: «Copia pronta: … file, …, fatta il …» con «Chiudi Atollo per importarla», «Annulla l'importazione», «Apri la cartella del kit» e «Poi riapri Atollo…». «Annulla l'importazione» toglie `.atollo-import\`; riaprendo Atollo non cambia niente.
- [ ] **Importare in un altro kit su un altro disco.** Copia `Atollo.exe` in una cartella vuota su un altro disco, aprilo, lascia finire la preparazione, poi importa lo zip del primo kit e «Chiudi Atollo per importarla»; riaprilo. Il kit parte normalmente, il log ha «kit copy imported; what it replaced: …», i progetti e i dati di prima sono in `.atollo-before-import-<ora>\`, il kit dice che è stato spostato, i programmi già scaricati restano e quelli che mancano si riscaricano. I progetti si aprono e si avviano; i loro database (MySQL, MongoDB) partono e hanno tabelle e utenti.
- [ ] **Errore all'avvio.** Con un file di `projects\` tenuto aperto da un altro programma durante lo scambio, Atollo si ferma con «Importazione della copia del kit non riuscita: …»; non si cancella niente: `projects\` è dov'era e la copia è ancora in `.atollo-import\`. Chiudendo l'altro programma e riaprendo Atollo l'importazione si completa.

### Chiusura, altre copie, rimozione dell'unità
- [ ] **Chiusura con qualcosa aperto.** Con editor, console o un progetto avviato, chiudere la finestra chiede conferma ed elenca cosa si chiude. «Annulla» (anche con Esc) lascia tutto aperto.
- [ ] **Due copie del kit aperte.** La Home della seconda mostra l'avviso con la cartella dell'altra.
- [ ] **Rimuovi l'unità.** Tutto si chiude e l'accesso di Antigravity torna nel kit. Dopo «Chiudi Atollo» la chiavetta si espelle senza «dispositivo in uso».

### Barra del titolo
- [ ] **Finestra principale senza cornice di Windows.** Trascinamento dalla barra, doppio clic per ingrandire, Riduci, Ingrandisci/Ripristina, Chiudi (con la conferma se qualcosa è aperto); ridimensionamento dai bordi; Win+frecce funzionano. I pulsanti si raggiungono con Tab e hanno un nome per l'Assistente vocale.
- [ ] **Database** (D-73). Pagina Database: «Avvia» su MySQL non installato lo installa e lo avvia; `mysql -uroot -h127.0.0.1` e `mongosh` rispondono; porta occupata: messaggio e cambio porta; «Avvia con Atollo» lo fa partire al riavvio dell'app; chiusura dell'app: la conferma conta i database e i log finiscono con «Shutdown complete» / «Now exiting». Un progetto Laravel con `DB_CONNECTION=mysql` riceve SQLTools.
- [ ] **Pagine dei database** (D-150). Pagina Database con MongoDB avviato: solo i dettagli, senza schede; il contenuto si apre con «Apri mongo-express», quello di MySQL con «Apri phpMyAdmin». SQLite non è nella lista.
- [ ] **Password dei database** (D-74). Pagina Database: password di root/admin nascosta, «Mostra» e «Copia». `mysql --no-defaults -uroot -h127.0.0.1` rifiutato; `mysql` nel terminale del kit entra. Pagina del progetto › Database: «Crea» e poi «Scrivi in .env» mostra prima le righe che cambiano.
- [ ] **Impostazioni del progetto** (D-78). Avanzate: «Nuova azione» con una variabile `NOME=valore` compare in tabella e in `panel.json`; «Predefinita» la mette sul pulsante Avvia; «Togli» la toglie. Editor: formatta al salvataggio e rientro scritti in `.vscode\settings.json`, aperti nell'editor valgono. MCP: un server aggiunto compare in `.mcp.json` e in `.codex\config.toml`; `codex mcp list` nel progetto lo elenca. Risorse › database: «Esporta» crea la copia, «Ripristina…» chiede conferma e riporta i dati.
- [ ] **Struttura a sei pagine** (D-76). Progetti è la prima pagina; «Nuovo progetto» con le tre strade (il clone dai repository chiede l'accesso a GitHub nelle Impostazioni); pagina del progetto con Riepilogo, Risorse, Avanzate; pannello Attività: avviare un progetto, aprire il pannello, vedere l'output, Apri nel browser, Ferma; installazione in corso nel pannello; database con errore (porta occupata) nel pannello.
- [ ] **Pagina del progetto** (D-75). Dal nome in Progetti e nella Home; le sei schede si scelgono anche con le frecce; Avvio avvia e ferma come prima; Database crea il database e scrive il `.env` dopo l'anteprima; Git mostra ramo e remoto di un progetto clonato da GitHub.
- [ ] **Branding** (D-72). App: logo nella barra del titolo e nella Home, accento verd'Atollo in tema scuro e chiaro, testo dei pulsanti leggibile. Editor: titolo «<progetto> - Atollo», logo nella barra, benvenuto con logo e progetto nell'editor vuoto, anche in tema chiaro.
- [ ] **Estensioni mentre si lavora** (D-71). Editor aperto su un progetto senza Ruby: si crea `app.rb`; entro circa 20 s (più il download) compare l'avviso con Ruby LSP e Ruby Debugger; «Riavvia le estensioni» ricarica l'editor sul progetto, «Più tardi» chiude l'avviso.
- [ ] **Finestra dell'editor con la barra di VSCodium** (D-69). Menu File…Help nella barra; trascinamento dalle zone vuote, doppio clic per ingrandire, Aero Snap; Riduci, Ingrandisci/Ripristina (l'icona cambia anche con Win+frecce), Chiudi; ridimensionamento dai bordi; tema chiaro e scuro. Con il server dell'editor fermo compare la barra di ripiego, che sposta e chiude la finestra.

### Accessibilità
- [ ] **Solo tastiera.** Tutte le pagine si usano: Tab mostra sempre il contorno di focus, «Vai al contenuto» è il primo elemento, il dialogo di chiusura riceve il focus.
- [ ] **Conferme nella pagina** (D-106). Uscita da un agente, da GitHub, ripristino di un database e «Applica» in Strumenti: l'Assistente vocale legge la domanda, il focus è su «Annulla», Esc chiude e il focus torna al pulsante di partenza. Lo zoom di una schermata della Guida tiene Tab dentro e si chiude con Esc.
- [ ] **Assistente vocale.** Le pagine hanno titoli e le icone hanno un nome. Errori e stati sono annunciati (`role="alert"` e `role="status"`).
- [ ] **Stesso aspetto dell'editor** (D-88, D-142). App ed editor affiancati, in tema scuro e in chiaro: stessi colori di sfondo, testo, focus, pulsanti, campi e menu a tendina; le stesse misure della UI moderna di VSCodium 1.135 (barra del titolo di 30 px, barra delle attività di 36 px, schede dell'editor di 24 px in una barra di 32 con la chiusura al passaggio, schede del pannello a pillola, righe delle liste rientrate di 4 px con raggio di 4, barre di scorrimento di 8 px, dialoghi con raggio di 12); select, campi e caselle si usano con mouse e tastiera; le icone codicon si vedono (nessun quadratino).
- [ ] **Tema e animazioni.** Con il tema su «Come il sistema» l'app segue il chiaro e lo scuro del sistema. Con «Effetti di animazione» spenti in Windows, nessuna icona gira.
