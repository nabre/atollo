# Atollo

[Italiano](../README.md) · English · [Français](README.fr.md) · [Deutsch](README.de.md)

A portable development environment for Windows in a single file: `Atollo.exe`. Integrated editor (VSCodium), the Claude Code, Codex and Antigravity agents, languages and tools downloaded on demand. No installation, no administrator rights, no file outside the kit folder.

On first start the app downloads the system components by itself: PowerShell 7, GitHub CLI, the editor and what they need (Git for GitHub CLI). Everything else is chosen on the "Languages and tools" page, or arrives when needed: Node.js with an `npm` action, uv with a `uvx` MCP server, and so on.

> In development. Status and plan (in Italian): [app/PLAN.md](app/PLAN.md), changes: [app/CHANGELOG.md](app/CHANGELOG.md).
> Inspired by [nabre/atollo](https://github.com/nabre/atollo), of which it is not a new version: it is a separate product.

## How to use it

1. Copy `Atollo.exe` into an empty folder: a USB stick, an external drive or a folder in your profile (spaces, brackets or `&` in the name are fine).
2. Double-click it: the app creates `projects\` and `data\`. On first start choose agents, languages and tools: the app downloads and verifies them (checksum or signature).
3. Create or clone a project on the **Projects** page. The editor, the terminal and the agents always open in a project's folder, never anywhere else.

## Pages

| Page | What it has |
| --- | --- |
| **Home** | recent projects with Editor, Terminal and agent; notices (kit moved, another copy of the kit open, installations running); where the kit is |
| **Projects** | new project, clone from GitHub or a Git URL; for each project its type, its languages (with their logo) and the recognised actions (`npm run dev`, `php -S`, `cargo run`, static site…), with port, output and opening in the browser |
| **Agents** | the installed Claude Code, Codex and Antigravity: version, sign-in, plan limits and usage of today and of the last 7 days; instructions, skills and MCP servers shared in `data\agents` |
| **Languages and tools** | the components with their logo, dependencies and disk space; install, update and remove |
| **Settings** | language, terminal, network (proxy, certificates, local network), Gemini key, Python and PHP versions, Rust compiler (GNU or MSVC), app updates |
| **Remove drive** | closes everything the kit opened, brings Antigravity's sign-in back into the kit and lets you eject the stick |

The kit console is the kit's portable Windows Terminal (or the classic Windows console, if chosen in Settings or if Windows Terminal is not installed) with the kit's PowerShell 7 and only the kit's environment: programs installed on the PC are not visible.

## What is left of the Atollo

The console menu of [nabre/atollo](https://github.com/nabre/atollo) is gone: each of its entries has a page in the app ("Proxy and settings" is the Settings page) and the logic of the scripts is in Rust inside the exe. The components are the same (plus PowerShell 7 and the integrated editor, minus VSCodium desktop), and so are the alignment of instructions, skills and MCP across the three agents and the relocation when the kit changes folder or drive letter. Atollo kits are not imported.

## Known limits

- Windows 10 and 11 only, x64 and ARM64 (on ARM64 the editor, PHP, Apache and TinyTeX run under x64 emulation; R is not available).
- Outside the kit there are only Windows' own traces, GitHub CLI's sign-in in the Credential Manager and Antigravity's while `agy` is open (for the session only).
- The folders in `projects\` are trusted by Claude Code and Codex: open only projects you trust. The editor instead asks the first time it opens each project and opens the ones you don't trust in "Restricted Mode".
- The agents' usage limits come from undocumented interfaces of the providers and may stop working after one of their updates.
- "Remove drive" does not see PC programs that keep only a file of the kit open.
- The exe is not signed (see below).

## Layout

```
Atollo\
  Atollo.exe     all the logic
  projects\       your projects
  data\           downloaded programs, settings, sign-ins, cache, temporary files
                  (the kit's settings of each project: data\app\projects\<project>\dev-settings.json)
```

For a backup, copy `projects\` and `data\` (without `data\apps\` and `data\cache\`, which are downloaded again).

## Command line

```
Atollo.exe                          opens the window
Atollo.exe env                      shows the environment the kit's programs get
Atollo.exe components               lists languages and tools, with the installed ones
Atollo.exe check                    checks that the installed programs answer
Atollo.exe prepare [options]        installs, updates and removes the chosen components
    --only a,b                       only these (and what they need)
    --skip a,b                       not these
    --update-all                     also update the installed ones
    --force                          download again
Atollo.exe sync [--project <folder>] [--verbose]
                                     aligns instructions, skills and MCP of the three agents
Atollo.exe keyring restore|save     Antigravity's sign-in (used by the kit's agy.cmd)
Atollo.exe serve <folder> <port>    file server for a static site
Atollo.exe version                  shows the version
Atollo.exe help                     shows the help
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

`npm run lint` runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, ESLint and Prettier; `npm test` runs `cargo test` and the interface tests; `npm -w ui run build` checks the types and builds the interface. Details (in Italian) in [app/TESTING.md](app/TESTING.md).

Development with live reload: `npm run dev`.

## If Windows blocks the exe

The exe is not signed yet. If SmartScreen shows "Windows protected your PC", choose **More info** › **Run anyway**. If the exe was downloaded from the Internet: right-click › **Properties** › **Unblock**. On company PCs with AppLocker you need permission from the IT department.

## License

MIT.
