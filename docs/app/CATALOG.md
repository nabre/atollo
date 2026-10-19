# Atollo App: catalogo di agenti, linguaggi e strumenti

Fonte: `nabre/atollo` al commit `a61a354` (`setup/components.json` e script di `setup/windows/`). È la specifica che il motore Rust (`setup/catalog.rs`, `install/`) segue; il catalogo dell'app è `tauri/catalog/components.json`. I riferimenti `file:riga` puntano agli script originali.

Percorsi nel nuovo kit (D-00, D-114): `A` = `data\apps\<target>\` (`win32-x64`, `win32-arm64`, `darwin-arm64`, `darwin-x64`, `linux-x64`, `linux-arm64`); versioni in `data\app\versions\<target>\<id>.txt`; aree di lavoro nel temp del sistema, altrimenti in `data\tmp\setup\` (D-90). Le tabelle del §2 descrivono Windows; le fonti degli altri sistemi sono al §1, «Fonti per sistema».
Prove reali: la CI (job x64) installa `pwsh`, `node`, `git` e l'editor; tutti i componenti sono stati installati e rimossi a mano nella fase 4 (CHANGELOG.md).

## 1. Componenti e gruppi

| Gruppo | Id | Nome | Predefinito | MB (x64) | Dipende da | Note |
| --- | --- | --- | --- | --- | --- | --- |
| obbligatori (`required`) | `gh` | GitHub CLI | sempre | 40 | | pull request, issue, accesso a GitHub (anche per i clone privati); funziona senza Git (D-81) |
| | `pwsh` | PowerShell 7 | sempre | 110 | | **nuovo**; terminale del kit |
| app (`app`) | `vscodium-server` | Editor (server web di VSCodium) | sempre | 250 | `pwsh` | **nuovo**; parte dell'app, versione fissata a ogni rilascio (DECISIONS.md D-58); in Linguaggi e strumenti compare con gli obbligatori, spuntato e non modificabile; SHA-256 fissato per target, x64 in emulazione su Windows ARM64 |
| agenti (`agents`) | `claude` | Claude Code | sì | 250 | `git` | |
| | `codex` | Codex | sì | 440 | | |
| | `antigravity` | Antigravity | sì | 190 | | accesso salvato nel kit (`agents/keyring.rs`) |
| linguaggi e strumenti (`languages`) | `git` | Git (Git Bash su Windows) | quando serve (con Claude Code) | 350 | | Git Bash è la shell dei comandi degli agenti su Windows; arriva con Claude Code, un clone o un progetto (D-81) |
| | `python` | Python + pip | sì | 70 | | l'ultima linea minore stabile, salvo `pythonVersion` nelle Impostazioni (D-81) |
| | `node` | Node.js LTS + npm | no | 110 | | quando serve: azioni con `npm`, server MCP con `npx` (D-59) |
| | `uv` | uv, uvx | no | 35 | | quando serve: azioni con `uv`, server MCP con `uvx` (D-59) |
| | `php` | PHP + Composer | sì | 100 | | Windows: con VC++ locale, x64 in emulazione su Windows ARM64; macOS e Linux: static-php-cli (D-112) |
| | `apache` | Apache | sì | 45 | `php` | Windows: Apache Lounge con il modulo PHP, x64 in emulazione su Windows ARM64; macOS e Linux: build del kit (`httpd-v<versione>`, SHA-256 fissati in `sha256ByLine`) con `php-fpm` (D-116) |
| | `caddy` | Caddy | no | 45 | | **nuovo** (D-111); server web dei pool e dei siti su tutti i sistemi; al posto di Nginx, tolto con la migrazione dei suoi pool e siti all'avvio (D-120) |
| | `rust` | Rust | no | 920 | `cc` su Windows ARM64 | host `gnu` su Windows x64, `gnullvm` su Windows ARM64 (collegato da llvm-mingw di `cc`), `msvc` dalle Impostazioni; nativo su macOS e Linux |
| | `go` | Go | no | 250 | | |
| | `java` | Java (Temurin, Maven, Gradle) | no | 300 | | |
| | `kotlin` | Kotlin | no | 80 | `java` | |
| | `android` | Android SDK | no | 600 | `java` | **nuovo** (D-84); strumenti a riga di comando, platform-tools, piattaforma e build-tools, senza emulatore; sceglierlo accetta le licenze dell'SDK di Google; x64 in emulazione su Windows ARM64; non disponibile su Linux ARM64 (`unavailable`: Google non pubblica platform-tools e build-tools) |
| | `dotnet` | .NET SDK | no | 250 | | |
| | `dart` | Dart | no | 200 | | |
| | `zig` | Zig | no | 200 | | |
| | `julia` | Julia | no | 500 | | x64 in emulazione su Windows ARM64 |
| | `bun` | Bun | no | 90 | | |
| | `deno` | Deno | no | 100 | | |
| | `typst` | Typst | no | 50 | | |
| | `tinytex` | TinyTeX | no | 280 | | x64 in emulazione su Windows ARM64 |
| | `pnpm` | pnpm, yarn | no | 10 | `node` | in `data\npm-global\` |
| | `cmake` | CMake, Ninja | no | 100 | | |
| | `cc` | C, C++ | no | 720 | `zig` su macOS e Linux | llvm-mingw su Windows; `zig cc` e GNU make su macOS e Linux (D-112); MB di Windows, su Unix circa 20 |
| | `ruby` | Ruby | no | 270 | | RubyInstaller su Windows (nativo anche su ARM64), conda-forge su macOS e Linux (D-112), non rilocabile lì |
| | `r` | R (conda-forge) | no | 1100 | | non disponibile su Windows ARM64 (`noArm64`: conda-forge non ha `win-arm64`); nativo su macOS e Linux, ARM64 compreso, non rilocabile lì |
| database (`databases`) | `mysql` | MySQL | no | 520 | | **nuovo** (D-110); l'unico server SQL del kit (D-117), linee 8.4 e 9.7 LTS; x64 in emulazione su Windows ARM64; su Linux ARM64 il download è di circa 940 MB |
| | `mongodb` | MongoDB, mongosh, Database Tools | no | 570 | | **nuovo** (D-73); Database Tools per le copie (D-78); x64 in emulazione su Windows ARM64; su Linux il server viene da conda-forge e non è rilocabile |
| | `phpmyadmin` | phpMyAdmin | no | 60 | `php`, `mysql` | **nuovo** (D-107, al posto di Adminer di D-85); MySQL nel browser, sui sei target (D-117) |
| | `sqlite` | SQLite (sqlite3) | no | 15 | | **nuovo** (D-73); su Linux ARM64 da conda-forge, non rilocabile |

I gruppi, i predefiniti e le dimensioni sono quelli di `components.json`. Proposta del primo avvio: obbligatori, editor, tre agenti, Python, PHP, Apache, più Git che arriva con Claude Code: circa 1,8 GB (1845 MB, di cui 250 dell'editor). Su macOS e Linux Apache non c'è: 1800 MB.

Tolti dal catalogo il 14/10/2026: Windows Terminal (`terminal`), perché le console erano la console classica di Windows (DECISIONS.md D-57), poi tornato lo stesso giorno come terminale predefinito delle console (D-68) e tolto di nuovo il 16/10/2026 (D-95), con il gruppo `os` che conteneva solo lui; VSCodium desktop (`vscodium`), perché l'editor è integrato. Il componente `conpty` proposto in D-47 non è stato aggiunto.

Campi del catalogo oltre a quelli di Atollo: `keys` (le altre parti di un componente: Maven e Gradle di Java, Ninja di CMake, Composer di PHP, yarn di pnpm, `node.exe` dell'editor; installato solo se ci sono tutte), `remove` (altri file posseduti; `tools\vcruntime` di PHP e Apache se ne va solo con l'ultimo dei due), `check` (comando della verifica finale, per tutti), `version` e `sha256` (gruppo `app`). I file chiave si scrivono con i nomi di Windows e si traducono per macOS e Linux (`translate_rel`: `git/bin/bash.exe` → `git/bin/git`, `.exe` e `.cmd` tolti).

**Disponibilità per target** (D-108). Campi facoltativi:

| Campo | Cosa dice | Esempio |
| --- | --- | --- |
| `targets` | i soli target dove il componente c'è; vuoto = tutti | nessuno oggi (Apache fino a D-116: `["win32-x64", "win32-arm64"]`) |
| `unavailable` | target → motivo (in italiano, tradotto dai cataloghi), mostrato al posto di «non disponibile per `<target>`» | Android: `linux-arm64` |
| `noArm64` | nessuna build gira su Windows ARM64, nemmeno in emulazione | R |
| `x64Only` | build x64 in emulazione solo su Windows ARM64 | PHP, Apache, Android SDK, Julia, TinyTeX, MySQL, MongoDB, editor |
| `rosetta` | build x64 su `darwin-arm64` con Rosetta 2; senza Rosetta il componente non è disponibile | nessuno oggi |
| `needsByTarget` | bisogni in più su un target, oltre a `needs`; `components.txt` salva solo quelli validi ovunque | `cc`: `zig` su macOS e Linux; `rust`: `cc` su `win32-arm64` |
| `removeUnix` | i file posseduti su macOS e Linux, al posto di `remove` | pnpm: `bin/pnpm`, `lib/node_modules/corepack` |
| `sha256ByTarget` | SHA-256 per target, con `version`, dove il fornitore non pubblica un hash | editor |
| `sha256ByFile` | SHA-256 per nome di file | PHP: gli archivi di static-php-cli |
| `sha256ByLine` | linea → `{ version, sha256ByTarget }`: si offrono solo le linee fissate | MySQL 8.4 e 9.7, Apache 2.4 su macOS e Linux |

Un componente non disponibile per il sistema si mostra «non disponibile per `<target>`» con il motivo, mai con un binario di un'altra architettura; «non disponibile su PC ARM64» resta solo per Windows ARM64 (BUG-24c). `Atollo components --target <id>` mostra il catalogo di un altro target.

**Fonti per sistema** (D-112). Dove la tabella del §2 dice Windows, gli altri sistemi usano:

| Componente | macOS | Linux | Verifica |
| --- | --- | --- | --- |
| Git | `desktop/dugite-native` (`macOS-arm64`, `macOS-x64`) | dugite-native (`ubuntu-x64`, `ubuntu-arm64`) | `.sha256` dell'asset; rilocabile |
| PHP | static-php-cli «bulk», `php-<v>-{cli,fpm}-macos-{aarch64,x86_64}.tar.gz` | stesso, `linux-{x86_64,aarch64}` | SHA-256 fissato (`sha256ByFile`) |
| C, C++ | shim su `zig cc` del kit, GNU make da conda-forge | come macOS | indice di Zig; micromamba |
| Ruby | conda-forge `ruby` (3.2 e successive) | come macOS | micromamba; non rilocabile |
| MongoDB | build del fornitore | server da conda-forge; mongosh generico; Database Tools per distribuzione | micromamba, JSON del fornitore |
| SQLite | sqlite.org | sqlite.org su x64, conda-forge `sqlite` su ARM64 | SHA3-256; micromamba |
| MySQL | `cdn.mysql.com`, `mysql-<v>-macos15-{arm64,x86_64}.tar.gz` | `mysql-<v>-linux-glibc2.28-x86_64-minimal.tar.xz`, `-aarch64.tar.xz` | SHA-256 fissato (`sha256ByLine`) dopo la firma GPG di Oracle |
| Caddy | GitHub `caddyserver/caddy`, `caddy_<v>_mac_{arm64,amd64}.tar.gz` | `caddy_<v>_linux_{arm64,amd64}.tar.gz` | `digest`, o SHA-512 di `checksums.txt` |
| Apache | build del kit (`scripts/build-httpd`, D-116), rilascio `httpd-v<v>` di questo repository, `httpd-<v>-darwin-{arm64,x64}.tar.gz` | `httpd-<v>-linux-{x64,arm64}.tar.gz` (glibc 2.35) | SHA-256 fissato (`sha256ByLine`) dopo la verifica GPG dei sorgenti; rilocabile; disponibile quando il primo rilascio è fissato |
| Antigravity | manifesto di Google, `.tar.gz` (dalla 1.4) | come macOS | SHA-512 del manifesto |
| R | conda-forge `r-base` | come macOS | micromamba; non rilocabile (D-92) |

Su Windows ARM64: MySQL in emulazione x64 (stesso archivio di Windows x64), Caddy nativo, Rust con l'host `aarch64-pc-windows-gnullvm` e llvm-mingw `ucrt-aarch64` di `cc` come linker, micromamba nativo (`micromamba-win-arm64`). Apache resta Apache Lounge x64 in emulazione: non c'è un Apache né un PHP nativo per Windows ARM64 (D-116).

**Componenti conda non rilocabili** (D-92). Un ambiente conda ha il suo percorso scritto dentro e non si sposta: quando il kit cambia cartella (`root-<os>.txt`), i componenti non rilocabili (`Installer::relocatable(target)` falso: R, Ruby, MongoDB su Linux, SQLite su Linux ARM64 e C/C++ fuori da Windows) ricevono `data/app/versions/<target>/<id>.reinstall`, ogni loro linea salvata `<id>@<linea>.reinstall`, e l'installazione automatica all'avvio li reinstalla: dagli archivi conservati in `data/app/conda/<target>/` (`pkgs/` e l'elenco esplicito `locks/<id>@<linea>.txt` con il suo `.json`), senza rete, dopo il controllo dello SHA-256 di ogni archivio; dalla rete come prima se qualcosa manca o è diverso (D-118). Git di dugite-native segue il kit. Fuori da Windows la radice si confronta tenendo conto di maiuscole e minuscole: `/media/u/Kit` → `/media/u/kit` è uno spostamento.

## 2. Come si installa ciascuno

Abbreviazioni:
- **IA**: installazione da archivio (`Install-Archive`, `_languages.ps1:43-68`). Lettura della versione, salto se aggiornato, download, verifica (SHA-512 se disponibile, altrimenti SHA-256; hash mancante = errore), estrazione, sostituzione della cartella. `inner`: `''` = archivio così com'è, `*` = unica cartella di primo livello, altrimenti nome della sottocartella. Il file chiave (e le parti di `keys` nella stessa cartella) deve esistere prima dello scambio.
- **GHA**: `api.github.com/repos/<repo>/releases/latest`; prima asset che corrisponde alla regex; SHA-256 da `asset.digest`, e per i rilasci più vecchi (senza `digest`) dal file di checksum del rilascio: `<asset>.sha256` (Kotlin), `<nome>.sha256sum` (Deno), `SHASUMS256.txt` (Bun); versione = tag senza `v` / `bun-v`.
- **Versioni** (componenti `versioned`, D-80): l'elenco delle `major.minor` scaricabili e l'ultima build di ognuna vengono dalla stessa fonte dell'ultima versione: per GHA gli ultimi 100 rilasci non prerelease (`releases?per_page=100`); Julia da `versions.json` (solo `stable` con zip `winnt`); Dart dalle cartelle di `channels/stable/release/` del bucket `dart-archive` (API JSON di Google Cloud Storage, da Dart 2); MySQL dalle linee fissate nel catalogo; MongoDB da `current.json` (su Linux dalle versioni di conda-forge); PHP fuori da Windows dalle linee fissate in `sha256ByFile` (8.4, 8.5); Ruby fuori da Windows da conda-forge (3.2 e successive). Si offre solo una versione verificabile: un rilascio senza SHA-256 (né `digest` né file di checksum: Typst fino a 0.13, RubyInstaller 3.1) non compare. Apache Lounge pubblica solo l'ultima build di Apache 2.4: Apache ha una sola versione. All'avvio l'app aggiorna in background gli elenchi mancanti o più vecchi di un giorno (`version-lines.json`), salvo con `ATOLLO_NO_UPDATE`. Verifica dal vivo di tutte le fonti: `cargo test live_lines -- --ignored --nocapture` (con `GITHUB_TOKEN`).
- **Cartella per linea** (D-80, D-91): i componenti `versioned` e `minorFolder` stanno in `A\<cartella>\<major.minor>\` (`A\git\2.56`, `A\node\22.11`); la tabella scrive la cartella del componente, i file chiave sono relativi a quella della linea. Una patch sostituisce la sua linea; per i `minorFolder` una linea nuova toglie la vecchia.
- **Lista**: righe `^([0-9a-f]{64})\s+\*?(\S+)\s*$`, nome del file esatto.

| Id | Versione | Download (x64 / ARM64) | Verifica | Estrazione e passi successivi | File chiave | Origine |
| --- | --- | --- | --- | --- | --- | --- |
| claude | npm `@anthropic-ai/claude-code-win32-<arch>/latest` | `dist.tarball`, nativo per entrambe | integrità npm `sha512-…`, poi Authenticode `Anthropic` su `claude.exe` (avviso se la revoca non è verificabile) | `package/claude.exe` → `A\claude\claude.exe`, vecchio rinominato `.old` | `claude/claude.exe` | `_prepare-core.ps1:10-37` |
| codex | npm `@openai/codex/latest`, poi `<ver>-win32-<arch>` | `dist.tarball`, nativo | integrità npm | `package/vendor/` con una sola cartella che contiene `bin/codex.exe` → `A\codex` | `codex/bin/codex.exe` | `_prepare-core.ps1:45-72` |
| antigravity | manifesto `antigravity-cli-auto-updater-974169037036.us-central1.run.app/manifests/windows_amd64.json` (`windows_arm64.json`) | `url` del manifesto | SHA-512 del manifesto (obbligatorio) | exe singolo → `A\antigravity\agy.exe`, vecchio rinominato `.old`; non eseguire `agy install` | `antigravity/agy.exe` | `_prepare-core.ps1:81-104` |
| git | GHA `git-for-windows/git`, asset `PortableGit-<v>-(64-bit\|arm64).7z.exe`; versione dal nome dell'asset | asset del rilascio | SHA-256 da `digest`; senza API di GitHub l'installazione fallisce (nessun ripiego) | 7z SFX (in Rust: estrazione 7z diretta, nessuna finestra); controllo `bin\bash.exe`; poi `post-install.bat` nella cartella (lo lancerebbe l'SFX; si cancella da solo, quindi esce sempre con 1) e configurazione Git (§3). **macOS/Linux**: GHA `desktop/dugite-native`, `dugite-native-v<v>-<sha>-(macOS-arm64\|macOS-x64\|ubuntu-x64\|ubuntu-arm64).tar.gz`, `.sha256` dell'asset; link simbolici conservati; preparato a parte se il PC non può eseguire il target; `.gitconfig` portabile; segue il kit (D-112) | `git/bin/bash.exe` (Unix: `git/bin/git`) | `_prepare-core.ps1:109-158` |
| python | `raw.githubusercontent.com/astral-sh/python-build-standalone/latest-release/latest-release.json` → `SHA256SUMS`, file `cpython-3.13.N+…-<triple>-install_only_stripped.tar.gz` (ultimo) | triple `x86_64-pc-windows-msvc` / `aarch64-pc-windows-msvc` | SHA-256 da `SHA256SUMS` | tar.gz `python\`; pacchetti conservati (§3); `python.exe` → copia `python3.exe`; pip offline da `Lib\ensurepip\_bundled\pip-*.whl` | `python/python.exe` | `_prepare-core.ps1:163-234` |
| node | `nodejs.org/dist/index.json`, primo LTS con `win-<arch>-zip` | `nodejs.org/dist/<v>/node-<v>-win-<arch>.zip` | SHA-256 da `SHASUMS256.txt` (Lista) | zip `node-<v>-win-<arch>`; versione con la `v` iniziale | `node/node.exe` | `_prepare-core.ps1:239-268` |
| uv | GHA `astral-sh/uv` | `uv-<triple>.zip` | SHA-256 da `digest` | `uv.exe`, `uvx.exe`, `uvw.exe` → `A\uv\<linea>\` | `uv/uv.exe` | `_prepare-core.ps1:273-302` |
| gh | GHA `cli/cli` | `gh_<v>_windows_amd64.zip` / `_arm64.zip` | SHA-256 da `digest` | zip (`bin\gh.exe`) → `A\gh\<linea>\` | `gh/bin/gh.exe` | `_prepare-core.ps1:307-335` |
| vscodium-server | fissata nel catalogo (`version`, oggi 1.135.06055; D-58), mai l'ultima | `github.com/VSCodium/vscodium/releases/download/<v>/vscodium-reh-web-win32-x64-<v>.tar.gz`, solo x64 (anche su ARM64) | SHA-256 fissato nel catalogo (`sha256`, uguale a quello di VSCodium e al `digest` di GitHub) | tar.gz (radice: `bin\`, `node.exe`, `out\`, `extensions\`) → `A\vscodium-server\`; prima dello scambio si fermano i processi del vecchio editor | `vscodium-server/out/server-main.js` + `vscodium-server/node.exe` | nuovo |
| pwsh | GHA `PowerShell/PowerShell`, `PowerShell-<v>-win-(x64\|arm64).zip` | nativo per entrambe | SHA-256 da `digest` | zip → `A\pwsh\` | `pwsh/pwsh.exe` | nuovo |
| php | `windows.php.net/downloads/releases/releases.json`, ramo più recente (o scelto), build Thread Safe `ts-vs*-x64` (o `ts-vc*-x64`) | solo x64 (emulato su ARM64, con avviso) | SHA-256 di `releases.json` | zip; DLL VC++ accanto a `php.exe`; `cacert.pem` da curl.se con SHA-256 (se fallisce, solo avviso); `php.ini` creato una volta (§3); versione salvata insieme a quella di Composer | `php/php.exe` + `tools/composer.phar` | `_prepare-web.ps1:89-167` |
| composer (parte di php) | `getcomposer.org/versions`, primo `stable` | `getcomposer.org<path>`, indipendente dall'architettura | SHA-256 da `<url>.sha256sum` | `A\tools\composer.phar` + `composer.cmd` (`@php "%~dp0composer.phar" %*`) e `composer` per Git Bash | `tools/composer.phar` | `_prepare-web.ps1:153-164` |
| VC++ (per php, apache) | nessuna: estratto una volta in `A\tools\vcruntime\` e riusato finché c'è | `aka.ms/vc14/vc_redist.x64.exe` | Authenticode `O=Microsoft Corporation` obbligatorio, sull'exe e su `vcruntime140.dll` | CAB incorporati (firma `MSCF`) estratti con `expand.exe` di Windows, manifesto Burn, payload `vcRuntimeMinimum_amd64\cab1.cab`; `*.dll_amd64` → `*.dll` in `A\tools\vcruntime\` | `tools/vcruntime/vcruntime140.dll` | `_prepare-web.ps1:13-83` |
| apache | HTML di `www.apachelounge.com/download/`, primo `httpd-X.Y.Z-N-Win64-VSnn.zip`. **macOS/Linux** (D-116): linee fissate in `sha256ByLine` (`2.4` → `2.4.69-<build>`), `github.com/nabre/atollo/releases/download/httpd-v<v>/httpd-<v>-<target>.tar.gz` | solo Win64 (x64 anche su ARM64); Unix nativo per i quattro target | SHA-256 da `<url>.txt`; Unix: SHA-256 fissato | zip `Apache24`; DLL VC++ in `bin\`; `httpd.conf`: `SRVROOT`, `Listen 80` commentato, `rewrite_module`, `Include …/data/apache/kit.conf`; `kit.conf` creato una volta (§3). Unix: tar.gz con la radice di Apache al primo livello e lo stesso `httpd.conf` (fine riga LF); `kit.conf` senza PHP | `apache/bin/httpd.exe` (Unix: `apache/bin/httpd`) | `_prepare-web.ps1:174-276` |
| rust | `static.rust-lang.org/dist/channel-rust-stable.toml` (`[pkg.rustc]`); installatore `static.rust-lang.org/rustup/dist/<arch>-pc-windows-msvc/rustup-init.exe` | host: x64 → `x86_64-pc-windows-gnu`, ARM64 → `aarch64-pc-windows-gnullvm` (linker: clang di llvm-mingw, componente `cc`); `msvc` dalle impostazioni su entrambi | SHA-256 da `rustup-init.exe.sha256` | `-y --no-modify-path --profile minimal --default-host <host> --default-toolchain stable -c rustfmt -c clippy`, `RUSTUP_HOME`, `CARGO_HOME` nel kit; `rustup set auto-self-update disable`; se già installato, `rustup update stable` sul posto; voce Rustup di HKCU (§3). Kit su FAT32/exFAT: errore. Kit di un altro sistema o architettura: `rustup-init` si scarica e si verifica nella cartella `.bootstrap` e l'installazione si completa al primo avvio sul target (D-114) | `rust/cargo/bin/rustup.exe` | `_prepare-rust.ps1:13-198` |
| go | `go.dev/dl/?mode=json`, primo `stable` | file `windows`/`amd64\|arm64`/archive | SHA-256 del JSON | IA `go` | `go/bin/go.exe` | `_languages.ps1:77-89` |
| java | `api.adoptium.net/v3/info/available_releases` → LTS; `assets/latest/<lts>/hotspot?architecture=x64\|aarch64&image_type=jdk&os=windows&vendor=eclipse` | primo `.zip` | SHA-256 `binary.package.checksum` | IA `*` | `java/bin/java.exe` + `maven/bin/mvn.cmd` + `gradle/bin/gradle.bat` | `_languages.ps1:94-105` |
| maven (parte di java) | `repo.maven.apache.org/…/apache-maven/maven-metadata.xml`, versione numerica più alta | `apache-maven-<v>-bin.zip` | SHA-512 da `.sha512` | IA `apache-maven-<v>`; `conf\settings.xml`: `<localRepository>${env.CP_DATA}/maven/repository</localRepository>` | `maven/bin/mvn.cmd` | `_languages.ps1:107-120` |
| gradle (parte di java) | `services.gradle.org/versions/current` | `downloadUrl` | SHA-256 da `checksumUrl` | IA `gradle-<v>` | `gradle/bin/gradle.bat` | `_languages.ps1:122-125` |
| kotlin | GHA `JetBrains/kotlin`, `kotlin-compiler-*.zip` | indipendente | `digest` | IA `kotlinc` | `kotlin/bin/kotlinc.bat` | `_languages.ps1:130-136` |
| android | pagina `developer.android.com/studio`: `commandlinetools-win-<build>_latest.zip` da `dl.google.com/android/repository/` | indipendente | SHA-256 della pagina (il repository XML ha solo SHA-1) | in `android\cmdline-tools\latest`, poi `sdkmanager --licenses`, `--install platform-tools`, ultima `platforms;android-N` e `build-tools;X.Y.Z` stabili, `--update` agli aggiornamenti (Java 17+ del kit) | `android/cmdline-tools/latest/bin/sdkmanager.bat` + `android/platform-tools/adb.exe` | **nuovo** |
| dotnet | `builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json`, canale LTS attivo → `latest-sdk` → `releases.json` | `rid=win-x64\|win-arm64`, `.zip` | SHA-512 `hash` | IA `''` | `dotnet/dotnet.exe` | `_languages.ps1:139-157` |
| dart | `storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION` | `dartsdk-windows-<x64\|arm64>-release.zip` | SHA-256 da `.sha256sum` | IA `dart-sdk` | `dart/bin/dart.exe` | `_languages.ps1:160-168` |
| zig | `ziglang.org/download/index.json`, versione numerica più alta | `x86_64-windows` / `aarch64-windows` | SHA-256 `shasum` | IA `*` | `zig/zig.exe` | `_languages.ps1:171-181` |
| julia | `julialang-s3.julialang.org/bin/versions.json`, ultima `stable` | `winnt`/`x86_64\|aarch64`/zip | SHA-256 | IA `*` | `julia/bin/julia.exe` | `_languages.ps1:184-195` |
| bun | GHA `oven-sh/bun`, `bun-windows-(x64\|aarch64).zip` | | `digest` | IA `*` | `bun/bun.exe` | `_languages.ps1:198-205` |
| deno | GHA `denoland/deno`, `deno-(x86_64\|aarch64)-pc-windows-msvc.zip` | | `digest` | IA `''` | `deno/deno.exe` | `_languages.ps1:206-213` |
| typst | GHA `typst/typst`, `typst-(x86_64\|aarch64)-pc-windows-msvc.zip` | | `digest` | IA `*` | `typst/typst.exe` | `_languages.ps1:216-223` |
| tinytex | GHA `rstudio/tinytex-releases`, `TinyTeX-1-windows-v*.exe` | x64 anche su ARM64 | `digest` | 7z SFX → `TinyTeX\` con `bin\windows\tlmgr.bat` | `tinytex/bin/windows/tlmgr.bat` | `_languages.ps1:228-250` |
| pnpm | registro npm, `pnpm/latest` e `corepack/latest` | registro npm | integrità di npm | con il Node del kit (`node npm-cli.js --global`, senza `cmd.exe`), prefisso `data\npm-global`: `install --ignore-scripts corepack@latest`, `corepack enable yarn`, poi `install --force --allow-scripts=pnpm pnpm@latest` | `@data/npm-global/pnpm.cmd` + `@data/npm-global/yarn.cmd` | `_languages.ps1:254-281` |
| cmake | GHA `Kitware/CMake`, `cmake-*-windows-(x86_64\|arm64).zip` | | `digest` | IA `*` | `cmake/bin/cmake.exe` + `tools/ninja.exe` | `_languages.ps1:286-291` |
| ninja (parte di cmake) | GHA `ninja-build/ninja`, `ninja-win.zip` / `ninja-winarm64.zip` | | `digest` | `ninja.exe` → `A\tools\` | `tools/ninja.exe` | `_languages.ps1:292-298` |
| cc | GHA `mstorsjo/llvm-mingw`, `llvm-mingw-*-ucrt-(x86_64\|aarch64).zip`; macOS/Linux: shim `cc`, `c++`, `gcc`, `g++`, `clang`, `clang++`, `ar`, `ranlib` sullo Zig del kit e GNU make da conda-forge (D-112; i framework Apple vogliono Xcode CLT) | | `digest`; micromamba | IA `*`; **eliminare** `bin\*-w64-mingw32-gcc.exe` e `-g++.exe` (romperebbero il linker di Rust `gnu`) | `cc/bin/clang.exe` | `_languages.ps1:308-317` |
| ruby | `api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=30`, esclusi prerelease/bozze, versione e build più alte | `rubyinstaller-X.Y.Z-N-(x64\|arm).7z` | `digest` | IA `*`; versione `X.Y.Z-N`; senza MSYS2 | `ruby/bin/ruby.exe` | `_languages.ps1:321-340` |
| micromamba (per r; su macOS e Linux per ruby e make; su Linux per il server di mongodb; su Linux ARM64 per sqlite) | tag dell'ultima release di `mamba-org/micromamba-releases` (API di GitHub) | `micromamba-win-64`, `micromamba-win-arm64` (`micromamba-osx-*`, `micromamba-linux-*` fuori da Windows) | SHA-256 da `micromamba-win-64.sha256` della release | exe → `A\tools\micromamba.exe`; anche la reinstallazione senza rete usa questo (`create --offline -f <elenco>`, D-118) | `tools/micromamba.exe` | `_languages.ps1:344-358` |
| r | `api.anaconda.org/package/conda-forge/r-base` → `latest_version` | solo `win-64` (Windows ARM64: non disponibile) | SHA-256 di ogni pacchetto (micromamba) | `micromamba create --no-rc -y -q -p <lavoro>\r -c conda-forge --override-channels r-base=<v> "libblas=*=*openblas"` con profilo, `MAMBA_ROOT_PREFIX` e `CONDA_PKGS_DIRS` nell'area di lavoro; per i componenti che non seguono il kit (fuori da Windows) archivi ed elenco esplicito della linea in `data/app/conda/<target>/`, per reinstallare senza rete (D-118); lanciatori `bin\R.cmd`, `bin\Rscript.cmd`; CRAN `cloud.r-project.org` in `Rprofile.site`; via `pkgs\` | `r/bin/R.cmd` | `_languages.ps1:368-429` |
| mysql | linee e versioni fissate nel catalogo (`sha256ByLine`: 8.4.11, 9.7.2) | `cdn.mysql.com/Downloads/MySQL-<linea>/mysql-<v>-winx64.zip` (x64 anche su ARM64) | SHA-256 fissato, calcolato dopo la firma GPG di Oracle | IA; via `mysql-test`, simboli e build di debug; su Linux `lib/private/libaio.so.1` verso `libaio.so.1t64` se serve | `mysql/bin/mysqld.exe` + `mysql.exe`, `mysqldump.exe`, `mysqladmin.exe` | nuovo (D-110) |
| caddy | GHA `caddyserver/caddy` | `caddy_<v>_windows_(amd64\|arm64).zip` | `digest`, o SHA-512 di `caddy_<v>_checksums.txt` | `caddy.exe` → `A\caddy\<linea>\` | `caddy/caddy.exe` | nuovo (D-111) |
| mongodb | `downloads.mongodb.org/current.json`, produzione `x.0.y`; mongosh da GHA `mongodb-js/mongosh` | `mongodb-windows-x86_64-<v>.zip`, `mongosh-<v>-win32-x64.zip` in `mongosh\`, Database Tools da `release.json` in `mongodb-tools\` | SHA-256 dell'elenco; `digest` | IA singola; via `.pdb` e `vc_redist`, VC++ del kit | `mongodb/bin/mongod.exe` + `mongosh/bin/mongosh.exe` + `mongodb-tools/bin/mongodump.exe` | nuovo (D-73) |
| sqlite | `sqlite.org/download.html`, righe `PRODUCT,` | `sqlite-tools-win-(x64\|arm64)-<n>.zip`, sempre quello dell'architettura del kit (D-108) | **SHA3-256** della pagina | IA | `sqlite/sqlite3.exe` | nuovo (D-73) |
| phpmyadmin | `www.phpmyadmin.net/home_page/version.json` | `files.phpmyadmin.net/phpMyAdmin/<v>/phpMyAdmin-<v>-all-languages.zip` (tutte le lingue) | file `.sha256` accanto allo zip | zip → `phpmyadmin\<linea>\`, senza `setup\`; `config.inc.php` scritto dall'app a ogni avvio; controllo `php -l` su `libraries/classes/Version.php` col PHP del kit | `phpmyadmin/index.php` | nuovo (D-107) |
| CodeLLDB (estensione) | GHA `vadimcn/codelldb` | `codelldb-win32-<arch>.vsix`; x64 anche su ARM64, come il server dell'editor | `digest` obbligatorio | installata nel server dell'editor con `--install-extension`, se un progetto ha Rust, Zig, C o C++ (matrice `extensions.json`, D-71), una volta sola | cartella `vadimcn.vscode-lldb-*` in `data\vscodium-server\extensions\` | `_vscodium-extensions.ps1:162-206` |

## 3. Regole trasversali

- **Ordine**: quello di `components.json`: claude, codex, antigravity, git, python, node, uv, gh, php, apache, caddy, rust, go, java (+maven, gradle), kotlin, android, dotnet, dart, zig, julia, bun, deno, typst, tinytex, pnpm, cmake (+ninja), cc, ruby, r, mysql, mongodb, phpmyadmin, sqlite, pwsh, vscodium-server. Prima le rimozioni, poi le installazioni; configurazione Git e sito Apache sono passi finali di quei componenti. Da riga di comando, in fondo a `prepare`, la verifica finale di ciò che è stato installato. Estensioni dell'editor dopo l'avvio dell'editor; allineamento degli agenti all'avvio dell'app e intorno a ogni agente. (In nabre/atollo: `Prepare-Atollo.ps1:250-285`.)
- **Un errore non ferma gli altri**: ogni passo è isolato e finisce nel riepilogo; esito complessivo "con errori" se uno fallisce.
- **Quando si installa**: file chiave o una parte di `keys` mancante, editor a una versione diversa da quella del rilascio (all'avvio dell'app, o in un `prepare` che lo comprende), oppure "Aggiorna tutto" (`--update-all`) / `--force`. `--only` installa o aggiorna i componenti elencati e ciò che serve loro e manca, obbligatori esclusi, senza rimuovere nulla; `--skip` salta anche ciò che ne dipende e non è ancora installato (mai gli obbligatori).
- **Quando serve** (D-59): avvio di un agente, clone, azione di un progetto installano prima i componenti che servono e li segnano come scelti in `components.txt`.
- **Parametri predefiniti**: Claude canale `latest`; Python: l'ultima linea minore stabile, salvo `pythonVersion` nelle Impostazioni (`^3\.\d+$`, D-81). `PYTHON_DEFAULT` (3.13) vale solo quando l'elenco delle linee non si legge: un kit nuovo ha 3.15 da ottobre 2026; PHP ramo più recente (`^\d+\.\d+$`); Rust `gnu` su Windows x64 e `gnullvm` su Windows ARM64; architettura dell'host. Nell'app: pagina Impostazioni › Versioni.
- **VC++** estratto una volta in `tools\vcruntime\` e riusato; DLL copiate accanto a `php.exe` e in `apache\bin` (distribuzione locale permessa da Microsoft); x64 anche su ARM64. La cartella resta finché c'è PHP o Apache.
- **File dell'utente creati una sola volta e mai sovrascritti**: `data\php\php.ini` (da `php.ini-development`: `extension_dir`, estensioni curl, fileinfo, gd, intl, mbstring, mysqli, openssl, pdo_mysql, pdo_sqlite, sodium, sqlite3, zip; `date.timezone = Europe/Rome`; `curl.cainfo` e `openssl.cafile` su `cacert.pem`), `data\apache\kit.conf` (porta 8080 su `127.0.0.1`, sito in `data\apache\htdocs`, PHP via `<IfFile>`, `conf.d\*.conf`), `data\home\.gitconfig`, `htdocs\index.php`. Testi in UTF-8 senza BOM. `httpd.conf` invece si riscrive a ogni installazione di Apache.
- **Python**: prima di aggiornare, `pip freeze` in `data\python-packages-<vecchia>.txt`; stessa versione minore → copia di `Lib\site-packages` e `Scripts` (escluso pip); minore diversa → solo l'elenco.
- **Rust**: `gnu` predefinito su Windows x64 (porta il proprio linker); `gnullvm` predefinito su Windows ARM64, collegato da llvm-mingw di `cc` (D-112); `msvc` dalle impostazioni su entrambi (richiede Visual Studio); `rustup-init` sempre build msvc. Senza collegamenti fisici (FAT32/exFAT) l'installazione si ferma con un errore; il ripiego degli script originali (toolchain sul disco del PC, copia nel kit di `rustup\`) non è portato.
- **Voce Rustup in HKCU** (`…\Uninstall\Rustup`): rustup la scrive sempre; si salva prima e si ripristina o si rimuove dopo (solo se punta al kit). È una scrittura temporanea nel registro **fatta da rustup, non dall'app** (DECISIONS.md D-38).
- **Configurazione Git**: `credential.helper` di sistema impostato a `manager` se è vuoto, `helper-selector` o un percorso di GCM; `.gitconfig` con `init.defaultBranch=main` e `safe.directory=*` (FAT32/exFAT non registrano il proprietario); `GCM_CREDENTIAL_STORE=dpapi` nell'ambiente.
- **Firma**: Authenticode con editore atteso; errore di revoca non verificabile = avviso, salvo dove la firma è l'unica verifica (VC++).
- **Scambi atomici**: il nuovo albero si prepara in `data\tmp\setup\<id>` (stesso disco), la cartella vecchia viene rinominata `<nome>.old-<ts>` (fallisce subito se un file è in uso), poi spostamento, ripristino in caso di errore; avanzi `.old-*`/`.new-*` cancellati all'installazione successiva. Exe singoli: vecchio rinominato `.old`, così si sostituisce anche se in uso.
- **FAT32/exFAT**: estrazione senza ripristinare le date dei file; prova dei collegamenti fisici prima di Rust; area di lavoro sempre in `data\tmp\setup\`.
- **Proxy**: impostazioni dell'app → `HTTPS_PROXY`/`HTTP_PROXY` → proxy di Windows (sola lettura); niente PAC (D-29). Le impostazioni finiscono anche nell'ambiente del kit (`HTTPS_PROXY`, `HTTP_PROXY`, `NO_PROXY`), che ricevono rustup, pip e npm. Download in `.part`, rinominato a fine download, 3 tentativi (5 s, 10 s).
- **Una preparazione alla volta** per kit (mutex sul percorso della radice).
- **Versioni come stringhe**: node con la `v`; Go senza `go`; GHA senza `v`/`bun-v`; ruby `X.Y.Z-N`; componenti con più parti uniscono le versioni (`8.5.0 (Composer 2.10.3)`). I confronti con `data\app\versions\` usano le stesse stringhe.

## 4. Differenze dagli script originali

| Script originali | Nuovo kit | Motivo |
| --- | --- | --- |
| VSCodium desktop obbligatorio | tolto | editor integrato |
| Git, Node.js, uv obbligatori | installati quando servono (DECISIONS.md D-59) | obbligatori solo PowerShell 7 e GitHub CLI |
| — | `pwsh` obbligatorio; `vscodium-server` parte dell'app con versione fissata (D-58) | terminale esclusivo del kit (console classica di Windows, D-57, D-95), editor integrato |
| Git: ripiego senza API (tag + Authenticode) | solo API di GitHub con `digest` | — |
| Git SFX e TinyTeX SFX aprono la finestra di 7-Zip | estrazione 7z in Rust, nessuna finestra | interfaccia pulita |
| Typst e TinyTeX non vengono rimossi quando si tolgono | rimossi | bug di `_prepare-migrate.ps1:128` |
| Rust su FAT32/exFAT: toolchain sul disco del PC | errore | da portare |
| `Test-Installation` con `rustc --version` ecc. | stessa verifica: `Atollo.exe check` e in fondo a `prepare` | |
| Versioni in `data\versions\` | `data\app\versions\` | tutto ciò che è dell'app sta in `data\app\` |
