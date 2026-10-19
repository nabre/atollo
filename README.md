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
  Editor, 20+ languages and runtimes, local databases and the Claude Code, Codex and Antigravity AI agents in one folder you can carry on a USB drive.
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
  <a href="docs/app/GUIDE.md"><b>User guide</b></a> ·
  <a href="https://github.com/nabre/atollo/issues"><b>Issues</b></a>
</p>

<p align="center">English · <a href="docs/README.it.md">Italiano</a> · <a href="docs/README.fr.md">Français</a> · <a href="docs/README.de.md">Deutsch</a></p>

**Atollo** is a self-contained *Universal Portable Dev & AI Workspace* (`Atollo.exe`) that unifies an **Editor** (integrated VSCodium), **Toolchain/Runtime Manager** (20+ on-demand languages and tools), **Local Servers/Databases** (MySQL, MongoDB, SQLite, Caddy/Apache), and an **AI Agent Orchestrator** (Claude Code, Codex, and Antigravity with synchronized instructions, skills, and MCP servers) into a single portable folder. Zero install, zero administrator rights, zero system services outside the workspace folder.

On first start the app downloads the system components by itself: PowerShell 7, GitHub CLI and the editor. Everything else is chosen on the "Tools" page, or arrives when needed: Node.js with an `npm` action, uv with a `uvx` MCP server, and so on.

<picture>
  <source media="(prefers-color-scheme: light)" srcset="site/shots/shot-projects-light.png">
  <img src="site/shots/shot-projects-dark.png" alt="Atollo's Projects page: quick panel and activity" width="100%">
</picture>

## Features

- **Isolated editor and terminals.** Built-in VSCodium with the Atollo dark and light themes. Terminals see only the kit's `PATH` and variables: programs on the host PC don't interfere, and the PC stays clean.
- **Languages and tools on demand.** 20+ languages and tools (Node.js, Python, uv, PHP, Rust, Go, Java, .NET, Dart, Julia, Ruby, C/C++, Bun, Deno, Typst, TinyTeX, R, Android SDK…), downloaded when needed and verified by SHA-256 checksum or signature. Side-by-side versions, one per project.
- **Local databases and web servers.** MySQL with phpMyAdmin, MongoDB and SQLite, started and stopped with one click, with a database, user and password for each project. Caddy and Apache for sites, on every system. No system services.
- **Three AI agents, one configuration.** Claude Code, Codex and Antigravity: install, sign in and start. Write instructions, skills and MCP servers once in `data/agents` and `atollo sync` keeps all three aligned. Plan limits and usage for today and the last 7 days.
- **One folder for six systems.** The same folder, or the same external SSD, moves from Windows to macOS to Linux with projects, databases and agent settings intact: each machine runs the native programs of its system. "Remove drive" closes everything Atollo opened and lets you eject the disk.

## Documentation

| Document | What it covers |
| --- | --- |
| [User guide](docs/app/GUIDE.md) | every page of the app, step by step, with screenshots |
| [Component catalog](docs/app/CATALOG.md) | the components, where each one is downloaded from, availability per system |
| [Tools × systems matrix](docs/app/TOOLS-MATRIX.html) | every component on each of the six targets: native, emulated or missing |
| [Architecture](docs/app/ARCHITECTURE.md) | how the code is organized and where a change goes |
| [Testing](docs/app/TESTING.md) | automated checks and the manual checklist per system |
| [Decisions](docs/app/DECISIONS.md) and [changelog](docs/app/CHANGELOG.md) | the reasons behind each choice and what changed, in Italian |

Open work is tracked in the [issues](https://github.com/nabre/atollo/issues).

## How to use it

1. Copy Atollo's exe (for example `Atollo-0.2.0-win32-x64.exe`) into an empty folder: a USB stick, an external drive or a folder in your profile (spaces, brackets or `&` in the name are fine).
2. Double-click it: the app creates `projects\` and `data\`. On first start choose agents, languages and tools: the app downloads and verifies them (checksum or signature).
3. Create or clone a project on the **Projects** page. The editor, the terminal and the agents always open in a project's folder, never anywhere else.

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
| **Projects** | the **quick panel** (what is running, favourite and recent projects with Run, Editor, Terminal and agent, what needs fixing), the project list with ▶ on every row, Ctrl+P to open one by name; **New project**, a tab beside the projects: empty, from your GitHub repositories (with clone, rename and delete on GitHub), from a Git URL or from one of 63 grouped bases (Laravel, Next.js, Django, WordPress, Astro, Tauri…); notices about the workspace. Each project has its own page: **Summary** (the main action with its output, languages, databases, agent, Git), **Resources** (components, the project's database with user and password, editor extensions), **Advanced** (actions, ports, static site, Git, agent instructions) |
| **Agents** | the installed Claude Code, Codex and Antigravity: version, sign-in, plan limits and usage of today and the last 7 days; instructions, skills and MCP servers shared in `data\agents` |
| **Databases** | Atollo's MySQL (with phpMyAdmin) and MongoDB servers: start and stop, administrator password, port, data in `data\db`, on this PC only; SQLite without a server |
| **Tools** | languages and tools with their logo, dependencies and disk space; install, update and remove; the editor extensions matrix |
| **Settings** | language, terminal, network (proxy, certificates, local network), Gemini key, Python and PHP versions, Rust compiler (GNU or MSVC), app updates; Atollo's **GitHub account**, one for everything (app, Git, editor, consoles, agents, GitHub CLI); where the workspace folder is |
| **Remove drive** | closes everything Atollo opened, brings Antigravity's sign-in back into `data\` and lets you eject the drive |

At the bottom of every page, the **Activity** panel: what is running or installing (projects, databases, components), with output and errors in one place.

Atollo's console is the classic Windows console with integrated PowerShell 7 and only Atollo's isolated environment: programs installed on the host PC do not interfere.

## Still to do

Atollo is in alpha (version 0.2.0). Windows x64 is the most tested system, real installs in CI included; macOS, Linux and Windows ARM64 are built and tested in CI only. The full checklist is in [docs/app/TESTING.md](docs/app/TESTING.md).

- **The first release, to be tried.** [The 0.2.0 alpha](https://github.com/nabre/atollo/releases/tag/app-v0.2.0) is on GitHub for Windows, macOS and Linux, each on x64 and ARM64. The published files are built and tested in CI: now they need downloading and starting on clean machines.
- **Alpha testing.** We need people using Atollo on real projects and different PCs, and reporting what goes wrong: component installs, projects, databases, agents.
- **macOS on real Macs** ([#5](https://github.com/nabre/atollo/issues/5), [#6](https://github.com/nabre/atollo/issues/6)). Apple Silicon and Intel are built and started in CI, never on a clean Mac: the zip downloaded with a browser, quarantine and Gatekeeper, App Translocation, a kit on an exFAT stick, the agents' sign-in carried from one Mac to another.
- **Linux on real machines** ([#5](https://github.com/nabre/atollo/issues/5)). x64 and ARM64 on Ubuntu 24.04, Fedora 42 and Debian 13: AppImage with and without FUSE, the `tar.gz` with WebKitGTK 4.1, exFAT sticks or sticks mounted `noexec`, sign-in with GNOME, KeePassXC or KWallet.
- **Updates and the kit's builds outside Windows.** Updating the app from one version to the next on macOS and Linux is not tried yet. The kit's Apache (2.4.69, D-116) is in the catalog but tried only in CI; so is the kit's PHP with `pdo_sqlite` (8.4.26 and 8.5.11, D-133), which the app uses from the version after 0.2.0 (0.2.0 still takes PHP from static-php-cli).
- **Code signing.** On macOS only an ad hoc signature, without Apple notarization: `xattr -dr com.apple.quarantine` is needed. On Windows 0.2.0 ships unsigned: the SignPath signature is still to be activated, and until then SmartScreen may warn on first launch.

Have a Mac, a Linux PC or a Windows ARM64 PC? Try Atollo and tell us how it went: [open an issue on GitHub](https://github.com/nabre/atollo/issues).

## Known limits

- On Windows 10 and 11 (`x64` and `ARM64`), macOS (Apple Silicon and Intel) and Linux (`x64` and `ARM64`). On Windows ARM64 the editor, PHP, Apache, MySQL, MongoDB, Android SDK, Julia, TinyTeX and R run under x64 emulation; on Linux ARM64 Android is missing. The macOS and Linux releases are tried in CI but not yet on clean machines ([#5](https://github.com/nabre/atollo/issues/5)), and the agents' sign-in still has to be tried on a Mac ([#6](https://github.com/nabre/atollo/issues/6)); components and sources per system in [docs/app/CATALOG.md](docs/app/CATALOG.md), the component × system matrix in [docs/app/TOOLS-MATRIX.html](docs/app/TOOLS-MATRIX.html). Apache on macOS and Linux is the kit's own build (D-116), with PHP through `php-fpm`.
- On Linux MySQL needs two system libraries, `libaio` and `libnuma`: on Ubuntu 24.04 and Debian 13 `sudo apt install libaio1t64 libnuma1` (on older releases `libaio1`), on Fedora and RHEL `sudo dnf install libaio numactl-libs`. If they are missing, Atollo says so with the command to run.
- On macOS and Linux an Atollo folder on exFAT or FAT (a drive formatted for Windows) works, but those file systems have no links: where a program ships some, Atollo copies the files instead (for instance Git's 142 internal links), taking more space. On a drive used only with a Mac or Linux, APFS or ext4 is better.
- Outside the Atollo folder there are only Windows' own traces, transient temporary files in the system temp folder (automatically removed on exit), GitHub CLI's sign-in in the Credential Manager if you sign in from a console with `gh auth login` (the one made in Settings stays in `data\gh`) and Antigravity's while `agy` is open (for the session only).
- The folders in `projects\` are trusted by Claude Code and Codex: open only projects you trust. The editor instead asks the first time it opens each project and opens the ones you don't trust in "Restricted Mode".
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

For a backup, copy `projects\` and `data\` (without `data\apps\` and `data\cache\`, which are downloaded again).

## Uninstallation

Atollo does not use an installer, registers no system services or scheduled tasks, and does not modify the Windows registry or `PATH`:

1. Open the **Remove drive** page (or close the application) to stop all processes started by Atollo and restore any active Antigravity session into `data\`.
2. Delete the folder containing `Atollo.exe`, `data\`, and `projects\` (after backing up any projects you wish to keep).
3. Any transient folders in `%TEMP%` are automatically removed on exit; if you ran `gh auth login` manually in a console (instead of the **Settings** page), you can remove the corresponding entry from the Windows **Credential Manager**.

## Command line

```
Atollo.exe                           opens the window
Atollo.exe env                       shows the environment Atollo's programs get
Atollo.exe components                lists languages and tools, with the installed ones
Atollo.exe check                     checks that the installed programs answer
Atollo.exe prepare [options]         installs, updates and removes the chosen components
    --only a,b                       only these (and what they need)
    --skip a,b                       not these
    --update-all                     also update the installed ones
    --force                          download again
Atollo.exe sync [--project <folder>] [--verbose]
                                     aligns instructions, skills and MCP of the three agents
Atollo.exe keyring restore|save      Antigravity's sign-in (used by agy.cmd)
Atollo.exe serve <folder> <port>     file server for a static site
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

Atollo (`Atollo.exe`) fully respects user privacy:

- **No telemetry or data collection:** the application does not collect personal data, usage statistics, telemetry, or crash reports, and never contacts any server operated by the project author. It also disables built-in telemetry in managed components (such as PowerShell 7 and the VSCodium server).
- **Local storage:** all settings, projects, local databases, logs, and credentials are stored exclusively locally inside Atollo's `data\` (and `projects\`) folder on the user's device.
- **Network connections:** the application only makes network requests to official upstream sources to check for updates and download user-requested components (`api.github.com`, official language/tool vendor mirrors, Open VSX registry for editor extensions) and, when the user configures and uses the GitHub integration, AI agents (**Claude Code**, **Codex**, **Antigravity**), or a Gemini API key, directly to the respective provider APIs under their own privacy policies.

## License

MIT.
