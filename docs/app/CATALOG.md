# Dev-Kit App: catalogo di agenti, linguaggi e strumenti

Fonte: `nabre/dev-kit` al commit `a61a354` (`setup/components.json` e script di `setup/windows/`). È la specifica che il motore Rust (`catalog.rs`, `install/*.rs`) deve rispettare. I riferimenti `file:riga` puntano agli script originali.

Percorsi nel nuovo kit (PLAN §2): `A` = `data\apps\win32-<arch>\`; versioni in `data\app\versions\<id>.txt`; aree di lavoro in `data\tmp\setup\`.
Endpoint letti dal codice, non ancora interrogati: ogni installatore Rust ha un test reale in CI (con cache) che li verifica.

## 1. Componenti e gruppi

| Gruppo | Id | Nome | Predefinito | MB (x64) | Dipende da | Note |
| --- | --- | --- | --- | --- | --- | --- |
| obbligatori | `git` | Git (Git Bash) | sempre | 350 | | shell dei comandi degli agenti |
| | `node` | Node.js LTS + npm | sempre | 110 | | server MCP npx, estensioni |
| | `uv` | uv, uvx | sempre | 35 | | server MCP uvx, Python |
| | `gh` | GitHub CLI | sempre | 40 | | clone, accesso a GitHub |
| | `vscodium-server` | Editor (server web di VSCodium) | sempre | 250 | | **nuovo**; x64 anche su ARM64 |
| | `pwsh` | PowerShell 7 | sempre | 110 | | **nuovo**; terminale del kit |
| agenti | `claude` | Claude Code | sì | 250 | `git` | |
| | `codex` | Codex | sì | 440 | | |
| | `antigravity` | Antigravity | sì | 190 | | accesso salvato nel kit (`keyring.rs`) |
| linguaggi | `python` | Python + pip | sì | 70 | | 3.13 predefinito |
| | `php` | PHP + Composer | sì | 100 | | con VC++ locale |
| | `apache` | Apache | sì | 45 | `php` | |
| | `rust` | Rust | no | 920 | | toolchain `gnu` su x64 |
| | `go` | Go | no | 250 | | |
| | `java` | Java (Temurin, Maven, Gradle) | no | 300 | | |
| | `kotlin` | Kotlin | no | 80 | `java` | |
| | `dotnet` | .NET SDK | no | 250 | | |
| | `dart` | Dart | no | 200 | | |
| | `zig` | Zig | no | 200 | | |
| | `julia` | Julia | no | 500 | | |
| | `bun` | Bun | no | 90 | | |
| | `deno` | Deno | no | 100 | | |
| | `pnpm` | pnpm, yarn | no | 10 | `node` | in `data\npm-global\` |
| | `cmake` | CMake, Ninja | no | 100 | | |
| | `cc` | C, C++ (llvm-mingw) | no | 720 | | |
| | `ruby` | Ruby | no | 270 | | |
| | `r` | R (conda-forge) | no | 1100 | | solo x64 |
| | `typst` | Typst | no | 50 | | |
| | `tinytex` | TinyTeX | no | 280 | | |
| facoltativi | `vscodium` | VSCodium desktop | no | 400 | | nel Dev-Kit era obbligatorio; qui l'editor è integrato |

I gruppi, i predefiniti e le dimensioni sono quelli di `components.json`, salvo le righe nuove e `vscodium`. Proposta del primo avvio: obbligatori + tre agenti + Python, PHP, Apache (~2,3 GB).

**Proposto, non ancora nel catalogo** (DECISIONS.md D-47): `conpty`, ConPTY aggiornato di Microsoft (`conpty.dll` + `OpenConsole.exe`), obbligatorio, circa 1,2 MB, per le console dell'app. Windows Terminal (`terminal`) è stato tolto il 14/10/2026: le console sono finestre dell'app (D-44).

## 2. Come si installa ciascuno

Abbreviazioni:
- **IA**: installazione da archivio (`Install-Archive`, `_languages.ps1:43-68`). Lettura della versione, salto se aggiornato, download, verifica (SHA-512 se disponibile, altrimenti SHA-256; hash mancante = errore), estrazione, sostituzione della cartella. `inner`: `''` = archivio così com'è, `*` = unica cartella di primo livello, altrimenti nome della sottocartella. Il file chiave deve esistere prima dello scambio.
- **GHA**: `api.github.com/repos/<repo>/releases/latest`; prima asset che corrisponde alla regex; SHA-256 da `asset.digest`; versione = tag senza `v` / `bun-v`.
- **Tag**: HEAD su `github.com/<repo>/releases/latest` senza seguire il redirect, tag letto dall'intestazione `Location` (nessun limite di frequenza dell'API).
- **Lista**: righe `^([0-9a-f]{64})\s+\*?(\S+)\s*$`, nome del file esatto.

| Id | Versione | Download (x64 / ARM64) | Verifica | Estrazione e passi successivi | File chiave | Origine |
| --- | --- | --- | --- | --- | --- | --- |
| claude | npm `@anthropic-ai/claude-code-win32-<arch>/<canale>`, canale `latest` (o `stable`, o un numero) | `dist.tarball`, nativo per entrambe | integrità npm `sha512-…`, poi Authenticode `Anthropic` su `claude.exe` (avviso se la revoca non è verificabile) | `package/claude.exe` → `A\claude\claude.exe`, vecchio rinominato `.old` | `claude/claude.exe` | `_prepare-core.ps1:10-37` |
| codex | npm `@openai/codex/latest`, poi `<ver>-win32-<arch>` | `dist.tarball`, nativo | integrità npm | `package/vendor/` con una sola cartella che contiene `bin/codex.exe` → `A\codex` | `codex/bin/codex.exe` | `_prepare-core.ps1:45-72` |
| antigravity | manifesto `antigravity-cli-auto-updater-974169037036.us-central1.run.app/manifests/windows_amd64.json` (`windows_arm64.json`) | `url` del manifesto | SHA-512 del manifesto (obbligatorio) | exe singolo → `A\antigravity\agy.exe`; togliere il contrassegno "scaricato da Internet"; non eseguire `agy install` | `antigravity/agy.exe` | `_prepare-core.ps1:81-104` |
| git | API `git-for-windows/git`, asset `PortableGit-…-(64-bit\|arm64).7z.exe`; ripiego Tag `v(X.Y.Z).windows.N` | asset del rilascio | SHA-256 da `digest`; senza API: Authenticode `Schindelin\|Git for Windows` obbligatorio | 7z SFX (in Rust: estrazione 7z diretta, nessuna finestra); controllo `bin\bash.exe`; poi `post-install.bat` nella cartella (lo lancerebbe l'SFX; si cancella da solo, quindi esce sempre con 1) e configurazione Git (§3) | `git/bin/bash.exe` | `_prepare-core.ps1:109-158` |
| python | `raw.githubusercontent.com/astral-sh/python-build-standalone/latest-release/latest-release.json` → `SHA256SUMS`, file `cpython-3.13.N+…-<triple>-install_only_stripped.tar.gz` (ultimo) | triple `x86_64-pc-windows-msvc` / `aarch64-pc-windows-msvc` | SHA-256 da `SHA256SUMS` | tar.gz `python\`; pacchetti conservati (§3); `python.exe` → copia `python3.exe`; pip offline da `Lib\ensurepip\_bundled\pip-*.whl` | `python/python.exe` | `_prepare-core.ps1:163-234` |
| node | `nodejs.org/dist/index.json`, primo LTS con `win-<arch>-zip` | `nodejs.org/dist/<v>/node-<v>-win-<arch>.zip` | SHA-256 da `SHASUMS256.txt` (Lista) | zip `node-<v>-win-<arch>`; versione con la `v` iniziale | `node/node.exe` | `_prepare-core.ps1:239-268` |
| uv | Tag `astral-sh/uv` | `uv-<triple>.zip` | SHA-256 da `<file>.sha256` | `uv.exe`, `uvx.exe`, `uvw.exe` → `A\tools\` | `tools/uv.exe` | `_prepare-core.ps1:273-302` |
| gh | Tag `cli/cli` | `gh_<v>_windows_amd64.zip` / `_arm64.zip` | SHA-256 da `gh_<v>_checksums.txt` | `gh.exe` → `A\tools\` | `tools/gh.exe` | `_prepare-core.ps1:307-335` |
| vscodium-server | GHA `VSCodium/vscodium`, asset `vscodium-reh-web-win32-x64-<v>.tar.gz` | solo x64 (anche su ARM64) | SHA-256 da `<asset>.sha256`, confrontato con il `digest` di GitHub | tar.gz (radice: `bin\`, `node.exe`, `out\`, `extensions\`) → `A\vscodium-server\` | `vscodium-server/bin/codium-server.cmd` (verificato il 14/10) | nuovo |
| pwsh | GHA `PowerShell/PowerShell`, `PowerShell-<v>-win-(x64\|arm64).zip` | nativo per entrambe | SHA-256 da `digest` (anche `hashes.sha256`) | zip → `A\pwsh\` | `pwsh/pwsh.exe` | nuovo |
| php | `windows.php.net/downloads/releases/releases.json`, ramo più recente (o scelto), build `ts-vs*-x64` | solo x64 (emulato su ARM64, con avviso) | SHA-256 di `releases.json` | zip; DLL VC++ accanto a `php.exe`; `cacert.pem` da curl.se con SHA-256; `php.ini` creato una volta (§3) | `php/php.exe` | `_prepare-web.ps1:89-167` |
| composer | `getcomposer.org/download/latest-stable/composer.phar` | indipendente dall'architettura | SHA-256 da `composer.phar.sha256sum` | `A\tools\composer.phar` + `composer.cmd` (`@php "%~dp0composer.phar" %*`) | `tools/composer.phar` | `_prepare-web.ps1:153-164` |
| VC++ (per php, apache) | `aka.ms/vc14/vc_redist.x64.exe`, versione = `ProductVersion` | solo x64 | Authenticode `O=Microsoft Corporation` obbligatorio, sull'exe e su `vcruntime140.dll` | estrazione dei CAB incorporati (firma `MSCF`), manifesto Burn, payload `vcRuntimeMinimum_amd64\cab1.cab`; `*.dll_amd64` → `*.dll` in `A\tools\vcruntime\` | `tools/vcruntime/vcruntime140.dll` | `_prepare-web.ps1:13-83` |
| apache | HTML di `www.apachelounge.com/download/`, primo `httpd-X.Y.Z-N-Win64-VSnn.zip` | solo Win64 | SHA-256 da `<url>.txt` | zip `Apache24`; DLL VC++ in `bin\`; `httpd.conf`: `SRVROOT`, `Listen 80` commentato, `rewrite_module`, `Include …/data/apache/kit.conf`; `kit.conf` creato una volta (§3) | `apache/bin/httpd.exe` | `_prepare-web.ps1:174-276` |
| rust | `static.rust-lang.org/rustup/dist/<arch>-pc-windows-msvc/rustup-init.exe`; toolchain `stable` | host: x64 → `x86_64-pc-windows-gnu`, ARM64 → `aarch64-pc-windows-msvc` | SHA-256 da `rustup-init.exe.sha256` | `-y --no-modify-path --profile minimal --default-host <host> --default-toolchain stable -c rustfmt -c clippy`, `RUSTUP_HOME`, `CARGO_HOME` nel kit; `rustup set auto-self-update disable`; voce Rustup di HKCU (§3) | `rust/cargo/bin/rustup.exe` | `_prepare-rust.ps1:13-198` |
| go | `go.dev/dl/?mode=json`, primo `stable` | file `windows`/`amd64\|arm64`/archive | SHA-256 del JSON | IA `go` | `go/bin/go.exe` | `_languages.ps1:77-89` |
| java | `api.adoptium.net/v3/info/available_releases` → LTS; `assets/latest/<lts>/hotspot?architecture=x64\|aarch64&image_type=jdk&os=windows&vendor=eclipse` | primo `.zip` | SHA-256 `binary.package.checksum` | IA `*` | `java/bin/java.exe` | `_languages.ps1:94-105` |
| maven | `repo.maven.apache.org/…/apache-maven/maven-metadata.xml`, versione numerica più alta | `apache-maven-<v>-bin.zip` | SHA-512 da `.sha512` | IA `apache-maven-<v>`; `conf\settings.xml`: `<localRepository>${env.CP_DATA}/maven/repository</localRepository>` | `maven/bin/mvn.cmd` | `_languages.ps1:107-120` |
| gradle | `services.gradle.org/versions/current` | `downloadUrl` | SHA-256 da `checksumUrl` | IA `gradle-<v>` | `gradle/bin/gradle.bat` | `_languages.ps1:122-125` |
| kotlin | GHA `JetBrains/kotlin`, `kotlin-compiler-*.zip` | indipendente | `digest` | IA `kotlinc` | `kotlin/bin/kotlinc.bat` | `_languages.ps1:130-136` |
| dotnet | `builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json`, canale LTS attivo → `latest-sdk` → `releases.json` | `rid=win-x64\|win-arm64`, `.zip` | SHA-512 `hash` | IA `''` | `dotnet/dotnet.exe` | `_languages.ps1:139-157` |
| dart | `storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION` | `dartsdk-windows-<x64\|arm64>-release.zip` | SHA-256 da `.sha256sum` | IA `dart-sdk` | `dart/bin/dart.exe` | `_languages.ps1:160-168` |
| zig | `ziglang.org/download/index.json`, versione numerica più alta | `x86_64-windows` / `aarch64-windows` | SHA-256 `shasum` | IA `*` | `zig/zig.exe` | `_languages.ps1:171-181` |
| julia | `julialang-s3.julialang.org/bin/versions.json`, ultima `stable` | `winnt`/`x86_64\|aarch64`/zip | SHA-256 | IA `*` | `julia/bin/julia.exe` | `_languages.ps1:184-195` |
| bun | GHA `oven-sh/bun`, `bun-windows-(x64\|aarch64).zip` | | `digest` | IA `*` | `bun/bun.exe` | `_languages.ps1:198-205` |
| deno | GHA `denoland/deno`, `deno-(x86_64\|aarch64)-pc-windows-msvc.zip` | | `digest` | IA `''` | `deno/deno.exe` | `_languages.ps1:206-213` |
| typst | GHA `typst/typst`, `typst-(x86_64\|aarch64)-pc-windows-msvc.zip` | | `digest` | IA `*` | `typst/typst.exe` | `_languages.ps1:216-223` |
| tinytex | GHA `rstudio/tinytex-releases`, `TinyTeX-1-windows-v*.exe` | x64 anche su ARM64 | `digest` | 7z SFX → `TinyTeX\` con `bin\windows\tlmgr.bat` | `tinytex/bin/windows/tlmgr.bat` | `_languages.ps1:228-250` |
| pnpm | npm `pnpm@latest yarn@latest` | registro npm | integrità di npm | `npm install --global --no-fund --no-audit --ignore-scripts` con il Node del kit, prefisso `data\npm-global` | `@data/npm-global/pnpm.cmd` | `_languages.ps1:254-281` |
| cmake | GHA `Kitware/CMake`, `cmake-*-windows-(x86_64\|arm64).zip` | | `digest` | IA `*` | `cmake/bin/cmake.exe` | `_languages.ps1:286-291` |
| ninja | GHA `ninja-build/ninja`, `ninja-win.zip` / `ninja-winarm64.zip` | | `digest` | `ninja.exe` → `A\tools\` | `tools/ninja.exe` | `_languages.ps1:292-298` |
| cc | GHA `mstorsjo/llvm-mingw`, `llvm-mingw-*-ucrt-(x86_64\|aarch64).zip` | | `digest` | IA `*`; **eliminare** `bin\*-w64-mingw32-gcc.exe` e `-g++.exe` (romperebbero il linker di Rust `gnu`) | `cc/bin/clang.exe` | `_languages.ps1:308-317` |
| ruby | `api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=30`, esclusi prerelease/bozze, versione e build più alte | `rubyinstaller-X.Y.Z-N-(x64\|arm).7z` | `digest` | IA `*`; versione `X.Y.Z-N`; senza MSYS2 | `ruby/bin/ruby.exe` | `_languages.ps1:321-340` |
| micromamba (per r) | Tag `mamba-org/micromamba-releases` | `micromamba-win-64` (solo x64) | SHA-256 da `.sha256` | exe → `A\tools\micromamba.exe` | `tools/micromamba.exe` | `_languages.ps1:344-358` |
| r | `api.anaconda.org/package/conda-forge/r-base` → `latest_version` | solo x64 (ARM64: non disponibile) | SHA-256 di ogni pacchetto (micromamba) | `micromamba create --no-rc -y -q -p <lavoro>\r -c conda-forge --override-channels r-base=<v> "libblas=*=*openblas"` con profilo e `MAMBA_ROOT_PREFIX` nell'area di lavoro; lanciatori `bin\R.cmd`, `bin\Rscript.cmd`; CRAN `cloud.r-project.org` in `Rprofile.site`; via `pkgs\` | `r/bin/R.cmd` | `_languages.ps1:368-429` |
| vscodium (facoltativo) | `raw.githubusercontent.com/VSCodium/versions/master/stable/win32/<arch>/archive/latest.json` | `url` | SHA-256 `sha256hash` | zip in modalità portabile (`data\` nella cartella), scambio conservando `data\` | `vscodium/VSCodium.exe` | `Install-VSCodium.ps1:342-473` |
| CodeLLDB (estensione) | API `vadimcn/codelldb` | `codelldb-win32-<arch>.vsix` | `digest` obbligatorio | installata nel server dell'editor; installata se c'è `platform.ok` | `…/vadimcn.vscode-lldb-*/platform.ok` | `_vscodium-extensions.ps1:162-206` |

## 3. Regole trasversali

- **Ordine**: claude, codex, antigravity, git, python, node, uv, gh, php, apache, sito Apache, rust, configurazione Git, poi go, java (+maven, gradle), kotlin, dotnet, dart, zig, julia, bun, deno, typst, tinytex, pnpm, cmake (+ninja), cc, ruby, r; nel nuovo kit poi pwsh, vscodium-server, estensioni, allineamento agenti, verifica finale. (`Prepare-DevKit.ps1:250-285`)
- **Un errore non ferma gli altri**: ogni passo è isolato e finisce nel riepilogo; esito complessivo "con errori" se uno fallisce.
- **Quando si installa**: file chiave mancante, oppure "Aggiorna tutto" / "Riscarica". Componenti con più file (java, cmake, pnpm): se ne manca uno qualsiasi. "Solo questi" salta tutto il resto, obbligatori compresi.
- **Parametri predefiniti**: Claude canale `latest`; Python `3.13` (`^3\.\d+$`); PHP ramo più recente (`^\d+\.\d+$`); Rust `gnu` su x64; architettura dell'host. Nell'app: pagina Impostazioni › Avanzate.
- **VC++** estratto una volta per esecuzione; DLL copiate accanto a `php.exe` e in `apache\bin` (distribuzione locale permessa da Microsoft); x64 anche su ARM64.
- **File dell'utente creati una sola volta e mai sovrascritti**: `data\php\php.ini` (da `php.ini-development`: `extension_dir`, estensioni curl, fileinfo, gd, intl, mbstring, mysqli, openssl, pdo_mysql, pdo_sqlite, sodium, sqlite3, zip; `date.timezone = Europe/Rome`; `curl.cainfo` e `openssl.cafile` su `cacert.pem`), `data\apache\kit.conf` (porta 8080 su `127.0.0.1`, sito in `data\apache\htdocs`, PHP via `<IfFile>`, `conf.d\*.conf`), `data\home\.gitconfig`, `htdocs\index.php`. Testi in UTF-8 senza BOM. `httpd.conf` invece si riscrive a ogni installazione di Apache.
- **Python**: prima di aggiornare, `pip freeze` in `data\python-packages-<vecchia>.txt`; stessa versione minore → copia di `Lib\site-packages` e `Scripts` (escluso pip); minore diversa → solo avviso.
- **Rust**: `gnu` predefinito su x64 (porta il proprio linker); ARM64 sempre `msvc`; `rustup-init` sempre build msvc. Senza collegamenti fisici (FAT32/exFAT): installazione sul disco del PC solo quando cambia `stable`, copia nel kit di `rustup\` e `rustup.exe`, nessun proxy.
- **Voce Rustup in HKCU** (`…\Uninstall\Rustup`): rustup la scrive sempre; si salva prima e si ripristina o si rimuove dopo (solo se punta al kit). È una scrittura temporanea nel registro **fatta da rustup, non dall'app**: eccezione del README da riportare in `DECISIONS.md`.
- **Configurazione Git**: `credential.helper` di sistema impostato a `manager` se è `helper-selector` o un percorso di GCM; `.gitconfig` con `init.defaultBranch=main` e `safe.directory=*` (FAT32/exFAT non registrano il proprietario); `GCM_CREDENTIAL_STORE=dpapi` nell'ambiente.
- **Firma**: Authenticode con editore atteso; errore di revoca non verificabile = avviso, salvo dove la firma è l'unica verifica (Git senza API, VC++).
- **Scambi atomici**: cartella nuova in `<dest>.new-<ts>`, vecchia rinominata `.old-<ts>` (fallisce subito se un file è in uso), poi spostamento, ripristino in caso di errore. Exe singoli: vecchio rinominato `.old`, così si sostituisce anche se in uso.
- **FAT32/exFAT**: estrazione senza ripristinare le date dei file; prova dei collegamenti fisici; area di lavoro sul disco del PC solo se serve, cancellata alla fine.
- **Proxy**: proxy di sistema con le credenziali predefinite + `HTTPS_PROXY`/`HTTP_PROXY`/`NO_PROXY` delle impostazioni, passati anche a rustup, pip e npm. Download in `.part`, rinominato a fine download, 3 tentativi (5 s, 10 s).
- **Una preparazione alla volta** per kit (mutex sul percorso della radice).
- **Versioni come stringhe**: node con la `v`; uv = tag; Go senza `go`; GHA senza `v`/`bun-v`; ruby `X.Y.Z-N`. I confronti con `data\app\versions\` usano le stesse stringhe.

## 4. Differenze dal Dev-Kit

| Dev-Kit | Nuovo kit | Motivo |
| --- | --- | --- |
| VSCodium desktop obbligatorio | facoltativo | editor integrato |
| — | `vscodium-server`, `pwsh` obbligatori | editor integrato, terminale esclusivo del kit (le console sono finestre dell'app, DECISIONS.md D-44) |
| Git SFX e TinyTeX SFX aprono la finestra di 7-Zip | estrazione 7z in Rust, nessuna finestra | interfaccia pulita |
| Typst e TinyTeX non vengono rimossi quando si tolgono | rimossi | bug di `_prepare-migrate.ps1:128` |
| `Test-Installation` con `rustc --version` ecc. | stessa verifica, risultati nella pagina Componenti | |
| Versioni in `data\versions\` | `data\app\versions\` | tutto ciò che è dell'app sta in `data\app\` |
