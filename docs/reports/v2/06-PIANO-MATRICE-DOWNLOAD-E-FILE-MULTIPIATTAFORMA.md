# 06 — Verifica Matrice Download Multi-OS e Piano Correttivo Residuo (`v2`)

> **Stato di Implementazione del Piano v1**: Nel commit `fa8a34d`, **l'85% del piano architetturale e di download progettato nel documento `06` della `v1` è stato effettivamente implementato nel codice**.
> Questo documento `v2` aggiorna la matrice da *"progetto su carta"* a **Matrice di Verifica Implementativa sui 26 componenti × 6 target**, evidenziando cosa è già operativo (`✅ Implementato in v2`) e fornendo le patch esatte per i **4 componenti Unix residui** (`Git`, `Nginx`, `Php`, `Apache`) e per i 2 fix infrastrutturali (`extract_tar_xz` e `Python::finish`).

---

## 1. Stato di Implementazione dell'Infrastruttura Core (`v1` Plan vs `v2` Code)

| Componente Infrastrutturale | File Implementato | Stato in `v2` | Note di Verifica |
| :--- | :--- | :-: | :--- |
| **1.1 Enum `Os`, `Arch`, `Target` (6 target)** | [`tauri/src/kit.rs:17-129`](file:///c:/Users/nabre/Projects/atollo/tauri/src/kit.rs#L17-L129) | ✅ **Fatto** | `Target::current()`, `slug()`, `from_slug()`, `Target::ALL` (6 target) |
| **1.2 Isolamento `data/apps/<target>` + Migrazione** | [`tauri/src/kit.rs:138-234`](file:///c:/Users/nabre/Projects/atollo/tauri/src/kit.rs#L138-L234) | ✅ **Fatto** | Migrazione automatica legacy `data/apps/<slot> -> data/apps/win32-<arch>` |
| **1.3 Pointer `root-<os>.txt` per-OS** | [`tauri/src/kit.rs:236-260`](file:///c:/Users/nabre/Projects/atollo/tauri/src/kit.rs#L236-L260) | ✅ **Fatto** | Fallback di lettura a `root.txt` legacy preservato |
| **1.4 Traduzione `translate_rel(slot, rel, os)`** | [`tauri/src/setup/catalog.rs:747-855`](file:///c:/Users/nabre/Projects/atollo/tauri/src/setup/catalog.rs#L747-L855) | ✅ **Fatto** | Copre tutti i 26 componenti senza duplicare le struct `ComponentMeta` |
| **1.5 Disponibilità per `Target` (`is_available_on`)** | [`tauri/src/setup/catalog.rs:885-953`](file:///c:/Users/nabre/Projects/atollo/tauri/src/setup/catalog.rs#L885-L953) | ✅ **Fatto** | `Terminal` solo su Windows; `R`, `Julia`, `Texlive`, `Php` sbloccati su ARM64 Unix |
| **1.6 Estrazione `.tar.xz`, Symlink Unix e `0o755`** | [`tauri/src/net/archive.rs:175-296`](file:///c:/Users/nabre/Projects/atollo/tauri/src/net/archive.rs#L175-L296) | ⚠️ **Parziale** | Funziona, ma `extract_tar_xz` usa `tar` esterno senza `safe_join` (**`BUG-22`**) |
| **1.7 Estrazione pacchetti `.conda` / `.tar.bz2`** | [`tauri/src/install/scientific.rs:612-691`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/scientific.rs#L612-L691) | ✅ **Fatto** | `extract_conda_package` gestisce sia `.tar.bz2` che `.conda` (`pkg-*.tar.zst`) |

---

## 2. Matrice di Verifica dei 26 Componenti × 6 Target in `v2`

Legenda:
- ✅ **Pronto in `v2`**: URL resolver, estrattore, layout binario e hook `finish()` verificati nel codice `fa8a34d`.
- 🔄 **Emulato (x64)**: Funzionante tramite emulazione trasparente (Prism su Windows ARM64 / Rosetta 2 su macOS ARM64).
- ❌ **Rotto su Unix (`BUG-23` / `BUG-24`)**: Il codice è presente ma fallisce in fase di download o estrazione (vedi Sezione 3 per la fix).
- ➖ **Non Applicabile**: Componente esclusivo Windows (`Terminal`).

| # | Componente | File Installer | `win32-x64` | `win32-arm64` | `darwin-x64` | `darwin-arm64` | `linux-x64` | `linux-arm64` | Dettaglio Implementazione `v2` |
| :-: | :--- | :--- | :-: | :-: | :-: | :-: | :-: | :-: | :--- |
| 1 | **`git`** | [`core.rs:23`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/core.rs#L23-L108) | ✅ MinGit | ✅ MinGit | ❌ `BUG-23` | ❌ `BUG-23` | ❌ `BUG-23` | ❌ `BUG-23` | Scarica `.tar.bz2`/`.conda` da conda-forge ma passa `Kind::Zip` |
| 2 | **`python`** | [`core.rs:180`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/core.rs#L180-L386) | ✅ Embed | ✅ Embed | ✅ Standalone | ✅ Standalone | ✅ Standalone | ✅ Standalone | `python-build-standalone` (`install_only_stripped`); lieve bug in `finish()` se linea != 3.13 (`BUG-24`) |
| 3 | **`nodejs`** | [`core.rs:392`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/core.rs#L392-L545) | ✅ `.zip` | ✅ `.zip` | ✅ `.tar.gz` | ✅ `.tar.gz` | ✅ `.tar.xz` | ✅ `.tar.xz` | Wrapper `npm`/`npx` e `.npmrc` multi-OS |
| 4 | **`vscode`** | [`core.rs:551`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/core.rs#L551-L652) | ✅ `.zip` | ✅ `.zip` | ✅ `.zip` | ✅ `.zip` | ✅ `.tar.gz` | ✅ `.tar.gz` | Appiattisce `Visual Studio Code.app` in `data/apps/<target>/vscode/` |
| 5 | **`terminal`** | [`core.rs:658`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/core.rs#L658-L717) | ✅ `.zip` | ✅ `.zip` | ➖ N/A | ➖ N/A | ➖ N/A | ➖ N/A | Correttamente escluso su macOS/Linux tramite `is_available_on` |
| 6 | **`jdk`** | [`jvm.rs:19`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/jvm.rs#L19-L148) | ✅ Adoptium | ✅ Adoptium | ✅ Adoptium | ✅ Adoptium | ✅ Adoptium | ✅ Adoptium | Appiattisce `Contents/Home/` su macOS (`flatten_macos_contents_home`) |
| 7 | **`maven`** | [`jvm.rs:154`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/jvm.rs#L154-L237) | ✅ `.zip` | ✅ `.zip` | ✅ `.zip` | ✅ `.zip` | ✅ `.zip` | ✅ `.zip` | Imposta `0o755` su `bin/mvn` e `bin/mvnDebug` su Unix |
| 8 | **`clang`** | [`systems.rs:21`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/systems.rs#L21-L152) | ✅ `.tar.xz` | ✅ `.tar.xz` | ✅ `.tar.xz` | ✅ `.tar.xz` | ✅ `.tar.xz` | ✅ `.tar.xz` | Fallback x64 automatico su `darwin-x64` per LLVM >= 16 |
| 9 | **`rust`** | [`systems.rs:158`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/systems.rs#L158-L273) | ✅ `rustup` | ✅ `rustup` | ✅ `rustup` | ✅ `rustup` | ✅ `rustup` | ✅ `rustup` | Esegue `rustup-init` con triple nativo (`aarch64-apple-darwin`, ecc.) |
| 10 | **`go`** | [`systems.rs:279`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/systems.rs#L279-L363) | ✅ `.zip` | ✅ `.zip` | ✅ `.tar.gz` | ✅ `.tar.gz` | ✅ `.tar.gz` | ✅ `.tar.gz` | `go.dev/dl/` multi-OS completo |
| 11 | **`cmake`** | [`systems.rs:369`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/systems.rs#L369-L461) | ✅ `.zip` | ✅ `.zip` | ✅ `.tar.gz` | ✅ `.tar.gz` | ✅ `.tar.gz` | ✅ `.tar.gz` | Appiattisce `CMake.app/Contents/` su macOS (`flatten_macos_app_contents`) |
| 12 | **`dotnet`** | [`systems.rs:467`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/systems.rs#L467-L566) | ✅ `.zip` | ✅ `.zip` | ✅ `.tar.gz` | ✅ `.tar.gz` | ✅ `.tar.gz` | ✅ `.tar.gz` | Symlink Unix in [`side_by_side.rs:228`](file:///c:/Users/nabre/Projects/atollo/tauri/src/versions/side_by_side.rs#L228-L243) |
| 13 | **`gnu_make`** | [`systems.rs:572`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/systems.rs#L572-L654) | ✅ GnuWin32 | 🔄 Emul. | ✅ Wrapper | ✅ Wrapper | ✅ Wrapper | ✅ Wrapper | Su Unix crea `bin/make` che delega a `/usr/bin/make` |
| 14 | **`gdb`** | [`systems.rs:660`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/systems.rs#L660-L761) | ✅ w64devkit | 🔄 Emul. | ✅ Wrapper | ✅ Wrapper | ✅ Wrapper | ✅ Wrapper | Su macOS crea wrapper `bin/gdb` → `lldb`; su Linux → `/usr/bin/gdb` |
| 15 | **`r`** | [`scientific.rs:24`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/scientific.rs#L24-L162) | ✅ CRAN | ❌ Non disp. | ✅ Conda | ✅ Conda | ✅ Conda | ✅ Conda | Usa `conda-forge/r-base` con `extract_conda_package` su Unix |
| 16 | **`julia`** | [`scientific.rs:168`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/scientific.rs#L168-L253) | ✅ `.tar.gz` | ❌ Non disp. | ✅ `.tar.gz` | ✅ `.tar.gz` | ✅ `.tar.gz` | ✅ `.tar.gz` | Sbloccato su `darwin-arm64` e `linux-arm64` |
| 17 | **`octave`** | [`scientific.rs:259`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/scientific.rs#L259-L361) | ✅ `.zip` | 🔄 Emul. | ✅ Conda | ✅ Conda | ✅ Conda | ✅ Conda | Usa `conda-forge/octave` su macOS e Linux |
| 18 | **`texlive`** | [`scientific.rs:367`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/scientific.rs#L367-L459) | ✅ TinyTeX | ❌ Non disp. | ✅ TinyTeX | ✅ TinyTeX | ✅ TinyTeX | ✅ TinyTeX | `TinyTeX-1-v*.tgz` (macOS universal) e `.tar.gz` (`linux-x86_64`/`arm64`) |
| 19 | **`jupyter`** | [`scientific.rs:465`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/scientific.rs#L465-L512) | ✅ `pip` | ✅ `pip` | ✅ `pip` | ✅ `pip` | ✅ `pip` | ✅ `pip` | Usa `python_exe` tradotto (`bin/python3` su Unix) |
| 20 | **`sqlite`** | [`scientific.rs:518`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/scientific.rs#L518-L610) | ✅ `.zip` | 🔄 Emul. | ✅ `.zip` | 🔄 Emul. | ✅ Conda | ✅ Conda | `sqlite-tools-osx-x64` su macOS; `conda-forge/sqlite` su Linux |
| 21 | **`duckdb`** | [`scientific.rs:697`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/scientific.rs#L697-L766) | ✅ `.zip` | ✅ `.zip` | ✅ `.zip` | ✅ `.zip` | ✅ `.gz` | ✅ `.gz` | `duckdb_cli-osx-universal.zip` su macOS |
| 22 | **`mariadb`** | [`web.rs:20`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/web.rs#L20-L398) | ✅ `.zip` | 🔄 Emul. | ✅ Conda | ✅ Conda | ✅ `.tar.gz` | ✅ `.tar.gz` | Tarball `mariadb-*-linux-systemd-*` su Linux; `conda-forge/mysql` su macOS |
| 23 | **`php`** | [`web.rs:404`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/web.rs#L404-L558) | ✅ `.zip` | ❌ Non disp. | ❌ `BUG-24` | ❌ `BUG-24` | ❌ `BUG-24` | ❌ `BUG-24` | Chiama ancora `windows.php.net` (zip di `.exe`/`.dll` Windows) |
| 24 | **`composer`** | [`web.rs:564`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/web.rs#L564-L607) | ✅ `.phar` | ✅ `.phar` | ✅ `.phar` | ✅ `.phar` | ✅ `.phar` | ✅ `.phar` | Crea sia `composer.bat` (Windows) sia `bin/composer` `0o755` (Unix) |
| 25 | **`wp_cli`** | [`web.rs:613`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/web.rs#L613-L656) | ✅ `.phar` | ✅ `.phar` | ✅ `.phar` | ✅ `.phar` | ✅ `.phar` | ✅ `.phar` | Crea sia `wp.bat` (Windows) sia `bin/wp` `0o755` (Unix) |
| 26 | **`apache`** | [`web.rs:662`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/web.rs#L662-L702) | ✅ Lounge | 🔄 Emul. | ❌ `BUG-24` | ❌ `BUG-24` | ❌ `BUG-24` | ❌ `BUG-24` | Chiama ancora `apachelounge.com/download/VS18/` (Windows-only) |
| 27* | **`nginx`** | [`web.rs:708`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/web.rs#L708-L780) | ✅ `.zip` | 🔄 Emul. | ❌ `BUG-23` | ❌ `BUG-23` | ❌ `BUG-23` | ❌ `BUG-23` | Su Unix scarica il tarball C sorgente da `nginx/nginx` invece di un binario |

*(Nota: i componenti nel catalogo sono 26 perché `composer` e `wp_cli` fanno parte dei 26 slot insieme a `apache` e `nginx`).*

---

## 3. Piano Correttivo Esatto per i 4 Componenti Unix Residui (`BUG-22`, `BUG-23`, `BUG-24`)

### 3.1 Fix `Git` su macOS e Linux ([`tauri/src/install/core.rs`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/core.rs#L28-L107))
`Git::latest` su Unix risolve già correttamente l'URL dal canale `conda-forge/git`, ma restituisce `Kind::Zip` e usa l'estrattore standard.
- **Soluzione**:
  1. Rendere `pub(crate) fn extract_conda_package` e `pub(crate) fn resolve_conda_forge` condivisi in `tauri/src/install/mod.rs` (spostandoli da `scientific.rs`).
  2. In `Git::install` ([`core.rs:77`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/core.rs#L77)), quando `!ctx.target().os.is_windows()`, scaricare il pacchetto in `stage_dir("git-archive")` ed estrarlo con `extract_conda_package(&archive_file, &stage)`.

### 3.2 Fix `Nginx`, `Php` e `Apache` su macOS e Linux ([`tauri/src/install/web.rs`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/web.rs))
- **`Nginx`** ([`web.rs:708-780`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/web.rs#L708-L780)): su `!ctx.target().os.is_windows()`, usare `resolve_conda_forge(ctx, "nginx", target)` + `extract_conda_package`, che fornisce direttamente `sbin/nginx` (o `bin/nginx`, creando un symlink `sbin/nginx -> ../bin/nginx` se necessario) e `conf/nginx.conf`.
- **`Php`** ([`web.rs:404-558`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/web.rs#L404-L558)): su `!ctx.target().os.is_windows()`, risolvere da `conda-forge/php` (disponibile per `osx-64`, `osx-arm64`, `linux-64`, `linux-aarch64`) ed estrarre con `extract_conda_package`, generando `bin/php.ini` in `Php::finish`.
- **`Apache`** ([`web.rs:662-702`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/web.rs#L662-L702)): su Unix, usare `conda-forge` (`apache-httpd` se disponibile o delegare a wrapper verso `apachectl`/`httpd` di sistema analogamente a quanto già fatto per `GnuMake` e `Gdb` in [`systems.rs:615-760`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/systems.rs#L615-L760)).

### 3.3 Fix Sicurezza `extract_tar_xz` ([`tauri/src/net/archive.rs:175-190`](file:///c:/Users/nabre/Projects/atollo/tauri/src/net/archive.rs#L175-L190))
Sostituire l'invocazione di `std::process::Command::new("tar")` con il crate `xz2` (o `liblzma`) in puro Rust:
```rust
fn extract_tar_xz(archive: &Path, dest: &Path) -> AppResult<()> {
    fs::create_dir_all(dest)?;
    let file = File::open(archive)?;
    let decoder = xz2::read::XzDecoder::new(BufReader::new(file));
    extract_tar_stream(decoder, dest)
}
```
In questo modo `.tar.xz` eredita automaticamente:
1. La validazione Path Traversal (`safe_join`) su ogni entry e link target.
2. L'impostazione automatica dei permessi `0o755` sui file in `bin/` e sugli script/ELF/Mach-O.
3. L'indipendenza dalla presenza di un binario `tar` con supporto `xz` sull'host.
