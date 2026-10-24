<p align="center">
  <a href="https://nabre.github.io/atollo/">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="docs/brand/wordmark-dark.svg">
      <img src="docs/brand/wordmark-light.svg" alt="Atollo" width="280">
    </picture>
  </a>
</p>

<h1 align="center">Atollo — Universal Portable Dev & AI Workspace</h1>

<p align="center">
  A portable, zero-install development environment for Windows, macOS and Linux, on x64 and ARM64.<br>
  Editor, 20+ languages and runtimes, local databases, local AI models and the Claude Code, Codex, Antigravity and Pi AI agents in one folder you can carry on a USB drive.
</p>

<p align="center">
  <a href="https://github.com/nabre/atollo/releases"><img alt="Latest release" src="https://img.shields.io/github/v/release/nabre/atollo?include_prereleases&label=release"></a>
  <a href="https://github.com/nabre/atollo/releases"><img alt="Downloads" src="https://img.shields.io/github/downloads/nabre/atollo/total?label=downloads"></a>
  <a href="https://github.com/nabre/atollo/actions/workflows/app.yml"><img alt="Build" src="https://img.shields.io/github/actions/workflow/status/nabre/atollo/app.yml?branch=main&label=build"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/github/license/nabre/atollo?label=license"></a>
  <img alt="Platforms: Windows, macOS, Linux" src="https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-informational">
  <img alt="Architectures: x64, ARM64" src="https://img.shields.io/badge/arch-x64%20%7C%20ARM64-informational">
  <img alt="Built with Rust and Tauri 2" src="https://img.shields.io/badge/built%20with-Rust%20%2B%20Tauri%202-informational">
</p>

<p align="center">
  <a href="https://nabre.github.io/atollo/"><b>Website</b></a> ·
  <a href="https://github.com/nabre/atollo/releases/latest"><b>Download</b></a> ·
  <a href="https://nabre.github.io/atollo/guide/"><b>User guide</b></a> ·
  <a href="https://github.com/nabre/atollo/issues"><b>Issues</b></a> ·
  <a href="docs/PRIVACY.md"><b>Privacy</b></a>
</p>

<p align="center">English · <a href="docs/README.it.md">Italiano</a> · <a href="docs/README.fr.md">Français</a> · <a href="docs/README.de.md">Deutsch</a></p>

**Atollo** is a self-contained *Universal Portable Dev & AI Workspace* (`Atollo.exe`) that unifies an **Editor** (integrated VSCodium), **Toolchain/Runtime Manager** (20+ on-demand languages and tools), **Local Servers/Databases** (MySQL, MongoDB, SQLite, Caddy/Apache), and an **AI Agent Orchestrator** (Claude Code, Codex, Antigravity, and Pi with synchronized instructions, skills, and MCP servers, plus local LLMs through llama.cpp) into a single portable folder. Zero install, zero administrator rights, zero system services outside the workspace folder.

On first start the app downloads the system components by itself: PowerShell 7, GitHub CLI and the editor. Everything else is chosen on the "Tools" page, or arrives when needed: Node.js with an `npm` action, uv with a `uvx` MCP server, and so on.

<picture>
  <source media="(prefers-color-scheme: light)" srcset="site/shots/en/shot-projects-light.png">
  <img src="site/shots/en/shot-projects-dark.png" alt="Atollo's Projects page: quick panel and activity" width="100%">
</picture>

## Features

- **Isolated editor and terminals.** Built-in VSCodium with the Atollo dark and light themes, in a window measured on VSCodium 1.135's own workbench: the same sizes, colors and codicon icons. Terminals see only the kit's `PATH` and variables: programs on the host PC don't interfere, and the PC stays clean.
- **Languages and tools on demand.** 20+ languages and tools (Node.js, Python, uv, PHP, Rust, Go, Java, .NET, Dart, Julia, Ruby, C/C++, Bun, Deno, Typst, TinyTeX, R, Android SDK…), downloaded when needed and verified by SHA-256 checksum or signature. Side-by-side versions, one per project.
- **Local databases and web servers.** MySQL with phpMyAdmin, MongoDB with mongo-express, and SQLite, started and stopped with one click, with a database, user and password for each project; several versions side by side, each keeping its own port. Caddy and Apache for sites at `<project>.localhost`, on every system, on their own or in a server shared by several projects. A port already in use is refused, naming whoever holds it. No system services.
- **Four AI agents, one configuration.** Claude Code, Codex, Antigravity and Pi: install, sign in and start. Write instructions, skills and MCP servers once in `data/agents` and `atollo sync` keeps all four aligned. Ready MCP servers are one click away: GitHub, Playwright and Fetch for the whole kit, the project's files and its MySQL or MongoDB database for a project, with no token or password written in the files. Plan limits and usage for today and the last 7 days.
- **Local AI models.** Language models that run on the PC, with no network and no account: GGUF models from Atollo's catalog or from any Hugging Face repository, downloads that resume where they stopped (even after closing Atollo) and are verified by SHA-256. The llama.cpp server (CPU, or GPU with Vulkan or Metal) listens on 127.0.0.1 only, with a key; Pi uses it out of the box, and its chat opens in the browser. One copy in `data/models`, shared by Windows, macOS and Linux.
- **One folder for six systems.** The same folder, or the same external SSD, moves from Windows to macOS to Linux with projects, databases and agent settings intact: each machine runs the native programs of its system. "Remove drive" closes everything Atollo opened and lets you eject the disk.

## Documentation

| Document | What it covers |
| --- | --- |
| [User guide](https://nabre.github.io/atollo/guide/) | every page of the app, step by step, with screenshots, on the website (also inside the app) |
| [Component catalog](docs/app/CATALOG.md) | the components, where each one is downloaded from, availability per system |
| [Tools × systems matrix](docs/app/TOOLS-MATRIX.html) | every component on each of the six targets: native, emulated or missing |
| [Architecture](docs/app/ARCHITECTURE.md) | how the code is organized and where a change goes |
| [Testing](docs/app/TESTING.md) | automated checks and the manual checklist per system |
| [Decisions](docs/app/DECISIONS.md) and [changelog](docs/app/CHANGELOG.md) | the reasons behind each choice and what changed, in Italian |

Open work is tracked in the [issues](https://github.com/nabre/atollo/issues).

## How to use it

1. Copy Atollo's exe (for example `Atollo-0.6.0-win32-x64.exe`) into an empty folder: a USB stick, an external drive or a folder in your profile (spaces, brackets or `&` in the name are fine).
2. Double-click it: the app creates `projects\` and `data\`. On first start choose agents, languages and tools: the app downloads and verifies them (checksum or signature).
3. Create or clone a project on the **Projects** page. A project's editor, terminal and agents always open in its folder.

From the [releases page](https://github.com/nabre/atollo/releases): `Atollo-<version>-win32-x64.exe` or `Atollo-<version>-win32-arm64.exe` for Windows, `Atollo-<version>-darwin-arm64.app.zip` (Apple Silicon) or `Atollo-<version>-darwin-x64.app.zip` (Intel) for macOS, `Atollo-<version>-linux-x64` or `Atollo-<version>-linux-arm64` (`.AppImage` and `.tar.gz`) for Linux; each file has its `.sha256`. The name tells the version: when the app updates itself, the exe and the AppImage take the new version's name. In the examples below `Atollo.exe` stands for your exe.

**macOS** (12 or later). The app only has an ad hoc signature, not notarized by Apple:
1. Unzip it and move `Atollo.app` with the Finder into the kit folder (for example `/Volumes/KIT/Atollo/`).
2. In Terminal, remove the browser's quarantine: `xattr -dr com.apple.quarantine /Volumes/KIT/Atollo/Atollo.app`.
3. Open `Atollo.app` from that folder: `projects/` and `data/` are created next to it, never inside.

Without step 2 macOS starts the app from a temporary copy (App Translocation) and Atollo stops showing the same command, instead of creating the kit in a folder that goes away.

**Linux** (x64 and ARM64, glibc 2.35 or later: Ubuntu 22.04, Debian 12 and newer). Copy `Atollo-<version>-linux-<arch>.AppImage` into the kit folder, `chmod +x Atollo-<version>-linux-<arch>.AppImage` and start it: the kit is the AppImage's folder. Without FUSE (containers, some minimal distributions): run `./Atollo-<version>-linux-<arch>.AppImage --appimage-extract-and-run`, or use `Atollo-<version>-linux-<arch>.tar.gz` with WebKitGTK 4.1 installed (on Ubuntu and Debian `sudo apt install libwebkit2gtk-4.1-0`): extract `Atollo` into the kit folder and run `./Atollo`. The AppImage updates itself; the `tar.gz` says when a new version is there to download.

## Pages

| Page | What it has |
| --- | --- |
| **Projects** | the **quick panel** (what is running, favourite and recent projects with Run, Editor, Terminal and agent, what needs fixing), the project list with ▶ on every row, Ctrl+P to open one by name, Ctrl+Shift+P for the command palette (start or stop a project, open it, an agent in it, go to a page, stop everything); **New project**, a tab beside the projects: empty, from your GitHub repositories (with clone, rename and delete on GitHub), from a Git URL or from one of 63 grouped bases (Laravel, Next.js, Django, WordPress, Astro, Tauri…); notices about the workspace. Each project has its own page, with Start, Editor, Terminal and agent at the top and seven sections: **Run** (actions, ports and their output), **Website**, **Components** (with the versions), **Databases** (the project's database with user and password, backups, a safety copy before every restore and change of version), **Extensions**, **Editor** (settings and MCP servers), **Git** |
| **AI** | all of the kit's AI in two panes. **Agents**: the installed Claude Code, Codex, Antigravity and Pi: version, sign-in (with Antigravity's Gemini key), plan limits and usage of today and the last 7 days; instructions, skills and MCP servers shared in `data\agents`. **Local models**: the llama.cpp server (start and stop, address, the browser chat and its key), the model Pi uses, Atollo's catalog of GGUF models with "fits / at the limit / too large" for this PC's RAM or video memory, any Hugging Face repository (with an optional token for gated models), downloads you can pause and resume; models in `data\models`, on this PC only |
| **Databases** | Atollo's MySQL (with phpMyAdmin) and MongoDB (with mongo-express) servers: start and stop, administrator password, port, several versions side by side each on its own port, data in `data\db`, on this PC only; SQLite without a server |
| **Web Servers** | the Caddy and Apache servers shared by the projects (pools) and your own profiles, with the `<project>.localhost` sites they serve; on this PC only, or on the local network if Settings allow it |
| **Terminals** | consoles with the kit's tools in any folder of the PC: "All tools" and terminals of your own, with the tools and versions you choose |
| **Tools** | languages and tools by kind (system and editor, AI agents, local models, languages, development tools, web servers, databases, database viewers), with their logo, dependencies and disk space; install, update and remove, several versions; the Rust compiler (GNU or MSVC) on Rust's page; the editor extensions matrix |
| **Guide** | the user guide inside the app, in nine chapters; every page opens its own topic |
| **Settings** | saved as you change them: language, theme, the programs' temporary files, the packages kept to reinstall without the network, network (proxy, certificates, local network), app updates, privacy; Atollo's **GitHub account**, one for everything (app, Git, editor, consoles, agents, GitHub CLI), and who signs the commits; where the workspace folder is; the frameworks' instructions; **The kit**: its folders, its disk, the platform and the versions, with Kit health, Copy of the kit, Export diagnostics and Report a problem |
| **Remove drive** | closes everything Atollo opened, brings Antigravity's sign-in back into `data\` and lets you eject the drive |

At the bottom of every page, the **Activity** panel: what is running or installing (project actions, every database version, web server pools, components), with output and errors in one place; "Stop all" stops everything after a confirmation. The status bar counts the same things.

Atollo's console is a tab of the app's terminal panel on every system, with the kit's PowerShell 7 (on Windows on Microsoft's ConPTY, the console of Windows Terminal, downloaded with the kit); it never opens the PC's terminal and has only Atollo's isolated environment: programs installed on the host PC do not interfere. If a terminal's saved version has been removed, it says which one it uses instead (for example PHP 8.2 → 8.4).

## Still to do

Atollo is in alpha (version 0.6.0). Windows x64 is the most tested system, real installs in CI included; macOS, Linux and Windows ARM64 are built and tested in CI only. The full checklist is in [docs/app/TESTING.md](docs/app/TESTING.md).

- **The alpha, to be tried.** [The 0.6.0 alpha](https://github.com/nabre/atollo/releases/tag/app-v0.6.0) is on GitHub for Windows, macOS and Linux, each on x64 and ARM64. The published files are built and tested in CI: now they need downloading and starting on clean machines.
- **Alpha testing.** We need people using Atollo on real projects and different PCs, and reporting what goes wrong: component installs, projects, databases, agents. Settings › The kit › “Report a problem” opens a GitHub issue with the version and the system already filled in; attach the zip of “Export diagnostics” next to it (secrets masked, made on your PC, never sent by the app).
- **macOS on real Macs** ([#5](https://github.com/nabre/atollo/issues/5), [#6](https://github.com/nabre/atollo/issues/6)). Apple Silicon and Intel are built and started in CI, never on a clean Mac: the zip downloaded with a browser, quarantine and Gatekeeper, App Translocation, a kit on an exFAT stick, the agents' sign-in carried from one Mac to another.
- **Linux on real machines** ([#5](https://github.com/nabre/atollo/issues/5)). x64 and ARM64 on Ubuntu 24.04, Fedora 42 and Debian 13: AppImage with and without FUSE, the `tar.gz` with WebKitGTK 4.1, exFAT sticks or sticks mounted `noexec`, sign-in with GNOME, KeePassXC or KWallet.
- **Updates and the kit's builds outside Windows.** Updating the app from one version to the next on macOS and Linux is not tried yet. The kit's Apache (2.4.69, D-116) is in the catalog but tried only in CI; so is the kit's PHP with `pdo_sqlite` (8.4.26 and 8.5.11, D-133), which the app uses from 0.3.0 on.
- **Code signing.** On macOS only an ad hoc signature, without Apple notarization: `xattr -dr com.apple.quarantine` is needed. On Windows 0.6.0 ships unsigned: the SignPath signature is still to be activated, and until then SmartScreen may warn on first launch.

Have a Mac, a Linux PC or a Windows ARM64 PC? Try Atollo and tell us how it went: [open an issue on GitHub](https://github.com/nabre/atollo/issues).

## Known limits

- On Windows 10 and 11 (`x64`), Windows 11 (`ARM64`), macOS (Apple Silicon and Intel) and Linux (`x64` and `ARM64`). On Windows ARM64 the editor, PHP, Apache, MySQL, MongoDB, Android SDK, Julia, TinyTeX and R run under x64 emulation; on Linux ARM64 Android is missing. The macOS and Linux releases are tried in CI but not yet on clean machines ([#5](https://github.com/nabre/atollo/issues/5)), and the agents' sign-in still has to be tried on a Mac ([#6](https://github.com/nabre/atollo/issues/6)); components and sources per system in [docs/app/CATALOG.md](docs/app/CATALOG.md), the component × system matrix in [docs/app/TOOLS-MATRIX.html](docs/app/TOOLS-MATRIX.html). Apache on macOS and Linux is the kit's own build (D-116), with PHP through `php-fpm`.
- On Linux MySQL needs two system libraries, `libaio` and `libnuma`: on Ubuntu 24.04 and Debian 13 `sudo apt install libaio1t64 libnuma1` (on older releases `libaio1`), on Fedora and RHEL `sudo dnf install libaio numactl-libs`. If they are missing, Atollo says so with the command to run.
- On macOS and Linux an Atollo folder on exFAT or FAT (a drive formatted for Windows) works, but those file systems have no links: where a program ships some, Atollo copies the files instead (for instance Git's 142 internal links), taking more space. On a drive used only with a Mac or Linux, APFS or ext4 is better.
- Outside the Atollo folder there are only Windows' own traces, transient temporary files in the system temp folder (automatically removed on exit; with Settings › General › Temporary files of programs, the programs may use it too while they run), GitHub CLI's sign-in in the Credential Manager if you sign in from a console with `gh auth login` (the one made in Settings stays in `data\gh`), Antigravity's while `agy` is open (for the session only), on macOS Claude Code's login in the Keychain (unless you connect with a token) and the web view's data in your Library, and the registry entry rustup writes on Windows, which Atollo removes. The full list is in [PRIVACY §4](docs/PRIVACY.md#4-data-stored-on-your-device).
- A project's folder is trusted by Claude Code, Codex and Pi once you open an agent in it from Atollo (the other folders in `projects\` are not): open an agent only in projects you trust. The agents' tokens (`CLAUDE_CODE_OAUTH_TOKEN`, `GEMINI_API_KEY`, the local models' `LLAMA_API_KEY`) go only to the agents Atollo launches, not to project actions, scripts, terminals or the editor; the local models' key is also written in Pi's `models.json` (`data\home\.pi\agent`, readable by your user only), so that `pi` works from the kit's terminals. The editor instead asks the first time it opens each project and opens the ones you don't trust in "Restricted Mode".
- The agents' usage limits come from undocumented interfaces of the providers and may stop working after one of their updates.
- "Remove drive" does not see PC programs that keep only a file of the workspace open.

## Layout

```
Atollo\
  Atollo.exe      all the logic
  projects\       your projects
  data\           downloaded programs, settings, sign-ins, cache, temporary files
                  (the settings of each project: data\projects\<project>\dev-settings.json)
```

For a backup, use Settings › The kit › **Copy of the kit**: it exports `projects\` and `data\` to a folder or a zip, without `data\apps\`, `data\cache\` and `data\models\` (downloaded again), with the databases stopped for the copy, and imports such a copy into a kit at its next start. By hand, copy the same folders with Atollo closed. The copy holds the kit's passwords: keep it safe.

## Uninstallation

Atollo does not use an installer, registers no system services or scheduled tasks, and does not modify the Windows registry or `PATH`:

1. Open the **Remove drive** page (or close the application) to stop all processes started by Atollo and restore any active Antigravity session into `data\`.
2. Delete the folder containing `Atollo.exe`, `data\`, and `projects\` (after backing up any projects you wish to keep).
3. Any transient folders in `%TEMP%` are automatically removed on exit; if you ran `gh auth login` manually in a console (instead of the **Settings** page), you can remove the corresponding entry from the Windows **Credential Manager**.

## Command line

```
Atollo.exe                           opens the window
Atollo.exe env [--project <name>]    shows the environment Atollo's programs get
                                     (with a project, with its versions)
Atollo.exe components [--target <system>]
                                     lists languages and tools, with the installed ones
Atollo.exe versions [<id>]           the versions on disk; with an id, the ones to download
Atollo.exe check                     checks that the installed programs answer
    --kit                            the kit's health: drive, space, path, permissions, libraries
    --files                          the programs' files: missing or changed since the installation
Atollo.exe prepare [options]         installs and updates the chosen components
                                     and removes the ones not chosen
    --only a,b                       only these (and what they need), removes nothing
    --skip a,b                       not these
    --update-all                     also update the installed ones
    --force                          download again
    --line id=x.y                    adds that version next to the others (can be repeated)
    --target <system>                prepares for another system: win32-x64, win32-arm64,
                                     darwin-arm64, darwin-x64, linux-x64, linux-arm64
Atollo.exe sync [--project <folder>] [--verbose]
                                     aligns the agents' instructions, skills and MCP servers
Atollo.exe keyring restore|save      Antigravity's sign-in (used by the kit's agy.cmd)
Atollo.exe serve <folder> <port>     file server for a static site
Atollo.exe models                    the local models: the catalog's and the downloaded ones
Atollo.exe models pull <model>       downloads a catalog id or a Hugging Face model
                                     (owner/name:quantization), resumed if interrupted
Atollo.exe models rm <id>            deletes a downloaded model
Atollo.exe llm start|stop|status     the server of the local models (chat and agents)
Atollo.exe diagnostics               makes the zip to attach to a problem report (secrets masked)
Atollo.exe version                   shows the version
Atollo.exe help                      shows the help
```

From PowerShell add `| Out-Host` (for example `.\Atollo.exe check | Out-Host`): `Atollo.exe` is a window program, and without a pipe PowerShell neither waits for it nor reads its output or exit code.

## Building

Prerequisites: stable Rust (at least 1.93) with the MSVC toolchain (Visual Studio Build Tools, C++ workload), Node.js LTS.

```
npm ci
npm run build
```

The JavaScript dependencies live only at the root (`node_modules` and `package-lock.json`): `ui` is an npm workspace.

The exe is in `target\release\Atollo.exe`. No installer is created (`--no-bundle`).

Checks before a change:

```
npm run lint
npm test
npm -w ui run build
```

`npm run lint` runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint and Prettier; `npm test` runs `cargo test` and the interface tests; `npm -w ui run build` checks the types and builds the interface. Details in [docs/app/TESTING.md](docs/app/TESTING.md). How the code is organized and where a change goes: [docs/app/ARCHITECTURE.md](docs/app/ARCHITECTURE.md).

Development with live reload: `npm run dev`.

## If Windows blocks the exe

Until code signing is active on the downloaded release, if SmartScreen shows "Windows protected your PC", choose **More info** › **Run anyway**. If the exe was downloaded from the Internet: right-click › **Properties** › **Unblock**. On company PCs with AppLocker you need permission from the IT department.

## Code signing policy

Free code signing on Windows provided by [SignPath.io](https://signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

- **Project roles (Committer, Reviewer, and Approver):** [nabre](https://github.com/nabre)
- **Binary provenance:** official release binaries are built and signed exclusively by the automated GitHub Actions pipeline ([`.github/workflows/app.yml`](.github/workflows/app.yml)) directly from the public source code in this repository, with no manual intervention on binaries.

## Privacy policy

The full text, with every network connection and every file the app keeps, is in **[Privacy policy and terms of use](docs/PRIVACY.md)**. In short:

- **No telemetry or data collection:** the application does not collect personal data, usage statistics, telemetry, or crash reports, and never contacts any server operated by the project author. It also disables built-in telemetry in managed components (such as PowerShell 7 and the VSCodium server).
- **Local storage:** settings, projects, local databases, logs, and credentials are stored locally inside Atollo's `data\` (and `projects\`) folder on the user's device; the few exceptions (temporary files, Antigravity's sign-in while it runs) are listed in the policy.
- **Network connections:** the application only connects to GitHub to check for its own updates (Settings › General › Automatic update, or `ATOLLO_NO_UPDATE`), to the official sources of the components it downloads, to the Open VSX registry for editor extensions and, when the user signs in to GitHub (Settings › GitHub account) or opens the AI page, to GitHub and to the agents' providers (**Claude Code**, **Codex**, **Antigravity**, and the cloud provider chosen in **Pi**) under their own privacy policies. Local models come from Hugging Face (`huggingface.co` and its CDN), the llama.cpp and Pi programs from their GitHub releases (`ggml-org/llama.cpp`, `earendil-works/pi`); a local model runs on the PC, and its prompts never leave it.

## License

MIT: provided "as is", without warranty. See [LICENSE](LICENSE) and the [terms of use](docs/PRIVACY.md#8-terms-of-use).
