# Testing

English · [Italiano](TESTING.it.md)

The automated tests run in CI (`.github/workflows/app.yml`) and locally, from the repository root:

```
npm run lint
npm test
npm -w ui run build
```

`npm run lint` checks `cargo fmt`, `cargo clippy -D warnings`, ESLint and Prettier (on the tests too, with the types from `test\ui\tsconfig.json`); `npm test` runs `cargo test` and the interface tests (Vitest); `npm -w ui run build` checks the types (`tsc -b`) and builds the interface.

All the tests live in `test\`, with the same structure as the sources:
- `test\rust\<path>.rs`: the tests of `tauri\src\<path>.rs`. They remain a child module of that file, which declares them at the bottom with `#[cfg(test)] #[path = "…/test/rust/<path>.rs"] mod tests;`: they see its private functions (`use super::*`), as before. A new test goes in the module's file in `test\rust`; a module that does not have one yet adds the three lines.
- `test\ui\<path>.test.ts(x)`: the tests of `ui\src\<path>`, which import from `../../ui/src/…`; `test\ui\setup.ts` sets up jsdom (Vitest, `ui\vite.config.ts`).

Filters: `cargo test ports::` or `cargo test db::services` (only the Rust tests whose path matches); `npm -w ui test -- activity` (only the interface's test files whose name matches).

The live tests on the version sources (network, usually ignored): `cargo test live_lines -- --ignored --nocapture` (with `GITHUB_TOKEN`), see CATALOG.md §2.

They cover, among other things:
- kit root, environment and PATH;
- kit files (`fsx.rs`): BOM, non-UTF-8 bytes, missing, empty or corrupted file, atomic write with no temporary files left behind;
- command errors (`commands\error.rs`): for `CmdError` Tauri sends the same value as for a string, `or_fail` sends the short text and `or_fail_chain` the whole chain;
- catalog, dependencies, parts (`keys`), ARM64 limits and components needed by commands and agents (`setup\needs.rs`);
- version lines read from the sources of each language (Go, Java, .NET, Zig: tests in `test\rust\install\lang\`);
- editor at the version pinned by the release;
- downloads with checksums and signatures;
- editor and real web server (with the token and without);
- editor windows that cannot call the app's commands;
- a real terminal session on Windows' ConPTY (`editor/conpty.rs`): the kit environment only, a folder with spaces, the exit code, typed input, and closing that ends the session's tree;
- agent alignment and Antigravity sign-in with the real Credential Manager;
- project detection on samples (Node, Next, Tauri, Electron, PHP, Laravel, Django, FastAPI, Flask, Streamlit, Rust, static, composite projects, `panel.json` and `pannello.json`);
- remembered and locked ports, starting and stopping an action through its job;
- the port registry (`test\rust\ports.rs`, D-141): every owner read from its file; a port of a pool, of a database version, of an action or of the editor refused to the others, with the owner named (`servers::`, `db::services::data`, `projects::run`), and never given to a new pool;
- database servers (D-140): operations on one server take turns (`operations_on_a_server_take_turns`), `keep_port` loses no entry, the ports stay when the server changes version, the login files differ per run;
- the project's `conf.d` in an Apache and a Caddy pool and in the project's own Caddy (`apache_pool_includes_the_project_conf_d`, `caddy_pool_imports_the_project_conf_d`, `project_conf_d_imported_in_the_site`);
- file server: host (`*.localhost`, an IP only with the local network), hidden files, `..`, links;
- exe swap during an update;
- DPAPI of `keyring-before.json`;
- catalogs of the four languages (Rust and interface), complete and with the same placeholders;
- `useLoad` (`test\ui\hooks.test.ts`): first load, the three error policies, a new load when `load` or `key` change but not the policy, `enabled`, periodic loading stopped on unmount, `setData`;
- interface controls on `vscode-elements` (`test\ui\workbench\controls.test.tsx`, `elements.test.tsx`): events that really fire, controlled values, Enter in forms, accessible names; the codicons (`codicon.test.tsx`): glyphs of the bundled font, token sizes, no `react-icons/vsc` left;
- the theme (`test\ui\theme\theme.test.ts`): every `--vscode-*` variable read by the components defined in both themes, every size declared once with its utility, the sizes of `docs\brand\vscodium-1.135\workbench-ref.css`, no token of the classic UI, every `--vscode-*` key Atollo reads known to VSCodium 1.135;
- the Activity panel (`test\ui\activity.test.tsx`): a secondary database version and a pool listed and counted, "Stop everything" stopping everything once confirmed; the Terminals page (`test\ui\terminals.test.tsx`): Esc closes the removal question;
- the command palette (`test\ui\commands.test.tsx`, D-160): the commands of each project, of the installed agents and database servers and of the other pages, "Stop everything" only while something runs, a project found by two words, every typed word matching in any order and case, a failure shown after it closed; `test\ui\App.test.tsx`: Ctrl+Shift+P, "vai impostazioni", Enter opens Settings;
- the components with a new version (`test\rust\setup\updates.rs`, D-161): versions compared by numbers, the notes' address with `{version}` or the site, only the installed components behind their source listed; `real_check_of_a_kit` (skipped without `ATOLLO_TEST_KIT`) asks the real sources of the kit it names; `test\ui\components\updates.test.tsx`: the pane's items, "What changes", "Update", "Check now", "Everything up to date" or "Not checked yet", the list read again when a preparation ends;
- "My bases" (`test\rust\projects\frameworks\mine.rs`, D-162): the ids from the names, the files without Git (no `vendor`, `node_modules`, `.env`, `.env.local`; `.env.example` kept), save with a folder left out, save again with the same name (replaced), the folder removed with the base, a `template` with `..` refused; `real_git_files` (skipped without `ATOLLO_TEST_KIT`) lists with the kit's real Git, `.gitignore` respected; `test\ui\projects\savebase.test.tsx`: the total when a folder is unticked, `project_save_base` with `skip`, the final message, "Le mie basi" the first group;
- the Guide's documents: `node scripts/guide-docs.cjs --check` (in the `Lint and test` job) fails if `docs\app\GUIDE.md` or `GUIDE.it.md` is behind the in-app Guide.

## Generated files and screenshots

None of these runs in `npm test`; each one is run by hand after the change it follows, and its output is committed.

| Command | When | Writes |
| --- | --- | --- |
| `python docs/brand/themes.py` | a color (`BRAND`) or a measure (`LAYOUT`) changed | `ui\src\theme\theme-vscode.css`, `theme-colors.css`, `theme-sizes.css`, `tauri\editor-theme\` |
| `node scripts/vscodium-css/extract.mjs [archive or workbench.css]` | the catalog pins another editor, or the app reads a new `--vscode-*` key | `docs\brand\vscodium-1.135\workbench-ref.css` (then `themes.py`); it fails on a key VSCodium does not know |
| `node scripts/guide-docs.cjs` | the Guide's text changed | `docs\app\GUIDE.md`, `GUIDE.it.md` |
| `python -I scripts/matrix/gen_matrix.py` | the catalog changed | `docs\app\TOOLS-MATRIX.html` |
| `node site/shots/make.mjs` | the pages in the screenshots changed | `site\shots\<lang>\shot-{projects,agents,tools}-{dark,light}.png` for en, it, fr, de |
| `node site/shots/social.mjs` | after `make.mjs` | `docs\brand\social.png` (from `en\shot-projects-dark.png`) |
| `node scripts/guide-shots/make.mjs` | the pages in the Guide changed | `ui\public\guide\<lang>\*.webp` (52 files; Python with Pillow) |

The screenshot scripts read the browser preview: first `npm -w ui run dev` (`127.0.0.1:5173`), with `playwright-core` installed next to the script or in a folder of its own and the system's Edge or Chrome (`SHOTS_BROWSER=msedge|chrome`, no download). `SHOTS_LANGS=en,it` (site) and `GUIDE_SHOTS_LANGS=en,it` (Guide) limit the languages; `SHOTS_URL=http://127.0.0.1:5199` takes the preview from another address, for both. Before committing, look at each language: the labels come from `ui\src\locales`, the engine's texts from `engineText`.

`npm run dev` runs `scripts/dev-ui.mjs`: closing the app must free port 5173 (D-138); with the port taken it says who has it and stops nothing.

## Linux, macOS and Windows ARM64

**CI** (`.github/workflows/app.yml`, WP-0.6). On every push to `main` and `feature/platforms` and on every pull request, only the jobs the change requires (D-134). The `Changes` job assigns each changed file the levels of the jobs that can see it break. The list is in the `Levels` step, and each level includes the ones below it:

- **bundle**: the package and the code that starts the app (`tauri/Cargo.toml`, `build.rs`, `tauri.conf.json`, `Info.plist`, `entitlements.plist`, `windows-app.manifest`, `icons/`; in `tauri/src`: `main.rs`, `cli.rs`, `kit.rs`, `log.rs`, `env.rs`, `settings.rs`, `fsx.rs`, `i18n.rs`, `os/`, `setup/`). `Bundle *` and `Release smoke *` run, plus everything below.
- **kit**: what `prepare` and the `real_` tests use on a real kit (`tauri/src/install/`, `net/`, `editor/`, `runtime/`, `tauri/catalog/components.json`). `Build …` runs with the exe, the real installation and the real tests, plus everything below.
- **rust**: the rest of the app's code and data (`tauri/src` in the other modules, the other catalogs, `i18n/`, `editor-theme/`, `capabilities/`, `permissions/`, `site/assets/fonts/`) and the Rust tests (`test/rust/`, `scripts/matrix/sources.json`, `scripts/test-linux.sh`, `rustfmt.toml`). `Lint and test`, `Test *` and `Test win32-arm64` run, with `clippy` and `cargo test` on every system.
- **ui**: the interface (`ui/`, `test/ui/`, `eslint.config.mjs`, `.prettierrc.json`, and also `components.json`, which the interface tests import, and `docs/app/GUIDE.md`, `GUIDE.it.md` and `scripts/guide-docs.cjs`, which `--check` compares). Lint, Vitest, the Guide's documents and build run in `Lint and test` (plus `cargo fmt`), and Vitest alone in `Test linux-x64`.

`Cargo.toml`, `Cargo.lock`, `package*.json`, `.gitattributes` and `app.yml` start everything. Generated files (`tauri/gen/`), documents, the site, the Apache and PHP scripts (with their workflows) and `scripts/cross-check/` start nothing but `Changes`. A path the list does not know starts everything, and the summary names it: it must be added to the list. The full run also always starts with an `app-v*` tag, with a manual start (`gh workflow run app.yml`) and when the comparison fails (for example after a force push). The summary of the `Changes` job shows the chosen levels.

| Job | Runner | What |
| --- | --- | --- |
| `Lint and test` | `windows-latest` | lint, Vitest, interface build, `cargo fmt`, `clippy`, `cargo test` |
| `Test linux-x64` | `ubuntu-latest` | Tauri libraries, interface build and Vitest, `clippy --all-targets -D warnings`, `cargo test` |
| `Test linux-arm64` | `ubuntu-24.04-arm` | as above, without Vitest |
| `Test darwin-arm64` | `macos-latest` | interface build, `clippy`, `cargo test` |
| `Test darwin-x64` | `macos-15-intel` | as above (`macos-13` no longer exists) |
| `Test win32-arm64` | `windows-11-arm` | interface build, native `cargo test` |
| `Build …` | `windows-latest` | x64 and arm64 exe, real tests (x64) |
| `Bundle darwin-*` | `macos-latest`, `macos-15-intel` | `Atollo.app` with verified ad hoc signature, `Info.plist`, zip, trial start (D-113) |
| `Bundle linux-*` | `ubuntu-24.04`, `ubuntu-24.04-arm`, `ubuntu:22.04` container | AppImage and `tar.gz`, glibc minimum with `objdump`, `--version` and `check` under `xvfb-run` |
| `Release smoke *` | `macos-latest`, `macos-15-intel`, `ubuntu-24.04`, `ubuntu-24.04-arm` (no container) | the `bundle-<target>` artifacts the release publishes: names from `self_update.rs` and `.sha256`; macOS: quarantine set and removed with `xattr -dr`, signature, `--version`, `check`, kit next to the `.app`; Linux: AppImage with the release name with `--appimage-extract-and-run`, `tar.gz` with only `libwebkit2gtk-4.1-0` (`ldd` with no “not found”), `--version` and `check` under `xvfb-run` (D-119) |
| `Release` | `windows-latest` | only on `app-v*` tags, after `Build …` and `Release smoke *`: exe, `.app.zip`, AppImage and `tar.gz` with the `.sha256` checked again, release notes |

**Apache build** (`.github/workflows/httpd.yml`, D-116), only on pull requests that touch `scripts/build-httpd/**`, by hand (`gh workflow run httpd.yml --ref <branch>`) and on the `httpd-v<version>-<build>` tag, which publishes the release: `httpd <target>` for the four Unix targets (Linux in the `ubuntu:22.04` container), `build.sh` and then `test.sh`, which starts Apache from a folder with a space and parentheses and after moving it, and checks the modules the kit loads, the glibc minimum and the libraries (Linux), the macOS 12 minimum and the signatures (macOS). Locally for Linux x64: `docker run ubuntu:22.04` with the job's packages, `WORK=/w bash scripts/build-httpd/build.sh linux-x64 0 /out`, then `test.sh /out/httpd-*.tar.gz`.

Each test job writes the passed, failed and ignored tests to the run summary, with the names of the failed ones. On macOS all of `cargo test` runs: with tauri 2.12.1 the `test` feature (the fake runtime of `test\rust\main.rs`) compiles there too, so no separate feature is needed (with 2.12.0 `transparent` was missing). In a repository without ARM64 runners (private) the repository variable `ARM64_RUNNERS=off` removes the `linux-arm64` and `win32-arm64` jobs, which would otherwise stay queued: `linux-arm64` drops out of the matrix's target list (the runner is derived from the target, without `include`, which GitHub applies after `exclude` and would put the removed target back). The `win32-arm64` job installs the `stable-aarch64-pc-windows-msvc` toolchain and fails if `rustc -vV` does not report `host: aarch64-pc-windows-msvc`: the tests never run in x64 emulation without saying so.

A test that only holds on Windows (paths with `\`, `cmd.exe`, `ping.exe`, junctions with `mklink /J`, the kit's `.exe` layout) carries `#[cfg_attr(not(windows), ignore = "<reason>")]`: on Windows it runs, elsewhere it counts among the ignored ones with the reason. A new test instead uses, when it can, pure functions with the target as a parameter (`Kit::at_target`, `fsx::join_rel` with `/`), so it holds for all six targets.

**WSL2 locally.** `scripts/test-linux.sh` copies the repository into the Linux file system (one copy per checkout or worktree in `~/atollo-copies/<folder>-<hash>`, or `ATOLLO_LINUX_DIR`; two runs on the same copy take turns with `flock`; with `rsync`, without `target*`, `node_modules`, `.claude/worktrees`, `.git` and the kit folders `data` and `projects`), checks Rust and the Tauri libraries (the `install ok installed` state from `dpkg-query`), then runs `cargo test` and prints the summary:

```
wsl -d Ubuntu -- bash scripts/test-linux.sh            # from Windows, in the repository root
bash scripts/test-linux.sh                             # inside WSL or on Linux
bash scripts/test-linux.sh --clippy -- servers::       # clippy too; filter and options after --
```

If something is missing it prints the command to run (`sudo apt-get install -y …`, or `rustup` for the user) and exits with 2; with failed tests it exits with 1. `target/` stays in the copy, so the runs after the first one compile only what changed. The copy uses `rsync --delete`: the script copies only into an empty folder or into a copy it made itself (file `.atollo-linux-copy`); otherwise it exits with 2 without touching anything. A clone of the repository without that file (with its own `.git` and unsaved work) is refused too: choose another folder with `ATOLLO_LINUX_DIR`. `bash scripts/test-linux.sh --check-dest` only says whether the folder is fine (0 yes, 2 no) and runs anywhere, Git Bash included. `cargo` runs inside the copy, so its `.cargo/config.toml` and `rust-toolchain.toml` apply, not the ones above the Windows folder. The built interface (`dist\`) comes from the Windows copy (`npm -w ui run build`); without it, the script builds it in Linux with Node.js. WSL2 requires virtualization enabled in the BIOS and the Windows “Virtual Machine Platform” (`wsl.exe --install --no-distribution`, as administrator). Before the script, in a worktree, run `npm -w ui run build` from Windows: inside WSL the `npm` that is found may be the Windows one.

**Cross-check from Windows.** `bash scripts/cross-check/xcheck.sh <target> clippy --all-targets -- -D warnings` (targets `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin`) compiles the Unix code with fake tools, without a linker: it finds type and `cfg` errors, not link errors (a duplicate `_EMBED_INFO_PLIST` on macOS was seen only by CI).

**CI on another branch.** Pushes to `main` and `feature/platforms` start CI on their own; for another branch: `git push -u origin <branch>`, then `gh workflow run app.yml --ref <branch>`.

**Tests with real programs** (`ATOLLO_TEST_KIT=<kit with the components>`, ignored without it):
- `real_databases_start_and_stop`: the kit's MySQL and MongoDB on free ports: start, a project's database and user (TCP access as that user), backup and restore (with the safety copy before it, D-154), deletion, stop with nothing left behind (D-110); on Windows and in WSL;
- `real_mongo_express_on_running_mongodb` (`db::mongo_express`, D-147; also `ATOLLO_TEST_MONGODB_PORT`, the port of the kit's MongoDB already running, with `node` and `mongo-express` in the kit): 403 without the token, with a wrong one and for another Host; the token gives an HttpOnly, SameSite=Strict cookie and leaves the address; the databases listed as admin; the same server when opened again on that version; stopped with it. The archive itself is checked by `scripts/build-mongo-express/test.sh` in CI, on MongoDB 7.0 and 8.0;
- `caddy_pool_two_php_sites_alias_and_reload`: a Caddy pool with two PHP sites and an alias, `caddy reload` after adding a site without stopping the other one (D-111);
- `apache_pool_two_php_sites_alias_and_reload` (with `start_reports_syntax_error` and `apache_pool_reload_twice`): an Apache pool with two PHP sites (module on Windows, `php-fpm` on macOS and Linux), an alias, `.env` and foreign hosts 403, a script and a hostile `.htaccess` in the alias folder neutralized, Drupal's `files` folder without PHP, a third site after the reload (D-116, SEC-V4-01); on Windows and in WSL. In WSL, until the build is pinned in the catalog, Apache is set up by hand: the archive in `data/apps/linux-x64/apache/2.4/`, `httpd.conf` rewritten as `httpd_conf` does, `data/app/versions/linux-x64/apache.txt`;
- `mysql_is_the_kits_sql_server` (`db::services::data`, no kit): a new kit shows MySQL and MongoDB, each installed MySQL version has its own port and a project's programs get the login of its version; `a_new_kit_installs_mysql_for_phpmyadmin_wordpress_and_laravel` (`commands::frameworks`): phpMyAdmin, WordPress and Laravel install MySQL, on the six targets.
- `real_offline_reinstall_after_a_move` (`install::conda`, D-118, macOS and Linux): with the conda environments installed in `ATOLLO_TEST_KIT` by this version of the app (`Atollo prepare --only r,ruby,mongodb,sqlite,cc`, which keeps archives and lists), the kit folder is renamed alongside (`<name>-moved-d118`; no spaces: conda-forge's `bin/R` writes `SED=` without quotes and breaks in any folder with a space), `Kit::prepare` flags the components, a preparation with a proxy that does not exist recreates them from the kept archives; each program answers from the new folder (`R RHOME` under it), gems and MongoDB data unchanged; then the same in reverse. To really test it without a network, run the test binary under `unshare -rn` (in WSL it works without root):

  ```
  cargo test --no-run   # prints the path of the test binary
  ATOLLO_TEST_KIT=~/d118/kit unshare -rn target/debug/deps/Atollo-<hash> real_offline_reinstall_after_a_move --nocapture
  ```

**Secret Service in WSL without root** (D-115). To test Antigravity with a real Secret Service:

```
mkdir -p ~/gk && cd ~/gk
apt-get download gnome-keyring libgcr-base-3-1 libgck-1-0 libsecret-tools
for f in *.deb; do dpkg -x "$f" root; done
export LD_LIBRARY_PATH=~/gk/root/usr/lib/x86_64-linux-gnu PATH=~/gk/root/usr/bin:$PATH
dbus-run-session -- sh -c 'echo | gnome-keyring-daemon --unlock --components=secrets; cargo test keyring'
```

With a `keyring.json` that contains `gemini:antigravity` and a Windows name (`jetski:other`), after `restore` and `save` the Secret Service is empty and `keyring.json` has both entries. Without a Secret Service the plaintext copy is written only to the Linux disk (600), never to `/mnt/<letter>`.

The `login` collection (D-115, issue #6) with the ignored test `real_secret_service_login_collection`, twice, each time with an empty `XDG_DATA_HOME`: with `echo | gnome-keyring-daemon --unlock --components=secrets` (`login` exists) the `agy` entry goes to `/org/freedesktop/secrets/collection/login`, is read back and deleted; with `gnome-keyring-daemon --start --components=secrets` (no `login` keyring) the write fails with `NO_LOGIN_COLLECTION` and nothing is left behind.

```
XDG_DATA_HOME=$(mktemp -d) dbus-run-session -- sh -c 'echo | gnome-keyring-daemon --unlock --components=secrets; cargo test real_secret_service_login_collection -- --ignored --nocapture'
```

**Kit shared between Windows and WSL.** PHP 8.5 installed from Windows and from WSL in the same kit folder, in both orders: `data/php/8.5/php.ini` with the 12 `extension=` lines and `data/php/linux-x64/8.5/php.ini` with none; `php -v` on Linux with no “Unable to load”. phpMyAdmin 5.2.3 on the static-php-cli PHP 8.5.8: `index.php` 200, no warning with `E_ALL`.

**Preparation from Windows** (D-114). `Atollo.exe prepare --target linux-x64 --only git` says `(bootstrap linux-x64)`; on the target the first start, or `prepare --only git`, writes “bootstrap pre-downloaded” and `git clone https://…` works. In WSL: `prepare --only cc` without Zig does not set `CC`/`CXX` or put `cc/bin` in the PATH; `prepare --only zig,cc` gives `CC=…/cc/bin/cc` and a hello in C compiles with `make` and runs.

**Linux bundle locally.** `docker run ubuntu:22.04` with the packages of the `Bundle linux-*` job, `tauri build --bundles appimage`, glibc minimum with `objdump -T`, then `--version` and `check` under `xvfb-run` for the AppImage (`--appimage-extract-and-run`) and for the `tar.gz`.

## Tests on the real interface

For the hands-off tests on the release exe (phase 6) a test kit outside the repository is used, with all the components, in a folder with spaces:
- **Starting outside the tool's session:** the app is started with WMI (`Win32_Process.Create`). The tool closes the processes it launches directly, so the app would not survive the command.
- **Clicks:** with UI Automation, on the accessible names of the buttons. The same names also serve Narrator.
- **Screenshots:** with `PrintWindow` on the “Atollo” window.
- **Process check:** after “Stop”, “Close everything” and “Remove drive”, no process may have its executable in the test kit (`Win32_Process.ExecutablePath`).
- **Stopping the tests:** the test processes are stopped **by PID**, never by name. A `taskkill /IM Atollo.exe` also closes the other copies of the app open on the PC (it happened during phase 6).

## Manual checklist

Compared with the checklist in the initial prompt (the requirements are in ARCHITECTURE.md, “Requirements”):
- the network item uses the Settings page instead of `data\proxy.cmd` (D-54);
- the item about the `setup\Windows.cmd` menu does not apply, because this is a separate product, without the kit's scripts (Q1 of the plan).

To be run before every release, with the release exe (`npm run build`), on a PC that has never seen the kit, when the item requires it.

### Installation and portability
- [ ] **New kit on a USB stick.** Double-click, first start: PowerShell 7, GitHub CLI (with Git) and the editor install on their own; then installation of the defaults, editor open on a project.
- [ ] **Editor at the release version.** With a kit prepared by another version of the app (or with a different number in `data\app\versions\vscodium-server.txt`), at startup the editor is installed again at the pinned version, even if it had been left open, without “Update all”. If a program on the PC keeps the editor folder open, the update ends “in use”, but Editor, Terminal and the agents open anyway with the editor already present. In Tools the editor is among the required ones, checked and not editable.
- [ ] **No trace outside the kit.** After a full session (editor, terminal, an agent, a started project) there is no new file in `%APPDATA%`, `%LOCALAPPDATA%`, `%TEMP%` or in the registry. Before/after comparison with Process Monitor or with a snapshot of the folders. Documented exceptions: Antigravity's credentials, which stay in Windows only for the lifetime of `agy` and with session persistence (D-41); the transit files in `%TEMP%\Atollo\<id>\`, which are gone at the end of the session (D-90).
- [ ] **Moved kit.** On another drive letter and in a folder with spaces everything works, the editor's state remains and Claude finds its history again.
- [ ] **PC without WebView2** (Windows 10 LTSC or a virtual machine). The message is clear and the app works with the fixed runtime in the kit.
- [ ] **Standard user** without administrator rights: no UAC prompt at any step.
- [ ] **Authenticated proxy and SSL inspection.** Settings › Network with a proxy and a PEM file: installation, editor extensions and agents work.
- [ ] **No network.** The app and the editor start, download errors are readable, the update check fails without blocking anything.
- [ ] **Forced close from Task Manager.** No `node.exe`, editor server, kit terminal (`OpenConsole.exe`, `pwsh.exe`) or project server is left.
- [ ] **ARM64.** The exe is native, the editor runs in emulation and the warning is shown.
- [ ] **Moved R** (D-104). Kit with R on `D:`, then on `E:`: after startup `lib\R\etc\x64\Makeconf` has `LOCAL_SOFT` on the new folder and `install.packages(..., type="source")` compiles.
- [ ] **macOS and Linux**: the “macOS, Linux and Windows ARM64” section below.

### Projects
- [ ] **New project** “test 1”: the folder appears, has the “Delete the empty folder” button and the deletion works.
- [ ] **Clone** `nabre/atollo` (or a private repository with `gh auth login` done in the kit's terminal): the folder has the right name. On a kit without Git, Git is installed before the clone. With an invalid address (`-x/y`, `ssh://…`) the error is readable.
- [ ] **Vite project.** “Start”, then the port 7100-7999 appears in the output. The state changes to “listening” and the button with the address opens the PC's browser. A second start reuses the same port. On a kit without Node.js, “Start” first installs Node.js (progress in the Activity panel and in the status bar) and then starts; Node.js shows as chosen in Tools.
- [ ] **Local network.** With “Let other PCs on the network reach the project servers”, another PC opens the started project with this PC's IP address; without it, it does not. The same for a simple PHP project started with `php -S`: `http://<IP of the PC>:<port>/` answers from another device, `curl -H "Host: evil.com"` still gets 421.
- [ ] **Site QR for the phone** (D-158). With “Local network” on, start a Vite project (after turning it on): the project page › the action's details have “From the phone” under “Address”, with this PC's private IP and the port (the same as `ipconfig` shows for the Wi-Fi or Ethernet adapter). “Copy” puts it in the clipboard; “Show the QR code” shows a black-on-white QR (in the dark theme too) and the hint; scanning it with the camera of a phone on the same Wi-Fi opens the site. “Hide the QR code” hides it. The first time the site starts, Windows' firewall asks: allowing private networks is enough.
- [ ] **No QR row.** With “Local network” off the row is not there. A project in a web server pool, with an address like `http://shop.localhost:<port>`, has no row even with “Local network” on. A stopped action has no row.
- [ ] **Port in use.** With a port chosen by hand that is already used by another program, the start gives a clear error and the port does not change on its own.
- [ ] **Port of another part of Atollo** (D-141). An action's port chosen by hand that is the port of a web server pool, of a database version (for example 3306) or in the editor's 7000-7099: refused, and the message names the owner (“Port 3306 already belongs to database MySQL 9.7: choose another one.”). The same for the port of a pool profile and for the port of a database server or of one of its versions; a new pool never gets one of them.
- [ ] **Static site.** A folder with only `index.html`: the kit's file server serves it, at `http://localhost:<port>` and at `http://<project>.localhost:<port>`. A `.env` file is not reachable (403).
- [ ] **Activity and “Stop everything”.** With an action, a secondary MySQL version and a pool running: the status bar, the Projects badge and the Activity panel count the three and the panel lists them; “Stop everything” asks first, “Cancel” changes nothing, the confirmation stops all three.
- [ ] **“Serve as a website”.** With the `public` folder and PHP: `panel.json` contains the `site` action and the other keys of the file remain.
- [ ] **“Stop”** on `npm run dev`: in Task Manager no `node.exe`/`esbuild.exe` of the project is left.
- [ ] **Electron or Tauri.** The app opens its window and the output reaches the page.

### My bases (D-162)
- [ ] **Laravel with Git.** In a Laravel project that is a Git repository, with `composer install` done and a `.env`: the project page › “Save as a base” (next to “Open folder”). “What to copy” lists the top folders and files with their count and size, without `vendor`, `node_modules` and `.env`, with `.env.example`; the total is under the list. “Save as a base” answers “Saved: you find it in New project › My bases…”. `data\frameworks\mine\mine-<name>` has the files, no `.env`, no `.git`; `data\app\frameworks.json` has the base with `"category": "mine"` and `"template": "mine-<name>"`.
- [ ] **.gitignore respected.** Add a folder to `.gitignore` (e.g. `docs/`) and a new file not ignored: the folder is not in “What to copy”, the new file is (counted in its folder).
- [ ] **Node without Git.** A Node project without `.git`, with `node_modules`, `dist` and `.env.local`: none of the three in the list or in the saved folder; `src`, `package.json` and `.env.example` are.
- [ ] **Unchecked folder.** Untick a folder: the total goes down; after saving, the folder is not in `data\frameworks\mine\<id>`.
- [ ] **New project from “My bases”.** “New project” (wizard, Quick panel, the sidebar tree after reopening Projects): “My bases”, with the star, is the first group. Creating a project from the saved base copies its files into `projects\<name>`, then writes `.env` and runs the steps; `package.json` or `composer.json` keep the original project's name. The same into an existing folder with files and no Git: the files already there stay.
- [ ] **Same name replaces.** Save again with the same name after removing a file from the project: the base is listed once and the file is gone from its folder.
- [ ] **Removed in the editor.** Settings › framework instructions › the base › “Remove”: after the save “Saved.”, `data\frameworks\mine\<id>` is gone and the base is no longer in the wizard. Deleting the folder by hand instead: creating from the base says “The base's folder is gone: save it again from the project.”

### Web servers and databases (v3 swarm, D-96…D-102)
- [ ] **Valid configurations.** With a PHP project started on dedicated Apache and Caddy, in a pool and with the kit's site: `httpd -t -f <conf>` and `caddy validate --config <Caddyfile>` on the configurations written by the kit give no errors. A `kit.conf` from an earlier kit, after startup, has the new blocks only once.
- [ ] **PHP extensions** (D-96). With PHP: `a.phtml` runs; `a.inc`, `a.php5` and `.env` give 403. Without PHP (Apache without the module, Caddy without php-cgi) `index.php` gives 403/503, never the source.
- [ ] **`php -S`** (D-97). Simple PHP project started with “Start”: `/.env`, `/a.inc`, `/index.php.` give 403; `curl -H "Host: evil.com"` gives 421; `/` and a static file 200. With a `router.php` in the project, that one applies.
- [ ] **Foreign host on the dedicated server** (D-98). Without “Local network”: `curl -H "Host: evil.com" http://127.0.0.1:<port>/` gives 403 (Apache) or 421 (Caddy); `localhost`, `127.0.0.1`, `<project>.localhost` and the project's domains answer. With “Local network” the PC's IP from another PC opens the site.
- [ ] **php-cgi dying** (D-99). PHP project on dedicated Caddy started (Windows): closing `php-cgi.exe` from Task Manager also stops `caddy.exe`; the run goes into error and the log has “php-cgi exited”.
- [ ] **Pools that follow the projects** (D-101). Two projects assigned to the same profile: “Start” on the first starts the pool, on the second reloads it; “Stop” on the first removes it (its address no longer answers) and the second stays served. With “Keep running” off, stopping the second too shuts down the pool; on, the pool stays. A pool started from the Web Servers page stays on regardless.
- [ ] **Start with Atollo** (D-101). Profile with “Start with Atollo”: with the app closed and reopened, the pool is on with no project started. A `servers.json` from before (without the two fields) is read with “Keep running” checked.
- [ ] **Two web actions of a project in the same pool** (BUG-V3-02). `http://<project>.localhost:<port>` and `http://<action>.<project>.localhost:<port>` serve the two folders.
- [ ] **Two MySQL versions together** (D-102). With 8.4 and 9.7 installed: Databases › “Other versions” starts 8.4 on its own port while 9.7 stays on 3306; `mysql -uroot -h127.0.0.1 -P<port> -e "select version()"` gives the two versions. Project with “Server version” 8.4: “Create” creates database and user on 8.4; a program of the project sees the `DB_PORT` of 8.4; with a `.env` that has `DB_PORT=3310`, that one applies. Changing the main version while 8.4 is running is refused; with the app closed, no `mysqld` is left.
- [ ] **One start at a time** (D-140). Two projects that need the same MySQL version started together (or “Start” twice in a row on the Databases page): one `mysqld`, and both get the same port. Changing the server's version with 8.4 and 9.7 installed: each version keeps its port, and the projects' connection strings still point to their version. Two backups started together both finish.
- [ ] **The project's `conf.d`** (D-141, BUG-V5-06). A `.conf` in `data\projects\<p>\apache\conf.d\` and a `.caddy` in `caddy\conf.d\` (for example a header added): the site has it on the dedicated Apache, on the dedicated Caddy, in an Apache pool and in a Caddy pool. With an empty `caddy\conf.d\` Caddy still starts.
- [ ] **“Delete profile”** of a pool asks first.
- [ ] **Tests with a real kit.** `ATOLLO_TEST_KIT=<kit with Apache and PHP> cargo test -- start_reports_syntax_error apache_pool_reload_twice apache_pool_two_php_sites_alias_and_reload --test-threads=1` passes.

### macOS, Linux and Windows ARM64 (D-105, D-108…D-115)
On macOS Apple Silicon and Intel, Linux x64 and ARM64 (a PC or a VM with a desktop: WSL2 has no real USB sticks or `noexec`), with the CI bundle or `tauri build --debug --no-bundle`.

First start:
- [ ] **Kit on an internal disk and on an exFAT USB stick.** Startup, `projects/` and `data/` next to `Atollo.app` or the AppImage, nothing inside the bundle. A `.app` downloaded with the browser and not moved (App Translocation) stops the startup with the `xattr` command.
- [ ] **Opening a second time** brings the first one to the front.
- [ ] **Required components** (PowerShell 7, GitHub CLI, Git, editor) installed and verified; `Atollo check` green.
- [ ] **Projects.** New empty project and clone of a private repository.
- [ ] **Editor**: opening, integrated terminal with the kit's environment (`echo $CP_DATA`), extensions installed.
- [ ] **Kit terminal and Claude Code** from a project: they open in a tab of the app, respond to keys, and closing the tab leaves no process behind.
- [ ] **Closing the app** with a terminal and a database open: no process is left. `kill -9` of the app: no kit process within 6 s (`pgrep -f <kit>`).
- [ ] **Kit moved** to another folder: everything starts again, Git without reinstalling.
- [ ] **Kit moved without a network** (D-118), with R, Ruby and (on Linux) MongoDB installed: on the offline restart they are recreated from the kept packages (“created again from the kept packages” in the Activity panel), `R --version`, `ruby -v`, `mongod --version` answer; Settings → General shows “Copy to reinstall without the network: N MB”; after “Free” the next move asks for the network.
- [ ] **USB stick mounted `noexec`** (Linux): message with the command to remount, no installation.
- [ ] **“Remove drive”**: closes everything and lets you eject.

Components:
- [ ] Every component available for the system: installation, `check`, removal with nothing left behind. Android on Linux ARM64 shows as not available with the reason.
- [ ] **C/C++** on Linux without `build-essential` and on macOS without Xcode CLT: `cc hello.c && ./a.out`, `cargo build` of a crate with `cc-rs`. On macOS without CLT the component's page warns about the Apple frameworks.
- [ ] **Rust on Windows 11 ARM64** without Visual Studio: `cargo new x && cargo run`.
- [ ] **Ruby**: `ruby -v`, `gem install json` (native); after a kit move the gems' commands start.

Web stack and databases:
- [ ] **Laravel with MySQL** on macOS and Linux: `composer create-project`, migrations, site on Caddy, phpMyAdmin.
- [ ] **WordPress with Caddy**: permalinks working without `.htaccess`. Drupal suggests Apache, which the catalog offers on every system (on macOS and Linux the kit's build, D-116).
- [ ] **Apache on macOS and Linux** (D-116). A Laravel project with its own `.htaccess`, Apache site: `php-fpm` starts with Apache (`data/projects/<p>/apache/fpm.conf`), the routes answer, `httpd` runs with `-DFOREGROUND`; stopping the site stops both.
- [ ] **Caddy pool** with two sites and an alias to another project; adding a site reloads the pool without stopping the other one.
- [ ] **Same MySQL data folder** read from Windows and from Linux with the same line.
- [ ] **MySQL on Ubuntu 24.04** without `libaio1` (only `libaio1t64`): the server starts; without `libnuma1` the error says `sudo apt install libaio1t64 libnuma1`.

Sign-ins and package:
- [ ] **Antigravity and Claude Code** connected on a Mac (Claude with “Connect with a token”), kit taken to another Mac: already connected; nothing is left in the first one's Keychain. With the token, Claude Code opened from Atollo starts connected; a kit terminal, the kit's editor, a project action and `Atollo env` do not have `CLAUDE_CODE_OAUTH_TOKEN`; after “Sign out” Claude Code opened from Atollo asks to sign in again.
- [ ] **Antigravity on Linux with KeePassXC or KWallet** without a `login` collection: no entry moved, warning “no "login" collection”; with GNOME `agy` finds the sign-in.
- [ ] **Antigravity on Linux** with a locked Secret Service: no entry moved, warning; without a Secret Service and with the kit on exFAT: refused.
- [ ] **Releases** (D-119, #5): on a clean Mac `Atollo-<version>-darwin-<arch>.app.zip` downloaded with the browser, extracted to an exFAT USB stick, opened following the README (`xattr -dr com.apple.quarantine`); without that step the startup stops with the command. AppImage started from exFAT on Ubuntu 24.04, Fedora 42 and Debian 13; without FUSE `./Atollo-<version>-linux-<arch>.AppImage --appimage-extract-and-run`; the `tar.gz` with WebKitGTK 4.1. App update from one test version to another on macOS and Linux: the AppImage takes the new version's name and the old one disappears at the next startup (the `tar.gz` says that the new version must be downloaded by hand).

Kit prepared on Windows for another system (at every release):
- [ ] On Windows `Atollo.exe prepare --target linux-x64` (then `darwin-arm64`, `linux-arm64`) on an exFAT USB stick, with the app for that system copied next to it.
- [ ] The “Platform” selector shows for that target what is pre-downloaded, what completes on first start and what is not available.
- [ ] “Tools” › Overview › “Grid” (D-121): the six columns with readable names and “this PC”, “in emulation” or “cannot run here”; on the prepared USB stick the target's cells say “pre-downloaded” or “completes on first start”, the unavailable ones the reason; a click opens the detail and “Show this system” changes “Platform”. With the app closed and reopened, “Platform” is still that system; once this PC's one is chosen, on the other system the page opens on its own.
- [ ] On the other system: first start from the USB stick, installations from `.bootstrap` completed (Rust, Git from dugite-native, R and the conda components, Android, pnpm), no new download for what was already downloaded.
- [ ] Back on Windows: the kit opens as before; `data/apps/win32-x64` and the Windows settings are intact, the choice of components has not changed.
- [ ] Same project opened on the two systems: the project's settings, ports and databases are shared (`data/projects/<p>`).

### Per-project data and transit files (D-89, D-90)
- [ ] **Migration from a version 1 kit.** With the copy of a real kit (data in `data\app\projects`, `ports.json`, `secrets.json`, `data\db\backup`): at the first start `data\projects\<project>\` appears, `data\app\layout.txt` is `2`, `data\app\pre-v2\` has the two copies. Projects, ports (locked ones too), passwords and database backups intact; the projects' `.env` still valid; a second start changes nothing.
- [ ] **exFAT USB stick.** Installation of Node, Python and Java: the progress gives no errors about space or about renames across disks; “Temporary files of programs: Automatic” uses the system; after the app closes no file is left in `data\tmp` and `%TEMP%\Atollo\<id>\system` no longer exists.
- [ ] **FAT32 USB stick.** A component with a file over 4 GiB is refused with an error that says which file, the version already installed stays intact and no `.atollo-new` is left.
- [ ] **Rust** (`695c177`).
  - NTFS kit without Rust: installation, `cargo --version` from `rust\cargo\bin`.
  - NTFS kit with Rust: in-place update (“rustup update” in the log), a `cargo install` tool remains.
  - FAT32/exFAT kit without Rust: no error about hard links, `rustup-init` in the temp folder and copy to the USB stick; `cargo`, `rustc` and `rustup show` work from the copies of the proxies.
  - FAT32/exFAT kit with Rust: update (“rustup update (copy in the temp folder)”), `cargo install` tools present, no `rust.atollo-new` left.
  - `TEMP` on a FAT/exFAT disk: clear error about the temp folder without hard links, kit intact.
- [ ] **Deleting a project with MySQL and MongoDB.** Server stopped: the confirmation lists folder, engines and backups; after “Delete” the servers start, database and user are removed (`SHOW DATABASES`, `SHOW GRANTS`) and `data\projects\<project>\` does not exist. With a server that does not start nothing is deleted; “Delete without the databases” deletes the folders and the log says which databases remain.
- [ ] **Restoring a backup made elsewhere.** A `.sql` backup obtained with `mysqldump --databases altro` is refused by the server (access denied) and the `altro` database stays intact; a backup of the project's database is restored.
- [ ] **Safety copy before a restore** (D-154). “Restore…” asks “Restore {file}? The kit first copies what the database holds now.”; after confirming, `data\projects\<project>\private\backups` has `<db>-prima-del-ripristino-<stamp>.sql` (MySQL) or `.archive.gz` (MongoDB) with the data from before the restore, and restoring that copy brings it back. MySQL with a view, a trigger, a routine and an event created by root in the project's database: the restore succeeds and they come back with the project's user as definer; a view on a dropped table does not stop the safety copy. With the copy impossible (server stopped while copying, disk full) nothing is restored.
- [ ] **Safety copy before a change of version** (D-154). With two versions of MySQL (or MongoDB) installed and a project's database with data, changing “Server version” leaves `<db>-prima-del-cambio-di-versione-<stamp>` in the backups, taken on the version left. With the version left uninstalled, the change goes on and the log says no copy was made.
- [ ] **Copy .env block** (D-157). Project page › Databases, with the database created: “Copy .env block” shows “Copied”. In a Laravel project (`DB_CONNECTION=` in `.env`) the clipboard has the six lines `DB_CONNECTION` … `DB_PASSWORD`; in a Node project without it, `DATABASE_URL=mysql://…` (MongoDB: `MONGODB_URI=mongodb://…`). Pasting into an empty `.env` or into Notepad gives the same lines, and the project connects with them. The `.env` file is not changed.
- [ ] **Only three safety copies kept** (D-154). After four restores there are three `prima-del-ripristino` copies (the newest), the copies made with “Export” are all still there, and the `prima-del-cambio-di-versione` ones are untouched.
- [ ] **Similar names.** Project `my-app` (database `my_app`) and project `my2app`: the user `my_app` does not see `my2app` (`SHOW GRANTS`). A project called `mysql` or `admin` cannot delete or change the servers' databases.
- [ ] **Project renamed in File Explorer.** It appears in “Data without a project” on the Projects page; “Link to…” brings it back with passwords and backups, and the `.env` keeps working. An orphan with databases is deleted with the same confirmation.
- [ ] **C: almost full.** The installations fall back to `data\tmp` and the log has the warning.
- [ ] **Site aliases.** A site with `/media` → `storage/uploads` and `/shared` → `@common/assets`, with Apache, with Caddy and in a pool: the files open in the browser.
- [ ] **Alias and junction.** `storage` replaced by a junction to `C:\Users`: the alias is not served and the site's page says so.
- [ ] **Settings › Temporary files of programs.** The page shows the kit's file system and disk type; Automatic, In the kit and In the system change `$env:TEMP` in a new kit console (`data\tmp\system` or `%TEMP%\Atollo\<id>\system`).
- [ ] **Aliases and `.htaccess`** (D-100, SEC-V4-01). In an alias folder served by Apache, a symbolic link does not open; an `.htaccess` with `Options +FollowSymLinks`, `RewriteRule`, `AddHandler` or `Require all granted` changes nothing (warning in Apache's log, the folder stays served); a `.php` or `.phtml` there gives 403 with Apache, 404 with Caddy.
- [ ] **Files downloaded by hand** (D-82). A file put by hand in the box ends up in `%TEMP%\Atollo\<id>\downloads`; one left in `data\downloads` is used anyway and moved.

### GitHub
- [ ] **Sign-in.** GitHub › “Sign in to GitHub”: github.com/login/device opens in the PC's browser and the page shows the code. After the authorization “Signed in as @…” appears without reloading, `data\gh\hosts.yml` contains the account and in the Windows Credential Manager there is no `gh:github.com` entry. “Cancel” during the wait leaves no `gh.exe`.
- [ ] **A single sign-in.** After signing in, `git config --global --get-all credential.https://github.com.helper` in a kit console gives an empty line and `!gh auth git-credential`. Push of a private repository from the editor (Source Control) and from a console without any Git Credential Manager window. Claude Code uses `gh` with the same account. After “Sign out” the helper disappears. After moving the kit to another drive letter, the push still works.
- [ ] **Repositories.** List with your own and your organizations' repositories (separate groups), search, “Clone” on a private repository: the folder is in `projects\` and `.git\config` does not contain the token. With the name already taken the folder becomes `<owner>-<name>`. On the cloned repository there is “Editor”.
- [ ] **Rename and delete** on a test repository: after “Rename” the clone's remote points to the new name. “Delete from GitHub” asks for confirmation; the first time it offers “Grant the permission”, then with a new sign-in the deletion succeeds and the clone stays in `projects\`.

### Agents
- [ ] **Claude Code, Codex, Antigravity and Pi.** Sign-in, start from the app and from the editor, alignment of instructions and skills among the four.
- [ ] **An agent's terminal.** Home or Projects › the Claude Code button: a tab of the app's terminal panel opens (no console window, no Windows Terminal, even if it is the PC's default terminal). Claude Code receives Enter, arrows, Esc and Ctrl+C. Closing the tab leaves no `claude.exe` nor `OpenConsole.exe`.
- [ ] **The kit's ConPTY.** In Task Manager the session's host is `OpenConsole.exe` from `data\apps\win32-*\conpty\`, never `conhost.exe`. With the component removed and no network, «Terminale» says in the app that it cannot install it; with the network, it is downloaded first (progress in «Attività») and the terminal opens.
- [ ] **Git when needed.** On a kit without Git, at startup only the system components are installed: GitHub CLI works without Git (D-81). Choosing Claude Code, cloning a project or starting an action with `git` installs Git; unchecking Git also removes Claude Code.
- [ ] **Editor updated with a folder open.** With File Explorer open on `data\apps\win32-x64\vscodium-server\extensions`, the editor of another version is still replaced at startup (D-60).

- [ ] **Usage limits.** Signed in to Claude Code and Codex, Agents shows the same values as `/usage` and `/status`, with the time they reset. Without a sign-in “not signed in” appears. Without a network the error is readable and the page stays usable. “Refresh usage” pressed several times in a row makes no more than one request every 20 s.
- [ ] **Only in the project.** In Home and Projects, Editor, Terminal and agent open in the project folder; there is no button for `projects\`. In the editor, File › Open Folder on another folder, “New Window” and “Close Folder” do not leave the project.
- [ ] **Trust in the editor.** The first time a project is opened, the editor asks whether to trust it. With “No” the “Restricted Mode” bar appears and tasks do not start; with “Yes” it no longer asks when reopening, even after restarting the app (D-67).
- [ ] **dev-settings.json.** After opening Codex in a project, `data\projects\<project>\dev-settings.json` has `"agent": "codex"` and the agent choice suggests Codex. Deleting an empty project from the app makes its folder in `data\projects\` disappear (D-89).

### Local models and Pi (D-151, D-152)
Verified by hand on Windows x64 (with an RTX 2070, Vulkan): installation of llama.cpp and of Pi 1.4.0 with Git, a download interrupted and resumed (0.5 GB, and 4.7 GB in two parts), start, an answer with the key and 401 without it, stop together with the models' processes, `sync`, Pi in `--print` from a project on the local model with the bash tool. **Not yet verified by hand**: macOS (Metal) and Linux (Vulkan, and the CPU without `libvulkan1`), Windows ARM64 and Linux ARM64, a PC without a GPU.
- [ ] **Download interrupted and resumed.** Local models › a catalog model › “Download”, then “Pause” (or close Atollo, or pull the network): `<file>.part` stays in `data\models\gguf\<owner>--<repo>\`. “Resume” (or `Atollo.exe models pull <id>` after a restart) continues from where it stopped, not from zero; at the end the SHA-256 matches and `atollo.json` has `complete`. A file altered by hand in `.part` is deleted at the end, with the error.
- [ ] **Model in several parts.** Qwen2.5-Coder 7B downloads both parts; “Delete” removes both and the folder.
- [ ] **FAT32 refusal.** With the kit on a FAT32 stick, a model with a file over 4 GiB (Qwen3.5 9B) is refused before the first byte, with what to do; Qwen2.5-Coder 7B (two parts under 4 GiB) downloads. A model larger than the free space is refused the same way.
- [ ] **Hugging Face.** “Search the GGUF files” on a public repository lists the quantizations with their size; a gated repository without a token says that a token is needed; with the token saved (`data\app\secrets\hf-token`, owner-only) it downloads.
- [ ] **Server with its key.** “Start”: `llama-server` listens on `127.0.0.1` only, on 9931 or the next free port; `curl http://127.0.0.1:<port>/v1/models` without a key answers 401, with `Authorization: Bearer <key>` (“Copy the key”) the list of models. The key is not on the command line of `llama-server` (Task Manager › Command line). `Atollo.exe llm status` says whether it runs without sending the key.
- [ ] **Chat in the browser.** “Open the chat”: llama.cpp's page asks for the key in its settings; with the copied key it answers.
- [ ] **Stopping.** The server is in Activity and in the status bar's count; “Stop everything”, “Remove drive” and closing Atollo stop it with its models' processes (none left in Task Manager). Deleting a model while the server runs stops it first, then the file goes away.
- [ ] **Pi from a project.** With a tool-using model downloaded (Qwen3.5 9B), the Pi button of a project opens Pi in the project folder on the local model with no other setting; asked to list the files, it runs the bash tool with the kit's Git Bash. `pi` in a kit terminal works too. With a cloud provider signed in via `/login`, Pi uses it and the local model stays available.
- [ ] **Pi's environment.** In `data\home\.pi\agent\`: `models.json` with the `atollo` provider (owner-only), `settings.json` with the kit's Git Bash as `shellPath`, `APPEND_SYSTEM.md` with the kit's block (the user's text kept), the project in `trust.json`. No update notice from Pi (`PI_SKIP_VERSION_CHECK=1`).

### Ready MCP servers (D-156)
Verified by hand on Windows x64: Claude Code 2.1.306 expands `${VAR}` in the headers of a user-level server (a local echo server received `Bearer <value>`); DBHub reads `DSN` from the environment. **Not yet verified by hand**: Antigravity and Pi expanding `${VAR}`, GitHub's remote server with the `gh` CLI's token through the kit's bridge (`atollo-mcp github`), mongodb-mcp-server end to end.
- [ ] **GitHub.** With the kit's GitHub account signed in, Agents › “The kit's MCP servers” › “Ready servers” › “Add” beside github: `data\agents\mcp.json` has `cmd /c atollo-mcp github` (`atollo-mcp github` on macOS and Linux) with `ATOLLO_EXE: ${ATOLLO_EXE}`, and no token. Claude Code started from the app, asked to list the issues of one of your repositories, lists them with the github server; the same with Codex, Antigravity and Pi.
- [ ] **The token is not in the agent's environment** (SEC-V7-01). With the github server in the kit, in Claude Code started from the app `echo $GITHUB_PERSONAL_ACCESS_TOKEN` (or `set GITHUB_PERSONAL_ACCESS_TOKEN` in cmd) prints nothing, and a test stdio server in a project's `.mcp.json` that writes its environment to a file finds no token there.
- [ ] **Playwright.** After adding playwright, an agent started from the app, asked to open a page, opens a browser (Node is installed first if missing).
- [ ] **Filesystem limited to the project.** Project › Editor › “Project MCP servers” › “Add” beside filesystem: `.mcp.json` has `npx -y @modelcontextprotocol/server-filesystem .`; the agent reads and writes the project's files with it, and a path outside the project folder is refused.
- [ ] **MySQL.** mysql is not offered until the project's database exists; after “Create database and user” with MySQL running, “Add” beside mysql writes `DSN: ${DSN}` in `.mcp.json` (no password); the agent started from the app lists the tables of the project's database. The same for MongoDB with mongodb.
- [ ] **No token for a cloned repository.** A cloned repository whose `.mcp.json` has a server asking for `${GITHUB_PERSONAL_ACCESS_TOKEN}` (for example in `env` or a header) gets nothing: even with the github server in the kit's `mcp.json`, that server receives an empty or unexpanded value, never the token.
- [ ] **Codex.** In a kit terminal `codex mcp list` shows the kit's servers (github with `env_vars` `ATOLLO_EXE`, playwright, fetch) and, in a project, the project's ones (mysql with `env_vars`).
- [ ] **At once, and by hand.** “Remove” on a kit server takes it away from Claude Code's `.claude.json`, Codex's `config.toml`, Antigravity's `mcp_config.json` and Pi's `mcp.json` without restarting the app. A `data\agents\mcp.json` that is not valid JSON shows the notice and is not rewritten.

### Command palette (D-160)
**Not yet tried by hand**: that WebView2 (Windows), WKWebView (macOS) and WebKitGTK (Linux) leave Ctrl+Shift+P to the page instead of using it themselves (the handler calls `preventDefault`).
- [ ] **From Projects.** Ctrl+Shift+P (⌘⇧P on macOS) on the Projects page opens “Commands” with the box “What do you want to do? Start, stop, open, a project, an agent…”; projects are newest first, “Go to” lists every page but Projects.
- [ ] **From another page.** In Settings (or Tools) Ctrl+Shift+P opens the same dialog; “Go to Settings” is not there, “Go to Projects” is. “go settings” typed on another page finds “Go to Settings” (the words in any order, any case).
- [ ] **From a terminal.** With the cursor in a kit terminal (the Terminals page or the editor's terminal in the page), Ctrl+Shift+P opens “Commands” and the terminal receives nothing; Ctrl+P in the terminal stays the shell's (in PowerShell or Bash, the previous history line).
- [ ] **Starting and stopping a project.** “Start <project>” starts its default action (it appears in Activity); reopening the palette, the entry is “Stop <project>” and stops it. A start error (for example a port in use) appears under the title bar, with the X that closes it.
- [ ] **Opening.** “Open <project>” goes to its page; “Open <project> in the editor” (only with the editor installed) opens the editor in its folder.
- [ ] **An agent in the project.** “Claude Code in <project>” (or another installed agent) opens the agent in the project's folder, as the project page's button does; an agent not installed has no entries.
- [ ] **Databases.** With MySQL installed, “Start MySQL” starts it and the entry becomes “Stop MySQL”; the same for MongoDB. A server not installed has no entries.
- [ ] **Stop everything.** With an action, a database and a pool running, “Stop everything” (there only then) stops them without asking for confirmation; the Activity panel is left empty. With nothing running the entry is not there.
- [ ] **Closing.** Esc, or a click outside the dialog, closes without doing anything; ↑/↓ move the choice, Enter runs the highlighted one.
- [ ] **Ctrl+P unchanged.** Ctrl+P still opens “Open a project” as before: Enter opens the page, Shift+Enter starts the default action, Esc closes.

### Tools
- [ ] **Kinds** (D-139). “All tools” has the blocks System and editor, AI agents, Local models, Languages, Development tools, Web server, Databases, each with its title, count, description and a box to choose them all, and closes with a click; the same kinds in the side bar's tree, in the matrix and at the first start.
- [ ] **Logos.** Every component has its logo, readable in the light theme and in the dark one; the projects show the language labels.
- [ ] **Dependencies.** Apache shows the “PHP” dependency; with Claude Code chosen, Git shows “for Claude Code”. Removing PHP also removes Apache.
- [ ] **PATH after an installation.** After installing a language from this page, Home › Terminal opens the kit console with the kit's PowerShell 7 and the new program is in the PATH, without restarting the app.
- [ ] **Check the files** (D-159). After installing a component (e.g. `gh`) its folder has `.atollo-manifest` (header `# Atollo manifest 1: sha256 size path`); “Check the files” on the toolbar shows “No damaged file: N folders checked.”
- [ ] **Files: damage found.** Flip one byte of a file in the component's folder (e.g. `bin\gh.exe`) and delete another: “Check the files” lists the component with “1 files changed, 1 missing” and the paths; with two damaged components “Reinstall all” appears too.
- [ ] **Files: Reinstall.** “Reinstall” on the damaged row prepares the component again at the same version (the projects and the databases stay as they were); “Check the files” is fine again.
- [ ] **Files: no manifest.** A component installed with a version before D-159 (or with `.atollo-manifest` deleted) appears in “Without a manifest, installed before Atollo kept one: …”; after its update it is checked.
- [ ] **Files: not damage.** `pip install requests` in the kit's Python, a gem in Ruby, an edit of Apache's `httpd.conf` and the logs of a running server: “Check the files” stays fine.
- [ ] **Files: command line.** `Atollo.exe check --files | Out-Host` prints `√`, `x` or `-` per folder, the damaged paths under the `x` lines; `$LASTEXITCODE` is 0 when all is fine, 1 with a damaged folder.
- [ ] **Updates: daily check** (D-161). With Settings › General › “Automatic update” on, a component installed behind its source (e.g. `gh` installed as an older release in a test kit) and no `data\app\updates.json`: about 10 seconds after a start the “Updates” pane of Tools fills (“2.100.0 → 2.103.0”), its title and the Tools icon in the activity bar show the count (“New versions: 1” in the tooltip), `updates.json` has `checked` and the versions. A second start the same day asks nothing (`checked` unchanged).
- [ ] **Updates: off.** With “Automatic update” off (or `ATOLLO_NO_UPDATE` set) and `updates.json` deleted, after a start nothing is asked: the pane says “Not checked yet…”, no badge, no `updates.json`.
- [ ] **Updates: Check now.** The circular arrow “Check now” on the pane's title asks at once (also with “Automatic update” off): the list and “Checked on <date>” are updated; with everything current it says “Everything up to date.”.
- [ ] **Updates: What changes.** “What changes” opens the release notes in the PC's browser (for `gh`, `https://github.com/cli/cli/releases/tag/v<new version>`); for a component without notes in the catalog, its site.
- [ ] **Updates: Update.** The pane's “Update” updates only that component (like the row's button); when the preparation ends the item disappears from the pane and the badge count goes down.
- [ ] **Official site.** On a tool's page “Official site” now opens the browser at the component's site (before D-161 it did nothing).

### Terminals
- [ ] **Replaced version.** A terminal saved with PHP 8.2, then 8.2 removed: the Terminals page says which one it uses instead (“PHP 8.2 → 8.4”) and the terminal opens with 8.4.
- [ ] **Deletion.** “Delete the terminal” asks first; Esc closes the question and the terminal stays.

### Settings
- [ ] **No Save** (D-137). A change is saved and applied on its own half a second after the last key: the tab bar says “Saving…”, then “Saved.”; `data\app\settings.json` has the new value.
- [ ] **Language.** Switching to English, Français and Deutsch, the whole interface changes immediately. With “System language” the system's language applies.
- [ ] **Theme.** “Same as the system”, “Light” and “Dark” change the app, the editor and the terminals at once, without a restart.
- [ ] **Gemini key.** It is saved when the field loses the focus, then the field empties and shows “saved”; it is in `data\app\secrets\gemini-api-key`, never in `settings.json`. It never appears in plain text (neither in the interface nor in `data\app\logs`). “Remove the key” removes it.
- [ ] **Invalid value** (proxy `proxy:8080`, a PEM file without certificates): the error is readable, the value stays on screen and the file does not change.
- [ ] **Editor running.** After a change of language, network or temporary files of programs, the app asks to restart the editor, and after the restart the editor's terminals see the new variables.
- [ ] **Privacy.** The section summarizes it and “Read the privacy policy” opens it in the app's language.
- [ ] **Update.** With a newer `app-v*` release published, “Check for updates” downloads the exe. On closing, `Atollo.exe` is the new one and `Atollo.old.exe` disappears on restart. With `ATOLLO_NO_UPDATE=1` no check runs at startup.
- [ ] **Export diagnostics** (D-153). Settings › The kit › “Export diagnostics” shows the path of `data\app\diagnostics\atollo-diagnostica-<yyyymmdd-hhmmss>.zip` and “Open folder” opens that folder; `Atollo.exe diagnostics | Out-Host` makes another one. The zip has `summary.txt`, `env.txt`, `check.txt`, `components.txt`, `ports.txt` and `logs\*.log`. Searching all its files: neither the key of `data\app\secrets\llama-key`, nor the Gemini key, nor a database password, nor the agents' tokens; neither the Windows user name nor the PC's name (`USERNAME=***`, `COMPUTERNAME=***` in `env.txt`); the profile folder appears as `~`.
- [ ] **Report a problem** (D-153). Settings › The kit › “Report a problem” opens `github.com/nabre/atollo/issues/new` in the PC's browser with the bug form, “Atollo version” and “System” already filled in (kit's system, Windows build, file system and kind of the disk). Nothing is created until “Create” is pressed on GitHub; the diagnostics zip can be dragged into the issue.
- [ ] **Kit health** (D-155). Settings › The kit › “Kit health” shows one line each for drive, space, path and the kit's files (on Linux the libraries too), all green on an NTFS kit in a plain folder; `Atollo.exe check --kit | Out-Host` prints the same lines and `$LASTEXITCODE` is 0. “Check again” runs the checks again.
- [ ] **Health: FAT32 stick.** With the kit on a FAT32 stick the drive line is a warning (files up to 4 GB, no links) that suggests exFAT or NTFS; on exFAT the line is fine on Windows.
- [ ] **Health: low space.** With less than 5 GB free on the kit's drive the space line is a warning; under 1 GB it is an error with “Free up space…” and `check --kit` ends with 1.
- [ ] **Health: risky path.** With the kit in a folder like `D:\R&D\Atollo` or `D:\Città\Atollo` the path line is a warning that lists “&” or “à”.
- [ ] **Health: files of another account.** A file of `data\home` made readable only by another account (Properties › Security, or copied from another PC with its permissions) gives an error with how many files, the first one and the command `takeown … && icacls … /reset /t`; once run as administrator, “Check again” is green again. On macOS and Linux the same with `sudo chown root` and the command `sudo chown -R …`.
- [ ] **Health: Linux without libaio.** On a Linux without `libaio` (or without `libnuma`, `libicu`, `libvulkan1`) the libraries line is a warning that names them, with `sudo apt install …` of the missing packages; once installed, “Check again” is fine. With the kit on a drive mounted `noexec` the drive line is an error with the command to remount it.

### Copy of the kit (D-163)
- [ ] **Export to a folder with MySQL running.** With MySQL started by hand and MongoDB marked “Start with Atollo”: Settings › The kit › “Copy of the kit” › “Export to a folder…” and choose a folder on another drive. MySQL and MongoDB stop during the copy; at the end MongoDB is running again and MySQL stays stopped. The page says “Copy made: … files, …” with the path; `<folder>\Atollo-<yyyymmdd-hhmmss>` has `projects\`, `data\` and `atollo-backup.json`, and no `data\apps`, `data\cache`, `data\models`, `data\webview2`, `data\app\logs`. A project's running action did not stop.
- [ ] **Export to a zip.** “Export to a zip…”: `Atollo-<yyyymmdd-hhmmss>.zip` opens in Explorer with the same content; while it is written there is a `.zip.part`, never a half zip with the final name.
- [ ] **Copy into the kit refused.** Choosing the kit's own folder (or one inside it) for an export gives an error and writes nothing; the same choosing a copy inside the kit for an import.
- [ ] **A zip not from Atollo refused.** “Import from a zip…” with any other zip (without `atollo-backup.json`, or with a path outside `projects/` and `data/`) gives an error; no `.atollo-import` folder is left in the kit.
- [ ] **Cancel.** Import a valid copy: “Copy ready: … files, …, made on …” with “Close Atollo to import it”, “Cancel the import”, “Open the kit's folder” and “Then open Atollo again…”. “Cancel the import” removes `.atollo-import\`; reopening Atollo changes nothing.
- [ ] **Import into another kit on another drive.** Copy `Atollo.exe` into an empty folder on another drive, open it, let the setup finish, then import the zip of the first kit and “Close Atollo to import it”; reopen. The kit starts normally, the log has “kit copy imported; what it replaced: …”, the previous projects and data are in `.atollo-before-import-<stamp>\`, the kit says it was moved, the programs already downloaded are kept and the missing ones are downloaded again. The projects open and start; their databases (MySQL, MongoDB) start and have their tables and users.
- [ ] **Failure at start.** With a file of `projects\` held open by another program during the swap, Atollo stops with “Importazione della copia del kit non riuscita: …”; nothing is deleted: `projects\` is where it was and the copy is still in `.atollo-import\`. Closing the other program and reopening Atollo completes the import.

### Closing, other copies, removing the drive
- [ ] **Closing with something open.** With the editor, a console or a started project, closing the window asks for confirmation and lists what will close. “Cancel” (Esc too) leaves everything open.
- [ ] **Two copies of the kit open.** The second one's Home shows the warning with the other one's folder.
- [ ] **Remove drive.** Everything closes and the Antigravity sign-in goes back into the kit. After “Close Atollo” the USB stick ejects without “device in use”.

### Title bar
- [ ] **Main window without the Windows frame.** Dragging from the bar, double-click to maximize, Minimize, Maximize/Restore, Close (with the confirmation if something is open); resizing from the edges; Win+arrows work. The buttons are reachable with Tab and have a name for Narrator.
- [ ] **Databases** (D-73). Databases page: “Start” on a MySQL that is not installed installs and starts it; `mysql -uroot -h127.0.0.1` and `mongosh` answer; port in use: message and port change; “Start with Atollo” makes it start when the app restarts; closing the app: the confirmation counts the databases and the logs end with “Shutdown complete” / “Now exiting”. A Laravel project with `DB_CONNECTION=mysql` gets SQLTools.
- [ ] **Database pages** (D-150). Databases page with MongoDB started: only its details, no tabs; its content opens with “Open mongo-express”, MySQL's with “Open phpMyAdmin”. SQLite is not in the list.
- [ ] **Database passwords** (D-74). Databases page: root/admin password hidden, “Show” and “Copy”. `mysql --no-defaults -uroot -h127.0.0.1` refused; `mysql` in the kit's terminal gets in. Project page › Databases: “Create” and then “Write to .env” first shows the lines that change.
- [ ] **Project settings** (D-78). Advanced: “New action” with a `NOME=valore` variable appears in the table and in `panel.json`; “Default” puts it on the Start button; “Remove” removes it. Editor: format on save and indentation written in `.vscode\settings.json`, apply when opened in the editor. MCP: an added server appears in `.mcp.json` and in `.codex\config.toml`; `codex mcp list` in the project lists it. Resources › database: “Export” creates the backup, “Restore…” asks for confirmation and brings the data back.
- [ ] **Six-page structure** (D-76). Projects is the first page; “New project” with the three paths (the clone from the repositories asks for the GitHub sign-in in Settings); project page with Summary, Resources, Advanced; Activity panel: start a project, open the panel, see the output, Open in the browser, Stop; installation in progress in the panel; database with an error (port in use) in the panel.
- [ ] **Project page** (D-75). From the name in Projects and in Home; the six tabs can also be chosen with the arrows; Run starts and stops as before; Databases creates the database and writes the `.env` after the preview; Git shows branch and remote of a project cloned from GitHub.
- [ ] **Branding** (D-72). App: logo in the title bar and in Home, Atollo green accent in the dark and light themes, button text readable. Editor: title “<project> - Atollo”, logo in the bar, welcome with logo and project in the empty editor, in the light theme too.
- [ ] **Extensions while working** (D-71). Editor open on a project without Ruby: `app.rb` is created; within about 20 s (plus the download) the notice with Ruby LSP and Ruby Debugger appears; “Restart extensions” reloads the editor on the project, “Later” closes the notice.
- [ ] **Editor window with the VSCodium bar** (D-69). File…Help menu in the bar; dragging from the empty areas, double-click to maximize, Aero Snap; Minimize, Maximize/Restore (the icon changes with Win+arrows too), Close; resizing from the edges; light and dark theme. With the editor server stopped the fallback bar appears, which moves and closes the window.

### Accessibility
- [ ] **Keyboard only.** Every page can be used: Tab always shows the focus outline, “Skip to content” is the first element, the closing dialog receives the focus.
- [ ] **In-page confirmations** (D-106). Signing out of an agent, of GitHub, restoring a database and “Apply” in Tools: Narrator reads the question, the focus is on “Cancel”, Esc closes and the focus goes back to the button you started from. The zoom of a Guide screenshot keeps Tab inside and closes with Esc.
- [ ] **Narrator.** The pages have headings and the icons have a name. Errors and states are announced (`role="alert"` and `role="status"`).
- [ ] **Same look as the editor** (D-88, D-142). App and editor side by side, in the dark and in the light theme: same colors for background, text, focus, buttons, fields and drop-down menus; the same measures as VSCodium 1.135's modern UI (title bar 30 px, activity bar 36 px, editor tabs 24 px in a 32 px bar with the close button on hover, pill-shaped panel tabs, list rows set in 4 px with a radius of 4, 8 px scroll bars, dialogs with a radius of 12); selects, fields and checkboxes can be used with mouse and keyboard; the codicon icons show (no little squares).
- [ ] **Theme and animations.** With the theme on “Same as the system” the app follows the system's light and dark. With “Animation effects” turned off in Windows, no icon spins.
