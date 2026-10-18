# Atollo — Universal Portable Dev & AI Workspace

[Italiano](../README.md) · English · [Français](README.fr.md) · [Deutsch](README.de.md)

**Atollo** is a self-contained *Universal Portable Dev & AI Workspace* (`Atollo.exe`) that unifies an **Editor** (integrated VSCodium), **Toolchain/Runtime Manager** (20+ on-demand languages and tools), **Local Servers/Databases** (MySQL, MongoDB, SQLite, Caddy/Apache), and an **AI Agent Orchestrator** (Claude Code, Codex, and Antigravity with synchronized instructions, skills, and MCP servers) into a single portable folder. Zero install, zero administrator rights, zero system services outside the workspace folder.

On first start the app downloads the system components by itself: PowerShell 7, GitHub CLI, the editor and what they need (Git for GitHub CLI). Everything else is chosen on the "Tools" page, or arrives when needed: Node.js with an `npm` action, uv with a `uvx` MCP server, and so on.

> In Italian: guide [app/GUIDA.md](app/GUIDA.md), decisions [app/DECISIONS.md](app/DECISIONS.md), changes [app/CHANGELOG.md](app/CHANGELOG.md); open work: [issues](https://github.com/nabre/dev-kit2/issues).

## How to use it

1. Copy `Atollo.exe` into an empty folder: a USB stick, an external drive or a folder in your profile (spaces, brackets or `&` in the name are fine).
2. Double-click it: the app creates `projects\` and `data\`. On first start choose agents, languages and tools: the app downloads and verifies them (checksum or signature).
3. Create or clone a project on the **Projects** page. The editor, the terminal and the agents always open in a project's folder, never anywhere else.

## Pages

| Page | What it has |
| --- | --- |
| **Projects** | the projects with Run, Editor, Terminal and agent; **New project**: empty, from your GitHub repositories (with clone, rename and delete on GitHub) or from a Git URL; notices about the workspace. Each project has its own page: **Summary** (the main action with its output, languages, databases, agent, Git), **Resources** (components, the project's database with user and password, editor extensions), **Advanced** (actions, ports, static site, Git, agent instructions) |
| **Agents** | the installed Claude Code, Codex and Antigravity: version, sign-in, plan limits and usage of today and the last 7 days; instructions, skills and MCP servers shared in `data\agents` |
| **Databases** | Atollo's MySQL (with phpMyAdmin) and MongoDB servers: start and stop, administrator password, port, data in `data\db`, on this PC only; SQLite without a server |
| **Tools** | languages and tools with their logo, dependencies and disk space; install, update and remove; the editor extensions matrix |
| **Settings** | language, terminal, network (proxy, certificates, local network), Gemini key, Python and PHP versions, Rust compiler (GNU or MSVC), app updates; Atollo's **GitHub account**, one for everything (app, Git, editor, consoles, agents, GitHub CLI); where the workspace folder is |
| **Remove drive** | closes everything Atollo opened, brings Antigravity's sign-in back into `data\` and lets you eject the drive |

At the bottom of every page, the **Activity** panel: what is running or installing (projects, databases, components), with output and errors in one place.

Atollo's console is the classic Windows console with integrated PowerShell 7 and only Atollo's isolated environment: programs installed on the host PC do not interfere.

## Known limits

- On Windows 10 and 11 (`x64` and `ARM64`), macOS (Apple Silicon and Intel) and Linux (`x64` and `ARM64`). On Windows ARM64 the editor, PHP, Apache, MySQL and TinyTeX run under x64 emulation and R is not available; on Linux ARM64 Android is missing. macOS and Linux have no releases yet ([#5](https://github.com/nabre/dev-kit2/issues/5)) and the agents' sign-in still has to be tried on a Mac ([#6](https://github.com/nabre/dev-kit2/issues/6)); components and sources per system in [app/CATALOG.md](app/CATALOG.md) (in Italian), the component × system matrix in [app/MATRICE-STRUMENTI.html](app/MATRICE-STRUMENTI.html). Apache on macOS and Linux comes with the first release of the kit's own build (D-116); until then the web server there is Caddy.
- On Linux MySQL needs two system libraries, `libaio` and `libnuma`: on Ubuntu 24.04 and Debian 13 `sudo apt install libaio1t64 libnuma1` (on older releases `libaio1`), on Fedora and RHEL `sudo dnf install libaio numactl-libs`. If they are missing, Atollo says so with the command to run.
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

`npm run lint` runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint and Prettier; `npm test` runs `cargo test` and the interface tests; `npm -w ui run build` checks the types and builds the interface. Details (in Italian) in [app/TESTING.md](app/TESTING.md). How the code is organized and where a change goes (in Italian): [app/ARCHITECTURE.md](app/ARCHITECTURE.md).

Development with live reload: `npm run dev`.

## If Windows blocks the exe

Until code signing is active on the downloaded release, if SmartScreen shows "Windows protected your PC", choose **More info** › **Run anyway**. If the exe was downloaded from the Internet: right-click › **Properties** › **Unblock**. On company PCs with AppLocker you need permission from the IT department.

## Code signing policy

Free code signing on Windows provided by [SignPath.io](https://signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

- **Project roles (Committer, Reviewer, and Approver):** [nabre](https://github.com/nabre)
- **Binary provenance:** official release binaries are built and signed exclusively by the automated GitHub Actions pipeline ([`.github/workflows/app.yml`](../.github/workflows/app.yml)) directly from the public source code in this repository, with no manual intervention on binaries.

## Privacy policy

Atollo (`Atollo.exe`) fully respects user privacy:

- **No telemetry or data collection:** the application does not collect personal data, usage statistics, telemetry, or crash reports, and never contacts any server operated by the project author. It also disables built-in telemetry in managed components (such as PowerShell 7 and the VSCodium server).
- **Local storage:** all settings, projects, local databases, logs, and credentials are stored exclusively locally inside Atollo's `data\` (and `projects\`) folder on the user's device.
- **Network connections:** the application only makes network requests to official upstream sources to check for updates and download user-requested components (`api.github.com`, official language/tool vendor mirrors, Open VSX registry for editor extensions) and, when the user configures and uses the GitHub integration, AI agents (**Claude Code**, **Codex**, **Antigravity**), or a Gemini API key, directly to the respective provider APIs under their own privacy policies.

## License

MIT.
