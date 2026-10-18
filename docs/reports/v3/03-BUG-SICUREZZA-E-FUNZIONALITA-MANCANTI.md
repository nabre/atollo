# Atollo (Atollo) — Relazione Tecnica di Valutazione `v3`
## Documento 03 · Audit Bug, Sicurezza, Regressioni e Funzionalità Mancanti (con Patch Concrete)

**Punteggio Dimensione D6 (Sicurezza, Integrità e Sandboxing)**: **95 / 100** *(v1: 87/100 · v2: 87/100 · Target v3.1: 98/100)*  
**Punteggio Dimensione D3 (Correttezza Logica e Gestione Stato)**: **94 / 100** *(v1: 89/100 · v2: 86/100 · Target v3.1: 98/100)*  

---

## 1. Stato di Verifica dei 27 Bug Censiti in `v2/07` a `HEAD 39b74bc`

La tabella seguente certifica, tramite ispezione diretta del codice a `HEAD 39b74bc`, l'esito dei **27 bug** pianificati in [`reports/v2/07-PIANO-RISOLUZIONE-BUG-SCIAME.md`](file:///c:/Users/nabre/Projects/atollo/reports/v2/07-PIANO-RISOLUZIONE-BUG-SCIAME.md):

| ID Bug (`v2/07`) | Severità Originale | Stato in `HEAD 39b74bc` | Verifica sul Codice Sorgente |
|---|---|---|---|
| **CI-0** | Critica | ✅ **CHIUSO** | Formattazione `rustfmt` pulita su tutti i file Rust. |
| **N-S1** | Critica | ✅ **CHIUSO** | [`projects/alias.rs`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/alias.rs) usa `safe_path::lexical` + `no_links` e rifiuta `..` in `clean_alias_target`. |
| **N-S2** | Critica | ✅ **CHIUSO** | [`projects/apache.rs:101-132`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/apache.rs#L101-L132) e [`projects/nginx.rs:148-165`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/nginx.rs#L148-L165) usano match case-insensitive `(?i)\.php$` / `~* \.php$` e negano l'accesso (403 / 503) quando PHP non è attivo. |
| **N-S3** | Alta | ✅ **CHIUSO** | [`projects/apache.rs:139-148`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/apache.rs#L139-L148) (`hidden_deny`) e [`projects/nginx.rs:126-128`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/nginx.rs#L126-L128) bloccano `.env`, `.git/`, `.htpasswd` lasciando aperto `.well-known`. |
| **N-S4** | Alta | ✅ **CHIUSO** | [`servers/mod.rs:417-429`](file:///c:/Users/nabre/Projects/atollo/tauri/src/servers/mod.rs#L417-L429) (`site_doc_root`) usa `detect::inside_dir` e salta il sito se la cartella non è valida, senza mai ripiegare sulla radice del progetto. |
| **N-S5** | Media | ✅ **CHIUSO** | [`servers/mod.rs:354-358, 401-406`](file:///c:/Users/nabre/Projects/atollo/tauri/src/servers/mod.rs#L354-L358) inseriscono un VirtualHost di default neutro (`localhost` → `htdocs`/`html` del server) prima/dopo i vHost dei progetti nei pool condivisi. |
| **BUG-22** | Critica | ✅ **CHIUSO** | [`net/archive.rs`](file:///c:/Users/nabre/Projects/atollo/tauri/src/net/archive.rs) estrae `.tar.xz` in puro Rust con `lzma-rust2` e valida le catene di symlink con `symlink_chain_stays_in`. |
| **BUG-03** | Alta | ✅ **CHIUSO** | [`servers/mod.rs:221`](file:///c:/Users/nabre/Projects/atollo/tauri/src/servers/mod.rs#L221) serializza ogni scrittura su `servers.json` tramite `store_lock` con ordine lock rigoroso. |
| **BUG-04** | Alta | ✅ **CHIUSO** | [`servers/mod.rs:152-157, 434-468`](file:///c:/Users/nabre/Projects/atollo/tauri/src/servers/mod.rs#L152-L157) riavvia Apache in modo pulito, attende il rilascio della porta (`wait_port_released`) e riporta le ultime 5 righe di `error.log`/`stderr.log`. |
| **BUG-05r** | Media | ⚠️ **PARZIALE** | Risolto nei pool condivisi ([`servers/mod.rs:135, 161`](file:///c:/Users/nabre/Projects/atollo/tauri/src/servers/mod.rs#L135)), ma in [`projects/run.rs:515`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/run.rs#L515) il server Nginx dedicato scarta ancora `_php` (`Child`). Vedi **`BUG-V3-01`** sotto. |
| **BUG-07** | Media | ✅ **CHIUSO** | [`servers/mod.rs:36, 209-212, 531-555`](file:///c:/Users/nabre/Projects/atollo/tauri/src/servers/mod.rs#L531-L555) implementa `ListCache` con `LIST_TTL = 15s` e `generation`. |
| **GAP-RUN** | Media | ✅ **CHIUSO** | [`projects/run.rs:110-128, 676-734`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/run.rs#L676-L734) collega «Avvia» in `ProjectsPage` al pool condiviso tramite `Runner::attach_pool`. |
| **BUG-23a** | Critica | ✅ **CHIUSO** | [`install/core.rs:26-66`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/core.rs#L26-L66) installa `git` su macOS/Linux via `micromamba` da `conda-forge` (con compromesso `D-92` che risolviamo al 100% nel [`Documento 04`](file:///c:/Users/nabre/Projects/atollo/reports/v3/04-PORTABILITA-100-E-MULTIPIATTAFORMA.md)). |
| **BUG-23b** | Alta | ✅ **MITIGATO (Opz. C)** | `nginx` limitato a Windows in `components.json:127`; nel [`Documento 04 §4`](file:///c:/Users/nabre/Projects/atollo/reports/v3/04-PORTABILITA-100-E-MULTIPIATTAFORMA.md) forniamo l'installer Unix completo via `conda-forge`. |
| **BUG-24a** | Alta | ✅ **MITIGATO (Opz. C)** | `php`, `apache`, `adminer` limitati a Windows in `components.json`; nel [`Documento 04 §4`](file:///c:/Users/nabre/Projects/atollo/reports/v3/04-PORTABILITA-100-E-MULTIPIATTAFORMA.md) forniamo l'installer Unix `static-php-cli` + `conda-forge`. |
| **BUG-24b** | Media | ✅ **CHIUSO** | [`setup/catalog.rs:188-221`](file:///c:/Users/nabre/Projects/atollo/tauri/src/setup/catalog.rs#L188-L221) (`translate_rel`) converte `.exe`, `.cmd`, `.bat` e `bin/windows/tlmgr.bat` per tutti i 6 target. |
| **BUG-24c** | Media | ✅ **CHIUSO** | [`install/mod.rs:92-97`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/mod.rs#L92-L97) (`target_of`) usa `component.effective_target(ctx.kit.target())` per scaricare `win32-x64` su `win32-arm64` quando il componente è x64-only. |
| **N-U1** | Media | ✅ **CHIUSO** | [`versions/mod.rs:168-196`](file:///c:/Users/nabre/Projects/atollo/tauri/src/versions/mod.rs#L168-L196) (`has_key_for` + `target_from_path`) verifica il binario chiave corretto per il target. |
| **BUG-18r** | Bassa | ✅ **CHIUSO** | [`projects/alias.rs`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/alias.rs) accetta sia `a.localhost` sia `a.b.localhost` in `valid_domain`. |
| **BUG-11r** | Bassa | ✅ **CHIUSO** | [`src/components/Sidebar.tsx`](file:///c:/Users/nabre/Projects/atollo/src/components/Sidebar.tsx) implementa navigazione frecce + Home/End. |
| **BUG-20r** | Bassa | ✅ **CHIUSO** | [`src/components/ConfirmModal.tsx`](file:///c:/Users/nabre/Projects/atollo/src/components/ConfirmModal.tsx) implementa focus trap e restore focus. |
| **UI-POLISH** | Bassa | ✅ **CHIUSO** | [`src/components/SetupModal.tsx`](file:///c:/Users/nabre/Projects/atollo/src/components/SetupModal.tsx) offre `Riprova` sui componenti falliti e progress bar determinata. |
| **ARCH-LAZY** | Bassa | ✅ **CHIUSO** | [`src/App.tsx`](file:///c:/Users/nabre/Projects/atollo/src/App.tsx) carica `GuidePage` con `React.lazy`. |
| **N-A1** | Strategica | ⏳ **IN CORSO (`v3`)** | `Cargo.toml` e `tauri/src/os/*.rs` sono ancora Windows-only; il codice completo per chiudere `N-A1` è fornito nei Documenti [`04`](file:///c:/Users/nabre/Projects/atollo/reports/v3/04-PORTABILITA-100-E-MULTIPIATTAFORMA.md) e [`07`](file:///c:/Users/nabre/Projects/atollo/reports/v3/07-PIANO-ESECUTIVO-PORTABILITA-E-MULTIPIATTAFORMA.md). |

---

## 2. Vulnerabilità di Sicurezza Residue a `HEAD 39b74bc` e Patch Concrete (`SEC-V3-01..03`)

### `SEC-V3-01` (Media) — Esposizione Sorgenti `.phtml`, `.php5`, `.php7`, `.phar`, `.inc` quando PHP non è installato
- **Evidenza nel codice**:
  - [`tauri/src/projects/apache.rs:114, 123`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/apache.rs#L114-L125): `<FilesMatch "(?i)\.php$">` intercetta solo l'estensione esatta `.php`.
  - [`tauri/src/projects/nginx.rs:148, 161`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/nginx.rs#L148-L163): `location ~* \.php$` intercetta solo `.php`.
- **Rischio**: Progetti legacy o framework (Magento, Zend/Laminas, template `.phtml`, archivi `.phar`, include `.inc`) contengono codice PHP e credenziali in file `.phtml`, `.php5`, `.phar` o `.inc`. Senza un match esteso, Apache e Nginx li servono come testo in chiaro (`text/plain` o download binario).
- **Soluzione Concreta (Patch Rust)**:
  In [`tauri/src/projects/apache.rs:114, 123`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/apache.rs#L114-L125):
  ```rust
  // Esecuzione PHP (quando il modulo è caricato): .php, .phtml, .phar
  "    <FilesMatch \"(?i)\\.(?:php[34578]?|phtml|phar)$\">".to_owned(),
  "        SetHandler application/x-httpd-php".to_owned(),
  "    </FilesMatch>".to_owned(),
  "    <FilesMatch \"(?i)\\.inc$\">".to_owned(),
  "        Require all denied".to_owned(),
  "    </FilesMatch>".to_owned(),
  ```
  E nel blocco `<IfModule !php_module>` (quando PHP non è installato) e in [`projects/nginx.rs:161`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/nginx.rs#L161):
  ```rust
  // Apache fallback (no PHP):
  "            <FilesMatch \"(?i)\\.(?:php[34578]?|phtml|phar|inc)$\">".to_owned(),
  "                Require all denied".to_owned(),
  "            </FilesMatch>".to_owned(),
  // Nginx fallback (no fcgi_port):
  "        location ~* \\.(?:php[34578]?|phtml|phar|inc)$ {".to_owned(),
  "            return 503;".to_owned(),
  "        }".to_owned(),
  ```

---

### `SEC-V3-02` (Media) — Mancato Blocco di `.env` / `.git` nel Sito Default del Kit (`data/apache/kit.conf`) e nel Built-in Server `php -S`
- **Evidenza nel codice**:
  - [`tauri/src/install/web.rs:845-895`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/web.rs#L845-L895) (`apache_site`): genera `data/apache/kit.conf` (porta 8080) usando `php_module_blocks`, ma **non** include `apache::hidden_deny("")`. Se l'utente clona un repository in `data/apache/htdocs/` o vi crea un `.env`, esso è scaricabile via `http://localhost:8080/.env`.
  - [`tauri/src/detect/mod.rs`](file:///c:/Users/nabre/Projects/atollo/tauri/src/detect/mod.rs): quando un progetto PHP viene avviato con il server built-in `php -S 127.0.0.1:{port}` (quando né Apache né Nginx sono usati), il server integrato di PHP serve qualunque file statico esistente su disco incluso `/.env` e `/.git/config`!
- **Soluzione Concreta**:
  1. In [`tauri/src/install/web.rs`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/web.rs) (`apache_site`) e in [`projects/apache.rs:migrate_kit_conf`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/apache.rs#L69-L96), aggiungere le direttive di `crate::projects::apache::hidden_deny("")`.
  2. Per `php -S`, fornire in `data/kit/router.php` uno script router di sicurezza di 12 righe che restituisce `403 Forbidden` se `preg_match('#/\.(?!well-known(/|$))#', $_SERVER['REQUEST_URI'])` o se l'estensione è `.inc`, altrimenti ritorna `false` lasciando a `php -S` il file serving nativo!

---

### `SEC-V3-03` (Bassa/Media) — DNS Rebinding sui Server Apache e Nginx Dedicati per Singolo Progetto
- **Evidenza nel codice**:
  - Lo Sciame ha protetto contro il DNS Rebinding i **pool condivisi** ([`servers/mod.rs:354, 401`](file:///c:/Users/nabre/Projects/atollo/tauri/src/servers/mod.rs#L354)) aggiungendo un primo `<VirtualHost *:port>` / `default_server` neutro.
  - Tuttavia, nei server **dedicati** ([`projects/apache.rs:219-245`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/apache.rs#L219-L245) e [`projects/nginx.rs:188-199`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/nginx.rs#L188-L199)), il sito del progetto è ancora il server principale / `default_server: true`. Un sito malevolo visitato nel browser dell'utente può eseguire DNS Rebinding verso `127.0.0.1:<port>` con header `Host: attacker.com` e leggere le risposte del progetto!
- **Soluzione Concreta**:
  In [`projects/nginx.rs:145`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/nginx.rs#L145) (quando `!lan`) e in [`projects/apache.rs:233`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/apache.rs#L233), validare l'header `Host` accettando solo `localhost`, `127.0.0.1`, `[::1]` e `*.localhost` (o i `domains` configurati):
  ```nginx
  # In projects/nginx.rs (quando !lan):
  if ($host !~* ^(?:localhost|127\.0\.0\.1|\[::1\]|.+\.localhost)$) {
      return 421;
  }
  ```

---

## 3. Bug Logici, Funzionali e Multipiattaforma a `HEAD 39b74bc` (`BUG-V3-01..09`)

### `BUG-V3-01` (Media) — Mancata Supervisione di `php-cgi` nel Server Nginx Dedicato (`projects/run.rs:515`)
- **File e Righe**: [`tauri/src/projects/run.rs:515-516`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/run.rs#L515-L516).
- **Causa**:
  ```rust
  let fcgi_port = crate::projects::nginx::spawn_php_cgi_if_present(&self.kit, env, &job)
      .map(|(port, _php)| port);
  ```
  L'handle `_php: Child` viene droppato immediatamente. Su Rust stdlib, il `Drop` di `std::process::Child` non uccide il processo (che resta vivo dentro il `Job`), ma nessuno ne controlla l'uscita imprevista. Se `php-cgi.exe` muore per un fatal error, Nginx continua a girare e risponde `502 Bad Gateway` su tutte le rotte PHP mentre l'interfaccia mostra il semaforo verde `Running`.
- **Patch Rust Concreta**:
  Conservare `mut php_child: Option<Child>` e nel thread di monitoraggio ([`projects/run.rs:659-665`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/run.rs#L659-L665)) controllare in parallelo sia `child` (Nginx) sia `php_child` (`php-cgi`):
  ```rust
  // In projects/run.rs:
  let mut php_child: Option<Child> = None;
  // ... nel ramo action.nginx_dir:
  let fcgi_port = crate::projects::nginx::spawn_php_cgi_if_present(&self.kit, env, &job)
      .map(|(port, php)| {
          php_child = Some(php);
          port
      });
  // ... nel thread di attesa (riga 659):
  std::thread::spawn(move || {
      let code = loop {
          if let Ok(Some(status)) = child.try_wait() {
              break status.code();
          }
          if let Some(php) = php_child.as_mut() {
              if let Ok(Some(_)) = php.try_wait() {
                  let _ = child.kill();
                  break child.wait().ok().and_then(|s| s.code());
              }
          }
          std::thread::sleep(Duration::from_millis(250));
      };
      for r in readers {
          let _ = r.join();
      }
      me.finish(&key2, code, &lang2);
  });
  ```

---

### `BUG-V3-02` (Media) — Collisione Nome File `vhosts/<project>.conf` con Più Azioni Web dello Stesso Progetto (`servers/mod.rs:924, 954`)
- **File e Righe**: [`tauri/src/servers/mod.rs:924`](file:///c:/Users/nabre/Projects/atollo/tauri/src/servers/mod.rs#L924) e [`954`](file:///c:/Users/nabre/Projects/atollo/tauri/src/servers/mod.rs#L954).
- **Causa**: `write_pool_configs` scrive `vhosts_dir.join(format!("{}.conf", site.project))`. Se un progetto ha due azioni web nello stesso pool (es. `site-a` e `site-b`), il secondo file sovrascrive il primo.
- **Patch Rust Concreta**:
  ```rust
  fn vhost_file_name(site: &AttachedSite) -> String {
      let act = slug(&site.action);
      if act.is_empty() || act == "web" || act == "site" {
          format!("{}.conf", site.project)
      } else {
          format!("{}--{act}.conf", site.project)
      }
  }
  ```

---

### `BUG-V3-03` (Critica Multi-OS) — `projects/run.rs:535-537` Hardcoda `C:\Windows\System32\cmd.exe`
- **File e Righe**: [`tauri/src/projects/run.rs:20, 535-537`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/run.rs#L535-L537).
- **Causa**: Importa `std::os::windows::process::CommandExt` senza `#[cfg(windows)]` e avvia tutti i comandi dei progetti invocando `C:\Windows\System32\cmd.exe /d /s /c`.
- **Patch Rust Concreta**: Usare `jobs::shell_command(&command, env)` (definito nel [`Documento 01 §3.1`](file:///c:/Users/nabre/Projects/atollo/reports/v3/01-ARCHITETTURA-E-LOGICA.md)) che usa `cmd.exe /d /s /c` su Windows e `/bin/sh -c` su macOS e Linux.

---

### `BUG-V3-04` — Ritirato (18/10/2026)
- Riguardava le build del server SQL su macOS e Linux. Il server SQL del kit è MySQL, con gli archivi ufficiali di Oracle per i sei target e gli SHA-256 fissati nel catalogo (`D-110`): il problema non esiste più.

---

### `BUG-V3-05` (Media Multi-OS) — `Cc` (`llvm/llvm-project`) su macOS x64 e Linux: Nomi Asset Inconsistenti e Assenza di `make` + `sysroot` C/C++
- **File e Righe**: [`tauri/src/install/lang/cc.rs:110-166`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/lang/cc.rs#L110-L166).
- **Causa**:
  1. Nelle release recenti di `llvm/llvm-project` (LLVM 19 / 20 / 21), gli asset macOS x86_64 (`macOS-X64`) spesso **non vengono pubblicati** nella release `latest` (viene pubblicato solo `LLVM-X.Y.Z-macOS-ARM64.tar.xz`), facendo fallire `Cc::latest` su `darwin-x64`.
  2. Inoltre, l'archivio `LLVM-*.tar.xz` di `llvm/llvm-project` su Linux e macOS contiene il compilatore `clang`/`clang++`, ma **non include né `make`, né gli header C standard (`stdio.h`, `stdlib.h` di `glibc`/`libSystem`), né `crt1.o`/`libc.so`**! Se sul Mac non sono installati gli Xcode Command Line Tools o sulla distro Linux non c'è `libc6-dev`/`glibc-devel` + `make`, `clang hello.c` fallisce con `fatal error: 'stdio.h' file not found`!
- **Soluzione Concreta Promossa (100% Self-Contained su Tutti gli OS)**:
  Su macOS e Linux, installare il componente `cc` tramite `micromamba` da `conda-forge` con il metapacchetto `c-compiler`, `cxx-compiler` e `make` (che su Linux installa `gcc`/`g++`/`clang`, `binutils`, `make` e **`sysroot_linux-64` con `glibc` + header completi `stdio.h`**, e su macOS installa `clang`, `cctools`, `ld64`, `make` e l'SDK macOS `MacOSX.sdk`!). In questo modo il compilatore C/C++ del kit compila codice C/C++ e crate Rust nativi **anche su un Mac senza Xcode CLT e su una distro Linux minimale senza `build-essential`**!

---

### `BUG-V3-06` (Media Portabilità) — `R` su Windows Non Riscrive `Makeconf` al Cambio Lettera d'Unità, e `git`/`r` su Unix Richiedono Internet Dopo Ogni Spostamento (`D-92`)
- **File e Righe**: [`tauri/src/kit.rs:515-545`](file:///c:/Users/nabre/Projects/atollo/tauri/src/kit.rs#L515-L545), [`tauri/src/install/core.rs:35-37`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/core.rs#L35-L37), [`tauri/src/install/native.rs:677-679`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/native.rs#L677-L679).
- **Causa**:
  - Su macOS e Linux, `Git::relocatable` e `R::relocatable` restituiscono `false`. Spostare la cartella del kit anche solo da `/Users/alice/Downloads/Atollo` a `/Users/alice/Atollo` invalida `git` e `r` e richiede di riscaricare ~1,2 GB da Internet.
  - Su Windows, `R::relocatable` restituisce `true`, ma `lib/R/etc/x64/Makeconf` conserva il vecchio percorso assoluto generato da `micromamba`: quando la chiavetta USB passa da `D:\` a `E:\`, `install.packages(..., type="source")` fallisce cercando i compilatori e le librerie in `D:\`.
- **Soluzione Concreta**: Rilocazione offline automatica `relocate_conda_prefix` + wrapper relativi descritti con codice completo nel [`Documento 04 §2`](file:///c:/Users/nabre/Projects/atollo/reports/v3/04-PORTABILITA-100-E-MULTIPIATTAFORMA.md).

---

### `BUG-V3-07` (Media Multi-OS) — `os/sessions.rs:157-166` (`other_kits`) Cerca Solo Percorsi Windows `\\data\\apps\\win32-` e `Atollo.exe`
- **File e Righe**: [`tauri/src/os/sessions.rs:151-174`](file:///c:/Users/nabre/Projects/atollo/tauri/src/os/sessions.rs#L151-L174).
- **Causa**: `other_kits` individua altri kit in esecuzione cercando `lower.find("\\data\\apps\\win32-")` e `p.name.eq_ignore_ascii_case("Atollo.exe")`. Su macOS e Linux i separatori sono `/` e i target sono `darwin-*` o `linux-*`.
- **Patch Rust Concreta**:
  ```rust
  pub fn other_kits(kit: &Kit) -> Vec<PathBuf> {
      let norm = |p: &Path| p.display().to_string().replace('\\', "/").to_lowercase();
      let mine = norm(kit.root());
      let mut out = BTreeSet::new();
      for p in processes::all() {
          let path_str = p.path.display().to_string();
          let clean = path_str.replace('\\', "/");
          let lower = clean.to_lowercase();
          let root = ["/data/apps/win32-", "/data/apps/darwin-", "/data/apps/linux-"]
              .iter()
              .find_map(|marker| lower.find(marker).map(|i| PathBuf::from(&path_str[..i])))
              .or_else(|| {
                  let is_app = p.name.eq_ignore_ascii_case("Atollo.exe")
                      || p.name.eq_ignore_ascii_case("Atollo");
                  is_app
                      .then(|| p.path.parent())
                      .flatten()
                      .filter(|d| d.join("data").join("app").is_dir())
                      .map(Path::to_path_buf)
              });
          if let Some(root) = root {
              if norm(&root) != mine {
                  out.insert(root);
              }
          }
      }
      out.into_iter().collect()
  }
  ```

---

### `BUG-V3-08` (Bassa/Media Multi-OS) — `projects/apache.rs` e `projects/nginx.rs` Cercano `bin\httpd.exe`, `nginx.exe` e `php-cgi.exe` Senza Traduzione Unix
- **File e Righe**: [`tauri/src/projects/apache.rs:257-290`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/apache.rs#L257-L290), [`tauri/src/projects/nginx.rs:20-43, 230-278`](file:///c:/Users/nabre/Projects/atollo/tauri/src/projects/nginx.rs#L20-L43), [`tauri/src/servers/mod.rs:1012, 1025, 1152`](file:///c:/Users/nabre/Projects/atollo/tauri/src/servers/mod.rs#L1012-L1025).
- **Causa**: Tutte le funzioni che risolvono gli eseguibili di Apache, Nginx e PHP-CGI controllano `d.join(r"bin\httpd.exe")`, `d.join("nginx.exe")` e `d.join("php-cgi.exe")`. Quando si abilitano `php`, `apache` e `nginx` su macOS e Linux, i binari sono rispettivamente `bin/httpd`, `sbin/nginx` (o `bin/nginx`) e `bin/php-cgi`.
- **Soluzione Concreta**: Funzioni helper `httpd_exe(&apache_dir, kit.os())`, `nginx_exe(&nginx_dir, kit.os())` e `php_cgi_exe(&php_dir, kit.os())` fornite nel [`Documento 04 §4.3`](file:///c:/Users/nabre/Projects/atollo/reports/v3/04-PORTABILITA-100-E-MULTIPIATTAFORMA.md).

---

### `BUG-V3-09` (Bassa) — `vscodium-server` (`components.json:39`) Non Pre-Popola `sha256ByTarget` per i 5 Target Non-`win32-x64`
- **File e Righe**: [`tauri/catalog/components.json:35-41`](file:///c:/Users/nabre/Projects/atollo/tauri/catalog/components.json#L35-L41) e [`tauri/src/install/editor.rs:87-102`](file:///c:/Users/nabre/Projects/atollo/tauri/src/install/editor.rs#L87-L102).
- **Analisi**: In `install/editor.rs:89-101`, lo Sciame ha aggiunto un ottimo fallback che scarica `{url}.sha256` da GitHub quando `component.sha256_for(target)` è `None`. Tuttavia, in `components.json:39` è presente `"sha256": "be8c80fad63268255ba8ca8a9ebbd54eb39dc4b13c4fe2f963700e898a220420"` (che `sha256_for` usa solo per `Target::WIN32_X64`), mentre `sha256ByTarget` è omesso. Pre-popolare `sha256ByTarget` in `components.json` per tutti i 6 target della release `1.112.01907` evita una richiesta HTTP extra e garantisce il pinning crittografico offline/immutabile (`D-58`) su tutte le piattaforme.

---

## 4. Stato di esecuzione

> Ramo `fix/v3-swarm`, da `59da30a` a `f03d0a5`. Sette pacchetti in parallelo (websec, pools, install, db, detect, ui, multios), uniti con `--no-ff` senza conflitti (`1ca0c42`…`c7412b5`, poi `a9eb3c9`), e un giro di revisione con 17 rilievi, tutti corretti (`3262f7a`…`f03d0a5`). Gate finali su Windows: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (426 passati, 3 ignorati), `npm -w ui test` (107), `lint`, `format:check`, `build`. Decisioni in [DECISIONS.md](../../app/DECISIONS.md) D-96…D-106, novità in [CHANGELOG.md](../../app/CHANGELOG.md).

| # | ID | Stato | Commit di chiusura |
| :-: | :-- | :-: | :-- |
| 1 | SEC-V3-01 | ✅ chiuso | `62a019d` fix(web): run .phtml/.phar as PHP, never serve .inc or .php3-8; `3262f7a` fix(router): SEC-V3-01 and SEC-V3-03 on php -S |
| 2 | SEC-V3-02 | ✅ chiuso | `79b2d27` fix(web): refuse hidden files in the kit's Apache site; `95be058` fix(detect): route php -S through the kit's router; `9901145` fix(detect): php -S router and project paths on Unix |
| 3 | SEC-V3-03 | ✅ chiuso | `49e3d02` fix(web): serve only local hosts on a project's own server; `3262f7a` (421 su `php -S`); `77f0490` fix(servers): 128-byte server name bucket in the Nginx pool |
| 4 | BUG-V3-01 | ✅ chiuso | `948a1f3` fix(run): stop a project's Nginx when its php-cgi ends (chiude anche BUG-05r di v2) |
| 5 | BUG-V3-02 | ✅ chiuso | `8b50e51` fix(servers): one ServerName and vhost file per project action |
| 6 | BUG-V3-03 | ✅ chiuso | `e48acda` fix(run): shell_command instead of a fixed cmd.exe |
| 7 | BUG-V3-04 | ➖ ritirato | il server SQL è MySQL, con build per i sei target (D-110) |
| 8 | BUG-V3-05 | ✅ chiuso | `706dfb5` fix(cc): newest LLVM release with the system's archive — ripiego su un rilascio più vecchio (macOS Intel su 20.1.7) invece di conda-forge; requisiti del PC nella descrizione |
| 9 | BUG-V3-06 | 🟡 parziale | `1412e9e` fix(r): R's Makeconf follows the kit on Windows — la parte Unix (rilocazione offline di Git e R dopo uno spostamento) non è fatta |
| 10 | BUG-V3-07 | ✅ chiuso | `cbcca8e` fix(sessions): other kits on macOS and Linux too |
| 11 | BUG-V3-08 | ✅ chiuso | `f159ad4` fix(servers): httpd, nginx and php-cgi per system |
| 12 | BUG-V3-09 | ✅ chiuso | `a677d14` fix(editor): pin the editor's SHA-256 on every target |
| 13 | N-A1 / B7 | 🟡 parziale | `dca57fd` build(cargo): Win32 dependencies only on Windows; `44b098e` feat(os): Unix implementations of the os layer; `20cec6b`; `c55e580` fix(keyring): gate every Windows API with a Unix answer; `17266ad` ci(app): cargo check on ubuntu-latest and macos-latest; `5751f38` fix(jobs): never SIGKILL a group whose leader pid was reused — compila per macOS e Linux, non provato su sistemi reali; PHP, Apache, Nginx e Adminer restano solo Windows |

Voci dei piani chiuse nello stesso ramo:

| Voce | Stato | Commit |
| :-- | :-: | :-- |
| PLAN §13.1 / PLAN-DATA 1.2c (sgancio, `keepAlive`, `autostart`) | 🟡 parziale | `2487dbe` feat(servers): pools follow their projects, keepAlive and autostart; `b964ff6` — resta l'aggancio al pool automatico per un sito senza `server` |
| PLAN §13.2 / PLAN-DATA 1.3d (più versioni di MySQL) | ✅ chiuso | `452acec`; dopo la revisione `0e5e0e8`, `d3c2389`, `f93adf4`, `ae9690b`, `3e7986f` |
| PLAN-DATA §9 difetto 4 (linea non installata) | ✅ chiuso | `c347a3a` |
| PLAN-DATA §9 difetto 5 (pool orfani) | 🟡 parziale | `d152706` — id stabili (SC-2) da fare |
| PLAN-DATA §9 difetto 6 (porte) | ✅ chiuso | `699e823`, `9aec654` |
| PLAN-DATA §9 difetto 7 (agganci doppi) | ✅ chiuso | `f2864f3` |
| PLAN-DATA §9 difetto 8 (`assign_project`) | ✅ chiuso | `bacc046` |
| PLAN-DATA §9 difetto 9 (php-cgi del pool) | ✅ chiuso | `32a7d9d`, `624f4d0` |
| PLAN-DATA §9 difetto 10 (`php.ini` per linea) | ⏳ aperto | — |
| Prove BUG-03/BUG-04 con kit reale | ✅ scritte | `9df0c9f` — saltate senza `ATOLLO_TEST_KIT`, mai eseguite su un httpd vero |
| WEB-8 (alias senza collegamenti) | ✅ chiuso | `4c9d901` |
| Rilevamento (CMS, Laravel, `nginx.conf`, Compose, PHP semplice) | ✅ chiuso | `a812a6c`, `89bc2b2`, `8a09983`, `49d747b`, `55b9e13`, `4522b97` |
| Versioni del progetto in due zone (PLAN-DATA §9 riga 4) | ✅ chiuso | `50cba6d` |
| Focus dei dialoghi (v3/02 §2.3) e quattro conferme `alertdialog` | ✅ chiuso | `384d6f4`, `98421ed`, `475ad2c`, `f03d0a5` |
| B6 (temporanei con «Nel kit») | ➖ non un bug | voluto da D-90 |
| R su Windows ARM64 | ➖ non fatto | R è `noArm64`; servirebbe rifare l'installatore |

Totale per questo documento: **10 chiusi, 1 ritirato, 2 parziali** (BUG-V3-06 per la parte Unix, N-A1 in attesa di prova su macOS e Linux reali).

### 4.1 Seguiti non fatti

| Voce | Dove | Nota |
| :-- | :-- | :-- |
| php-cgi senza autenticazione | `projects/nginx.rs`, `servers/mod.rs` | `-b 127.0.0.1:<porta>`: un processo locale (o una SSRF locale) esegue PHP arbitrario; named pipe o `FCGI_WEB_SERVER_ADDRS` |
| Nginx su Windows e collegamenti negli alias | `projects/nginx.rs` | `disable_symlinks` non esiste nella build win32; solo la cartella dell'alias è controllata |
| Pool: rifiuto degli `Host` estranei | `servers/mod.rs` | il server di default del pool serve la pagina neutra di httpd/nginx, non 403/421 come i server dedicati |
| Router del progetto | `detect/mod.rs` | un `router.php` del progetto sostituisce quello del kit senza avviso: i file nascosti non sono più bloccati |
| Aggancio al pool automatico | `projects/run.rs`, `servers/mod.rs` | un sito senza `server` in `panel.json` usa il server dedicato |
| Id stabili dei pool (SC-2) | `servers/mod.rs` | `apache-2.4_php-7.4` con migrazione delle porte; oggi l'orfano in elenco fa da ripiego |
| `php.ini` per linea di PHP (difetto 10) | `install/web.rs`, `env.rs`, `projects/nginx.rs` | un solo `php.ini` per tutte le versioni |
| Porte di pool spariti | `servers/mod.rs` | si tolgono solo con una lista nuova («Aggiorna»): una porta vecchia può bloccare un profilo |
| Interfaccia dei pool | `ui/src/servers.tsx`, `ui/src/projects/site.tsx` | siti sganciati non distinti; i testi annunciano `<progetto>.localhost` anche per le azioni secondarie (`<azione>.<progetto>.localhost`) |
| Pool con troppi siti | `servers/mod.rs` | Apache ora è proposto a ogni progetto PHP: i pool automatici elencano ogni progetto con un'azione Apache/Nginx (PLAN-DATA 1.1) |
| Versioni di MySQL: avvio automatico | `db/services` | parte solo la versione principale |
| Versioni di MySQL: strumenti | `env.rs`, phpMyAdmin | il `mysql` del terminale e phpMyAdmin vedono solo la versione principale |
| Versioni di MySQL: eliminazione | `db/auth.rs` | un progetto che ha cambiato versione perde il database solo sull'ultima |
| Versione disinstallata | `db/services` | avvio e creazione falliscono con «not installed»; l'interfaccia non lo segnala |
| Lock di `db.json` | `db/auth.rs` | per processo: un CLI insieme all'app potrebbe correre |
| `Project.lines` | `ui/src/api/projects.ts`, `ui/src/versions/project.tsx` | rilevato da Compose/Dockerfile, non usato dall'interfaccia |
| LLVM su macOS Intel | `install/lang/cc.rs` | quando 20.1.7 esce dagli ultimi 100 rilasci, `cc` su darwin-x64 fallisce: segnarlo non disponibile o fissare il rilascio |
| Editor aggiornato | `tauri/catalog/components.json` | a ogni nuova versione di `vscodium-server` rinnovare tutti i `sha256ByTarget` |
| Unix: prova reale | `os/*`, `os/jobs.rs`, `stats` | istanza (flock e socket), sweep del temp, `killpg`, `/proc` e libproc, `local_day_start` con l'ora legale, test `cfg(unix)` |
| Unix: ancora a forma di Windows | `env.rs`, conf di Apache/Nginx, `db/services`, `editor/ext` | `apache\bin`, `httpd.exe`, `mongod.exe`, `mysqld.exe`, `node.exe` senza `translate_rel` |
| CI macOS `--all-targets` | `tauri/Cargo.toml` | tauri 2.12.0 (feature `test`) contro tauri-runtime 2.12.1: allineare le versioni |
| Prove con kit reale | `test/rust/servers/` | `start_reports_syntax_error` e `apache_pool_reload_twice` da eseguire in CI con `ATOLLO_TEST_KIT` |
| Configurazioni vere | — | nessun `httpd -t`/`nginx -t` sulle configurazioni nuove: httpd e nginx non erano installati (prove in TESTING.md) |
