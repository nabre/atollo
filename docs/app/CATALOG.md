# Atollo App: catalogo di agenti, linguaggi e strumenti

Fonte: `nabre/atollo` al commit `a61a354` (`setup/components.json` e script di `setup/windows/`). È la specifica che il motore Rust (`setup/catalog.rs`, `install/`) segue; il catalogo dell'app è `tauri/catalog/components.json`. I riferimenti `file:riga` puntano agli script originali.

Percorsi nel nuovo kit (PLAN §2): `A` = `data\apps\win32-<arch>\`; versioni in `data\app\versions\<id>.txt`; aree di lavoro in `data\tmp\setup\`.
Prove reali: la CI (job x64) installa `pwsh`, `node`, `git` e l'editor; tutti i componenti sono stati installati e rimossi a mano nella fase 4 (CHANGELOG.md).

## 1. Componenti e gruppi

| Gruppo | Id | Nome | Predefinito | MB (x64) | Dipende da | Note |
| --- | --- | --- | --- | --- | --- | --- |
| obbligatori (`required`) | `gh` | GitHub CLI | sempre | 40 | `git` | pull request, issue, accesso a GitHub (anche per i clone privati) |
| | `pwsh` | PowerShell 7 | sempre | 110 | | **nuovo**; terminale del kit |
| app (`app`) | `vscodium-server` | Editor (server web di VSCodium) | sempre | 250 | `pwsh` | **nuovo**; parte dell'app, versione fissata a ogni rilascio (DECISIONS.md D-58); in Linguaggi e strumenti compare con gli obbligatori, spuntato e non modificabile; x64 anche su ARM64 |
| agenti (`agents`) | `claude` | Claude Code | sì | 250 | `git` | |
| | `codex` | Codex | sì | 440 | | |
| | `antigravity` | Antigravity | sì | 190 | | accesso salvato nel kit (`agents/keyring.rs`) |
| linguaggi e strumenti (`languages`) | `git` | Git (Git Bash) | sempre (per GitHub CLI) | 350 | | shell dei comandi degli agenti; dipendenza obbligatoria di GitHub CLI (D-59) |
| | `python` | Python + pip | sì | 70 | | 3.13 predefinito |
| | `node` | Node.js LTS + npm | no | 110 | | quando serve: azioni con `npm`, server MCP con `npx` (D-59) |
| | `uv` | uv, uvx | no | 35 | | quando serve: azioni con `uv`, server MCP con `uvx` (D-59) |
| | `php` | PHP + Composer | sì | 100 | | con VC++ locale; x64 anche su ARM64 |
| | `apache` | Apache | sì | 45 | `php` | x64 anche su ARM64 |
| | `rust` | Rust | no | 920 | | toolchain `gnu` su x64 |
| | `go` | Go | no | 250 | | |
| | `java` | Java (Temurin, Maven, Gradle) | no | 300 | | |
| | `kotlin` | Kotlin | no | 80 | `java` | |
| | `android` | Android SDK | no | 600 | `java` | **nuovo** (D-84); strumenti a riga di comando, platform-tools, piattaforma e build-tools, senza emulatore; sceglierlo accetta le licenze dell'SDK di Google; x64 anche su ARM64 |
| | `dotnet` | .NET SDK | no | 250 | | |
| | `dart` | Dart | no | 200 | | |
| | `zig` | Zig | no | 200 | | |
| | `julia` | Julia | no | 500 | | |
| | `bun` | Bun | no | 90 | | |
| | `deno` | Deno | no | 100 | | |
| | `typst` | Typst | no | 50 | | |
| | `tinytex` | TinyTeX | no | 280 | | x64 anche su ARM64 |
| | `pnpm` | pnpm, yarn | no | 10 | `node` | in `data\npm-global\` |
| | `cmake` | CMake, Ninja | no | 100 | | |
| | `cc` | C, C++ (llvm-mingw) | no | 720 | | |
| | `ruby` | Ruby | no | 270 | | |
| | `r` | R (conda-forge) | no | 1100 | | solo x64: non disponibile sui kit ARM64 (`noArm64`) |
| database (`databases`) | `mariadb` | MariaDB | no | 230 | | **nuovo** (D-73); compatibile con MySQL; x64 anche su ARM64 |
| | `mongodb` | MongoDB, mongosh, Database Tools | no | 570 | | **nuovo** (D-73); Database Tools per le copie (D-78); x64 anche su ARM64 |
| | `adminer` | Adminer | no | 1 | `php`, `mariadb` | **nuovo** (D-85); MariaDB nel browser, come phpMyAdmin |
| | `sqlite` | SQLite (sqlite3) | no | 15 | | **nuovo** (D-73) |
| legati a un sistema operativo (`os`) | `terminal` | Windows Terminal | sì | 45 | `pwsh` | **nuovo**; il terminale del kit per le console fuori dall'editor (DECISIONS.md D-68); la console classica di Windows resta nelle Impostazioni |

I gruppi, i predefiniti e le dimensioni sono quelli di `components.json`. Proposta del primo avvio: obbligatori, editor, tre agenti, Python, PHP, Apache, più Git che arriva con GitHub CLI (e con Claude Code): più Windows Terminal: circa 1,9 GB (1890 MB, di cui 250 dell'editor).

Tolti dal catalogo il 14/10/2026: Windows Terminal (`terminal`), perché le console erano la console classica di Windows (DECISIONS.md D-57), poi tornato lo stesso giorno come terminale predefinito delle console (D-68); VSCodium desktop (`vscodium`), perché l'editor è integrato. Il componente `conpty` proposto in D-47 non è stato aggiunto.

Campi del catalogo oltre a quelli del Atollo: `keys` (le altre parti di un componente: Maven e Gradle di Java, Ninja di CMake, Composer di PHP, yarn di pnpm, `node.exe` dell'editor; installato solo se ci sono tutte), `x64Only` (build x64 anche su ARM64, in emulazione), `noArm64` (nessuna build per ARM64), `remove` (altri file posseduti; `tools\vcruntime` di PHP e Apache se ne va solo con l'ultimo dei due), `check` (comando della verifica finale, per tutti; `<file-version>` legge la versione dal file senza avviarlo, per Windows Terminal), `version` e `sha256` (gruppo `app`).

## 2. Come si installa ciascuno

Abbreviazioni:
- **IA**: installazione da archivio (`Install-Archive`, `_languages.ps1:43-68`). Lettura della versione, salto se aggiornato, download, verifica (SHA-512 se disponibile, altrimenti SHA-256; hash mancante = errore), estrazione, sostituzione della cartella. `inner`: `''` = archivio così com'è, `*` = unica cartella di primo livello, altrimenti nome della sottocartella. Il file chiave (e le parti di `keys` nella stessa cartella) deve esistere prima dello scambio.
- **GHA**: `api.github.com/repos/<repo>/releases/latest`; prima asset che corrisponde alla regex; SHA-256 da `asset.digest`, e per i rilasci più vecchi (senza `digest`) dal file di checksum del rilascio: `<asset>.sha256` (Kotlin), `<nome>.sha256sum` (Deno), `SHASUMS256.txt` (Bun); versione = tag senza `v` / `bun-v`.
- **Versioni** (componenti `versioned`, D-80): l'elenco delle `major.minor` scaricabili e l'ultima build di ognuna vengono dalla stessa fonte dell'ultima versione: per GHA gli ultimi 100 rilasci non prerelease (`releases?per_page=100`); Julia da `versions.json` (solo `stable` con zip `winnt`); Dart dalle cartelle di `channels/stable/release/` del bucket `dart-archive` (API JSON di Google Cloud Storage, da Dart 2); MariaDB dalla REST API; MongoDB da `current.json`. Si offre solo una versione verificabile: un rilascio senza SHA-256 (né `digest` né file di checksum: Typst fino a 0.13, nginx fino a 1.27, RubyInstaller 3.1) non compare. Apache Lounge pubblica solo l'ultima build di Apache 2.4: Apache ha una sola versione. All'avvio l'app aggiorna in background gli elenchi mancanti o più vecchi di un giorno (`version-lines.json`), salvo con `ATOLLO_NO_UPDATE`. Verifica dal vivo di tutte le fonti: `cargo test live_lines -- --ignored --nocapture` (con `GITHUB_TOKEN`).
- **Lista**: righe `^([0-9a-f]{64})\s+\*?(\S+)\s*$`, nome del file esatto.

| Id | Versione | Download (x64 / ARM64) | Verifica | Estrazione e passi successivi | File chiave | Origine |
| --- | --- | --- | --- | --- | --- | --- |
| claude | npm `@anthropic-ai/claude-code-win32-<arch>/latest` | `dist.tarball`, nativo per entrambe | integrità npm `sha512-…`, poi Authenticode `Anthropic` su `claude.exe` (avviso se la revoca non è verificabile) | `package/claude.exe` → `A\claude\claude.exe`, vecchio rinominato `.old` | `claude/claude.exe` | `_prepare-core.ps1:10-37` |
| codex | npm `@openai/codex/latest`, poi `<ver>-win32-<arch>` | `dist.tarball`, nativo | integrità npm | `package/vendor/` con una sola cartella che contiene `bin/codex.exe` → `A\codex` | `codex/bin/codex.exe` | `_prepare-core.ps1:45-72` |
| antigravity | manifesto `antigravity-cli-auto-updater-974169037036.us-central1.run.app/manifests/windows_amd64.json` (`windows_arm64.json`) | `url` del manifesto | SHA-512 del manifesto (obbligatorio) | exe singolo → `A\antigravity\agy.exe`, vecchio rinominato `.old`; non eseguire `agy install` | `antigravity/agy.exe` | `_prepare-core.ps1:81-104` |
| git | GHA `git-for-windows/git`, asset `PortableGit-<v>-(64-bit\|arm64).7z.exe`; versione dal nome dell'asset | asset del rilascio | SHA-256 da `digest`; senza API di GitHub l'installazione fallisce (nessun ripiego) | 7z SFX (in Rust: estrazione 7z diretta, nessuna finestra); controllo `bin\bash.exe`; poi `post-install.bat` nella cartella (lo lancerebbe l'SFX; si cancella da solo, quindi esce sempre con 1) e configurazione Git (§3) | `git/bin/bash.exe` | `_prepare-core.ps1:109-158` |
| python | `raw.githubusercontent.com/astral-sh/python-build-standalone/latest-release/latest-release.json` → `SHA256SUMS`, file `cpython-3.13.N+…-<triple>-install_only_stripped.tar.gz` (ultimo) | triple `x86_64-pc-windows-msvc` / `aarch64-pc-windows-msvc` | SHA-256 da `SHA256SUMS` | tar.gz `python\`; pacchetti conservati (§3); `python.exe` → copia `python3.exe`; pip offline da `Lib\ensurepip\_bundled\pip-*.whl` | `python/python.exe` | `_prepare-core.ps1:163-234` |
| node | `nodejs.org/dist/index.json`, primo LTS con `win-<arch>-zip` | `nodejs.org/dist/<v>/node-<v>-win-<arch>.zip` | SHA-256 da `SHASUMS256.txt` (Lista) | zip `node-<v>-win-<arch>`; versione con la `v` iniziale | `node/node.exe` | `_prepare-core.ps1:239-268` |
| uv | GHA `astral-sh/uv` | `uv-<triple>.zip` | SHA-256 da `digest` | `uv.exe`, `uvx.exe`, `uvw.exe` → `A\tools\` | `tools/uv.exe` | `_prepare-core.ps1:273-302` |
| gh | GHA `cli/cli` | `gh_<v>_windows_amd64.zip` / `_arm64.zip` | SHA-256 da `digest` | `gh.exe` → `A\tools\` | `tools/gh.exe` | `_prepare-core.ps1:307-335` |
| vscodium-server | fissata nel catalogo (`version`, oggi 1.135.06055; D-58), mai l'ultima | `github.com/VSCodium/vscodium/releases/download/<v>/vscodium-reh-web-win32-x64-<v>.tar.gz`, solo x64 (anche su ARM64) | SHA-256 fissato nel catalogo (`sha256`, uguale a quello di VSCodium e al `digest` di GitHub) | tar.gz (radice: `bin\`, `node.exe`, `out\`, `extensions\`) → `A\vscodium-server\`; prima dello scambio si fermano i processi del vecchio editor | `vscodium-server/out/server-main.js` + `vscodium-server/node.exe` | nuovo |
| pwsh | GHA `PowerShell/PowerShell`, `PowerShell-<v>-win-(x64\|arm64).zip` | nativo per entrambe | SHA-256 da `digest` | zip → `A\pwsh\` | `pwsh/pwsh.exe` | nuovo |
| terminal | GHA `microsoft/terminal`, `Microsoft.WindowsTerminal_<v>_(x64\|arm64).zip` | nativo per entrambe | SHA-256 da `digest` | zip (unica cartella) → `A\terminal\` + `.portable` | `terminal/WindowsTerminal.exe` | nuovo (D-68) |
| php | `windows.php.net/downloads/releases/releases.json`, ramo più recente (o scelto), build Thread Safe `ts-vs*-x64` (o `ts-vc*-x64`) | solo x64 (emulato su ARM64, con avviso) | SHA-256 di `releases.json` | zip; DLL VC++ accanto a `php.exe`; `cacert.pem` da curl.se con SHA-256 (se fallisce, solo avviso); `php.ini` creato una volta (§3); versione salvata insieme a quella di Composer | `php/php.exe` + `tools/composer.phar` | `_prepare-web.ps1:89-167` |
| composer (parte di php) | `getcomposer.org/versions`, primo `stable` | `getcomposer.org<path>`, indipendente dall'architettura | SHA-256 da `<url>.sha256sum` | `A\tools\composer.phar` + `composer.cmd` (`@php "%~dp0composer.phar" %*`) e `composer` per Git Bash | `tools/composer.phar` | `_prepare-web.ps1:153-164` |
| VC++ (per php, apache) | nessuna: estratto una volta in `A\tools\vcruntime\` e riusato finché c'è | `aka.ms/vc14/vc_redist.x64.exe` | Authenticode `O=Microsoft Corporation` obbligatorio, sull'exe e su `vcruntime140.dll` | CAB incorporati (firma `MSCF`) estratti con `expand.exe` di Windows, manifesto Burn, payload `vcRuntimeMinimum_amd64\cab1.cab`; `*.dll_amd64` → `*.dll` in `A\tools\vcruntime\` | `tools/vcruntime/vcruntime140.dll` | `_prepare-web.ps1:13-83` |
| apache | HTML di `www.apachelounge.com/download/`, primo `httpd-X.Y.Z-N-Win64-VSnn.zip` | solo Win64 | SHA-256 da `<url>.txt` | zip `Apache24`; DLL VC++ in `bin\`; `httpd.conf`: `SRVROOT`, `Listen 80` commentato, `rewrite_module`, `Include …/data/apache/kit.conf`; `kit.conf` creato una volta (§3) | `apache/bin/httpd.exe` | `_prepare-web.ps1:174-276` |
| rust | `static.rust-lang.org/dist/channel-rust-stable.toml` (`[pkg.rustc]`); installatore `static.rust-lang.org/rustup/dist/<arch>-pc-windows-msvc/rustup-init.exe` | host: x64 → `x86_64-pc-windows-gnu` (o `msvc` dalle impostazioni), ARM64 → `aarch64-pc-windows-msvc` | SHA-256 da `rustup-init.exe.sha256` | `-y --no-modify-path --profile minimal --default-host <host> --default-toolchain stable -c rustfmt -c clippy`, `RUSTUP_HOME`, `CARGO_HOME` nel kit; `rustup set auto-self-update disable`; se già installato, `rustup update stable` sul posto; voce Rustup di HKCU (§3). Kit su FAT32/exFAT o PC di un'altra architettura: errore | `rust/cargo/bin/rustup.exe` | `_prepare-rust.ps1:13-198` |
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
| pnpm | npm `pnpm@latest yarn@latest` | registro npm | integrità di npm | `node npm-cli.js install --global --no-fund --no-audit --ignore-scripts` con il Node del kit (senza `cmd.exe`), prefisso `data\npm-global` | `@data/npm-global/pnpm.cmd` + `@data/npm-global/yarn.cmd` | `_languages.ps1:254-281` |
| cmake | GHA `Kitware/CMake`, `cmake-*-windows-(x86_64\|arm64).zip` | | `digest` | IA `*` | `cmake/bin/cmake.exe` + `tools/ninja.exe` | `_languages.ps1:286-291` |
| ninja (parte di cmake) | GHA `ninja-build/ninja`, `ninja-win.zip` / `ninja-winarm64.zip` | | `digest` | `ninja.exe` → `A\tools\` | `tools/ninja.exe` | `_languages.ps1:292-298` |
| cc | GHA `mstorsjo/llvm-mingw`, `llvm-mingw-*-ucrt-(x86_64\|aarch64).zip` | | `digest` | IA `*`; **eliminare** `bin\*-w64-mingw32-gcc.exe` e `-g++.exe` (romperebbero il linker di Rust `gnu`) | `cc/bin/clang.exe` | `_languages.ps1:308-317` |
| ruby | `api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=30`, esclusi prerelease/bozze, versione e build più alte | `rubyinstaller-X.Y.Z-N-(x64\|arm).7z` | `digest` | IA `*`; versione `X.Y.Z-N`; senza MSYS2 | `ruby/bin/ruby.exe` | `_languages.ps1:321-340` |
| micromamba (per r) | tag dell'ultima release di `mamba-org/micromamba-releases` (API di GitHub) | `micromamba-win-64` (solo x64) | SHA-256 da `micromamba-win-64.sha256` della release | exe → `A\tools\micromamba.exe` | `tools/micromamba.exe` | `_languages.ps1:344-358` |
| r | `api.anaconda.org/package/conda-forge/r-base` → `latest_version` | solo x64 (ARM64: non disponibile) | SHA-256 di ogni pacchetto (micromamba) | `micromamba create --no-rc -y -q -p <lavoro>\r -c conda-forge --override-channels r-base=<v> "libblas=*=*openblas"` con profilo e `MAMBA_ROOT_PREFIX` nell'area di lavoro; lanciatori `bin\R.cmd`, `bin\Rscript.cmd`; CRAN `cloud.r-project.org` in `Rprofile.site`; via `pkgs\` | `r/bin/R.cmd` | `_languages.ps1:368-429` |
| mariadb | `downloads.mariadb.org/rest-api/mariadb/` → LTS stabile più recente → `/<major>/latest/` | `mariadb-<v>-winx64.zip` (via https) | SHA-256 dell'API | IA singola; via `.pdb` | `mariadb/bin/mariadbd.exe` + `mariadb/bin/mariadb.exe` | nuovo (D-73) |
| mongodb | `downloads.mongodb.org/current.json`, produzione `x.0.y`; mongosh da GHA `mongodb-js/mongosh` | `mongodb-windows-x86_64-<v>.zip`, `mongosh-<v>-win32-x64.zip` in `mongosh\`, Database Tools da `release.json` in `mongodb-tools\` | SHA-256 dell'elenco; `digest` | IA singola; via `.pdb` e `vc_redist`, VC++ del kit | `mongodb/bin/mongod.exe` + `mongosh/bin/mongosh.exe` + `mongodb-tools/bin/mongodump.exe` | nuovo (D-73) |
| sqlite | `sqlite.org/download.html`, righe `PRODUCT,` | `sqlite-tools-win-(x64\|arm64)-<n>.zip` | **SHA3-256** della pagina | IA | `sqlite/sqlite3.exe` | nuovo (D-73) |
| adminer | GHA `vrana/adminer` | `adminer-<v>-mysql.php` (un file, tutte le lingue) | `digest` | file → `adminer\adminer.php`; controllo `php -l` col PHP del kit | `adminer/adminer.php` | nuovo (D-85) |
| CodeLLDB (estensione) | GHA `vadimcn/codelldb` | `codelldb-win32-<arch>.vsix`; x64 anche su ARM64, come il server dell'editor | `digest` obbligatorio | installata nel server dell'editor con `--install-extension`, se un progetto ha Rust, Zig, C o C++ (matrice `extensions.json`, D-71), una volta sola | cartella `vadimcn.vscode-lldb-*` in `data\vscodium-server\extensions\` | `_vscodium-extensions.ps1:162-206` |

## 3. Regole trasversali

- **Ordine**: quello di `components.json`: claude, codex, antigravity, git, python, node, uv, gh, php, apache, rust, go, java (+maven, gradle), kotlin, android, dotnet, dart, zig, julia, bun, deno, typst, tinytex, pnpm, cmake (+ninja), cc, ruby, r, pwsh, terminal, vscodium-server. Prima le rimozioni, poi le installazioni; configurazione Git e sito Apache sono passi finali di quei componenti. Da riga di comando, in fondo a `prepare`, la verifica finale di ciò che è stato installato. Estensioni dell'editor dopo l'avvio dell'editor; allineamento degli agenti all'avvio dell'app e intorno a ogni agente. (Nel Atollo: `Prepare-Atollo.ps1:250-285`.)
- **Un errore non ferma gli altri**: ogni passo è isolato e finisce nel riepilogo; esito complessivo "con errori" se uno fallisce.
- **Quando si installa**: file chiave o una parte di `keys` mancante, editor a una versione diversa da quella del rilascio (all'avvio dell'app, o in un `prepare` che lo comprende), oppure "Aggiorna tutto" (`--update-all`) / `--force`. `--only` installa o aggiorna i componenti elencati e ciò che serve loro e manca, obbligatori esclusi, senza rimuovere nulla; `--skip` salta anche ciò che ne dipende e non è ancora installato (mai gli obbligatori).
- **Quando serve** (D-59): avvio di un agente, clone, azione di un progetto installano prima i componenti che servono e li segnano come scelti in `components.txt`.
- **Parametri predefiniti**: Claude canale `latest`; Python `3.13` (`^3\.\d+$`); PHP ramo più recente (`^\d+\.\d+$`); Rust `gnu` su x64; architettura dell'host. Nell'app: pagina Impostazioni › Versioni.
- **VC++** estratto una volta in `tools\vcruntime\` e riusato; DLL copiate accanto a `php.exe` e in `apache\bin` (distribuzione locale permessa da Microsoft); x64 anche su ARM64. La cartella resta finché c'è PHP o Apache.
- **File dell'utente creati una sola volta e mai sovrascritti**: `data\php\php.ini` (da `php.ini-development`: `extension_dir`, estensioni curl, fileinfo, gd, intl, mbstring, mysqli, openssl, pdo_mysql, pdo_sqlite, sodium, sqlite3, zip; `date.timezone = Europe/Rome`; `curl.cainfo` e `openssl.cafile` su `cacert.pem`), `data\apache\kit.conf` (porta 8080 su `127.0.0.1`, sito in `data\apache\htdocs`, PHP via `<IfFile>`, `conf.d\*.conf`), `data\home\.gitconfig`, `htdocs\index.php`. Testi in UTF-8 senza BOM. `httpd.conf` invece si riscrive a ogni installazione di Apache.
- **Python**: prima di aggiornare, `pip freeze` in `data\python-packages-<vecchia>.txt`; stessa versione minore → copia di `Lib\site-packages` e `Scripts` (escluso pip); minore diversa → solo l'elenco.
- **Rust**: `gnu` predefinito su x64 (porta il proprio linker); ARM64 sempre `msvc`; `rustup-init` sempre build msvc. Senza collegamenti fisici (FAT32/exFAT) l'installazione si ferma con un errore; il ripiego del Atollo (toolchain sul disco del PC, copia nel kit di `rustup\`) non è portato.
- **Voce Rustup in HKCU** (`…\Uninstall\Rustup`): rustup la scrive sempre; si salva prima e si ripristina o si rimuove dopo (solo se punta al kit). È una scrittura temporanea nel registro **fatta da rustup, non dall'app** (DECISIONS.md D-38).
- **Configurazione Git**: `credential.helper` di sistema impostato a `manager` se è vuoto, `helper-selector` o un percorso di GCM; `.gitconfig` con `init.defaultBranch=main` e `safe.directory=*` (FAT32/exFAT non registrano il proprietario); `GCM_CREDENTIAL_STORE=dpapi` nell'ambiente.
- **Firma**: Authenticode con editore atteso; errore di revoca non verificabile = avviso, salvo dove la firma è l'unica verifica (VC++).
- **Scambi atomici**: il nuovo albero si prepara in `data\tmp\setup\<id>` (stesso disco), la cartella vecchia viene rinominata `<nome>.old-<ts>` (fallisce subito se un file è in uso), poi spostamento, ripristino in caso di errore; avanzi `.old-*`/`.new-*` cancellati all'installazione successiva. Exe singoli: vecchio rinominato `.old`, così si sostituisce anche se in uso.
- **FAT32/exFAT**: estrazione senza ripristinare le date dei file; prova dei collegamenti fisici prima di Rust; area di lavoro sempre in `data\tmp\setup\`.
- **Proxy**: impostazioni dell'app → `HTTPS_PROXY`/`HTTP_PROXY` → proxy di Windows (sola lettura); niente PAC (D-29). Le impostazioni finiscono anche nell'ambiente del kit (`HTTPS_PROXY`, `HTTP_PROXY`, `NO_PROXY`), che ricevono rustup, pip e npm. Download in `.part`, rinominato a fine download, 3 tentativi (5 s, 10 s).
- **Una preparazione alla volta** per kit (mutex sul percorso della radice).
- **Versioni come stringhe**: node con la `v`; Go senza `go`; GHA senza `v`/`bun-v`; ruby `X.Y.Z-N`; componenti con più parti uniscono le versioni (`8.5.0 (Composer 2.10.3)`). I confronti con `data\app\versions\` usano le stesse stringhe.

## 4. Differenze dal Atollo

| Atollo | Nuovo kit | Motivo |
| --- | --- | --- |
| VSCodium desktop obbligatorio | tolto | editor integrato |
| Git, Node.js, uv obbligatori | installati quando servono (DECISIONS.md D-59) | obbligatori solo PowerShell 7 e GitHub CLI |
| — | `pwsh` obbligatorio; `vscodium-server` parte dell'app con versione fissata (D-58) | terminale esclusivo del kit (Windows Terminal portabile, D-68, o la console classica di Windows, D-57), editor integrato |
| Git: ripiego senza API (tag + Authenticode) | solo API di GitHub con `digest` | — |
| Git SFX e TinyTeX SFX aprono la finestra di 7-Zip | estrazione 7z in Rust, nessuna finestra | interfaccia pulita |
| Typst e TinyTeX non vengono rimossi quando si tolgono | rimossi | bug di `_prepare-migrate.ps1:128` |
| Rust su FAT32/exFAT: toolchain sul disco del PC | errore | da portare |
| `Test-Installation` con `rustc --version` ecc. | stessa verifica: `Atollo.exe check` e in fondo a `prepare` | |
| Versioni in `data\versions\` | `data\app\versions\` | tutto ciò che è dell'app sta in `data\app\` |
