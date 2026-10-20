# Atollo App: catalog of agents, languages and tools

English · [Italiano](CATALOG.it.md)

Source: `nabre/atollo` at commit `a61a354` (`setup/components.json` and the scripts in `setup/windows/`). It is the specification that the Rust engine (`setup/catalog.rs`, `install/`) follows; the app's catalog is `tauri/catalog/components.json`. The `file:line` references point to the original scripts.

Paths in the new kit (D-00, D-114): `A` = `data\apps\<target>\` (`win32-x64`, `win32-arm64`, `darwin-arm64`, `darwin-x64`, `linux-x64`, `linux-arm64`); versions in `data\app\versions\<target>\<id>.txt`; work areas in the system temp folder, otherwise in `data\tmp\setup\` (D-90). The tables in §2 describe Windows; the sources for the other systems are in §1, "Sources per system".
Real tests: the CI (x64 job) installs `pwsh`, `node`, `git` and the editor; all components were installed and removed by hand in phase 4 (CHANGELOG.md (in Italian)).

## 1. Components and groups

| Group | Id | Name | Default | MB (x64) | Depends on | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| required (`required`) | `gh` | GitHub CLI | always | 40 | | pull requests, issues, access to GitHub (also for private clones); works without Git (D-81) |
| | `pwsh` | PowerShell 7 | always | 110 | | **new**; the kit's terminal |
| app (`app`) | `vscodium-server` | Editor (VSCodium web server) | always | 250 | `pwsh` | **new**; part of the app, version pinned at each release (DECISIONS.md (in Italian) D-58); in Languages and tools it appears with the required ones, checked and not editable; SHA-256 pinned per target, x64 under emulation on Windows ARM64 |
| agents (`agents`) | `claude` | Claude Code | yes | 250 | `git` | |
| | `codex` | Codex | yes | 440 | | |
| | `antigravity` | Antigravity | yes | 190 | | sign-in saved in the kit (`agents/keyring.rs`) |
| languages and tools (`languages`) | `git` | Git (Git Bash on Windows) | when needed (with Claude Code) | 350 | | Git Bash is the agents' command shell on Windows; it comes with Claude Code, a clone or a project (D-81) |
| | `python` | Python + pip | yes | 70 | | the latest stable minor line, unless `pythonVersion` is set in Settings (D-81) |
| | `node` | Node.js LTS + npm | no | 110 | | when needed: actions with `npm`, MCP servers with `npx` (D-59) |
| | `uv` | uv, uvx | no | 35 | | when needed: actions with `uv`, MCP servers with `uvx` (D-59) |
| | `php` | PHP + Composer | yes | 100 | | Windows: with local VC++, x64 under emulation on Windows ARM64; macOS and Linux: the kit's build (`php-v<version>-<build>`, with `pdo_sqlite`, SHA-256 pinned in `sha256ByLine`, D-133); a line the kit does not pin: static-php-cli (D-112) |
| | `apache` | Apache | yes | 45 | `php` | Windows: Apache Lounge with the PHP module, x64 under emulation on Windows ARM64; macOS and Linux: the kit's build (`httpd-v<version>`, SHA-256 pinned in `sha256ByLine`) with `php-fpm` (D-116) |
| | `caddy` | Caddy | no | 45 | | **new** (D-111); web server of the pools and sites on all systems; replaces Nginx, removed with the migration of its pools and sites at startup (D-120) |
| | `rust` | Rust | no | 920 | `cc` on Windows ARM64 | `gnu` host on Windows x64, `gnullvm` on Windows ARM64 (linked by the llvm-mingw of `cc`), `msvc` from Settings; native on macOS and Linux |
| | `go` | Go | no | 250 | | |
| | `java` | Java (Temurin, Maven, Gradle) | no | 300 | | |
| | `kotlin` | Kotlin | no | 80 | `java` | |
| | `android` | Android SDK | no | 600 | `java` | **new** (D-84); command-line tools, platform-tools, platform and build-tools, without emulator; choosing it accepts Google's SDK licenses; x64 under emulation on Windows ARM64; not available on Linux ARM64 (`unavailable`: Google does not publish platform-tools and build-tools) |
| | `dotnet` | .NET SDK | no | 250 | | |
| | `dart` | Dart | no | 200 | | |
| | `zig` | Zig | no | 200 | | |
| | `julia` | Julia | no | 500 | | x64 under emulation on Windows ARM64 |
| | `bun` | Bun | no | 90 | | |
| | `deno` | Deno | no | 100 | | |
| | `typst` | Typst | no | 50 | | |
| | `tinytex` | TinyTeX | no | 280 | | x64 under emulation on Windows ARM64 |
| | `pnpm` | pnpm, yarn | no | 10 | `node` | in `data\npm-global\` |
| | `cmake` | CMake, Ninja | no | 100 | | |
| | `cc` | C, C++ | no | 720 | `zig` on macOS and Linux | llvm-mingw on Windows; `zig cc` and GNU make on macOS and Linux (D-112); MB for Windows, about 20 on Unix |
| | `ruby` | Ruby | no | 270 | | RubyInstaller on Windows (native on ARM64 too), conda-forge on macOS and Linux (D-112), not relocatable there |
| | `r` | R (conda-forge) | no | 1100 | | Windows ARM64: `win-64` build under emulation (`x64Only`, conda-forge has no `win-arm64`; native micromamba with `--platform win-64`, D-132); native on macOS and Linux, ARM64 included, not relocatable there |
| databases (`databases`) | `mysql` | MySQL | no | 520 | | **new** (D-110); the kit's only SQL server (D-117), lines 8.4 and 9.7 LTS; x64 under emulation on Windows ARM64; on Linux ARM64 the download is about 940 MB |
| | `mongodb` | MongoDB, mongosh, Database Tools | no | 570 | | **new** (D-73); Database Tools for backups (D-78); x64 under emulation on Windows ARM64; on Linux the server comes from conda-forge and is not relocatable |
| | `phpmyadmin` | phpMyAdmin | no | 60 | `php`, `mysql` | **new** (D-107, replacing the Adminer of D-85); MySQL in the browser, on all six targets (D-117) |
| | `sqlite` | SQLite (sqlite3) | no | 15 | | **new** (D-73); on Linux ARM64 from conda-forge, not relocatable |

The groups, defaults and sizes are those of `components.json`. First-run proposal: required, editor, three agents, Python, PHP, Apache, plus Git that is always installed: about 1.8 GB (1845 MB, of which 250 for the editor), on every system (Apache too since D-116).

Removed from the catalog on 14/10/2026: Windows Terminal (`terminal`), because the consoles were the classic Windows console (DECISIONS.md D-57), then back the same day as the consoles' default terminal (D-68) and removed again on 16/10/2026 (D-95), along with the `os` group that contained only it; VSCodium desktop (`vscodium`), because the editor is built in. The `conpty` component proposed in D-47 was not added.

Catalog fields beyond those of Atollo: `keys` (the other parts of a component: Maven and Gradle for Java, Ninja for CMake, Composer for PHP, yarn for pnpm, the editor's `node.exe`; installed only if all of them are there), `remove` (other owned files; `tools\vcruntime` of PHP and Apache goes away only with the last of the two), `check` (command of the final check, for all), `version` and `sha256` (`app` group). Key files are written with Windows names and translated for macOS and Linux (`translate_rel`: `git/bin/bash.exe` → `git/bin/git`, `.exe` and `.cmd` removed).

**Availability per target** (D-108). Optional fields:

| Field | What it says | Example |
| --- | --- | --- |
| `targets` | the only targets where the component exists; empty = all | none today (Apache until D-116: `["win32-x64", "win32-arm64"]`) |
| `unavailable` | target → reason (in Italian, translated by the catalogs), shown instead of "not available for `<target>`" | Android: `linux-arm64` |
| `noArm64` | no build runs on Windows ARM64, not even under emulation | none today (R until D-132) |
| `x64Only` | x64 build under emulation, only on Windows ARM64 | PHP, Apache, Android SDK, Julia, TinyTeX, R, MySQL, MongoDB, editor |
| `rosetta` | x64 build on `darwin-arm64` with Rosetta 2; without Rosetta the component is not available | none today |
| `needsByTarget` | extra needs on a target, beyond `needs`; `components.txt` saves only those valid everywhere | `cc`: `zig` on macOS and Linux; `rust`: `cc` on `win32-arm64` |
| `removeUnix` | the owned files on macOS and Linux, instead of `remove` | pnpm: `bin/pnpm`, `lib/node_modules/corepack` |
| `sha256ByTarget` | SHA-256 per target, with `version`, where the vendor does not publish a hash | editor |
| `sha256ByFile` | SHA-256 per file name | PHP: the static-php-cli archives, for a line the kit does not pin (D-133) |
| `sha256ByLine` | line → `{ version, sha256ByTarget }`: only the pinned lines are offered | MySQL 8.4 and 9.7, Apache 2.4 and PHP 8.4/8.5 on macOS and Linux |

A component not available for the system is shown as "not available for `<target>`" with the reason, never with a binary of another architecture; "not available on ARM64 PCs" remains only for Windows ARM64 (BUG-24c). `Atollo components --target <id>` shows the catalog of another target.

**Sources per system** (D-112). Where the table in §2 says Windows, the other systems use:

| Component | macOS | Linux | Check |
| --- | --- | --- | --- |
| Git | `desktop/dugite-native` (`macOS-arm64`, `macOS-x64`) | dugite-native (`ubuntu-x64`, `ubuntu-arm64`) | the asset's `.sha256`; relocatable |
| PHP | the kit's build (`scripts/build-php`, D-133), release `php-v<v>-<build>` of this repository, `php-<v>-<build>-darwin-{arm64,x64}.tar.gz`; a line not pinned: static-php-cli "bulk" | `php-<v>-<build>-linux-{x64,arm64}.tar.gz` (static, musl) | pinned SHA-256 (`sha256ByLine`; `sha256ByFile` for "bulk") |
| C, C++ | shims over the kit's `zig cc`, GNU make from conda-forge | as macOS | Zig index; micromamba |
| Ruby | conda-forge `ruby` (3.2 and later) | as macOS | micromamba; not relocatable |
| MongoDB | vendor build | server from conda-forge; generic mongosh; Database Tools per distribution | micromamba, vendor JSON |
| SQLite | sqlite.org | sqlite.org on x64, conda-forge `sqlite` on ARM64 | SHA3-256; micromamba |
| MySQL | `cdn.mysql.com`, `mysql-<v>-macos15-{arm64,x86_64}.tar.gz` | `mysql-<v>-linux-glibc2.28-x86_64-minimal.tar.xz`, `-aarch64.tar.xz` | pinned SHA-256 (`sha256ByLine`) after Oracle's GPG signature |
| Caddy | GitHub `caddyserver/caddy`, `caddy_<v>_mac_{arm64,amd64}.tar.gz` | `caddy_<v>_linux_{arm64,amd64}.tar.gz` | `digest`, or SHA-512 from `checksums.txt` |
| Apache | the kit's build (`scripts/build-httpd`, D-116), release `httpd-v<v>` of this repository, `httpd-<v>-darwin-{arm64,x64}.tar.gz` | `httpd-<v>-linux-{x64,arm64}.tar.gz` (glibc 2.35) | pinned SHA-256 (`sha256ByLine`) after the GPG check of the sources; relocatable; pinned at `2.4.69-1` |
| Antigravity | Google's manifest, `.tar.gz` (from 1.4) | as macOS | SHA-512 from the manifest |
| R | conda-forge `r-base` | as macOS | micromamba; not relocatable (D-92) |

On Windows ARM64: MySQL under x64 emulation (same archive as Windows x64), native Caddy, Rust with the `aarch64-pc-windows-gnullvm` host and the `ucrt-aarch64` llvm-mingw of `cc` as linker, native micromamba (`micromamba-win-arm64`). Apache remains Apache Lounge x64 under emulation: there is no native Apache or PHP for Windows ARM64 (D-116).

**Non-relocatable conda components** (D-92). A conda environment has its path written inside and cannot be moved: when the kit changes folder (`root-<os>.txt`), the non-relocatable components (`Installer::relocatable(target)` false: R, Ruby, MongoDB on Linux, SQLite on Linux ARM64 and C/C++ outside Windows) receive `data/app/versions/<target>/<id>.reinstall`, each of their saved lines `<id>@<line>.reinstall`, and the automatic installation at startup reinstalls them: from the archives kept in `data/app/conda/<target>/` (`pkgs/` and the explicit list `locks/<id>@<line>.txt` with its `.json`), without network, after checking the SHA-256 of each archive; from the network as before if something is missing or different (D-118). Git from dugite-native follows the kit. Outside Windows the root is compared case-sensitively: `/media/u/Kit` → `/media/u/kit` is a move.

## 2. How each one is installed

Abbreviations:
- **IA**: installation from archive (`Install-Archive`, `_languages.ps1:43-68`). Reading the version, skip if up to date, download, check (SHA-512 if available, otherwise SHA-256; missing hash = error), extraction, replacement of the folder. `inner`: `''` = archive as is, `*` = single top-level folder, otherwise name of the subfolder. The key file (and the `keys` parts in the same folder) must exist before the swap.
- **GHA**: `api.github.com/repos/<repo>/releases/latest`; first asset matching the regex; SHA-256 from `asset.digest`, and for older releases (without `digest`) from the release's checksum file: `<asset>.sha256` (Kotlin), `<name>.sha256sum` (Deno), `SHASUMS256.txt` (Bun); version = tag without `v` / `bun-v`.
- **Versions** (`versioned` components, D-80): the list of downloadable `major.minor` lines and the latest build of each come from the same source as the latest version: for GHA the last 100 non-prerelease releases (`releases?per_page=100`); Julia from `versions.json` (only `stable` with a `winnt` zip); Dart from the folders of `channels/stable/release/` in the `dart-archive` bucket (Google Cloud Storage JSON API, from Dart 2); MySQL from the lines pinned in the catalog; MongoDB from `current.json` (on Linux from the conda-forge versions); PHP outside Windows from the lines pinned in `sha256ByFile` (8.4, 8.5); Ruby outside Windows from conda-forge (3.2 and later). Only a verifiable version is offered: a release without SHA-256 (neither `digest` nor checksum file: Typst up to 0.13, RubyInstaller 3.1) does not appear. Apache Lounge publishes only the latest build of Apache 2.4: Apache has a single version. At startup the app updates in the background the lists that are missing or older than one day (`version-lines.json`), except with `ATOLLO_NO_UPDATE`. Live check of all sources: `cargo test live_lines -- --ignored --nocapture` (with `GITHUB_TOKEN`).
- **Folder per line** (D-80, D-91): `versioned` and `minorFolder` components live in `A\<folder>\<major.minor>\` (`A\git\2.56`, `A\node\22.11`); the table gives the component's folder, the key files are relative to the line's folder. A patch replaces its line; for `minorFolder` components a new line removes the old one.
- **List**: lines `^([0-9a-f]{64})\s+\*?(\S+)\s*$`, exact file name.

| Id | Version | Download (x64 / ARM64) | Check | Extraction and next steps | Key file | Origin |
| --- | --- | --- | --- | --- | --- | --- |
| claude | npm `@anthropic-ai/claude-code-win32-<arch>/latest` | `dist.tarball`, native for both | npm integrity `sha512-…`, then Authenticode `Anthropic` on `claude.exe` (warning if revocation cannot be checked) | `package/claude.exe` → `A\claude\claude.exe`, old one renamed `.old` | `claude/claude.exe` | `_prepare-core.ps1:10-37` |
| codex | npm `@openai/codex/latest`, then `<ver>-win32-<arch>` | `dist.tarball`, native | npm integrity | `package/vendor/` with a single folder containing `bin/codex.exe` → `A\codex` | `codex/bin/codex.exe` | `_prepare-core.ps1:45-72` |
| antigravity | manifest `antigravity-cli-auto-updater-974169037036.us-central1.run.app/manifests/windows_amd64.json` (`windows_arm64.json`) | the manifest's `url` | SHA-512 from the manifest (required) | single exe → `A\antigravity\agy.exe`, old one renamed `.old`; do not run `agy install` | `antigravity/agy.exe` | `_prepare-core.ps1:81-104` |
| git | GHA `git-for-windows/git`, asset `PortableGit-<v>-(64-bit\|arm64).7z.exe`; version from the asset name | release asset | SHA-256 from `digest`; without the GitHub API the installation fails (no fallback) | 7z SFX (in Rust: direct 7z extraction, no window); check of `bin\bash.exe`; then `post-install.bat` in the folder (the SFX would run it; it deletes itself, so it always exits with 1) and Git configuration (§3). **macOS/Linux**: GHA `desktop/dugite-native`, `dugite-native-v<v>-<sha>-(macOS-arm64\|macOS-x64\|ubuntu-x64\|ubuntu-arm64).tar.gz`, the asset's `.sha256`; symbolic links kept; prepared separately if the PC cannot run the target; portable `.gitconfig`; follows the kit (D-112) | `git/bin/bash.exe` (Unix: `git/bin/git`) | `_prepare-core.ps1:109-158` |
| python | `raw.githubusercontent.com/astral-sh/python-build-standalone/latest-release/latest-release.json` → `SHA256SUMS`, file `cpython-3.13.N+…-<triple>-install_only_stripped.tar.gz` (latest) | triple `x86_64-pc-windows-msvc` / `aarch64-pc-windows-msvc` | SHA-256 from `SHA256SUMS` | tar.gz `python\`; packages kept (§3); `python.exe` → copy `python3.exe`; pip offline from `Lib\ensurepip\_bundled\pip-*.whl` | `python/python.exe` | `_prepare-core.ps1:163-234` |
| node | `nodejs.org/dist/index.json`, first LTS with `win-<arch>-zip` | `nodejs.org/dist/<v>/node-<v>-win-<arch>.zip` | SHA-256 from `SHASUMS256.txt` (List) | zip `node-<v>-win-<arch>`; version with the leading `v` | `node/node.exe` | `_prepare-core.ps1:239-268` |
| uv | GHA `astral-sh/uv` | `uv-<triple>.zip` | SHA-256 from `digest` | `uv.exe`, `uvx.exe`, `uvw.exe` → `A\uv\<line>\` | `uv/uv.exe` | `_prepare-core.ps1:273-302` |
| gh | GHA `cli/cli` | `gh_<v>_windows_amd64.zip` / `_arm64.zip` | SHA-256 from `digest` | zip (`bin\gh.exe`) → `A\gh\<line>\` | `gh/bin/gh.exe` | `_prepare-core.ps1:307-335` |
| vscodium-server | pinned in the catalog (`version`, today 1.135.06055; D-58), never the latest | `github.com/VSCodium/vscodium/releases/download/<v>/vscodium-reh-web-win32-x64-<v>.tar.gz`, x64 only (on ARM64 too) | SHA-256 pinned in the catalog (`sha256`, same as VSCodium's and as GitHub's `digest`) | tar.gz (root: `bin\`, `node.exe`, `out\`, `extensions\`) → `A\vscodium-server\`; before the swap the old editor's processes are stopped | `vscodium-server/out/server-main.js` + `vscodium-server/node.exe` | new |
| pwsh | GHA `PowerShell/PowerShell`, `PowerShell-<v>-win-(x64\|arm64).zip` | native for both | SHA-256 from `digest` | zip → `A\pwsh\` | `pwsh/pwsh.exe` | new |
| php | `windows.php.net/downloads/releases/releases.json`, most recent (or chosen) branch, Thread Safe build `ts-vs*-x64` (or `ts-vc*-x64`) | x64 only (emulated on ARM64, with a warning) | SHA-256 from `releases.json` | zip; VC++ DLLs next to `php.exe`; `cacert.pem` from curl.se with SHA-256 (if it fails, only a warning); `php.ini` created once (§3); version saved together with Composer's | `php/php.exe` + `tools/composer.phar` | `_prepare-web.ps1:89-167` |
| composer (part of php) | `getcomposer.org/versions`, first `stable` | `getcomposer.org<path>`, architecture-independent | SHA-256 from `<url>.sha256sum` | `A\tools\composer.phar` + `composer.cmd` (`@php "%~dp0composer.phar" %*`) and `composer` for Git Bash | `tools/composer.phar` | `_prepare-web.ps1:153-164` |
| VC++ (for php, apache) | none: extracted once into `A\tools\vcruntime\` and reused while it is there | `aka.ms/vc14/vc_redist.x64.exe` | Authenticode `O=Microsoft Corporation` required, on the exe and on `vcruntime140.dll` | embedded CABs (`MSCF` signature) extracted with Windows' `expand.exe`, Burn manifest, payload `vcRuntimeMinimum_amd64\cab1.cab`; `*.dll_amd64` → `*.dll` in `A\tools\vcruntime\` | `tools/vcruntime/vcruntime140.dll` | `_prepare-web.ps1:13-83` |
| apache | HTML of `www.apachelounge.com/download/`, first `httpd-X.Y.Z-N-Win64-VSnn.zip`. **macOS/Linux** (D-116): lines pinned in `sha256ByLine` (`2.4` → `2.4.69-<build>`), `github.com/nabre/atollo/releases/download/httpd-v<v>/httpd-<v>-<target>.tar.gz` | Win64 only (x64 on ARM64 too); Unix native for the four targets | SHA-256 from `<url>.txt`; Unix: pinned SHA-256 | zip `Apache24`; VC++ DLLs in `bin\`; `httpd.conf`: `SRVROOT`, `Listen 80` commented out, `rewrite_module`, `Include …/data/apache/kit.conf`; `kit.conf` created once (§3). Unix: tar.gz with the Apache root at the top level and the same `httpd.conf` (LF line endings); `kit.conf` without PHP | `apache/bin/httpd.exe` (Unix: `apache/bin/httpd`) | `_prepare-web.ps1:174-276` |
| rust | `static.rust-lang.org/dist/channel-rust-stable.toml` (`[pkg.rustc]`); installer `static.rust-lang.org/rustup/dist/<arch>-pc-windows-msvc/rustup-init.exe` | host: x64 → `x86_64-pc-windows-gnu`, ARM64 → `aarch64-pc-windows-gnullvm` (linker: clang from llvm-mingw, `cc` component); `msvc` from the settings on both | SHA-256 from `rustup-init.exe.sha256` | `-y --no-modify-path --profile minimal --default-host <host> --default-toolchain stable -c rustfmt -c clippy`, `RUSTUP_HOME`, `CARGO_HOME` in the kit; `rustup set auto-self-update disable`; if already installed, `rustup update stable` in place; HKCU Rustup entry (§3). Kit on FAT32/exFAT: error. Kit of another system or architecture: `rustup-init` is downloaded and checked in the `.bootstrap` folder and the installation completes at the first start on the target (D-114) | `rust/cargo/bin/rustup.exe` | `_prepare-rust.ps1:13-198` |
| go | `go.dev/dl/?mode=json`, first `stable` | file `windows`/`amd64\|arm64`/archive | SHA-256 from the JSON | IA `go` | `go/bin/go.exe` | `_languages.ps1:77-89` |
| java | `api.adoptium.net/v3/info/available_releases` → LTS; `assets/latest/<lts>/hotspot?architecture=x64\|aarch64&image_type=jdk&os=windows&vendor=eclipse` | first `.zip` | SHA-256 `binary.package.checksum` | IA `*` | `java/bin/java.exe` + `maven/bin/mvn.cmd` + `gradle/bin/gradle.bat` | `_languages.ps1:94-105` |
| maven (part of java) | `repo.maven.apache.org/…/apache-maven/maven-metadata.xml`, highest numeric version | `apache-maven-<v>-bin.zip` | SHA-512 from `.sha512` | IA `apache-maven-<v>`; `conf\settings.xml`: `<localRepository>${env.CP_DATA}/maven/repository</localRepository>` | `maven/bin/mvn.cmd` | `_languages.ps1:107-120` |
| gradle (part of java) | `services.gradle.org/versions/current` | `downloadUrl` | SHA-256 from `checksumUrl` | IA `gradle-<v>` | `gradle/bin/gradle.bat` | `_languages.ps1:122-125` |
| kotlin | GHA `JetBrains/kotlin`, `kotlin-compiler-*.zip` | independent | `digest` | IA `kotlinc` | `kotlin/bin/kotlinc.bat` | `_languages.ps1:130-136` |
| android | page `developer.android.com/studio`: `commandlinetools-win-<build>_latest.zip` from `dl.google.com/android/repository/` | independent | SHA-256 from the page (the XML repository has only SHA-1) | into `android\cmdline-tools\latest`, then `sdkmanager --licenses`, `--install platform-tools`, latest stable `platforms;android-N` and `build-tools;X.Y.Z`, `--update` on updates (the kit's Java 17+) | `android/cmdline-tools/latest/bin/sdkmanager.bat` + `android/platform-tools/adb.exe` | **new** |
| dotnet | `builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json`, active LTS channel → `latest-sdk` → `releases.json` | `rid=win-x64\|win-arm64`, `.zip` | SHA-512 `hash` | IA `''` | `dotnet/dotnet.exe` | `_languages.ps1:139-157` |
| dart | `storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION` | `dartsdk-windows-<x64\|arm64>-release.zip` | SHA-256 from `.sha256sum` | IA `dart-sdk` | `dart/bin/dart.exe` | `_languages.ps1:160-168` |
| zig | `ziglang.org/download/index.json`, highest numeric version | `x86_64-windows` / `aarch64-windows` | SHA-256 `shasum` | IA `*` | `zig/zig.exe` | `_languages.ps1:171-181` |
| julia | `julialang-s3.julialang.org/bin/versions.json`, latest `stable` | `winnt`/`x86_64\|aarch64`/zip | SHA-256 | IA `*` | `julia/bin/julia.exe` | `_languages.ps1:184-195` |
| bun | GHA `oven-sh/bun`, `bun-windows-(x64\|aarch64).zip` | | `digest` | IA `*` | `bun/bun.exe` | `_languages.ps1:198-205` |
| deno | GHA `denoland/deno`, `deno-(x86_64\|aarch64)-pc-windows-msvc.zip` | | `digest` | IA `''` | `deno/deno.exe` | `_languages.ps1:206-213` |
| typst | GHA `typst/typst`, `typst-(x86_64\|aarch64)-pc-windows-msvc.zip` | | `digest` | IA `*` | `typst/typst.exe` | `_languages.ps1:216-223` |
| tinytex | GHA `rstudio/tinytex-releases`, `TinyTeX-1-windows-v*.exe` | x64 on ARM64 too | `digest` | 7z SFX → `TinyTeX\` with `bin\windows\tlmgr.bat` | `tinytex/bin/windows/tlmgr.bat` | `_languages.ps1:228-250` |
| pnpm | npm registry, `pnpm/latest` and `corepack/latest` | npm registry | npm integrity | with the kit's Node (`node npm-cli.js --global`, without `cmd.exe`), prefix `data\npm-global`: `install --ignore-scripts corepack@latest`, `corepack enable yarn`, then `install --force --allow-scripts=pnpm pnpm@latest` | `@data/npm-global/pnpm.cmd` + `@data/npm-global/yarn.cmd` | `_languages.ps1:254-281` |
| cmake | GHA `Kitware/CMake`, `cmake-*-windows-(x86_64\|arm64).zip` | | `digest` | IA `*` | `cmake/bin/cmake.exe` + `tools/ninja.exe` | `_languages.ps1:286-291` |
| ninja (part of cmake) | GHA `ninja-build/ninja`, `ninja-win.zip` / `ninja-winarm64.zip` | | `digest` | `ninja.exe` → `A\tools\` | `tools/ninja.exe` | `_languages.ps1:292-298` |
| cc | GHA `mstorsjo/llvm-mingw`, `llvm-mingw-*-ucrt-(x86_64\|aarch64).zip`; macOS/Linux: shims `cc`, `c++`, `gcc`, `g++`, `clang`, `clang++`, `ar`, `ranlib` over the kit's Zig and GNU make from conda-forge (D-112; Apple frameworks need Xcode CLT) | | `digest`; micromamba | IA `*`; **delete** `bin\*-w64-mingw32-gcc.exe` and `-g++.exe` (they would break Rust's `gnu` linker) | `cc/bin/clang.exe` | `_languages.ps1:308-317` |
| ruby | `api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=30`, excluding prereleases/drafts, highest version and build | `rubyinstaller-X.Y.Z-N-(x64\|arm).7z` | `digest` | IA `*`; version `X.Y.Z-N`; without MSYS2 | `ruby/bin/ruby.exe` | `_languages.ps1:321-340` |
| micromamba (for r; on macOS and Linux for ruby and make; on Linux for the mongodb server; on Linux ARM64 for sqlite) | tag of the latest release of `mamba-org/micromamba-releases` (GitHub API) | `micromamba-win-64`, `micromamba-win-arm64` (`micromamba-osx-*`, `micromamba-linux-*` outside Windows) | SHA-256 from the release's `micromamba-win-64.sha256` | exe → `A\tools\micromamba.exe`; the reinstallation without network uses it too (`create --offline -f <list>`, D-118) | `tools/micromamba.exe` | `_languages.ps1:344-358` |
| r | `api.anaconda.org/package/conda-forge/r-base` → `latest_version` | `win-64` only (on Windows ARM64 too, under x64 emulation, D-132) | SHA-256 of each package (micromamba) | `micromamba create --no-rc -y -q -p <work>\r -c conda-forge --override-channels r-base=<v> "libblas=*=*openblas"` with profile, `MAMBA_ROOT_PREFIX` and `CONDA_PKGS_DIRS` in the work area; for the components that do not follow the kit (outside Windows) archives and explicit list of the line in `data/app/conda/<target>/`, to reinstall without network (D-118); launchers `bin\R.cmd`, `bin\Rscript.cmd`; CRAN `cloud.r-project.org` in `Rprofile.site`; `pkgs\` removed | `r/bin/R.cmd` | `_languages.ps1:368-429` |
| mysql | lines and versions pinned in the catalog (`sha256ByLine`: 8.4.11, 9.7.2) | `cdn.mysql.com/Downloads/MySQL-<line>/mysql-<v>-winx64.zip` (x64 on ARM64 too) | pinned SHA-256, computed after Oracle's GPG signature | IA; `mysql-test`, symbols and debug builds removed; on Linux `lib/private/libaio.so.1` pointing to `libaio.so.1t64` if needed | `mysql/bin/mysqld.exe` + `mysql.exe`, `mysqldump.exe`, `mysqladmin.exe` | new (D-110) |
| caddy | GHA `caddyserver/caddy` | `caddy_<v>_windows_(amd64\|arm64).zip` | `digest`, or SHA-512 from `caddy_<v>_checksums.txt` | `caddy.exe` → `A\caddy\<line>\` | `caddy/caddy.exe` | new (D-111) |
| mongodb | `downloads.mongodb.org/current.json`, production `x.0.y`; mongosh from GHA `mongodb-js/mongosh` | `mongodb-windows-x86_64-<v>.zip`, `mongosh-<v>-win32-x64.zip` into `mongosh\`, Database Tools from `release.json` into `mongodb-tools\` | SHA-256 from the list; `digest` | single IA; `.pdb` and `vc_redist` removed, the kit's VC++ | `mongodb/bin/mongod.exe` + `mongosh/bin/mongosh.exe` + `mongodb-tools/bin/mongodump.exe` | new (D-73) |
| sqlite | `sqlite.org/download.html`, `PRODUCT,` lines | `sqlite-tools-win-(x64\|arm64)-<n>.zip`, always the one for the kit's architecture (D-108) | **SHA3-256** from the page | IA | `sqlite/sqlite3.exe` | new (D-73) |
| phpmyadmin | `www.phpmyadmin.net/home_page/version.json` | `files.phpmyadmin.net/phpMyAdmin/<v>/phpMyAdmin-<v>-all-languages.zip` (all languages) | `.sha256` file next to the zip | zip → `phpmyadmin\<line>\`, without `setup\`; `config.inc.php` written by the app at each start; `php -l` check on `libraries/classes/Version.php` with the kit's PHP | `phpmyadmin/index.php` | new (D-107) |
| CodeLLDB (extension) | GHA `vadimcn/codelldb` | `codelldb-win32-<arch>.vsix`; x64 on ARM64 too, like the editor's server | `digest` required | installed in the editor's server with `--install-extension`, if a project has Rust, Zig, C or C++ (`extensions.json` matrix, D-71), only once | folder `vadimcn.vscode-lldb-*` in `data\vscodium-server\extensions\` | `_vscodium-extensions.ps1:162-206` |

## 3. Cross-cutting rules

- **Order**: that of `components.json`: claude, codex, antigravity, git, python, node, uv, gh, php, apache, caddy, rust, go, java (+maven, gradle), kotlin, android, dotnet, dart, zig, julia, bun, deno, typst, tinytex, pnpm, cmake (+ninja), cc, ruby, r, mysql, mongodb, phpmyadmin, sqlite, pwsh, vscodium-server. Removals first, then installations; Git configuration and the Apache site are final steps of those components. From the command line, at the end of `prepare`, the final check of what was installed. Editor extensions after the editor starts; alignment of the agents at app startup and around each agent. (In nabre/atollo: `Prepare-Atollo.ps1:250-285`.)
- **One error does not stop the others**: each step is isolated and ends up in the summary; overall result "with errors" if one fails.
- **When it is installed**: key file or a `keys` part missing, editor at a version different from the release's (at app startup, or in a `prepare` that includes it), or "Update all" (`--update-all`) / `--force`. `--only` installs or updates the listed components and whatever they need and is missing, required ones excluded, without removing anything; `--skip` also skips what depends on them and is not yet installed (never the required ones).
- **When needed** (D-59): starting an agent, a clone, a project action first install the components they need and mark them as chosen in `components.txt`.
- **Default parameters**: Claude channel `latest`; Python: the latest stable minor line, unless `pythonVersion` is set in Settings (`^3\.\d+$`, D-81). `PYTHON_DEFAULT` (3.13) applies only when the list of lines cannot be read: a new kit has 3.15 since October 2026; PHP most recent branch (`^\d+\.\d+$`); Rust `gnu` on Windows x64 and `gnullvm` on Windows ARM64; host architecture. In the app: Settings › Versions page.
- **VC++** extracted once into `tools\vcruntime\` and reused; DLLs copied next to `php.exe` and into `apache\bin` (local distribution allowed by Microsoft); x64 on ARM64 too. The folder stays as long as PHP or Apache is there.
- **User files created only once and never overwritten**: `data\php\php.ini` (from `php.ini-development`: `extension_dir`, extensions curl, fileinfo, gd, intl, mbstring, mysqli, openssl, pdo_mysql, pdo_sqlite, sodium, sqlite3, zip; `date.timezone = Europe/Rome`; `curl.cainfo` and `openssl.cafile` on `cacert.pem`), `data\apache\kit.conf` (port 8080 on `127.0.0.1`, site in `data\apache\htdocs`, PHP via `<IfFile>`, `conf.d\*.conf`), `data\home\.gitconfig`, `htdocs\index.php`. Texts in UTF-8 without BOM. `httpd.conf` instead is rewritten at each Apache installation.
- **Python**: before updating, `pip freeze` into `data\python-packages-<old>.txt`; same minor version → copy of `Lib\site-packages` and `Scripts` (pip excluded); different minor → only the list.
- **Rust**: `gnu` default on Windows x64 (brings its own linker); `gnullvm` default on Windows ARM64, linked by the llvm-mingw of `cc` (D-112); `msvc` from the settings on both (requires Visual Studio); `rustup-init` always an msvc build. Without hard links (FAT32/exFAT) the installation stops with an error; the fallback of the original scripts (toolchain on the PC's disk, copy of `rustup\` into the kit) is not ported.
- **Rustup entry in HKCU** (`…\Uninstall\Rustup`): rustup always writes it; it is saved before and restored or removed after (only if it points to the kit). It is a temporary registry write **made by rustup, not by the app** (DECISIONS.md D-38).
- **Git configuration**: system `credential.helper` set to `manager` if it is empty, `helper-selector` or a GCM path; `.gitconfig` with `init.defaultBranch=main` and `safe.directory=*` (FAT32/exFAT do not record the owner); `GCM_CREDENTIAL_STORE=dpapi` in the environment.
- **Signature**: Authenticode with expected publisher; revocation that cannot be checked = warning, except where the signature is the only check (VC++).
- **Atomic swaps**: the new tree is prepared in `data\tmp\setup\<id>` (same disk), the old folder is renamed `<name>.old-<ts>` (fails at once if a file is in use), then move, restore on error; leftover `.old-*`/`.new-*` deleted at the next installation. Single exes: old one renamed `.old`, so it is replaced even if in use.
- **FAT32/exFAT**: extraction without restoring file dates; hard link test before Rust; work area always in `data\tmp\setup\`.
- **Proxy**: app settings → `HTTPS_PROXY`/`HTTP_PROXY` → Windows proxy (read only); no PAC (D-29). The settings also end up in the kit's environment (`HTTPS_PROXY`, `HTTP_PROXY`, `NO_PROXY`), which rustup, pip and npm receive. Downloads into `.part`, renamed at the end of the download, 3 attempts (5 s, 10 s).
- **One preparation at a time** per kit (mutex on the root path).
- **Versions as strings**: node with the `v`; Go without `go`; GHA without `v`/`bun-v`; ruby `X.Y.Z-N`; components with several parts join the versions (`8.5.0 (Composer 2.10.3)`). Comparisons with `data\app\versions\` use the same strings.

## 4. Differences from the original scripts

| Original scripts | New kit | Reason |
| --- | --- | --- |
| VSCodium desktop required | removed | built-in editor |
| Git, Node.js, uv required | installed when needed (DECISIONS.md D-59) | only PowerShell 7 and GitHub CLI are required |
| — | `pwsh` required; `vscodium-server` part of the app with pinned version (D-58) | the kit's exclusive terminal (classic Windows console, D-57, D-95), built-in editor |
| Git: fallback without API (tag + Authenticode) | only the GitHub API with `digest` | — |
| Git SFX and TinyTeX SFX open the 7-Zip window | 7z extraction in Rust, no window | clean interface |
| Typst and TinyTeX are not removed when deselected | removed | bug in `_prepare-migrate.ps1:128` |
| Rust on FAT32/exFAT: toolchain on the PC's disk | error | to be ported |
| `Test-Installation` with `rustc --version` etc. | same check: `Atollo.exe check` and at the end of `prepare` | |
| Versions in `data\versions\` | `data\app\versions\` | everything that belongs to the app lives in `data\app\` |
