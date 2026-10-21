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
- a real console in `conhost.exe`, in a folder with spaces;
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
- the Activity panel (`test\ui\activity.test.tsx`): a secondary database version and a pool listed and counted, "Stop all" stopping everything once confirmed; the Terminals page (`test\ui\terminals.test.tsx`): Esc closes the removal question;
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
- `real_databases_start_and_stop`: the kit's MySQL and MongoDB on free ports: start, a project's database and user (TCP access as that user), backup and restore, deletion, stop with nothing left behind (D-110); on Windows and in WSL;
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
- [ ] **Forced close from Task Manager.** No `node.exe`, editor server, kit console (`conhost.exe`, `pwsh.exe`) or project server is left.
- [ ] **ARM64.** The exe is native, the editor runs in emulation and the warning is shown.
- [ ] **Moved R** (D-104). Kit with R on `D:`, then on `E:`: after startup `lib\R\etc\x64\Makeconf` has `LOCAL_SOFT` on the new folder and `install.packages(..., type="source")` compiles.
- [ ] **macOS and Linux**: the “macOS, Linux and Windows ARM64” section below.

### Projects
- [ ] **New project** “test 1”: the folder appears, has the “Delete the empty folder” button and the deletion works.
- [ ] **Clone** `nabre/atollo` (or a private repository with `gh auth login` done in the kit's terminal): the folder has the right name. On a kit without Git, Git is installed before the clone. With an invalid address (`-x/y`, `ssh://…`) the error is readable.
- [ ] **Vite project.** “Start”, then the port 7100-7999 appears in the output. The state changes to “listening” and the button with the address opens the PC's browser. A second start reuses the same port. On a kit without Node.js, “Start” first installs Node.js (progress in the Activity panel and in the status bar) and then starts; Node.js shows as chosen in Tools.
- [ ] **Local network.** With “Let other PCs on the network reach the project servers”, another PC opens the started project with this PC's IP address; without it, it does not. The same for a simple PHP project started with `php -S`: `http://<IP of the PC>:<port>/` answers from another device, `curl -H "Host: evil.com"` still gets 421.
- [ ] **Port in use.** With a port chosen by hand that is already used by another program, the start gives a clear error and the port does not change on its own.
- [ ] **Port of another part of Atollo** (D-141). An action's port chosen by hand that is the port of a web server pool, of a database version (for example 3306) or in the editor's 7000-7099: refused, and the message names the owner (“Port 3306 already belongs to database MySQL 9.7: choose another one.”). The same for the port of a pool profile and for the port of a database server or of one of its versions; a new pool never gets one of them.
- [ ] **Static site.** A folder with only `index.html`: the kit's file server serves it, at `http://localhost:<port>` and at `http://<project>.localhost:<port>`. A `.env` file is not reachable (403).
- [ ] **Activity and “Stop all”.** With an action, a secondary MySQL version and a pool running: the status bar, the Projects badge and the Activity panel count the three and the panel lists them; “Stop all” asks first, “Cancel” changes nothing, the confirmation stops all three.
- [ ] **“Serve as a website”.** With the `public` folder and PHP: `panel.json` contains the `site` action and the other keys of the file remain.
- [ ] **“Stop”** on `npm run dev`: in Task Manager no `node.exe`/`esbuild.exe` of the project is left.
- [ ] **Electron or Tauri.** The app opens its window and the output reaches the page.

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
- [ ] **Antigravity and Claude Code** connected on a Mac (Claude with “Connect with a token”), kit taken to another Mac: already connected; nothing is left in the first one's Keychain. With the token, `claude` starts connected also from a terminal and from the kit's editor, and `Atollo env` shows `CLAUDE_CODE_OAUTH_TOKEN=***`; after “Sign out” a new terminal no longer has the variable.
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
- [ ] **Claude Code, Codex and Antigravity.** Sign-in, start from the app and from the editor, alignment of instructions and skills among the three.
- [ ] **An agent's console.** Home or Projects › Agent with Claude Code: the classic Windows console opens, even if the PC's default terminal is Windows Terminal or another one. Claude Code receives Enter, arrows and Ctrl+C. Closing the console leaves no `claude.exe`.
- [ ] **Git when needed.** On a kit without Git, at startup only the system components are installed: GitHub CLI works without Git (D-81). Choosing Claude Code, cloning a project or starting an action with `git` installs Git; unchecking Git also removes Claude Code.
- [ ] **Editor updated with a folder open.** With File Explorer open on `data\apps\win32-x64\vscodium-server\extensions`, the editor of another version is still replaced at startup (D-60).

- [ ] **Usage limits.** Signed in to Claude Code and Codex, Agents shows the same values as `/usage` and `/status`, with the time they reset. Without a sign-in “not signed in” appears. Without a network the error is readable and the page stays usable. “Refresh usage” pressed several times in a row makes no more than one request every 20 s.
- [ ] **Only in the project.** In Home and Projects, Editor, Terminal and agent open in the project folder; there is no button for `projects\`. In the editor, File › Open Folder on another folder, “New Window” and “Close Folder” do not leave the project.
- [ ] **Trust in the editor.** The first time a project is opened, the editor asks whether to trust it. With “No” the “Restricted Mode” bar appears and tasks do not start; with “Yes” it no longer asks when reopening, even after restarting the app (D-67).
- [ ] **dev-settings.json.** After opening Codex in a project, `data\projects\<project>\dev-settings.json` has `"agent": "codex"` and the agent choice suggests Codex. Deleting an empty project from the app makes its folder in `data\projects\` disappear (D-89).

### Tools
- [ ] **Kinds** (D-139). “All tools” has the blocks System and editor, AI agents, Languages, Development tools, Web server, Databases, each with its title, count, description and a box to choose them all, and closes with a click; the same kinds in the side bar's tree, in the matrix and at the first start.
- [ ] **Logos.** Every component has its logo, readable in the light theme and in the dark one; the projects show the language labels.
- [ ] **Dependencies.** Apache shows the “PHP” dependency; with Claude Code chosen, Git shows “for Claude Code”. Removing PHP also removes Apache.
- [ ] **PATH after an installation.** After installing a language from this page, Home › Terminal opens the kit console with the kit's PowerShell 7 and the new program is in the PATH, without restarting the app.

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

### Closing, other copies, removing the drive
- [ ] **Closing with something open.** With the editor, a console or a started project, closing the window asks for confirmation and lists what will close. “Cancel” (Esc too) leaves everything open.
- [ ] **Two copies of the kit open.** The second one's Home shows the warning with the other one's folder.
- [ ] **Remove drive.** Everything closes and the Antigravity sign-in goes back into the kit. After “Close Atollo” the USB stick ejects without “device in use”.

### Title bar
- [ ] **Main window without the Windows frame.** Dragging from the bar, double-click to maximize, Minimize, Maximize/Restore, Close (with the confirmation if something is open); resizing from the edges; Win+arrows work. The buttons are reachable with Tab and have a name for Narrator.
- [ ] **Databases** (D-73). Databases page: “Start” on a MySQL that is not installed installs and starts it; `mysql -uroot -h127.0.0.1` and `mongosh` answer; port in use: message and port change; “Start with Atollo” makes it start when the app restarts; closing the app: the confirmation counts the databases and the logs end with “Shutdown complete” / “Now exiting”. A Laravel project with `DB_CONNECTION=mysql` gets SQLTools.
- [ ] **Database content** (D-79). Databases page with MongoDB started (MySQL is viewed in phpMyAdmin, D-85): “Content” shows a project's database and its tables; a table with more than 100 rows pages forward and back; BLOB columns as “n bytes of binary data”. MongoDB: collections and documents with `ObjectId(…)`. Server stopped: “Start the server to see what is in it.”. SQLite is not in the list.
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
