# Atollo Guide

English · [Italiano](GUIDE.it.md)

Atollo is a complete development environment that lives in a single folder: on a USB stick, an external disk or the PC's own disk. It does not install and does not ask for administrator rights: copy the folder to another PC and you find everything again, projects included.

## Contents

1. [First steps](#1-first-steps)
   - [1.1 Welcome to Atollo](#11-welcome-to-atollo)
   - [1.2 The first start](#12-the-first-start)
   - [1.3 Getting to know the window](#13-getting-to-know-the-window)
   - [1.4 The first project, step by step](#14-the-first-project-step-by-step)
2. [Projects](#2-projects)
   - [2.1 Creating a project](#21-creating-a-project)
   - [2.2 The Projects page and the “Quick panel”](#22-the-projects-page-and-the-quick-panel)
   - [2.3 Starting a project: actions and ports](#23-starting-a-project-actions-and-ports)
   - [2.4 Components and versions of a project](#24-components-and-versions-of-a-project)
   - [2.5 Editor, terminal and extensions](#25-editor-terminal-and-extensions)
   - [2.6 A project's Git](#26-a-projects-git)
   - [2.7 Deleting a project and the “Data without a project”](#27-deleting-a-project-and-the-data-without-a-project)
3. [Databases, websites and terminals](#3-databases-websites-and-terminals)
   - [3.1 The database servers](#31-the-database-servers)
   - [3.2 A project's database](#32-a-projects-database)
   - [3.3 Websites and .localhost addresses](#33-websites-and-localhost-addresses)
   - [3.4 The kit's terminals](#34-the-kits-terminals)
4. [AI agents](#4-ai-agents)
   - [4.1 The agents: Claude Code, Codex, Antigravity](#41-the-agents-claude-code-codex-antigravity)
   - [4.2 Shared instructions, skills and MCP servers](#42-shared-instructions-skills-and-mcp-servers)
   - [4.3 Agent limits and usage](#43-agent-limits-and-usage)
5. [Tools and languages](#5-tools-and-languages)
   - [5.1 The Tools page](#51-the-tools-page)
   - [5.2 Several versions of the same tool](#52-several-versions-of-the-same-tool)
   - [5.3 Overviews: matrix, relations, extensions, requirements](#53-overviews-matrix-relations-extensions-requirements)
6. [Account and settings](#6-account-and-settings)
   - [6.1 The kit's GitHub account](#61-the-kits-github-account)
   - [6.2 The Settings, one by one](#62-the-settings-one-by-one)
   - [6.3 Company network: proxy, certificates, local network](#63-company-network-proxy-certificates-local-network)
   - [6.4 The framework instructions](#64-the-framework-instructions)
7. [Taking the kit with you](#7-taking-the-kit-with-you)
   - [7.1 Safely removing the drive](#71-safely-removing-the-drive)
   - [7.2 On another PC, another letter, another folder](#72-on-another-pc-another-letter-another-folder)
   - [7.3 Updates and backups](#73-updates-and-backups)
8. [How it works inside](#8-how-it-works-inside)
   - [8.1 Portable and isolated: what it means](#81-portable-and-isolated-what-it-means)
   - [8.2 The kit's folders on disk](#82-the-kits-folders-on-disk)
   - [8.3 Background processes](#83-background-processes)
   - [8.4 Where the tools come from](#84-where-the-tools-come-from)
   - [8.5 The command line](#85-the-command-line)
9. [Help](#9-help)
   - [9.1 When something goes wrong](#91-when-something-goes-wrong)
   - [9.2 Frequently asked questions](#92-frequently-asked-questions)
   - [9.3 Glossary](#93-glossary)

## 1. First steps

### 1.1 Welcome to Atollo

*What Atollo is, what you find inside and how this guide is organized. The place to start if it is your first time.*

> **In short**
>
> - Atollo is a complete development environment that lives in a single folder: on a USB stick, an external disk or the PC's own disk.
> - It does not install and does not ask for administrator rights: copy the folder to another PC and you find everything again, projects included.
> - Inside you find languages in several versions, databases, web servers, an editor, consoles, Git and GitHub, and three AI agents.
> - This guide goes from the simplest things to the most technical ones: you can read it in order or jump to what you need.

#### What it is, in simple words

Picture a work backpack that is already packed: inside is everything you need to write programs and websites, tidy and working. You carry it with you, open it on any computer and work, without installing anything and without leaving a mess on the PC that hosts you. When you are done, you close it and take it away.

Atollo is that backpack. It is a single program that sits in a folder together with two subfolders: projects, where your projects live, and data, where the kit keeps its programs, settings and database data. It works on Windows 10 and 11, macOS and Linux, with x64 and ARM64 processors (on ARM64 Windows 11 is needed).

> [!NOTE]
> **Who it is for**
>
> For people who code on several computers (home, office, lab), people who cannot install programs on their work PC, people who teach or study and want the same environment for everyone, and people who want to try a language or a framework without cluttering their own system.

#### What you find inside

| What | What it is for | Where it is in the app |
| --- | --- | --- |
| Projects | Your working folders, each with its own actions, versions, database and website. | Projects |
| Languages and tools | Python, Node.js, PHP, Java, .NET, Go, Rust and many others, even several versions side by side. | Tools |
| Databases | A MySQL server and a MongoDB one, for this PC only, with passwords already set up. | Databases |
| Web servers | Caddy and Apache, to see your websites at addresses like mio-sito.localhost. | Web Servers |
| Editor | VSCodium, the free version of Visual Studio Code, in an Atollo window. | Projects › Editor |
| Console | PowerShell 7 with all the kit's tools ready to use. | Terminals |
| Git and GitHub | Git and GitHub CLI with a single sign-in to your account for the whole kit. | Settings › GitHub account |
| AI agents | Claude Code, Codex and Antigravity, with shared instructions, skills and MCP servers. | Agents |

#### Three ideas to keep in mind

**Everything in one folder** — The kit writes only inside its own folder (with a few declared exceptions). To make a copy of everything, just copy the folder; to remove it, just delete it.

**The kit takes care of the tools** — When a project or an agent needs a program that is missing, the kit downloads it from the official source, checks it and puts it in its place. You choose, it installs.

**Only on this PC** — Databases and websites answer only the computer you are working on (address 127.0.0.1), with passwords generated by the kit. Nobody on the network sees them, unless you ask for it.

#### How this guide is organized

The guide has nine chapters. The first three are for everyone: how to get started, how to work on projects, how to use databases and websites. Then agents, tools and settings. The “How it works inside” chapter is for those who want to understand the technical details; the last one gathers common problems, frequently asked questions and a glossary.

In every topic you find: an “In short” box at the start, the steps to follow, tips and warnings, and a collapsed “Under the hood” part with the technical details (files, folders, commands). The button at the top right opens the app page the topic is about. The search box on the left searches titles, descriptions and keywords.

1. **[First steps](#1-first-steps)** — Welcome to Atollo · The first start · Getting to know the window · The first project, step by step
2. **[Projects](#2-projects)** — Creating a project · The Projects page and the “Quick panel” · Starting a project: actions and ports · Components and versions of a project · Editor, terminal and extensions · A project's Git · Deleting a project and the “Data without a project”
3. **[Databases, websites and terminals](#3-databases-websites-and-terminals)** — The database servers · A project's database · Websites and .localhost addresses · The kit's terminals
4. **[AI agents](#4-ai-agents)** — The agents: Claude Code, Codex, Antigravity · Shared instructions, skills and MCP servers · Agent limits and usage
5. **[Tools and languages](#5-tools-and-languages)** — The Tools page · Several versions of the same tool · Overviews: matrix, relations, extensions, requirements
6. **[Account and settings](#6-account-and-settings)** — The kit's GitHub account · The Settings, one by one · Company network: proxy, certificates, local network · The framework instructions
7. **[Taking the kit with you](#7-taking-the-kit-with-you)** — Safely removing the drive · On another PC, another letter, another folder · Updates and backups
8. **[How it works inside](#8-how-it-works-inside)** — Portable and isolated: what it means · The kit's folders on disk · Background processes · Where the tools come from · The command line
9. **[Help](#9-help)** — When something goes wrong · Frequently asked questions · Glossary

Related topics: [1.2 The first start](#12-the-first-start), [1.3 Getting to know the window](#13-getting-to-know-the-window), [1.4 The first project, step by step](#14-the-first-project-step-by-step)

### 1.2 The first start

*Where to put the kit, the “Welcome” page, which components to choose and what happens while they download.*

> **In short**
>
> - Put the Atollo program in an empty folder, where you want to keep the kit, and open it.
> - The first time, the “Welcome” page appears: choose the language and, if needed, proxy and certificates, then agents and languages, and press “Install”. Nothing is downloaded before “Install”.
> - The components download in the background from the official sources: you can already use the app while they arrive.
> - Nothing is final: everything can be added or removed later from the Tools page.

#### 1. Where to put the kit

The kit lives where its executable is: the folder that contains it becomes the “kit folder”, and that is where projects and data will be created. So choose an empty folder with a simple name, for example `D:\Atollo` on a USB stick or an external disk, or `C:\Atollo` on the PC's disk.

- On Windows the executable is called Atollo.exe, or Atollo-\<version\>-win32-x64.exe as you download it from the releases page. On macOS it is Atollo.app, on Linux an AppImage file.
- Space: the choice suggested at the first start takes about 1.8 GB. Each extra language or version adds space, and projects with their dependencies grow over time: on a USB stick it is best to have at least 16 GB free.
- Speed: an external SSD is much faster than a cheap USB stick. The kit works on both, but installing packages with npm or Composer on a slow USB stick takes patience.

> [!WARNING]
> **Two things to avoid in the path**
>
> - On Windows, a folder in the path whose name starts with a digit, with the letter x or with the letter u (for example `D:\2024\Atollo` or `D:\utenti\Atollo`) stops the editor from noticing changed files. If that happens, the app warns you: move the kit to a different path.
> - On macOS and Linux a disk mounted without execute permission (noexec) stops the app: the kit must be able to start its programs.

<details>
<summary>Which file system to choose</summary>

On Windows the best format is NTFS: it is the only one that lets the kit protect the folders holding passwords (databases, sign-ins) so that only your user can read them, and use links instead of copies. On exFAT and FAT32 the kit works, but without that protection; FAT32 also does not accept files larger than 4 GB. If you use the same USB stick on Windows, macOS and Linux, exFAT is the format all of them can read and write: in that case keep in mind that links become copies.

The Settings › “The kit” page shows the disk the kit is on: file system and type (fixed, removable, network).

</details>

#### 2. The “Welcome” page

At the first start, instead of the usual pages, the “Welcome” tab appears, with the title “Welcome to Atollo”. It explains that agents and languages will be downloaded from the official sources, checked and placed in the kit's data folder, with no installation and no administrator rights.

At the top there are two groups of settings, saved by themselves as you change them (as on the Settings page, where you will find them again): “Before you start”, with “Language”, “Theme” and “Automatic update”, and “Network”, with proxy, additional certificates and local network. Fill in “Network” only if your network goes through a proxy or inspects HTTPS traffic: since nothing is downloaded before “Install”, the very first downloads will use them too.

Below is the list of components, split into six kinds (each with its description), with a search box and an index on the left. Each row has a checkbox, the logo, the name, a description, the size and some tags that say what it needs or what it is for.

| Type | What it contains | What to choose |
| --- | --- | --- |
| System and editor | The editor, PowerShell 7 (the kit's console), GitHub CLI and Git. Below, the PC requirements. | They are always installed, together with the others, when you press “Install”: the checkboxes are locked. |
| AI agents | Claude Code, Codex and Antigravity. | Optional. Keep the ones you have an account for; they require a subscription or an API key from the provider. |
| Languages | All languages and runtimes: Python, Node.js, PHP, Java, Go, .NET, Rust, Ruby, the C/C++ compiler and the others. Python, Node.js, PHP, Go and Java can have several versions; the others stay at the latest. | The ones you use. Others can be added when needed. |
| Development tools | uv, pnpm and yarn, CMake and Ninja, Android SDK, Typst, TinyTeX: a single version, always the latest. | Usually nothing: the kit installs them by itself when a project or an agent asks for them. |
| Web server | Apache and Caddy, for the projects' sites and the shared Web Servers. | The one your projects use; Apache is already in the suggested choice. |
| Databases | MySQL, MongoDB, SQLite; in the browser phpMyAdmin and mongo-express. | Only if you need them. MySQL is also installed the first time the server starts. |

The suggested choice includes the three agents, Python, PHP with Apache and the “System and editor” components, Git included. About 1.8 GB in all. When you tick a component, the kit also ticks the ones it needs (for example Kotlin brings Java with it, phpMyAdmin brings PHP and MySQL).

1. In “Before you start” choose the language and, if you like, the theme; if you are behind a company proxy, also fill in “Network”.
2. Untick what does not interest you and add what you use.
3. Press “Install” at the bottom. There is no confirmation: the choice is saved and the download starts right away.
4. The app leaves the “Welcome” page and shows the normal pages. The download continues in the background: you see it in the status bar at the bottom and in the “Activity” panel.

> [!TIP]
> **Tip**
>
> At the first start you do not choose versions: for each language the latest one is installed. Other versions can be added later, from the Tools page.

> [!WARNING]
> **Trust in projects**
>
> When you open an agent in a project from Atollo, the project's folder becomes trusted by Claude Code and Codex, which can then run commands in it without asking every time. Open an agent only in projects you trust.

#### 3. What happens while it downloads

1. **Download** — From the manufacturer's official source, over HTTPS.
2. **Check** — Compares the file's fingerprint with the published one; if they do not match, it discards it.
3. **Extract** — Into a temporary working folder.
4. **Set up** — Puts the program in `data\apps` and checks that it responds.

Every component goes through these four steps. If a download fails (no network, company proxy, manufacturer's site unreachable) the kit tries three times; if it still does not work, the row shows the error and, when possible, a box to download the file with the browser and hand it to the kit.

The details on sources and checks are in [8.4 Where the tools come from](#84-where-the-tools-come-from); if you are behind a company proxy, read [6.3 Company network: proxy, certificates, local network](#63-company-network-proxy-certificates-local-network) first.

<details>
<summary>Under the hood</summary>

- The choice of components is saved in `data\app\components.txt`, one line per component (id=1 chosen, id=0 not).
- At the first start nothing is downloaded, not even the required components, until you press “Install”: this way the proxy and certificates chosen on the page already apply to the first downloads. From the following starts the kit checks that the required components are there and reinstalls them by itself if they are missing, without asking.
- If the first start is interrupted, the “Welcome” page does not come back on reopening if the choice has already been saved: you pick up from Tools, where the rows not downloaded show the state “not present” or “error”.

</details>

Related topics: [1.3 Getting to know the window](#13-getting-to-know-the-window), [1.4 The first project, step by step](#14-the-first-project-step-by-step), [5.1 The Tools page](#51-the-tools-page), [6.3 Company network: proxy, certificates, local network](#63-company-network-proxy-certificates-local-network)

### 1.3 Getting to know the window

*Activity bar, sidebar, tabs, status bar and the “Activity” panel: where things are and what they are for.*

> **In short**
>
> - The window looks like Visual Studio Code's: if you know it, you will find your way right away.
> - On the left the activity bar changes page; next to it, the sidebar shows lists and trees.
> - In the center the tabs; at the bottom the status bar and the “Activity” panel with what is in progress.
> - <kbd>Ctrl+P</kbd> opens a project by name from any page.

#### The parts of the window

```
+-------------------------------------------------------------+
| 1 · Title bar                                               |
+------------+-----------------+------------------------------+
| 2 ·        | 3 · Sidebar     | 4 · Tab area                 |
| Activity   |                 |                              |
| bar        |                 +------------------------------+
|            |                 | 5 · “Activity” panel         |
+------------+-----------------+------------------------------+
| 6 · Status bar                                              |
+-------------------------------------------------------------+
```

| Part | What it contains |
| --- | --- |
| 1 · Title bar | The name Atollo and the open page; on the right minimize, maximize and close. Drag it to move the window, double-click to maximize it. |
| 2 · Activity bar | The page icons. At the top the working ones: Projects, Agents, Databases, Web Servers, Terminals, Tools. At the bottom: Guide, Settings, Remove drive. A number on the Projects icon tells how many things are running: project actions, database versions and web server pools. |
| 3 · Sidebar | The page title with its icons (including the book “Open the guide for this page”), often a search box, and panes that open and close with a click on their title. |
| 4 · Tab area | What you choose in the sidebar opens in a tab. The first tab of some pages is fixed: the “Quick panel” in Projects, “All tools” in Tools. |
| 5 · “Activity” panel | It opens from the status bar. It lists the installation in progress, the running database versions, the running web server pools and the project actions, each with its output; “Stop all” stops them all after a confirmation. |
| 6 · Status bar | On the left the Atollo item and the counter of what is running and of the errors; on the right the open terminals (macOS and Linux), the editor status, the system (for example win32-x64), the Atollo version and, last, the icon that shows or hides the panel. |

#### The pages

**Projects** — The heart of the work: the list of projects, the “Quick panel”, creating new projects and the page of each project. [How it works](#22-the-projects-page-and-the-quick-panel)

**Agents** — The installed AI agents: signing in to your account, plan limits and usage, shared folder. [How it works](#41-the-agents-claude-code-codex-antigravity)

**Databases** — The kit's MySQL and MongoDB servers: starting, passwords, ports, versions, phpMyAdmin and mongo-express. [How it works](#31-the-database-servers)

**Web Servers** — The Caddy and Apache servers shared among projects and the .localhost websites they serve. [How it works](#33-websites-and-localhost-addresses)

**Terminals** — Consoles with the kit's tools, in any folder on the PC. [How it works](#34-the-kits-terminals)

**Tools** — The kit's languages, agents, databases and programs: install, update, remove, several versions. [How it works](#51-the-tools-page)

**Guide** — This guide. Every page also has a book-shaped icon that opens the right topic. [How it works](#11-welcome-to-atollo)

**Settings** — Language, theme, network, GitHub account and who signs the commits, Gemini key, updates, privacy and framework instructions, saved by themselves. [How it works](#62-the-settings-one-by-one)

**Remove drive** — Closes everything the kit has opened, before you unplug the USB stick. [How it works](#71-safely-removing-the-drive)

#### The status bar and the “Activity” panel

| Item | What it says | What a click does |
| --- | --- | --- |
| Atollo (on the left) | Hovering over it shows the kit folder. | — |
| ▶ number · ⨯ number | How many things are running (project actions, every running database version, even several of the same database, and the running web server pools) and how many actions stopped with an error. It turns red if there is at least one error. | Opens or closes the “Activity” panel. |
| Spinner with text | A component installation in progress and the step it is on. | Opens the panel. |
| Open terminals (macOS and Linux) | How many sessions the app's terminal panel has. | Shows or hides the terminals. |
| Editor running | The editor server is on (hovering over it shows its address). | — |
| win32-x64, darwin-arm64… | The system and the processor the kit runs on. | — |
| v0.x.y | The Atollo version. | — |
| Panel icon (the last on the right) | Hovering over it says “Show the panel” or “Hide the panel”. | Opens or closes the “Activity” panel. |

In the “Activity” panel you select each item to see its details: for a project action the status, the address and the live output, with “Open in the browser”, “Stop” or, if it has finished, “Remove”; for a database the name, the address, any errors, the log file and “Stop”; for a web server pool the address, the attached VirtualHosts and “Stop”. “Stop all”, at the top, asks “Stop the running project actions, databases and web servers?” and then stops actions, databases and pools. If there is nothing, “nothing running” appears. You resize the panel by dragging its edge; a double-click on the edge brings it back to its initial height.

#### Useful shortcuts and habits

| Keys | What it does |
| --- | --- |
| <kbd>Ctrl+P</kbd> | Opens “Open a project” (⌘P on macOS): type part of the name, the platform or the language. Enter opens the project's page, Shift+Enter starts its default action, Esc closes. |
| <kbd>Esc</kbd> | Closes confirmation dialogs and image zoom, and cancels the questions in the page (for example the one of “Stop all” or of removing a terminal). |

- Hover the mouse over icons, dots and tags: almost everything has a tooltip that explains what it is or what it does.
- The language and the theme are changed in Settings › General › “Language” and “Theme”: by default the light or dark theme follows the operating system's, and it applies to the editor and the terminals too.
- The book-shaped icon in the sidebar of every page opens the guide topic that explains it.

Related topics: [2.2 The Projects page and the “Quick panel”](#22-the-projects-page-and-the-quick-panel), [8.3 Background processes](#83-background-processes), [1.4 The first project, step by step](#14-the-first-project-step-by-step)

### 1.4 The first project, step by step

*A complete walkthrough in ten minutes: create a project, start it, see it in the browser and open it in the editor.*

> **In short**
>
> - A complete example: a small TypeScript website with Vite, from creation to the browser.
> - All you need is the Node.js component, which the kit installs by itself if it is missing, and an Internet connection.
> - By the end you will know how to create a project, start it, open it in the browser and edit it in the editor.

#### 1. Create the project

1. Open the “Projects” page from the activity bar.
2. In the “Quick panel”, under “New project”, click “From a framework or CMS”. The “New project” tab opens.
3. In “1. Base” choose the “Web with JavaScript and TypeScript” group and the “Vite” card (“Website or web application in TypeScript”).
4. In “2. Additions” nothing is needed for this test.
5. In “3. Name” type primo-sito. The name becomes the `projects\primo-sito` folder.
6. Look at the “Summary”: you will see that the kit will use Node.js, run npm create vite in the projects folder and then npm install in the project, and save the “Start” and “Build” actions.
7. Press “Create the project”. In the “Progress” part the commands and their output scroll by.
8. When “Project primo-sito created.” appears, press “Open the project”.

> [!NOTE]
> **Good to know**
>
> If Node.js was not installed, the kit downloads it before running the commands: you see it in the first lines of the progress (“Installing the kit tools needed…”).

#### 2. Start it and open it in the browser

1. The project page opens on the “Run” section. At the bottom, the “Actions” list shows what the project can do: the actions recognized from the package.json file (npm run dev, npm run build, npm install) and those from the recipe (Start, Build).
2. Press ▶ next to “npm run dev”. It is a web action: the kit assigns it a port, starts it and waits for it to respond. The status goes from “starting” to “listening”.
3. Press “Open in the browser”: Vite's sample site opens at http://localhost:\<port\>/, where the port is a number between 7100 and 7999 chosen by the kit and kept the same at every start.
4. Do you want the big “Start” button at the top to always start this action? Select it and press the “Default” star.

#### 3. Edit it in the editor

1. Press “Editor” at the top of the project page. The kit starts the editor (the first time it may take a few seconds) and opens an Atollo window with the project folder.
2. Open src/main.ts, change some text and save with Ctrl+S.
3. Go back to the browser: Vite updates the page by itself.
4. To stop the website go back to Atollo and press “Stop” (or ■ next to the action). Closing Atollo stops everything anyway.

#### What next?

- Putting the project on GitHub or cloning one: [6.1 The kit's GitHub account](#61-the-kits-github-account).
- Giving the project a database: [3.2 A project's database](#32-a-projects-database).
- Getting help from an AI agent in the project: [4.1 The agents: Claude Code, Codex, Antigravity](#41-the-agents-claude-code-codex-antigravity).
- Trying other bases (Laravel, Django, WordPress, Astro…): [2.1 Creating a project](#21-creating-a-project).

Related topics: [2.1 Creating a project](#21-creating-a-project), [2.3 Starting a project: actions and ports](#23-starting-a-project-actions-and-ports), [2.5 Editor, terminal and extensions](#25-editor-terminal-and-extensions)

## 2. Projects

### 2.1 Creating a project

*The three ways: an empty folder, a Git repository, or a framework or CMS base with its additions. Plus your GitHub repositories.*

> **In short**
>
> - A project is simply a folder inside projects: the kit recognizes it by itself.
> - There are three ways to create one: “Empty project”, “From a Git address” and “From a framework or CMS”. Plus the list of your GitHub repositories.
> - With a framework, the kit installs the tools needed and runs the framework's official commands.
> - You can also copy a folder into projects by hand: it appears in the list after “Refresh”.

![The “From a framework or CMS” procedure](../../ui/public/guide/en/project-new.webp)

*Base, add-ons, name, .env file and summary, one below the other. On the left, the tree of the three ways.*

#### Where you create a project

All roads lead to the “New project” tab of the Projects page, which opens in one of these ways:

- from the “Quick panel”, with the three boxes under “New project”;
- from the icons at the top of the sidebar: “New project” (folder with a plus) and “New project from a framework or CMS” (wand);
- from the “New project” pane of the sidebar, a tree with the three ways; under “From a framework or CMS” are the groups and the bases: a click on a base opens the wizard with that base already chosen;
- from <kbd>Ctrl+P</kbd>, which has the three “New project” entries at the bottom of the list.

A created or cloned project also appears at once on the “Web Servers” page.

#### Way 1 · “Empty project”

A new, empty folder in projects, to fill as you like (even by copying files into it).

1. Type the folder name in the “Empty” field.
2. Press “Create”. The new project's page opens.

The name cannot contain `\` / : \* ? " \< \> |, cannot start or end with a dot and cannot be a reserved Windows name (like CON or NUL). If a folder with that name already exists, the kit tells you.

#### Way 2 · “From a Git address”

Copies (clones) an existing Git repository into a folder in projects.

1. In the first field type owner/repository for a GitHub repository (for example laravel/laravel), or the full https:// address of any Git server.
2. In the second field, optional, the folder name; if you leave it empty it is the last part of the address.
3. Press “Clone”.

After the copy the kit looks at the project's files, installs the components needed and picks the versions the project declares (for example the Node version written in .nvmrc or the PHP one in composer.json), downloading them if missing.

> [!NOTE]
> **Good to know**
>
> Only https:// addresses are accepted, not ssh. A private GitHub repository needs you to be signed in with the kit's GitHub account; public repositories do not. If the copy fails, the folder is removed and the last line of Git's error appears.

#### Way 3 · “From a framework or CMS”

A wizard that creates a project starting from a base (a framework like Laravel, Django or Vite, a CMS like WordPress, a site generator, a desktop or mobile app…) and from optional additions (Tailwind, React, a database, testing tools…). The kit installs the tools needed and runs the framework's official commands, the same ones you would find in its documentation.

| Step | What you do |
| --- | --- |
| 1. Base | Choose a group with the tags at the top (Web with PHP, Web with JavaScript and TypeScript, Web with Python, Web with other languages, CMS, Static sites and documentation, Desktop app, Mobile app, Data and documents, Command line and libraries) and then the base's card. |
| 2. Additions | Tick the additions you want. They are divided by type (Interface, Styles, Tests, Databases…): you choose only one of each type, and choosing another replaces the first. |
| 3. Name | Type the project name. Alternatively use an existing project that is empty (“empty”) or has files but no Git (“with files”): the framework's files are added and the ones already there stay as they are. A Git repository cannot be used. |
| 4. .env file | The variables the kit will write in the project's .env file. {name} becomes the project name, {package} an identifier like com.example.nome, {password} a new password. You can change the values, remove lines or add some with “Add a variable”. |
| Summary | What will happen: the kit tools, the project database, the commands in the projects folder, those in the project folder and the actions the project will have. |

1. Check the “Summary” and press “Create the project”.
2. In the “Progress” part you see “Installing the kit tools needed…”, then “Command 1 of N” with the output of each command.
3. At the end “Project … created.” appears with “Open the project”. If a command fails, the wizard stops and shows the error.

<!-- -->

1. **Tools** — Installs what the base and the additions ask for (Node, PHP, Composer…).
2. **Creation** — Runs the creation commands in the projects folder.
3. **Databases** — If planned, creates the database and user and writes the connection in .env.
4. **Finishing** — Runs the steps in the project folder (installations, migrations).
5. **Actions** — Saves the recipe's actions in the project's panel.json file.

> [!TIP]
> **Tip**
>
> The recipes can be edited, or you can add your own: [6.4 The framework instructions](#64-the-framework-instructions).

<details>
<summary>Under the hood</summary>

- One project is created at a time. “Cancel”, during the progress, stops the running command.
- Each line of the recipe is a command run with the kit's PowerShell and the kit's environment, so with the versions of the kit's tools.
- If you use an existing project with files, the framework is generated in a temporary system folder and then moved into the project: the files already there stay, and the progress shows “Already in the project, left as it is”.
- If the plan includes MySQL or MongoDB, the kit starts the server, creates the project's database and user and passes the variables ATOLLO_DB_HOST, ATOLLO_DB_PORT, ATOLLO_DB_NAME, ATOLLO_DB_USER, ATOLLO_DB_PASSWORD and ATOLLO_DB_URL to the commands. The password the kit generates takes the place of an empty one that comes from the base (for example DB_PASSWORD= in its .env.example).
- At the end, if there is a .env file that Git would pick up, the kit adds /.env to the .gitignore file.
- The recipe's actions go into the project's panel.json file with an identifier that starts with fw- (for example fw-avvia).
- Some bases require particular components: Android apps want the Android SDK component; Laravel with MongoDB wants PHP's mongodb extension, which the kit does not install.

</details>

#### Your GitHub repositories

If you have connected the GitHub account, the “From your GitHub repositories” pane of the sidebar lists your repositories and those of the organizations you belong to, grouped by owner, with a “Search the repositories” box.

- “Clone” copies the repository into projects (into the folder with its name, or owner-name if the name is already taken). An already cloned repository has its name in green and the “Editor” button.
- “GitHub” opens the repository in the browser.
- A click on the row opens the details: private or public, language, stars, last push, where it is cloned, “Copy the Git address” and, for the repositories you administer, “Rename” and “Delete from GitHub”.

> [!NOTE]
> **Good to know**
>
> As with “From a Git address”, after the copy the kit installs the components needed and picks the versions the project declares. All about the account in [6.1 The kit's GitHub account](#61-the-kits-github-account).

Related topics: [1.4 The first project, step by step](#14-the-first-project-step-by-step), [2.2 The Projects page and the “Quick panel”](#22-the-projects-page-and-the-quick-panel), [6.4 The framework instructions](#64-the-framework-instructions), [6.1 The kit's GitHub account](#61-the-kits-github-account)

### 2.2 The Projects page and the “Quick panel”

*The list of projects, the “Quick panel” with favorites and recent ones, quick open with Ctrl+P and the seven sections of a project.*

> **In short**
>
> - In the sidebar you find the list of projects; a click opens a project's page in a tab.
> - The “Quick panel” gathers what is running, the favorites, the recent ones and what needs fixing.
> - A project's page has the main buttons at the top and seven sections below.

![The “Quick panel”](../../ui/public/guide/en/projects-home.webp)

*Recent projects, things to fix and the three ways to a new project. On the left, the list of projects.*

#### The sidebar

At the top, the icons: “New project”, “New project from a framework or CMS”, “Open a project (Ctrl+P)”, “Refresh”, “Open folder” (opens projects) and the guide's book. Below, four panes:

| Pane | What it contains |
| --- | --- |
| Projects | All the projects in alphabetical order, with the count at the top. Folders starting with a dot and links do not count. With more than six projects the “Search projects” box appears. |
| Data without a project | It appears only if the kit has data of projects that are no longer in projects. See [2.7 Deleting a project and the “Data without a project”](#27-deleting-a-project-and-the-data-without-a-project). |
| New project | The tree of the three ways to create a project, with the framework bases. |
| From your GitHub repositories | The repositories of your account, ready to clone. |

Each project row has an icon that says what kind of project it is: a globe for a website, a window for an app with a window, a terminal for a command-line program. A green dot means something is running; a yellow triangle that components are missing. Hovering over the row shows the buttons ▶ “Start” (or ■ “Stop”), “Editor”, “Terminal” and one for each installed agent, with its logo.

#### The “Quick panel”

It is the first tab of the page, always there. It only shows the parts that have something to show:

| Part | What it shows |
| --- | --- |
| Running | The actions started in this session, with the address and the “Open in the browser”, “Show the output”, “Stop” buttons. |
| Favorites | The projects you have marked with the star (“Add … to favorites”), as cards with their languages and buttons. |
| Recent | The six projects used or changed most recently, excluding those already among the favorites. |
| Needs attention | The actions stopped with an error (“Show the output”) and the projects missing components (“Install”, which opens the project's Components section). |
| New project | The three ways to create a project, plus the links to GitHub repositories, the projects folder and the guide. |

#### Opening a project by name

From any page press <kbd>Ctrl+P</kbd> (⌘P on macOS): “Open a project” appears. Type part of the name, the platform or a language; projects are sorted by recent use. <kbd>Enter</kbd> opens the page, <kbd>Shift+Enter</kbd> immediately starts the default action.

#### A project's page

At the top are the name, the type and the recognized platform (for example “Laravel · website”), the language logos, the Git branch and, if components are missing, a “Missing: …” warning. Then the buttons:

| Button | What it does |
| --- | --- |
| Start / Stop | Starts the project's default action, or stops it if it is running. |
| Editor | Opens the project in the editor, in an Atollo window. |
| Terminal | Opens a kit console in the project folder, with its versions. |
| Agent logos | One button for each installed agent: a click on its logo opens that agent in the project. From then on Claude Code and Codex trust the project's folder. |
| Open folder | Opens the project folder in File Explorer (or Finder). |
| Refresh | Rereads the project: useful after changing panel.json or the project's files by hand. |

Below, the seven sections:

| Section | What it is for | Topic |
| --- | --- | --- |
| Run | The project's actions, their status, the port and the output. | [2.3 Starting a project: actions and ports](#23-starting-a-project-actions-and-ports) |
| Website | Serving a project folder as a website, with static files, PHP, Caddy or Apache. | [3.3 Websites and .localhost addresses](#33-websites-and-localhost-addresses) |
| Components | The tools needed, how the project was recognized, the versions. | [2.4 Components and versions of a project](#24-components-and-versions-of-a-project) |
| Databases | The project's database and user, the connection in the .env file, the copies. | [3.2 A project's database](#32-a-projects-database) |
| Extensions | The editor extensions active in this project. | [2.5 Editor, terminal and extensions](#25-editor-terminal-and-extensions) |
| Editor | Editor settings and the project's MCP servers. | [2.5 Editor, terminal and extensions](#25-editor-terminal-and-extensions) |
| Git | Remote, branch and sync, read-only. | [2.6 A project's Git](#26-a-projects-git) |

> [!NOTE]
> **How the kit recognizes a project**
>
> The kit looks at the files in the folder: package.json means Node.js (and with vite or next inside, a website), composer.json PHP, artisan Laravel, pyproject.toml Python, Cargo.toml Rust, go.mod Go and so on. From there it works out the type, the platform, the languages, the components needed and the actions. The “Components” › “How it was recognized” section says which files it started from. The recognition can be corrected with the panel.json file.

Related topics: [2.3 Starting a project: actions and ports](#23-starting-a-project-actions-and-ports), [2.1 Creating a project](#21-creating-a-project), [1.3 Getting to know the window](#13-getting-to-know-the-window)

### 2.3 Starting a project: actions and ports

*What an action is, how to start and stop it, the output, the port, the default action and how to add your own in the panel.json file.*

> **In short**
>
> - An action is a project command with a name: “npm run dev”, “php artisan serve”, the website…
> - Web actions get a port from the kit, which waits for them to respond and gives you the address to open.
> - The “Default” star picks the action of the “Start” button.
> - Actions are recognized by themselves; you add your own with “New action” and they end up in the project's panel.json file.

![A project's page, “Run” section](../../ui/public/guide/en/project-run.webp)

*At the top the Start, Editor, Terminal and Agent buttons; below, the chosen action with state, command and port, and the list of actions.*

#### The “Run” section

At the top is the detail of the chosen action (at first the default one), below the “Actions” list. A click on a row selects it and shows its output; ▶ and ■ on the row start and stop it.

| Item | What it says |
| --- | --- |
| Status | stopped, starting, listening (web action ready), running, exited, stopped, error (with the exit code). |
| Command | The command run, or the server that serves the website (the kit's file server, Apache or Caddy). |
| Folder | Where the command is run. |
| Port | Only for web actions: “automatic” or a number you choose. |
| Address | While the action is listening, the address to open (a link). |
| Source | panel.json, if the action is written by hand in the project. |

Below the detail is the “Output of …” pane with the “Follow” checkbox, which keeps the last lines in view. The kit keeps the last 3000 lines of each action; its notes appear as lines starting with \[Atollo\]. The buttons at the top: the “Default” star, “Open in the browser” (while a web action is listening) and “Clear the output” (when the action has finished).

#### What happens when you press “Start”

1. **Rereads** — Recognizes the project again: panel.json might have changed.
2. **Installs** — If the action is missing a component, it installs it first.
3. **Prepares** — The kit's environment, the project's versions, port, the action's variables.
4. **Starts** — Runs the command in the action's folder.
5. **Waits** — For a web action, until the port responds: then it is “listening”.

- The browser does not open by itself: use “Open in the browser” (here, in the “Quick panel” or in the “Activity” panel). The address is usually http://localhost:\<port\>/.
- If the program ignores the assigned port and prints another one in the output, the kit follows that one.
- “Stop” closes the action's whole process tree. Closing Atollo stops all actions.
- Running actions also appear in the “Activity” panel and in the status bar counter; in the panel “Stop all”, after a confirmation, stops them together with the running databases and web servers.

#### The port

Each web action has a port of its own, chosen by the kit between 7100 and 7999 the first time and then kept the same at every start: that way the address does not change. If you need a specific number, type it in the “Port” field and press Enter (or leave the field); emptying it takes you back to the automatic port.

- The port must be between 1024 and 65535.
- Ports 7000 to 7099 belong to the editor: one of them is refused with “Ports 7000 to 7099 belong to the editor”.
- A port already used by another action is refused with “Port already assigned to …”, followed by project → action.
- A port that a web server pool or a database version already has is refused too, and the message names its owner: “Port … already belongs to server …: choose another one.” or “Port … already belongs to database …: choose another one.” (for example MySQL 8.4).

#### The default action

It is the one that starts with the page's “Start” button, with ▶ in the project row and with Shift+Enter in “Open a project”. To change it, choose the action in the list and press the “Default” star. When you serve the project as a website, the website becomes the default action by itself.

#### Adding, editing and removing actions

1. In the “Run” section press “New action”, next to the “Actions” heading.
2. Fill in “Name”, “Type” (“Website or web server”, “Windowed app” or “Command”) and “Command”; if needed, “Folder” (relative to the project), “Address” (websites only) and “Variables”, one NAME=value per line. In the command, {port} becomes the action's port.
3. Press “Save”. The action appears in the list; its id comes from the name.

- “Edit”, on the chosen action, opens the same form again. On a recognized action it saves your own version with the same id, which takes its place.
- “Remove” exists only for your own actions and asks for confirmation; if a recognized action has the same id, that one comes back.
- The website is edited in the “Website” section, not here.

#### The panel.json file

Your own actions, the star and the website live in the panel.json file in the project folder. You can also write it by hand and press “Refresh”. The file lives in the project, so it goes into Git and applies to anyone who uses the project with Atollo.

**Example panel.json** `projects\gestionale\panel.json`

```
{
  "default": "dev",
  "actions": [
    {
      "id": "dev",
      "label": "Avvia tutto",
      "type": "web",
      "command": "npm run dev -- --port {port}",
      "folder": "frontend",
      "env": { "API_URL": "http://localhost:8000" }
    },
    {
      "id": "test",
      "label": "Prove",
      "type": "command",
      "command": "php artisan test"
    }
  ]
}
```

| Key | Meaning |
| --- | --- |
| `id` | Identifier of the action: lowercase letters, digits and - _ : . (40 characters at most). |
| `label` | The name you see in the list. |
| `type` | web (a server with a port), desktop (an app with a window) or command (a command that does something and finishes). |
| `command` | The command. {port} is replaced with the action's port. |
| `folder` | The subfolder to run it in (optional). |
| `url` | The address to open, if it is not http://localhost:{port}/ (optional). |
| `env` | Extra environment variables just for this action (optional). |
| `default` | At the top of the file: the id of the default action. |
| `replace` | At the top of the file: true to remove all the recognized actions and keep only yours. |

> [!NOTE]
> **Good to know**
>
> An action with the same id as a recognized one replaces it. If the file is not valid JSON, the kit ignores it and says so in “Components” › “How it was recognized”; fix it before using the star, the website or “New action”, which write to the same file.

<details>
<summary>Under the hood</summary>

- The command is run with cmd.exe on Windows and /bin/sh on macOS and Linux, in the action's folder, with the kit's environment and the project's versions. The agents' tokens (CLAUDE_CODE_OAUTH_TOKEN, GEMINI_API_KEY) are not in it: only the agents Atollo launches get them.
- Web actions get the PORT (and ROCKET_PORT) variables with the port; all of them get BROWSER=none, PYTHONUNBUFFERED=1 and ATOLLO_LANG. If the project has a kit MySQL database and its .env does not name another port, they also get DB_PORT.
- The assigned ports and the ones you chose are in `data\projects\<project>\ports.json`, outside the project.
- Each action runs in its own process group (a Job Object on Windows): “Stop” also closes the child processes, for example the server started by npm.
- In old files the Italian keys (azioni, etichetta, comando, cartella, predefinita) and {porta} instead of {port} are still accepted.

</details>

Related topics: [3.3 Websites and .localhost addresses](#33-websites-and-localhost-addresses), [2.2 The Projects page and the “Quick panel”](#22-the-projects-page-and-the-quick-panel), [9.1 When something goes wrong](#91-when-something-goes-wrong)

### 2.4 Components and versions of a project

*The tools a project needs, the version of each language chosen just for it and how the kit recognizes it from the project's files.*

> **In short**
>
> - The “Components” section says which kit tools the project needs and installs them with one click.
> - In “Versions” you choose, for this project only, the version of each language: for example Node 20 here and Node 24 elsewhere.
> - A new project keeps the latest versions that were there when it was created, until you change them.
> - The choice applies to the project's actions, terminal, agents, editor and web server.

![The “Components” section](../../ui/public/guide/en/project-versions.webp)

*The components the project needs, how it was recognized and the versions of its languages.*

#### Components needed

The “Components” part lists the kit tools the project needs, each “installed” or “to install”. If something is missing, “Install what is missing” installs them and waits for them to finish. The “How it was recognized” part explains from which files the kit worked out the project type, with any warnings (for example “Dependencies not installed: run npm install first”).

> [!NOTE]
> **Good to know**
>
> “Install what is missing” installs the kit's components (Node.js, PHP, Python…), not the project's dependencies: for those run npm install, composer install or similar, with an action or in the project terminal.

#### Choosing the project's versions

The “Versions” part has two groups: “Languages active in the project”, open, and “Languages not present in the project”, closed, to set other languages to use in the terminal or in scripts.

1. Open the menu next to the language.
2. The first entry, “The latest (…)”, lets the kit decide: it will always use the newest version on disk.
3. Under “Downloaded” are the versions already in the kit; under “To download: 8.x” those the source offers and the kit does not have yet.
4. Choose a version: if it is not downloaded, the download starts right away (“Downloading …”). Until it is ready the project uses the newest one (“… is not downloaded: … is used for now”, with “Download” or “Retry”).
5. The button with the arrows, “Look for other versions of …”, asks the source again for the list of versions.

Below the menu the kit can show some suggestions: “Required by package.json: \>=20” if the project declares a version, or “From the project's containers (Compose or Dockerfile): php 8.3” with the “Use 8.3” button.

> [!TIP]
> **When the new version takes effect**
>
> For the actions, the project terminal and the agents from the next start; for the editor's terminals from those opened after the change; for the website served by Caddy or Apache from the next start of the website (the server picks the project's PHP).

#### Where the starting versions come from

- A project created by the app (empty or with a framework) pins the latest versions present at that moment: a future kit update does not change things under the feet of a project that works.
- A project cloned “From a Git address” uses the versions its code asks for, if the kit recognizes them: .nvmrc, .node-version, .python-version, pyproject.toml, composer.json, .php-version, go.mod, global.json, .ruby-version, pom.xml and others.
- The summary of who asks for what is in the “Relations” view of the Tools page: [5.3 Overviews: matrix, relations, extensions, requirements](#53-overviews-matrix-relations-extensions-requirements).

<details>
<summary>Under the hood</summary>

- The choice is saved in `data\projects\<project>\dev-settings.json`, under versions; the first menu entry removes it.
- The kit also writes versions.json and versions.txt in the same folder: the PowerShell and Git Bash profiles read them to put the right versions in the PATH when a terminal opens in that project.
- Besides the PATH, the kit sets JAVA_HOME, DOTNET_ROOT and CP_PHP to the chosen versions.
- The version of the project's database server is chosen in the “Databases” section, not here.

</details>

Related topics: [5.2 Several versions of the same tool](#52-several-versions-of-the-same-tool), [3.2 A project's database](#32-a-projects-database), [5.3 Overviews: matrix, relations, extensions, requirements](#53-overviews-matrix-relations-extensions-requirements)

### 2.5 Editor, terminal and extensions

*Opening the project in the editor or in a kit console, choosing the extensions, the editor settings and the project's MCP servers.*

> **In short**
>
> - “Editor” opens the project in VSCodium, inside an Atollo window, with all the kit's tools.
> - “Terminal” opens a kit console in the project folder.
> - Extensions are chosen per project; the installed ones apply to the whole editor.
> - In the “Editor” section you set indentation, formatting, hidden folders and the project's MCP servers.

#### The editor

The kit's editor is VSCodium, the free version of Visual Studio Code. The “Editor” button (on the project page, in the list row or in the “Quick panel”) starts the editor server the first time and opens an Atollo window on the project. “Editor running” appears in the status bar.

- The editor's terminals already have the kit's environment. On Windows the profiles are “Atollo” (PowerShell, default), “Atollo - Git Bash” and “Atollo - cmd”; on macOS and Linux “Atollo”, “Atollo - Bash” and, on macOS, “Atollo - zsh”.
- Commit, push, comparing changes: everything Git does in the editor works with the kit's GitHub account.
- The editor's “Accounts” menu and its “Publish to GitHub” and “Clone from GitHub” functions do not see the kit's account: use Git, the Projects page or GitHub CLI.

> [!NOTE]
> **Good to know**
>
> If, with the editor open, you change the language, the network (for example the proxy) or the programs' temporary files in Settings, the app suggests “Restart the editor”, so that the editor's terminals use the new values.

#### The project terminal

“Terminal” opens a kit console in the project folder: on Windows a classic console window with the kit's PowerShell 7; on macOS and Linux a tab of the terminal panel inside Atollo. The tools are the kit's, at the versions chosen for the project. The prompt shows Atollo and the folder.

For consoles in other folders or with tools you choose there is the Terminals page: [3.4 The kit's terminals](#34-the-kits-terminals).

#### The “Extensions” section

Atollo pairs each language with its editor extensions, taken from Open VSX (the free extension catalog). A project gets those of its languages: the section shows them in groups (one per language, then “Tools” and “Added by you”), with the status “installed”, “to install” or “off”.

1. Untick an extension to turn it off in this project, or tick it again.
2. To add one that is not there, type its publisher.name identifier (for example ms-python.python) in the “Add an extension” field and press “Add”.

> [!NOTE]
> **Good to know**
>
> Installed extensions apply to the whole editor; new ones are installed when the editor is open. At the top, the summary says how many are active and how many are to install.

#### The “Editor” section

Two panes.

| Setting | Values |
| --- | --- |
| Format on save | like the editor, yes, no |
| Indentation | like the editor, 2 spaces, 4 spaces, tab |
| Folders hidden in the editor | One per line, for example \*\*/vendor or \*\*/node_modules: they do not appear in the file list. |

“Save” writes them in the project's `.vscode\settings.json` file; the other settings in the file stay. If the file contains comments or is not plain JSON, Atollo does not rewrite it and asks you to edit it in the editor.

The second pane, “Project MCP servers”, adds MCP servers that apply only in this project, on top of the kit's. How they work and how to add them: [4.2 Shared instructions, skills and MCP servers](#42-shared-instructions-skills-and-mcp-servers).

Related topics: [3.4 The kit's terminals](#34-the-kits-terminals), [4.2 Shared instructions, skills and MCP servers](#42-shared-instructions-skills-and-mcp-servers), [5.3 Overviews: matrix, relations, extensions, requirements](#53-overviews-matrix-relations-extensions-requirements)

### 2.6 A project's Git

*What the “Git” section shows, where you commit and push, and why the kit reads the status without running Git.*

> **In short**
>
> - The “Git” section shows the repository status: remote, branch, link to the remote, sync.
> - It is read-only: commits, pushes and the list of changes are done in the editor, in a terminal or with an agent.
> - Git uses the kit's GitHub account without asking for passwords.

#### What it shows

| Item | What it says |
| --- | --- |
| Remote | The address of the remote repository (origin), or “none”. |
| Branch | The current branch; “no branch” if you are on a specific commit. |
| Remote branch | The linked remote branch, or “branch not linked to a remote”. |
| In sync | “yes, at the same commit”, “no: there are commits to push or to pull”, or “unknown”. It refers to the last update from the remote (fetch). |

“Open on the web” opens the repository in the browser (for https addresses and for GitHub via ssh). If the folder is not a repository, the section says so.

#### Where you work with Git

- In the editor: the “Source Control” panel for commits, push, pull, branches and comparisons.
- In the project or editor terminal: the git and gh commands, already authenticated.
- With an agent: Claude Code, Codex and Antigravity can commit, create branches and open pull requests.

> [!WARNING]
> **Before the first commit**
>
> Git needs a name and an email for commits. If the kit is signed in to GitHub it takes them from the account (with GitHub's noreply address); otherwise type them in Settings › “GitHub account” › “Who signs the commits”. They are saved in the kit's .gitconfig file and follow you on every PC.

<details>
<summary>Under the hood</summary>

Atollo reads the status directly from the files in the .git folder (HEAD, config, refs), without running Git in the project: that way it cannot start the repository's hooks or programs just because you opened the page.

</details>

Related topics: [6.1 The kit's GitHub account](#61-the-kits-github-account), [2.5 Editor, terminal and extensions](#25-editor-terminal-and-extensions), [4.1 The agents: Claude Code, Codex, Antigravity](#41-the-agents-claude-code-codex-antigravity)

### 2.7 Deleting a project and the “Data without a project”

*What gets deleted and what stays, and how to relink or clean up the data of a project renamed or deleted outside the app.*

> **In short**
>
> - The kit keeps some data of each project outside the project, in `data\projects`: settings, ports, database passwords, copies.
> - From the app you can only delete a project with an empty folder; for the others you delete the folder yourself.
> - When the folder disappears, its data becomes “Data without a project”: you relink it or delete it.
> - Data without a project is never deleted by itself.

#### Where a project's data is

| Inside the project (goes into Git) | Outside, in `data\projects\<project>` (stays in the kit) |
| --- | --- |
| Your code, panel.json, .env, `.vscode\settings.json`, .mcp.json | Chosen versions, extensions, favorites (dev-settings.json), ports (ports.json), Apache and Caddy configurations, database passwords and names (private), database copies (`private\backups`) |

#### Deleting a project

If the project folder is empty, the “Run” section shows “Delete the empty folder”. For a project with files, the kit does not delete anything for you: delete or move the folder in File Explorer, then press “Refresh”; its data appears in “Data without a project”.

The “Delete …?” window lists what will disappear (the project also disappears at once from the “Web Servers” page):

- the empty folder and the kit's settings for that project;
- the database and its user on MySQL or MongoDB (a stopped server is started for the purpose);
- the database copies, with “Open the copies folder” to save them elsewhere first.

If deleting a database fails, nothing else is deleted; “Delete without the databases” then also appears, which leaves databases and users on the servers.

> [!WARNING]
> **Watch out**
>
> The database copies are deleted together with the project. If you need them, copy them outside the kit first.

#### The “Data without a project”

If you rename or delete a project's folder outside the app, the kit ends up with data without a project. The “Data without a project (N)” pane lists it, with what it contains: settings, ports, databases, copies.

| Button | What it does |
| --- | --- |
| Link to… | Moves settings, ports, databases and copies to a project you choose in the “Project” menu. Database and user keep the old name, so the .env file keeps working. |
| Delete | Deletes that data, databases included, with the same confirmation window as when deleting a project. |

> [!NOTE]
> **When “Link to…” cannot be used**
>
> The target project may have at most its settings (versions, favorite, last agent): those of the linked data take their place. If it already has chosen ports, servers, a database or copies, the link is refused, so the data of two projects never mix.

> [!TIP]
> **Tip**
>
> To rename a project without losing anything: rename the folder in File Explorer, go back to Atollo, press “Refresh” and use “Link to…” on its old data, choosing the project with the new name.

Related topics: [3.2 A project's database](#32-a-projects-database), [8.2 The kit's folders on disk](#82-the-kits-folders-on-disk), [7.3 Updates and backups](#73-updates-and-backups)

## 3. Databases, websites and terminals

### 3.1 The database servers

*The kit's MySQL and MongoDB: starting them, passwords, ports, several versions side by side, phpMyAdmin, mongo-express and the data explorer.*

> **In short**
>
> - The kit has a MySQL server and a MongoDB server, shared by all projects and reachable only from this PC.
> - “Start” installs the server if it is missing, turns it on and generates the administrator password.
> - Each project then has its own database and its own user, which you create from the project's page.
> - You browse and edit MySQL with phpMyAdmin and MongoDB with mongo-express, both in the browser; MongoDB can also be read in the “Content” tab.

![The Databases page, MySQL server](../../ui/public/guide/en/databases.webp)

*State and address, connection, version, port, “Start with Atollo” and the other installed versions.*

#### The Databases page

In the sidebar, the “Servers” box lists MySQL and MongoDB with their port, status and the “Start” or “Stop” button. Clicking a server opens its page. At the top of the sidebar: “Refresh”, “Open the data folder” (`data\db`) and the guide's book.

| Status | Meaning |
| --- | --- |
| Stopped (grey) | The server is off. |
| Starting… (yellow) | The server is starting; the kit waits for it to respond (up to 90 seconds). |
| Running (green) | The server responds on 127.0.0.1 at its port. |
| Error (red) | It did not start: the page shows the last error and the log file. |
| Not installed: installed at the first start | The component is not there yet; “Start” downloads it first. |

#### A server's page

At the top: the status, the address 127.0.0.1:\<port\>, the version and the “Start” or “Stop” buttons; for MySQL also “Open phpMyAdmin”, for MongoDB “Open mongo-express”.

| Part | What it contains |
| --- | --- |
| Connection | “String”: the address to connect as administrator (without the password). “Password of root” (MySQL) or “Password of admin” (MongoDB): hidden, with “Show” and “Copy”. Before the first start: “Generated at the first start.” |
| Settings | “Version” (if you have more than one; you change it while the server is stopped), “Port” (between 1024 and 65535, applies from the next start) and “Start with Atollo” (the server starts by itself when the app opens). |
| Other versions | Appears if you have installed several versions of the same server: each runs on its own port with its own data (see below). |
| Files | “Data”: the data folder of that version. “Log”: the file with the server's output. |

> [!NOTE]
> **Only from this PC, always with a password**
>
> The servers respond only at the address 127.0.0.1, that is, to this computer: nobody on the network can connect, not even with the “Local network” option in Settings. The administrator passwords are generated by the kit (32 random characters) at the first start. In the kit console the mysql command already logs in as root.

#### Browsing the data

| Server | How you see the data |
| --- | --- |
| MySQL | “Open phpMyAdmin”. The first time, the kit installs phpMyAdmin (and PHP if it is missing), turns MySQL on if it is off and opens phpMyAdmin in the browser, already connected as root. For another version there is “phpMyAdmin on …” in the “Other versions” part. |
| MongoDB | “Open mongo-express”. The first time, the kit installs mongo-express (and Node.js if it is missing), turns MongoDB on if it is off and opens mongo-express in the browser, already connected as admin: databases, collections, documents, indexes, import and export. For another version there is “mongo-express on …”. Read-only, in the page: the “Content” tab, on the left the databases with their collections and an estimate of the documents (the system ones under “System”), on the right the documents of the chosen collection, 100 per page, with “Previous page”, “Next page” and “Refresh”. |

<details>
<summary>Under the hood: phpMyAdmin and mongo-express</summary>

phpMyAdmin runs with the kit's built-in PHP server on a random loopback address (127.x.y.z, not 127.0.0.1) and on a port chosen by the system, with a new token at every start: without the token it responds 403. Only one is open at a time; it stops with the MySQL version it is connected to or with the app. Its language is Atollo's.

mongo-express is a build of Atollo's: compiled in CI from mongo-express's code at a pinned commit, published in Atollo's releases and checked with its SHA-256 before it is installed. It runs with the kit's Node.js, the same way as phpMyAdmin: a loopback address of its own, a port no other part of Atollo uses, a new token at every start, 403 without it. MongoDB's admin password reaches it only in its environment, never on the command line or in the browser. mongo-express's interface is in English.

</details>

#### Several versions of the same server

You can install several versions of MySQL or MongoDB (from the Tools page). The server uses the one chosen in “Version”, usually the newest. The others appear in the “Other versions” part: each has its own port (3307, 3308… for MySQL, assigned at the first start) and its own data folder, and starts together with the main server if a project uses it.

- At the first start of a newer version, the kit copies the data of the previous one, which stays as it was: you can go back.
- A project chooses the server version in its “Databases” section; it receives the port as DB_PORT.
- Each version keeps its own port: changing the server's “Version” moves no port, and the projects' connections stay right.
- A port that another version, a web server pool or a project action already has, or one between 7000 and 7099 (the editor's), is refused: the message says whose it is.
- Two starts of the same version at once (from the page and from a project, for example) turn on a single server, on the same port.

<details>
<summary>Under the hood</summary>

| What | Where |
| --- | --- |
| Data of each version | `data\db\mysql\8.4 · data\db\mongodb\8.0` |
| Ports, automatic start, chosen version | `data\app\services.json` |
| root and admin passwords | `data\app\db\secrets.json` |
| mysql login in the console | `data\app\db\mysql.cnf` |
| Logs | `data\app\logs\mysql.log · mongodb.log` |

The `data\app\db` folder can be read only by your user on NTFS drives. MySQL uses a my.cnf file written by the kit at every start (only 127.0.0.1, no binary log, lowercase table names); MongoDB always starts with authentication. The mysql command in the console logs in as root only on the main version.

</details>

Related topics: [3.2 A project's database](#32-a-projects-database), [5.2 Several versions of the same tool](#52-several-versions-of-the-same-tool), [9.1 When something goes wrong](#91-when-something-goes-wrong)

### 3.2 A project's database

*A database and a user just for the project, the connection written in the .env file, backups and restoring.*

> **In short**
>
> - Each project has a database and a user all its own, with a password generated by the kit.
> - “Write to .env” puts the connection in the project's .env file, after showing you what changes.
> - “Export” saves a copy of the database; “Restore…” puts it back.
> - Everything is done from Projects › the project › “Databases” section.

![A project's “Databases” section](../../ui/public/guide/en/project-db.webp)

*The project's database, user, password and address, “Write to .env” and the copies, with “Export” and “Restore…”.*

#### Creating the database and user

The section has one part for MySQL and one for MongoDB (with “used by the project” if the kit sees that the project uses it). If the project uses SQLite, a note explains that the database is a file of the project and no server is needed.

1. Press “Create database and user” in the part of the server you need.
2. The kit installs and starts the server if needed, then creates a database and a user with the project's name (lowercase letters, digits and underscore: gestionale-fatture becomes gestionale_fatture) and a 32-character password.
3. The part shows “Database”, “User”, “Password” and “String” (hidden, with “Show” and “Copy”) and “Address”.

> [!NOTE]
> **Good to know**
>
> The project's user can do everything, but only on its own database: a project does not see the data of the others. If the name is reserved (like mysql or admin), the kit adds _db.

#### Writing the connection to the .env file

1. Press “Write to .env”.
2. “Lines of .env that change” appears: for each key, the old value crossed out and the new one (passwords hidden).
3. Press “Write” to confirm, or “Cancel”.

The kit chooses the keys based on your .env file:

| If the .env… | MySQL writes | MongoDB writes |
| --- | --- | --- |
| already contains DB_CONNECTION= (Laravel style) | `DB_CONNECTION, DB_HOST, DB_PORT, DB_DATABASE, DB_USERNAME, DB_PASSWORD` | `DB_CONNECTION=mongodb, DB_DATABASE, MONGODB_URI` |
| does not contain it | `DATABASE_URL` | `MONGODB_URI` |

**Example: Laravel project with MySQL** `.env`

```
DB_CONNECTION=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_DATABASE=gestionale
DB_USERNAME=gestionale
DB_PASSWORD=3f9c…(32 characters generated by the kit)
```

**Example: another project with MySQL and MongoDB** `.env`

```
DATABASE_URL=mysql://portale:3f9c…@127.0.0.1:3306/portale
MONGODB_URI=mongodb://portale:a71e…@127.0.0.1:27017/portale?authSource=portale
```

- If the .env file does not exist, it is created by copying .env.example. The other lines and the comments stay.
- If the connection is already the right one: “The .env file already has this connection.”

> [!WARNING]
> **Watch out**
>
> The .env file contains the password: it must not end up in the repository. If .env is not in the .gitignore file, the kit points it out: add it.

#### Copies and restore

1. In the “Copies” part press “Export”: “Copy saved: …” appears and the copy is added to the list.
2. To go back to a copy, press “Restore…” next to the file and confirm: “The current data is replaced.”
3. “Open the copies folder” opens the folder: to restore a copy that came from outside (from a colleague, from another kit), put it there and it will appear in the list.

> [!TIP]
> **Tip**
>
> The copies are in the kit, in `data\projects\<project>\private\backups`. To be safe, keep one outside the kit too: if you lose the USB stick, you also lose the copies.

#### Changing the server version

If you have several versions of MySQL or MongoDB, the “Server version” menu chooses which one the project's database runs on. When you change it, the database and user are also created on the new version, empty: you carry the data over with a copy (Export on the old one, Restore on the new one). If the chosen version is not installed, “Install …” appears.

<details>
<summary>Under the hood</summary>

- Names, passwords and versions of the project's database: `data\projects\<project>\private\db.json`, readable only by your user on NTFS.
- MySQL: user with caching_sha2_password on localhost and 127.0.0.1, all privileges on the project's database only. MongoDB: dbOwner role on the database, authSource equal to the database name.
- MySQL copies are .sql files made with mysqldump (--single-transaction, with routines, triggers and events); MongoDB copies are .archive.gz files made with mongodump. The MySQL restore runs with the project's user and without client commands; the MongoDB one touches only the project's database.
- When you delete the project from the app, the kit also deletes the database and user on every version where it had created them.

</details>

Related topics: [3.1 The database servers](#31-the-database-servers), [2.7 Deleting a project and the “Data without a project”](#27-deleting-a-project-and-the-data-without-a-project), [7.3 Updates and backups](#73-updates-and-backups)

### 3.3 Websites and .localhost addresses

*Serving a folder as a website: static files, PHP, Caddy or Apache. The shared servers of the “Web Servers” page, .localhost domains and aliases.*

> **In short**
>
> - In a project's “Website” section you choose a folder and how to serve it: static files, PHP, Caddy or Apache.
> - With Caddy and Apache the site joins a server shared with the other projects of the same versions, at an address like http://mio-sito.localhost:8081.
> - The “Web Servers” page shows these shared servers (pools) and lets you create your own (profiles).
> - The sites respond only to this PC, unless you turn on the local network in Settings.

![A project's “Website” section](../../ui/public/guide/en/project-site.webp)

*The folder to serve, the way (static files, PHP, Caddy, Apache) and, for Caddy and Apache, profile, domains and aliases.*

![The Web Servers page](../../ui/public/guide/en/servers.webp)

*Pools and profiles on the left; on the right the linked sites, the details and the start options of the chosen pool.*

#### Serving a project as a website

1. Open the project and its “Website” section.
2. In “Folder to serve as a website” write the folder to publish: . for the whole project, or for example public.
3. In “with” choose how to serve it (see the table).
4. For Caddy and Apache, also choose the “Server profile”.
5. Press “Save”. The site becomes the project's default action: it starts with “Start”.

| Choice | When to use it |
| --- | --- |
| static files | Ready-made HTML, CSS, JavaScript. It is the initial choice. The kit uses its small file server, which also answers as \<project\>.localhost. |
| PHP (built-in server) | A simple PHP site, served by PHP's built-in server. |
| Caddy | A PHP or static site served by Caddy. Caddy does not read .htaccess files: the kit generates the rewrite rules from the project type. |
| Apache | A site that needs Apache and its .htaccess files (many PHP CMSs). |

“Remove the website” removes the configuration. To change it, stop the site first: the change applies from the next start.

#### The server profile: shared or its own

| Profile | What happens |
| --- | --- |
| Automatic (pool by version) | The initial choice. The site joins the shared server of its versions (for example Apache 2.4 with PHP 8.4), together with the other projects that use the same ones: a single port, one name per site. |
| Its own server (the project's port) | A Caddy or an Apache just for this project, on the project's port. The configuration is in `data\projects\<project>\apache` or `\caddy`, and your directives go in the conf.d subfolder (.conf files for Apache, .caddy for Caddy). |
| A profile of your own | One of the profiles created on the Web Servers page, for example with a specific PHP version. |

1. **Start** — Press “Start” in the project.
2. **Pool** — The kit turns on the pool of its versions, or reloads it if it is already on.
3. **Name** — The site gets the name \<project\>.localhost.
4. **Address** — You open it at http://\<project\>.localhost:\<pool port\>.

- “Stop” detaches the site from the pool. A pool turned on by projects shuts down with the last project, unless you started it from the Web Servers page or its profile says “Keep running even without projects”.
- If the automatic pool fails to start, the site uses its own server and the output says so: “The shared server … did not start (…): the site uses its own server.”
- In the pool the site keeps your directives, as with its own server: Apache includes the .conf files of `data\projects\<project>\apache\conf.d`, Caddy imports the .caddy files of `data\projects\<project>\caddy\conf.d`.

#### .localhost domains and aliases

With Caddy and Apache you can give the site other names and link folders from other places. Names ending in .localhost always lead to this PC, without touching the Windows hosts file.

| Field | Example | Effect |
| --- | --- | --- |
| Extra .localhost domains | `api.negozio.localhost, admin.negozio.localhost` | The site also responds to these names. Only names ending in .localhost. |
| Portable path aliases | `/media → storage/uploads` | The address /media serves the project's storage/uploads folder. With @other-project/public you can serve a folder of another project. |

“Add alias” adds a “URL path” → “Relative folder” row; “Remove” removes it. The folders must be relative (no absolute paths and no ..), so the site keeps working if the kit changes drive letter or PC.

#### The “Web Servers” page

The “Active pools and profiles” box lists the shared servers with their port, status and number of sites. Clicking one opens its page:

| Part | What it contains |
| --- | --- |
| Attached VirtualHosts | The pool's sites: project, folder, clickable address, names and aliases. “Detached: the project is stopped” if the project has been stopped. |
| Pool details | Engine, server version, PHP version, port, type (automatic pool or manual profile), configuration folder. |
| Start and stop | For a profile: “Keep running even without projects” and “Start with Atollo”. An automatic pool starts with the first project that uses it and stops with the last one. |

The buttons at the top: “Start” or “Stop”, “Reload configuration” and, for profiles, “Delete profile”, which asks for a confirmation first. A pool still on but no longer tied to projects or profiles shows a warning: stop it. A project created, cloned or deleted appears on the page or disappears from it at once.

1. To create a profile, press “New server profile” at the top of the sidebar.
2. Fill in “Profile name”, “Web server” (Caddy or Apache httpd), “Server version”, “PHP version” and, if you want, “Port” (otherwise the kit picks a free one between 8081 and 8999). A port that another pool, a database version or a project action already has, or one between 7000 and 7099 (the editor's), is refused: the message says whose it is.
3. Choose whether to keep it on even without projects and whether to start it with Atollo, then press “Create profile”.
4. In the projects' “Website” section, choose the profile in the “Server profile” menu.

#### Being reachable from other PCs on the network

Normally the sites respond only to this PC. To try them from a phone or another computer on the same network, turn on “Let other PCs on the network reach the project servers” in Settings › Network: [6.3 Company network: proxy, certificates, local network](#63-company-network-proxy-certificates-local-network). The other devices use this PC's IP address and the port, with PHP's built-in server too; .localhost names work only here. Databases and phpMyAdmin stay closed to the network in any case.

<details>
<summary>Under the hood</summary>

- The site configuration is in the project's panel.json file (site action); the profiles in `data\app\servers.json`.
- The pools' configuration is regenerated at every start and reload in `data\servers\<pool>` (httpd.conf or Caddyfile, a vhosts folder with one file per site, the logs), never in the project.
- Automatic pools have a name like apache-2.4_php-8.4 or caddy-2.11 and a port between 8081 and 8999 that is remembered from one start to the next, chosen outside the ports the other pools, the databases and the project actions already have.
- HTTP only, no HTTPS. PHP runs as FastCGI (php-cgi with Caddy on Windows, php-fpm on macOS and Linux).
- Reload: Caddy reloads without stopping; Apache, or a change of port or PHP, restarts the pool.

</details>

Related topics: [2.3 Starting a project: actions and ports](#23-starting-a-project-actions-and-ports), [6.3 Company network: proxy, certificates, local network](#63-company-network-proxy-certificates-local-network), [9.1 When something goes wrong](#91-when-something-goes-wrong)

### 3.4 The kit's terminals

*Consoles with the kit's tools in any folder on the PC, and terminals of your own with the tools and versions you choose.*

> **In short**
>
> - A kit terminal is a console with only the kit's environment: no profile and no programs from the PC.
> - It opens in any folder on the PC, not only in projects.
> - The “All tools” terminal is always ready; you can create your own with the tools and versions you choose.

![The Terminals page](../../ui/public/guide/en/terminals.webp)

*The “All tools” terminal and your own; on the right the start folder, tools and versions.*

#### The Terminals page

The “Terminals” box lists the default “All tools” terminal (“Every tool at its latest version”) and the ones you created. “Open” on a row opens the console in its starting folder.

- On Windows a classic console window opens with the kit's PowerShell 7.
- On macOS and Linux a tab opens in the terminal panel inside Atollo (with the kit's PowerShell if there is one, otherwise bash).
- When it opens, “Atollo · \<terminal name\>” appears with the list of tools found; the prompt shows Atollo and the current folder.

#### Creating your own terminal

Useful when you always need the same set: for example a “PHP 7.4 legacy” terminal with only PHP 7.4 and Composer, opened in the folder of an old project outside the kit.

1. Press “New terminal” at the top of the sidebar, or “Duplicate” on an existing terminal.
2. In “General” write the “Name” and the “Starting folder” (any folder on the PC: “Choose…” to browse, “Use projects” for the projects folder).
3. In “Tools and versions” tick the tools (groups Agents, Languages, Databases; “All” and “None” to go quickly) and for each one the version, or “The latest”.
4. Press “Create terminal” (or “Save” for an existing one), then “Open”.

- The tools not chosen stay out of the PATH. PowerShell 7, GitHub CLI, the kit's commands and the system's are always there.
- “Open in…” opens the terminal in another folder just that once.
- Choosing a database tool puts its client in the PATH (for example mysql), but does not start the server: you start that from the Databases page.
- The default terminal cannot be changed: duplicate it to create your own. “Delete” asks for a confirmation (Esc closes the question) and removes only the terminal, not the tools.
- If a chosen version is no longer installed, the terminal uses the newest one and says so, for example “PHP 8.2 → 8.4”; a tool removed altogether stays out.

#### Terminals on macOS and Linux

The terminal panel is at the bottom, above the “Activity” panel. Each session is a tab (“app · bash”, “app · claude”…); “Close the terminal” closes the selected one (if a program is still running inside, the app asks for confirmation) and “Hide the terminals” hides the panel. In the status bar, an item counts the open terminals and shows or hides them.

<details>
<summary>Under the hood</summary>

- The terminals you created are in `data\app\terminals.json` (name, folder, tools and versions).
- The console starts with pwsh -NoProfile and the kit's profile (`data\kit\profile.ps1`): your PowerShell profiles and modules on the PC are not loaded.
- In a terminal from the Terminals page, the ATOLLO_TERMINAL variable holds its name: when you enter a project's folder it does not switch versions, it keeps the ones chosen for the terminal. A project's “Terminal”, on the other hand, uses the project's versions.
- Nothing is downloaded when it opens (except the kit's PowerShell if it is missing): what counts is what is already on disk.

</details>

Related topics: [2.5 Editor, terminal and extensions](#25-editor-terminal-and-extensions), [8.1 Portable and isolated: what it means](#81-portable-and-isolated-what-it-means), [5.2 Several versions of the same tool](#52-several-versions-of-the-same-tool)

## 4. AI agents

### 4.1 The agents: Claude Code, Codex, Antigravity

*What they are, how to install them, how to open them in a project, signing in to your account and signing out.*

> **In short**
>
> - An AI agent is an assistant that works in the project folder: it reads the code, changes it, runs commands.
> - The kit offers three: Claude Code (Anthropic), Codex (OpenAI) and Antigravity (Google). You install them from Tools.
> - They always open inside a project, from the Projects page, with the button that shows the agent's logo.
> - You sign in to your account the first time the agent starts, and the sign-in stays in the kit, even on another PC.

![The Agents page](../../ui/public/guide/en/agents.webp)

*The installed agents with sign-in and highest limit; on the right limits and usage, sign-in and shared folder.*

#### What they are and what you need

An agent is a command-line program that talks with an artificial intelligence model from its provider. You tell it in plain English what to do (“add a contact page”, “why does this test fail?”) and it reads the files, proposes changes, runs commands and shows you the results. To use it you need an account with the provider and a plan that includes it, or an API key.

| Agent | Provider | Sign-in |
| --- | --- | --- |
| Claude Code | Anthropic | Claude account (subscription); on macOS also a token from claude setup-token. |
| Codex | OpenAI | ChatGPT account or an API key saved by Codex with codex login --with-api-key. |
| Antigravity | Google | Google account or a Gemini API key, to enter in Settings › Agents. |

Agents are installed from the Tools page, “AI agents” kind (the first start already suggests them). They always follow the latest version and bring along what they need; Git, which they all use, is already among the components that are always installed.

#### Opening an agent in a project

The Agents page does not start agents: an agent always works in a project's folder, so you open it from the Projects page.

1. Open the project.
2. At the top, next to “Editor” and “Terminal”, there is a button for each installed agent, with its logo (“Open Claude Code”, “Open Codex”, “Open Antigravity”). Click the one of the agent you want to use.
3. The agent starts right away. On Windows a kit console opens in the project folder with the agent started; on macOS and Linux, a tab in the terminal panel.

The same buttons are on the project's row (sidebar or “Quick panel”): a click on a logo opens that agent in the project. An agent that is not installed has no button; if none is installed, there are no agent buttons.

<details>
<summary>Under the hood</summary>

- The last agent opened is noted in the project's dev-settings.json file.
- Before the start, the kit installs what is missing (the agent, its dependencies, the programs of the MCP servers such as Node.js or uv), applies the project's versions and syncs instructions, skills and MCP servers. When the agent ends, it syncs them again.
- Antigravity starts through a small kit program (agy) that puts your sign-in back into the system before the start and brings it back into the kit at the end.

</details>

#### The Agents page

On the left, the “Installed” box lists the agents with their version, sign-in status, “running” when they are open and the highest limit reached (for example “5 hours 42%”). On the right, for the chosen agent, three parts:

| Part | What it contains |
| --- | --- |
| Limits and usage | [4.3 Agent limits and usage](#43-agent-limits-and-usage) |
| Sign-in | The sign-in status, “Sign out” and, on macOS for Claude Code, “Connect with a token”. |
| Shared by the agents | [4.2 Shared instructions, skills and MCP servers](#42-shared-instructions-skills-and-mcp-servers) |

At the top of the sidebar: “Refresh usage”, “Open shared folder” (`data\agents`) and the guide's book. If there is no agent: “No agent installed: add them from Tools.”

#### Signing in to your account

| Text | Meaning |
| --- | --- |
| not signed in: you sign in the first time the agent starts | Open the agent in a project and follow its instructions (usually a page in the browser). |
| signed in | The agent is signed in with your account, saved in the kit. |
| signed in with an API key | Codex with the key saved by codex login --with-api-key, or Antigravity with the Gemini key from Settings: no plan limits, you pay for what you use. |
| signed in with a token | Only Claude Code on macOS, connected with a token (see below). |

The sign-in stays in the kit folder: when you take the kit to another PC you do not have to sign in again. “Sign out” removes the sign-in saved in the kit after a question (“Remove the sign-in saved in the kit?”, Esc cancels), with the agent closed: while it is open the button says “Close the agent first”. With an API key there is no “Sign out”.

> [!WARNING]
> **If you lose the USB stick**
>
> The sign-ins of the agents and of GitHub are saved in the kit, unencrypted. “Sign out” does not revoke the sessions on the provider's site: if you lose the USB stick, revoke them from your account settings at Anthropic, OpenAI, Google and GitHub.

> [!NOTE]
> **Claude Code on macOS**
>
> On macOS Claude Code would save the sign-in in the Keychain, which stays on that Mac. “Connect with a token” opens claude setup-token in the app's terminal: you complete the sign-in in the browser and the token (valid for one year) is saved in the kit, so it works on every Mac. If it is not recognized, paste it into the “Claude Code token” field and press “Save the token”. The token reaches, as CLAUDE_CODE_OAUTH_TOKEN, only the Claude Code that Atollo launches: not project actions, scripts, terminals or the editor.

<details>
<summary>Under the hood</summary>

| Agent | Where the sign-in is stored |
| --- | --- |
| Claude Code | `data\home\.claude\.credentials.json` |
| Codex | `data\home\.codex\auth.json` |
| Antigravity | `data\antigravity\keyring.json` |
| Claude Code, token on macOS | `data\agents\claude\oauth-token` |

Antigravity saves its sign-in only in the system keychain (Credential Manager on Windows, Keychain on macOS, Secret Service on Linux). The kit puts it there before the start and brings it back into keyring.json at the end, restoring any Antigravity credentials of the PC (kept, encrypted, in keyring-before.json). On Windows the sign-in in the system lasts only for the Windows session.

</details>

Related topics: [4.2 Shared instructions, skills and MCP servers](#42-shared-instructions-skills-and-mcp-servers), [4.3 Agent limits and usage](#43-agent-limits-and-usage), [6.2 The Settings, one by one](#62-the-settings-one-by-one)

### 4.2 Shared instructions, skills and MCP servers

*A single folder to instruct all three agents at once: AGENTS.md, the skills, the kit's MCP servers and those of a project.*

> **In short**
>
> - Instructions, skills and MCP servers are written only once, in the `data\agents` folder, and apply to all three agents.
> - The kit syncs them when the app starts, before and after each agent and when an agent ends.
> - You can also edit an agent's files: the change passes to the others.
> - MCP servers for a single project are added in the project's “Editor” section.

#### What is shared

| What | What it is for | The file to edit |
| --- | --- | --- |
| Global instructions | Rules and preferences that apply in every project (“reply in English”, “use pnpm”, your coding style). | `data\agents\AGENTS.md` |
| Skills | Reusable procedures that the agent loads when needed, one folder per skill with a SKILL.md file. | `data\agents\skills\<name>\SKILL.md` |
| MCP servers | Programs or services that give the agent new tools (reading documentation, querying a service, using the browser). | `data\agents\mcp.json` |

1. On the Agents page, press “Open shared folder”.
2. Edit AGENTS.md with a text editor, add folders in skills or servers in mcp.json.
3. The change reaches the agents at their next start (or right away, if you restart the app).

**Example mcp.json** `data\agents\mcp.json`

```
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "."]
    },
    "docs": {
      "type": "http",
      "url": "https://example.com/mcp"
    }
  }
}
```

> [!TIP]
> **Tip**
>
> The MCP servers' programs install themselves when an agent starts: npx and node bring Node.js, uvx and uv bring uv, python brings Python.

#### How the sync works

- Instructions: AGENTS.md is copied as CLAUDE.md for Claude Code, AGENTS.md for Codex and AGENTS.md for Antigravity. If you edit one of those files, the change goes back into AGENTS.md and on to the others. If both sides change, the most recent wins and the other is kept in the conflicts folder.
- Skills: each agent sees the skills folder through a link (copies on FAT32 and exFAT drives). A skill created inside an agent is moved to `data\agents\skills` and becomes everyone's; a skill deleted in an agent is set aside and can be recovered.
- MCP servers: mcp.json is translated into each agent's format. With Claude Code it works in both directions: a server added with claude mcp add -s user goes back into mcp.json. Codex and Antigravity receive the kit's servers and leave alone the ones you added by hand in their configurations.

> [!NOTE]
> **The kit's rule**
>
> At every sync the kit gives the agents a common rule: in projects the files CLAUDE.md, AGENTS.md and GEMINI.md are equivalent, and a project's skills are in its .agents/skills folder.

#### MCP servers for a single project

1. Open the project, “Editor” section, “Project MCP servers” box.
2. Press “Add a server”.
3. Write the “Name” (letters, digits, - and _) and choose the “Type”: “program” (with “Program”, for example npx, and the “Arguments, one per line”) or “address (http)” (with the “Address”).
4. Press “Add”. “Remove” on a row removes it.

These servers apply only in that project, in addition to the kit's. Claude Code reads them from the project's .mcp.json file (and asks for confirmation on first use); Codex from `.codex\config.toml`, which Atollo keeps in sync; Antigravity uses only the kit's servers.

<details>
<summary>Under the hood</summary>

| Original | Copies for the agents |
| --- | --- |
| `data\agents\AGENTS.md` | `data\home\.claude\CLAUDE.md · data\home\.codex\AGENTS.md · data\home\.gemini\AGENTS.md` |
| `data\agents\skills` | `data\home\.claude\skills · data\home\.codex\skills · data\home\.gemini\antigravity-cli\skills` |
| `data\agents\mcp.json` | `data\home\.claude\.claude.json · data\home\.codex\config.toml · data\home\.gemini\config\mcp_config.json` |

- The sync state, the conflicts and the deleted skills are in `data\agents\.sync` (conflicts, deleted).
- Sync warnings end up in the app's log, `data\app\logs\app.log`.
- You can also run the sync from the command line: `Atollo.exe sync [--project <folder>] [--verbose]`.
- Claude Code and Codex trust a project's folder only after you open an agent in that project from Atollo: from then on they can run commands there without asking for trust every time. The other folders in projects are not trusted in advance.
- In Codex ${VAR} variables are not expanded: the kit translates them where Codex has a dedicated key, otherwise it copies them as they are with a comment.

</details>

Related topics: [4.1 The agents: Claude Code, Codex, Antigravity](#41-the-agents-claude-code-codex-antigravity), [2.5 Editor, terminal and extensions](#25-editor-terminal-and-extensions), [8.5 The command line](#85-the-command-line)

### 4.3 Agent limits and usage

*How much of your plan you have used, when it resets, and the tokens and sessions of today and the last seven days.*

> **In short**
>
> - “Plan limits” shows how much of your plan you have used in each window and when it resets.
> - “Usage” shows tokens and sessions for today and the last seven days, with a chart per day.
> - The limits are read from the provider's servers every two minutes; “Refresh usage” reads them again right away.

#### Plan limits

For each limit window there is a box with the percentage used, a bar and “resets …”. The windows are “5 hours” and “Week” for Claude Code, “… hours” and “Week” for Codex, and per-model quotas for Antigravity. Above: the plan, the data source and when the data was read.

| Color | When |
| --- | --- |
| Green | below 70% |
| Orange | from 70% |
| Red | from 90% |

The highest value also appears on the agent's row in the sidebar. For Claude Code, paid extra credits are shown separately (“Extra credits: … of …”).

| Message | What to do |
| --- | --- |
| not signed in: open … to see limits and usage | Open the agent in a project and sign in. |
| sign-in expired: open Claude Code to renew it | Open the agent: it renews the sign-in by itself. |
| API key: no plan limits | With an API key there are no limit windows. |
| no data yet: the limits arrive with Codex's next reply | Use Codex once: the limits arrive with the response. |
| The server asks to wait: next reading … | The provider has limited the readings: the kit tries again by itself later. |

#### Usage

- “Tokens · Today” with the tokens of today and the last 7 days; hovering over it shows how many are new, how many come from the cache and how many are output.
- “Sessions · Today” with the sessions of today and the last 7 days.
- A bar chart for the 7 days, the last activity and the three most used models.
- Antigravity does not save tokens: for it only sessions are counted.

> [!NOTE]
> **Good to know**
>
> Usage is calculated from the session files saved in the kit, so it includes the sessions opened from this kit on any PC. There are no money costs: only tokens and sessions.

<details>
<summary>Under the hood</summary>

- Sources of the limits: for Claude Code the Anthropic server, for Codex the OpenAI one (or, failing that, Codex's last response), for Antigravity the Google one. The kit uses the saved sign-in only to read: it does not renew it and does not send it anywhere else.
- Readings at most every 2 minutes, every 20 seconds with “Refresh usage”; after a “too many requests” response it waits as long as the server asks (from 2 to 30 minutes).
- Session files: `data\home\.claude\projects`, `data\home\.codex\sessions`, `data\home\.gemini` (Antigravity conversations).

</details>

Related topics: [4.1 The agents: Claude Code, Codex, Antigravity](#41-the-agents-claude-code-codex-antigravity), [4.2 Shared instructions, skills and MCP servers](#42-shared-instructions-skills-and-mcp-servers)

## 5. Tools and languages

### 5.1 The Tools page

*The table of all the tools split into the six kinds, the five states, installing and updating right away or choosing first and then “Apply”.*

> **In short**
>
> - The Tools page lists all the components the kit can install, divided into six kinds as on the site: system and editor, AI agents, languages, development tools, web servers, databases.
> - The icons on the rows act right away: “Install” and “Update” start as soon as you press them.
> - The checkboxes, instead, prepare a choice that you confirm with “Apply”: this way you install and remove several things at once.
> - Each row shows the status on disk with an icon: not present, pre-installed, installed, to update, error.

![The “All tools” table](../../ui/public/guide/en/tools.webp)

*Filters at the top, one row per component with state, versions and actions; at the bottom the bar with “Update all” and “Apply”.*

#### How the page is laid out

| Part | What it contains |
| --- | --- |
| Search tools | Searches the name, the description and the identifier. |
| “Components” box | A tree with the six kinds and their components, with how many are installed. A click opens the component's tab. |
| “Overview” box | The overall views: “All tools”, “Matrix”, “Editor extensions”, “Relations”, “PC requirements” ([5.3 Overviews: matrix, relations, extensions, requirements](#53-overviews-matrix-relations-extensions-requirements)). |
| “All tools” tab | The first tab, always open: the table of all components. |
| “Apply” bar | At the bottom, always visible: the pending changes, “Update all”, “Apply”. |

| Type | What it contains |
| --- | --- |
| System and editor | Editor, PowerShell 7, GitHub CLI, Git: always installed. |
| AI agents | Claude Code, Codex, Antigravity: optional, always at the latest version. |
| Languages | All languages and runtimes: Python, Node.js, PHP, Go, Java, Rust, Kotlin, .NET, Dart, Zig, Julia, Bun, Deno, C/C++, Ruby, R. All of them can have several versions side by side, except Rust and C/C++, which stay at the latest version. |
| Development tools | uv, pnpm/yarn, CMake/Ninja, Android SDK, Typst, TinyTeX: a single version, always the latest (only Typst can keep more than one); the ones that are needed install themselves. |
| Web server | Apache and Caddy, for the projects' sites and the shared Web Servers. |
| Databases | The servers: MySQL and MongoDB (several versions), SQLite. |
| Database viewers | The databases in the browser: phpMyAdmin for MySQL, mongo-express for MongoDB. They open from the Databases page, which installs them the first time. |

#### The “All tools” table

Above the table are the filters: the search, “Platform”, “Type” (“All types” or one of the six) and “Status” (All, Installed, Not installed, To apply, Used by projects), plus the “Expand” and “Verify” buttons. The columns: name, version, versions on disk, size, status, projects that use it, what it requires and the actions.

The rows are divided by kind. Each kind has a title row: the arrow to fold or unfold it, its name, how many components it holds (“3”, or “2 of 5” when a filter hides some), its description and a “Choose all: …” checkbox that chooses all its components that can be chosen; the checkbox in the header, “Choose all those shown”, does the same with all the visible rows. A click on a column header sorts the rows inside each kind (by name at first).

| Status | Icon | Meaning |
| --- | --- | --- |
| not present | empty circle | There is nothing on disk. The tooltip says why (not chosen, not available on that system…). |
| pre-installed | green package | The archives are already in the kit and the installation completes at the first start on that system. |
| installed | green circle with a check mark | Ready to use. |
| to update | yellow triangle | It must be reinstalled because the kit was moved and that component does not follow it (this happens only on macOS and Linux). It does not mean that a new version exists. |
| error | red circle with an X | The last installation failed, or a part is missing on disk. The tooltip says what. |

While work is in progress, four more states appear, only on the system shown: “in progress” (with the download percentage), “queued”, “to install” and “to remove” (choices not yet applied).

> [!TIP]
> **Tip**
>
> Hover over the status icon: the tooltip gives the component, system, status and the detail (for example “download 42%” or “in use: close the kit programs and try again”).

#### Acting right away, or choosing and applying

| Command | Where | What it does |
| --- | --- | --- |
| Install (cloud) | Row of a component that is not installed | Installs it right away, at the latest version. |
| Install a version (+) | Row of a language that is not installed | Installs it right away in the version you choose from the menu. |
| Update | Row of an installed component | Downloads the new version right away, if there is one. |
| Add a version (+) | Row of an installed language | Downloads another version alongside the ones you have. |
| Checkbox | Every row except those always installed; “Choose all: …” in the kind's row | Marks the component to install or remove, waiting for “Apply”. |
| Apply | Bar at the bottom | Shows the summary and, with “Confirm”, installs and removes what you marked. |
| Update all | Bar at the bottom | Brings every installed component to the latest version and every version to the latest patch. |

1. Tick the components to add and untick the ones to remove. The changed rows are highlighted in color.
2. At the bottom the bar says, for example, “2 to install (180 MB), 1 to remove”.
3. Press “Apply”: “Apply the changes?” appears with the list of what will be installed and removed. “Back” or Esc closes the question without changing anything.
4. Press “Confirm”. First things are removed, then installed; the progress runs in the bar (“Installing 1/3: …”). “Cancel” stops at the end of the current step.

> [!WARNING]
> **Watch out**
>
> “Update all” also applies the pending choices, without the confirmation window. If you have changed checkboxes that you do not want to apply, put them back as they were first.

- Removing a component is possible only with the checkbox and “Apply”, never with an icon: it is an operation that needs confirmation.
- Dependencies follow in both directions: choosing Kotlin chooses Java, removing Java removes Kotlin and Android. The labels on the rows explain it (“Needs Java: installed with it”, “Needed by Kotlin: stays installed while it is needed”).
- A component required by a project but not chosen has a triangle and the label “Needed by the projects: … Not chosen: choose it so they work.”
- A component deleted by hand from the disk is removed from the selection when the page opens, with a warning.

#### A component's tab

A click on the name opens the tab. At the top, like the page of an editor extension: the large logo, the name, kind, status and size, the description and the buttons “Install”/“Keep” and “Remove”/“Do not install” (choices to apply), plus “Official site”. Below, a bar with three sections, one at a time:

| Section | What it contains |
| --- | --- |
| Versions | Only for components with several versions (languages, web servers, databases): the versions on disk and the ones to download ([5.2 Several versions of the same tool](#52-several-versions-of-the-same-tool)). |
| Details | Kind, version, size, what it requires, what needs it, site. |
| Projects | The projects that use it and, for languages, the version each one chose. |

#### “Platform”, “Expand” and “Verify”

- “Platform” chooses which system to view (and prepare) the components for: Windows x64 or ARM64, macOS Apple Silicon or Intel, Linux x64 or ARM64. The same kit on a USB stick opens on all six: from here you can prepare the programs for another system before taking the stick there.
- “Expand” adds a status column for each of the six systems; “Collapse” goes back to the one of the chosen system.
- “Verify” asks every installed component to respond with its version, and adds a ✓ or ⚠ mark in the status cell.

<details>
<summary>Under the hood</summary>

- The selection is in `data\app\components.txt`, one for all systems.
- The programs are in `data\apps\<system>\<component>`; the installed version in `data\app\versions\<system>\<component>.txt`.
- A removed component is also removed from the folders of the other systems that have it on disk.
- The “to update” status corresponds to a \<component\>.reinstall file, written when the kit changes folder and a conda environment (R, Ruby, MongoDB and SQLite on Linux, make) cannot follow it.

</details>

Related topics: [5.2 Several versions of the same tool](#52-several-versions-of-the-same-tool), [5.3 Overviews: matrix, relations, extensions, requirements](#53-overviews-matrix-relations-extensions-requirements), [8.4 Where the tools come from](#84-where-the-tools-come-from)

### 5.2 Several versions of the same tool

*Keeping PHP 8.3 and 8.4 or Node 20 and 24 side by side: adding, removing, pinned versions and which one the kit uses.*

> **In short**
>
> - Languages and servers can have several versions installed together, one per minor version (8.3, 8.4…).
> - The kit always uses the newest one on disk; each project can choose another one on its page.
> - A version used by a project is “locked”: it cannot be removed while the project uses it.
> - “Update all” brings every version to the latest patch (8.4.1 → 8.4.3) without removing any.

#### A component's versions

In the tab's “Versions” section (or under the row, in the table) there is a row for each version on disk, for example “8.4 (8.4.3)”, with some labels:

| Label | Meaning |
| --- | --- |
| latest | It is the newest version offered by the source. |
| locked (padlock) | Some projects use it: the “Used by” menu says which ones. It cannot be removed. |
| not for … | Another component is not compatible with this version (for example an extension that does not support it). |
| trash can “Remove …” | Removes that version. It appears only if there are at least two: the last one is removed with the component's checkbox. |

1. To add a version, press + “Add a version of …”: the “Versions to download” menu groups them (8.x, 7.x…).
2. Choose the version: the download starts and the row appears in grey until it is ready.
3. If the list seems old, “Look for other versions of …” asks the source for it again.

> [!NOTE]
> **Which version the kit uses**
>
> There is no “default” version to choose: the kit uses the newest one it has on disk. A new project pins the one available when it is created; to change it, go to the project's page, “Components” › “Versions” ([2.4 Components and versions of a project](#24-components-and-versions-of-a-project)).

#### Special cases

- Composer follows PHP: there is one copy for each PHP version, compatible with it; it is installed, updated and removed with PHP.
- Maven and Gradle follow Java and appear as labels on the version's row.
- MySQL and MongoDB: each version has its own data and its own port, which stays its own even when you change the server's version; see the Databases page.
- Python and PHP: a new installation takes the newest line; the other lines are downloaded from this menu and each project chooses its own. Settings no longer has a version choice (the pythonVersion and phpVersion values of an older settings.json are ignored).
- Rust, only on Windows: on its page, “Details” section, “Rust compiler” chooses GNU (default, all in the kit) or MSVC, which requires Visual Studio Build Tools on the PC. It applies from the next install or update of Rust. On macOS and Linux Rust has a single compiler and the setting does not appear.

<details>
<summary>Under the hood</summary>

- Each version lives in its own folder: `data\apps\<system>\<component>\<minor version>`, with the full version in the .atollo-version file.
- The lists of offered versions are in `data\app\version-lines.json`, refreshed when they are more than 24 hours old.
- On Windows PHP also offers the last builds of versions no longer supported (8.1, 8.0, 7.4…), useful for old projects; on macOS and Linux only those with a build pinned by the kit.

</details>

Related topics: [2.4 Components and versions of a project](#24-components-and-versions-of-a-project), [5.1 The Tools page](#51-the-tools-page), [3.1 The database servers](#31-the-database-servers)

### 5.3 Overviews: matrix, relations, extensions, requirements

*The overall views of the Tools page: the six systems, who requires whom, the extensions per language and what the PC needs.*

> **In short**
>
> - “Matrix”: where each component can be, on the six systems the kit supports.
> - “Relations”: what requires what and with which versions, and the versions the projects ask for.
> - “Editor extensions”: which extensions each language receives.
> - “PC requirements”: the system components the PC needs.

#### Matrix

A table with one component per row, split into the same six kinds as the “All tools” table, and the six systems as columns. Each cell says whether the component can be on that system: ✓ available, ≈ emulated (the x64 version on an ARM64 PC), ↓ pre-installed, ◷ completes at the first start, ✕ not available; a green dot means installed. A click on a cell shows the reason, the version and the “Show this system” button.

> [!NOTE]
> **Good to know**
>
> The Matrix says what is possible; the “All tools” table says what is really on disk.

#### Relations

Two tables, recalculated when something changes (or with “Compute the relations again”):

| Table | What it says |
| --- | --- |
| Relations matrix | For each tool, what it requires, with which versions, and whether it is fine (“yes”, “no: the kit uses …”, “… missing”). |
| Versions the projects ask for | For each project, the version its files ask for, from which file, and the status (“uses …”, “to download: …”), with the “Use …” and “Download and use …” buttons. |

#### Editor extensions

A read-only table: the extensions as rows, the languages and tools as columns. ● means the extension is on by default in projects with that language, ○ that it is optional; ✓ that it is installed. The languages of your projects are highlighted. Extensions are chosen per project: [2.5 Editor, terminal and extensions](#25-editor-terminal-and-extensions).

#### PC requirements

Some components need something from the system: Visual C++ 2015-2026 for PHP, Apache and MongoDB on Windows, WebView2 for the app window, the Windows 11 x64 emulation on ARM64 PCs. Each row says which components need it and whether it is “present”, “missing” or “not needed”.

<details>
<summary>Under the hood</summary>

- The relation rules are in the kit's catalog (tauri/catalog/relations.json); the result in `data\app\relations.json`.
- Project files the kit reads for the versions: `package.json, .nvmrc, .python-version, pyproject.toml, composer.json, go.mod, global.json, pom.xml, Gemfile, Cargo.toml` and others.
- The extension catalog (from Open VSX) is tauri/catalog/extensions.json.

</details>

Related topics: [5.1 The Tools page](#51-the-tools-page), [2.4 Components and versions of a project](#24-components-and-versions-of-a-project), [7.2 On another PC, another letter, another folder](#72-on-another-pc-another-letter-another-folder)

## 6. Account and settings

### 6.1 The kit's GitHub account

*Why to connect it, how to sign in, what it unlocks (repositories, Git, GitHub CLI) and what works without it too.*

> **In short**
>
> - One GitHub sign-in works for the whole kit: the Projects page, Git (in the editor, the consoles and the agents too) and GitHub CLI.
> - You do it in Settings › “GitHub account” with “Sign in to GitHub” and a code to type in the browser.
> - Without an account the kit works all the same: you clone public repositories and work locally.
> - The sign-in stays in the kit: on another PC you are already signed in.
> - With the sign-in, the commits' name and email are filled from the account, with GitHub's noreply address; you change them in “Who signs the commits”.

#### What it unlocks

| With the account | Even without |
| --- | --- |
| The list of your repositories in Projects, with clone, rename and delete | Cloning public repositories with “From a Git address” |
| Cloning private repositories | Creating projects, starting them, databases, sites, agents, tools |
| git push and git pull to GitHub without a password | Using Git locally (commits, branches) |
| The gh commands in the consoles, the editor and the agents | — |
| More requests to the GitHub API for downloads (5,000 an hour instead of 60) | Downloading the components (with the lower limit and a few fallbacks) |

#### Signing in

1. Open Settings › “GitHub account”.
2. Press “Sign in to GitHub”. After “Preparing the sign-in…” an eight-character code appears (XXXX-XXXX) and github.com opens in the browser.
3. Press “Copy the code”, paste it into the GitHub page and authorize GitHub CLI. If the page did not open, use “Open the GitHub page”.
4. Go back to Atollo: “Signed in as @your-name” appears.

“Sign out” removes the sign-in from the kit after the confirmation “Sign out of GitHub? Git, the editor and the kit's agents lose the access too.”; “Cancel” or Esc closes the question.

> [!WARNING]
> **Watch out**
>
> The GitHub token is saved in the kit, in plain text, so that it follows you to every PC. “Sign out” does not revoke the session on github.com: if you lose the USB stick, revoke the GitHub CLI authorization from your GitHub account settings (Applications).

#### The permission to delete repositories

The normal sign-in does not include the permission to delete repositories. The first time you press “Delete from GitHub” on one of your repositories, the panel says the permission is missing and shows “Grant the permission”: the kit signs in again, asking for that too. The code to type appears in Settings › “GitHub account”. Then press “Delete from GitHub” again.

> [!WARNING]
> **Watch out**
>
> Deleting a repository from GitHub cannot be undone: issues, pull requests, wiki and releases disappear too. The copy in the kit, if there is one, stays.

#### Your name in commits

In Settings › “GitHub account”, below the account, “Who signs the commits” has two fields, “Name” and “Email”: they are the name and email of the commits made with the kit's Git. Each one is saved when the field loses focus.

- If both are empty and the kit is signed in to GitHub, Atollo fills them by itself, at sign-in and at every start: the account's name (or login) and GitHub's private address, id+login@users.noreply.github.com. That way your real email stays private.
- You can type another name and email whenever you like. If you empty both, they are removed; at the next start, if the kit is still signed in to GitHub, the account's come back.
- A value that is not valid shows “Invalid name.” or “Invalid email address.”. If Git is not in the kit yet, “Git is not in the kit yet: it is installed with GitHub CLI.” appears.

They are Git's user.name and user.email values: in a kit terminal git config --global works too.

**Name and email for commits**

```
git config --global user.name "Nome Cognome"
git config --global user.email "tu@esempio.it"
```

<details>
<summary>Under the hood</summary>

- The sign-in runs gh auth login --web --insecure-storage --scopes workflow: besides the standard GitHub CLI permissions (repo, read:org, gist) it asks for workflow; delete_repo only with “Grant the permission”.
- The token is in `data\gh\hosts.yml` (GH_CONFIG_DIR points there), never in the Windows Credential Manager.
- Git uses the same account through a credential helper (gh auth git-credential) written in the file `data\home\.gitconfig` for github.com and gist.github.com, without absolute paths: it survives moving the kit. At every start the kit aligns it with the state of gh; a helper you wrote yourself for those addresses stays as it is.
- “Who signs the commits” reads and writes git config --global user.name and user.email, that is the kit's `data\home\.gitconfig` file. Filling them from the account takes one gh api user call.
- Git trusts only the repositories in the kit's folder: the kit's .gitconfig has safe.directory = \<kit folder\>/\* (for example D:/Atollo/\*), rewritten at every start so it follows the kit when it changes folder; values you added stay. Before it was safe.directory = \*, that is every repository on the PC, even other users'.
- For a repository outside the kit that belongs to another user, Git again warns about “dubious ownership”. If you trust it, in a kit terminal: git config --global --add safe.directory \<folder\>.
- To clone a private repository the token is passed to Git only for that operation, never on the command line nor in the `.git\config` file.
- For downloads the kit first uses the PC's GITHUB_TOKEN or GH_TOKEN, if present, then the kit's account. The API responses are kept in `data\cache\github`.
- The editor's “Accounts” menu does not see the kit's account; push and pull in the editor work anyway, through Git.

</details>

Related topics: [2.1 Creating a project](#21-creating-a-project), [2.6 A project's Git](#26-a-projects-git), [8.4 Where the tools come from](#84-where-the-tools-come-from)

### 6.2 The Settings, one by one

*Automatic saving and the seven sections: language and theme, temporary files, network, Gemini key, automatic updates, who signs the commits, privacy and the information about the kit.*

> **In short**
>
> - Settings has seven sections in the sidebar (General, Network, Agents, Updates, GitHub account, Privacy, The kit) and, below them, the framework instructions.
> - There is no “Save” button: every change saves itself half a second later, and the tab bar shows “Saving…” and then “Saved.”.
> - They apply at once to the app, the consoles and the agents; to the editor's terminals after “Restart the editor”.

![The Settings page](../../ui/public/guide/en/settings.webp)

*The sections on the left, the framework instructions below them; the saving status on the tab bar.*

#### How changes are saved

- Every change is saved and applied by itself, about half a second after the last keystroke: the tab bar shows “Saving…”, then “Saved.”.
- If a value is not valid (for example a PEM file that does not exist), its error appears and the value stays on screen: the kit tries again at the next change.
- The framework instructions save themselves too, a little under a second after the last edit.
- As in the editor, a setting changed from its default value has a colored bar on its left edge.
- If the editor is open and you change the language, the network or the programs' temporary files, “Restart the editor so that its terminals use the new settings.” appears: the other settings do not need it. The theme reaches the editor and its terminals at once.

#### General

| Setting | What it does |
| --- | --- |
| Language | The language of Atollo: the system's (“System language”), Italian, English, French or German. It changes at once. |
| Theme | Light or dark: “Same as the system” (the default), “Light” or “Dark”. It applies at once to every window: Atollo, the integrated editor and its terminals, without restarting the editor. |
| Temporary files of programs | Where the kit's programs put their temporary files. “Automatic” uses the system's temporary folder if the kit is on a USB stick, a network drive or a FAT32/exFAT drive, otherwise `data\tmp`. “In the kit” always `data\tmp`; “In the system” always the system's folder (the files stay on the PC until you close the app). |
| Automatic update | “Download new versions of Atollo automatically (they replace the exe on close)”: the new version replaces the current one when you close the app. On by default. It applies at once: turning it on starts a check right away; turning it off before the check 20 seconds after start stops that check. |
| Packages to reinstall without the network | Only on macOS and Linux: keeps in the kit the packages of the components that must be reinstalled when the kit changes folder (R, Ruby, MongoDB, SQLite, make), so the network is not needed. “Free” deletes them. |

> [!TIP]
> **Why temporary files go outside the kit on a USB stick**
>
> Programs constantly write and delete temporary files: on a USB stick this slows everything down and wears it out. On the PC's disk they are much faster, and the kit deletes them when you close the app.

> [!NOTE]
> **Versions and Rust compiler**
>
> Settings no longer chooses the Python and PHP versions: a new installation takes the newest line, the other lines are downloaded from each tool's versions menu in Tools, and each project chooses its own ([5.2 Several versions of the same tool](#52-several-versions-of-the-same-tool)). The pythonVersion and phpVersion values of an older settings.json are ignored. The “Rust compiler” is on Rust's page in Tools, “Details” section.

#### Network

Proxy, additional certificates and local network: [6.3 Company network: proxy, certificates, local network](#63-company-network-proxy-certificates-local-network).

#### Agents

“Gemini API key (Antigravity without a Google account)”: paste the key and move on (a click outside the field, or Tab). The key is saved when the field loses focus, then the field empties and shows “saved: type to replace it”: the key is no longer shown. “Remove the key” deletes it at once. Antigravity will show “signed in with an API key”. The key is saved on its own, in `data\app\secrets\gemini-api-key`, readable by your user only where the disk has permissions, and passed as GEMINI_API_KEY only to the Antigravity that Atollo launches: not to project actions, scripts, terminals or the editor.

#### Updates

The state of Atollo's updates and the “Check for updates” button: [7.3 Updates and backups](#73-updates-and-backups).

#### GitHub account

The GitHub sign-in for the whole kit: [6.1 The kit's GitHub account](#61-the-kits-github-account).

Below the account, “Who signs the commits” has the “Name” and “Email” fields of the commits made with the kit's Git; each one is saved when the field loses focus.

#### Privacy

“What Atollo does with your data”, in short: no telemetry and no account; settings, keys and sign-ins stay in the kit's folder. Atollo contacts GitHub for its own updates (if automatic updates are on), the official sources of languages and tools when you install or update them and, once a day, to read the available versions (ATOLLO_NO_UPDATE turns this off too), and Open VSX for the editor's extensions. While the Agents page is open, every 2 minutes it reads the plan limits of each signed-in agent, sending that agent's token only to its provider; with the GitHub account signed in it uses that account's token with api.github.com. Agents, GitHub and package registries talk directly to their own services, under their own rules.

“Read the privacy policy” opens the full policy in the browser (docs/PRIVACY.md on GitHub), in the app's language.

#### The kit

Read-only information: kit folder, projects folder, data and programs folder, the kit's disk (file system and type: fixed, removable, network), programs' temporary folder, platform, WebView2 and Atollo versions. If the disk has something to report, a warning appears.

#### Frameworks

The instructions of “New project”: [6.4 The framework instructions](#64-the-framework-instructions).

<details>
<summary>Under the hood</summary>

- The settings are in `data\app\settings.json`. Every save reads the file again, checks the values, writes it safely (first a copy, then the replacement) and rebuilds the kit's environment. Saves go through one at a time, always with the latest values.
- The theme is the theme entry of settings.json. Atollo passes it to the windows (Tauri set_theme): their pages see it as prefers-color-scheme. The integrated editor follows it thanks to window.autoDetectColorScheme, and the terminals with it.
- If the file cannot be read, the app uses the default values and rewrites it at the first change: “The settings in … cannot be read: using the defaults. The file will be rewritten at the first change.” Entries written by a newer version of Atollo, which this one does not know, stay in the file.
- The Settings search box also searches the framework bases and additions.

</details>

Related topics: [6.3 Company network: proxy, certificates, local network](#63-company-network-proxy-certificates-local-network), [7.3 Updates and backups](#73-updates-and-backups), [6.4 The framework instructions](#64-the-framework-instructions)

### 6.3 Company network: proxy, certificates, local network

*When you need them, how to fill them in and what they apply to. Plus how to let the other PCs on the network see your websites.*

> **In short**
>
> - They are needed only if the network goes through a proxy or inspects HTTPS traffic, as in many companies and schools.
> - They apply to the kit's downloads, to the agents and to the tools in the consoles.
> - “Local network” lets the other PCs on the network see the projects' sites.

#### Working out whether you need them

If everything downloads at home but at the office downloads fail with connection or certificate errors, the company network probably uses a proxy, or inspects HTTPS traffic with its own certificate. Your IT support can give you the right values; often they are the same as those set in the company browser.

#### The fields

| Field | Example | Notes |
| --- | --- | --- |
| HTTPS proxy | `http://proxy.azienda.it:8080` | The most important one: Atollo's downloads use it too. |
| HTTP proxy | `http://proxy.azienda.it:8080` | For unencrypted connections. |
| No proxy for | `localhost,127.0.0.1,.azienda.it` | Addresses to reach directly, separated by commas. |
| Additional certificates (PEM file) | `E:\certificati\azienda.pem` | The company's certificate, for programs like Node.js that do not use the system's ones. |

1. Open Settings › “Network” (at the first start, the “Network” group of the “Welcome to Atollo” page: nothing downloads before “Install”) and fill in the fields you need.
2. There is nothing to press: the values save themselves as you type. If the editor is open, press “Restart the editor” in the notice that appears.
3. Go back to Tools and retry the download that was failing.

- The proxy must start with http:// or https:// and have no spaces; if a user is needed: http://user:password@host:port.
- The PEM file must exist: the kit checks the path at every save. As long as the value is not valid its error appears and the field is not saved; the kit tries again at the next change.

#### Local network

The “Let other PCs on the network reach the project servers” checkbox opens the projects' sites to the network: useful to try a site from your phone. From the other devices you use this PC's IP address with the site's port (for example http://192.168.1.20:8000). Databases and phpMyAdmin stay reachable only from this PC.

- PHP's built-in server (php -S) and the file server of static sites answer whoever calls them by the PC's IP address; with any other name the request stays refused.
- On this PC a static site also answers at http://\<project\>.localhost:\<port\>, besides localhost and 127.0.0.1, with the checkbox on or off.

> [!WARNING]
> **Watch out**
>
> Anyone on the same network will be able to see your test sites. Turn it on on networks you trust and turn it off when you are done. Windows may ask for the firewall permission the first time.

<details>
<summary>Under the hood</summary>

- The values end up in the HTTPS_PROXY, HTTP_PROXY, NO_PROXY and NODE_EXTRA_CA_CERTS variables of all the kit's programs. With a PEM file, `data\app\ca-bundle.pem` joins the system's certificates and its own, for the tools that read a single file (SSL_CERT_FILE, REQUESTS_CA_BUNDLE, PIP_CERT, CURL_CA_BUNDLE, GIT_SSL_CAINFO, CARGO_HTTP_CAINFO). A PEM file inside the kit is saved relative to the kit: it follows it to another drive or folder.
- Atollo's own downloads use these settings too: the “HTTPS proxy” (if empty the “HTTP proxy”, otherwise the PC's variables or the system proxy), the “No proxy for” hosts reached directly, and the system's certificates plus those of the PEM file. If the company certificate is already installed on the PC, the PEM file is not needed. A file with no PEM certificate is refused.
- PAC files (automatic proxy configuration) are not supported: ask for the proxy address.
- “Local network” sets ATOLLO_ALLOW_LAN=1: the projects' commands listen on all interfaces (127.0.0.1 in the command becomes 0.0.0.0), and so do the file server and the Caddy and Apache pools. The php -S router and the file server then also accept an IP address as Host; to any other name that is not localhost, \*.localhost, 127.0.0.1 or \[::1\] they answer 421.

</details>

Related topics: [8.4 Where the tools come from](#84-where-the-tools-come-from), [9.1 When something goes wrong](#91-when-something-goes-wrong), [3.3 Websites and .localhost addresses](#33-websites-and-localhost-addresses)

### 6.4 The framework instructions

*Editing the recipes of “New project”: bases, additions, commands, .env variables, actions. And trying a combination before using it.*

> **In short**
>
> - The bases and additions of “New project” are recipes: lists of commands you can read and change.
> - You find them in Settings, in the “Frameworks” panel.
> - You can change an official recipe (and restore it) or create your own.
> - “Try a combination” shows what a base with some additions would do, before you use it.

#### What a recipe is made of

1. **Tools** — The kit components it needs (Node.js, PHP…).
2. **Create** — The commands that create the project, run in the projects folder.
3. **Then** — The commands to run in the project's folder.
4. **Extras** — Database, .env variables, project actions.

Each line is a command run with PowerShell and the kit's tools. {name} becomes the project's name (in quotes), {package} com.example. followed by the name in lowercase.

#### The “Frameworks” panel

The tree has the “Bases”, divided by group, and under each base the additions that go with it; then the “Additions” by type; at the bottom “Try a combination”. At the top, the “Add a base” and “Add an addition” icons.

| Field | What to write |
| --- | --- |
| Name, Description, Official page | How it appears in the wizard and the link to the documentation. |
| Kit tools | The components to install first (“Add a kit tool”). |
| Create the project (in the projects folder) | For example: npm create vite@latest {name} -- --template vanilla-ts |
| Then, in the project's folder | For example: npm install |
| Kit database for the project | None, MySQL or MongoDB. The password the wizard generates for the database wins over an empty one from the base (for example DB_PASSWORD= in .env.example). |
| .env file variables (KEY=value) | For example: APP_NAME={name} |
| Project actions (Name: command) | For example: Start: npm run dev |

An addition also has the “Group” (two additions of the same group do not go together), “Use it also with” for the compatible bases and, for each base, the commands to add or “Create the project instead of the base”.

1. Choose a base or an addition in the tree and edit the fields.
2. The instructions save themselves a little under a second after the last edit: the tab bar shows “Saving…”, then “Saved.”. A changed official recipe shows “Modified”.
3. To go back to the original version use “Restore the official instructions”; a recipe of your own is removed with “Remove”.

#### Try a combination

Choose a base and some additions: the tab shows the complete plan the wizard would run (tools, commands, database, .env, actions), with the instructions as they are now, even if the last edit is not saved yet. It runs nothing.

<details>
<summary>Under the hood</summary>

- The official recipes are in the kit's catalog (tauri/catalog/frameworks.json): 63 bases and 18 additions.
- Your changes are in `data\app\frameworks.json`, which contains only the differences from the official recipes.
- If that file cannot be read, the kit uses the official recipes and does not touch it: “The instructions in … cannot be read: using the official ones and leaving the file as it is.”

</details>

Related topics: [2.1 Creating a project](#21-creating-a-project), [6.2 The Settings, one by one](#62-the-settings-one-by-one)

## 7. Taking the kit with you

### 7.1 Safely removing the drive

*Closing everything the kit has opened before unplugging the USB stick or the disk, without losing data.*

> **In short**
>
> - Before unplugging the kit's USB stick or drive, use the “Remove drive” page.
> - It neatly closes everything the kit has opened: projects, servers, editor, consoles, databases.
> - Then close Atollo and remove the drive with Windows' “Safely Remove Hardware”.

![The “Remove drive” page](../../ui/public/guide/en/eject.webp)

*What is open right now and the button that closes everything before you unplug the drive.*

#### Why it matters

A database switched off abruptly can lose the last data written, and a program with a file open on the USB stick prevents Windows from removing it. The page closes everything the right way: the databases with their shutdown command, the servers and the programs one by one. And it saves the Antigravity sign-in in the kit, so nothing is left on the PC.

#### Step by step

1. Save the open work in the editor and the other programs.
2. Open “Remove drive” (the last icon at the bottom of the activity bar). The “Open now” part lists what is open: installation in progress, editor, consoles, running projects, running databases and the kit programs started outside the app, with name and PID.
3. Press “Close everything and prepare for removal”.
4. When “Kit programs closed and Antigravity sign-in saved in the kit.” appears, press “Close Atollo”. If something is still open, it is listed under “Still open (close them manually)”.
5. Remove the drive from the system: on Windows with the “Safely Remove Hardware” icon; on macOS by ejecting it from the Finder.

> [!WARNING]
> **Watch out**
>
> Unsaved files in the listed programs will be lost. The page does not eject the drive by itself: you do the last step from the system.

#### And if I just close the window?

If something is open, Atollo asks “Close Atollo?” and lists what will be closed. “Close everything” stops actions and servers, shuts down the databases neatly and exits. This is safe too; the “Remove drive” page also closes the kit programs started outside the app (for example from a console) and tells you if anything is left.

The same goes for “Close Atollo” in the menu of Atollo's icon, in the Windows notification area, the macOS menu bar or the Linux status area. That menu also has “Show Atollo” and “Projects”, with the most recent projects: a click opens the project's page.

<details>
<summary>Under the hood</summary>

The order in which the page closes things:

1. **Installation** — It stops at the end of the current step.
2. **Projects** — The running actions.
3. **Web servers** — The Caddy and Apache pools.
4. **Editor** — The windows; the editor server stops right after the databases.
5. **Databases** — Orderly shutdown of MySQL and MongoDB, every running version.
6. **The rest** — Consoles and every process whose program is inside the kit.
7. **Antigravity** — The sign-in goes back into the kit, the system goes back to how it was.

The kit cannot find the PC's programs that have opened a file on the USB stick (for example a document open in an outside app): close them yourself.

</details>

Related topics: [7.2 On another PC, another letter, another folder](#72-on-another-pc-another-letter-another-folder), [8.3 Background processes](#83-background-processes), [7.3 Updates and backups](#73-updates-and-backups)

### 7.2 On another PC, another letter, another folder

*What happens when the kit changes place, and how it behaves on the same disk opened from Windows, macOS and Linux.*

> **In short**
>
> - You can move or copy the kit folder wherever you like: another drive letter, another folder, another PC.
> - At the first start in the new place the kit notices and fixes the paths by itself.
> - The same USB stick opens on Windows, macOS and Linux: projects and data are shared, the programs are kept separate for each system.

#### Changing letter, folder or PC

On another PC the USB stick might become E: instead of D:. You do not have to do anything: at start the kit compares the current folder with the one from last time and, if it changed, shows “The kit has been moved: it was in …, it is now in …” and updates the paths.

- The programs' environment (PATH and variables) is rebuilt at every start from the current folder.
- The paths written in the agents' and the editor's files are rewritten with the new path.
- The web servers' configurations are regenerated at every start and use relative paths.
- In the kit's .gitconfig, safe.directory (the folders Git trusts) is rewritten at every start with the kit's current folder, for example D:/Atollo/\*. The kit removes only the value it wrote before (it remembers it in `data\app\safe-directory.txt`): the folders you added yourself stay.
- The certificates' PEM file (Settings › Network) and a terminal's folder, when they are inside the kit, are saved relative to the kit: they follow the move.
- Sign-ins, settings, projects and databases are in the folder: they travel with it.

> [!NOTE]
> **Good to know**
>
> Only one kit at a time can be open per folder: starting Atollo again from the same folder brings the window already open to the front. Two different kits (two USB sticks), on the other hand, can run together.

#### The same kit on Windows, macOS and Linux

The programs are different for each system, and the kit keeps them in separate folders (`data\apps\win32-x64`, `data\apps\darwin-arm64`, `data\apps\linux-x64…`). Everything else is shared: projects, settings, sign-ins, agent instructions, databases.

1. Format the USB stick as exFAT, which all three systems can read and write.
2. Put the executable of each system in the kit folder (Atollo.exe, Atollo.app, the AppImage file).
3. On each system, open Atollo: the first time it installs the components chosen for that system. Alternatively, on the Tools page choose that system in “Platform” and prepare the programs in advance from another PC.

> [!WARNING]
> **Watch out**
>
> A database used from several systems must always be shut down before changing PC (use “Remove drive”). On macOS and Linux some components (R, Ruby, MongoDB on Linux, SQLite on Linux ARM64, make) must be reinstalled when the kit changes folder: they show as “to update” and the kit reinstalls them by itself, without the network if you turned on “Packages to reinstall without the network”.

<details>
<summary>Under the hood</summary>

- The last folder is in `data\app\root.txt` (root-darwin.txt and root-linux.txt on macOS and Linux); the previous one in old-root.txt until the agents' files have been updated.
- On Windows the folder comparison ignores upper and lower case; on macOS and Linux it does not.
- The installed versions of each system are in `data\app\versions\<system>`.
- macOS: the app is not notarized by Apple. If macOS opens it from a temporary location (“App Translocation”), Atollo stops and shows the xattr command to run to remove the quarantine. Linux: glibc 2.35 or newer and WebKitGTK 4.1 are needed; if they are missing, Atollo shows the command to install them.

</details>

Related topics: [7.1 Safely removing the drive](#71-safely-removing-the-drive), [8.1 Portable and isolated: what it means](#81-portable-and-isolated-what-it-means), [5.3 Overviews: matrix, relations, extensions, requirements](#53-overviews-matrix-relations-extensions-requirements)

### 7.3 Updates and backups

*Updating Atollo and its tools, and what to copy to have a complete backup of your work.*

> **In short**
>
> - Atollo updates itself: it downloads the new version and puts it in place of the old one when you close the app.
> - The tools are updated from the Tools page, with “Update \<name\>” on a row or “Update all”.
> - For a complete backup it is enough to copy projects and data (without the programs, which can be downloaded again).

#### Updating Atollo

With “Automatic update” on (Settings › General), twenty seconds after start the kit checks whether there is a new version, downloads it, verifies its hash and prepares it. When you close the app, the new version takes the place of the old one; at the next start the old copy is deleted. The switch applies at once: turning it on starts a check right away, turning it off before the twenty seconds stops that check.

In Settings › “Updates” you see the state (“Atollo … is the latest version.”, “Version … is ready: it replaces this one when Atollo closes.”) and the “Check for updates” button, which checks and downloads right away even with automatic update off.

> [!NOTE]
> **Good to know**
>
> On macOS and Linux, if you use the plain program (from the tar.gz archive) instead of Atollo.app or the AppImage, the kit cannot replace it: it tells you the new version is available and you download it from the release page.

#### Updating the tools

- “Update \<name\>” (the icon with the two arrows) on a component's row downloads its new version right away, if there is one.
- “Update all”, at the bottom of the Tools page, updates every installed component and brings every version to its latest fix (for example PHP 8.4.1 → 8.4.3), without adding or removing versions.
- The agents always follow the latest version and do not update themselves while you work: the kit does it with “Update \<name\>” or “Update all”.

> [!TIP]
> **Tip**
>
> Update when you have no projects running: programs in use cannot be replaced, and the row would show “in use: close the kit programs and try again”.

#### Backups

| To copy | Why |
| --- | --- |
| `projects\` | Your projects. |
| `data\` | Settings, sign-ins, agent instructions, database data, database backups, project data. |

Inside data you can leave out `data\apps` (the programs) and `data\cache` (downloaded files that can be downloaded again): they are the biggest part and the kit rebuilds them. To copy the database data consistently, shut down the servers first (or use “Remove drive”).

> [!WARNING]
> **Watch out**
>
> The data folder contains sign-ins and passwords. Keep the backups in a safe place, and encrypt the USB stick if you carry it around (for example with BitLocker To Go on Windows).

For a single database there is the backup from the project page: [3.2 A project's database](#32-a-projects-database).

<details>
<summary>Under the hood</summary>

- New versions come from the app-v… releases of the GitHub repository nabre/atollo; the SHA-256 hash comes from GitHub or from the .sha256 file next to the file.
- On Windows the new version waits in `data\app\update`; the replaced copy stays as Atollo.old.exe until the next start.
- The ATOLLO_NO_UPDATE variable, set on the PC, turns off the automatic check and also the daily reading of the versions the sources offer.

</details>

Related topics: [5.1 The Tools page](#51-the-tools-page), [7.1 Safely removing the drive](#71-safely-removing-the-drive), [8.2 The kit's folders on disk](#82-the-kits-folders-on-disk)

## 8. How it works inside

### 8.1 Portable and isolated: what it means

*Why the kit does not install, does not ask for administrator rights and does not touch the PC that hosts it. And the exceptions.*

> **In short**
>
> - Atollo does not install itself, does not write to the registry and does not ask for administrator rights.
> - The kit's programs start in an environment built from scratch, which points only to kit folders.
> - So they do not see the PC's programs and the PC does not see them: no conflicts, no traces.
> - A few exceptions are declared: the system's temporary folder and, for a few seconds, the keychain.

#### The idea, with an example

A colleague's PC already has Node.js 18 installed, with its global packages and its npm settings. If you open a kit console and type node --version, the kit's Node 24 answers, not the PC's 18: the kit console does not even know the other one exists. And when you install a global package, it ends up in the kit folder, not in your colleague's profile.

This happens because every program started by the kit receives an environment built from scratch: the kit takes only a handful of system variables from the PC (computer name, Windows folders, language…) and sets all the others so that they point inside data.

#### What is redirected

| Group | Main variables | Where they point |
| --- | --- | --- |
| User folder | `HOME, USERPROFILE, APPDATA, LOCALAPPDATA` | `data\home` |
| Temporary files | `TEMP, TMP (TMPDIR su macOS e Linux)` | `data\tmp\system` or the system's temporary folder, depending on Settings |
| Agents | `CLAUDE_CONFIG_DIR, CODEX_HOME` | `data\home\.claude, data\home\.codex` |
| GitHub CLI | `GH_CONFIG_DIR` | `data\gh` |
| Package caches | `NPM_CONFIG_CACHE, PIP_CACHE_DIR, UV_CACHE_DIR, COMPOSER_CACHE_DIR, GOCACHE, NUGET_PACKAGES` | `data\cache\…` |
| Global packages | `NPM_CONFIG_PREFIX, PYTHONUSERBASE, COMPOSER_HOME, GOPATH, GRADLE_USER_HOME, CARGO_HOME` | `data\npm-global, data\python-user, data\composer, data\go, data\gradle…` |
| Languages | `JAVA_HOME, DOTNET_ROOT, PHPRC, RUSTUP_HOME…` | The folders of the kit's programs, at the versions in use |
| Certificates | `SSL_CERT_FILE, REQUESTS_CA_BUNDLE, PIP_CERT, CURL_CA_BUNDLE, GIT_SSL_CAINFO, CARGO_HTTP_CAINFO, NODE_EXTRA_CA_CERTS` | Only with a PEM file in Settings › Network: `data\app\ca-bundle.pem` (the system's certificates plus those of the file); Node.js gets the PEM file |
| No telemetry and no automatic updates | `DOTNET_CLI_TELEMETRY_OPTOUT, POWERSHELL_TELEMETRY_OPTOUT, GOTELEMETRY=off, DISABLE_UPDATES` | Off: the tools send no data and do not update themselves |
| PATH | `PATH` | First the kit folders, then only the system folders (System32 on Windows, /usr/bin and similar on macOS and Linux) |

To see the complete environment open a console in the kit folder and use the command `.\Atollo.exe env | Out-Host` (keys and passwords are masked with \*\*\*).

#### What stays on the PC

The kit does not write to the Windows registry, does not change the system PATH, and creates no services or scheduled tasks. The exceptions, declared:

- The system's temporary folder, in `Atollo\<kit id>`: the downloads' work area, files handed over manually, programs' temporary files. It is cleaned on closing and at the following starts.
- While Antigravity is open, its sign-in is in the system keychain (on Windows only for the session); on closing it goes back into the kit.
- An empty folder that PowerShell always creates in `%LOCALAPPDATA%\Microsoft\PowerShell`.
- If you run gh auth login manually in a console, GitHub CLI may save the sign-in in the Windows Credential Manager: sign in from Settings instead.
- On macOS: the window's data in ~/Library/WebKit and, if you do not use the token, the Claude Code sign-in in the Keychain.

> [!NOTE]
> **Administrator rights**
>
> Atollo always starts as a normal user. The only things that need an administrator are outside the kit and optional: Visual Studio Build Tools for Rust's MSVC compiler, the Xcode Command Line Tools on macOS for some C/C++ programs.

<details>
<summary>Under the hood</summary>

- The environment starts empty and copies about 40 system variables from the PC (ComSpec, SystemRoot, PROCESSOR_ARCHITECTURE, LANG, DISPLAY…) plus the proxy variables; the kit sets about a hundred. Atollo env on Windows prints about 130 lines.
- Every child program starts with the environment emptied and then filled with the kit's: nothing leaks in from the PC.
- The app's process too redirects its own user folder and temporary files, before starting anything.
- Variables containing KEY, TOKEN, SECRET or PASSWORD are masked in the env command and in the log.

</details>

Related topics: [8.2 The kit's folders on disk](#82-the-kits-folders-on-disk), [8.3 Background processes](#83-background-processes), [7.2 On another PC, another letter, another folder](#72-on-another-pc-another-letter-another-folder)

### 8.2 The kit's folders on disk

*projects and data: what is in each folder, which files are yours, which belong to the kit and which are protected.*

> **In short**
>
> - Everything is in the kit folder: the executable, projects (yours) and data (the kit's).
> - In data the programs are separated by system; the rest is shared between Windows, macOS and Linux.
> - The folders with passwords can be read only by your user (on NTFS drives).

#### The folders

- **Projects folder** — `projects\`
- **Shared agents** — `data\agents\`
- **Database data** — `data\db\`
- **Database backups** — `data\projects\<project>\private\backups\`

| Folder or file | What it contains |
| --- | --- |
| `<kit folder>\` | The folder that contains the executable, for example `D:\Atollo`. |
| &emsp;`Atollo.exe` | The only program: it is the app with the window and also the command line. On macOS Atollo.app, on Linux the AppImage. |
| &emsp;`projects\` | Your projects, one folder each. It is yours: the kit does not write there unless you ask. |
| &emsp;`data\` | Everything that belongs to the kit. |
| &emsp;&emsp;`app\` | The app's state: settings.json (Settings), `secrets\` (the Gemini key, protected), ca-bundle.pem (the certificates, when Settings has a PEM file), components.txt (chosen components), `versions\` (installed versions), `logs\`, services.json and servers.json (databases and web servers, with the ports of every version and every pool), terminals.json, frameworks.json, root.txt (last kit folder), safe-directory.txt (the last safe.directory the kit wrote). |
| &emsp;&emsp;&emsp;`app\db\` | Database administrator passwords and mysql logins. Protected: only your user can read it. |
| &emsp;&emsp;`apps\<system>\` | The programs, by system (win32-x64, darwin-arm64…), one folder per component and per version. |
| &emsp;&emsp;`projects\<project>\` | The kit's data for each project: dev-settings.json (versions, extensions, favorites), ports.json (the ports of its actions), Apache and Caddy configurations (your directives in `apache\conf.d\*.conf` and `caddy\conf.d\*.caddy` apply both to its own server and in a pool), `private\` (database passwords and backups, protected). |
| &emsp;&emsp;`db\<mysql\|mongodb>\<version>\` | The databases' data, one folder per version. |
| &emsp;&emsp;`servers\<pool>\` | The generated configurations of the shared web servers. |
| &emsp;&emsp;`agents\` | What the three agents share: AGENTS.md, `skills\`, mcp.json, and the sync state in `.sync\`. |
| &emsp;&emsp;`antigravity\` | The Antigravity sign-in kept in the kit (keyring.json). |
| &emsp;&emsp;`home\` | The “user folder” of the kit's programs: .claude, .codex, .gemini (agent profiles), .gitconfig, .npmrc, AppData (on Windows). |
| &emsp;&emsp;`gh\` | The GitHub CLI sign-in (hosts.yml). |
| &emsp;&emsp;`vscodium-server\` | Settings and extensions of the editor server. |
| &emsp;&emsp;`kit\` | Files the kit rewrites at every start: the PowerShell profile, helper commands (agy, mysql, gradle). |
| &emsp;&emsp;`cache\` | Cache of the package managers (npm, pip, uv, Composer, Go, NuGet…) and of GitHub's responses. It can be deleted. |
| &emsp;&emsp;`python-user\, npm-global\, composer\, go\, gradle\…` | Packages and tools installed globally with pip, npm, Composer, Go and the others. |
| &emsp;&emsp;`tmp\` | Temporary files, when Settings wants them in the kit. |
| &emsp;&emsp;`webview2\` | The data of the app window. |

#### What is yours and what is the kit's

| You can edit by hand | Better left to the kit |
| --- | --- |
| Everything in projects; `data\agents` (instructions, skills, mcp.json); `data\home\.gitconfig` | `data\app`, `data\apps`, `data\kit`, `data\servers`, `data\db`: the kit manages them, and some are rewritten at every start |

> [!WARNING]
> **Watch out**
>
> Do not delete the database folders (`data\db`) or the projects' private folders by hand: you would lose data and passwords. To remove a component use the Tools page; to clean up a project's data use “Data without a project”.

<details>
<summary>Under the hood</summary>

- Protected folders (only your user, on NTFS; permissions 700 on macOS and Linux): `data\app\db`, `data\app\secrets`, `data\projects\<project>\private`, `data\agents\claude`, `data\tmp` when it is Claude's temporary folder.
- On exFAT and FAT32 permissions do not exist: the kit works, but those folders are not protected.
- Configuration files are written safely: first a temporary file, then the replacement, so an interruption does not leave half-written files.
- `data\kit` is rewritten at every start: do not put your own files there.

</details>

Related topics: [8.1 Portable and isolated: what it means](#81-portable-and-isolated-what-it-means), [7.3 Updates and backups](#73-updates-and-backups), [2.7 Deleting a project and the “Data without a project”](#27-deleting-a-project-and-the-data-without-a-project)

### 8.3 Background processes

*What starts with the app, what stops when you close it, and why no programs stay running without your knowledge.*

> **In short**
>
> - When you open Atollo only the things you asked for start: the databases and servers with “Start with Atollo”.
> - Every program started by the kit is tied to the app: if the app closes, even in the worst way, they close with it.
> - Only one window per kit folder: opening it again brings the existing one to the front.

#### At start

1. **Folders** — Creates projects and data if they are missing and checks that it can write.
2. **Place** — Notices if the kit has been moved and takes note.
3. **Environment** — Reads Settings and builds the kit's environment.
4. **Window** — Only one per folder; then it opens the window.
5. **Background** — Starts what is marked “Start with Atollo”, syncs the agents and Git, checks for updates.

- Database servers and web server profiles with “Start with Atollo” start right after opening.
- Missing required components and those to reinstall are installed automatically; not at the first start: there everything is downloaded only with “Install” on the “Welcome” page.
- The agents' instructions, skills and MCP servers are synced; if the kit has been moved, the paths are updated.
- Git follows the kit folder (safe.directory) and the GitHub account: if “Who signs the commits” is empty, the name and private address come from the account.
- After 10 seconds, the versions each source offers, for the menus that download one (at most once a day).
- After 20 seconds, if turned on, the check for Atollo updates (turning “Automatic update” off before then stops it).

> [!NOTE]
> **Good to know**
>
> Opening the Databases page does not start the databases: they start with “Start”, with “Start with Atollo” or when a project needs them (to create its database or make a backup).

#### While you work

| What | When it starts | When it stops |
| --- | --- | --- |
| Project actions | With “Start” or ▶ | With “Stop”, or when the app closes |
| Shared web servers | With the first project that uses them, from the Web Servers page or with “Start with Atollo” | With the last project, unless started from the Web Servers page or with “Keep running even without projects”; with “Stop”; on closing |
| Databases | With “Start”, “Start with Atollo” or when a project asks for them | With “Stop”, or on closing (orderly shutdown) |
| Editor | With the first “Editor” | When the app closes |
| Consoles and agents | With “Terminal”, “Open”, the agent logos | When you close them, or when the app closes |

Closing a console does not shut down the databases: they stay on until you stop them or close the app. The Activity panel (from the status bar) lists the project actions, every running database version and the active pools; “Stop all” stops them after a confirmation.

#### On closing

If something is open, Atollo asks “Close Atollo?” with the list; “Close everything” stops the actions and the web servers and shuts down the databases neatly, then exits. Before unplugging the drive use [7.1 Safely removing the drive](#71-safely-removing-the-drive).

<details>
<summary>Under the hood</summary>

- On Windows all the kit's programs live in a Job Object with the “kill on close” option: if the app ends in any way, Windows closes them too. The editor and actions have their own Job, inside the app's.
- On macOS and Linux a small watchdog process notices if the app dies and closes the kit's process groups (SIGTERM, then SIGKILL after 5 seconds).
- The single instance uses an identifier derived from the kit folder (a mutex on Windows, a lock on macOS and Linux): two different kits do not get in each other's way.
- Every port has only one owner: a pool (by default 8081 to 8999), a database version, a project action (by default 7100 to 7999); 7000 to 7099 belong to the editor. A port already taken is refused with the name of whoever holds it (for example “Port 3306 already belongs to database MySQL 8.4: choose another one.”), and a new pool's automatic port avoids them all.
- Every version of a database keeps its own port, also when you change the server's version; two starts of the same version run only one server, on the same port.
- The browser and File Explorer are opened outside the Job: they stay open even after Atollo closes.
- On closing, if an update is ready, the executable is replaced.

</details>

Related topics: [7.1 Safely removing the drive](#71-safely-removing-the-drive), [8.1 Portable and isolated: what it means](#81-portable-and-isolated-what-it-means), [3.1 The database servers](#31-the-database-servers)

### 8.4 Where the tools come from

*The official sources of each component, the cryptographic checks, downloading by hand and the full table of sources.*

> **In short**
>
> - Every component comes from its maker's official source, over HTTPS: no mirrors, no third-party copies.
> - Every file is verified with its cryptographic hash before it is used; if it does not match, it is discarded.
> - If a download fails, you can download the file with the browser and hand it to the kit, which verifies it the same way.

#### The path of a download

1. **Request** — Which is the latest version and what its hash is, from the official source.
2. **Download** — Into a .part file in the work folder; three attempts.
3. **Check** — Computes the file's hash and compares it with the expected one.
4. **Extract** — zip, tar.gz, tar.xz, 7z or single file.
5. **Replace** — Puts the new folder in place of the old one, all at once.
6. **Test** — Runs the program to check that it responds.

| Check | What it means | Examples |
| --- | --- | --- |
| SHA-256, SHA-512, SHA3-256 | The file's hash must be the one published by the maker (or pinned in the kit's catalog). | Python, Node.js, Go, Maven, Caddy, SQLite, the editor |
| npm sha512 | The integrity hash of the npm registry. | Claude Code, Codex |
| Authenticode (Windows only) | In addition, the maker's digital signature on the program, with the revocation check. | Claude Code, Visual C++ runtime |

> [!TIP]
> **Tip**
>
> Every part of a component (for example PHP and its Composer) is downloaded and verified before touching the installed one: if something goes wrong, the version you had stays as it was.

#### When a download fails: “manually”

If the download fails after three attempts (a blocking proxy, an unreachable site), a panel appears under the component's row:

1. Click “Download it in the browser: \<file name\>”: the exact address opens in the browser.
2. When the file has downloaded, drag it onto the panel or use “browse…”.
3. The kit keeps it and uses it only if its hash is the published one; the installation restarts by itself (“Added …: trying again.”).

The panel appears only when the file's hash is known. Files handed over and not used are deleted after 30 days.

#### GitHub and the API limits

Many components are published on GitHub. Without a sign-in, the GitHub API allows 60 requests an hour; with the kit's account ([6.1 The kit's GitHub account](#61-the-kits-github-account)) or with a GITHUB_TOKEN variable on the PC, 5,000. The kit keeps the responses and, if the limit is reached, reads the latest version straight from the GitHub pages.

#### Other services the app contacts

| Service | Why |
| --- | --- |
| `api.github.com/repos/nabre/atollo/releases` | Atollo's updates. |
| `open-vsx.org` | The editor's extensions. |
| `api.anthropic.com · chatgpt.com · cloudcode-pa.googleapis.com` | The limits of the agents' plans, with the saved sign-in, read-only and only with the Agents page open. |
| `registry.npmjs.org · pypi.org · packagist.org…` | Only when the commands of projects and frameworks ask for it. |

#### Complete table of sources (36 components and 5 parts)

For each component: where the kit reads the versions from, where it downloads the file from, how it verifies it and where it puts it. The columns describe Windows; where macOS and Linux use another source, it is written under the address.

| Component | Versions from | Files from | Check | In `data\apps\<system>\` |
| --- | --- | --- | --- | --- |
| [PowerShell 7 (pwsh)](https://github.com/PowerShell/PowerShell) | api.github.com/repos/PowerShell/PowerShell/releases/latest | PowerShell-\<v\>-win-(x64\|arm64).zip | SHA-256 — GitHub Release asset.digest | `pwsh\<line>\pwsh.exe` |
| [GitHub CLI (gh)](https://cli.github.com) | api.github.com/repos/cli/cli/releases/latest | gh_\<v\>_windows_(amd64\|arm64).zip | SHA-256 — GitHub Release asset.digest | `gh\<line>\bin\gh.exe` |
| [Editor (VSCodium Reh Web)](https://vscodium.com) | Built-in catalog (pinned version 1.135.06055, D-58) | github.com/VSCodium/vscodium/releases/download/\<v\>/vscodium-reh-web-win32-x64-\<v\>.tar.gz<br>macOS and Linux: vscodium-reh-web-\<target\>-\<v\>.tar.gz, SHA-256 pinned per target (native on ARM64 too) | SHA-256 — SHA-256 hardcoded in components.json | `vscodium-server\` |
| [Claude Code](https://claude.com/product/claude-code) | registry.npmjs.org/@anthropic-ai/claude-code-win32-\<arch\>/latest | dist.tarball field of the npm package | Authenticode + SHA — npm sha512 integrity + Anthropic Authenticode signature | `claude\<line>\claude.exe` |
| [Codex (OpenAI)](https://github.com/openai/codex) | registry.npmjs.org/@openai/codex/latest → /\<ver\>-win32-\<arch\> | dist.tarball field of the architecture-specific package | npm sha512 — npm dist.integrity (sha512-...) | `codex\<line>\bin\codex.exe` |
| [Antigravity (Google)](https://antigravity.google) | antigravity-cli-auto-updater-974169037036.us-central1.run.app/manifests/windows_(amd64\|arm64).json | url field of the official JSON manifest<br>macOS and Linux: manifests/\<os\>_\<arch\>.json, .tar.gz archive | SHA-512 — SHA-512 (128 hex) from the manifest sha512 field | `antigravity\<line>\agy.exe (via data\kit\bin\agy.cmd)` |
| [Git + Git Bash (PortableGit)](https://git-scm.com) | api.github.com/repos/git-for-windows/git/releases/latest | PortableGit-\<v\>-(64-bit\|arm64).7z.exe (extracted in Rust)<br>macOS and Linux: GitHub desktop/dugite-native (dugite-native-v\<v\>-\<sha\>-\<system\>.tar.gz, the asset's .sha256), follows the kit | SHA-256 — GitHub Release asset.digest | `git\<line>\cmd\git.exe and git\<line>\bin\bash.exe` |
| [Python + pip](https://www.python.org) | raw.githubusercontent.com/astral-sh/python-build-standalone/latest-release/latest-release.json | cpython-3.\<minor\>.\<patch\>+\<date\>-\<triple\>-install_only_stripped.tar.gz | SHA-256 — SHA-256 from the official SHA256SUMS list | `python\<line>\python.exe (+ pip)` |
| [Node.js + npm](https://nodejs.org) | nodejs.org/dist/index.json (even LTS \>= 18 + latest major) | nodejs.org/dist/\<v\>/node-\<v\>-win-\<arch\>.zip | SHA-256 — SHA-256 from nodejs.org/dist/\<v\>/SHASUMS256.txt | `node\<line>\node.exe` |
| [uv (uv, uvx, uvw)](https://docs.astral.sh/uv/) | api.github.com/repos/astral-sh/uv/releases/latest | uv-\<triple\>.zip | SHA-256 — GitHub Release asset.digest | `uv\<line>\uv.exe, uvx.exe, uvw.exe` |
| [PHP (Thread Safe)](https://www.php.net) | windows.php.net/downloads/releases/releases.json (+ EOL archives 8.1..5.6) | windows.php.net/downloads/releases/php-\<v\>-Win32-vs\*-x64.zip<br>macOS and Linux: static-php-cli “bulk”, cli and fpm archives with SHA-256 pinned in the catalog (8.4 and 8.5) | SHA-256 — SHA-256 from releases.json (or verified EOL table) | `php\<line>\php.exe` |
| [↳ Part: Composer](https://getcomposer.org) | getcomposer.org/versions (stable channel or 2.2 LTS for PHP \< 7.2.5) | getcomposer.org/download/\<v\>/composer.phar | SHA-256 — SHA-256 from \<url\>.sha256sum | `php\<line>\composer.phar (+ composer.cmd)` |
| [↳ Part: SSL CA Bundle (curl)](https://curl.se) | curl.se/ca/cacert.pem.sha256 | curl.se/ca/cacert.pem | SHA-256 — SHA-256 from cacert.pem.sha256 | `php\<line>\extras\ssl\cacert.pem` |
| [↳ Part: Visual C++ Runtime](https://learn.microsoft.com) | Extracted from CAB `vcRuntimeMinimum_amd64\cab1.cab` if missing | aka.ms/vc14/vc_redist.x64.exe<br>Windows only | Authenticode + SHA — Authenticode signature O=Microsoft Corporation on exe and vcruntime140.dll | `tools\vcruntime\ (copied into PHP, Apache, MongoDB)` |
| [Apache httpd](https://httpd.apache.org) | apachelounge.com/download/ (version and checksum scraping) | apachelounge.com/download/VS\<nn\>/binaries/httpd-\<v\>-Win64-VS\<nn\>.zip<br>macOS and Linux: the kit's build, github.com/nabre/atollo/releases/download/httpd-v\<v\>/httpd-\<v\>-\<target\>.tar.gz, SHA-256 pinned per target in the catalog, PHP through php-fpm (D-116) | SHA-256 — SHA-256 from \<url\>.txt (SHA256-Checksum for:) | `apache\<line>\bin\httpd.exe` |
| [Caddy](https://caddyserver.com) | api.github.com/repos/caddyserver/caddy/releases?per_page=100 | caddy_\<v\>_windows_amd64.zip asset from the official GitHub release | SHA-256 — GitHub Release asset.digest (from 2.10.1), otherwise SHA-512 from caddy_\<v\>_checksums.txt | `caddy\<line>\caddy.exe` |
| [Rust (rustup, cargo, clippy)](https://www.rust-lang.org) | static.rust-lang.org/dist/channel-rust-stable.toml (\[pkg.rustc\]) | static.rust-lang.org/rustup/dist/\<triple\>/rustup-init.exe<br>Host gnu on Windows x64, gnullvm on Windows ARM64 (with C/C++), msvc from Rust's page in Tools (“Details”); macOS and Linux: rustup with the native host | SHA-256 — SHA-256 from \<url\>.sha256 | `rust\cargo\bin\rustc.exe and rustup.exe` |
| [Go SDK](https://go.dev) | go.dev/dl/?mode=json (&include=all) | go.dev/dl/go\<v\>.windows-(amd64\|arm64).zip | SHA-256 — SHA-256 from the sha256 field of the official JSON | `go\<line>\bin\go.exe` |
| [Java JDK (Eclipse Temurin)](https://adoptium.net) | api.adoptium.net/v3/info/available_releases + /v3/assets/latest/\<major\>/hotspot | .zip package returned by the Adoptium API | SHA-256 — SHA-256 from binary.package.checksum | `java\<line>\bin\java.exe` |
| [↳ Part: Apache Maven](https://maven.apache.org) | repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml | apache-maven-\<v\>-bin.zip | SHA-512 — SHA-512 from \<url\>.sha512 | `maven\bin\mvn.cmd` |
| [↳ Part: Gradle](https://gradle.org) | services.gradle.org/versions/current | downloadUrl field of the official JSON | SHA-256 — SHA-256 from checksumUrl of the JSON | `gradle\bin\gradle.bat (via data\kit\bin\gradle.cmd)` |
| [Kotlin Compiler](https://kotlinlang.org) | api.github.com/repos/JetBrains/kotlin/releases?per_page=100 | Asset kotlin-compiler-\<v\>.zip | SHA-256 — GitHub asset.digest or \<asset\>.sha256 | `kotlin\<line>\bin\kotlinc.bat` |
| [Android SDK + Command-line Tools](https://developer.android.com/studio) | developer.android.com/studio (commandlinetools-win-\*_latest.zip table) | dl.google.com/android/repository/commandlinetools-win-\<build\>_latest.zip | SHA-256 — SHA-256 published in the official Android table | `android\cmdline-tools\latest\bin\sdkmanager.bat and adb.exe` |
| [.NET SDK](https://dotnet.microsoft.com) | builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json | .zip archive for rid=win-x64\|win-arm64 | SHA-512 — SHA-512 from the hash field of releases.json | `dotnet\<line>\dotnet.exe (with shared SDK junctions)` |
| [Dart SDK](https://dart.dev) | storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION | dartsdk-windows-\<arch\>-release.zip | SHA-256 — SHA-256 from \<url\>.sha256sum | `dart\<line>\bin\dart.exe` |
| [Zig](https://ziglang.org) | ziglang.org/download/index.json | tarball field for (x86_64\|aarch64)-windows | SHA-256 — SHA-256 from the shasum field of the JSON | `zig\<line>\zig.exe` |
| [Julia](https://julialang.org) | julialang-s3.julialang.org/bin/versions.json | .zip archive for os=winnt, arch=x86_64 | SHA-256 — SHA-256 from the sha256 field of versions.json | `julia\<line>\bin\julia.exe` |
| [Bun](https://bun.sh) | api.github.com/repos/oven-sh/bun/releases | Asset bun-windows-(x64\|aarch64).zip | SHA-256 — GitHub asset.digest or SHASUMS256.txt | `bun\<line>\bun.exe` |
| [Deno](https://deno.com) | api.github.com/repos/denoland/deno/releases | Asset deno-(x86_64\|aarch64)-pc-windows-msvc.zip | SHA-256 — GitHub asset.digest or \<asset\>.sha256sum | `deno\<line>\deno.exe` |
| [Typst](https://typst.app) | api.github.com/repos/typst/typst/releases | Asset typst-(x86_64\|aarch64)-pc-windows-msvc.zip | SHA-256 — GitHub Release asset.digest | `typst\<line>\typst.exe` |
| [TinyTeX (LaTeX)](https://yihui.org/tinytex/) | api.github.com/repos/rstudio/tinytex-releases/releases/latest | Asset TinyTeX-1-windows-v\*.exe (7z SFX extracted in Rust)<br>macOS and Linux: TinyTeX-1-\<os\>-\<arch\>-v\*.tar.xz (universal on macOS) | SHA-256 — GitHub Release asset.digest | `tinytex\bin\windows\tlmgr.bat and pdftex.exe` |
| [pnpm + yarn (corepack)](https://pnpm.io) | registry.npmjs.org/pnpm/latest and /corepack/latest | Installed via npm-cli.js of the kit's Node.js | npm sha512 — npm registry integrity | `data\npm-global\pnpm.cmd and yarn.cmd` |
| [CMake + Ninja](https://cmake.org) | GitHub API: Kitware/CMake and ninja-build/ninja (releases/latest) | cmake-\*-windows-(x86_64\|arm64).zip and ninja-win(arm64).zip | SHA-256 — GitHub Release asset.digest | `cmake\<line>\bin\cmake.exe and tools\ninja.exe` |
| [C/C++ (llvm-mingw)](https://github.com/mstorsjo/llvm-mingw) | api.github.com/repos/mstorsjo/llvm-mingw/releases/latest | Asset llvm-mingw-\*-ucrt-(x86_64\|aarch64).zip<br>macOS and Linux: cc, c++, gcc, g++, clang shims over the kit's zig cc (needs Zig) and GNU make from conda-forge, not relocatable | SHA-256 — GitHub Release asset.digest | `cc\bin\clang.exe (+ make.cmd)` |
| [Ruby + Bundler](https://www.ruby-lang.org) | api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=30 | Asset rubyinstaller-\<v\>-(x64\|arm).7z<br>macOS and Linux: conda-forge ruby (3.2 and later) via micromamba, not relocatable | SHA-256 — GitHub Release asset.digest | `ruby\<line>\bin\ruby.exe` |
| [R (conda-forge + OpenBLAS)](https://www.r-project.org) | api.anaconda.org/package/conda-forge/r-base + mamba-org/micromamba-releases | micromamba-win-64 + r-base and libblas=\*=\*openblas packages<br>Windows ARM64: the win-64 packages under x64 emulation, from the native micromamba (--platform win-64). macOS and Linux: the same packages for osx-\* and linux-\*, launchers in kit-bin, not relocatable | SHA-256 — micromamba SHA-256 + conda-forge package SHA-256 | `r\<line>\bin\R.cmd and Rscript.cmd` |
| [MySQL Community Server](https://www.mysql.com) | 8.4 and 9.7 LTS lines pinned in the catalog (sha256ByLine) | cdn.mysql.com/Downloads/MySQL-\<line\>/mysql-\<v\>-winx64.zip | SHA-256 — SHA-256 pinned per line and target after Oracle's GPG signature | `mysql\<line>\bin\mysqld.exe and mysql.exe` |
| [MongoDB + mongosh + Database Tools](https://www.mongodb.com) | downloads.mongodb.org/current.json, GitHub mongodb-js/mongosh, tools/db/release.json | Windows x64 .zip archives of mongod, mongosh and mongodump/mongorestore<br>macOS: vendor archives; Linux: server from conda-forge (not relocatable), generic mongosh, Database Tools per distribution | SHA-256 — SHA-256 from MongoDB JSONs + GitHub asset.digest for mongosh | `mongodb\<line>\bin\mongod.exe, mongosh\, mongodb-tools\` |
| [phpMyAdmin (for MySQL)](https://www.phpmyadmin.net) | www.phpmyadmin.net/home_page/version.json | files.phpmyadmin.net/phpMyAdmin/\<v\>/phpMyAdmin-\<v\>-all-languages.zip | SHA-256 — SHA-256 from the official .sha256 file + php -l syntax check | `phpmyadmin\<line>\index.php` |
| [mongo-express (for MongoDB)](https://github.com/mongo-express/mongo-express) | version pinned in Atollo's catalog (the kit's build, D-147) | github.com/nabre/atollo/releases/download/mongo-express-v\<v\>/mongo-express-\<v\>.zip | SHA-256 — SHA-256 pinned in the catalog (or the .sha256 of the kit's release); built in CI from mongo-express's pinned commit | `mongo-express\<line>\app.js` |
| [SQLite CLI](https://sqlite.org) | sqlite.org/download.html (PRODUCT,... CSV rows) | sqlite.org/\<year\>/sqlite-tools-win-(x64\|arm64)-\<v\>.zip<br>Linux ARM64: conda-forge sqlite (sqlite.org publishes no tools), not relocatable; Windows ARM64: the x64 zip when there is no arm64 one | SHA3-256 — SHA3-256 published in the PRODUCT row of the official page | `sqlite\<line>\sqlite3.exe` |

<details>
<summary>Under the hood</summary>

- Work folder: `<system temp>\Atollo\<kit id>\setup`, or `data\tmp\setup` if the system disk has less than 4 GB free or cannot be written to.
- Three attempts 5 and 10 seconds apart; the download starts over at every attempt.
- Connections use the proxy of Settings and the system's certificates plus those of “Additional certificates (PEM file)”: a firewall's HTTPS inspection works if its certificate is installed on the PC or is in that file.
- The installed version is written in `data\app\versions\<system>\<component>.txt`.

</details>

Related topics: [5.1 The Tools page](#51-the-tools-page), [6.3 Company network: proxy, certificates, local network](#63-company-network-proxy-certificates-local-network), [6.1 The kit's GitHub account](#61-the-kits-github-account)

### 8.5 The command line

*The commands of the Atollo executable to prepare, check and sync the kit without opening the window.*

> **In short**
>
> - Atollo's executable is also a command-line program: given a command, it does the job and exits, without opening the window.
> - Useful to prepare a kit for another system, check the components, or see the environment.
> - It works even while the app is open.

#### How to use it

Open a console in the kit folder. On Windows, in PowerShell, add | Out-Host so the output appears in the console (the executable is a windowed program); in Command Prompt it is not needed. On macOS and Linux use the program's path (for example ./Atollo.AppImage).

**Example**

```
cd D:\Atollo
.\Atollo.exe components | Out-Host
.\Atollo.exe prepare --only node,php --line php=8.3 | Out-Host
.\Atollo.exe check | Out-Host
```

#### The commands

| Command | What it does |
| --- | --- |
| `env [--project <name>]` | Prints the kit's environment (secrets masked); with a project, with its versions. |
| `components [--target <system>]` | Lists the components: chosen or not, group, installed version or why it is not available. |
| `versions [<id>]` | Without id: the versions on disk of the installed tools. With an id: asks the source for the versions offered and saves them. |
| `check` | Asks every installed component for its version and marks ✓ or ✗. |
| `prepare [options]` | Installs, updates and removes components according to the choice. The options are in the table below. |
| `sync [--project <folder>] [--verbose]` | Syncs the agents' instructions, skills and MCP servers. |
| `serve <folder> <port>` | Serves a folder as a static site (the kit's file server). |
| `keyring restore \| save` | Used by the kit around Antigravity: restores or saves its sign-in. |
| `version · help` | The version (also --version, -V) and the help (also --help, -h, /?). |

| prepare option | Effect |
| --- | --- |
| `--only a,b` | Only these components (plus what they need), without removing the others. |
| `--skip a,b` | All except these (the required ones cannot be skipped). |
| `--update-all` | Also updates what is already installed. |
| `--force` | Reinstalls even if the version is already that one. |
| `--line id=x.y` | Adds that version next to the others (can be repeated). |
| `--target <system>` | Prepares for another system: win32-x64, win32-arm64, darwin-arm64, darwin-x64, linux-x64, linux-arm64. |

> [!WARNING]
> **Watch out**
>
> Without --only, prepare removes the components not chosen, from the other systems' folders too. If you are not sure, look at components first.

<details>
<summary>Under the hood</summary>

| Exit code | Meaning |
| --- | --- |
| `0` | All good. |
| `1` | prepare or check: something failed. |
| `2` | Unknown command (the help is printed too). |
| `3` | prepare: another preparation of the same kit is already running. |

From the command line prepare uses the console's GITHUB_TOKEN or GH_TOKEN, if present, to raise the GitHub API limit; it does not use the kit's account. If a download fails it prints the address to download by hand and the folder to put it in.

</details>

Related topics: [5.1 The Tools page](#51-the-tools-page), [7.2 On another PC, another letter, another folder](#72-on-another-pc-another-letter-another-folder), [8.1 Portable and isolated: what it means](#81-portable-and-isolated-what-it-means)

## 9. Help

### 9.1 When something goes wrong

*The most common problems, the message you see and what to do: downloads, ports in use, servers that won't start, expired sign-ins.*

> **In short**
>
> - Search for the message you see in the box below: the guide collects the most common ones, with the remedy.
> - Almost every error has a tooltip on mouse-over and a line of output or log that tells you more.
> - The app's log is in `data\app\logs\app.log`; the servers' logs are next to it.

#### Download and installation

**A component stays in “error” with “failed” and a network or certificate message.**

Often it is the network: a company proxy or HTTPS inspection. Fill in Settings › Network and try again. If that is not enough, use the “Download it in the browser” panel under the row and hand the file to the kit.

Learn more: [6.3 Company network: proxy, certificates, local network](#63-company-network-proxy-certificates-local-network)

**“in use: close the kit programs and try again”**

A kit program (a running project, a console, the editor, a database) is using the files to be replaced. Stop it, close the consoles and try again. If you do not know what is open, the “Remove drive” page lists it.

Learn more: [7.1 Safely removing the drive](#71-safely-removing-the-drive)

**“An installation is already running.” or “An installation is already in progress”**

Wait for it to finish (the status bar shows the step) or press “Cancel” in the “Apply” bar.

**A component is “to update” (yellow triangle).**

On macOS and Linux, after the kit is moved, some components must be reinstalled: the kit does it by itself at start. If it stays that way, press “Update all” at the bottom of the Tools page.

Learn more: [7.2 On another PC, another letter, another folder](#72-on-another-pc-another-letter-another-folder)

#### Projects and sites

**“Port already assigned to …”, “Port … already belongs to server/database/project …: choose another one.”, “Ports 7000 to 7099 belong to the editor” or a program saying the port is taken.**

A port belongs to one thing only: an action of a project, a pool of the Web Servers page, a version of a database or the editor (7000-7099). The message says who already has it: “Port already assigned to …” another action, “already belongs to server …” a pool, “already belongs to database …” a version of MySQL or MongoDB, “already belongs to project …” an action. It applies to the port of an action, of a pool and of a database version. If instead a program on the PC has the port, that program says so. For an action, empty the “Port” field to go back to the automatic port; otherwise choose another one between 1024 and 65535.

Learn more: [2.3 Starting a project: actions and ports](#23-starting-a-project-actions-and-ports)

**An action stops at once with “error (code 1)”.**

Read the last lines of the output: they usually say what is missing. Common cases: dependencies not installed (run npm install or composer install first), a missing component (“Components” section › “Install what is missing”), a wrong version (“Versions” section).

Learn more: [2.4 Components and versions of a project](#24-components-and-versions-of-a-project)

**“Action not found: press Refresh”**

The project has changed (for example panel.json). Press “Refresh” on the project page.

**“panel.json is not valid JSON: ignored.”**

There is a syntax error in the file (an extra comma, a missing quote). Fix it in the editor and press “Refresh”.

Learn more: [2.3 Starting a project: actions and ports](#23-starting-a-project-actions-and-ports)

**“The shared server … did not start: see the Web Servers page.”**

Open the Web Servers page, choose the pool and look at the error. Often the port is taken: change the profile's port, or choose “Its own server” in the project's “Website” section.

Learn more: [3.3 Websites and .localhost addresses](#33-websites-and-localhost-addresses)

**The site opens but shows old files or a blank page.**

Check the folder in “Folder to serve as a website” (for many PHP frameworks it is public) and reload the browser page bypassing the cache (Ctrl+F5).

**The editor does not notice changed files.**

On Windows this happens if a folder in the kit's path starts with a digit, with x or with u: the app flags it with a warning. Move the kit to a different path.

Learn more: [1.2 The first start](#12-the-first-start)

#### Databases

**“Port … is already used by another program: change it below.”**

Another MySQL or MongoDB (installed separately) runs on the PC on the same port. Change the “Port” on the server page (for example 3307) and start it again; then rewrite the connection in the projects' .env with “Write to .env”.

Learn more: [3.1 The database servers](#31-the-database-servers)

**The server stays in “Error” after starting.**

The server page shows the last error and the path of the log (`data\app\logs`). On Linux MySQL may ask for the libaio and libnuma libraries: the message gives the command to install them.

**The project does not connect to the database.**

Check that the server is “Running”, then use “Write to .env” in the project's “Databases” section: the .env will have the right user, password and port. If the project uses another version of the server, the port is different (3307…).

Learn more: [3.2 A project's database](#32-a-projects-database)

#### Agents, Git and GitHub

**“sign-in expired” in the agents.**

Open the agent in a project: it renews the sign-in, or has you sign in again.

Learn more: [4.1 The agents: Claude Code, Codex, Antigravity](#41-the-agents-claude-code-codex-antigravity)

**Git says “Please tell me who you are” at the first commit.**

Type your name and email in Settings › “GitHub account” › “Who signs the commits”, or sign in to GitHub: if they are empty, the account fills them. In a kit terminal git config --global user.name and user.email work too.

Learn more: [6.1 The kit's GitHub account](#61-the-kits-github-account)

**“GitHub refused the sign-in: sign in again.”**

The token is no longer valid (revoked or expired). In Settings › “GitHub account” press “Sign out” and then “Sign in to GitHub”.

Learn more: [6.1 The kit's GitHub account](#61-the-kits-github-account)

#### App start

**“The kit's disk is read-only: cannot write to the folder …”**

The USB stick is write-protected (sometimes there is a switch) or mounted read-only. The kit must be able to write in its folder.

**macOS: “macOS opened Atollo from a temporary read-only copy (App Translocation)”.**

It happens with downloaded apps that are not notarized. The message shows an xattr command: run it in Terminal and reopen Atollo.

Learn more: [7.2 On another PC, another letter, another folder](#72-on-another-pc-another-letter-another-folder)

**Linux: the disk is mounted with noexec, or WebKitGTK is missing.**

The message explains what to do: remount the disk with execute permission, or install the packages it names.

**Windows: WebView2 is missing.**

WebView2 is the Windows component that draws the window; it is already in Windows 11 and almost always in Windows 10. If it is missing, Atollo shows the link to install it.

**Warning: “Programs from another copy of the kit are open on this PC”.**

Just information: another kit is open. The two copies share nothing.

#### If the problem is not here

- Read the action's output (“Run” section) or the server's log: the last line usually gives the cause.
- Close Atollo and open it again: many things (environment, agents, kit files) are regenerated at every start.
- Check the components with “Verify” on the Tools page, or with `.\Atollo.exe check | Out-Host`.
- Report the problem in the issues of the GitHub repository nabre/atollo, attaching the log lines (passwords are already masked).

Related topics: [9.2 Frequently asked questions](#92-frequently-asked-questions), [6.3 Company network: proxy, certificates, local network](#63-company-network-proxy-certificates-local-network), [8.3 Background processes](#83-background-processes)

### 9.2 Frequently asked questions

*Short answers to the questions that come up most often.*

> **In short**
>
> - Short answers; each one points to the topic that explains more.

#### Frequently asked questions

**Does it work without the Internet?**

Yes, for everything already installed: projects, databases, sites, editor, consoles. The Internet is needed for component downloads, for commands that download packages (npm, Composer, pip) and for the AI agents, which talk to the provider's servers.

**How much does it cost?**

Atollo is free and open source (MIT license), like the components it downloads. The AI agents require an account or an API key from the provider, which may be paid.

**Does my code go anywhere?**

The kit sends nothing: no telemetry, and the tools have telemetry turned off. When you use an AI agent, however, the code you have it read is sent to the model's provider (Anthropic, OpenAI, Google), under its terms. The summary is in Settings › “Privacy”, with the link “Read the privacy policy”.

**How do I uninstall Atollo?**

Close the app and delete the kit folder (after putting the projects you need somewhere safe). There are no registry entries or installed programs to remove. The system's temporary folder is cleaned on closing.

**Can I keep projects outside projects?**

The Projects page shows only the folders inside projects. To work with the kit's tools on any folder of the PC use the Terminals page ([3.4 The kit's terminals](#34-the-kits-terminals)), which opens consoles anywhere.

**Can I use another editor?**

Projects are normal folders: you can open them with any editor. But the kit's tools (Node, PHP…) and the project's versions are ready only in the kit's consoles and editor, because it is the kit that prepares their environment.

**Does Atollo replace Docker?**

No, and it does not include it. It uses native programs for each system, lighter and without an administrator. If a project has a compose.yaml or a Dockerfile, the kit reads them only to suggest the right versions (PHP, Node, MySQL…).

**Can I have two kits?**

Yes: each folder is an independent kit, with its own projects, sign-ins and settings, and two kits can be open together. The same kit, however, opens only once.

**Does it work on a network drive?**

Yes, but it is slower: the programs read many small files. With “Temporary files of programs” set to “Automatic”, temporary files go to the PC's disk anyway.

**How much space does it need?**

The initial selection takes about 1.8 GB. Each language adds from a few tens to a few hundred MB (the Android SDK and TinyTeX more), each extra version as much again. Projects with their dependencies (node_modules, vendor) grow over time. The Tools page shows the size of each component.

**Do the tools update themselves?**

Atollo does (if you leave automatic update on); the components do not, you update them from the Tools page. See [7.3 Updates and backups](#73-updates-and-backups).

**Why doesn't the Agents page open the agents?**

Because an agent always works in a project: you open it from the Projects page, with the button that shows the agent's logo. The Agents page is for sign-in, limits and the shared folder.

> [!TIP]
> **Tip**
>
> Can't find the answer? Try [9.1 When something goes wrong](#91-when-something-goes-wrong) or the guide's search box.

Related topics: [9.1 When something goes wrong](#91-when-something-goes-wrong), [9.3 Glossary](#93-glossary), [1.1 Welcome to Atollo](#11-welcome-to-atollo)

### 9.3 Glossary

*The words of the guide explained simply, from A to Z.*

> **In short**
>
> - The words that come up in the guide and the app, explained without taking anything for granted.

#### From A to Z

- **.env** — A text file in the project with settings and secrets (like the database password), one KEY=value line each. It must not go into Git.
- **Action** — A named project command, started and stopped from the “Run” section.
- **Addition** — In the “From a framework or CMS” wizard, an extra piece to add to the base (Tailwind, React, a database, tests).
- **AI agent** — A command-line assistant that works in the project folder: it reads, edits and runs commands. In the kit: Claude Code, Codex, Antigravity.
- **API key** — A secret code that lets you use a service (for example an AI model) paying as you go, without a subscription.
- **Backup** — A file with the whole content of a database, to put it back as it was.
- **Base** — The starting point of a project created with a framework or a CMS: Laravel, Vite, Django, WordPress…
- **Clone** — Copy a Git repository to your computer, with all its history.
- **Commit** — A save in Git's history: a set of changes with a message.
- **Component** — A program the kit can install: a language, an agent, a database, a tool.
- **Console (terminal)** — A window where you type commands. The kit's one is PowerShell 7 with the kit's tools; on macOS and Linux, if PowerShell is not installed, it is bash.
- **exFAT, FAT32, NTFS** — Disk formats. NTFS (Windows) lets you protect folders; exFAT is read by Windows, macOS and Linux; FAT32 does not accept files over 4 GB.
- **Extension** — An add-on for the editor that makes it able to understand a language or do new things.
- **Framework** — A ready-made structure for building a type of application (Laravel for PHP sites, Django for Python…).
- **Git** — The program that keeps the history of a project's changes.
- **GitHub, GitHub CLI** — GitHub is the online service that hosts Git repositories; GitHub CLI (gh) is its command-line program.
- **Hash** — A code computed from a file's content: if the file changes by even one byte, the hash changes. It is used to verify that a download is the right one.
- **Kit folder** — The folder that contains the Atollo executable, projects and data.
- **localhost, 127.0.0.1** — The name and address a computer uses to call itself. A name ending in .localhost always leads to this PC.
- **MCP (MCP server)** — Model Context Protocol: a standard way to give agents new tools, through a program or a service.
- **Minor version** — The first two parts of a version: for PHP 8.4.3 the minor version is 8.4. The kit keeps one folder per minor version.
- **Open VSX** — The free extension catalog used by VSCodium.
- **PATH** — The list of folders where the system looks for programs when you type a command.
- **Path alias** — An address of the site that serves a different folder, for example /media → storage/uploads.
- **Pool** — A web server (Caddy or Apache) shared by several projects, each with its own .localhost name and its own conf.d folder.
- **Port** — A number that tells services apart on the same computer: http://localhost:7100 and http://localhost:7101 are two different sites. In the kit each port belongs to one thing only (an action, a pool, a version of a database); 7000-7099 belong to the editor.
- **Project** — A folder inside projects. The kit recognizes its type from the files.
- **Proxy** — An intermediate server that Internet traffic goes through, common in company networks.
- **Push, pull** — Sending commits to the remote repository (push) and receiving them (pull).
- **Repository** — A project managed with Git, with its history. It can live on your computer and on GitHub.
- **Server profile** — A pool you create on the Web Servers page, with the versions and port you choose.
- **Skill** — A written procedure an agent loads when needed, in a folder with a SKILL.md file.
- **Token** — A secret code that proves a sign-in already done (GitHub, Claude). For AI models, “token” is also the unit used to count text and usage.
- **VirtualHost** — A site inside a web server, recognized by the name you call it with.
- **VSCodium** — The free version of Visual Studio Code: the kit's editor.

Related topics: [1.1 Welcome to Atollo](#11-welcome-to-atollo), [9.2 Frequently asked questions](#92-frequently-asked-questions)

The same guide is in the app, on the Guide page, in four languages.
