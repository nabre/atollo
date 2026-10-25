# Atollo Privacy Policy and Terms of Use

English · [Italiano](PRIVACY.it.md) · [Français](PRIVACY.fr.md) · [Deutsch](PRIVACY.de.md)

Effective date: 25 October 2026

This page explains what the Atollo desktop app (`Atollo.exe`, the macOS `.app` and the Linux AppImage) and its website do with your data. Atollo is open source: everything written here can be checked in the code of this repository.

## 1. Summary

- Atollo has **no telemetry, no analytics and no crash reporting**. It has no account and no server of its own.
- Everything Atollo stores stays **in the Atollo folder** on your device (`data/` and `projects/`), with the few exceptions listed in section 4.
- Atollo connects to the network only to **check for its own updates**, to **download the tools you use from their official sources**, and to talk to **GitHub and the AI agents' providers when you use those features**. The table in section 3 lists every connection, with how to turn it off.
- The tools and AI agents you run inside Atollo (Claude Code, Codex, Antigravity, Pi, Git, npm…) are third-party programs with their own privacy policies.
- **Local AI models** run on your computer: with a local model, your prompts and files never leave it.
- Projects run in microVMs of **Docker Sandboxes** (`sbx`, by Docker), installed in the kit. They need a **Docker account**, signed in with `sbx login` on each computer; Atollo runs `sbx` with Docker's telemetry and diagnostics upload off. What Docker Sandboxes keeps outside the Atollo folder is listed in section 4.

## 2. What Atollo does not collect

Atollo does not collect, send or sell:

- personal data, names, e-mail addresses or device identifiers;
- usage statistics, clicks, page views or feature usage;
- crash reports or logs (the diagnostics zip of section 4 is made on your device and leaves it only if you attach it yourself);
- the content of your projects, files, databases or conversations with AI agents.

Nobody running the Atollo project receives any data from the app. The project does not operate servers: updates and releases are hosted on GitHub.

Atollo also turns off the telemetry and self-update checks of several programs it manages, inside its own environment: the integrated editor (VSCodium) runs with telemetry off and extension auto-update off; the environment sets `GOTELEMETRY=off`, `DOTNET_CLI_TELEMETRY_OPTOUT=1`, `POWERSHELL_TELEMETRY_OPTOUT=1`, `POWERSHELL_UPDATECHECK=Off`, `PIP_DISABLE_PIP_VERSION_CHECK=1`, `GH_NO_UPDATE_NOTIFIER=1` (for the app's GitHub CLI calls), `DISABLE_UPDATES=1` (Claude Code), `AGY_CLI_DISABLE_AUTO_UPDATE=true` (Antigravity), `PI_SKIP_VERSION_CHECK=1` and `PI_TELEMETRY=0` (Pi). Every run of the kit's Docker Sandboxes gets `SBX_NO_TELEMETRY=1` (Docker's usage reports off; that it stops every report is still to be checked for real, and `sbx` keeps its local queue of events in the kit) and `LLMMAN_RUNTIME=path` (its local-models helper downloads nothing), and Atollo keeps three settings on its daemon: `diagnostics.autoUpload=no` (no diagnostics bundle is uploaded to Docker), `ssh.agentForwardingEnabled=false` (the VMs never use your SSH keys) and `browser.enabled=false` (a VM never opens your browser). Other programs keep their own defaults.

## 3. Network connections the app makes

Every request the app makes itself carries a `User-Agent` of the form `Atollo/<version> (+https://github.com/nabre/atollo)` and, like any Internet connection, reveals your IP address to the server contacted. Requests follow the proxy set in Settings, otherwise `HTTPS_PROXY`/`HTTP_PROXY`, otherwise the system's proxy setting.

| When | To whom | What is sent | How to turn it off |
|---|---|---|---|
| About 20 seconds after start, and when you press "Check for updates" | GitHub (`api.github.com`, then the release download from `github.com`) | A request for the list of releases of `nabre/atollo`; if a newer version exists, its download. If a `GITHUB_TOKEN` or `GH_TOKEN` variable is set on the computer, it is sent to `api.github.com` only (to avoid GitHub's rate limit). | Settings › General › **Automatic update** off, or set the `ATOLLO_NO_UPDATE` variable on the computer. The button still works on demand. |
| About 10 seconds after start, at most once a day per tool | The official source of each tool that offers several versions (for example `nodejs.org`, `go.dev`, `api.adoptium.net`, `api.github.com`) | A request for the list of available versions. To `api.github.com` only: the token above, or your GitHub account's token if you signed in (section 4). | Set the `ATOLLO_NO_UPDATE` variable. |
| Right after the previous row, at most once a day; and when you press "Check now" in the "Updates" pane of the Tools page | The official source of each installed tool, the same one an update of it asks (for example `nodejs.org`, `go.dev`, `api.adoptium.net`, `api.github.com`) | A request for the tool's latest version, one per installed tool; GitHub's answers go through the cache of section 4. To `api.github.com` only: the same token as the row above. | Settings › General › **Automatic update** off, or set the `ATOLLO_NO_UPDATE` variable. "Check now" still works on demand. |
| At first start, when you confirm the first-start page (after choosing language and network); later, automatically, when a required component is missing or an app update needs newer ones | Official sources of PowerShell 7, GitHub CLI, the editor and Docker Sandboxes (GitHub releases; Docker Sandboxes from `docker/sbx-releases`, on Windows x64, Apple silicon Macs and Linux) | Download requests; checksums are verified locally. | No switch: these components are required. Nothing else is installed without your action. |
| When you install or update a tool (Tools page) or an action needs one (for example Node.js for `npm`) | The tool's official source, for example `nodejs.org`, `registry.npmjs.org`, `go.dev`, `static.rust-lang.org`, `api.adoptium.net`, `repo.maven.apache.org`, `services.gradle.org`, `builds.dotnet.microsoft.com`, `windows.php.net`, `getcomposer.org`, `cdn.mysql.com`, `downloads.mongodb.org`, `sqlite.org`, `files.phpmyadmin.net`, `conda.anaconda.org`, `api.anaconda.org`, `dl.google.com`, `storage.googleapis.com`, `raw.githubusercontent.com`, `www.apachelounge.com`, `dl.static-php.dev`, `curl.se`, `aka.ms`, `api.nuget.org`, `julialang-s3.julialang.org`, `ziglang.org`, Google's Antigravity update server, GitHub releases (among them `ggml-org/llama.cpp` for the local models' server and `earendil-works/pi` for Pi) | Download requests; checksums are verified locally. | Do not install the tool. |
| Only when you download a local model (AI page, Local models pane, or `models pull`), or look up a Hugging Face repository | Hugging Face: `huggingface.co` (the API, for a repository's last commit and its GGUF files), then its CDN for the model files | Requests for the repository's listing and for the model's files at a fixed commit; a paused or interrupted download resumes with a `Range` request. Your Hugging Face token, only if you saved one for gated models, and only to `huggingface.co` (it is not forwarded to the CDN). Each file's SHA-256 is verified locally. | Do not download models. |
| In the background after the editor starts, and when a project is opened | The editor's extension gallery (Open VSX); CodeLLDB from its GitHub release | Requests for the extensions the kit and your projects use. | Do not open the editor, or remove the extensions from it. |
| Only when you sign in to GitHub (Settings › GitHub account) and use your repositories (Projects › “From your GitHub repositories”) | GitHub (`github.com`, `api.github.com`), through GitHub CLI | The sign-in (a one-time code you type at `github.com/login/device`), then your account's token with requests to list, clone, rename or delete your repositories. | Do not sign in, or sign out in Settings › GitHub account. |
| Once, at sign-in or at a start while signed in, only if the commit name and email are empty | GitHub (`api.github.com/user`), through GitHub CLI | Your account's token, to read the account's name, login and number: they become the name of your commits and GitHub's private `noreply` address. | Fill in Settings › GitHub account › “Who signs the commits”, or do not sign in. |
| When an agent uses the kit's **github** MCP server (Agents › The kit's MCP servers), only while you are signed in to GitHub | GitHub's MCP server (`api.githubcopilot.com`), through `Atollo mcp github`, the bridge the agent starts | The agent's requests to GitHub's tools (issues, pull requests, repositories) with your GitHub account's token, which the bridge adds itself: the agent never sees it. | Do not add the github server, or remove it from the kit's MCP servers. |
| Only while the AI page is shown (on opening, then every 2 minutes), for each agent you are signed in to | Claude Code: `api.anthropic.com`; Codex: `chatgpt.com`; Antigravity: `cloudcode-pa.googleapis.com` | The agent's own login token, sent only to that agent's provider, to read your plan's usage limits. The token is only read, never refreshed, and never written to the log. | Do not open the AI page, or do not sign in to the agent. |

Things that are **not** connections made by Atollo itself:

- programs you run in the editor, in the consoles or through actions (`git`, `npm`, `pip`, `cargo`, the AI agents…) connect wherever they are designed to;
- **Docker Sandboxes** (`sbx`, section 5), which Atollo starts for the microVMs, talks to Docker itself: the sign-in of your Docker account (`sbx login`, whose link opens in your browser when you click it in Atollo's terminal), and the images the VMs need (the `shell` sandbox, `php:<version>-apache` or `httpd:2.4` for a site in a VM). The programs inside a VM reach the network only through `sbx`'s proxy, under Docker's `balanced` network policy, which Atollo sets on the kit's `sbx`; towards your computer, a project's VM reaches only the port of its own database. With "GitHub in the VM" on for a project (off by default), the proxy adds your GitHub token to that VM's requests to GitHub: `sbx` gets the token by running `Atollo secret github`, which answers only the kit's `sbx` with a nonce Atollo registered, and the token never enters the VM;
- links you click open in your own browser, and so does “Report a problem” (section 4).

Local servers: the editor and the projects' servers listen on this computer only (`127.0.0.1`/`localhost`), unless you turn on **Local network** in Settings for the projects' servers. With it on, the QR code of a site's address for the phone is made by Atollo on this computer: no service on the internet sees the address, and finding this PC's IP on the local network sends no packet. The local models' server (llama.cpp) always listens on `127.0.0.1` only, on a port of its own (9931 and following), and answers only requests that carry its random key; `llm status` does not send that key to whatever listens on the port. The models run on this computer: with a local model nothing goes to the network. A site that runs in a project's microVM has its port published on this computer's `127.0.0.1`, or on its local network when **Local network** is on; turning it off takes that publication back at the site's next start (if it cannot, the site does not start). Stopping the site takes its port back.

## 4. Data stored on your device

Atollo is portable: it keeps its data next to the executable, in the `data/` folder, and your projects in `projects/`. Nothing is uploaded. Deleting the Atollo folder deletes all of it.

| What | Where (inside the Atollo folder) |
|---|---|
| Settings: language, theme, proxy, certificates, automatic update, Rust compiler | `data/app/settings.json` |
| Git's settings, with the name and e-mail that sign your commits | `data/home/.gitconfig` |
| The **Gemini API key**, if you enter one (AI page, Antigravity › Sign-in) (readable by your user only, see below) | `data/app/secrets/gemini-api-key` |
| The system's root certificates plus your extra PEM file, only when you set one | `data/app/ca-bundle.pem` |
| App logs (secrets masked), rotated past 1 MB | `data/app/logs/` |
| Diagnostics zips, only when you make one (secrets masked, see below) | `data/app/diagnostics/` |
| Installed tools and their versions | `data/apps/`, `data/app/versions/` |
| The latest version of each installed tool, with the time of the last check | `data/app/updates.json` |
| Cached answers of the GitHub API | `data/cache/github/` |
| The interface's own storage (window layout) and web view cache | `data/webview2/` (Windows and Linux) |
| GitHub account (GitHub CLI token, stored as a file so it travels with the kit) | `data/gh/` |
| Git credentials for other hosts (Git Credential Manager, encrypted with Windows DPAPI for your Windows user) | `data/home/.gcm/` (Windows) |
| AI agents' logins and sessions, written by the agents themselves | `data/home/.claude/`, `data/home/.codex/`, `data/home/.gemini/` |
| Antigravity's Google sign-in, moved out of the system's credential store after each run | `data/antigravity/keyring.json` |
| Claude Code's long-lived token, if you use "Connect with a token" (macOS) | `data/agents/claude/oauth-token` |
| Database passwords (random, generated by Atollo) | `data/app/db/secrets.json`, `data/app/db/mysql.cnf`, `data/projects/<project>/private/db.json` |
| Local models (GGUF files) and what Atollo downloaded of them | `data/models/gguf/<owner>--<repo>/` (with `atollo.json`) |
| Local models' settings (port, automatic start, the model Pi uses) and the server's presets | `data/app/llm.json`, `data/app/llm/presets.ini` |
| The **local models' server key** (random, generated by Atollo, readable by your user only) | `data/app/secrets/llama-key` |
| The **Hugging Face token**, if you enter one for gated models (readable by your user only) | `data/app/secrets/hf-token` |
| Pi's settings, sign-ins (`auth.json`), sessions and the kit's `models.json` with the local server's address and key (readable by your user only) | `data/home/.pi/agent/` |
| Editor server data and its per-session access token | `data/vscodium-server/` |
| Docker Sandboxes: its settings and logs, the VMs' images and disks (each VM's home, `/tmp` and its own Docker) | the kit's home: `data/home/AppData/Local/DockerSandboxes/` on Windows, under `data/home/` on macOS and Linux |
| Each sandbox's kit: its specification, whose it is (project, or terminal and folder), its mounts and opened ports; for a project with a database, the `db.env` with the project's database login given to its VM (readable by your user only) | `data/app/sandboxes/<sandbox>/` |
| The notes of the "GitHub in the VM" commands Atollo registered in Docker Sandboxes, with their nonces (readable by your user only) | `data/app/sandboxes/secrets.json` |
| The editor's sessions in the VM: whether the VM was on when the editor opened (`.level`) and, for each project opened in the editor, the path of `sbx`, its variables and the arguments of its terminal, its tasks and its Claude Code and Codex sandboxes (no password: the database login stays in `db.env`); cleared at every start of the app | `data/app/sandboxes/sessions/` |
| The script the editor's terminals, tasks and agent profiles start (it reads the sessions above), written again at every start of the app | `data/kit/vm.ps1` |
| Your projects | `projects/` |
| A kit copy waiting to be imported, only after “Import from…” | `.atollo-import/` |
| What an imported copy replaced, kept until you delete it | `.atollo-before-import-<date-time>/` |

The database passwords (`db.env` of the sandboxes included), the Gemini API key, the Claude token file, the local models' key and the Hugging Face token are created readable by your user only, where the disk supports permissions (NTFS, APFS, ext4…). The other files, `settings.json` included, are ordinary files. On USB sticks formatted FAT32 or exFAT there are no permissions at all: **anyone with the stick can read every file above**. Keep the stick safe, or use a disk with permissions.

Settings › The kit › **“Copy of the kit”** writes `projects/` and `data/` (without programs, caches, local models and the VMs' images and disks) to a folder or a zip **you choose**, possibly outside the Atollo folder. The copy holds the kit's secrets **in clear** (`data/app/db/secrets.json`, `data/app/secrets/`, the agents' and GitHub sign-ins): keep it safe. Atollo never sends it anywhere.

The Claude token and the Gemini API key are given (as `CLAUDE_CODE_OAUTH_TOKEN` and `GEMINI_API_KEY`) only to the agents Atollo launches, not to project actions, scripts, terminals or the editor. The local models' key goes to Pi when Atollo launches it (`LLAMA_API_KEY`) and into Pi's own `models.json`, so that `pi` works from the kit's terminals too. The Hugging Face token is sent only to `huggingface.co`.

Exceptions, outside the Atollo folder:

- **Temporary files**: files in transit go to `<system temp>/Atollo/`; depending on Settings › General › **Temporary files of programs**, programs may also use the system's temporary folder (by default when the kit is on a USB stick, a network drive or FAT32/exFAT).
- **Antigravity sign-in**: while Antigravity runs, its login is placed in the system's credential store (on Windows for the logon session only), then moved back into the kit and removed. Credentials that were already on the computer are put back as they were; a sealed note of them (`keyring-before.json`, readable only on that computer) stays in the kit until the next save.
- **Claude Code on macOS** keeps its own login in the Keychain, unless you use "Connect with a token".
- **macOS web view**: on macOS the system web view keeps its data where macOS decides (in your user Library).
- **rustup** writes a Windows registry entry at each install; Atollo removes it.
- **Docker Sandboxes**, because `sbx` or the system leave no other place (everything else of it stays in `data/`):
  - the hypervisor (Windows Hypervisor Platform, `/dev/kvm`), turned on once by you as an administrator, never by Atollo;
  - your Docker sign-in (`sbx login`), kept by `sbx` in the system's credential store, once per computer;
  - for each project with "GitHub in the VM" on, an entry in the system's credential store (on Windows the Credential Manager, `com.docker.sandboxes:…`) holding the command `sbx` runs to get the token, not the token itself; the value `sbx` obtains may stay in its memory according to its own refresh policy;
  - a `sboxd-<id>` junction and a `gvisor` folder in the system's temporary folder, for the daemon's sockets, whose paths must be short;
  - on Ubuntu, an AppArmor profile for the kit's microVM shim (`/etc/apparmor.d/atollo-sbx-shim`), which you add with the `sudo` command Atollo shows.

### Reporting a problem

Settings › The kit › “Reporting a problem” has two buttons; nothing in them runs on its own.

- **“Export diagnostics”** (or `Atollo diagnostics` on the command line) makes a zip **on your device**, in `data/app/diagnostics/`: Atollo's version, the kit's and the computer's system (Windows build, RAM), the kit's disk (file system, kind, free space), the kit's folder, the kit's environment, the check of the installed programs, the components, the ports and the last 256 KB of each log of `data/app/logs/`. Before the zip is written, every secret Atollo keeps (the files of `data/app/secrets/`, Claude Code's token, the database passwords, the agents' logins in the kit, the GitHub token) is replaced by `***` wherever it appears, on top of the masking of variables whose name looks secret; your profile folder becomes `~`, and the names of your user and of the computer (`USERNAME`, `COMPUTERNAME`, `HOSTNAME`…) become `***`. **The app never sends the zip**: open it, read it, and attach it yourself if you want to. A project's own output in the logs may still hold things Atollo does not know are private.
- **“Report a problem”** opens `github.com/nabre/atollo/issues/new` in your browser, with the bug form and two fields filled in through the address: Atollo's version and the system (the kit's system, the Windows build, the file system and kind of the kit's disk). Nothing else from your device is in the address, and nothing reaches GitHub until you sign in and create the issue there. From then on the issue, with whatever you attach, is public and [GitHub's Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement) applies.

## 5. Third-party tools and AI agents

Atollo installs and starts programs made by others. When you use them, they talk directly to their own services and **their own privacy policies and terms apply**. Atollo does not proxy, read or store that traffic, except the sessions and logins those programs themselves write to the kit's folder (section 4).

- **Claude Code** (Anthropic): [anthropic.com/legal/privacy](https://www.anthropic.com/legal/privacy)
- **Codex** (OpenAI): [openai.com/policies/privacy-policy](https://openai.com/policies/privacy-policy/)
- **Antigravity** and the Gemini API (Google): [policies.google.com/privacy](https://policies.google.com/privacy)
- **Pi** (Earendil): [pi.dev](https://pi.dev); with a cloud provider (signed in with `/login`), that provider's policy applies
- **Hugging Face**, where local models are downloaded from: [huggingface.co/privacy](https://huggingface.co/privacy)
- **llama.cpp** (ggml-org), the local models' server: it runs on your computer and sends nothing; [github.com/ggml-org/llama.cpp](https://github.com/ggml-org/llama.cpp)
- **Docker Sandboxes** (Docker), the projects' microVMs, with your Docker account: [docker.com/legal/docker-privacy-policy](https://www.docker.com/legal/docker-privacy-policy/)
- **GitHub** and GitHub CLI: [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
- **VSCodium** and Open VSX: [open-vsx.org](https://open-vsx.org/)
- Package registries (npm, PyPI, crates.io, Maven Central, NuGet…) and the download sites of each tool, under their own policies.

The AI agents send your prompts and the files they read to their providers: that is how they work. Choose what you open with them. Pi is the exception when it uses a local model: then prompts and files stay on your computer; with a cloud provider chosen in Pi, they go to that provider.

## 6. Website

The website [nabre.github.io/atollo](https://nabre.github.io/atollo/) is a static page hosted on GitHub Pages, with the Guide's pages under [`/guide/`](https://nabre.github.io/atollo/guide/) (the same Guide that is inside the app, which reads it from the app itself, with no connection).

- No analytics, no tracking, no cookies, no advertising. Fonts and images are served from the site itself.
- The site remembers your theme and language choice in your browser's `localStorage` (`atollo-theme`, `atollo-lang`); it never leaves your browser.
- To point the download button at the right file, your browser asks `api.github.com` for the list of Atollo releases.
- GitHub, as the host, may log technical data such as IP addresses: see the [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).
- The site is verified in Google Search Console with a verification file; this adds no tracking for visitors.

## 7. Children

Atollo is a developer tool and is not aimed at children. It collects no data from anyone, children included.

## 8. Terms of use

Atollo is free software released under the [MIT License](../LICENSE). It is provided **"as is", without warranty of any kind**, express or implied; the authors are not liable for any claim, damage or other liability arising from its use. You are responsible for how you use it and for respecting the licenses and terms of the third-party tools and services it installs or connects to (section 5).

## 9. Changes

When this policy changes, the new version is published in this file with a new effective date; earlier versions remain in the repository's history.

## 10. Contact

Questions or concerns: open an issue at [github.com/nabre/atollo/issues](https://github.com/nabre/atollo/issues).
